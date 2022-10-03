use jfa_cpu::MatrixJfa;
use retro_blit::rendering::blittable::{BufferProvider, BufferProviderMut};
use retro_blit::rendering::BlittableSurface;
use retro_blit::utility::StopWatch;
use retro_blit::window::{RetroBlitContext, ContextHandler, WindowMode};

const WANG_MASK_BYTES: &[u8] = include_bytes!("jfa_test_image.im256");
const VORONOI_DOTS_BYTES: &[u8] = include_bytes!("voronoi_dots.im256");

struct App {
    jfa: MatrixJfa,
    wang_mask: BlittableSurface,
    voronoi_dots: BlittableSurface,
}
impl ContextHandler for App {
    fn get_window_title(&self) -> &'static str {
        "jump_flood_algorithm_test"
    }

    fn get_window_mode(&self) -> WindowMode {
        WindowMode::Mode256x256
    }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        for idx in 0..=255 {
            ctx.set_palette(idx, [idx, idx, idx]);
        }
    }

    fn update(&mut self, ctx: &mut RetroBlitContext, _dt: f32) {
        ctx.clear(0);
        {
            let buffer = ctx.get_buffer_mut();
            let _sw = StopWatch::named("jfa");

            let voronoi = {
                let voronoi_buffer = self.voronoi_dots.get_buffer();
                let voronoi_jfa = self.jfa.calc::<64, 64>(
                    (0..4096)
                        .into_iter()
                        .filter_map(|idx| {
                            let (i, j) = (idx % 64, idx / 64);
                            let col = unsafe { voronoi_buffer.get_unchecked(idx) };
                            if *col == 0 {
                                None
                            } else {
                                Some((i, j))
                            }
                        })
                );
                voronoi_jfa
                    .iter()
                    .enumerate()
                    .map(|(idx, &nearest_coord)| {
                        let x = idx % 64;
                        let y = idx / 64;
                        let dx = x as f32 - nearest_coord.0 as f32;
                        let dy = y as f32 - nearest_coord.1 as f32;
                        1.0 - ((dx * dx + dy * dy) / 128.0).clamp(0.0, 1.0)
                    })
                    .collect::<Vec<_>>()
            };

            let wang_buffer = self.wang_mask.get_buffer();

            for y in 0..4 {
                for x in 0..4 {
                    let start_i = x * 64;
                    let start_j = y * 64;
                    let jfa = self.jfa.calc::<64, 64>(
                        (0..4096)
                            .into_iter()
                            .filter_map(|idx| {
                                let (i, j) = (idx % 64, idx / 64);
                                let idx = start_i + i + (start_j + j) * 256;
                                let col = unsafe { wang_buffer.get_unchecked(idx) };
                                if *col == 0 {
                                    None
                                } else {
                                    Some((i, j))
                                }
                            })
                    );

                    for (idx, &nearest_coord) in jfa.iter().enumerate() {
                        let x = idx % 64;
                        let y = idx / 64;
                        let dx = x as f32 - nearest_coord.0 as f32;
                        let dy = y as f32 - nearest_coord.1 as f32;
                        let distance = (dx * dx + dy * dy).sqrt();
                        let distance = smooth_step(0.0, 12.0, distance);
                        let distance_mask = 1.0 - distance;
                        let noise = voronoi[idx];
                        let terrain_height = 0.3 + 0.1 * noise.powf(0.5) as f32;
                        let final_height = (terrain_height + distance_mask).clamp(0.0, 1.0);

                        unsafe {
                            let idx = start_i + x + (start_j + y) * 256;
                            *buffer.get_unchecked_mut(idx) = (final_height * 255.0).clamp(0.0, 255.0) as u8;
                        }
                    }
                }
            }
        }
    }
}

fn smooth_step(edge_0: f32, edge_1: f32, x: f32) -> f32 {
    match () {
        _ if x < edge_0 => 0.0,
        _ if x >= edge_1 => 1.0,
        _ => {
            let x = (x - edge_0) / (edge_1 - edge_0);
            x * x * (3.0 - 2.0 * x)
        }.clamp(0.0, 1.0)
    }
}

fn main() {
    let (_, wang_mask) = retro_blit::format_loaders::im_256::Image
        ::load_from(WANG_MASK_BYTES)
        .unwrap();

    let (_, voronoi_dots) = retro_blit::format_loaders::im_256::Image
    ::load_from(VORONOI_DOTS_BYTES)
        .unwrap();

    retro_blit::window::start(App{
        wang_mask,
        voronoi_dots,
        jfa: MatrixJfa::new()
    })
}