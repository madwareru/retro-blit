use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

use super::{ContextHandler, RetroBlitContext};

pub(crate) struct EguiIntegration {
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
    pending_textures: egui::TexturesDelta,
}

pub(crate) struct EguiFrame {
    paint_jobs: Vec<egui::ClippedPrimitive>,
    textures_delta: egui::TexturesDelta,
    pixels_per_point: f32,
}

impl EguiIntegration {
    pub(crate) fn new(
        window: &Window,
        egui_context: egui::Context,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Self {
        let state = egui_winit::State::new(
            egui_context,
            egui::ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            window.theme(),
            Some(device.limits().max_texture_dimension_2d as usize),
        );
        let renderer = egui_wgpu::Renderer::new(
            device,
            format,
            egui_wgpu::RendererOptions {
                msaa_samples: 1,
                ..Default::default()
            },
        );
        Self {
            state,
            renderer,
            pending_textures: egui::TexturesDelta::default(),
        }
    }

    pub(crate) fn on_window_event(&mut self, window: &Window, event: &WindowEvent) {
        let _ = self.state.on_window_event(window, event);
    }

    pub(crate) fn run_frame<Handler: ContextHandler>(
        &mut self,
        window: &Window,
        event_loop: &ActiveEventLoop,
        handler: &mut Handler,
        context: &mut RetroBlitContext,
    ) -> EguiFrame {
        let egui_context = context.get_egui_ctx();
        let raw_input = self.state.take_egui_input(window);
        let output = run_context(&egui_context, raw_input, handler, context);
        self.state.handle_platform_output_with_event_loop(
            window,
            event_loop,
            output.platform_output,
        );
        let paint_jobs = egui_context.tessellate(output.shapes, output.pixels_per_point);
        EguiFrame {
            paint_jobs,
            textures_delta: merge_pending(&mut self.pending_textures, output.textures_delta),
            pixels_per_point: output.pixels_per_point,
        }
    }

    pub(crate) fn defer(&mut self, frame: EguiFrame) {
        defer_frame(&mut self.pending_textures, frame);
    }

    pub(crate) fn encode(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        config: &wgpu::SurfaceConfiguration,
        frame: &mut EguiFrame,
    ) -> Vec<wgpu::CommandBuffer> {
        for (id, deltas) in frame.textures_delta.set.drain() {
            for delta in deltas {
                self.renderer.update_texture(device, queue, id, &delta);
            }
        }
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [config.width, config.height],
            pixels_per_point: frame.pixels_per_point,
        };
        let command_buffers = self.renderer.update_buffers(
            device,
            queue,
            encoder,
            &frame.paint_jobs,
            &screen_descriptor,
        );
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("egui"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        self.renderer.render(
            &mut pass.forget_lifetime(),
            &frame.paint_jobs,
            &screen_descriptor,
        );
        command_buffers
    }

    pub(crate) fn finish(&mut self, frame: &mut EguiFrame) {
        for id in frame.textures_delta.free.drain() {
            self.renderer.free_texture(&id);
        }
    }
}

fn defer_frame(pending: &mut egui::TexturesDelta, mut frame: EguiFrame) {
    pending.append(std::mem::take(&mut frame.textures_delta));
}

fn merge_pending(
    pending: &mut egui::TexturesDelta,
    newer: egui::TexturesDelta,
) -> egui::TexturesDelta {
    let mut merged = std::mem::take(pending);
    merged.append(newer);
    merged
}

impl Drop for EguiIntegration {
    fn drop(&mut self) {
        self.pending_textures.clear();
    }
}

impl Drop for EguiFrame {
    fn drop(&mut self) {
        self.textures_delta.clear();
    }
}

fn run_context<Handler: ContextHandler>(
    egui_context: &egui::Context,
    raw_input: egui::RawInput,
    handler: &mut Handler,
    context: &mut RetroBlitContext,
) -> egui::FullOutput {
    egui_context.run_pass(raw_input, |egui_context| {
        handler.egui(context, egui_context);
    })
}
