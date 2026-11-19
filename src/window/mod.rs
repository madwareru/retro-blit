use std::collections::HashSet;
use std::convert::TryFrom;

use crate::audio::SoundDriver;
use crate::rendering::blittable::{BufferProviderMut, Rect, SizedSurface};

mod app;
mod egui_integration;
pub mod monitor_obj_loader;
mod renderer;
mod texture_upload;

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
    Menu,
}

pub enum KeyMod {
    Shift,
    Control,
    Option,
    Command,
}

#[derive(Copy, Clone, Debug)]
pub struct KeyMods {
    pub shift: bool,
    pub control: bool,
    pub option: bool,
    pub command: bool,
}

impl TryFrom<winit::keyboard::KeyCode> for KeyCode {
    type Error = ();

    fn try_from(value: winit::keyboard::KeyCode) -> Result<Self, Self::Error> {
        use winit::keyboard::KeyCode as Winit;
        match value {
            Winit::Space => Ok(KeyCode::Space),
            Winit::Quote => Ok(KeyCode::Apostrophe),
            Winit::Comma => Ok(KeyCode::Comma),
            Winit::Minus => Ok(KeyCode::Minus),
            Winit::Period => Ok(KeyCode::Period),
            Winit::Slash => Ok(KeyCode::Slash),
            Winit::Digit0 => Ok(KeyCode::Key0),
            Winit::Digit1 => Ok(KeyCode::Key1),
            Winit::Digit2 => Ok(KeyCode::Key2),
            Winit::Digit3 => Ok(KeyCode::Key3),
            Winit::Digit4 => Ok(KeyCode::Key4),
            Winit::Digit5 => Ok(KeyCode::Key5),
            Winit::Digit6 => Ok(KeyCode::Key6),
            Winit::Digit7 => Ok(KeyCode::Key7),
            Winit::Digit8 => Ok(KeyCode::Key8),
            Winit::Digit9 => Ok(KeyCode::Key9),
            Winit::Semicolon => Ok(KeyCode::Semicolon),
            Winit::Equal => Ok(KeyCode::Equal),
            Winit::KeyA => Ok(KeyCode::A),
            Winit::KeyB => Ok(KeyCode::B),
            Winit::KeyC => Ok(KeyCode::C),
            Winit::KeyD => Ok(KeyCode::D),
            Winit::KeyE => Ok(KeyCode::E),
            Winit::KeyF => Ok(KeyCode::F),
            Winit::KeyG => Ok(KeyCode::G),
            Winit::KeyH => Ok(KeyCode::H),
            Winit::KeyI => Ok(KeyCode::I),
            Winit::KeyJ => Ok(KeyCode::J),
            Winit::KeyK => Ok(KeyCode::K),
            Winit::KeyL => Ok(KeyCode::L),
            Winit::KeyM => Ok(KeyCode::M),
            Winit::KeyN => Ok(KeyCode::N),
            Winit::KeyO => Ok(KeyCode::O),
            Winit::KeyP => Ok(KeyCode::P),
            Winit::KeyQ => Ok(KeyCode::Q),
            Winit::KeyR => Ok(KeyCode::R),
            Winit::KeyS => Ok(KeyCode::S),
            Winit::KeyT => Ok(KeyCode::T),
            Winit::KeyU => Ok(KeyCode::U),
            Winit::KeyV => Ok(KeyCode::V),
            Winit::KeyW => Ok(KeyCode::W),
            Winit::KeyX => Ok(KeyCode::X),
            Winit::KeyY => Ok(KeyCode::Y),
            Winit::KeyZ => Ok(KeyCode::Z),
            Winit::BracketLeft => Ok(KeyCode::LeftBracket),
            Winit::Backslash => Ok(KeyCode::Backslash),
            Winit::BracketRight => Ok(KeyCode::RightBracket),
            Winit::Backquote => Ok(KeyCode::GraveAccent),
            Winit::Escape => Ok(KeyCode::Escape),
            Winit::Enter => Ok(KeyCode::Enter),
            Winit::Tab => Ok(KeyCode::Tab),
            Winit::Backspace => Ok(KeyCode::Backspace),
            Winit::Insert => Ok(KeyCode::Insert),
            Winit::Delete => Ok(KeyCode::Delete),
            Winit::ArrowRight => Ok(KeyCode::Right),
            Winit::ArrowLeft => Ok(KeyCode::Left),
            Winit::ArrowDown => Ok(KeyCode::Down),
            Winit::ArrowUp => Ok(KeyCode::Up),
            Winit::PageUp => Ok(KeyCode::PageUp),
            Winit::PageDown => Ok(KeyCode::PageDown),
            Winit::Home => Ok(KeyCode::Home),
            Winit::End => Ok(KeyCode::End),
            Winit::CapsLock => Ok(KeyCode::CapsLock),
            Winit::ScrollLock => Ok(KeyCode::ScrollLock),
            Winit::NumLock => Ok(KeyCode::NumLock),
            Winit::PrintScreen => Ok(KeyCode::PrintScreen),
            Winit::Pause => Ok(KeyCode::Pause),
            Winit::F1 => Ok(KeyCode::F1),
            Winit::F2 => Ok(KeyCode::F2),
            Winit::F3 => Ok(KeyCode::F3),
            Winit::F4 => Ok(KeyCode::F4),
            Winit::F5 => Ok(KeyCode::F5),
            Winit::F6 => Ok(KeyCode::F6),
            Winit::F7 => Ok(KeyCode::F7),
            Winit::F8 => Ok(KeyCode::F8),
            Winit::F9 => Ok(KeyCode::F9),
            Winit::F10 => Ok(KeyCode::F10),
            Winit::F11 => Ok(KeyCode::F11),
            Winit::F12 => Ok(KeyCode::F12),
            Winit::F13 => Ok(KeyCode::F13),
            Winit::F14 => Ok(KeyCode::F14),
            Winit::F15 => Ok(KeyCode::F15),
            Winit::F16 => Ok(KeyCode::F16),
            Winit::F17 => Ok(KeyCode::F17),
            Winit::F18 => Ok(KeyCode::F18),
            Winit::F19 => Ok(KeyCode::F19),
            Winit::F20 => Ok(KeyCode::F20),
            Winit::F21 => Ok(KeyCode::F21),
            Winit::F22 => Ok(KeyCode::F22),
            Winit::F23 => Ok(KeyCode::F23),
            Winit::F24 => Ok(KeyCode::F24),
            Winit::F25 => Ok(KeyCode::F25),
            Winit::Numpad0 => Ok(KeyCode::Kp0),
            Winit::Numpad1 => Ok(KeyCode::Kp1),
            Winit::Numpad2 => Ok(KeyCode::Kp2),
            Winit::Numpad3 => Ok(KeyCode::Kp3),
            Winit::Numpad4 => Ok(KeyCode::Kp4),
            Winit::Numpad5 => Ok(KeyCode::Kp5),
            Winit::Numpad6 => Ok(KeyCode::Kp6),
            Winit::Numpad7 => Ok(KeyCode::Kp7),
            Winit::Numpad8 => Ok(KeyCode::Kp8),
            Winit::Numpad9 => Ok(KeyCode::Kp9),
            Winit::NumpadDecimal => Ok(KeyCode::KpDecimal),
            Winit::NumpadDivide => Ok(KeyCode::KpDivide),
            Winit::NumpadMultiply => Ok(KeyCode::KpMultiply),
            Winit::NumpadSubtract => Ok(KeyCode::KpSubtract),
            Winit::NumpadAdd => Ok(KeyCode::KpAdd),
            Winit::NumpadEnter => Ok(KeyCode::KpEnter),
            Winit::NumpadEqual => Ok(KeyCode::KpEqual),
            Winit::ShiftLeft => Ok(KeyCode::LeftShift),
            Winit::ControlLeft => Ok(KeyCode::LeftControl),
            Winit::AltLeft => Ok(KeyCode::LeftAlt),
            Winit::SuperLeft => Ok(KeyCode::LeftSuper),
            Winit::ShiftRight => Ok(KeyCode::RightShift),
            Winit::ControlRight => Ok(KeyCode::RightControl),
            Winit::AltRight => Ok(KeyCode::RightAlt),
            Winit::SuperRight => Ok(KeyCode::RightSuper),
            Winit::ContextMenu => Ok(KeyCode::Menu),
            _ => Err(()),
        }
    }
}

pub struct RetroBlitContext {
    egui: egui::Context,
    sound_driver: Option<SoundDriver>,
    buffer_width: usize,
    buffer_height: usize,
    colors: [u8; 256 * 3],
    buffer_pixels: Vec<u8>,
    mouse_x: f32,
    mouse_y: f32,
    keys_pressed: HashSet<KeyCode>,
    key_mods_pressed: KeyMods,
    quit_fired: bool,
    cursor_hidden_fired: Option<bool>,
}

impl RetroBlitContext {
    fn new(handler: &impl ContextHandler) -> Self {
        let (buffer_width, buffer_height) = get_buffer_dimensions(handler);
        Self {
            egui: egui::Context::default(),
            sound_driver: None,
            buffer_width,
            buffer_height,
            colors: [0; 256 * 3],
            buffer_pixels: vec![0; buffer_width * buffer_height],
            mouse_x: 0.0,
            mouse_y: 0.0,
            keys_pressed: HashSet::new(),
            key_mods_pressed: KeyMods {
                shift: false,
                control: false,
                option: false,
                command: false,
            },
            quit_fired: false,
            cursor_hidden_fired: None,
        }
    }

    pub fn hide_cursor(&mut self) {
        self.cursor_hidden_fired = Some(true);
    }

    pub fn show_cursor(&mut self) {
        self.cursor_hidden_fired = Some(false);
    }

    fn init_audio(&mut self) {
        match SoundDriver::try_create() {
            Ok(driver) => self.sound_driver = Some(driver),
            Err(error) => println!("Failed to init audio: {}", &error),
        }
    }

    pub fn quit(&mut self) {
        self.quit_fired = true;
    }

    pub fn borrow_sound_driver(&mut self) -> Option<&mut SoundDriver> {
        self.sound_driver.as_mut()
    }

    pub fn put_pixel(&mut self, x: i16, y: i16, color: u8) {
        if (0..self.buffer_width as i16).contains(&x) && (0..self.buffer_height as i16).contains(&y)
        {
            let idx = y as usize * self.buffer_width + x as usize;
            self.get_buffer_mut()[idx] = color;
        }
    }

    pub fn clear(&mut self, color_idx: u8) {
        for pixel in &mut self.buffer_pixels {
            *pixel = color_idx;
        }
    }

    pub fn clip_rect(&mut self, rect: Rect, color_idx: u8) {
        for pixel in &mut self.buffer_pixels[..self.buffer_width * rect.y_range.start] {
            *pixel = color_idx;
        }
        for pixel in
            &mut self.buffer_pixels[self.buffer_width * (rect.y_range.start + rect.get_height())..]
        {
            *pixel = color_idx;
        }
        for j in 0..rect.get_height() {
            let stride = (rect.y_range.start + j) * self.buffer_width;
            for pixel in &mut self.buffer_pixels[stride..stride + rect.x_range.start] {
                *pixel = color_idx;
            }
            for pixel in &mut self.buffer_pixels
                [stride + rect.x_range.start + rect.get_width()..stride + self.buffer_width]
            {
                *pixel = color_idx;
            }
        }
    }

    pub fn is_egui_wants_keyboard_input(&self) -> bool {
        self.egui.egui_wants_keyboard_input()
    }

    pub fn is_egui_wants_pointer_input(&self) -> bool {
        self.egui.egui_wants_pointer_input()
    }

    pub fn is_egui_area_under_pointer(&self) -> bool {
        self.egui.is_pointer_over_egui()
    }

    pub fn get_egui_ctx(&self) -> egui::Context {
        self.egui.clone()
    }

    pub fn is_key_mod_pressed(&self, key_mod: KeyMod) -> bool {
        match key_mod {
            KeyMod::Shift => self.key_mods_pressed.shift,
            KeyMod::Control => self.key_mods_pressed.control,
            KeyMod::Option => self.key_mods_pressed.option,
            KeyMod::Command => self.key_mods_pressed.command,
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
        [
            self.colors[offset],
            self.colors[offset + 1],
            self.colors[offset + 2],
        ]
    }

    pub fn set_palette(&mut self, index: u8, new_value: [u8; 3]) {
        let offset = self.make_palette_offset(index as usize);
        self.colors[offset..offset + 3].copy_from_slice(&new_value);
    }

    pub fn scroll_palette(&mut self, scroll_kind: ScrollKind, scroll_direction: ScrollDirection) {
        let (start_idx, last_idx) = match scroll_kind {
            ScrollKind::AllPalette => (0, 255),
            ScrollKind::Range { start_idx, len } => (start_idx, start_idx + len - 1),
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
        }
    }

    #[inline(always)]
    fn make_palette_offset(&self, ix: usize) -> usize {
        ((ix % 256) * 3) % self.colors.len()
    }
}

impl SizedSurface for RetroBlitContext {
    fn get_width(&self) -> usize {
        self.buffer_width
    }

    fn get_height(&self) -> usize {
        self.buffer_height
    }
}

impl BufferProviderMut<u8> for RetroBlitContext {
    fn get_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer_pixels
    }
}

pub enum ScrollKind {
    AllPalette,
    Range { start_idx: u8, len: u8 },
}

pub enum ScrollDirection {
    Forward,
    Backward,
}

pub trait ContextHandler {
    fn get_window_title(&self) -> &'static str;
    fn get_window_mode(&self) -> WindowMode;
    fn on_mouse_down(&mut self, _ctx: &mut RetroBlitContext, _button_number: u8) {}
    fn on_mouse_up(&mut self, _ctx: &mut RetroBlitContext, _button_number: u8) {}
    fn on_key_down(&mut self, _ctx: &mut RetroBlitContext, _key_code: KeyCode, _key_mods: KeyMods) {
    }
    fn on_key_up(&mut self, _ctx: &mut RetroBlitContext, _key_code: KeyCode, _key_mods: KeyMods) {}
    fn init(&mut self, ctx: &mut RetroBlitContext);
    fn update(&mut self, ctx: &mut RetroBlitContext, dt: f32);
    fn egui(&mut self, _ctx: &mut RetroBlitContext, _egui_ctx: &egui::Context) {}
}

fn get_buffer_dimensions(handler: &impl ContextHandler) -> (usize, usize) {
    handler.get_window_mode().get_buffer_dimensions()
}

#[derive(Copy, Clone)]
pub enum WindowMode {
    Mode13,
    ModeX,
    Mode13Frameless,
    ModeXFrameless,
    Mode64x64,
    Mode128x128,
    Mode256x256,
    Mode160x120,
    Mode800x600,
    Mode240x150,
    Mode480x300,
    Mode640x400,
    Mode960x600,
}

impl WindowMode {
    fn get_render_texture_dimensions(&self) -> (usize, usize) {
        match self {
            WindowMode::Mode13 | WindowMode::ModeX => (1600, 1200),
            WindowMode::Mode13Frameless | WindowMode::ModeXFrameless => (1600, 1200),
            WindowMode::Mode64x64 | WindowMode::Mode128x128 | WindowMode::Mode256x256 => {
                (2048, 2048)
            }
            WindowMode::Mode160x120 | WindowMode::Mode800x600 => (1600, 1200),
            WindowMode::Mode240x150
            | WindowMode::Mode480x300
            | WindowMode::Mode640x400
            | WindowMode::Mode960x600 => (1920, 1200),
        }
    }

    fn get_buffer_dimensions(&self) -> (usize, usize) {
        match self {
            WindowMode::Mode13 | WindowMode::Mode13Frameless => (320, 200),
            WindowMode::ModeX | WindowMode::ModeXFrameless => (320, 240),
            WindowMode::Mode64x64 => (64, 64),
            WindowMode::Mode128x128 => (128, 128),
            WindowMode::Mode256x256 => (256, 256),
            WindowMode::Mode160x120 => (160, 120),
            WindowMode::Mode800x600 => (800, 600),
            WindowMode::Mode240x150 => (240, 150),
            WindowMode::Mode480x300 => (480, 300),
            WindowMode::Mode640x400 => (640, 400),
            WindowMode::Mode960x600 => (960, 600),
        }
    }
}

pub fn start<CtxHandler: 'static + ContextHandler>(handler: CtxHandler) {
    let (mut width, mut height) = handler.get_window_mode().get_buffer_dimensions();
    if width == 640 && height == 400 {
        width = 1920;
        height = 1200;
    } else {
        while height < 600 {
            width *= 2;
            height *= 2;
        }
    }
    app::run(handler, width as u32, height as u32);
}
