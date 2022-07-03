use crate::{
    components::*,
    DemoGame
};

impl DemoGame {
    pub fn update_space_partitioning(&mut self) {
        for (_, (position, spatial_handle)) in self.ecs_world
            .query::<(&Position, &SpatialHandle)>()
            .iter() {
            self.spatial_map.set_position(spatial_handle.handle, [position.x, position.y]);
        }
        self.spatial_map.maintain();
    }
}