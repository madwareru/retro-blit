pub struct Rect {
    pub x_range: std::ops::Range<usize>,
    pub y_range: std::ops::Range<usize>
}
impl Rect {
    pub fn get_width(&self) -> usize {
        self.x_range.end - self.x_range.start
    }
    pub fn get_height(&self) -> usize {
        self.y_range.end - self.y_range.start
    }
}

pub trait SizedSurface {
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;
}

pub trait BufferProvider<T: Copy> {
    fn get_buffer(&self) -> & [T];
}

pub trait BufferProviderMut<T: Copy> {
    fn get_buffer_mut(&mut self) -> &mut [T];
}

pub trait Blittable<T: Copy> : SizedSurface + BufferProvider<T> {
    #[inline(always)]
    fn blend_function(&self, dst: &mut T, src: &T) { *dst = *src; }

    fn blit_impl(&self, buffer: &mut [T], buffer_width: usize, self_rect: Rect, dst_rect: Rect, flip: Flip) {
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
                        .zip((&src_buffer[src_stride..src_stride+span_length]).iter().rev());
                    for (dest, src) in zipped {
                        self.blend_function(dest, src);
                    }
                    src_stride += width;
                    dst_stride -= buffer_width;
                }
            } else {
                for _ in 0..span_count {
                    let zipped = (&mut buffer[dst_stride..dst_stride+span_length])
                        .iter_mut()
                        .zip(&src_buffer[src_stride..src_stride+span_length]);
                    for (dest, src) in zipped {
                        self.blend_function(dest, src);
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
                        .zip((&src_buffer[src_stride..src_stride+span_length]).iter().rev());
                    for (dest, src) in zipped {
                        self.blend_function(dest, src);
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
                        self.blend_function(dest, src);
                    }
                    src_stride += width;
                    dst_stride += buffer_width;
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub enum Flip {
    None,
    X,
    Y,
    XY
}

fn blit_ext<T: Copy, TBlittable: Blittable<T>>(
    drawable: &TBlittable, buffer: &mut [T], buffer_width: usize,
    src_x: usize, src_y: usize,
    src_width: usize, src_height: usize,
    dst_x: i32, dst_y: i32,
    dst_width: usize, dst_height: usize,
    flip: Flip
) {
    let src_width_max = (src_width + src_x).min(drawable.get_width());
    let src_height_max = (src_height + src_y).min(drawable.get_height());

    let dst_width_max = ((dst_width as i32 + dst_x) as usize).min(buffer_width);
    let dst_height_max = ((dst_height as i32 + dst_y) as usize).min(buffer.len() / buffer_width);

    let mut src_rect = Rect {
        x_range: src_x.min(src_width_max)..src_width_max,
        y_range: src_y.min(src_height_max)..src_height_max
    };
    let mut dst_rect = Rect{
        x_range: 0..dst_width_max,
        y_range: 0..dst_height_max
    };

    if dst_x < 0 {
        src_rect.x_range.start = (src_rect.x_range.start + (-dst_x) as usize)
            .min(src_rect.x_range.end);
    } else {
        dst_rect.x_range.start = ((dst_rect.x_range.start as i32 + dst_x) as usize)
            .min(dst_rect.x_range.end);
    }
    if dst_y < 0 {
        src_rect.y_range.start = (src_rect.y_range.start + (-dst_y) as usize)
            .min(src_rect.y_range.end);
    } else {
        dst_rect.y_range.start = ((dst_rect.y_range.start as i32 + dst_y) as usize)
            .min(dst_rect.y_range.end);
    }

    let (flip_x, flip_y) = match flip {
        Flip::None => (false, false),
        Flip::X => (true, false),
        Flip::Y => (false, true),
        Flip::XY => (true, true)
    };

    if flip_x {
        if src_x < src_rect.x_range.start {
            let x_diff = src_width - src_rect.get_width();
            src_rect.x_range.start -= x_diff;
            src_rect.x_range.end -= x_diff;
        } else if src_rect.get_width() > dst_rect.get_width() {
            let x_diff = src_rect.get_width() - dst_rect.get_width();
            src_rect.x_range.start += x_diff;
            src_rect.x_range.end += x_diff;
        }
    }

    if flip_y {
        if src_y < src_rect.y_range.start {
            let y_diff = src_height - src_rect.get_height();
            src_rect.y_range.start -= y_diff;
            src_rect.y_range.end -= y_diff;
        } else if src_rect.get_height() > dst_rect.get_height() {
            let y_diff = src_rect.get_height() - dst_rect.get_height();
            src_rect.y_range.start += y_diff;
            src_rect.y_range.end += y_diff;
        }
    }

    drawable.blit_impl(
        buffer,
        buffer_width,
        src_rect,
        dst_rect,
        flip
    )
}

pub struct BlitBuilder<'a, T: Copy, TBlittable: Blittable<T>> {
    drawable: &'a TBlittable,
    buffer: &'a mut [T],
    buffer_width: usize,
    src_x: usize,
    src_y: usize,
    src_width: usize,
    src_height: usize,
    dst_x: i32,
    dst_y: i32,
    dst_width: usize,
    dst_height: usize,
    flip: Flip
}
impl<'a, T: Copy, TBlittable: Blittable<T>> BlitBuilder<'a, T, TBlittable> {
    pub fn create_ext(buffer: &'a mut [T], buffer_width: usize, drawable: &'a TBlittable) -> Self {
        let dst_height = buffer.len() / buffer_width;
        Self {
            drawable,
            buffer,
            buffer_width,
            src_x: 0,
            src_y: 0,
            src_width: drawable.get_width(),
            src_height: drawable.get_height(),
            dst_x: 0,
            dst_y: 0,
            dst_width: buffer_width,
            dst_height,
            flip: Flip::None
        }
    }
    pub fn create(
        dest: & 'a mut impl BlitDestination<'a, T, TBlittable>,
        src: &'a TBlittable
    ) -> Self {
        dest.initiate_blit_on_self(src)
    }
    pub fn with_dest_pos(self, dst_x: i32, dst_y: i32) -> Self {
        Self {
            dst_x,
            dst_y,
            ..self
        }
    }
    pub fn with_source_subrect(self, src_x: usize, src_y: usize, src_width: usize, src_height: usize) -> Self {
        Self {
            src_x,
            src_y,
            src_width,
            src_height,
            ..self
        }
    }
    pub fn with_dest_subrect(self, dst_x: i32, dst_y: i32, dst_width: usize, dst_height: usize) -> Self {
        Self {
            dst_x,
            dst_y,
            dst_width,
            dst_height,
            ..self
        }
    }
    pub fn with_flip(self, flip: Flip) -> Self {
        Self {
            flip,
            ..self
        }
    }
    pub fn blit(&mut self) {
        blit_ext(
            self.drawable,
            self.buffer,
            self.buffer_width,
            self.src_x,
            self.src_y,
            self.src_width,
            self.src_height,
            self.dst_x,
            self.dst_y,
            self.dst_width,
            self.dst_height,
            self.flip
        )
    }
}

pub trait BlitDestination<'a, T:Copy, TBlittable: Blittable<T>> : BufferProviderMut<T> + SizedSurface {
    fn initiate_blit_on_self(&'a mut self, source_blittable: &'a TBlittable) -> BlitBuilder<'a, T, TBlittable> {
        let width = self.get_width();
        BlitBuilder::create_ext(
            self.get_buffer_mut(),
            width,
            source_blittable
        )
    }
}