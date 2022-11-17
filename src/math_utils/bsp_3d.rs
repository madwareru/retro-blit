use std::collections::VecDeque;
use std::iter::FromIterator;
use glam::{Vec3, vec3};

const PLANE_EPSILON: f32 = 1e-5f32;

mod polygon_alignment {
    pub const COPLANAR: u8 = 0b00;
    pub const FRONT: u8 = 0b01;
    pub const BACK: u8 = 0b10;
    pub const SPANNING: u8 = 0b11;
}

#[derive(Copy, Clone)]
pub struct Vertex { pub pos: Vec3, pub normal: Vec3 }

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3) -> Self { Self { pos: position, normal } }

    pub fn flip(&mut self) { self.normal *= -1.0; }

    pub fn lerp(&self, v: Vertex, t: f32) -> Self {
        Self { pos: self.pos.lerp(v.pos, t), normal: self.normal.lerp(v.normal, t) }
    }
}

#[derive(Copy, Clone)]
pub struct Plane { pub normal: Vec3, pub w: f32 }

impl Plane {
    pub fn new(normal: Vec3, w: f32) -> Self { Plane { normal, w } }

    pub fn from_points(a: Vec3, b: Vec3, c: Vec3) -> Plane {
        let n = (b - a).cross(c - a).normalize_or_zero();
        Plane { normal: n, w: n.dot(a) }
    }

    pub fn flip(&mut self) {
        self.normal *= -1.0;
        self.w = -self.w;
    }

    pub fn split_polygon<TShared: Copy>(
        &self,
        polygon: Polygon<TShared>,
        coplanar_front: Option<&mut Vec<Polygon<TShared>>>,
        coplanar_back: Option<&mut Vec<Polygon<TShared>>>,
        front: &mut Vec<Polygon<TShared>>,
        back: &mut Vec<Polygon<TShared>>
    ) {
        let mut polygon_type = polygon_alignment::COPLANAR;

        let polygon_length = polygon.vertices.len();
        let mut types = Vec::with_capacity(polygon_length);

        for i in 0..polygon_length {
            let t = self.normal.dot(polygon.vertices[i].pos) - self.w;
            let p_type = if t < -PLANE_EPSILON {
                polygon_alignment::BACK
            } else if t > PLANE_EPSILON {
                polygon_alignment::FRONT
            } else {
                polygon_alignment::COPLANAR
            };
            polygon_type |= p_type;
            types.push(p_type);
        }

        match polygon_type {
            polygon_alignment::COPLANAR => {
                let proj = self.normal.dot(polygon.plane.normal);
                match (coplanar_front, coplanar_back) {
                    (Some(f), Some(b)) => if proj > 0f32 {
                        f.push(polygon.clone());
                    } else {
                        b.push(polygon.clone());
                    },
                    (Some(f), None) => f.push(polygon.clone()),
                    (None, Some(b)) => b.push(polygon.clone()),
                    _ => if proj > 0f32 {
                        front.push(polygon.clone());
                    } else {
                        back.push(polygon.clone());
                    }
                }
            },
            polygon_alignment::FRONT => front.push(polygon.clone()),
            polygon_alignment::BACK => back.push(polygon.clone()),
            polygon_alignment::SPANNING => {
                let mut f = Vec::with_capacity(polygon_length);
                let mut b = Vec::with_capacity(polygon_length);

                for i in 0..polygon_length {
                    let j = (i + 1) % polygon_length;
                    let ti = types[i];
                    let tj = types[j];
                    let f_vi = polygon.vertices[i];
                    let b_vi = polygon.vertices[i];
                    let vj = polygon.vertices[j];

                    if ti != polygon_alignment::BACK {
                        f.push(f_vi);
                    }
                    if ti != polygon_alignment::FRONT {
                        if ti != polygon_alignment::BACK {
                            b.push(b_vi);
                        } else {
                            b.push(b_vi);
                        }
                    }

                    if (ti | tj) == polygon_alignment::SPANNING {
                        let t = (self.w - self.normal.dot(polygon.vertices[i].pos)) /
                            self.normal.dot(vj.pos - polygon.vertices[i].pos);
                        let fv = polygon.vertices[i].lerp(vj, t);

                        f.push(fv);
                        b.push(fv);
                    }
                }

                if f.len() >= 3 {
                    front.push(Polygon::new(f, polygon.shared));
                }

                if b.len() >= 3 {
                    back.push(Polygon::new(b, polygon.shared));
                }
            },
            _ => unreachable!()
        }
    }
}

#[derive(Clone)]
pub struct Polygon<TShared: Copy> {
    pub vertices: Vec<Vertex>,
    pub plane: Plane,
    pub shared: TShared
}

impl<TShared: Copy> Polygon<TShared> {
    pub fn new(vertices: Vec<Vertex>, shared: TShared) -> Self {
        let plane = Plane::from_points(vertices[0].pos, vertices[1].pos, vertices[2].pos);
        Polygon { vertices, shared, plane }
    }

    pub fn flip(&mut self) {
        self.vertices.reverse();
        for v in &mut self.vertices { v.flip(); }
        self.plane.flip();
    }
}

#[derive(Clone)]
pub struct Node<TShared: Copy> {
    plane: Option<Plane>,
    front: Option<Box<Node<TShared>>>,
    back: Option<Box<Node<TShared>>>,
    polygons: Vec<Polygon<TShared>>
}

impl<TShared: Copy> Node<TShared> {
    pub fn new(polygons: Option<Vec<Polygon<TShared>>>) -> Self {
        let mut node = Node {
            plane: None,
            front: None,
            back: None,
            polygons: vec![]
        };

        match polygons {
            None => {}
            Some(polys) => {
                node.build(&polys);
            }
        }

        return node;
    }

    pub fn invert(&mut self) {
        for i in 0..self.polygons.len() {
            self.polygons[i].flip();
        }

        match &mut self.plane {
            None => {}
            Some(p) => {p.flip()}
        }

        match &mut self.front {
            None => {}
            Some(f) => {f.invert()}
        }

        match &mut self.back {
            None => {}
            Some(b) => {b.invert()}
        }

        std::mem::swap(&mut self.front, &mut self.back);
    }

    pub fn clip_polygons(&self, polygons: Vec<Polygon<TShared>>) -> Vec<Polygon<TShared>> {
        if self.plane.is_none() {
            return polygons;
        }

        let mut front = vec![];
        let mut back = vec![];

        if let Some(p) = &self.plane {
            for i in 0..polygons.len() {
                p.split_polygon(polygons[i].clone(), None, None, &mut front, &mut back);
            }
        }

        if let Some(f) = &self.front { front = f.clip_polygons(front); }
        if let Some(b) = &self.back { back = b.clip_polygons(back); } else { back.clear(); }

        front.extend(back);

        front
    }

    pub fn clip_to(&mut self, bsp: &Node<TShared>) {
        let mut queue = VecDeque::new();
        queue.push_front(self);
        while let Some(node) = queue.pop_back() {
            node.polygons = bsp.clip_polygons(node.polygons.clone());

            if let Some(f) = &mut node.front { queue.push_front(f); }
            if let Some(b) = &mut node.back { queue.push_front(b); }
        }
    }

    pub fn all_polygons(&self) -> Vec<Polygon<TShared>> {
        let mut polygons = Vec::new();

        let mut queue = VecDeque::new();
        queue.push_front(self);

        while let Some(node) = queue.pop_back() {
            polygons.extend(node.polygons.clone());
            if let Some(f) = &node.front { queue.push_front(f); }
            if let Some(b) = &node.back { queue.push_front(b); }
        }

        return polygons;
    }

    fn build(&mut self, polygons: &Vec<Polygon<TShared>>) {
        if polygons.len() == 0 {
            return;
        }

        if self.plane.is_none() {
            self.plane = Some(polygons[0].plane.clone());
        }

        let mut front = vec![];
        let mut back = vec![];

        for i in 0..polygons.len() {
            if let Some(p) = &self.plane {
                p.split_polygon(
                    polygons[i].clone(),
                    Some(&mut self.polygons),
                    None,
                    &mut front,
                    &mut back
                );
            }
        }

        if front.len() > 0 {
            if self.front.is_none() {
                self.front = Some(Box::new(Node::new(None)));
            }
            self.front.as_mut().unwrap().build(&front);
        }

        if back.len() > 0 {
            if self.back.is_none() {
                self.back = Some(Box::new(Node::new(None)));
            }
            self.back.as_mut().unwrap().build(&back);
        }
    }
}

#[derive(Clone)]
pub struct CSG<TShared: Copy> { pub polygons: Vec<Polygon<TShared>> }

impl<TShared: Copy> CSG<TShared> {
    pub fn from_polygons(p: impl IntoIterator<Item=Polygon<TShared>>) -> Self {
        Self { polygons: Vec::from_iter(p.into_iter()) }
    }

    pub fn union(&self, csg: &Self) -> Self {
        let mut a = Node::new(Some(self.clone().polygons));
        let mut b = Node::new(Some(csg.clone().polygons));

        a.clip_to(&b);
        b.clip_to(&a);
        b.invert();
        b.clip_to(&a);
        b.invert();
        a.build(&b.all_polygons());

        Self::from_polygons(a.all_polygons())
    }

    pub fn subtract(&self, csg: &Self) -> Self {
        let mut a = Node::new(Some(self.clone().polygons));
        let mut b = Node::new(Some(csg.clone().polygons));

        a.invert();
        a.clip_to(&b);
        b.clip_to(&a);
        b.invert();
        b.clip_to(&a);
        b.invert();
        a.build(&b.all_polygons());
        a.invert();

        Self::from_polygons(a.all_polygons())
    }

    pub fn intersect(&self, csg: &Self) -> Self {
        let mut a = Node::new(Some(self.clone().polygons));
        let mut b = Node::new(Some(csg.clone().polygons));

        a.invert();
        b.clip_to(&a);
        b.invert();
        a.clip_to(&b);
        b.clip_to(&a);
        a.build(&b.all_polygons());
        a.invert();

        Self::from_polygons(a.all_polygons())
    }

    pub fn cuboid(center: [f32; 3], extents: [f32; 3], shared: TShared) -> Self {
        let c = Vec3::from_array(center);

        let cube_topology = [
            ([0, 4, 6, 2], [-1, 0, 0]),
            ([1, 3, 7, 5], [1, 0, 0]),
            ([0, 1, 5, 4], [0, -1, 0]),
            ([2, 6, 7, 3], [0, 1, 0]),
            ([0, 2, 3, 1], [0, 0, -1]),
            ([4, 5, 7, 6], [0, 0, 1])
        ];

        let cube_polygons: Vec<_> = cube_topology.iter().map(|t| {
            let (position, normal) = t;

            let vertices: Vec<Vertex> = position.iter().map(|i| -> Vertex {
                let vp = Vec3::new(
                    c.x + extents[0] * (2f32 * (if !!(i & 1) != 0 { 1f32 } else { 0f32 }) - 1f32),
                    c.y + extents[1] * (2f32 * (if !!(i & 2) != 0 { 1f32 } else { 0f32 }) - 1f32),
                    c.z + extents[2] * (2f32 * (if !!(i & 4) != 0 { 1f32 } else { 0f32 }) - 1f32)
                );

                let vn = Vec3::from_array(normal.map(|n| n as f32));

                return Vertex::new(vp, vn);
            }).collect();

            return Polygon::new(vertices, shared);
        }).collect();

        Self::from_polygons(cube_polygons)
    }

    pub fn sphere(center: [f32; 3], radius: f32, slices: i32, stacks: i32, shared: TShared) -> Self {
        let c = Vec3::from_array(center);

        let mut sphere_polygons = vec![];

        for i in 0..slices {
            for j in 0..stacks {
                let i = i as f32;
                let j = j as f32;
                let slices = slices as f32;
                let stacks = stacks as f32;

                let mut vertices: Vec<Vertex> = vec![];

                let mut count = 0;

                let mut fn_vertex = |mut theta: f32, mut phi: f32| {
                    theta *= std::f32::consts::PI * 2f32;
                    phi *= std::f32::consts::PI;

                    let dir = Vec3::new(
                        theta.cos() * phi.sin(),
                        phi.cos(),
                        theta.sin() * phi.sin()
                    );

                    count += 1;
                    vertices.push(Vertex::new(c + (dir * radius ), dir));
                };

                fn_vertex(i / slices, j / stacks);

                if j > 0f32 {
                    fn_vertex((i + 1f32) / slices, j / stacks);
                }

                if j < stacks - 1f32 {
                    fn_vertex((i + 1f32) / slices, (j + 1f32) / stacks)
                }

                fn_vertex(i / slices, (j + 1f32) / stacks);

                let normal_avg = (&vertices[vertices.len()-count..])
                    .iter()
                    .fold(vec3(0.0, 0.0, 0.0), |acc, next| acc + next.normal) / count as f32;

                for i in vertices.len()-count..vertices.len() {
                    vertices[i].normal = normal_avg;
                }

                sphere_polygons.push(Polygon::new(vertices, shared));
            }
        }

        Self::from_polygons(sphere_polygons)
    }

    pub fn cylinder(radius: f32, slices: i32, start: [f32; 3], end: [f32; 3], shared: TShared) -> Self {
        let s = Vec3::from_array(start);
        let e = Vec3::from_array(end);
        let ray = e - s;
        let r = radius;

        let axis_z = ray.normalize_or_zero();
        let is_y = if axis_z.y.abs() > 0.5 { 1f32 } else { 0f32 };
        let axis_x = Vec3::new(is_y, -is_y, 0f32).cross(axis_z).normalize_or_zero();

        let axis_y = axis_x.cross(axis_z).normalize_or_zero();
        let v_start = Vertex::new(s, -axis_z);
        let v_end = Vertex::new(e, axis_z.normalize_or_zero());

        let mut cylinder_polygons = vec![];

        let point = |stack: f32, slice: f32, normal_blend: f32| -> Vertex {
            let angle = slice * std::f32::consts::PI * 2f32;
            let out = axis_x * angle.cos() + axis_y * angle.sin();
            let pos = s + ray * stack + out * r;
            let normal = out * (1f32 - normal_blend.abs()) + axis_z * normal_blend;

            let p = vec3(pos.x.max(0.0), pos.y.max(0.0), pos.z.max(0.0));

            let n = vec3(normal.x.max(0.0), normal.y.max(0.0), normal.z.max(0.0));

            let v = Vertex::new(p, n);

            return v;
        };

        for i in 0..slices {
            let i = i as f32;
            let slices = slices as f32;

            let t0 = i / slices;
            let t1 = (i + 1f32) / slices;

            cylinder_polygons.push(Polygon::new(vec![v_start.clone(), point(0f32, t0, -1f32), point(0f32, t1, -1f32)], shared));
            cylinder_polygons.push(Polygon::new(vec![point(0f32, t1, 0f32), point(0f32, t0, 0f32), point(1f32, t0, 0f32), point(1f32, t1, 0f32)], shared));
            cylinder_polygons.push(Polygon::new(vec![v_end.clone(), point(1f32, t1, 1f32), point(1f32, t0, 1f32)], shared));
        }

        Self::from_polygons(cylinder_polygons)
    }
}