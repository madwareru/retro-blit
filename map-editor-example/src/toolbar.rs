use retro_blit::{
    rendering::blittable::{BlitBuilder, BufferProvider, Rect, SizedSurface},
    rendering::BlittableSurface,
    window::RetroBlitContext
};
use retro_blit::rendering::blittable::{Blittable, Flip};

#[derive(Copy, Clone)]
pub enum ToolbarKind {
    Vertical,
    Horizontal
}

#[derive(Copy, Clone)]
pub enum HoverState {
    None,
    Hovered(u8),
    Clicked(u8)
}

pub struct Toolbar {
    x: usize,
    y: usize,
    kind: ToolbarKind,
    rect: Rect,
    hovered_index: HoverState,
    selected_index: Option<u8>,
    button_down: bool
}
impl Toolbar {
    pub fn make(x: usize, y: usize, rect: Rect, kind: ToolbarKind) -> Self {
        Self {
            x,
            y,
            kind,
            rect,
            hovered_index: HoverState::None,
            selected_index: None,
            button_down: false
        }
    }

    pub fn set_selection(&mut self, ix: Option<u8>) {
        self.selected_index = ix;
    }

    pub fn get_selection(&self) -> Option<u8>{
        self.selected_index
    }

    pub fn on_button_down(&mut self) {
        self.button_down = true;
        match self.hovered_index {
            HoverState::Hovered(ix) => {
                self.hovered_index = HoverState::Clicked(ix)
            },
            _ => {}
        }
    }

    pub fn on_button_up(&mut self) {
        self.button_down = false;
        match self.hovered_index {
            HoverState::Clicked(ix) => {
                self.selected_index = Some(ix);
            },
            _ => {}
        }
    }

    pub fn update(&mut self, mouse_pos: (f32, f32), surface: &BlittableSurface) {
        let (mx, my) = (mouse_pos.0 as i16, mouse_pos.1  as i16);
        let (sr_x, sr_y, sr_w, sr_h) = self.get_source_rect();
        if !(self.x as i16 .. (self.x + sr_w) as i16).contains(&mx) {
            self.hovered_index = HoverState::None;
            return;
        }
        if !(self.y as i16 .. (self.y + sr_h) as i16).contains(&my) {
            self.hovered_index = HoverState::None;
            return;
        }
        let (offset_x, offset_y) = (mx as usize - self.x, my as usize - self.y);
        self.hovered_index = {
            let mut sr_x = sr_x + offset_x;
            let mut sr_y = sr_y + offset_y;
            match self.kind {
                ToolbarKind::Vertical => {
                    sr_x += sr_w * 4;
                }
                ToolbarKind::Horizontal => {
                    sr_y += sr_h * 4;
                }
            }
            let buffer_width = surface.get_width();
            if !(0..buffer_width).contains(&sr_x) {
                HoverState::None
            }
            else {
                let offset = sr_x + sr_y * buffer_width;
                let buffer = surface.get_buffer();
                if (0..buffer.len()).contains(&offset) {
                    let ix = buffer[offset];
                    if ix == 0 {
                        HoverState::None
                    } else {
                        if self.button_down {
                            HoverState::Clicked(ix)
                        } else {
                            HoverState::Hovered(ix)
                        }
                    }
                }
                else {
                    HoverState::None
                }
            }
        };
    }

    pub fn draw(&self, surface: &BlittableSurface, dest: &mut RetroBlitContext) {
        let (sr_x, sr_y, sr_w, sr_h) = self.get_source_rect();

        let color_keyed = surface.with_color_key(0);
        BlitBuilder::create(dest, &color_keyed)
            .with_dest_pos(self.x as _, self.y as _)
            .with_source_subrect(sr_x, sr_y, sr_w, sr_h)
            .blit();

        match self.hovered_index {
            HoverState::None => {}
            HoverState::Hovered(ix) => {
                let (sr_x, sr_y, mask_offset_x, mask_offset_y) = match self.kind {
                    ToolbarKind::Vertical => {
                        (sr_x + sr_w, sr_y, sr_w * 3, 0)
                    }
                    ToolbarKind::Horizontal => {
                        (sr_x, sr_y + sr_h, 0, sr_h * 3)
                    }
                };
                let color_masked = surface.with_color_mask(ix, mask_offset_x, mask_offset_y);
                BlitBuilder::create(dest, &color_masked)
                    .with_dest_pos(self.x as _, self.y as _)
                    .with_source_subrect(sr_x, sr_y, sr_w, sr_h)
                    .blit();
            }
            HoverState::Clicked(ix) => {
                let (sr_x, sr_y, mask_offset_x, mask_offset_y) = match self.kind {
                    ToolbarKind::Vertical => {
                        (sr_x + sr_w * 2, sr_y, sr_w * 2, 0)
                    }
                    ToolbarKind::Horizontal => {
                        (sr_x, sr_y + sr_h * 2, 0, sr_h * 2)
                    }
                };
                let color_masked = surface.with_color_mask(ix, mask_offset_x, mask_offset_y);
                BlitBuilder::create(dest, &color_masked)
                    .with_dest_pos(self.x as _, self.y as _)
                    .with_source_subrect(sr_x, sr_y, sr_w, sr_h)
                    .blit();
            }
        }

        match self.selected_index {
            None => {}
            Some(ix) => {
                let (sr_x, sr_y, mask_offset_x, mask_offset_y) = match self.kind {
                    ToolbarKind::Vertical => {
                        (sr_x + 3 * sr_w, sr_y, sr_w, 0)
                    }
                    ToolbarKind::Horizontal => {
                        (sr_x, sr_y + 3 * sr_h, 0, sr_h)
                    }
                };
                let color_masked = surface.with_color_mask(ix, mask_offset_x, mask_offset_y);
                BlitBuilder::create(dest, &color_masked)
                    .with_dest_pos(self.x as _, self.y as _)
                    .with_source_subrect(sr_x, sr_y, sr_w, sr_h)
                    .blit();
            }
        }
    }

    fn get_source_rect(&self) -> (usize, usize, usize, usize) {
        let (sr_x, sr_y, sr_w, sr_h) = match self.kind {
            ToolbarKind::Vertical => {
                let sr_w = (self.rect.x_range.end - self.rect.x_range.start) / 5;
                let sr_h = self.rect.y_range.end - self.rect.y_range.start;
                (self.rect.x_range.start, self.rect.y_range.start, sr_w, sr_h)
            }
            ToolbarKind::Horizontal => {
                let sr_w = self.rect.x_range.end - self.rect.x_range.start;
                let sr_h = (self.rect.y_range.end - self.rect.y_range.start) / 5;
                (self.rect.x_range.start, self.rect.y_range.start, sr_w, sr_h)
            }
        };
        (sr_x, sr_y, sr_w, sr_h)
    }
}

pub struct MaskedWrapper<'a> {
    wrapped: &'a BlittableSurface,
    mask_offset: usize,
    color_mask: u8
}

impl SizedSurface for MaskedWrapper<'_> {
    fn get_width(&self) -> usize {
        self.wrapped.get_width()
    }

    fn get_height(&self) -> usize {
        self.wrapped.get_height()
    }
}

impl BufferProvider<u8> for MaskedWrapper<'_> {
    fn get_buffer(&self) -> &[u8] {
        self.wrapped.get_buffer()
    }
}

impl Blittable<u8> for MaskedWrapper<'_> {
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

        let (flip_x, flip_y) = match flip {
            Flip::None => (false, false),
            Flip::X => (true, false),
            Flip::Y => (false, true),
            Flip::XY => (true, true)
        };

        if flip_y {
            let mut dst_stride = (dst_rect.y_range.start + span_count - 1) * buffer_width + dst_rect.x_range.start;
            if flip_x {
                for _ in 0..span_count {
                    let zipped = (&mut buffer[dst_stride..dst_stride+span_length])
                        .iter_mut()
                        .zip((&src_buffer[src_stride..src_stride+span_length]).iter().rev())
                        .zip((&src_buffer[src_stride+self.mask_offset..src_stride+span_length+self.mask_offset]).iter().rev());
                    for ((dst, src), mask_id) in zipped {
                        *dst = if *mask_id != self.color_mask { *dst } else { *src };
                    }
                    src_stride += width;
                    dst_stride -= buffer_width;
                }
            } else {
                for _ in 0..span_count {
                    let zipped = (&mut buffer[dst_stride..dst_stride+span_length])
                        .iter_mut()
                        .zip(&src_buffer[src_stride..src_stride+span_length])
                        .zip(&src_buffer[src_stride+self.mask_offset..src_stride+span_length+self.mask_offset]);
                    for ((dst, src), mask_id) in zipped {
                        *dst = if *mask_id != self.color_mask { *dst } else { *src };
                    }
                    src_stride += width;
                    dst_stride -= buffer_width;
                }
            }
        } else {
            let mut dst_stride = dst_rect.y_range.start * buffer_width + dst_rect.x_range.start;
            if flip_x {
                for _ in 0..span_count {
                    let zipped = (&mut buffer[dst_stride..dst_stride+span_length])
                        .iter_mut()
                        .zip((&src_buffer[src_stride..src_stride+span_length]).iter().rev())
                        .zip((&src_buffer[src_stride+self.mask_offset..src_stride+span_length+self.mask_offset]).iter().rev());
                    for ((dst, src), mask_id) in zipped {
                        *dst = if *mask_id != self.color_mask { *dst } else { *src };
                    }
                    src_stride += width;
                    dst_stride += buffer_width;
                }
            } else {
                for _ in 0..span_count {
                    let zipped = (&mut buffer[dst_stride..dst_stride+span_length])
                        .iter_mut()
                        .zip(&src_buffer[src_stride..src_stride+span_length])
                        .zip(&src_buffer[src_stride+self.mask_offset..src_stride+span_length+self.mask_offset]);
                    for ((dst, src), mask_id) in zipped {
                        *dst = if *mask_id != self.color_mask { *dst } else { *src };
                    }
                    src_stride += width;
                    dst_stride += buffer_width;
                }
            }
        }
    }
}

pub trait WithColorMask {
    fn with_color_mask(&self, color_mask: u8, mask_offset_x: usize, mask_offset_y: usize) -> MaskedWrapper;
}

impl WithColorMask for BlittableSurface {
    fn with_color_mask(&self, color_mask: u8, mask_offset_x: usize, mask_offset_y: usize) -> MaskedWrapper {
        MaskedWrapper {
            wrapped: self,
            color_mask,
            mask_offset: mask_offset_y * self.get_width() + mask_offset_x
        }
    }
}