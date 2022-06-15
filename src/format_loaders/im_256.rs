use std::io::Read;
use bin_serialization_rs::{Endianness, Reflectable, SerializationReflector};

#[derive(Clone)]
pub struct Image {
    _signature: u16,
    palette_size: u16,
    width: u16,
    height: u16,
    palette: [u8; 256*3],
    color_buffer: Vec<u8>
}

impl Default for Image {
    fn default() -> Self {
        let _signature = (b'I' as u16) * 0x100 + b'M' as u16;
        let _nop = 0;
        Self {
            _signature,
            palette_size: Default::default(),
            width: Default::default(),
            height: Default::default(),
            palette: [0; 256*3],
            color_buffer: Vec::new()
        }
    }
}

impl Reflectable for Image {
    fn reflect<TSerializationReflector: SerializationReflector>(&mut self, reflector: &mut TSerializationReflector) -> std::io::Result<()> {
        reflector.reflect_u16(&mut self._signature)?;
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

impl Image {
    pub fn load_from(mut source: impl Read) -> std::io::Result<Self> {
        Image::deserialize(&mut source, Endianness::LittleEndian)
    }

    pub fn get_width(&self) -> usize {
        self.width as usize
    }

    pub fn get_height(&self) -> usize {
        self.height as usize
    }

    pub fn get_palette_size(&self) -> usize {
        self.palette_size as usize
    }

    pub fn get_palette(&self) -> &[u8] {
        &self.palette
    }

    pub fn get_buffer(&self) -> &[u8] {
        &self.color_buffer
    }

    pub fn get_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.color_buffer
    }
}