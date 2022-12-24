use std::path::{PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;
use notify::{RecursiveMode, Watcher, RecommendedWatcher, DebouncedEvent};
use retro_blit::rendering::bresenham::{BresenhamCircleDrawer, LineRasterizer};
use retro_blit::rendering::fonts::font_align::{HorizontalAlignment, VerticalAlignment};
use retro_blit::rendering::fonts::tri_spaced::{Font, TextDrawer};
use retro_blit::window::{RetroBlitContext, WindowMode};
use crate::map_data::MapData;

mod map_data;

struct App {
    ron_path: PathBuf,
    _watcher: RecommendedWatcher,
    map_data: MapData,
    font: Font,
    file_buffer: Vec<u8>,
    rx: mpsc::Receiver<DebouncedEvent>
}

impl retro_blit::window::ContextHandler for App {
    fn get_window_title(&self) -> &'static str {
        "portal map viewer"
    }

    fn get_window_mode(&self) -> WindowMode {
        WindowMode::ModeXFrameless
    }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        for id in 0..=255 {
            ctx.set_palette(id, [id, id, id]);
        }
        ctx.set_palette(255, [100, 100, 255])
    }

    fn update(&mut self, ctx: &mut RetroBlitContext, _dt: f32) {
        self.poll_file_changes();
        ctx.clear(0);

        for region in self.map_data.regions.iter() {
            let color_id = ((region.height * 4.0) as u8).clamp(0, 254);

            let n = region.walls.len();

            let (mut a, mut cx, mut cy) = (0..n)
                .fold((0.0, 0.0, 0.0), |acc, i| {
                    let i_next = (i + 1) % n;
                    let x0 = self.map_data.points[region.walls[i]].0;
                    let x1 = self.map_data.points[region.walls[i_next]].0;
                    let y0 = self.map_data.points[region.walls[i]].1;
                    let y1 = self.map_data.points[region.walls[i_next]].1;
                    let t = x0 * y1 - y0 * x1;
                    (
                        acc.0 + t,
                        acc.1 + (x0 + x1) * t,
                        acc.2 + (y0 + y1) * t
                    )
                });

            a /= 2.0;
            cx /= 6.0 * a;
            cy /= 6.0 * a;

            for i in 0..n {
                let point = self.map_data.points[region.walls[i]];

                let pp = (
                    (point.0 * self.map_data.pixels_per_meter as f32) as i16,
                    240 - (point.1 * self.map_data.pixels_per_meter as f32) as i16
                );

                self.font.draw_text_in_box(
                    ctx,
                    pp.0 - 5, pp.1 - 13,
                    10, 10,
                    HorizontalAlignment::Left,
                    VerticalAlignment::Center,
                    &format!("{}", region.walls[i]),
                    Some(color_id)
                );

                BresenhamCircleDrawer::create(ctx)
                    .with_position(pp)
                    .with_radius(3)
                    .draw(color_id);
            }

            let centroid_p = (
                (cx * self.map_data.pixels_per_meter as f32) as i16,
                240 - (cy * self.map_data.pixels_per_meter as f32) as i16
            );

            for i in 0..n {
                let next_i = (i + 1) % n;
                let point_0 = self.map_data.points[region.walls[i]];
                let point_1 = self.map_data.points[region.walls[next_i]];

                let point_center = (
                    (point_0.0 + point_1.0) / 2.0,
                    (point_0.1 + point_1.1) / 2.0
                );

                let normal = glam::Mat2::from_angle(-std::f32::consts::FRAC_PI_2) *
                    glam::vec2(
                        point_1.0 - point_0.0,
                        point_1.1 - point_0.1
                    ).normalize_or_zero();

                let a = (
                    (point_0.0 * self.map_data.pixels_per_meter as f32) as i16,
                    240 - (point_0.1 * self.map_data.pixels_per_meter as f32) as i16
                );
                let b = (
                    (point_1.0 * self.map_data.pixels_per_meter as f32) as i16,
                    240 - (point_1.1 * self.map_data.pixels_per_meter as f32) as i16
                );

                let c = (
                    (point_center.0 * self.map_data.pixels_per_meter as f32) as i16,
                    240 - (point_center.1 * self.map_data.pixels_per_meter as f32) as i16
                );
                let d = (
                    (point_center.0 * self.map_data.pixels_per_meter as f32 + (normal.x * 4.0)) as i16,
                    240 - ((point_center.1 * self.map_data.pixels_per_meter as f32 + (normal.y * 4.0)) as i16)
                );

                let color_id = if region.portals.contains_key(&i) {
                    255
                } else {
                    color_id
                };

                LineRasterizer::create(ctx)
                    .from(a)
                    .to(b)
                    .rasterize(color_id);

                LineRasterizer::create(ctx)
                    .from(c)
                    .to(d)
                    .rasterize(color_id);
            }

            self.font.draw_text_in_box(
                ctx,
                centroid_p.0 - 150, centroid_p.1 - 150,
                300, 300,
                HorizontalAlignment::Center,
                VerticalAlignment::Center,
                &format!("{}", region.height),
                Some(color_id)
            );
        }
    }
}

impl App {
    fn poll_file_changes(&mut self) {
        loop {
            match self.rx.try_recv() {
                Ok(DebouncedEvent::Write(_)) => {
                    println!("Change detected!");
                    self.map_data = MapData::read_from_path(&self.ron_path, &mut self.file_buffer);
                },
                Ok(_) => (),
                Err(TryRecvError::Disconnected) => {
                    println!("Disconnected!");
                    break;
                },
                _ => {
                    break;
                }
            }
        }
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();

    if args.len() < 2 {
        println!("usage: portal-map-viewer path_to_map.ron");
        return;
    }

    let ron_file_name = args[1].clone();

    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Duration::from_millis(100))
        .unwrap();

    let ron_path: PathBuf = (&ron_file_name).into();

    let mut file_buffer = Vec::new();

    let map_data = MapData::read_from_path(&ron_path, &mut file_buffer);

    watcher
        .watch(&ron_path, RecursiveMode::NonRecursive)
        .unwrap();

    let font = Font::default_font_small().unwrap();

    let app = App {
        ron_path,
        _watcher: watcher,
        map_data,
        font,
        file_buffer,
        rx
    };

    retro_blit::window::start(app);

}
