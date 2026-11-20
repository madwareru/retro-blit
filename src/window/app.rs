use std::convert::TryFrom;
use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{ModifiersState, PhysicalKey};
use winit::window::{Window, WindowId};

use super::egui_integration::EguiIntegration;
use super::monitor_obj_loader::Vec4;
use super::renderer::{RenderOutcome, Renderer};
use super::{ContextHandler, KeyCode, KeyMods, RetroBlitContext, WindowMode};
use crate::math_utils::Barycentric2D;

pub(crate) struct App<CtxHandler: ContextHandler> {
    handler: CtxHandler,
    context_data: RetroBlitContext,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    egui_integration: Option<EguiIntegration>,
    window_width: u32,
    window_height: u32,
    cursor_position: PhysicalPosition<f64>,
    modifiers: ModifiersState,
    last_instant: Instant,
}

fn screen_coordinates(
    mode: WindowMode,
    width: u32,
    height: u32,
    position: PhysicalPosition<f64>,
) -> (f32, f32) {
    let window_aspect = width as f32 / height as f32;
    let x = (position.x as f32 / width as f32 - 0.5) * 2.0 * window_aspect;
    let y = -((position.y as f32 / height as f32 - 0.5) * 2.0);
    if matches!(mode, WindowMode::ModeX | WindowMode::Mode13) {
        return (x, y);
    }
    let scale = mode.get_screen_scale(width, height);
    (x / (window_aspect * scale[0]), y / scale[1])
}

impl<CtxHandler: ContextHandler> App<CtxHandler> {
    fn new(handler: CtxHandler, window_width: u32, window_height: u32) -> Self {
        let context_data = RetroBlitContext::new(&handler);
        Self {
            handler,
            context_data,
            window: None,
            renderer: None,
            egui_integration: None,
            window_width,
            window_height,
            cursor_position: PhysicalPosition::new(0.0, 0.0),
            modifiers: ModifiersState::default(),
            last_instant: Instant::now(),
        }
    }

    fn update_mouse(&mut self, position: PhysicalPosition<f64>) {
        self.cursor_position = position;
        let Some(window) = &self.window else {
            return;
        };
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return;
        }
        let (x, y) = screen_coordinates(
            self.handler.get_window_mode(),
            size.width,
            size.height,
            position,
        );
        self.check_for_hit_test(x, y);
    }

    fn check_for_hit_test(&mut self, x: f32, y: f32) {
        match self.handler.get_window_mode() {
            super::WindowMode::ModeX | super::WindowMode::Mode13 => {
                let point = Vec4 {
                    x: x.clamp(-1.0, 1.0),
                    y: y.clamp(-1.0, 1.0),
                    z: 0.0,
                    w: 1.0,
                };
                let Some(renderer) = &self.renderer else {
                    return;
                };
                let mesh = &renderer.screen_mesh;
                for triangle in mesh.indices.chunks_exact(3) {
                    let vert0 = mesh.vertices[triangle[0] as usize];
                    let vert1 = mesh.vertices[triangle[1] as usize];
                    let vert2 = mesh.vertices[triangle[2] as usize];
                    if let Some([bar_u, bar_v, bar_w]) = Vec4::get_barycentric_2d(
                        point,
                        [vert0.position, vert1.position, vert2.position],
                    ) {
                        let u = bar_u * vert0.uv.x + bar_v * vert1.uv.x + bar_w * vert2.uv.x;
                        let v =
                            1.0 - (bar_u * vert0.uv.y + bar_v * vert1.uv.y + bar_w * vert2.uv.y);
                        self.context_data.mouse_x = u * self.context_data.buffer_width as f32;
                        self.context_data.mouse_y = v * self.context_data.buffer_height as f32;
                        return;
                    }
                }
            }
            _ => {
                let (render_width, render_height) = self
                    .handler
                    .get_window_mode()
                    .get_render_texture_dimensions();
                let aspect = render_width as f32 / render_height as f32;
                let u = ((x / aspect).clamp(-1.0, 1.0) + 1.0) / 2.0;
                let v = 1.0 - (y.clamp(-1.0, 1.0) + 1.0) / 2.0;
                self.context_data.mouse_x = u * self.context_data.buffer_width as f32;
                self.context_data.mouse_y = v * self.context_data.buffer_height as f32;
            }
        }
    }

    fn release_all_keys(&mut self) {
        let key_mods = self.context_data.key_mods_pressed;
        for key_code in self.context_data.keys_pressed.drain().collect::<Vec<_>>() {
            self.handler
                .on_key_up(&mut self.context_data, key_code, key_mods);
        }
        self.modifiers = ModifiersState::default();
        self.context_data.key_mods_pressed = self.key_mods();
    }

    fn key_mods(&self) -> KeyMods {
        KeyMods {
            shift: self.modifiers.shift_key(),
            control: self.modifiers.control_key(),
            option: self.modifiers.alt_key(),
            command: self.modifiers.super_key(),
        }
    }
}

impl<CtxHandler: ContextHandler> ApplicationHandler for App<CtxHandler> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title(self.handler.get_window_title())
            .with_inner_size(LogicalSize::new(self.window_width, self.window_height))
            .with_resizable(true);
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("failed to create window"),
        );
        self.context_data.init_audio();
        self.handler.init(&mut self.context_data);
        let (width, height) = (
            self.context_data.buffer_width,
            self.context_data.buffer_height,
        );
        let renderer = Renderer::new(
            Arc::clone(&window),
            self.handler.get_window_mode(),
            width,
            height,
            &self.context_data.buffer_pixels,
            &self.context_data.colors,
        );
        self.egui_integration = Some(EguiIntegration::new(
            &window,
            self.context_data.get_egui_ctx(),
            &renderer.device,
            renderer.config.format,
        ));
        self.renderer = Some(renderer);
        self.last_instant = Instant::now();
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref().map(Arc::clone) else {
            return;
        };
        if window.id() != window_id {
            return;
        }
        if let Some(egui) = &mut self.egui_integration {
            egui.on_window_event(&window, &event);
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Focused(false) => self.release_all_keys(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                let size = window.inner_size();
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.update_mouse(position);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let position = self.cursor_position;
                self.update_mouse(position);
                let button_number = match button {
                    MouseButton::Left => Some(0),
                    MouseButton::Middle => Some(1),
                    MouseButton::Right => Some(2),
                    _ => None,
                };
                if let Some(button_number) = button_number {
                    match state {
                        ElementState::Pressed => {
                            self.handler
                                .on_mouse_down(&mut self.context_data, button_number);
                        }
                        ElementState::Released => {
                            self.handler
                                .on_mouse_up(&mut self.context_data, button_number);
                        }
                    }
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers.state();
                self.context_data.key_mods_pressed = self.key_mods();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let key_mods = self.key_mods();
                self.context_data.key_mods_pressed = key_mods;
                let PhysicalKey::Code(code) = event.physical_key else {
                    return;
                };
                let Ok(key_code) = KeyCode::try_from(code) else {
                    return;
                };
                match event.state {
                    ElementState::Pressed => {
                        self.context_data.keys_pressed.insert(key_code);
                        self.handler
                            .on_key_down(&mut self.context_data, key_code, key_mods);
                    }
                    ElementState::Released => {
                        self.context_data.keys_pressed.remove(&key_code);
                        self.handler
                            .on_key_up(&mut self.context_data, key_code, key_mods);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let Some(renderer) = &mut self.renderer else {
                    return;
                };
                let egui_frame = if renderer.config.width == 0 || renderer.config.height == 0 {
                    None
                } else if let Some(egui) = &mut self.egui_integration {
                    Some(egui.run_frame(
                        &window,
                        event_loop,
                        &mut self.handler,
                        &mut self.context_data,
                    ))
                } else {
                    None
                };
                let egui = self.egui_integration.as_mut().zip(egui_frame);
                match renderer.render(
                    self.context_data.buffer_width,
                    &self.context_data.buffer_pixels,
                    &self.context_data.colors,
                    egui,
                ) {
                    RenderOutcome::Presented | RenderOutcome::Skip => {}
                    RenderOutcome::Reconfigure => {
                        let size = window.inner_size();
                        renderer.resize(size.width, size.height);
                    }
                    RenderOutcome::OutOfMemory => event_loop.exit(),
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = &self.window else {
            return;
        };
        if self.context_data.quit_fired {
            event_loop.exit();
            return;
        }
        if let Some(hide) = self.context_data.cursor_hidden_fired {
            window.set_cursor_visible(!hide);
            self.context_data.cursor_hidden_fired = None;
        }
        let dt = self.last_instant.elapsed().as_micros() as f32 / 1_000_000.0;
        self.last_instant = Instant::now();
        if let Some(driver) = &mut self.context_data.sound_driver {
            driver.maintain();
        }
        self.handler.update(&mut self.context_data, dt);
        window.request_redraw();
    }
}

pub(crate) fn run<CtxHandler: 'static + ContextHandler>(
    handler: CtxHandler,
    width: u32,
    height: u32,
) {
    let event_loop = EventLoop::new().expect("failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(handler, width, height);
    event_loop.run_app(&mut app).expect("event loop failed");
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::WindowMode;

    struct TestHandler(WindowMode);

    impl ContextHandler for TestHandler {
        fn get_window_title(&self) -> &'static str {
            "test"
        }

        fn get_window_mode(&self) -> WindowMode {
            self.0
        }

        fn init(&mut self, _ctx: &mut RetroBlitContext) {}

        fn update(&mut self, _ctx: &mut RetroBlitContext, _dt: f32) {}
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.001, "expected {}, got {}", expected, actual);
    }

    #[test]
    fn flat_screen_scale_fits_both_window_axes() {
        let cases = [
            (WindowMode::Mode64x64, 1600, 900, [0.5625, 1.0]),
            (WindowMode::Mode64x64, 900, 1600, [1.0, 0.5625]),
            (WindowMode::Mode64x64, 900, 900, [1.0, 1.0]),
            (WindowMode::ModeXFrameless, 1600, 900, [0.5625, 1.0]),
            (WindowMode::ModeXFrameless, 900, 1600, [0.75, 0.421875]),
            (WindowMode::ModeXFrameless, 1200, 900, [0.75, 1.0]),
            (WindowMode::Mode240x150, 1600, 900, [0.5625, 1.0]),
            (WindowMode::Mode240x150, 900, 1600, [0.625, 0.3515625]),
            (WindowMode::Mode240x150, 1600, 1000, [0.625, 1.0]),
        ];
        for (mode, width, height, expected) in cases.iter().copied() {
            let scale = mode.get_screen_scale(width, height);
            assert_close(scale[0], expected[0]);
            assert_close(scale[1], expected[1]);
        }

        for (width, height) in [(1600, 900), (900, 1600)].iter().copied() {
            let scale = WindowMode::Mode13.get_screen_scale(width, height);
            assert_close(scale[0], height as f32 / width as f32);
            assert_close(scale[1], 1.0);
        }
    }

    #[test]
    fn flat_mouse_mapping_matches_contained_screen() {
        let cases = [
            (WindowMode::Mode64x64, 350.0, 1250.0, 350.0, 1250.0),
            (WindowMode::ModeXFrameless, 200.0, 1400.0, 462.5, 1137.5),
            (WindowMode::Mode240x150, 80.0, 1520.0, 518.75, 1081.25),
        ];
        for (mode, wide_left, wide_right, tall_top, tall_bottom) in cases.iter().copied() {
            let (buffer_width, buffer_height) = mode.get_buffer_dimensions();
            let mut app = App::new(
                TestHandler(mode),
                buffer_width as u32,
                buffer_height as u32,
            );
            let mut check = |window_width, window_height, x, y| {
                let (x, y) = screen_coordinates(
                    mode,
                    window_width,
                    window_height,
                    PhysicalPosition::new(x, y),
                );
                app.check_for_hit_test(x, y);
                (app.context_data.mouse_x, app.context_data.mouse_y)
            };

            for x in [0.0, wide_left].iter().copied() {
                let (mouse_x, mouse_y) = check(1600, 900, x, 450.0);
                assert_close(mouse_x, 0.0);
                assert_close(mouse_y, buffer_height as f32 / 2.0);
            }
            for x in [wide_right, 1600.0].iter().copied() {
                let (mouse_x, mouse_y) = check(1600, 900, x, 450.0);
                assert_close(mouse_x, buffer_width as f32);
                assert_close(mouse_y, buffer_height as f32 / 2.0);
            }
            let (mouse_x, mouse_y) = check(1600, 900, 800.0, 450.0);
            assert_close(mouse_x, buffer_width as f32 / 2.0);
            assert_close(mouse_y, buffer_height as f32 / 2.0);

            for y in [0.0, tall_top].iter().copied() {
                let (mouse_x, mouse_y) = check(900, 1600, 450.0, y);
                assert_close(mouse_x, buffer_width as f32 / 2.0);
                assert_close(mouse_y, 0.0);
            }
            for y in [tall_bottom, 1600.0].iter().copied() {
                let (mouse_x, mouse_y) = check(900, 1600, 450.0, y);
                assert_close(mouse_x, buffer_width as f32 / 2.0);
                assert_close(mouse_y, buffer_height as f32);
            }
            let (mouse_x, mouse_y) = check(900, 1600, 450.0, 800.0);
            assert_close(mouse_x, buffer_width as f32 / 2.0);
            assert_close(mouse_y, buffer_height as f32 / 2.0);
        }
    }

    #[test]
    fn flat_modes_map_display_edges_to_buffer_edges() {
        let modes = [
            (WindowMode::Mode13Frameless, 320, 200, 4.0 / 3.0),
            (WindowMode::ModeXFrameless, 320, 240, 4.0 / 3.0),
            (WindowMode::Mode64x64, 64, 64, 1.0),
            (WindowMode::Mode128x128, 128, 128, 1.0),
            (WindowMode::Mode256x256, 256, 256, 1.0),
            (WindowMode::Mode160x120, 160, 120, 4.0 / 3.0),
            (WindowMode::Mode800x600, 800, 600, 4.0 / 3.0),
            (WindowMode::Mode240x150, 240, 150, 16.0 / 10.0),
            (WindowMode::Mode480x300, 480, 300, 16.0 / 10.0),
            (WindowMode::Mode640x400, 640, 400, 16.0 / 10.0),
            (WindowMode::Mode960x600, 960, 600, 16.0 / 10.0),
        ];
        for (mode, width, height, display_aspect) in modes.iter().copied() {
            let mut app = App::new(TestHandler(mode), width as u32, height as u32);
            app.check_for_hit_test(display_aspect, 0.0);
            assert_close(app.context_data.mouse_x, width as f32);
            assert_close(app.context_data.mouse_y, height as f32 / 2.0);
            app.check_for_hit_test(-display_aspect, 0.0);
            assert_close(app.context_data.mouse_x, 0.0);
        }
    }
}
