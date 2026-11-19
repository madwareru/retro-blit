use retro_blit::window::{KeyCode, RetroBlitContext};
use crate::{components::*, constants::*, DemoGame, play_sound_and_forget};

impl DemoGame {
    pub fn update_player_controls(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        if let Some(player_entity) = self.player_entity {
            let mut angle_change = 0.0;
            if ctx.is_key_pressed(KeyCode::A) {
                angle_change -= dt * PLAYER_ANGULAR_SPEED_DEGREES.to_radians();
            }
            if ctx.is_key_pressed(KeyCode::D) {
                angle_change += dt * PLAYER_ANGULAR_SPEED_DEGREES.to_radians();
            }

            let mut velocity_change = 0.0;
            if ctx.is_key_pressed(KeyCode::W) {
                velocity_change += dt * PLAYER_THROTTLE;
            }
            if ctx.is_key_pressed(KeyCode::S) {
                velocity_change -= dt * PLAYER_THROTTLE;
            }

            if let Ok((_, rotation, velocity)) = self.ecs_world
                .query_one_mut::<(&Player, &mut Rotation, &mut Velocity)>(player_entity) {
                let new_angle = rotation.angle + angle_change;
                rotation.angle = new_angle;
                let new_angle_corr = new_angle - 90f32.to_radians();

                let vel_dx = new_angle_corr.cos() * velocity_change;
                let vel_dy = new_angle_corr.sin() * velocity_change;

                velocity.x = (velocity.x + vel_dx).clamp(-MAX_PLAYER_VELOCITY, MAX_PLAYER_VELOCITY);
                velocity.y = (velocity.y + vel_dy).clamp(-MAX_PLAYER_VELOCITY, MAX_PLAYER_VELOCITY);
            }
        }
    }

    pub fn update_player_fire(&mut self, ctx: &mut RetroBlitContext) {
        if !ctx.is_key_pressed(KeyCode::Space) {
            return;
        }

        if let Some(player_entity) = self.player_entity {
            let position_and_angle = self.ecs_world
                .query_one_mut::<(&Player, &Position, &Rotation, &mut FireCoolDown)>(player_entity)
                .ok()
                .and_then(|(_, &position, &rotation, FireCoolDown(amount))| {
                    if *amount <= 0.0 {
                        *amount += PLAYER_FIRE_COOL_DOWN;
                        Some((position, rotation.angle - 90f32.to_radians()))
                    } else {
                        None
                    }
                });

            if let Some((position, angle)) = position_and_angle {
                play_sound_and_forget(ctx, self.sounds.laser_shot.clone());
                self.spawn_bullet(position, angle);
            }
        }
    }

}