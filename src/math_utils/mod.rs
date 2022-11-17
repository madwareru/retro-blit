pub mod collision_queries;
pub mod bsp_3d;

use glam::{Vec2, vec3a, Vec3A};

pub(crate) trait Barycentric2D<const N: usize> : Copy {
    fn get_barycentric_2d(p: Self, positions: [Self; N]) -> Option<[f32; N]>;
}

impl Barycentric2D<3> for (f32, f32) {
    fn get_barycentric_2d(p: Self, positions: [Self; 3]) -> Option<[f32; 3]> {
        let pt = vec3a(p.0, p.1, 0.0);
        let positions = positions.map(|it| vec3a(it.0, it.1, 0.0));
        shared_get_barycentric_triangle_2d(pt, positions)
    }
}

impl Barycentric2D<3> for (i32, i32) {
    fn get_barycentric_2d(p: Self, positions: [Self; 3]) -> Option<[f32; 3]> {
        let pt = vec3a(p.0 as f32 + 0.5, p.1 as f32 + 0.5, 0.0);
        let positions = positions.map(|it| vec3a(it.0 as f32 + 0.5, it.1 as f32 + 0.5, 0.0));
        shared_get_barycentric_triangle_2d(pt, positions)
    }
}

impl Barycentric2D<3> for crate::window::monitor_obj_loader::Vec4 {
    fn get_barycentric_2d(p: Self, positions: [Self; 3]) -> Option<[f32; 3]> {
        let pt = vec3a(p.x, p.y, 0.0);
        let positions = positions.map(|it| vec3a(it.x, it.y, 0.0));
        shared_get_barycentric_triangle_2d(pt, positions)
    }
}

impl Barycentric2D<3> for crate::window::monitor_obj_loader::Vec2 {
    fn get_barycentric_2d(p: Self, positions: [Self; 3]) -> Option<[f32; 3]> {
        let pt = vec3a(p.x, p.y, 0.0);
        let positions = positions.map(|it| vec3a(it.x, it.y, 0.0));
        shared_get_barycentric_triangle_2d(pt, positions)
    }
}

fn shared_get_barycentric_triangle_2d(pt: Vec3A, [pt0, pt1, pt2]: [Vec3A; 3]) -> Option<[f32; 3]> {
    let e0 = pt1 - pt0;
    let e1 = pt2 - pt1;

    let v0 = pt - pt0;
    let v1 = pt - pt1;
    let v2 = pt - pt2;

    let a = e0.cross2(e1);
    let (bar_u, bar_v, bar_w) = (
        v1.cross2(v2) / a,
        v2.cross2(v0) / a,
        v0.cross2(v1) / a
    );
    if (bar_u >= 0.0) && (bar_v >= 0.0) && (bar_w >= 0.0) {
        Some([bar_u, bar_v, bar_w])
    } else {
        None
    }
}

pub trait CrossProduct2 where Self: Copy {
    fn cross2(self, other: Self) -> f32;
}

impl CrossProduct2 for Vec3A {
    fn cross2(self, other: Self) -> f32 {
        self.cross(other).z
    }
}

impl CrossProduct2 for Vec2 {
    fn cross2(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }
}