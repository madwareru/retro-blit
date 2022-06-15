use std::io::{Read, Seek, SeekFrom};
use std::ops::Deref;
use bin_serialization_rs::{Endianness, Reflectable, SerializationReflector};
use thiserror::Error;
use crate::rendering::blittable::{SizedSurface};

#[derive(Default, Debug, Clone)]
struct RawBmpHeader {
    pub width: u32,
    pub height: i32,
    _bi_planes: u16,
    pub bi_bit_count: u16,
    _bi_compression: u32,
    _bi_size_image: u32,
    _bi_x_pels_per_meter: u32,
    _bi_y_pels_per_meter: u32,
    _bi_clr_used: u32,
    _bi_clr_important: u32,
}
impl Reflectable for RawBmpHeader {
    fn reflect<TSerializationReflector: SerializationReflector>(
        &mut self, reflector: &mut TSerializationReflector
    ) -> std::io::Result<()> {
        reflector.reflect_u32(&mut self.width)?;
        reflector.reflect_i32(&mut self.height)?;
        reflector.reflect_u16(&mut self._bi_planes)?;
        reflector.reflect_u16(&mut self.bi_bit_count)?;
        reflector.reflect_u32(&mut self._bi_compression)?;
        reflector.reflect_u32(&mut self._bi_size_image)?;
        reflector.reflect_u32(&mut self._bi_x_pels_per_meter)?;
        reflector.reflect_u32(&mut self._bi_y_pels_per_meter)?;
        reflector.reflect_u32(&mut self._bi_clr_used)?;
        reflector.reflect_u32(&mut self._bi_clr_important)
    }
}

#[derive(Default, Clone, PartialEq)]
struct U32Wrapper(pub u32);
impl Reflectable for U32Wrapper {
    fn reflect<TSerializationReflector: SerializationReflector>(
        &mut self,
        reflector: &mut TSerializationReflector,
    ) -> std::io::Result<()> {
        reflector.reflect_u32(&mut self.0)
    }
}
impl AsRef<u32> for U32Wrapper {
    fn as_ref(&self) -> &u32 {
        &(self.0)
    }
}
impl Deref for U32Wrapper {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &(self.0)
    }
}

struct RawBmp {
    pub header: RawBmpHeader,
    pub palette: Option<[u32; 256]>, // Exists only for 8bit images
    pub scanline_padding: usize,
    pub raw_data: Vec<u8>
}
impl RawBmp {
    pub fn read_from<TStream: Read + Seek>(stream: &mut TStream) -> std::io::Result<Option<Self>> {
        let magic = &mut [0u8, 0u8];
        stream.read(magic)?;
        if magic != &[b'B', b'M'] {
            return Ok(None); // not a bmp file. Just return None in this case
        }
        stream.seek(SeekFrom::Current(8))?; // ignoring 8 unused bytes
        let bfh_pixel_data = *U32Wrapper::deserialize(stream, Endianness::LittleEndian)? as u64;
        let bi_version = *U32Wrapper::deserialize(stream, Endianness::LittleEndian)?;
        if bi_version != 40 {
            Ok(None)
        } else {
            let header = RawBmpHeader::deserialize(stream, Endianness::LittleEndian)?;
            let palette = if header.bi_bit_count == 8 {
                let mut arr = [0u32; 256];
                for arr_entry in arr.iter_mut() {
                    *arr_entry = *U32Wrapper::deserialize(stream, Endianness::LittleEndian)?;
                }
                Some(arr)
            } else {
                None
            };
            let scanline_size = header.width as usize * header.bi_bit_count as usize / 8;
            let remainder = scanline_size % 4;
            let scanline_padding = if remainder == 0 { 0 } else { 4 - remainder };
            let data_size = (scanline_size + scanline_padding) * header.height.abs() as usize;
            let mut raw_data = vec![0u8; data_size];
            stream.seek(SeekFrom::Start(bfh_pixel_data))?;
            stream.read(&mut raw_data)?;
            Ok(Some(Self {
                header,
                palette,
                scanline_padding,
                raw_data
            }))
        }
    }
}

#[derive(Error, Debug)]
pub enum BmpLoadingError {
    #[error("IO error")]
    FailedToParseFloat(#[from] std::io::Error),
    #[error("Unsupported bmp file")]
    FileTypeIsUnsupported
}

pub struct Bmp {
    width : u32,
    height : u32,
    palette: [u8; 256*3],
    buffer: Vec<u8>,
    color_key: Option<u8>
}

impl Bmp {
    pub fn read_from<TStream: Read + Seek>(stream: &mut TStream) -> Result<Self, BmpLoadingError> {
        let raw_bmp = RawBmp::read_from(stream)?;
        match raw_bmp {
            None => Err(BmpLoadingError::FileTypeIsUnsupported),
            Some(bmp) => {
                let upside_down = bmp.header.height > 0;
                let width = bmp.header.width as usize;
                let height = bmp.header.height.abs() as usize;
                match bmp.header.bi_bit_count {
                    8 => {
                        let mut palette_indexes = vec![0u8; bmp.raw_data.len()];
                        let mut d_offset = if upside_down { height * width - width } else { 0 };
                        let slide = width * 2;
                        let mut s_offset = 0;
                        for _ in 0..height {
                            for _ in 0..width {
                                palette_indexes[d_offset] = bmp.raw_data[s_offset];
                                s_offset += 1;
                                d_offset += 1;
                            }
                            s_offset += bmp.scanline_padding;
                            if !upside_down {
                                d_offset += width;
                                continue;
                            }
                            if d_offset >= slide { d_offset -= slide; }
                        }
                        if let Some(pal) = bmp.palette {
                            let mut palette = [0u8; 256*3];
                            let mut offset = 0;
                            for entry in pal.iter() {
                                let mut clr = *entry;
                                let b = clr & 0xFF; clr = clr / 0x100;
                                let g = clr & 0xFF; clr = clr / 0x100;
                                let r = clr & 0xFF;

                                palette[offset] = r as u8; offset += 1;
                                palette[offset] = g as u8; offset += 1;
                                palette[offset] = b as u8; offset += 1;
                            }
                            Ok(
                                Self {
                                    width: width as _,
                                    height: width as _,
                                    palette,
                                    buffer: palette_indexes,
                                    color_key: None
                                }
                            )
                        } else {
                            Err(BmpLoadingError::FileTypeIsUnsupported)
                        }
                    },
                    _ => Err(BmpLoadingError::FileTypeIsUnsupported)
                }
            }
        }
    }

    pub fn get_palette_size(&self) -> usize { 256 }

    pub fn get_palette(&self) -> &[u8] { &self.palette }

    pub fn get_buffer(&self) -> &[u8] { &self.buffer }

    pub fn get_buffer_mut(&mut self) -> &mut [u8] { &mut self.buffer }

    pub fn set_color_key(&mut self, color_key: Option<u8>) {
        self.color_key = color_key;
    }
}

impl SizedSurface for Bmp {
    fn get_width(&self) -> usize { self.width as _ }

    fn get_height(&self) -> usize { self.height as _ }
}