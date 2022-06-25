use glam::{vec3a, Vec3A};
use crate::rendering::blittable::{Blittable, BufferProviderMut, SizedSurface};
use crate::rendering::transform::Transform;

pub struct Vertex {
    pub position: (i32, i32)
}

pub struct TexturedVertex {
    pub position: (i32, i32),
    pub uv: (u16, u16)
}

pub struct TriangleRasterizer<'a, T: Copy> {
    buffer: &'a mut [T],
    buffer_width: usize,
    buffer_height: usize,
    transform: Transform
}
impl<'a, T: Copy> TriangleRasterizer<'a, T> {
    pub fn create(buffer_provider: &'a mut (impl BufferProviderMut<T>+SizedSurface)) -> Self {
        let buffer_width = buffer_provider.get_width();
        let buffer = buffer_provider.get_buffer_mut();
        let buffer_height = buffer.len() / buffer_width;
        Self {
            buffer,
            buffer_width,
            buffer_height,
            transform: Transform::from_identity()
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

    fn is_degrade_triangle(&self, positions: [(i32, i32); 3]) -> bool {
        positions[0] == positions[1] ||
            positions[1] == positions[2] ||
            positions[2] == positions[0] ||
        {
            let v0 = glam::vec2(
                (positions[1].0 - positions[0].0) as f32,
                (positions[1].1 - positions[0].1) as f32
            ).normalize();
            let v1 = glam::vec2(
                (positions[2].0 - positions[0].0) as f32,
                (positions[2].1 - positions[0].1) as f32
            ).normalize();
            (1.0 - v0.dot(v1).abs()) < 1e-7
        }
    }

    pub fn rasterize_with_color(
        mut self,
        color: T,
        vertices: &[Vertex],
        indices: &[u16],
    ) {
        for ii in (0..indices.len()).step_by(3) {
            let positions = [indices[ii], indices[ii+1], indices[ii+2]]
                .map(|index| vertices[index as usize].position);
            if self.is_degrade_triangle(positions) {
                continue;
            }

            let [top_pos, middle_pos, bottom_pos] = {
                let mut positions = self.get_transformed_positions(positions);
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
                positions
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
        let ((mut x0, _), (mut x1, _)) = (left_pos, right_pos);
        let (dx0, dx1, dy) = (
            middle_pos.0 - left_pos.0, middle_pos.0 - right_pos.0,
            (middle_pos.1 as i32 - left_pos.1 as i32) as f32
        );
        let (dx0, dx1) = (dx0 / dy, dx1 / dy);
        for y in left_pos.1 as i32..=middle_pos.1 as i32 {
            self.draw_span_colored(color, x0, x1, y);
            x0 += dx0;
            x1 += dx1;
        }
    }
    fn draw_span_colored(&mut self, color: T, x0: f32, x1: f32, y: i32) {
        if x1 < 0.0 || x0 >= self.buffer_width as f32 {
            return;
        }
        if (0..(self.buffer_height) as i32).contains(&y) {
            let stride = y as usize * self.buffer_width;
            let span_left = stride + x0.max(0.0) as usize;
            let span_right = stride + ((x1 + 0.15) as usize).min(self.buffer_width - 1);
            if span_left > span_right {
                return;
            }
            for pix in &mut self.buffer[span_left..=span_right] {
                *pix = color;
            }
        }
    }

    pub fn rasterize_with_surface(
        mut self,
        drawable: &'a impl Blittable<T>,
        vertices: &[TexturedVertex],
        indices: &[u16]
    ) {
        for ii in (0..indices.len()).step_by(3) {
            let idx_triple = [
                indices[ii] as usize,
                indices[ii+1] as usize,
                indices[ii+2] as usize
            ];
            let positions = idx_triple.map(|index| vertices[index as usize].position);
            if self.is_degrade_triangle(positions) {
                continue;
            }
            let (
                [top_pos, middle_pos, bottom_pos],
                [top_uv, middle_uv, bottom_uv]
            ) = {
                let mut positions = self.get_transformed_positions(positions);
                let mut uvs = idx_triple.map(|index| vertices[index as usize].uv);
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
                    positions,
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

        let mut interpolator_0 = vec3a(left_pos.0, left_uv.0, left_uv.1);
        let mut interpolator_1 = vec3a(right_pos.0, right_uv.0, right_uv.1);
        let delta_0 = vec3a(
            middle_pos.0 - left_pos.0,
            middle_uv.0 - left_uv.0 ,
            middle_uv.1 - left_uv.1
        ) / (middle_pos.1 - left_pos.1);
        let delta_1 = vec3a(
             middle_pos.0 - right_pos.0,
             middle_uv.0 - right_uv.0,
             middle_uv.1 - right_uv.1
        ) / (middle_pos.1 - left_pos.1);

        for y in left_pos.1 as i32..=middle_pos.1 as i32 {
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
        if interpolator_0.x >= (self.buffer_width - 1) as f32 || interpolator_1.x < 0.0 {
            return;
        }
        if (0..(self.buffer_height) as i32).contains(&y) {
            let stride = y as usize * self.buffer_width;
            let x0 = interpolator_0.x.clamp(0.0, (self.buffer_width - 1) as f32);
            let x1 = (interpolator_1.x + 0.15).clamp(0.0, (self.buffer_width - 1) as f32);

            let span_left = stride + x0 as usize;
            let span_right= stride + x1 as usize;
            let dw = drawable.get_width();
            let dh = drawable.get_height();
            let drawable_buffer = drawable.get_buffer();
            let mut x_acc = x0.floor();

            if span_right < span_left {
                return;
            }

            for pix in &mut self.buffer[span_left..=span_right] {
                let t = ((x_acc - interpolator_0.x) / (interpolator_1.x - interpolator_0.x))
                    .clamp(0.0, 1.0);
                let interpolated = interpolator_0 * (1.0 - t) + interpolator_1 * t;
                let u = interpolated.y.clamp(0.05, (dw-1) as f32) as usize;
                let v = interpolated.z.clamp(0.05, (dh-1) as f32) as usize;
                let uv_idx = v * dw + u;
                drawable.blend_function(pix, &drawable_buffer[uv_idx]);
                x_acc += 1.0;
            }
        }
    }
    fn get_transformed_positions(&self, positions: [(i32, i32); 3]) -> [(f32, f32); 3] {
        positions.map(|it| {
            let p = self.transform.matrix * vec3a(it.0 as f32 + 0.5, it.1 as f32 + 0.5, 1.0);
            (p.x.floor() + 0.5, p.y.floor() + 0.5)
        })
    }
}

