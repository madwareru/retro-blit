use crate::format_loaders::bmp_256::Bmp;
use crate::format_loaders::im_256::Image;

pub mod blittable;
use blittable::{BlitBuilder, Blittable, Rect, SizedSurface};
use crate::rendering::blittable::Flip;

#[derive(Clone)]
pub struct BlittableSurface {
    width: u16,
    height: u16,
    buffer: Vec<u8>,
    color_key: Option<u8>
}

impl BlittableSurface {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            buffer: vec![0u8; width as usize * height as usize],
            color_key: None
        }
    }

    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn get_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    pub fn get_color_keu(&self) -> Option<u8> { self.color_key }

    pub fn set_color_key(&mut self, color_key: Option<u8>) {
        self.color_key = color_key;
    }
}

impl SizedSurface for BlittableSurface {
    fn get_width(&self) -> usize { self.width as _ }

    fn get_height(&self) -> usize { self.height as _ }
}

impl From<&Image> for BlittableSurface {
    fn from(img: &Image) -> Self {
        let width = img.get_width() as _;
        let height = img.get_height() as _;
        let buffer = img.get_buffer().iter().map(|it| *it).collect::<Vec<_>>();
        let color_key = None;
        Self { width, height, buffer, color_key }
    }
}

impl From<&Bmp> for BlittableSurface {
    fn from(img: &Bmp) -> Self {
        let width = img.get_width() as _;
        let height = img.get_height() as _;
        let buffer = img.get_buffer().iter().map(|it| *it).collect::<Vec<_>>();
        let color_key = None;
        Self { width, height, buffer, color_key }
    }
}

impl Blittable<u8> for BlittableSurface {
    fn blit_impl(&self, buffer: &mut [u8], buffer_width: usize, self_rect: Rect, dst_rect: Rect, flip: Flip) {
        let src_rect = self_rect;
        let dst_rect = dst_rect;
        let span_length = (
            src_rect.x_range.end - src_rect.x_range.start
        ).min(
            dst_rect.x_range.end - dst_rect.x_range.start
        );
        let span_count = (
            src_rect.y_range.end - src_rect.y_range.start
        ).min(
            dst_rect.y_range.end - dst_rect.y_range.start
        );
        let width = self.get_width();
        let mut src_stride = src_rect.y_range.start * width + src_rect.x_range.start;
        let src_buffer = self.get_buffer();

        match self.color_key {
            None => {
                if let Flip::Vertically = flip {
                    let mut dst_stride = (dst_rect.y_range.start + span_count - 1) * buffer_width + dst_rect.x_range.start;
                    for _ in 0..span_count {
                        let zipped = (&mut buffer[dst_stride..dst_stride+span_length])
                            .iter_mut()
                            .zip(&src_buffer[src_stride..src_stride+span_length]);
                        for (dest, src) in zipped {
                            *dest = *src;
                        }
                        src_stride += width;
                        dst_stride -= buffer_width;
                    }
                } else {
                    let mut dst_stride = dst_rect.y_range.start * buffer_width + dst_rect.x_range.start;
                    if let Flip::Horizontally = flip {
                        for _ in 0..span_count {
                            let zipped = (&mut buffer[dst_stride..dst_stride+span_length])
                                .iter_mut()
                                .zip((&src_buffer[src_stride..src_stride+span_length]).iter().rev());
                            for (dest, src) in zipped {
                                *dest = *src;
                            }
                            src_stride += width;
                            dst_stride += buffer_width;
                        }
                    } else {
                        for _ in 0..span_count {
                            let zipped = (&mut buffer[dst_stride..dst_stride+span_length])
                                .iter_mut()
                                .zip(&src_buffer[src_stride..src_stride+span_length]);
                            for (dest, src) in zipped {
                                *dest = *src;
                            }
                            src_stride += width;
                            dst_stride += buffer_width;
                        }
                    }
                }
            },
            Some(color_key) => {
                if let Flip::Vertically = flip {
                    let mut dst_stride = (dst_rect.y_range.start + span_count - 1) * buffer_width + dst_rect.x_range.start;
                    for _ in 0..span_count {
                        let zipped = (&mut buffer[dst_stride..dst_stride+span_length])
                            .iter_mut()
                            .zip(&src_buffer[src_stride..src_stride+span_length]);
                        for (dest, src) in zipped {
                            *dest = if color_key == *src { *dest } else { *src };
                        }
                        src_stride += width;
                        dst_stride -= buffer_width;
                    }
                } else {
                    let mut dst_stride = dst_rect.y_range.start * buffer_width + dst_rect.x_range.start;
                    if let Flip::Horizontally = flip {
                        for _ in 0..span_count {
                            let zipped = (&mut buffer[dst_stride..dst_stride+span_length])
                                .iter_mut()
                                .zip((&src_buffer[src_stride..src_stride+span_length]).iter().rev());
                            for (dest, src) in zipped {
                                *dest = if color_key == *src { *dest } else { *src };
                            }
                            src_stride += width;
                            dst_stride += buffer_width;
                        }
                    } else {
                        for _ in 0..span_count {
                            let zipped = (&mut buffer[dst_stride..dst_stride+span_length])
                                .iter_mut()
                                .zip(&src_buffer[src_stride..src_stride+span_length]);
                            for (dest, src) in zipped {
                                *dest = if color_key == *src { *dest } else { *src };
                            }
                            src_stride += width;
                            dst_stride += buffer_width;
                        }
                    }
                }
            }
        }
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

impl<'a, TBlittable: Blittable<u8>> blittable::BlitDestination<'a, u8, TBlittable> for crate::window::ContextData {
    fn initiate_blit_on_self(&'a mut self, source_blittable: &'a TBlittable) -> BlitBuilder<'a, u8, TBlittable> {
        let width = self.get_buffer_width();
        BlitBuilder::create_ext(
            &mut self.buffer_pixels,
            width,
            source_blittable
        )
    }
}