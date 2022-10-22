pub fn smooth_step(edge_0: f32, edge_1: f32, x: f32) -> f32 {
    match () {
        _ if x < edge_0 => 0.0,
        _ if x >= edge_1 => 1.0,
        _ => {
            let x = (x - edge_0) / (edge_1 - edge_0);
            x * x * (3.0 - 2.0 * x)
        }.clamp(0.0, 1.0)
    }
}

#[inline(always)]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}

#[inline(always)]
pub fn dot(a: (f32, f32), b: (f32, f32)) -> f32 {
    a.0 * b.0 + a.1 * b.1
}

const GOLDEN_RATIO_CONST: f32 = 2.2360679775;

pub fn get_point_on_golden_ratio_disk(t: f32) -> (f32, f32) {
    let r = t.sqrt();
    let theta = std::f32::consts::PI * (1.0 + GOLDEN_RATIO_CONST) * t;
    (r * theta.cos(), r * theta.sin())
}