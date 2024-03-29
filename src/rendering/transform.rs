use glam::{Mat3A, vec2, vec3a};

#[derive(Copy, Clone)]
pub struct Transform {
    translation: (i16, i16),
    rotation: f32,
    scale: (f32, f32),
    pub(crate) matrix: Mat3A
}

impl Transform {
    pub fn from_identity() -> Transform {
        let translation = (0, 0);
        let rotation = 0.0;
        let scale = (1.0, 1.0);
        let matrix = Mat3A::IDENTITY;
        Self {
            translation,
            rotation,
            scale,
            matrix
        }
    }
    pub fn from_translation(x: i16, y: i16) -> Self {
        let translation = (x, y);
        let rotation = 0.0;
        let scale = (1.0, 1.0);
        let matrix = Mat3A::from_translation(vec2(x as f32, y as f32));
        Self {
            translation,
            rotation,
            scale,
            matrix
        }
    }
    pub fn from_angle_and_translation(angle: f32, x: i16, y: i16) -> Self {
        let translation = (x, y);
        let rotation = angle;
        let scale = (1.0, 1.0);
        let matrix = Mat3A::from_translation(vec2(x as f32, y as f32)) *
            Mat3A::from_angle(rotation);
        Self {
            translation,
            rotation,
            scale,
            matrix
        }
    }
    pub fn from_angle_translation_scale(angle: f32, translation: (i16, i16), scale: (f32, f32)) -> Self {
        let rotation = angle;
        let matrix = Mat3A::from_translation(vec2(translation.0 as f32, translation.1 as f32)) *
            Mat3A::from_angle(rotation) *
            Mat3A::from_scale(vec2(scale.0, scale.1));
        Self {
            translation,
            rotation,
            scale,
            matrix
        }
    }
    pub fn with_rotation(self, angle: f32) -> Self {
        Self {
            rotation: angle,
            matrix: Mat3A::from_translation(vec2(self.translation.0 as f32, self.translation.1 as f32)) *
                Mat3A::from_angle(angle) *
                Mat3A::from_scale(vec2(self.scale.0, self.scale.1)),
            ..self
        }
    }
    pub fn with_translation(self, translation: (i16, i16)) -> Self {
        Self {
            translation,
            matrix: Mat3A::from_translation(vec2(translation.0 as f32, translation.1 as f32)) *
                Mat3A::from_angle(self.rotation) *
                Mat3A::from_scale(vec2(self.scale.0, self.scale.1)),
            ..self
        }
    }
    pub fn with_scale(self, scale: (f32, f32)) -> Self {
        Self {
            scale,
            matrix: Mat3A::from_translation(vec2(self.translation.0 as f32, self.translation.1 as f32)) *
                Mat3A::from_angle(self.rotation) *
                Mat3A::from_scale(vec2(scale.0, scale.1)),
            ..self
        }
    }

    fn actualize_matrix(&mut self) {
        self.matrix =
            Mat3A::from_translation(vec2(self.translation.0 as f32, self.translation.1 as f32)) *
                Mat3A::from_angle(self.rotation) *
                Mat3A::from_scale(vec2(self.scale.0, self.scale.1))
    }

    pub fn set_scale(&mut self, scale: (f32, f32)) {
        self.scale = scale;
        self.actualize_matrix();
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
        self.actualize_matrix();
    }

    pub fn set_translation(&mut self, translation: (i16, i16)) {
        self.translation = translation;
        self.actualize_matrix();
    }

    pub fn transform_positions(&self, positions: [(f32, f32); 3]) -> [(f32, f32); 3] {
        positions.map(|it| {
            let p = self.matrix * vec3a(it.0, it.1, 1.0);
            (p.x, p.y)
        })
    }
}

