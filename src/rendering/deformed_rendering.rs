use glam::{vec2, Vec2, vec3a, Vec3A, Vec3Swizzles};
use crate::rendering::blittable::{Blittable, BufferProviderMut, SizedSurface};
use crate::rendering::transform::Transform;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: (f32, f32)
}

#[derive(Copy, Clone)]
pub struct TexturedVertex {
    pub position: (f32, f32),
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

    pub fn with_translation(self, translation: (i16, i16)) -> Self {
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

    pub fn rasterize_with_color(
        self,
        color: T,
        vertices: &[Vertex],
        indices: &[u16],
    ) {
        let transform = self.transform;
        self.rasterize_with_color_iter(
            (0..indices.len())
                .step_by(3)
                .map(|ii| {
                    let idx_triple = [
                        indices[ii] as usize,
                        indices[ii+1] as usize,
                        indices[ii+2] as usize
                    ];
                    let mut vertices = idx_triple.map(|index| vertices[index as usize]);
                    let positions = transform.transform_positions(vertices.map(|it| it.position));
                    for (v, p) in vertices.iter_mut().zip(positions.iter()) {
                        v.position = (p.0 as _, p.1 as _);
                    }
                    (vertices, color)
                })
        );
    }

    pub fn rasterize_with_color_iter(mut self, triangles: impl IntoIterator<Item=([Vertex; 3], T)>) {
        for (triangle, color) in triangles.into_iter() {
            let positions = triangle.map(|it| it.position);

            let [top_pos, middle_pos, bottom_pos] = {
                let mut positions = positions;
                for i in 0..3 {
                    // insertion sort is decently fast for this size
                    for j in (i + 1..3).rev() {
                        if positions[j].1 < positions[j - 1].1 {
                            positions.swap(j, j-1);
                        }
                    }
                }
                positions
            };

            if top_pos.1 as i16 == middle_pos.1 as i16 {
                self.draw_flat_top_colored(
                    color,
                    top_pos,
                    middle_pos,
                    bottom_pos
                );
            } else if bottom_pos.1 as i16 == middle_pos.1 as i16 {
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

        let (y_l_i, y_m_i) = (
            left_pos.1.ceil(),
            middle_pos.1.ceil()
        );

        if y_l_i as i16 == y_m_i as i16 {
            return;
        }

        let (dx0_dy0, dx1_dy1) = (
            (-middle_pos.0 + left_pos.0) / (-middle_pos.1 + left_pos.1),
            (-middle_pos.0 + right_pos.0) / (-middle_pos.1 + right_pos.1),
        );

        let (mut x0, mut x1) = (
            middle_pos.0 + dx0_dy0 * (y_m_i - middle_pos.1),
            middle_pos.0 + dx1_dy1 * (y_m_i - middle_pos.1)
        );

        for y in y_m_i as i16..y_l_i as i16 {
            self.draw_span_colored(color, x0, x1, y);
            x0 += dx0_dy0;
            x1 += dx1_dy1;
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

        let (y_l_i, y_m_i, y_r_i) = (
            left_pos.1.ceil(),
            middle_pos.1.ceil(),
            right_pos.1.ceil()
        );

        if y_l_i as i16 == y_m_i as i16 {
            return;
        }

        let (dx0_dy0, dx1_dy1) = (
            (middle_pos.0 - left_pos.0) / (middle_pos.1 - left_pos.1),
            (middle_pos.0 - right_pos.0) / (middle_pos.1 - right_pos.1),
        );

        let (mut x0, mut x1) = (
            left_pos.0 + dx0_dy0 * (y_l_i - left_pos.1),
            right_pos.0 + dx1_dy1 * (y_r_i - right_pos.1)
        );

        for y in y_l_i as i16..y_m_i as i16 {
            self.draw_span_colored(color, x0, x1, y);
            x0 += dx0_dy0;
            x1 += dx1_dy1;
        }
    }
    fn draw_span_colored(&mut self, color: T, x0: f32, x1: f32, y: i16) {
        let x0 = x0.ceil();
        let x1 = x1.ceil();

        if x1 < 0.0 || x0 >= self.buffer_width as f32 {
            return;
        }
        if x0 > x1 {
            return;
        }
        if (0..(self.buffer_height) as i16).contains(&y) {
            let stride = y as usize * self.buffer_width;

            let xl = x0.max(0.0) as usize;
            let xr = (x1 as usize).min(self.buffer_width-1);

            let span_left = stride + xl;
            let span_right = stride + xr;

            for pix in &mut self.buffer[span_left..=span_right] {
                *pix = color;
            }
        }
    }

    pub fn rasterize_with_surface(
        self,
        drawable: &'a impl Blittable<T>,
        vertices: &[TexturedVertex],
        indices: &[u16]
    ) {
        let transform = self.transform;
        self.rasterize_with_surface_iter(
            (0..indices.len())
                .step_by(3)
                .map(|ii| {
                    let idx_triple = [
                        indices[ii] as usize,
                        indices[ii+1] as usize,
                        indices[ii+2] as usize
                    ];
                    let mut vertices = idx_triple.map(|index| vertices[index as usize]);
                    let positions = transform.transform_positions(vertices.map(|it| it.position));
                    for (v, p) in vertices.iter_mut().zip(positions.iter()) {
                        v.position = (p.0 as _, p.1 as _);
                    }
                    (vertices, drawable)
                })
        );
    }

    pub fn rasterize_with_surface_iter(
        mut self,
        triangles: impl IntoIterator<Item=([TexturedVertex; 3], &'a (impl Blittable<T> + 'a))>
    ) {
        for (triangle, drawable) in triangles.into_iter() {
            let mut positions = triangle.map(|it| it.position);

            let (
                [top_pos, middle_pos, bottom_pos],
                [top_uv, middle_uv, bottom_uv]
            ) = {
                let mut uvs = triangle.map(|it| it.uv);
                for i in 0..3 {
                    // insertion sort is decently fast for this size
                    for j in (i + 1..3).rev() {
                        if positions[j].1 < positions[j - 1].1 {
                            positions.swap(j, j-1);
                            uvs.swap(j, j-1);
                        }
                    }
                }
                (
                    positions,
                    uvs.map(|it| (it.0 as f32, it.1 as f32))
                )
            };

            if top_pos.1 as i16 == middle_pos.1 as i16 {
                self.draw_flat_top_with_surface(
                    drawable,
                    top_pos,
                    middle_pos,
                    bottom_pos,
                    top_uv,
                    middle_uv,
                    bottom_uv
                );
            } else if bottom_pos.1 as i16 == middle_pos.1 as i16 {
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

    fn draw_flat_bottom_with_surface(
        &mut self,
        drawable: &'a impl Blittable<T>,
        top_pos: (f32, f32),
        middle_pos: (f32, f32),
        bottom_pos: (f32, f32),
        top_uv: (f32, f32),
        middle_uv: (f32, f32),
        bottom_uv: (f32, f32)
    ) {
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

        let (y_l_i, y_m_i) = (
            left_pos.1.ceil(),
            middle_pos.1.ceil()
        );

        if y_l_i as i16 == y_m_i as i16 {
            return;
        }

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

        let mut interpolator_0 = vec3a(middle_pos.0, middle_uv.0, middle_uv.1)
            + delta_0 * (y_m_i - middle_pos.1);
        let mut interpolator_1 = vec3a(middle_pos.0, middle_uv.0, middle_uv.1)
            + delta_1 * (y_m_i - middle_pos.1);

        for y in y_m_i as i16..y_l_i as i16 {
            self.draw_span(drawable, interpolator_0, interpolator_1, y);
            interpolator_0 += delta_0;
            interpolator_1 += delta_1;
        }
    }
    fn draw_flat_top_with_surface(
        &mut self,
        drawable: &'a impl Blittable<T>,
        top_pos: (f32, f32),
        middle_pos: (f32, f32),
        bottom_pos: (f32, f32),
        top_uv: (f32, f32),
        middle_uv: (f32, f32),
        bottom_uv: (f32, f32)
    ) {
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

        let (y_l_i, y_m_i, y_r_i) = (
            left_pos.1.ceil(),
            middle_pos.1.ceil(),
            right_pos.1.ceil()
        );

        if y_l_i as i16 == y_m_i as i16 {
            return;
        }

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

        let mut interpolator_0 = vec3a(left_pos.0, left_uv.0, left_uv.1)
            + delta_0 * (y_l_i - left_pos.1);
        let mut interpolator_1 = vec3a(right_pos.0, right_uv.0, right_uv.1)
            + delta_1 * (y_r_i - right_pos.1);

        for y in y_l_i as i16..y_m_i as i16 {
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
        y: i16
    ) {
        let x0 = interpolator_0.x.ceil();
        let x1 = interpolator_1.x.ceil();

        if x0 >= (self.buffer_width - 1) as f32 || x1 < 0.0 {
            return;
        }

        if (0..(self.buffer_height) as i16).contains(&y) {
            let stride = y as usize * self.buffer_width;
            let corr = x0 - interpolator_0.x;

            let delta = (interpolator_1.yz() - interpolator_0.yz()) /
                (interpolator_1.x - interpolator_0.x);

            let mut uv = interpolator_0.yz() + delta * corr;

            let span_left = stride + x0.clamp(0.0, (self.buffer_width - 1) as f32) as usize;
            let span_right = stride + x1.clamp(0.0, (self.buffer_width - 1) as f32) as usize;

            let dw = drawable.get_width();
            let dh = drawable.get_height();

            let drawable_buffer = drawable.get_buffer();

            if span_right < span_left {
                return;
            }

            for pix in &mut self.buffer[span_left..=span_right] {
                let uv_clamped = uv.clamp(Vec2::ZERO, vec2((dw-1) as f32, (dh-1) as f32));
                let uv_idx = (uv_clamped.y as usize) * dw + uv_clamped.x as usize;
                drawable.blend_function(pix, &drawable_buffer[uv_idx]);
                uv += delta;
            }
        }
    }
}

