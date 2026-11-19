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
use super::{ContextHandler, KeyCode, KeyMods, RetroBlitContext};
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
        let aspect = size.width as f32 / size.height as f32;
        let x = (position.x as f32 / size.width as f32 - 0.5) * 2.0 * aspect;
        let y = -((position.y as f32 / size.height as f32 - 0.5) * 2.0);
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
                let aspect =
                    self.context_data.buffer_width as f32 / self.context_data.buffer_height as f32;
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
