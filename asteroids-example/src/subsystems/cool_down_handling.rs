use crate::{
    components::*,
    DemoGame
};

impl DemoGame {
    pub fn update_fire_cool_downs(&mut self, dt: f32) {
        for (_, (FireCoolDown(amount),)) in self.ecs_world
            .query_mut::<(&mut FireCoolDown,)>() {
            if *amount <= 0.0 {
                return;
            }
            *amount -= dt;
        }
    }

    pub fn update_revive_cool_down(&mut self, dt: f32) {
        let mut elapsed_countdown = None;
        for (e, (PlayerReviveCountDown { time_remaining},)) in self.ecs_world
            .query_mut::<(&mut PlayerReviveCountDown,)>() {
            if *time_remaining <= 0.0 {
                elapsed_countdown = Some(e);
                break;
            }
            *time_remaining -= dt;
        }
        if let Some(e) = elapsed_countdown {
            self.spawn_new_player();
            self.ecs_world.despawn(e).unwrap();
        }
    }
}