use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::window::Window;

use super::egui_integration::{EguiFrame, EguiIntegration};
use super::monitor_obj_loader::{Mesh, Vec4};
use super::texture_upload::{palette_as_rgba, TextureUploader};
use super::{WindowMode, IMAGE_BYTES};

const SAMPLE_COUNT: u32 = 4;

pub(crate) enum RenderOutcome {
    Presented,
    Reconfigure,
    Skip,
    OutOfMemory,
}

struct GpuMesh {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    index_count: u32,
}

pub(crate) struct Renderer {
    _instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pixel_texture: wgpu::Texture,
    palette_texture: wgpu::Texture,
    offscreen_pipeline: wgpu::RenderPipeline,
    offscreen_bind_group: wgpu::BindGroup,
    offscreen_view: wgpu::TextureView,
    offscreen_mesh: GpuMesh,
    screen_pipeline: wgpu::RenderPipeline,
    screen_bind_group: wgpu::BindGroup,
    screen_gpu_mesh: GpuMesh,
    mask_pipeline: wgpu::RenderPipeline,
    mask_bind_group: wgpu::BindGroup,
    mask_gpu_mesh: GpuMesh,
    aspect_buffer: wgpu::Buffer,
    msaa_view: wgpu::TextureView,
    texture_uploader: TextureUploader,
    out_of_memory: Arc<AtomicBool>,
    pub(crate) screen_mesh: Mesh,
}

impl Renderer {
    pub(crate) fn new(
        window: Arc<Window>,
        mode: WindowMode,
        buffer_width: usize,
        buffer_height: usize,
        pixels: &[u8],
        colors: &[u8; 256 * 3],
    ) -> Self {
        let initial_size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .expect("failed to create wgpu surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("failed to find a compatible wgpu adapter");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("failed to create wgpu device");

        let out_of_memory = Arc::new(AtomicBool::new(false));
        let error_flag = Arc::clone(&out_of_memory);
        device.on_uncaptured_error(Arc::new(move |error| match error {
            wgpu::Error::OutOfMemory { .. } => {
                error_flag.store(true, Ordering::Relaxed);
            }
            other => panic!("wgpu error: {}", other),
        }));

        let mut config = surface
            .get_default_config(
                &adapter,
                initial_size.width.max(1),
                initial_size.height.max(1),
            )
            .expect("surface is incompatible with the selected adapter");
        config.format = surface
            .get_capabilities(&adapter)
            .formats
            .iter()
            .copied()
            .find(|format| !format.is_srgb())
            .unwrap_or(config.format);
        surface.configure(&device, &config);

        let pixel_texture = create_texture(
            &device,
            "indexed pixels",
            buffer_width as u32,
            buffer_height as u32,
            wgpu::TextureFormat::R8Unorm,
            wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            1,
        );
        let palette_texture = create_texture(
            &device,
            "palette",
            256,
            1,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            1,
        );
        write_initial_textures(
            &queue,
            &pixel_texture,
            &palette_texture,
            buffer_width,
            buffer_height,
            pixels,
            colors,
        );

        let (mask_mesh, screen_mesh) = make_meshes(mode);
        let (render_width, render_height) = mode.get_render_texture_dimensions();
        let offscreen_texture = create_texture(
            &device,
            "palette-composed screen",
            render_width as u32,
            render_height as u32,
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            1,
        );
        let offscreen_view = offscreen_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mask_image = image::load_from_memory(IMAGE_BYTES)
            .unwrap_or_else(|error| panic!("{}", error))
            .to_rgba8();
        let mask_texture = create_texture(
            &device,
            "monitor mask",
            mask_image.width(),
            mask_image.height(),
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            1,
        );
        queue.write_texture(
            mask_texture.as_image_copy(),
            mask_image.as_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(mask_image.width() * 4),
                rows_per_image: Some(mask_image.height()),
            },
            wgpu::Extent3d {
                width: mask_image.width(),
                height: mask_image.height(),
                depth_or_array_layers: 1,
            },
        );
        let mask_view = mask_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let nearest_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nearest sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("linear sampler"),
            ..Default::default()
        });
        let aspect_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("screen aspect"),
            contents: bytemuck::bytes_of(&1.0f32),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let offscreen_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("offscreen bind group layout"),
            entries: &[
                texture_layout_entry(0),
                sampler_layout_entry(1),
                texture_layout_entry(2),
            ],
        });
        let offscreen_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("offscreen pipeline layout"),
                bind_group_layouts: &[Some(&offscreen_layout)],
                immediate_size: 0,
            });
        let offscreen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("offscreen bind group"),
            layout: &offscreen_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &pixel_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&nearest_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &palette_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
            ],
        });
        let offscreen_pipeline = create_pipeline(
            &device,
            "offscreen pipeline",
            include_str!("shaders/offscreen.wgsl"),
            &offscreen_pipeline_layout,
            offscreen_vertex_layout(),
            wgpu::TextureFormat::Rgba8Unorm,
            1,
            None,
        );

        let screen_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("screen bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                texture_layout_entry(1),
                sampler_layout_entry(2),
            ],
        });
        let screen_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("screen pipeline layout"),
                bind_group_layouts: &[Some(&screen_layout)],
                immediate_size: 0,
            });
        let screen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen bind group"),
            layout: &screen_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: aspect_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&offscreen_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&linear_sampler),
                },
            ],
        });
        let mask_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mask bind group"),
            layout: &screen_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: aspect_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&mask_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&linear_sampler),
                },
            ],
        });
        let blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Zero,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let screen_pipeline = create_pipeline(
            &device,
            "screen pipeline",
            include_str!("shaders/screen.wgsl"),
            &screen_pipeline_layout,
            mesh_vertex_layout(),
            config.format,
            SAMPLE_COUNT,
            Some(blend),
        );
        let mask_pipeline = create_pipeline(
            &device,
            "mask pipeline",
            include_str!("shaders/mask.wgsl"),
            &screen_pipeline_layout,
            mesh_vertex_layout(),
            config.format,
            SAMPLE_COUNT,
            Some(blend),
        );

        let offscreen_mesh = create_offscreen_mesh(&device);
        let screen_gpu_mesh = create_gpu_mesh(&device, "screen", &screen_mesh);
        let mask_gpu_mesh = create_gpu_mesh(&device, "mask", &mask_mesh);
        let msaa_view = create_msaa_view(&device, &config);

        Self {
            _instance: instance,
            surface,
            config,
            device,
            queue,
            pixel_texture,
            palette_texture,
            offscreen_pipeline,
            offscreen_bind_group,
            offscreen_view,
            offscreen_mesh,
            screen_pipeline,
            screen_bind_group,
            screen_gpu_mesh,
            mask_pipeline,
            mask_bind_group,
            mask_gpu_mesh,
            aspect_buffer,
            msaa_view,
            texture_uploader: TextureUploader::new(pixels, colors),
            out_of_memory,
            screen_mesh,
        }
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        if width == 0 || height == 0 {
            return;
        }
        self.surface.configure(&self.device, &self.config);
        self.msaa_view = create_msaa_view(&self.device, &self.config);
    }

    pub(crate) fn render(
        &mut self,
        width: usize,
        pixels: &[u8],
        colors: &[u8; 256 * 3],
        mut egui: Option<(&mut EguiIntegration, EguiFrame)>,
    ) -> RenderOutcome {
        if self.out_of_memory.load(Ordering::Relaxed) {
            defer_egui(&mut egui);
            return RenderOutcome::OutOfMemory;
        }
        if self.config.width == 0 || self.config.height == 0 {
            defer_egui(&mut egui);
            return RenderOutcome::Skip;
        }

        let (frame, reconfigure) = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => (frame, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => (frame, true),
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                defer_egui(&mut egui);
                return RenderOutcome::Reconfigure;
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                defer_egui(&mut egui);
                return RenderOutcome::Skip;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                panic!("wgpu surface validation error");
            }
        };
        let frame_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let aspect = self.config.height as f32 / self.config.width as f32;
        self.queue
            .write_buffer(&self.aspect_buffer, 0, bytemuck::bytes_of(&aspect));

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("retro-blit frame"),
            });
        self.texture_uploader.encode_staged(
            &self.device,
            &self.queue,
            &mut encoder,
            &self.pixel_texture,
            &self.palette_texture,
            width,
            pixels,
            colors,
        );
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("palette composition"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.offscreen_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: clear_operations(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.offscreen_pipeline);
            pass.set_bind_group(0, &self.offscreen_bind_group, &[]);
            set_mesh(&mut pass, &self.offscreen_mesh);
            pass.draw_indexed(0..self.offscreen_mesh.index_count, 0, 0..1);
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("screen and monitor mask"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.msaa_view,
                    depth_slice: None,
                    resolve_target: Some(&frame_view),
                    ops: clear_operations(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if self.screen_gpu_mesh.index_count > 0 {
                pass.set_pipeline(&self.screen_pipeline);
                pass.set_bind_group(0, &self.screen_bind_group, &[]);
                set_mesh(&mut pass, &self.screen_gpu_mesh);
                pass.draw_indexed(0..self.screen_gpu_mesh.index_count, 0, 0..1);
            }
            if self.mask_gpu_mesh.index_count > 0 {
                pass.set_pipeline(&self.mask_pipeline);
                pass.set_bind_group(0, &self.mask_bind_group, &[]);
                set_mesh(&mut pass, &self.mask_gpu_mesh);
                pass.draw_indexed(0..self.mask_gpu_mesh.index_count, 0, 0..1);
            }
        }
        let mut egui_command_buffers = Vec::new();
        if let Some((integration, frame)) = egui.as_mut() {
            egui_command_buffers = integration.encode(
                &self.device,
                &self.queue,
                &mut encoder,
                &frame_view,
                &self.config,
                frame,
            );
        }
        self.queue.submit(
            egui_command_buffers
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );
        self.texture_uploader.finish_submission();
        if let Some((integration, mut frame)) = egui {
            integration.finish(&mut frame);
        }
        self.queue.present(frame);

        if self.out_of_memory.load(Ordering::Relaxed) {
            RenderOutcome::OutOfMemory
        } else if reconfigure {
            RenderOutcome::Reconfigure
        } else {
            RenderOutcome::Presented
        }
    }
}

fn defer_egui(egui: &mut Option<(&mut EguiIntegration, EguiFrame)>) {
    if let Some((integration, frame)) = egui.take() {
        integration.defer(frame);
    }
}

fn make_meshes(mode: WindowMode) -> (Mesh, Mesh) {
    match mode {
        WindowMode::ModeX | WindowMode::Mode13 => {
            let monitor_models = Mesh::load_meshes().unwrap();
            let mut mask_mesh = monitor_models.get("mask").unwrap().clone();
            let mut screen_mesh = monitor_models.get("screen").unwrap().clone();
            let cs_t = (-0.0025f32).cos();
            let sn_t = (-0.0025f32).sin();
            for vertex in &mut mask_mesh.vertices {
                let Vec4 { x, z, .. } = vertex.position;
                vertex.position.x = -z * 0.75;
                vertex.position.y *= 0.75;
                vertex.position.z = x * 0.75;
                let x_new = vertex.position.x * cs_t - vertex.position.y * sn_t;
                let y_new = vertex.position.x * sn_t + vertex.position.y * cs_t;
                vertex.position.x = x_new;
                vertex.position.y = y_new;
            }
            for vertex in &mut screen_mesh.vertices {
                let Vec4 { x, z, .. } = vertex.position;
                vertex.position.x = -z * 0.75;
                vertex.position.y *= 0.75;
                vertex.position.z = x * 0.75;
                let d_x = vertex.uv.x - 0.5;
                let d_y = vertex.uv.y - 0.5;
                let curvature_x = (1.0 - d_x * d_x * 4.0) * d_y / 40.0;
                let curvature_y = (1.0 - d_y * d_y * 4.0) * d_x / 40.0;
                vertex.position.x += curvature_y;
                vertex.position.y += curvature_x;
            }
            (mask_mesh, screen_mesh)
        }
        WindowMode::Mode13Frameless
        | WindowMode::ModeXFrameless
        | WindowMode::Mode160x120
        | WindowMode::Mode800x600 => (Mesh::make_empty(), Mesh::make_4x3()),
        WindowMode::Mode64x64 | WindowMode::Mode128x128 | WindowMode::Mode256x256 => {
            (Mesh::make_empty(), Mesh::make_square())
        }
        WindowMode::Mode240x150
        | WindowMode::Mode480x300
        | WindowMode::Mode640x400
        | WindowMode::Mode960x600 => (Mesh::make_empty(), Mesh::make_16x10()),
    }
}

fn create_texture(
    device: &wgpu::Device,
    label: &str,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
    sample_count: u32,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

fn write_initial_textures(
    queue: &wgpu::Queue,
    pixel_texture: &wgpu::Texture,
    palette_texture: &wgpu::Texture,
    width: usize,
    height: usize,
    pixels: &[u8],
    colors: &[u8; 256 * 3],
) {
    queue.write_texture(
        pixel_texture.as_image_copy(),
        pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width as u32),
            rows_per_image: Some(height as u32),
        },
        wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
    );
    queue.write_texture(
        palette_texture.as_image_copy(),
        &palette_as_rgba(colors),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(256 * 4),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: 256,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
}

fn texture_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn sampler_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}

fn offscreen_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: 16,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 8,
                shader_location: 1,
            },
        ],
    }
}

fn mesh_vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: 24,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 16,
                shader_location: 1,
            },
        ],
    }
}

fn create_pipeline(
    device: &wgpu::Device,
    label: &str,
    shader_source: &str,
    layout: &wgpu::PipelineLayout,
    vertex_layout: wgpu::VertexBufferLayout<'_>,
    format: wgpu::TextureFormat,
    sample_count: u32,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[Some(vertex_layout)],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: sample_count,
            ..Default::default()
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn create_gpu_mesh(device: &wgpu::Device, label: &str, mesh: &Mesh) -> GpuMesh {
    GpuMesh {
        vertices: device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }),
        indices: device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        }),
        index_count: mesh.indices.len() as u32,
    }
}

fn create_offscreen_mesh(device: &wgpu::Device) -> GpuMesh {
    #[rustfmt::skip]
    let vertices: &[f32] = &[
        -1.0, -1.0, 0.0, 0.0,
         1.0,  1.0, 1.0, 1.0,
        -1.0,  1.0, 0.0, 1.0,
         1.0, -1.0, 1.0, 0.0,
    ];
    GpuMesh {
        vertices: device.create_buffer_init(&BufferInitDescriptor {
            label: Some("offscreen vertices"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }),
        indices: device.create_buffer_init(&BufferInitDescriptor {
            label: Some("offscreen indices"),
            contents: bytemuck::cast_slice(&[0u16, 1, 2, 0, 3, 1]),
            usage: wgpu::BufferUsages::INDEX,
        }),
        index_count: 6,
    }
}

fn create_msaa_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::TextureView {
    create_texture(
        device,
        "4x multisampled output",
        config.width,
        config.height,
        config.format,
        wgpu::TextureUsages::RENDER_ATTACHMENT,
        SAMPLE_COUNT,
    )
    .create_view(&wgpu::TextureViewDescriptor::default())
}

fn clear_operations() -> wgpu::Operations<wgpu::Color> {
    wgpu::Operations {
        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
        store: wgpu::StoreOp::Store,
    }
}

fn set_mesh(pass: &mut wgpu::RenderPass<'_>, mesh: &GpuMesh) {
    pass.set_vertex_buffer(0, mesh.vertices.slice(..));
    pass.set_index_buffer(mesh.indices.slice(..), wgpu::IndexFormat::Uint16);
}
