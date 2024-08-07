use glam::vec2;
use rand::{Rng, thread_rng};
use retro_blit::window::RetroBlitContext;
use crate::systems_base::SystemMut;
use crate::{App, CollisionTag, FreezeStun, HP, Monster, MonsterCorpseGhost, PaletteState, Player, Position};
use crate::collision::move_position_towards;
use crate::components::{DesiredVelocity, SpatialHandle};

pub struct Blackboard {
    /// shared data on a placement of player updated each frame which can then be observed by AI agents
    pub player_position: Position
}

#[derive(Copy, Clone)]
pub enum FightPhase {
    /// Each hit does not take a place immediately, first a cool down phase should last.
    /// When **time_left** reaches a zero, fight phase switches to a hip.
    CoolDown { time_left: f32 },
    /// A phase of jump towards a player.
    /// When **end_position** reached, a distance toward player compared to **hit_distance**.
    /// If lower than or equal, the hit takes a place and player gets a damage.
    /// Whenever the player is hit or not, fight phase switches to a hop.
    Hip { start_position: Position, end_position: Position, t: f32 },
    /// A phase of jump back after the hip.
    /// When **end_position** reached, fight phase switches to a cool down.
    Hop { start_position: Position, end_position: Position, t: f32 }
}

#[derive(Copy, Clone)]
pub enum MobState {
    /// Just stay at a place for a while
    /// * **player spotted far** -> go to Anxious state
    /// * **point reached** -> update point and start over
    PreWandering { time: f32 },
    /// Just wander to a random point on a map
    /// * **player spotted far** -> go to Anxious state
    /// * **point reached or time ended** -> go to PreWandering state
    Wandering { destination: Position, time: f32 },
    /// Recently player spotted at far
    /// * **uncertainty reached 0** -> go to Angry state
    /// * **player spotted near** -> go to Angry state
    /// * **player out of sight** -> go to Wandering state
    Anxious { uncertainty: f32 },
    /// Player is near, a scent of blood is sweet
    /// * **player out of sight** -> go to Wandering state
    /// * **distance to a player is lower than fighting_range** -> go to Fight state
    Angry,
    /// Player is near enough to be hit
    /// * **player out of lost_fight_range** -> go to Angry state
    /// * **else** -> handle FightPhase
    Fight(FightPhase)
}

impl App {
    pub(crate) fn maintain_monster_hp(&mut self) {
        let spatial = &mut self.spatial_map;
        let cb = &mut self.command_buffer;

        for (e, (monster, pos, hp, sp_handle)) in self.world.query::<(&Monster, &Position, &HP, &SpatialHandle)>().iter() {
            if hp.0 == 0 {
                spatial.remove(sp_handle.handle);
                cb.spawn(
                    (
                        MonsterCorpseGhost {
                            monster: *monster,
                            life_time: 1.0,
                            frozen: self.world.get::<FreezeStun>(e).is_ok()
                        },
                        *pos
                    )
                );
                cb.despawn(e);
            }
        }

        cb.run_on(&mut self.world);
    }

    pub(crate) fn update_blackboard(&mut self) {
        let world = &mut self.world;
        let blackboard = &mut self.blackboard;

        crate::works::ai::UpdateBlackboard.run_mut(world, blackboard);

        if let Some((_, (_, position))) = self.world.query::<(&Player, &Position)>().iter().next() {
            self.blackboard.player_position = *position;
        }
    }

    pub(crate) fn update_spatial_partition(&mut self) {
        for (_, (spatial_handle, position)) in self.world.query::<(
            &SpatialHandle,
            &Position
        )>().iter() {
            self.spatial_map.set_position(spatial_handle.handle, [position.x, position.y]);
        }
        self.spatial_map.maintain();
    }


    fn get_collisions_nearby(&self, x_pos: f32, y_pos: f32) -> super::collision::CollisionVec {
        let mut coll_vec = super::collision::CollisionVec::new();
        self.with_wang_data(|wang_data| {
            super::collision::populate_collisions_data_from_position(
                &mut coll_vec,
                x_pos,
                y_pos,
                wang_data
            );
        });
        coll_vec
    }

    pub(crate) fn update_ai(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        const PLAYER_LOST_DIST: f32 = 256.0 * 2.0;
        const PLAYER_SPOT_DIST: f32 = 192.0 * 2.0;
        const PLAYER_SPOT_NEAR_DIST: f32 = 128.0 * 2.0;
        const UNCERTAIN_SECONDS: f32 = 1.0;
        const HIT_SPEED: f32 = 5.0;

        let player_position: glam::Vec2 = self.blackboard.player_position.into();

        let mut damage = 0;

        for (_, data) in self.world.query::<(&Monster, &mut Position, &mut DesiredVelocity, &mut MobState)>()
            .iter()
            .filter(|(e, _)| self.world.get::<FreezeStun>(*e).is_err())
        {
            let (monster, pos, desired_velocity, state) = data;
            let p: glam::Vec2 = (*pos).into();

            match state {
                MobState::PreWandering { time } => {
                    *time -= dt;
                    if *time <= 0.0 {
                        let mut rng = thread_rng();
                        let time = rng.gen_range(3.0..6.0);
                        let rnd_t = rng.gen_range(0.4 ..= 0.9);
                        let pt = super::utils::get_point_on_golden_ratio_disk(rnd_t);
                        let delta = vec2(pt.0, pt.1) * 256.0;
                        let collisions_nearby = self.get_collisions_nearby(p.x, p.y);

                        let destination = match super::collision::cast_circle(
                            &collisions_nearby,
                            p,
                            delta.normalize_or_zero(),
                            CollisionTag::All
                        ) {
                            None => {
                                Position {
                                    x: p.x + delta.x,
                                    y: p.y + delta.y
                                }
                            }
                            Some((t, _)) if t * t > delta.length_squared() => {
                                Position {
                                    x: p.x + delta.x,
                                    y: p.y + delta.y
                                }
                            },
                            Some((t, _)) => {
                                let delta = delta.normalize_or_zero() * t * 0.95;
                                Position {
                                    x: p.x + delta.x,
                                    y: p.y + delta.y
                                }
                            }
                        };
                        desired_velocity.x = 0.0;
                        desired_velocity.y = 0.0;
                        *state = MobState::Wandering { destination, time};
                    }
                },
                MobState::Wandering { destination, time } => {
                    let dest = (*destination).into();
                    *time -= dt;
                    if p.distance_squared(player_position) < PLAYER_LOST_DIST * PLAYER_SPOT_DIST {
                        desired_velocity.x = 0.0;
                        desired_velocity.y = 0.0;
                        *state = MobState::Anxious { uncertainty: UNCERTAIN_SECONDS }
                    } else if p.distance_squared(dest) < 1024.0 || *time < 0.01 {
                        let mut rng = thread_rng();
                        let time = rng.gen_range(1.0..2.0);
                        *state = MobState::PreWandering { time };
                    } else {
                        let dir = (dest - p).normalize_or_zero();
                        desired_velocity.x = dir.x;
                        desired_velocity.y = dir.y;
                    }
                },
                MobState::Anxious{ uncertainty } => {
                    *uncertainty -= dt;
                    desired_velocity.x = 0.0;
                    desired_velocity.y = 0.0;
                    if *uncertainty <= 0.0 || p.distance_squared(player_position) < PLAYER_SPOT_NEAR_DIST * PLAYER_SPOT_NEAR_DIST {
                        *state = MobState::Angry;
                    }
                },
                MobState::Angry => {
                    let dst_sqr = p.distance_squared(player_position);
                    if dst_sqr > PLAYER_LOST_DIST * PLAYER_LOST_DIST {
                        desired_velocity.x = 0.0;
                        desired_velocity.y = 0.0;
                        *state = MobState::PreWandering { time: 0.5 };
                    } else if dst_sqr < monster.fight_distance() * monster.fight_distance() {
                        desired_velocity.x = 0.0;
                        desired_velocity.y = 0.0;
                        *state = MobState::Fight(FightPhase::CoolDown { time_left: 0.5 })
                    } else {
                        let delta = (player_position - p).normalize_or_zero();
                        desired_velocity.x = delta.x;
                        desired_velocity.y = delta.y;
                    }
                },
                MobState::Fight(fight_phase) => {
                    match p.distance_squared(player_position) {
                        dst_sqr if dst_sqr > monster.lost_fight_distance() * monster.lost_fight_distance() => {
                            *state = MobState::Angry;
                        }
                        _ => {
                            match fight_phase {
                                FightPhase::CoolDown { time_left } => {
                                    *time_left -= dt;
                                    if *time_left < 0.0 {
                                        let delta = (player_position - p).normalize_or_zero();
                                        *fight_phase = FightPhase::Hip {
                                            start_position: Position {
                                                x: p.x,
                                                y: p.y
                                            },
                                            end_position: Position {
                                                x: p.x + delta.x * 12.0,
                                                y: p.y + delta.y * 12.0
                                            },
                                            t: 0.0
                                        }
                                    }
                                }
                                FightPhase::Hip { start_position, end_position, t } => {
                                    *t = (*t + dt * HIT_SPEED).clamp(0.0, 1.0);
                                    pos.x = super::utils::lerp(
                                        start_position.x,
                                        end_position.x,
                                        *t
                                    );
                                    pos.y = super::utils::lerp(
                                        start_position.y,
                                        end_position.y,
                                        *t
                                    );
                                    if *t > 0.9999 {
                                        {
                                            let p: glam::Vec2 = (*pos).into();
                                            let hit_dst_sqr = monster.hit_distance() * monster.hit_distance();
                                            if p.distance_squared(player_position) < hit_dst_sqr {
                                                damage += monster.damage();
                                            }
                                        }
                                        *fight_phase = FightPhase::Hop {
                                            start_position: *end_position,
                                            end_position: *start_position,
                                            t: 0.0
                                        }
                                    }
                                }
                                FightPhase::Hop { start_position, end_position, t } => {
                                    *t = (*t + dt * HIT_SPEED).clamp(0.0, 1.0);
                                    pos.x = super::utils::lerp(
                                        start_position.x,
                                        end_position.x,
                                        *t
                                    );
                                    pos.y = super::utils::lerp(
                                        start_position.y,
                                        end_position.y,
                                        *t
                                    );
                                    if *t > 0.9999 {
                                        pos.x = end_position.x;
                                        pos.y = end_position.y;
                                        *fight_phase = FightPhase::CoolDown {
                                            time_left: 0.5
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if damage != 0 {
            self.set_palette_state(ctx, PaletteState::DamageTint { t: 1.0 });
            if let Some((_, (_, hp))) =self.world.query::<(&Player, &mut HP)>().iter().next() {
                hp.0 = (hp.0 - damage).max(0);
            }
        }

        for (_, (monster, pos, desired_velocity)) in self.world.query::<(&Monster, &mut Position, &mut DesiredVelocity)>()
            .iter()
            .filter(|(e, _)| self.world.get::<FreezeStun>(*e).is_err())
        {
            let dir = vec2(desired_velocity.x, desired_velocity.y) * monster.speed() * dt;
            self.with_wang_data(|wang_data|{
                let (new_pos, _) = move_position_towards(*pos, dir, CollisionTag::All, wang_data);
                *pos = new_pos;
            })
        }
    }
}
