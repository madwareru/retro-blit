use glam::Vec3Swizzles;
use crate::math_utils::CrossProduct2;
use crate::rendering::transform::Transform;

pub trait RayCircleIntersectionQuery where Self: Copy {
    fn ray_circle_intersection_t(self, dir: Self, center: Self, radius: f32) -> Option<f32>;
}

impl RayCircleIntersectionQuery for glam::Vec2 {
    fn ray_circle_intersection_t(self, dir: Self, center: Self, radius: f32) -> Option<f32> {
        let origin = self;
        let oc = origin - center;
        let a = dir.dot(dir);
        let b = 2.0 * oc.dot(dir);
        let c = oc.dot(oc) - radius * radius;
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            None
        } else {
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            if t < 0.0 {
                None
            } else {
                Some(t)
            }
        }
    }
}

pub trait RaySegmentIntersectionQuery where Self: Copy {
    fn ray_segment_intersection_t(self, dir: Self, segment: [Self; 2]) -> Option<f32>;
}

impl RaySegmentIntersectionQuery for glam::Vec2 {
    fn ray_segment_intersection_t(self, p_dir: Self, [p0, p1]: [Self; 2]) -> Option<f32> {
        let p0_p1= p1 - p0;
        let p0_p1_len = p0_p1.length();

        let p = self;
        let r = p_dir.normalize_or_zero();
        let q = p0;
        let s = p0_p1 / p0_p1_len;

        let r_cross_s = r.cross2(s);

        if (r_cross_s.abs()) < 0.000001 {
            // the ray is parallel to an edge
            return None;
        }

        let t = (q - p).cross2(s) / r_cross_s;
        let u = (q - p).cross2(r) / r_cross_s;

        if (0.0f32..p0_p1_len).contains(&u) && t > 0.0 {
            Some(t)
        } else {
            None
        }
    }
}

pub trait SegmentIntersectionQuery where Self: Copy {
    fn intersect(lhs: [Self; 2], rhs: [Self; 2]) -> Option<Self>;
    fn is_intersect(lhs: [Self; 2], rhs: [Self; 2]) -> bool;
}

impl SegmentIntersectionQuery for (i16, i16) {
    fn intersect(lhs: [Self; 2], rhs: [Self; 2]) -> Option<Self> {
        let [origin, end] = lhs.map(|it|
            glam::vec2(it.0 as f32 + 0.5, it.1 as f32 + 0.5)
        );
        let rhs = rhs.map(|it|
            glam::vec2(it.0 as f32 + 0.5, it.1 as f32 + 0.5)
        );
        let dir_vec = end - origin;
        let dir_length = dir_vec.length();
        if dir_length < 0.00001 {
            return None;
        }
        let dir_vec = dir_vec / dir_length;
        let t = origin.ray_segment_intersection_t(dir_vec, rhs)?;

        if t > dir_length {
            None
        } else {
            let pos = origin + dir_vec * t;
            Some((pos.x as i16, pos.y as i16))
        }
    }

    fn is_intersect(lhs: [Self; 2], rhs: [Self; 2]) -> bool {
        SegmentIntersectionQuery::intersect(lhs, rhs).is_some()
    }
}

pub trait SegmentPolyIntersectionQuery where Self: Copy {
    fn intersect(segment: [Self; 2], poly_transform: Option<Transform>, poly: &[Self], out_vec: &mut Vec<Self>);
    fn is_intersect(segment: [Self; 2], poly_transform: Option<Transform>, poly: &[Self]) -> bool;
}

impl SegmentPolyIntersectionQuery for (i16, i16) {
    fn intersect(segment: [Self; 2], poly_transform: Option<Transform>, poly: &[Self], out_vec: &mut Vec<Self>) {
        let edges = make_edges(poly_transform, poly);
        for edge in edges {
            let result = SegmentIntersectionQuery::intersect(segment, edge);
            if let Some(pt) = result {
                out_vec.push(pt);
            }
        }
    }

    fn is_intersect(segment: [Self; 2], poly_transform: Option<Transform>, poly: &[Self]) -> bool {
        let edges = make_edges(poly_transform, poly);
        for edge in edges {
            let result = SegmentIntersectionQuery::intersect(segment, edge);
            if result.is_some() {
                return true;
            }
        }
        false
    }
}

fn make_edges(poly_transform: Option<Transform>, poly: &[(i16, i16)]) -> impl Iterator<Item=[(i16, i16); 2]> + '_ {
    let edge_count = poly.len();
    let transform = poly_transform.unwrap_or_else(|| Transform::from_identity());
    (0..edge_count)
        .map(move |ix| {
            let p0 = poly[ix];
            let p0 = glam::vec3a(p0.0 as f32 + 0.5, p0.1 as f32 + 0.5, 1.0);
            let p1 = poly[(ix + 1) % edge_count];
            let p1 = glam::vec3a(p1.0 as f32 + 0.5, p1.1 as f32 + 0.5, 1.0);
            let p0 = (transform.matrix * p0).xy().floor();
            let p1 = (transform.matrix * p1).xy().floor();
            [
                (p0.x as i16, p0.y as i16),
                (p1.x as i16, p1.y as i16)
            ]
        })
}

pub trait PolyIntersectionQuery where Self: Copy {
    fn intersect(
        lhs_transform: Option<Transform>, lhs_poly: &[Self],
        rhs_transform: Option<Transform>, rhs_poly: &[Self],
        out_vec: &mut Vec<Self>
    );
    fn is_intersect(
        lhs_transform: Option<Transform>, lhs_poly: &[Self],
        rhs_transform: Option<Transform>, rhs_poly: &[Self]
    ) -> bool;
}

impl PolyIntersectionQuery for (i16, i16) {
    fn intersect(
        lhs_transform: Option<Transform>, lhs_poly: &[Self],
        rhs_transform: Option<Transform>, rhs_poly: &[Self],
        out_vec: &mut Vec<Self>
    ) {
        let edges = make_edges(lhs_transform, lhs_poly);
        for edge in edges {
            SegmentPolyIntersectionQuery::intersect(
                edge,
                rhs_transform,
                rhs_poly,
                out_vec
            );
        }
    }

    fn is_intersect(
        lhs_transform: Option<Transform>, lhs_poly: &[Self],
        rhs_transform: Option<Transform>, rhs_poly: &[Self]
    ) -> bool {
        let edges = make_edges(lhs_transform, lhs_poly);
        for edge in edges {
            if SegmentPolyIntersectionQuery::is_intersect(
                edge,
                rhs_transform,
                rhs_poly
            ) {
                return true;
            }
        }
        false
    }
}

pub trait PointInPolyQuery where Self: Copy {
    fn is_in_poly(self, poly_transform: Option<Transform>, poly: &[Self]) -> bool;
}

impl PointInPolyQuery for (i16, i16) {
    fn is_in_poly(self, poly_transform: Option<Transform>, poly: &[(i16, i16)]) -> bool {
        let p = glam::vec2(self.0 as f32 + 0.5, self.1 as f32 + 0.5);
        let p_dir = glam::vec2(1.0, 0.0);
        let poly_corr = glam::vec2(0.5, 0.49);
        let edge_count = poly.len();
        let transform = poly_transform.unwrap_or_else(|| Transform::from_identity());
        let edges = (0..edge_count)
            .map(|ix| {
                let p0 = poly[ix];
                let p0 = glam::vec3a(p0.0 as f32 + 0.5, p0.1 as f32 + 0.5, 1.0);
                let p1 = poly[(ix + 1) % edge_count];
                let p1 = glam::vec3a(p1.0 as f32 + 0.5, p1.1 as f32 + 0.5, 1.0);
                [
                    (transform.matrix * p0).xy().floor() + poly_corr,
                    (transform.matrix * p1).xy().floor() + poly_corr
                ]
            });
        let intersection_count = edges
            .fold(0, |acc, edge| {
                match p.ray_segment_intersection_t(p_dir, edge) {
                    None => acc,
                    Some(_) => acc + 1
                }
            });
        intersection_count % 2 != 0
    }
}

pub trait SegmentCircleCastQuery where Self: Copy {
    fn circle_cast_segment(self, dir: Self, radius: f32, segment: [Self; 2]) -> Option<(f32, Self)>;
}

impl SegmentCircleCastQuery for glam::Vec2 {
    fn circle_cast_segment(self, p_dir: Self, radius: f32, [p0, p1]: [Self; 2]) -> Option<(f32, Self)> {
        let (ba, oa) = (p1 - p0, self - p0);
        let (baba, bard, baoa, rdoa, oaoa) = (
            ba.dot(ba),
            ba.dot(p_dir),
            ba.dot(oa),
            p_dir.dot(oa),
            oa.dot(oa)
        );
        let (a, b, c) = (
            baba - bard * bard,
            baba * rdoa - baoa * bard,
            baba * oaoa - baoa * baoa - radius * radius *baba
        );
        let h = b * b - a * c;
        if h >= 0.0 {
            let t = (-b - h.sqrt()) / a;
            let y = baoa + t * bard;
            if  y > 0.0 && y < baba {  // body
                Some(t)
            } else { //caps
                let oc = if y <= 0.0 {oa} else {self - p1};
                let b = p_dir.dot(oc);
                let c = oc.dot(oc) - radius * radius;
                let h = b * b - c;
                if h > 0.0 {
                    let t = -b - h.sqrt();
                    Some(t)
                } else {
                    None
                }
            }
        } else {
            None
        }.and_then(|t| if t < 0.0 { None } else { Some(t) })
        .map(|t| {
            let pos = self + p_dir * t;
            let pa = pos - p0;
            let h = (pa.dot(ba) / baba).clamp(0.0, 1.0);
            (t, (pa - h * ba) / radius)
        })
    }
}