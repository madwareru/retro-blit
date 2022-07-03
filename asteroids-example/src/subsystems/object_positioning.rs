use crate::{
    components::*,
    constants::*,
    DemoGame
};

impl DemoGame {
    pub fn update_object_positions(&mut self, dt: f32) {
        for (_, (position, velocity)) in self.ecs_world
            .query_mut::<(&mut Position, &Velocity)>() {
            position.x += velocity.x * dt;
            position.y += velocity.y * dt;

            if position.x > MAX_X {
                position.x -= X_CORRECTION;
            } else if position.x < MIN_X {
                position.x += X_CORRECTION;
            }

            if position.y > MAX_Y {
                position.y -= Y_CORRECTION;
            } else if position.y < MIN_Y {
                position.y += Y_CORRECTION;
            }
        }
    }
}