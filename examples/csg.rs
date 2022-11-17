use std::cmp::Ordering;
use glam::{Mat4, vec3, Vec3, vec4, Vec4Swizzles};
use retro_blit::math_utils::bsp_3d::{CSG};
use retro_blit::rendering::deformed_rendering::{TriangleRasterizer, Vertex};
use retro_blit::window::{RetroBlitContext, ContextHandler, WindowMode};

#[derive(Copy, Clone)]
struct Color(u8);

#[derive(Copy, Clone)]
struct Vert {
    pos: Vec3,
    normal: Vec3,
    color: Color
}

struct App {
    triangles: Vec<[Vert; 3]>
}
impl ContextHandler for App {
    fn get_window_title(&self) -> &'static str {
        "csg"
    }

    fn get_window_mode(&self) -> WindowMode {
        WindowMode::Mode256x256
    }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        let mut idx = 0;

        for i in 0..64 {
            let red = 230.0 * (i as f32) / 63.0;
            let green = 64.0 * (i as f32) / 63.0;
            let blue = 23.0 * (i as f32) / 63.0;
            ctx.set_palette(idx, [red as _, green as _, blue as _]);
            if idx < 255 {
                idx += 1;
            }
        }

        for i in 0..64 {
            let green = 255.0 * (i as f32) / 63.0;
            ctx.set_palette(idx, [0, green as _, 0]);
            if idx < 255 {
                idx += 1;
            }
        }

        for i in 0..64 {
            let blue = 255.0 * (i as f32) / 63.0;
            ctx.set_palette(idx, [0, 0, blue as _]);
            if idx < 255 {
                idx += 1;
            }
        }

        for i in 0..64 {
            let gray = 127.0 * (i as f32) / 63.0;
            ctx.set_palette(idx, [gray as _, gray as _, gray as _]);
            if idx < 255 {
                idx += 1;
            }
        }
    }

    fn update(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        ctx.clear(0);

        let forward = vec3(0.0, 0.0, 1.0);

        let matrix = Mat4::from_axis_angle(vec3(1.0, 1.0, 0.0).normalize_or_zero(), 1.2 * dt);
        for triangle in self.triangles.iter_mut() {
            for vert in triangle.iter_mut() {
                let n = matrix * vec4(vert.normal.x, vert.normal.y, vert.normal.z, 0.0);
                let p = matrix * vec4(vert.pos.x, vert.pos.y, vert.pos.z, 1.0);
                vert.normal = n.xyz();
                vert.pos = p.xyz();
            }
        }

        self.triangles.sort_by(|lhs, rhs| {
            let lhs_center_z = lhs.into_iter()
                .map(|it| if forward.dot(it.normal) > 0.0 { f32::MAX } else { it.pos.z } )
                .fold(0.0, |acc, next| acc + next) / 3.0;
            let rhs_center_z = rhs.into_iter()
                .map(|it| if forward.dot(it.normal) > 0.0 { f32::MAX } else { it.pos.z } )
                .fold(0.0, |acc, next| acc + next) / 3.0;
            if lhs_center_z > rhs_center_z {
                Ordering::Less
            } else if lhs_center_z < rhs_center_z {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });

        let light = vec3(0.4, -0.2, 1.0).normalize_or_zero();

        TriangleRasterizer::create(ctx).rasterize_with_color_iter(
            self.triangles.iter()
                .filter(|triangle| forward.dot(triangle[0].normal) <= 0.0)
                .map(|triangle| {
                    let mut color_id = triangle[0].color.0 * 64;
                    let att = (-light).dot(triangle[0].normal).max(0.0) * 63.0;
                    color_id += att as u8;
                    (
                        [
                             Vertex { position: (
                                 128.0 + triangle[0].pos.x * 40.0,
                                 128.0 + triangle[0].pos.y * 40.0
                             ) },
                             Vertex { position: (
                                 128.0 + triangle[1].pos.x * 40.0,
                                 128.0 + triangle[1].pos.y * 40.0
                             ) },
                             Vertex { position: (
                                 128.0 + triangle[2].pos.x * 40.0,
                                 128.0 + triangle[2].pos.y * 40.0
                             ) }
                         ],
                        color_id
                    )
                })
        );
    }
}

fn main() {
    let polygons = CSG::cuboid([0.0; 3], [1.0, 3.0, 1.0], Color(0))
        .union(&CSG::cuboid([0.0; 3], [3.0, 1.0, 1.0], Color(1)))
        .union(&CSG::cuboid([0.0; 3], [1.0, 1.0, 3.0], Color(2)))
        .subtract(&CSG::cuboid([0.0; 3], [2.0, 2.0, 2.0], Color(3)))
        .polygons;

    let mut triangles = Vec::new();
    for poly in polygons.iter() {
        if poly.vertices.len() >= 3 {
            triangles.push(
                [
                    Vert{
                        pos: poly.vertices[0].pos,
                        normal: poly.vertices[0].normal,
                        color: poly.shared
                    },
                    Vert{
                        pos: poly.vertices[1].pos,
                        normal: poly.vertices[1].normal,
                        color: poly.shared
                    },
                    Vert{
                        pos: poly.vertices[2].pos,
                        normal: poly.vertices[2].normal,
                        color: poly.shared
                    }
                ]
            );
            if poly.vertices.len() > 3 {
                // let's triangle fan then
                for i in 3..poly.vertices.len() {
                    triangles.push(
                        [
                            Vert{
                                pos: poly.vertices[0].pos,
                                normal: poly.vertices[0].normal,
                                color: poly.shared
                            },
                            Vert{
                                pos: poly.vertices[i-1].pos,
                                normal: poly.vertices[i-1].normal,
                                color: poly.shared
                            },
                            Vert{
                                pos: poly.vertices[i].pos,
                                normal: poly.vertices[i].normal,
                                color: poly.shared
                            }
                        ]
                    );
                }
            }
        }
    }

    retro_blit::window::start(App{ triangles })
}