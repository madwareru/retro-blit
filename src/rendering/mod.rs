pub mod blittable;
pub mod fonts;

use crate::format_loaders::bmp_256::Bmp;
use crate::format_loaders::im_256::Image;
use blittable::{BlitBuilder, Blittable, SizedSurface};
use crate::rendering::blittable::{BufferProvider, BufferProviderMut};

#[derive(Clone)]
pub struct BlittableSurface {
    width: u16,
    height: u16,
    buffer: Vec<u8>
}

impl BlittableSurface {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            buffer: vec![0u8; width as usize * height as usize]
        }
    }

    pub fn with_color_key(&self, color_key: u8) -> ColorKeyWrapper {
        ColorKeyWrapper{
            wrapped: self,
            color_key
        }
    }

    pub fn with_color_key_blink(&self, color_key: u8, blink_color: u8) -> ColorKeyBlinkWrapper {
        ColorKeyBlinkWrapper{
            wrapped: self,
            color_key,
            blink_color
        }
    }
}

impl SizedSurface for BlittableSurface {
    fn get_width(&self) -> usize { self.width as _ }

    fn get_height(&self) -> usize { self.height as _ }
}

impl BufferProvider<u8> for BlittableSurface {
    fn get_buffer(&self) -> &[u8] {
        &self.buffer
    }
}

impl BufferProviderMut<u8> for BlittableSurface {
    fn get_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }
}

impl From<&Image> for BlittableSurface {
    fn from(img: &Image) -> Self {
        let width = img.get_width() as _;
        let height = img.get_height() as _;
        let buffer = img.get_buffer().iter().map(|it| *it).collect::<Vec<_>>();
        Self { width, height, buffer }
    }
}

impl From<&Bmp> for BlittableSurface {
    fn from(img: &Bmp) -> Self {
        let width = img.get_width() as _;
        let height = img.get_height() as _;
        let buffer = img.get_buffer().iter().map(|it| *it).collect::<Vec<_>>();
        Self { width, height, buffer }
    }
}

impl Blittable<u8> for BlittableSurface {}

pub struct ColorKeyWrapper<'a> {
    wrapped: &'a BlittableSurface,
    color_key: u8
}

impl SizedSurface for ColorKeyWrapper<'_> {
    fn get_width(&self) -> usize {
        self.wrapped.get_width()
    }

    fn get_height(&self) -> usize {
        self.wrapped.get_height()
    }
}

impl BufferProvider<u8> for ColorKeyWrapper<'_> {
    fn get_buffer(&self) -> &[u8] {
        self.wrapped.get_buffer()
    }
}

impl Blittable<u8> for ColorKeyWrapper<'_> {
    #[inline(always)]
    fn blend_function(&self, dst: &mut u8, src: &u8) {
        *dst = if *src == self.color_key { *dst} else {*src};
    }
}

pub struct ColorKeyBlinkWrapper<'a> {
    wrapped: &'a BlittableSurface,
    blink_color: u8,
    color_key: u8
}

impl SizedSurface for ColorKeyBlinkWrapper<'_> {
    fn get_width(&self) -> usize {
        self.wrapped.get_width()
    }

    fn get_height(&self) -> usize {
        self.wrapped.get_height()
    }
}

impl BufferProvider<u8> for ColorKeyBlinkWrapper<'_> {
    fn get_buffer(&self) -> &[u8] {
        self.wrapped.get_buffer()
    }
}

impl Blittable<u8> for ColorKeyBlinkWrapper<'_> {
    #[inline(always)]
    fn blend_function(&self, dst: &mut u8, src: &u8) {
        *dst = if *src == self.color_key { *dst} else {self.blink_color};
    }
}

impl<'a, TBlittable: Blittable<u8>> blittable::BlitDestination<'a, u8, TBlittable> for BlittableSurface {
    fn initiate_blit_on_self(&'a mut self, source_blittable: &'a TBlittable) -> BlitBuilder<'a, u8, TBlittable> {
        let width = self.get_width();
        BlitBuilder::create_ext(
            self.get_buffer_mut(),
            width,
            source_blittable
        )
    }
}

impl<'a, TBlittable: Blittable<u8>> blittable::BlitDestination<'a, u8, TBlittable> for crate::window::RetroBlitContext {
}