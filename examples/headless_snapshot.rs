use retro_blit::rendering::bresenham::BresenhamCircleDrawer;
use retro_blit::window::{headless::HeadlessRunner, ContextHandler, RetroBlitContext, WindowMode};

struct Scene;

impl ContextHandler for Scene {
    fn get_window_title(&self) -> &'static str {
        "headless snapshot"
    }

    fn get_window_mode(&self) -> WindowMode {
        WindowMode::Mode13Frameless
    }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        ctx.set_palette(1, [24, 32, 56]);
        ctx.set_palette(2, [239, 83, 80]);
        ctx.set_palette(3, [255, 202, 40]);
        ctx.set_palette(4, [38, 198, 218]);

        ctx.clear(1);
        BresenhamCircleDrawer::create(ctx)
            .with_position((105, 100))
            .with_radius(60)
            .fill(2);
        BresenhamCircleDrawer::create(ctx)
            .with_position((190, 80))
            .with_radius(48)
            .fill(3);
        BresenhamCircleDrawer::create(ctx)
            .with_position((205, 135))
            .with_radius(38)
            .fill(4);
    }

    fn update(&mut self, _ctx: &mut RetroBlitContext, _dt: f32) {}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runner = HeadlessRunner::new(Scene);
    let snapshot = runner.snapshot();
    let path = "target/headless_snapshot.png";
    std::fs::write(path, snapshot.to_png_bytes()?)?;
    println!("Saved {path}");
    Ok(())
}
