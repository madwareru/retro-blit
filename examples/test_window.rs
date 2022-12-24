use retro_blit::rendering::BlittableSurface;
use retro_blit::rendering::deformed_rendering::{TexturedVertex, TriangleRasterizer};
use retro_blit::window::{RetroBlitContext, ContextHandler, WindowMode};
use egui::*;

const PICTURE_BYTES: &[u8] = include_bytes!("spritesheet.im256");

#[derive(Copy, Clone)]
pub enum Tile {
    Water,
    Grass,
    Dirt,
    Sand,
    Wheat
}

struct MyGame {
    time: f32,
    palette: Vec<[u8; 3]>,
    sprite_sheet: BlittableSurface,
    button_pressed: bool,
}
impl MyGame {
    pub fn new() -> Self {
        let (palette, sprite_sheet) = retro_blit::format_loaders::im_256::Image::load_from(PICTURE_BYTES).unwrap();
        Self {
            time: 0.0,
            palette,
            sprite_sheet,
            button_pressed: false
        }
    }
}

impl ContextHandler for MyGame {
    fn get_window_title(&self) -> &'static str {
        "test_window"
    }

    fn get_window_mode(&self) -> WindowMode {
        WindowMode::Mode13Frameless
    }

    fn on_mouse_down(&mut self, _ctx: &mut RetroBlitContext, button_number: u8) {
        if button_number == 0 {
            self.button_pressed = true;
        }
    }

    fn on_mouse_up(&mut self, _ctx: &mut RetroBlitContext, button_number: u8) {
        if button_number == 0 {
            self.button_pressed = false;
        }
    }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        for i in 0..self.palette.len() {
            ctx.set_palette(i as _, self.palette[i]);
        }
    }

    fn update(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        self.time += dt;

        ctx.clear(1);

        let sprite_sheet_with_color_key = self
            .sprite_sheet
            .with_color_key(0);

        TriangleRasterizer::create(ctx)
            .with_translation((160, 100))
            .with_rotation(self.time)
            .with_scale((2.0 + 1.0 * self.time.sin(), 2.0 + 1.0 * self.time.sin()))
            .rasterize_with_surface(
                &sprite_sheet_with_color_key,
                &[
                    TexturedVertex {
                        position: (-24.0, -20.0),
                        uv: (0, 0)
                    },
                    TexturedVertex {
                        position: (24.0, -20.0),
                        uv: (23, 0)
                    },
                    TexturedVertex {
                        position: (-24.0, 20.0),
                        uv: (0, 20)
                    },
                    TexturedVertex {
                        position: (24.0, 20.0),
                        uv: (23, 20)
                    },
                ],
                &[0, 1, 2, 2, 1, 3]
            );
    }

    fn egui(&mut self, ctx: &mut RetroBlitContext, egui_ctx: egui::Context) {
        egui::Window::new("general")
            .show(&egui_ctx, |ui| {
                ui.label("Some label");
                if ui.button("Quit").clicked() {
                    println!("clicked on quit");
                    ctx.quit();
                }
            });
    }
}


fn main() {
    retro_blit::window::start(MyGame::new());
}