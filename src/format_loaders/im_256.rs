use std::io::Read;
use std::ops::Deref;
use thiserror::Error;
use bin_serialization_rs::{Endianness, Reflectable, SerializationReflector};
use crate::rendering::blittable::{SizedSurface};
use crate::rendering::BlittableSurface;

#[derive(Error, Debug)]
pub enum Im256LoadingError {
    #[error("IO error")]
    FailedToParseFloat(#[from] std::io::Error),
    #[error("Incorrect signature. 'IM' expected")]
    IncorrectSignature
}

#[derive(Clone)]
pub struct Image {
    palette_size: u16,
    width: u16,
    height: u16,
    palette: [u8; 256*3],
    color_buffer: Vec<u8>,
    color_key: Option<u8>
}

impl Default for Image {
    fn default() -> Self {
        Self {
            palette_size: Default::default(),
            width: Default::default(),
            height: Default::default(),
            palette: [0; 256*3],
            color_buffer: Vec::new(),
            color_key: None
        }
    }
}

impl Reflectable for Image {
    fn reflect<TSerializationReflector: SerializationReflector>(&mut self, reflector: &mut TSerializationReflector) -> std::io::Result<()> {
        reflector.reflect_u16(&mut self.palette_size)?;
        reflector.reflect_u16(&mut self.width)?;
        reflector.reflect_u16(&mut self.height)?;
        for i in 0..self.palette_size {
            let offset = i as usize * 3;
            reflector.reflect_u8(&mut self.palette[offset])?;
            reflector.reflect_u8(&mut self.palette[offset+1])?;
            reflector.reflect_u8(&mut self.palette[offset+2])?;
        }
        let wh = self.width as usize * self.height as usize;
        self.color_buffer.resize(wh, 0);
        for i in 0..wh {
            reflector.reflect_u8(&mut self.color_buffer[i])?;
        }
        Ok(())
    }
}

#[derive(Default, Clone, PartialEq)]
struct U8Wrapper(pub u8);
impl Reflectable for U8Wrapper {
    fn reflect<TSerializationReflector: SerializationReflector>(
        &mut self,
        reflector: &mut TSerializationReflector,
    ) -> std::io::Result<()> {
        reflector.reflect_u8(&mut self.0)
    }
}
impl AsRef<u8> for U8Wrapper {
    fn as_ref(&self) -> &u8 {
        &(self.0)
    }
}
impl Deref for U8Wrapper {
    type Target = u8;
    fn deref(&self) -> &Self::Target {
        &(self.0)
    }
}

impl Image {
    pub fn load_from(mut source: impl Read) -> Result<(Vec<[u8; 3]>, BlittableSurface), Im256LoadingError> {
        let signature_0 = U8Wrapper::deserialize(&mut source, Endianness::LittleEndian)?;
        let signature_1 = U8Wrapper::deserialize(&mut source, Endianness::LittleEndian)?;
        if [*signature_0, *signature_1] != [b'I', b'M'] {
            return Err(Im256LoadingError::IncorrectSignature);
        }
        let img = Image::deserialize(&mut source, Endianness::LittleEndian)?;
        let mut palette = Vec::with_capacity(img.palette_size as usize);
        for i in 0..img.palette_size as usize {
            let offset = i * 3;
            palette.push([img.palette[offset], img.palette[offset + 1], img.palette[offset + 2]]);
        }
        Ok((palette, BlittableSurface::from(&img)))
    }

    pub fn get_buffer(&self) -> &[u8] {
        &self.color_buffer
    }

    pub fn get_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.color_buffer
    }

    pub fn set_color_key(&mut self, color_key: Option<u8>) {
        self.color_key = color_key;
    }
}

impl SizedSurface for Image {
    fn get_width(&self) -> usize { self.width as _ }

    fn get_height(&self) -> usize { self.height as _ }
}