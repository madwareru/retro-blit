use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const STAGING_SLOT_COUNT: usize = 3;

struct StagingSlot {
    buffer: wgpu::Buffer,
    pending: Arc<AtomicBool>,
}

pub(crate) struct TextureUploader {
    uploaded_pixels: Vec<u8>,
    uploaded_colors: [u8; 256 * 3],
    dirty_spans: Vec<Range<usize>>,
    staging_slots: Vec<StagingSlot>,
    used_slot: Option<usize>,
}

impl TextureUploader {
    pub(crate) fn new(pixels: &[u8], colors: &[u8; 256 * 3]) -> Self {
        Self {
            uploaded_pixels: pixels.to_vec(),
            uploaded_colors: *colors,
            dirty_spans: Vec::new(),
            staging_slots: Vec::new(),
            used_slot: None,
        }
    }

    pub(crate) fn encode_staged(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        pixel_texture: &wgpu::Texture,
        palette_texture: &wgpu::Texture,
        width: usize,
        pixels: &[u8],
        colors: &[u8; 256 * 3],
    ) -> bool {
        device.poll(wgpu::PollType::Poll).unwrap();
        debug_assert!(self.used_slot.is_none());
        find_dirty_row_spans(
            pixels,
            &self.uploaded_pixels,
            width,
            16,
            &mut self.dirty_spans,
        );
        if self.dirty_spans.is_empty() {
            self.upload_palette(queue, palette_texture, colors);
            return false;
        }

        self.ensure_slots(device, width, pixels.len() / width);
        let Some(slot_index) = self
            .staging_slots
            .iter()
            .position(|slot| !slot.pending.load(Ordering::Acquire))
        else {
            self.upload_direct(queue, pixel_texture, width, pixels);
            self.upload_palette(queue, palette_texture, colors);
            return false;
        };

        let padded = padded_bytes_per_row(width);
        let slot = &self.staging_slots[slot_index];
        {
            let mut mapped = slot
                .buffer
                .get_mapped_range_mut(..)
                .expect("staging slot is not writable");
            let mut offset = 0;
            for span in &self.dirty_spans {
                for row in span.clone() {
                    let source = &pixels[row * width..(row + 1) * width];
                    mapped.slice(offset..offset + width).copy_from_slice(source);
                    mapped.slice(offset + width..offset + padded).fill(0);
                    offset += padded;
                }
            }
        }
        slot.buffer.unmap();

        let mut offset = 0;
        for span in &self.dirty_spans {
            let height = span.end - span.start;
            encoder.copy_buffer_to_texture(
                wgpu::TexelCopyBufferInfo {
                    buffer: &slot.buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: offset as u64,
                        bytes_per_row: Some(padded as u32),
                        rows_per_image: Some(height as u32),
                    },
                },
                wgpu::TexelCopyTextureInfo {
                    texture: pixel_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: span.start as u32,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: width as u32,
                    height: height as u32,
                    depth_or_array_layers: 1,
                },
            );
            let start = span.start * width;
            let end = span.end * width;
            self.uploaded_pixels[start..end].copy_from_slice(&pixels[start..end]);
            offset += height * padded;
        }
        self.used_slot = Some(slot_index);
        self.upload_palette(queue, palette_texture, colors);
        true
    }

    pub(crate) fn finish_submission(&mut self) {
        let Some(slot_index) = self.used_slot.take() else {
            return;
        };
        let slot = &self.staging_slots[slot_index];
        slot.pending.store(true, Ordering::Release);
        let pending = Arc::clone(&slot.pending);
        slot.buffer.map_async(wgpu::MapMode::Write, .., move |result| {
            if result.is_ok() {
                pending.store(false, Ordering::Release);
            }
        });
    }

    fn ensure_slots(&mut self, device: &wgpu::Device, width: usize, height: usize) {
        if !self.staging_slots.is_empty() {
            return;
        }
        let size = padded_bytes_per_row(width) * height;
        self.staging_slots = (0..STAGING_SLOT_COUNT)
            .map(|index| StagingSlot {
                buffer: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(match index {
                        0 => "pixel staging 0",
                        1 => "pixel staging 1",
                        _ => "pixel staging 2",
                    }),
                    size: size as u64,
                    usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: true,
                }),
                pending: Arc::new(AtomicBool::new(false)),
            })
            .collect();
    }

    fn upload_direct(
        &mut self,
        queue: &wgpu::Queue,
        pixel_texture: &wgpu::Texture,
        width: usize,
        pixels: &[u8],
    ) {
        for span in &self.dirty_spans {
            let start = span.start * width;
            let end = span.end * width;
            let height = span.end - span.start;
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: pixel_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: span.start as u32,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &pixels[start..end],
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
            self.uploaded_pixels[start..end].copy_from_slice(&pixels[start..end]);
        }
    }

    fn upload_palette(
        &mut self,
        queue: &wgpu::Queue,
        palette_texture: &wgpu::Texture,
        colors: &[u8; 256 * 3],
    ) {
        if self.uploaded_colors == *colors {
            return;
        }
        let rgba = palette_as_rgba(colors);
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: palette_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
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
        self.uploaded_colors = *colors;
    }
}

pub(crate) fn palette_as_rgba(colors: &[u8; 256 * 3]) -> [u8; 256 * 4] {
    let mut rgba = [0; 256 * 4];
    for (rgb, entry) in colors.chunks_exact(3).zip(rgba.chunks_exact_mut(4)) {
        entry[..3].copy_from_slice(rgb);
        entry[3] = 255;
    }
    rgba
}

pub(crate) fn padded_bytes_per_row(width: usize) -> usize {
    (width + 255) & !255
}

pub(crate) fn find_dirty_row_spans(
    current: &[u8],
    uploaded: &[u8],
    width: usize,
    max_unchanged_gap: usize,
    output: &mut Vec<Range<usize>>,
) {
    output.clear();
    assert_eq!(current.len(), uploaded.len());
    assert!(width > 0);
    assert_eq!(current.len() % width, 0);

    let mut span_start = None;
    let mut last_dirty_row = 0;

    for (row, (current_row, uploaded_row)) in current
        .chunks_exact(width)
        .zip(uploaded.chunks_exact(width))
        .enumerate()
    {
        if current_row == uploaded_row {
            continue;
        }

        if let Some(start) = span_start {
            if row - last_dirty_row - 1 > max_unchanged_gap {
                output.push(start..last_dirty_row + 1);
                span_start = Some(row);
            }
        } else {
            span_start = Some(row);
        }
        last_dirty_row = row;
    }

    if let Some(start) = span_start {
        output.push(start..last_dirty_row + 1);
    }
}
