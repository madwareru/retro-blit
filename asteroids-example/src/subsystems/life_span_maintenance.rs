use crate::{
    components::*,
    DemoGame
};

impl DemoGame {
    pub fn update_life_spans(&mut self, dt: f32) {
        let bump_allocator = std::mem::take(&mut self.bump_allocator);
        self.bump_allocator = {
            {
                let mut dead_entities = bumpalo::collections::Vec::new_in(&bump_allocator);
                for (entity, (LifeSpan(amount), &spatial_handle)) in self.ecs_world
                    .query_mut::<(&mut LifeSpan, &SpatialHandle)>() {
                    if *amount <= 0.0 {
                        dead_entities.push((entity, spatial_handle));
                        continue;
                    }
                    *amount -= dt;
                }
                for (entity, spatial_handle) in dead_entities.drain(..) {
                    self.kill_entity(entity, spatial_handle);
                }
            }
            bump_allocator
        };
    }
}