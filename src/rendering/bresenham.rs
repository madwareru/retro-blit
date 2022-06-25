use glam::vec3a;
use crate::rendering::blittable::{BufferProviderMut, SizedSurface};
use crate::rendering::transform::Transform;

fn plot_bresenham_circle(
    cx: i32, cy: i32, r: i32,
    mut plot_func: impl FnMut(i32, i32) -> ()
) {
    fn plot_all_octants(cx: i32, cy: i32, dx: i32, dy: i32, plot_func: &mut impl FnMut(i32, i32) -> ()) {
        let (x0, x1, x2, x3, x4, x5, x6, x7) = (
            cx + dx, cx + dx, cx - dx, cx - dx, cx + dy, cx + dy, cx - dy, cx - dy
        );
        let (y0, y1, y2, y3, y4, y5, y6, y7) = (
            cy + dy, cy - dy, cy + dy, cy - dy, cy + dx, cy - dx, cy + dx, cy - dx
        );
        plot_func(x0, y0);
        plot_func(x1, y1);
        plot_func(x2, y2);
        plot_func(x3, y3);
        plot_func(x4, y4);
        plot_func(x5, y5);
        plot_func(x6, y6);
        plot_func(x7, y7);
    }
    let mut d = 3 - r * 2;
    let mut x = 0;
    let mut y = r;
    plot_all_octants(cx, cy, x, y, &mut plot_func);
    while x < y {
        if d <= 0 {
            d += 6 + (x << 2);
        } else {
            d += 10 + ((x - y) << 2);
            y -= 1;
        }
        x += 1;
        plot_all_octants(cx, cy, x, y, &mut plot_func);
    }
}

fn plot_bresenham_line<F : FnMut(i32, i32) -> ()>(x0: i32, y0: i32, x1: i32, y1: i32, mut plot_func: F) {
    if y0 == y1 {
        for x in x0.min(x1)..=x0.max(x1) { plot_func(x, y0); }
    } else if x0 == x1 {
        for y in y0.min(y1)..=y0.max(y1) { plot_func(x0, y); }
    } else {
        let (dx_abs, dy_abs) = ((x1 - x0).abs(), (y1 - y0).abs());
        let (dx2, dy2) = (dx_abs << 1, dy_abs << 1);
        if dx_abs >= dy_abs {
            let (x0, x1, y0, y1) = if x0 > x1 {
                (x1, x0, y1, y0)
            } else {
                (x0, x1, y0, y1)
            };
            let sign = if y0 < y1 { 1 } else { -1 };
            let mut y = y0;
            let mut d = dy2 - dx_abs;
            for x in x0..=x1 {
                plot_func(x, y);
                if d > 0 {
                    d -= dx2;
                    y += sign;
                }
                d += dy2;
            }
        } else {
            let (x0, x1, y0, y1) = if y0 > y1 {
                (x1, x0, y1, y0)
            } else {
                (x0, x1, y0, y1)
            };
            let sign = if x0 < x1 { 1 } else { -1 };
            let mut x = x0;
            let mut d = dx2 - dy_abs;
            for y in y0..=y1 {
                plot_func(x, y);
                if d > 0 {
                    d -= dy2;
                    x += sign;
                }
                d += dx2;
            }
        }
    }
}

pub struct BresenhamCircleDrawer<'a, T: Copy> {
    buffer: &'a mut [T],
    buffer_width: usize,
    position: (i32, i32),
    radius: i32
}

impl<'a, T: Copy> BresenhamCircleDrawer<'a, T> {
    pub fn create(buffer_provider: &'a mut (impl BufferProviderMut<T>+SizedSurface)) -> Self {
        let buffer_width = buffer_provider.get_width();
        let buffer = buffer_provider.get_buffer_mut();
        Self {
            buffer,
            buffer_width,
            position: (0, 0),
            radius: 0
        }
    }

    pub fn with_position(self, position: (i32, i32)) -> Self {
        Self { position, ..self }
    }

    pub fn with_radius(self, radius: i32) -> Self {
        Self { radius, ..self }
    }

    pub fn draw(self, color: T) {
        let buffer_height = self.buffer.len() / self.buffer_width;
        plot_bresenham_circle(
            self.position.0,
            self.position.1,
            self.radius,
            |x, y| {
                if !(0..self.buffer_width as i32).contains(&x) {
                    return;
                }
                if !(0..buffer_height as i32).contains(&y) {
                    return;
                }
                self.buffer[x as usize + y as usize * self.buffer_width] = color;
            }
        )
    }
}

pub struct LineStripRasterizer<'a, T: Copy + Default>  {
    buffer: &'a mut [T],
    buffer_width: usize,
    transform: Transform,
    color: T
}
impl<'a, T: Copy + Default> LineStripRasterizer<'a, T> {
    pub fn create(buffer_provider: &'a mut (impl BufferProviderMut<T>+SizedSurface)) -> Self {
        let buffer_width = buffer_provider.get_width();
        let buffer = buffer_provider.get_buffer_mut();
        Self {
            buffer,
            buffer_width,
            transform: Transform::from_identity(),
            color: Default::default()
        }
    }

    pub fn with_color(self, color: T) -> Self {
        Self {
            color,
            ..self
        }
    }

    pub fn with_transform(self, transform: Transform) -> Self {
        Self {
            transform,
            ..self
        }
    }

    pub fn with_translation(self, translation: (i32, i32)) -> Self {
        Self {
            transform: self.transform.with_translation(translation),
            ..self
        }
    }

    pub fn with_rotation(self, rotation: f32) -> Self {
        Self {
            transform: self.transform.with_rotation(rotation),
            ..self
        }
    }

    pub fn with_scale(self, scale: (f32, f32)) -> Self {
        Self {
            transform: self.transform.with_scale(scale),
            ..self
        }
    }

    fn get_transformed_positions(&self, positions: [(i32, i32); 2]) -> [(i32, i32); 2] {
        positions.map(|it| {
            let p = self.transform.matrix * vec3a(it.0 as f32 + 0.5, it.1 as f32 + 0.5, 1.0);
            (p.x.floor() as i32, p.y.floor() as i32)
        })
    }

    pub fn rasterize_slice(self, closed: bool, positions: &[(i32, i32)]) {
        if positions.len() <= 1 {
            return;
        }
        if closed {
            for i in 1..=positions.len() {
                let next = self.get_transformed_positions(
                    [
                        positions[i-1],
                        positions[i % positions.len()]
                    ]
                );
                LineRasterizer::create_from_raw(self.buffer, self.buffer_width)
                    .from(next[0])
                    .to(next[1])
                    .rasterize(self.color);
            }
        } else {
            for i in 1..positions.len() {
                let next = self.get_transformed_positions(
                    [
                        positions[i-1],
                        positions[i]
                    ]
                );
                LineRasterizer::create_from_raw(self.buffer, self.buffer_width)
                    .from(next[0])
                    .to(next[1])
                    .rasterize(self.color);
            }
        }

    }
}

pub struct LineRasterizer<'a, T: Copy> {
    buffer: &'a mut [T],
    buffer_width: usize,
    from: (i32, i32),
    to: (i32, i32)
}

impl<'a, T: Copy> LineRasterizer<'a, T> {
    pub fn create_from_raw(buffer: &'a mut [T], buffer_width: usize) -> Self {
        Self {
            buffer,
            buffer_width,
            from: (0, 0),
            to: (0, 0)
        }
    }

    pub fn create(buffer_provider: &'a mut (impl BufferProviderMut<T>+SizedSurface)) -> Self {
        let buffer_width = buffer_provider.get_width();
        let buffer = buffer_provider.get_buffer_mut();
        Self {
            buffer,
            buffer_width,
            from: (0, 0),
            to: (0, 0)
        }
    }

    pub fn from(self, from: (i32, i32)) -> Self {
        Self { from, ..self }
    }

    pub fn to(self, to: (i32, i32)) -> Self {
        Self { to, ..self }
    }

    pub fn rasterize(self, color: T) {
        let buffer_height = self.buffer.len() / self.buffer_width;
        plot_bresenham_line(
            self.from.0,
            self.from.1,
            self.to.0,
            self.to.1,
            |x, y| {
                if (0..self.buffer_width as i32).contains(&x) &&
                    (0..buffer_height as i32).contains(&y)
                {
                    self.buffer[x as usize + y as usize * self.buffer_width] = color;
                }
            }
        )
    }
}