use glam::{vec3a, Vec3A};
use crate::rendering::blittable::{Blittable, BufferProviderMut, SizedSurface};

pub struct TriangleRasterizer<'a, T: Copy> {
    buffer: &'a mut [T],
    buffer_width: usize,
    uvs: [(usize, usize); 3],
    positions: [(i32, i32); 3]
}
impl<'a, T: Copy> TriangleRasterizer<'a, T> {
    pub fn create(buffer_provider: &'a mut (impl BufferProviderMut<T>+SizedSurface)) -> Self {
        let buffer_width = buffer_provider.get_width();
        let buffer = buffer_provider.get_buffer_mut();
        Self {
            buffer,
            buffer_width,
            uvs: [(0, 0); 3],
            positions: [(0, 0); 3]
        }
    }

    pub fn with_uvs(self, uvs: [(usize, usize); 3]) -> Self {
        Self { uvs, ..self }
    }

    pub fn with_positions(self, positions: [(i32, i32); 3]) -> Self {
        Self { positions, ..self }
    }

    fn is_degrade_triangle(&self) -> bool {
        self.positions[0] == self.positions[1] ||
            self.positions[1] == self.positions[2] ||
            self.positions[2] == self.positions[0] ||
        {
            let v0 = glam::vec2(
                (self.positions[1].0 - self.positions[0].0) as f32,
                (self.positions[1].1 - self.positions[0].1) as f32
            ).normalize();
            let v1 = glam::vec2(
                (self.positions[2].0 - self.positions[0].0) as f32,
                (self.positions[2].1 - self.positions[0].1) as f32
            ).normalize();
            (1.0 - v0.dot(v1).abs()) < 1e-7
        }
    }

    pub fn rasterize_with_color(mut self, color: T) {
        if self.is_degrade_triangle() {
            return;
        }

        let [top_pos, middle_pos, bottom_pos] = {
            let mut positions = self.positions;
            for i in 0..3 {
                // insertion sort is decently fast for this size
                for j in (i + 1..3).rev() {
                    if positions[j].1 < positions[j - 1].1 {
                        let memorized = positions[j];
                        positions[j] = positions[j-1];
                        positions[j-1] = memorized;
                    }
                }
            }
            positions.map(|it| (it.0 as f32 + 0.5, it.1 as f32 + 0.5))
        };

        if top_pos.1 as i32 == middle_pos.1 as i32 {
            self.draw_flat_top_colored(
                color,
                top_pos,
                middle_pos,
                bottom_pos
            );
        } else if bottom_pos.1 as i32 == middle_pos.1 as i32 {
            self.draw_flat_bottom_colored(
                color,
                top_pos,
                middle_pos,
                bottom_pos,
            );
        } else {
            // default case
            let half_t = (middle_pos.1 - top_pos.1) / (bottom_pos.1 - top_pos.1);
            let mid_point_x = top_pos.0 + (bottom_pos.0 - top_pos.0) * half_t;

            self.draw_flat_bottom_colored(
                color,
                top_pos,
                middle_pos,
                (mid_point_x, middle_pos.1)
            );
            self.draw_flat_top_colored(
                color,
                middle_pos,
                (mid_point_x, middle_pos.1),
                bottom_pos
            );
        }
    }

    fn draw_flat_bottom_colored(&mut self, color: T, top_pos: (f32, f32), middle_pos: (f32, f32), bottom_pos: (f32, f32)) {
        let [left_pos, middle_pos, right_pos] = {
            if bottom_pos.0 <= middle_pos.0 {
                [bottom_pos, top_pos, middle_pos]
            } else {
                [middle_pos, top_pos, bottom_pos]
            }
        };
        let ((mut x0, _), (mut x1, _)) = (middle_pos, middle_pos);
        let ( dx0, dx1, dy ) = (
            left_pos.0 - middle_pos.0, right_pos.0 - middle_pos.0,
            (left_pos.1 as i32 - middle_pos.1 as i32) as f32
        );
        let (dx0, dx1) = (dx0 / dy, dx1 / dy);
        for y in middle_pos.1 as i32..=left_pos.1 as i32 {
            self.draw_span_colored(color, x0, x1, y);
            x0 += dx0;
            x1 += dx1;
        }
    }

    fn draw_flat_top_colored(&mut self, color: T, top_pos: (f32, f32), middle_pos: (f32, f32), bottom_pos: (f32, f32)) {
        let [left_pos, middle_pos, right_pos] = {
            if top_pos.0 <= middle_pos.0 {
                [top_pos, bottom_pos, middle_pos]
            } else {
                [middle_pos, bottom_pos, top_pos]
            }
        };
        let ((mut x0, _), (mut x1, _)) = (middle_pos, middle_pos);
        let (dx0, dx1, dy) = (
            left_pos.0 - middle_pos.0, right_pos.0 - middle_pos.0,
            (middle_pos.1 as i32 - left_pos.1 as i32) as f32
        );
        let (dx0, dx1) = (dx0 / dy, dx1 / dy);
        for y in (left_pos.1 as i32..=middle_pos.1 as i32).rev() {
            self.draw_span_colored(color, x0, x1, y);
            x0 += dx0;
            x1 += dx1;
        }
    }

    fn draw_span_colored(&mut self, color: T, x0: f32, x1: f32, y: i32) {
        let span_left = x0 as i32;
        let span_right = (x1 + 0.5) as i32;
        for x in span_left..=span_right {
            if y >= 0 && (0..self.buffer_width as i32).contains(&x) {
                let idx = y as usize * self.buffer_width + x as usize;
                if idx < self.buffer.len() {
                    self.buffer[idx] = color;
                }
            }
        }
    }

    pub fn rasterize_with_surface(mut self, drawable: &'a impl Blittable<T>) {
        if self.is_degrade_triangle() {
            return;
        }

        let (
            [top_pos, middle_pos, bottom_pos],
            [top_uv, middle_uv, bottom_uv]
        ) = {
            let mut positions = self.positions;
            let mut uvs = self.uvs;
            for i in 0..3 {
                // insertion sort is decently fast for this size
                for j in (i + 1..3).rev() {
                    if positions[j].1 < positions[j - 1].1 {
                        let memorized = (positions[j], uvs[j]);
                        positions[j] = positions[j-1];
                        uvs[j] = uvs[j-1];
                        positions[j-1] = memorized.0;
                        uvs[j-1] = memorized.1;
                    }
                }
            }
            (
                positions.map(|it| (it.0 as f32 + 0.5, it.1 as f32 + 0.5)),
                uvs.map(|it| (it.0 as f32 + 0.5, it.1 as f32 + 0.5))
            )
        };

        if top_pos.1 as i32 == middle_pos.1 as i32 {
            self.draw_flat_top_with_surface(
                drawable,
                top_pos,
                middle_pos,
                bottom_pos,
                top_uv,
                middle_uv,
                bottom_uv
            );
        } else if bottom_pos.1 as i32 == middle_pos.1 as i32 {
            self.draw_flat_bottom_with_surface(
                drawable,
                top_pos,
                middle_pos,
                bottom_pos,
                top_uv,
                middle_uv,
                bottom_uv
            );
        } else {
            // default case
            let half_t = (middle_pos.1 - top_pos.1) / (bottom_pos.1 - top_pos.1);
            let mid_point_x = top_pos.0 + (bottom_pos.0 - top_pos.0) * half_t;
            let mid_point_u = top_uv.0 + (bottom_uv.0 - top_uv.0) * half_t;
            let mid_point_v = top_uv.1 + (bottom_uv.1 - top_uv.1) * half_t;

            self.draw_flat_bottom_with_surface(
                drawable,
                top_pos, middle_pos, (mid_point_x, middle_pos.1),
                top_uv, middle_uv, (mid_point_u, mid_point_v)
            );
            self.draw_flat_top_with_surface(
                drawable,
                middle_pos, (mid_point_x, middle_pos.1), bottom_pos,
                middle_uv, (mid_point_u, mid_point_v), bottom_uv
            );
        }
    }

    fn draw_flat_bottom_with_surface(&mut self, drawable: &'a impl Blittable<T>, top_pos: (f32, f32), middle_pos: (f32, f32), bottom_pos: (f32, f32), top_uv: (f32, f32), middle_uv: (f32, f32), bottom_uv: (f32, f32)) {
        let (
            [left_pos, middle_pos, right_pos],
            [left_uv, middle_uv, right_uv]
        ) = {
            if bottom_pos.0 <= middle_pos.0 {
                (
                    [bottom_pos, top_pos, middle_pos],
                    [bottom_uv, top_uv, middle_uv]
                )
            } else {
                (
                    [middle_pos, top_pos, bottom_pos],
                    [middle_uv, top_uv, bottom_uv]
                )
            }
        };



        let mut interpolator_0 = vec3a(middle_pos.0, middle_uv.0, middle_uv.1);
        let mut interpolator_1 = vec3a(middle_pos.0, middle_uv.0, middle_uv.1);
        let delta_0 = vec3a(
            left_pos.0 - middle_pos.0,
            left_uv.0 - middle_uv.0,
            left_uv.1 - middle_uv.1
        ) / (left_pos.1 - middle_pos.1);
        let delta_1 = vec3a(
            right_pos.0 - middle_pos.0,
            right_uv.0 - middle_uv.0,
            right_uv.1 - middle_uv.1
        ) / (left_pos.1 - middle_pos.1);

        for y in middle_pos.1 as i32..=left_pos.1 as i32 {
            self.draw_span(drawable, interpolator_0, interpolator_1, y);
            interpolator_0 += delta_0;
            interpolator_1 += delta_1;
        }
    }
    fn draw_flat_top_with_surface(&mut self, drawable: &'a impl Blittable<T>, top_pos: (f32, f32), middle_pos: (f32, f32), bottom_pos: (f32, f32), top_uv: (f32, f32), middle_uv: (f32, f32), bottom_uv: (f32, f32)) {
        let (
            [left_pos, middle_pos, right_pos],
            [left_uv, middle_uv, right_uv]
        ) = {
            if top_pos.0 > middle_pos.0{
                (
                    [middle_pos, bottom_pos, top_pos],
                    [middle_uv, bottom_uv, top_uv]
                )
            } else {
                (
                    [top_pos, bottom_pos, middle_pos],
                    [top_uv, bottom_uv, middle_uv]
                )

            }
        };

        let mut interpolator_0 = vec3a(middle_pos.0, middle_uv.0, middle_uv.1);
        let mut interpolator_1 = vec3a(middle_pos.0, middle_uv.0, middle_uv.1);
        let delta_0 = vec3a(
            left_pos.0 - middle_pos.0,
            left_uv.0 - middle_uv.0,
            left_uv.1 - middle_uv.1
        ) / (middle_pos.1 - left_pos.1);
        let delta_1 = vec3a(
            right_pos.0 - middle_pos.0,
            right_uv.0 - middle_uv.0,
            right_uv.1 - middle_uv.1
        ) / (middle_pos.1 - left_pos.1);

        for y in (left_pos.1 as i32..=middle_pos.1 as i32).rev() {
            self.draw_span(drawable, interpolator_0, interpolator_1, y);
            interpolator_0 += delta_0;
            interpolator_1 += delta_1;
        }
    }

    fn draw_span(
        &mut self,
        drawable: &impl Blittable<T>,
        interpolator_0: Vec3A,
        interpolator_1: Vec3A,
        y: i32
    ) {
        let mut x_acc = interpolator_0.x.floor();
        let dw = drawable.get_width();
        let drawable_buffer = drawable.get_buffer();
        for x in interpolator_0.x as i32 ..= (interpolator_1.x + 0.5) as i32 {
            let t = ((x_acc - interpolator_0.x) / (interpolator_1.x - interpolator_0.x))
                .clamp(0.0, 1.0);
            let u = (interpolator_0.y + (interpolator_1.y - interpolator_0.y) * t)
                .clamp(0.0, dw as f32) as usize;
            let v = (interpolator_0.z + (interpolator_1.z - interpolator_0.z) * t)
                .clamp(0.0, dw as f32) as usize;
            let uv_idx = v * dw + u;
            if uv_idx < drawable_buffer.len() && y >= 0 && (0..self.buffer_width as i32).contains(&x) {
                let color = drawable_buffer[uv_idx];
                let idx = y as usize * self.buffer_width + x as usize;
                if idx < self.buffer.len() {
                    drawable.blend_function(&mut self.buffer[idx], &color);
                }
            }
            x_acc += 1.0;
        }
    }
}