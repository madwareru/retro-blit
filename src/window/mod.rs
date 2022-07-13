use std::collections::HashSet;
use std::convert::TryFrom;
use std::time::Instant;
use orom_miniquad::*;

pub mod monitor_obj_loader;
use monitor_obj_loader::Vec4;
use crate::audio::{SoundDriver};
use crate::rendering::blittable::{BufferProviderMut, SizedSurface};
use crate::math_utils::Barycentric2D;
use crate::window::monitor_obj_loader::Mesh;

const IMAGE_BYTES: &[u8] = include_bytes!("monitor_mask.png");

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum KeyCode {
    Space,
    Apostrophe,
    Comma,
    Minus,
    Period,
    Slash,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Semicolon,
    Equal,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,
    Backslash,
    RightBracket,
    GraveAccent,
    World1,
    World2,
    Escape,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDecimal,
    KpDivide,
    KpMultiply,
    KpSubtract,
    KpAdd,
    KpEnter,
    KpEqual,
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    Menu
}
pub enum KeyMod {
    Shift,
    Control,
    Option,
    Command
}

#[derive(Copy, Clone, Debug)]
pub struct KeyMods {
    pub shift: bool,
    pub control: bool,
    pub option: bool,
    pub command: bool,
}

impl TryFrom<orom_miniquad::KeyCode> for KeyCode {
    type Error = ();

    fn try_from(value: orom_miniquad::KeyCode) -> Result<Self, Self::Error> {
        match value {
            orom_miniquad::KeyCode::A => Ok(KeyCode::A),
            orom_miniquad::KeyCode::B => Ok(KeyCode::B),
            orom_miniquad::KeyCode::C => Ok(KeyCode::C),
            orom_miniquad::KeyCode::D => Ok(KeyCode::D),
            orom_miniquad::KeyCode::E => Ok(KeyCode::E),
            orom_miniquad::KeyCode::F => Ok(KeyCode::F),
            orom_miniquad::KeyCode::G => Ok(KeyCode::G),
            orom_miniquad::KeyCode::H => Ok(KeyCode::H),
            orom_miniquad::KeyCode::I => Ok(KeyCode::I),
            orom_miniquad::KeyCode::J => Ok(KeyCode::J),
            orom_miniquad::KeyCode::K => Ok(KeyCode::K),
            orom_miniquad::KeyCode::L => Ok(KeyCode::L),
            orom_miniquad::KeyCode::M => Ok(KeyCode::M),
            orom_miniquad::KeyCode::N => Ok(KeyCode::N),
            orom_miniquad::KeyCode::O => Ok(KeyCode::O),
            orom_miniquad::KeyCode::P => Ok(KeyCode::P),
            orom_miniquad::KeyCode::Q => Ok(KeyCode::Q),
            orom_miniquad::KeyCode::R => Ok(KeyCode::R),
            orom_miniquad::KeyCode::S => Ok(KeyCode::S),
            orom_miniquad::KeyCode::T => Ok(KeyCode::T),
            orom_miniquad::KeyCode::U => Ok(KeyCode::U),
            orom_miniquad::KeyCode::V => Ok(KeyCode::V),
            orom_miniquad::KeyCode::W => Ok(KeyCode::W),
            orom_miniquad::KeyCode::X => Ok(KeyCode::X),
            orom_miniquad::KeyCode::Y => Ok(KeyCode::Y),
            orom_miniquad::KeyCode::Z => Ok(KeyCode::Z),
            orom_miniquad::KeyCode::Space => Ok(KeyCode::Space),
            orom_miniquad::KeyCode::Apostrophe => Ok(KeyCode::Apostrophe),
            orom_miniquad::KeyCode::Comma => Ok(KeyCode::Comma),
            orom_miniquad::KeyCode::Minus => Ok(KeyCode::Minus),
            orom_miniquad::KeyCode::Period => Ok(KeyCode::Period),
            orom_miniquad::KeyCode::Slash => Ok(KeyCode::Slash),
            orom_miniquad::KeyCode::Key0 => Ok(KeyCode::Key0),
            orom_miniquad::KeyCode::Key1 => Ok(KeyCode::Key1),
            orom_miniquad::KeyCode::Key2 => Ok(KeyCode::Key2),
            orom_miniquad::KeyCode::Key3 => Ok(KeyCode::Key3),
            orom_miniquad::KeyCode::Key4 => Ok(KeyCode::Key4),
            orom_miniquad::KeyCode::Key5 => Ok(KeyCode::Key5),
            orom_miniquad::KeyCode::Key6 => Ok(KeyCode::Key6),
            orom_miniquad::KeyCode::Key7 => Ok(KeyCode::Key7),
            orom_miniquad::KeyCode::Key8 => Ok(KeyCode::Key8),
            orom_miniquad::KeyCode::Key9 => Ok(KeyCode::Key9),
            orom_miniquad::KeyCode::Semicolon => Ok(KeyCode::Semicolon),
            orom_miniquad::KeyCode::Equal => Ok(KeyCode::Equal),
            orom_miniquad::KeyCode::LeftBracket => Ok(KeyCode::LeftBracket),
            orom_miniquad::KeyCode::Backslash => Ok(KeyCode::Backslash),
            orom_miniquad::KeyCode::RightBracket => Ok(KeyCode::RightBracket),
            orom_miniquad::KeyCode::GraveAccent => Ok(KeyCode::GraveAccent),
            orom_miniquad::KeyCode::World1 => Ok(KeyCode::World1),
            orom_miniquad::KeyCode::World2 => Ok(KeyCode::World2),
            orom_miniquad::KeyCode::Escape => Ok(KeyCode::Escape),
            orom_miniquad::KeyCode::Enter => Ok(KeyCode::Enter),
            orom_miniquad::KeyCode::Tab => Ok(KeyCode::Tab),
            orom_miniquad::KeyCode::Backspace => Ok(KeyCode::Backspace),
            orom_miniquad::KeyCode::Insert => Ok(KeyCode::Insert),
            orom_miniquad::KeyCode::Delete => Ok(KeyCode::Delete),
            orom_miniquad::KeyCode::Right => Ok(KeyCode::Right),
            orom_miniquad::KeyCode::Left => Ok(KeyCode::Left),
            orom_miniquad::KeyCode::Down => Ok(KeyCode::Down),
            orom_miniquad::KeyCode::Up => Ok(KeyCode::Up),
            orom_miniquad::KeyCode::PageUp => Ok(KeyCode::PageUp),
            orom_miniquad::KeyCode::PageDown => Ok(KeyCode::PageDown),
            orom_miniquad::KeyCode::Home => Ok(KeyCode::Home),
            orom_miniquad::KeyCode::End => Ok(KeyCode::End),
            orom_miniquad::KeyCode::CapsLock => Ok(KeyCode::CapsLock),
            orom_miniquad::KeyCode::ScrollLock => Ok(KeyCode::ScrollLock),
            orom_miniquad::KeyCode::NumLock => Ok(KeyCode::NumLock),
            orom_miniquad::KeyCode::PrintScreen => Ok(KeyCode::PrintScreen),
            orom_miniquad::KeyCode::Pause => Ok(KeyCode::Pause),
            orom_miniquad::KeyCode::F1 => Ok(KeyCode::F1),
            orom_miniquad::KeyCode::F2 => Ok(KeyCode::F2),
            orom_miniquad::KeyCode::F3 => Ok(KeyCode::F3),
            orom_miniquad::KeyCode::F4 => Ok(KeyCode::F4),
            orom_miniquad::KeyCode::F5 => Ok(KeyCode::F5),
            orom_miniquad::KeyCode::F6 => Ok(KeyCode::F6),
            orom_miniquad::KeyCode::F7 => Ok(KeyCode::F7),
            orom_miniquad::KeyCode::F8 => Ok(KeyCode::F8),
            orom_miniquad::KeyCode::F9 => Ok(KeyCode::F9),
            orom_miniquad::KeyCode::F10 => Ok(KeyCode::F10),
            orom_miniquad::KeyCode::F11 => Ok(KeyCode::F11),
            orom_miniquad::KeyCode::F12 => Ok(KeyCode::F12),
            orom_miniquad::KeyCode::F13 => Ok(KeyCode::F13),
            orom_miniquad::KeyCode::F14 => Ok(KeyCode::F14),
            orom_miniquad::KeyCode::F15 => Ok(KeyCode::F15),
            orom_miniquad::KeyCode::F16 => Ok(KeyCode::F16),
            orom_miniquad::KeyCode::F17 => Ok(KeyCode::F17),
            orom_miniquad::KeyCode::F18 => Ok(KeyCode::F18),
            orom_miniquad::KeyCode::F19 => Ok(KeyCode::F19),
            orom_miniquad::KeyCode::F20 => Ok(KeyCode::F20),
            orom_miniquad::KeyCode::F21 => Ok(KeyCode::F21),
            orom_miniquad::KeyCode::F22 => Ok(KeyCode::F22),
            orom_miniquad::KeyCode::F23 => Ok(KeyCode::F23),
            orom_miniquad::KeyCode::F24 => Ok(KeyCode::F24),
            orom_miniquad::KeyCode::F25 => Ok(KeyCode::F25),
            orom_miniquad::KeyCode::Kp0 => Ok(KeyCode::Kp0),
            orom_miniquad::KeyCode::Kp1 => Ok(KeyCode::Kp1),
            orom_miniquad::KeyCode::Kp2 => Ok(KeyCode::Kp2),
            orom_miniquad::KeyCode::Kp3 => Ok(KeyCode::Kp3),
            orom_miniquad::KeyCode::Kp4 => Ok(KeyCode::Kp4),
            orom_miniquad::KeyCode::Kp5 => Ok(KeyCode::Kp5),
            orom_miniquad::KeyCode::Kp6 => Ok(KeyCode::Kp6),
            orom_miniquad::KeyCode::Kp7 => Ok(KeyCode::Kp7),
            orom_miniquad::KeyCode::Kp8 => Ok(KeyCode::Kp8),
            orom_miniquad::KeyCode::Kp9 => Ok(KeyCode::Kp9),
            orom_miniquad::KeyCode::KpDecimal => Ok(KeyCode::KpDecimal),
            orom_miniquad::KeyCode::KpDivide => Ok(KeyCode::KpDivide),
            orom_miniquad::KeyCode::KpMultiply => Ok(KeyCode::KpMultiply),
            orom_miniquad::KeyCode::KpSubtract => Ok(KeyCode::KpSubtract),
            orom_miniquad::KeyCode::KpAdd => Ok(KeyCode::KpAdd),
            orom_miniquad::KeyCode::KpEnter => Ok(KeyCode::KpEnter),
            orom_miniquad::KeyCode::KpEqual => Ok(KeyCode::KpEqual),
            orom_miniquad::KeyCode::LeftShift => Ok(KeyCode::LeftShift),
            orom_miniquad::KeyCode::LeftControl => Ok(KeyCode::LeftControl),
            orom_miniquad::KeyCode::LeftAlt => Ok(KeyCode::LeftAlt),
            orom_miniquad::KeyCode::LeftSuper => Ok(KeyCode::LeftSuper),
            orom_miniquad::KeyCode::RightShift => Ok(KeyCode::RightShift),
            orom_miniquad::KeyCode::RightControl => Ok(KeyCode::RightControl),
            orom_miniquad::KeyCode::RightAlt => Ok(KeyCode::RightAlt),
            orom_miniquad::KeyCode::RightSuper => Ok(KeyCode::RightSuper),
            orom_miniquad::KeyCode::Menu => Ok(KeyCode::Menu),
            orom_miniquad::KeyCode::Unknown => Err(()),
        }
    }
}

pub struct RetroBlitContext {
    sound_driver: Option<SoundDriver>,
    buffer_width: usize,
    buffer_height: usize,
    colors: [u8; 256 * 3],
    buffer_pixels: Vec<u8>,
    mouse_x: f32,
    mouse_y: f32,
    keys_pressed: HashSet<KeyCode>,
    key_mods_pressed: KeyMods
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

    fn init_audio(&mut self) {
        let sound_driver = SoundDriver::try_create();
        match sound_driver {
            Ok(driver) => {
                self.sound_driver = Some(driver);
            },
            Err(error) => {
                println!("Failed to init audio: {}", &error);
            }
        }
    }

    pub fn borrow_sound_driver(&mut self) -> Option<&mut SoundDriver> {
        match &mut (self.sound_driver) {
            Some(driver) => {
                Some(driver)
            },
            _ => {
                None
            }
        }
    }

    pub fn put_pixel(&mut self, x: i16, y: i16, color: u8) {
        if (0..self.buffer_width as i16).contains(&x) && (0..self.buffer_height as i16).contains(&y) {
            let idx = y as usize * self.buffer_width + x as usize;
            self.get_buffer_mut()[idx] = color;
        }
    }

    pub fn clear(&mut self, color_idx: u8) {
        for pixel in self.buffer_pixels.iter_mut() {
            *pixel = color_idx;
        }
    }

    pub fn is_key_mod_pressed(&self, key_mod: KeyMod) -> bool {
        match key_mod {
            KeyMod::Shift => self.key_mods_pressed.shift,
            KeyMod::Control => self.key_mods_pressed.control,
            KeyMod::Option => self.key_mods_pressed.option,
            KeyMod::Command => self.key_mods_pressed.command
        }
    }

    pub fn is_key_pressed(&self, key_code: KeyCode) -> bool {
        self.keys_pressed.contains(&key_code)
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
    fn on_mouse_down(&mut self, _ctx: &mut RetroBlitContext, _button_number: u8){}
    fn on_mouse_up(&mut self, _ctx: &mut RetroBlitContext, _button_number: u8){}
    fn on_key_down(&mut self, _ctx: &mut RetroBlitContext, _key_code: KeyCode, _key_mods: KeyMods){}
    fn on_key_up(&mut self, _ctx: &mut RetroBlitContext, _key_code: KeyCode, _key_mods: KeyMods){}
    fn init(&mut self, ctx: &mut RetroBlitContext);
    fn update(&mut self, ctx: &mut RetroBlitContext, dt: f32);
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
    colors_texture: Texture,
    last_instant: Instant
}

impl<CtxHandler: ContextHandler> Stage<CtxHandler> {
    pub fn new(ctx: &mut Context, handler: CtxHandler) -> Stage<CtxHandler> {
        let (mask_mesh, screen_mesh) = match handler.get_window_mode() {
            WindowMode::ModeX | WindowMode::Mode13 => {
                // it's okay to crash here since we can't do anything useful without monitor models
                // And still it will print a meaningful message, so we leave it like this
                let monitor_models = monitor_obj_loader::Mesh::load_meshes().unwrap();
                let mut mask_mesh = monitor_models.get("mask").unwrap().clone();
                let mut screen_mesh = monitor_models.get("screen").unwrap().clone();

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
                (mask_mesh, screen_mesh)
            },
            WindowMode::Mode13Frameless | WindowMode::ModeXFrameless => (
                Mesh::make_empty(),
                Mesh::make_3x4()
            ),
            WindowMode::Mode64x64 | WindowMode::Mode128x128 | WindowMode::Mode256x256 => (
                Mesh::make_empty(),
                Mesh::make_square()
            )
        };

        let mask_vertices_count = mask_mesh.vertices.len();
        let screen_vertices_count = screen_mesh.vertices.len();

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

        let (rtw, rth) = handler.get_window_mode().get_render_texture_dimensions();

        let render_target_tex = Texture::new_render_texture(
            ctx,
            TextureParams {
                width: rtw as _,
                height: rth as _,
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
            sound_driver: None,
            buffer_width,
            buffer_height,
            buffer_pixels: vec![0u8; buffer_width * buffer_height],
            colors: [0u8; 256 * 3],
            mouse_x: 0.0,
            mouse_y: 0.0,
            keys_pressed: HashSet::new(),
            key_mods_pressed: KeyMods {
                shift: false,
                control: false,
                option: false,
                command: false
            }
        };
        context_data.init_audio();

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
            colors_texture,
            last_instant: Instant::now()
        }
    }
}

impl<CtxHandler: ContextHandler> EventHandler for Stage<CtxHandler> {
    fn update(&mut self, ctx: &mut Context) {
        let dt = self.last_instant.elapsed().as_micros() as f32 / 1000000.0;
        self.last_instant = Instant::now();
        if let Some(driver) = &mut self.context_data.sound_driver {
            driver.maintain();
        }
        self.handler.update(&mut self.context_data, dt);
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

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: orom_miniquad::KeyCode,
        keymods: orom_miniquad::KeyMods,
        _repeat: bool,
    ) {
        let new_key_mods = KeyMods {
            shift: keymods.shift,
            option: keymods.alt,
            control: keymods.ctrl,
            command: keymods.logo
        };
        self.context_data.key_mods_pressed = new_key_mods;
        if let Ok(key_code) = KeyCode::try_from(keycode) {
            self.context_data.keys_pressed.insert(key_code);
            self.handler.on_key_down(
                &mut self.context_data,
                key_code,
                new_key_mods
            );
        }
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        keycode: orom_miniquad::KeyCode,
        keymods: orom_miniquad::KeyMods
    ) {
        let new_key_mods = KeyMods {
            shift: keymods.shift,
            option: keymods.alt,
            control: keymods.ctrl,
            command: keymods.logo
        };
        self.context_data.key_mods_pressed = new_key_mods;
        if let Ok(key_code) = KeyCode::try_from(keycode) {
            self.context_data.keys_pressed.remove(&key_code);
            self.handler.on_key_up(
                &mut self.context_data,
                key_code,
                new_key_mods
            );
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
    ModeX,
    Mode13Frameless,
    ModeXFrameless,
    Mode64x64,
    Mode128x128,
    Mode256x256
}
impl WindowMode {
    fn get_render_texture_dimensions(&self) -> (usize, usize) {
        match self {
            WindowMode::Mode13 => (1600, 1200),
            WindowMode::ModeX => (1600, 1200),
            WindowMode::Mode13Frameless => (1600, 1200),
            WindowMode::ModeXFrameless => (1600, 1200),
            WindowMode::Mode64x64 => (2048, 2048),
            WindowMode::Mode128x128 => (2048, 2048),
            WindowMode::Mode256x256 => (2048, 2048),
        }
    }

    fn get_buffer_dimensions(&self) -> (usize, usize) {
        match self {
            WindowMode::Mode13 => (320, 200),
            WindowMode::ModeX => (320, 240),
            WindowMode::Mode13Frameless => (320, 200),
            WindowMode::ModeXFrameless => (320, 240),
            WindowMode::Mode64x64 => (64, 64),
            WindowMode::Mode128x128 => (128, 128),
            WindowMode::Mode256x256 => (256, 256),
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