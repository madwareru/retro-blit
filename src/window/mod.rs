use orom_miniquad::*;

pub mod monitor_obj_loader;
use monitor_obj_loader::Vec4;
use crate::rendering::blittable::{BufferProviderMut, SizedSurface};
use crate::utility::Barycentric2D;

const IMAGE_BYTES: &[u8] = include_bytes!("monitor_mask.png");

pub struct RetroBlitContext {
    buffer_width: usize,
    buffer_height: usize,
    colors: [u8; 256 * 3],
    buffer_pixels: Vec<u8>,
    mouse_x: f32,
    mouse_y: f32
}

pub enum ScrollKind {
    AllPalette,
    Range{ start_idx: u8, len: u8 }
}

pub enum ScrollDirection {
    Forward,
    Backward
}

impl SizedSurface for RetroBlitContext {
    fn get_width(&self) -> usize {
        self.buffer_width
    }

    fn get_height(&self) -> usize {
        self.buffer_height
    }
}

impl BufferProviderMut<u8> for RetroBlitContext  {
    fn get_buffer_mut(&mut self) -> &mut [u8] { &mut self.buffer_pixels }
}

impl RetroBlitContext {

    pub fn clear(&mut self, color_idx: u8) {
        for pixel in self.buffer_pixels.iter_mut() {
            *pixel = color_idx;
        }
    }

    pub fn get_mouse_pos(&self) -> (f32, f32) {
        (self.mouse_x, self.mouse_y)
    }

    pub fn get_palette(&self, index: u8) -> [u8; 3] {
        let offset = self.make_palette_offset(index as usize);
        [self.colors[offset], self.colors[offset + 1], self.colors[offset + 2]]
    }

    pub fn set_palette(&mut self, index: u8, new_value: [u8; 3]) {
        let offset = self.make_palette_offset(index as usize);
        self.colors[offset..offset+3].copy_from_slice(&new_value);
    }

    pub fn scroll_palette(&mut self, scroll_kind: ScrollKind, scroll_direction: ScrollDirection) {
        let (start_idx, last_idx) = match scroll_kind {
            ScrollKind::AllPalette => (0, 255),
            ScrollKind::Range { start_idx, len } => {
                (start_idx, start_idx + len - 1)
            }
        };

        match scroll_direction {
            ScrollDirection::Forward => {
                let start_pal = self.get_palette(last_idx);
                for i in ((start_idx + 1)..=last_idx).rev() {
                    self.set_palette(i, self.get_palette(i - 1));
                }
                self.set_palette(start_idx, start_pal);
            }
            ScrollDirection::Backward => {
                let start_pal = self.get_palette(start_idx);
                for i in start_idx..last_idx {
                    self.set_palette(i, self.get_palette(i + 1));
                }
                self.set_palette(last_idx, start_pal);
            }
        };
    }

    #[inline(always)]
    fn make_palette_offset(&self, ix: usize) -> usize { ((ix % 256) * 3) % self.colors.len() }
}

pub trait ContextHandler {
    fn get_window_title(&self) -> &'static str;
    fn get_window_mode(&self) -> WindowMode;
    fn on_mouse_down(&mut self, ctx: &mut RetroBlitContext, button_number: u8);
    fn on_mouse_up(&mut self, ctx: &mut RetroBlitContext, button_number: u8);
    fn init(&mut self, ctx: &mut RetroBlitContext);
    fn update(&mut self, ctx: &mut RetroBlitContext);
}

fn get_buffer_dimensions(handler: &impl ContextHandler) -> (usize, usize) {
    handler.get_window_mode().get_buffer_dimensions()
}

pub struct Stage<CtxHandler: ContextHandler> {
    mask_vertices_count: usize,
    screen_vertices_count: usize,
    mask_pipeline: Pipeline,
    mask_binding: Bindings,
    screen_mesh: monitor_obj_loader::Mesh,
    screen_pipeline: Pipeline,
    screen_binding: Bindings,
    offscreen_pipeline: Pipeline,
    offscreen_binding: Bindings,
    offscreen_pass: RenderPass,
    context_data: RetroBlitContext,
    handler: CtxHandler,
    buffer_texture: Texture,
    colors_texture: Texture
}

impl<CtxHandler: ContextHandler> Stage<CtxHandler> {
    pub fn new(ctx: &mut Context, handler: CtxHandler) -> Stage<CtxHandler> {
        // it's okay to crash here since we can't do anything useful without monitor models
        // And still it will print a meaningful message, so we leave it like this
        let monitor_models = monitor_obj_loader::Mesh::load_meshes().unwrap();

        let mut mask_mesh = monitor_models.get("mask").unwrap().clone();
        let mut screen_mesh = monitor_models.get("screen").unwrap().clone();

        let mask_vertices_count = mask_mesh.vertices.len();
        let screen_vertices_count = screen_mesh.vertices.len();

        let cs_t = (-0.0025f32).cos();
        let sn_t = (-0.0025f32).sin();

        for v in mask_mesh.vertices.iter_mut() {
            let Vec4 { x, z, .. } = v.position;
            v.position.x = -z;
            v.position.z = x;
            v.position.x *= 0.75;
            v.position.y *= 0.75;
            v.position.z *= 0.75;

            //we need to slightly rotate screen to align it with a screen
            let x_new = v.position.x * cs_t - v.position.y * sn_t;
            let y_new = v.position.x * sn_t + v.position.y * cs_t;

            v.position.x = x_new;
            v.position.y = y_new;
        }

        for v in screen_mesh.vertices.iter_mut() {
            let Vec4 { x, z, .. } = v.position;
            v.position.x = -z;
            v.position.z = x;
            v.position.x *= 0.75;
            v.position.y *= 0.75;
            v.position.z *= 0.75;

            let d_x = v.uv.x - 0.5;
            let d_y = v.uv.y - 0.5;
            let curvature_x = (1.0 - d_x * d_x * 4.0 ) * d_y / 40.0;
            let curvature_y = (1.0 - d_y * d_y * 4.0 ) * d_x / 40.0;

            v.position.x += curvature_y;
            v.position.y += curvature_x;
        }

        let mask_vertex_buffer = Buffer::immutable(
            ctx,
            BufferType::VertexBuffer,
            &mask_mesh.vertices
        );

        let mask_index_buffer = Buffer::immutable(
            ctx,
            BufferType::IndexBuffer,
            &mask_mesh.indices
        );

        let screen_vertex_buffer = Buffer::immutable(
            ctx,
            BufferType::VertexBuffer,
            &screen_mesh.vertices
        );

        let screen_index_buffer = Buffer::immutable(
            ctx,
            BufferType::IndexBuffer,
            &screen_mesh.indices
        );

        let mask_img = image::load_from_memory(IMAGE_BYTES)
            .unwrap_or_else(|e| panic!("{}", e))
            .to_rgba8();
        let mask_img_bytes = &mask_img.as_raw()[..];

        let mask_texture= Texture::from_data_and_format(
            ctx,
            &mask_img_bytes,
            TextureParams {
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                filter: FilterMode::Linear,
                width: mask_img.width() as _,
                height: mask_img.height() as _,
                depth: 1
            },
            TextureKind::Texture2D
        );

        let mask_binding = Bindings {
            vertex_buffers: vec![mask_vertex_buffer.clone()],
            index_buffer: mask_index_buffer.clone(),
            images: vec![mask_texture]
        };

        let render_target_tex = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: 1600,
                height: 1200,
                format: TextureFormat::RGBA8,
                ..TextureParams::default()
            }
        );

        let screen_binding = Bindings {
            vertex_buffers: vec![screen_vertex_buffer.clone()],
            index_buffer: screen_index_buffer.clone(),
            images: vec![render_target_tex]
        };

        let offscreen_pass = RenderPass::new(
            ctx,
            render_target_tex,
            None
        );

        // I give up, we will just use a fullscreen quad
        #[rustfmt::skip]
            let verts: &[f32] = &[
            /* pos         uv */
            -1.0, -1.0,    0.0, 0.0,
            1.0,  1.0,    1.0, 1.0,
            -1.0,  1.0,    0.0, 1.0,
            1.0, -1.0,    1.0, 0.0,
        ];

        let vertex_buffer = Buffer::immutable(
            ctx,
            BufferType::VertexBuffer,
            &verts
        );

        let index_buffer = Buffer::immutable(
            ctx,
            BufferType::IndexBuffer,
            &[0, 1, 2, 0, 3, 1]
        );

        let (buffer_width, buffer_height) = get_buffer_dimensions(&handler);

        let mut context_data = RetroBlitContext {
            buffer_width,
            buffer_height,
            buffer_pixels: vec![0u8; buffer_width * buffer_height],
            colors: [0u8; 256 * 3],
            mouse_x: 0.0,
            mouse_y: 0.0
        };

        let mut handler = handler;
        handler.init(&mut context_data);

        let colors_texture = Texture::from_data_and_format(
            ctx,
            &context_data.colors,
            TextureParams {
                format: TextureFormat::RGB8,
                wrap: TextureWrap::Clamp,
                filter: FilterMode::Nearest,
                width: 256,
                height: 1,
                depth: 1
            },
            TextureKind::Texture2D
        );

        let buffer_texture = Texture::from_data_and_format(
            ctx,
            &context_data.buffer_pixels,
            TextureParams {
                format: TextureFormat::Alpha,
                wrap: TextureWrap::Clamp,
                filter: FilterMode::Nearest,
                width: buffer_width as _,
                height: buffer_height as _,
                depth: 1
            },
            TextureKind::Texture2D
        );

        let offscreen_binding = Bindings {
            vertex_buffers: vec![vertex_buffer.clone()],
            index_buffer: index_buffer.clone(),
            images: vec![colors_texture, buffer_texture]
        };

        let shader = Shader::new(
            ctx,
            offscreen_shader::VERTEX,
            offscreen_shader::FRAGMENT,
            offscreen_shader::meta()
        ).unwrap(); // crash if failed to create a shader

        let offscreen_pipeline = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float2),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader
        );

        let shader = Shader::new(
            ctx,
            mask_shader::VERTEX,
            mask_shader::FRAGMENT,
            mask_shader::meta()
        ).unwrap(); // crash if failed to create a shader

        let mask_pipeline = Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float4),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader,
            PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha))
                ),
                alpha_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Zero,
                    BlendFactor::One)
                ),
                ..Default::default()
            }
        );

        let shader = Shader::new(
            ctx,
            screen_shader::VERTEX,
            screen_shader::FRAGMENT,
            screen_shader::meta()
        ).unwrap(); // crash if failed to create a shader

        let screen_pipeline = Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[
                VertexAttribute::new("pos", VertexFormat::Float4),
                VertexAttribute::new("uv", VertexFormat::Float2),
            ],
            shader,
            PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha))
                ),
                alpha_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Zero,
                    BlendFactor::One)
                ),
                ..Default::default()
            }
        );

        Self {
            mask_vertices_count,
            screen_vertices_count,
            mask_pipeline,
            mask_binding,
            screen_mesh,
            screen_pipeline,
            screen_binding,
            offscreen_pipeline,
            offscreen_binding,
            offscreen_pass,
            context_data,
            handler,
            buffer_texture,
            colors_texture
        }
    }
}

impl<CtxHandler: ContextHandler> EventHandler for Stage<CtxHandler> {
    fn update(&mut self, ctx: &mut Context) {
        self.handler.update(&mut self.context_data);
        self.colors_texture.update(ctx, &self.context_data.colors);
        self.buffer_texture.update(ctx, &self.context_data.buffer_pixels);
    }

    fn draw(&mut self, ctx: &mut Context) {
        { // render out color buffer into offscreen texture
            ctx.begin_pass(
                self.offscreen_pass,
                PassAction::clear_color(0.0, 0.0, 0.0, 1.0)
            );

            ctx.apply_pipeline(&self.offscreen_pipeline);
            ctx.apply_bindings(&self.offscreen_binding);
            ctx.draw(0, 6, 1);

            ctx.end_render_pass();
        }

        let aspect = ctx.screen_size().1 / ctx.screen_size().0;

        ctx.begin_default_pass(PassAction::clear_color(0.0, 0.0, 0.0, 1.0));
        { // render a screen
            ctx.apply_pipeline(&self.screen_pipeline);
            ctx.apply_bindings(&self.screen_binding);
            ctx.apply_uniforms(&screen_shader::Uniforms{ aspect });
            ctx.draw(0, self.screen_vertices_count as _, 1);
        }

        { // render a mask on the top of the screen
            ctx.apply_pipeline(&self.mask_pipeline);
            ctx.apply_bindings(&self.mask_binding);
            ctx.apply_uniforms(&mask_shader::Uniforms{ aspect });
            ctx.draw(0, self.mask_vertices_count as _, 1);
        }
        ctx.end_render_pass();

        ctx.commit_frame();
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        let screen_size = ctx.screen_size();
        let aspect = screen_size.0 / screen_size.1;

        let x = ((x / screen_size.0 - 0.5) * 2.0 * aspect).clamp(-1.0, 1.0);
        let y = -((y / screen_size.1 - 0.5) * 2.0).clamp(-1.0, 1.0);

        self.check_for_hit_test(x, y);
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        self.mouse_motion_event(ctx, x, y);
        match button {
            MouseButton::Left => { self.handler.on_mouse_down(&mut self.context_data, 0); },
            MouseButton::Middle => { self.handler.on_mouse_down(&mut self.context_data, 1); },
            MouseButton::Right => { self.handler.on_mouse_down(&mut self.context_data, 2); },
            _ => {}
        }
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        self.mouse_motion_event(ctx, x, y);
        match button {
            MouseButton::Left => { self.handler.on_mouse_up(&mut self.context_data, 0); },
            MouseButton::Middle => { self.handler.on_mouse_up(&mut self.context_data, 1); },
            MouseButton::Right => { self.handler.on_mouse_up(&mut self.context_data, 2); },
            _ => {}
        }
    }
}

impl<CtxHandler: ContextHandler> Stage<CtxHandler> {
    fn check_for_hit_test(&mut self, x: f32, y: f32) {
        let pt = Vec4 {x, y, z: 0.0, w: 1.0 };

        let mut offset = 0;
        while offset < self.screen_mesh.indices.len() {
            let vert0 = self.screen_mesh.vertices[self.screen_mesh.indices[offset] as usize];
            let vert1 = self.screen_mesh.vertices[self.screen_mesh.indices[offset + 1] as usize];
            let vert2 = self.screen_mesh.vertices[self.screen_mesh.indices[offset + 2] as usize];

            let hit_test = Vec4::get_barycentric_2d(
                pt,
                [
                    vert0.position,
                    vert1.position,
                    vert2.position
                ]
            );

            match hit_test {
                None => {}
                Some([bar_u, bar_v, bar_w]) => {
                    let u = bar_u * vert0.uv.x + bar_v * vert1.uv.x + bar_w * vert2.uv.x;
                    let v = 1.0 - (bar_u * vert0.uv.y + bar_v * vert1.uv.y + bar_w * vert2.uv.y);
                    self.context_data.mouse_x = u * self.context_data.buffer_width as f32;
                    self.context_data.mouse_y = v * self.context_data.buffer_height as f32;
                    return;
                }
            }
            offset += 3;
        }
    }
}

mod offscreen_shader {
    use orom_miniquad::*;

    pub const VERTEX:&str = r#"#version 100
        attribute vec2 pos;
        attribute vec2 uv;

        varying lowp vec2 texcoord;

        void main() {
            gl_Position = vec4(pos, 0.0, 1.0);
            texcoord = uv;
        }
    "#;

    pub const FRAGMENT:&str = r#"#version 100
        varying lowp vec2 texcoord;

        uniform sampler2D colors;
        uniform sampler2D tex;

        lowp vec3 fetch(lowp vec2 texcoord) {
            lowp float idx = texture2D(tex, texcoord).x;
            lowp vec2 uv = vec2(idx, 0.0);
            return texture2D(colors, uv).xyz;
        }

        void main() {
            gl_FragColor = vec4(fetch(texcoord), 1.0);
        }
    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["colors".to_string(), "tex".to_string()],
            uniforms: UniformBlockLayout { uniforms: Vec::new() }
        }
    }
}

mod mask_shader {
    use orom_miniquad::*;

    pub const VERTEX:&str = r#"#version 100
        attribute vec4 pos;
        attribute vec2 uv;

        varying lowp vec2 texcoord;
        uniform float aspect;

        void main() {
            gl_Position = vec4(pos.x * aspect, pos.yzw);
            texcoord = vec2(uv.x, 1.0 - uv.y);
        }
    "#;

    pub const FRAGMENT:&str = r#"#version 100
        varying lowp vec2 texcoord;

        uniform sampler2D tex;

        void main() {
            gl_FragColor = texture2D(tex, texcoord);
        }
    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("aspect", UniformType::Float1)]
            }
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub aspect: f32
    }
}

mod screen_shader {
    use orom_miniquad::*;

    pub const VERTEX:&str = r#"#version 100
        attribute vec4 pos;
        attribute vec2 uv;

        varying lowp vec2 texcoord;
        uniform float aspect;

        void main() {
            gl_Position = vec4(pos.x * aspect, pos.yzw);
            texcoord = vec2(uv.x, 1.0 - uv.y);
        }
    "#;

    pub const FRAGMENT:&str = r#"#version 100
        varying lowp vec2 texcoord;

        uniform sampler2D tex;

        void main() {
            gl_FragColor = vec4(texture2D(tex, texcoord).rgb, 1.0);
        }
    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec!["tex".to_string()],
            uniforms: UniformBlockLayout {
                uniforms: vec![UniformDesc::new("aspect", UniformType::Float1)]
            }
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub aspect: f32
    }
}

#[derive(Copy, Clone)]
pub enum WindowMode {
    Mode13,
    ModeX
}
impl WindowMode {
    fn get_buffer_dimensions(&self) -> (usize, usize) {
        match self {
            WindowMode::Mode13 => (320, 200),
            WindowMode::ModeX => (320, 240)
        }
    }
}

pub fn start<CtxHandler: 'static + ContextHandler>(handler: CtxHandler) {
    let conf = conf::Conf {
        window_title: handler.get_window_title().to_string(),
        window_width: 1024,
        window_height: 1024,
        high_dpi: true,
        fullscreen: false,
        sample_count: 6,
        window_resizable: true
    };

    orom_miniquad::start(conf, |mut ctx| {
        UserData::owning(Stage::new(&mut ctx, handler), ctx)
    });
}