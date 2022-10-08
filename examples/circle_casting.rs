use retro_blit::math_utils::collision_queries::{SegmentCircleCastQuery};
use retro_blit::rendering::bresenham::{BresenhamCircleDrawer, LineRasterizer};
use retro_blit::window::{RetroBlitContext, ContextHandler, WindowMode, KeyCode};

const SKIN: f32 = 2.5;
const MINIMAL_DISTANCE: f32 = 0.001;
const TURN_SPEED: f32 = 80.0;
const MOVEMENT_SPEED: f32 = 80.0;
const RADIUS: f32 = 32.0;

struct App {
    angle: f32,
    pos: glam::Vec2,
    segments: Vec<[glam::Vec2; 2]>
}

impl App {
    fn move_towards(&mut self, ctx: &mut RetroBlitContext, dir: glam::Vec2) {
        let mut current_dir = dir;
        let mut distance_to_go = dir.length();
        current_dir /= distance_to_go;
        let mut current_pos = self.pos;

        for _ in 0..4 {
            if distance_to_go < MINIMAL_DISTANCE {
                break;
            }

            distance_to_go = match self.cast_circle(current_pos, current_dir, RADIUS) {
                None =>  {
                    current_pos = current_pos + current_dir;
                    0.0
                },
                Some((t, normal)) => {
                    let contact_p = current_pos + current_dir * t;
                    let pos_plus_normal = contact_p + 16.0 * normal;
                    LineRasterizer::create(ctx)
                        .from((contact_p.x as _, contact_p.y as _))
                        .to((pos_plus_normal.x as _, pos_plus_normal.y as _))
                        .rasterize(0b0110_0000);

                    if t >= distance_to_go {
                        current_pos = current_pos + current_dir;
                        0.0
                    } else {
                        let direct_distance = t;
                        let rest_distance = distance_to_go - direct_distance;
                        let dir = current_dir * direct_distance;
                        current_pos += dir;

                        let dir_rest = current_dir * rest_distance;
                        let norm_proj = normal.normalize_or_zero() *
                            dir_rest.dot(normal.normalize_or_zero());

                        current_dir = dir_rest - norm_proj;
                        let d = current_dir.length();
                        current_dir = current_dir / d;
                        d
                    }
                }
            }
        }
        self.pos = current_pos;
    }
}

impl ContextHandler for App {
    fn get_window_title(&self) -> &'static str {
        "circle cast character controller playground"
    }

    fn get_window_mode(&self) -> WindowMode {
        WindowMode::ModeXFrameless
    }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        let mut idx = 0;
        for j in 0..16 {
            for i in 0..16 {
                let red = 255.0 * (i as f32) / 15.0;
                let green = 255.0 * (j as f32) / 15.0;
                let grayscale = (red + green) / 2.0;

                ctx.set_palette(idx, [
                    ((red + grayscale) / 2.0) as _,
                    ((green + grayscale) / 2.0) as _,
                    (grayscale / 2.0) as _
                ]);
                if idx < 255 {
                    idx += 1;
                }
            }
        }
    }

    fn update(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        ctx.clear(0);
        match (ctx.is_key_pressed(KeyCode::Left), ctx.is_key_pressed(KeyCode::Right)) {
            (true, false) => {
                self.angle += TURN_SPEED * dt;
            },
            (false, true) => {
                self.angle -= TURN_SPEED * dt;
            },
            _ => ()
        }

        let angle = (self.angle + 90.0).to_radians();
        let ray_dir = glam::vec2(angle.cos(), -angle.sin());

        match (ctx.is_key_pressed(KeyCode::Up), ctx.is_key_pressed(KeyCode::Down)) {
            (true, false) => {
                self.move_towards(ctx, MOVEMENT_SPEED * dt * ray_dir);
            },
            (false, true) => {
                self.move_towards(ctx, -MOVEMENT_SPEED * dt * ray_dir);
            },
            _ => ()
        }

        self.draw_all_segments(ctx);

        let current_pos = self.pos;
        let current_dir = ray_dir;
        let end_p = current_pos + current_dir * (RADIUS * 1.3);

        BresenhamCircleDrawer::create(ctx)
            .with_position((current_pos.x as _, current_pos.y as _))
            .with_radius(RADIUS as _)
            .draw(255);

        LineRasterizer::create(ctx)
            .from((current_pos.x as _, current_pos.y as _))
            .to((end_p.x as _, end_p.y as _))
            .rasterize(255);
    }
}

impl App {
    fn draw_all_segments(&mut self, ctx: &mut RetroBlitContext) {
        for [p0, p1] in self.segments.iter() {
            LineRasterizer::create(ctx)
                .from((p0.x as _, p0.y as _))
                .to((p1.x as _, p1.y as _))
                .rasterize(0b0100_1110);
        }
    }

    fn cast_circle(&self,
                   origin: glam::Vec2,
                   p_dir: glam::Vec2,
                   radius: f32
    ) -> Option<(f32, glam::Vec2)> {
        let mut t = None;
        for segment in self.segments.iter() {
            match (t, SegmentCircleCastQuery::circle_cast_segment(
                origin, p_dir, radius, *segment
            )) {
                (None, next) => t = next,
                (Some((old_t, _)), Some((new_t, norm))) if new_t < old_t => t = Some((new_t, norm)),
                _ => ()
            }
        }
        t.map(|(t, normal)| (t - SKIN, normal))
    }
}

fn main() {
    let segments = vec![
        [glam::vec2(35.0, 88.0), glam::vec2(285.0, 190.0)],
        [glam::vec2(35.0, 88.0), glam::vec2(35.0, 40.0)]
    ];

    retro_blit::window::start(App{
        angle: 180.0,
        pos: glam::vec2(160.0, 120.0),
        segments
    })
}