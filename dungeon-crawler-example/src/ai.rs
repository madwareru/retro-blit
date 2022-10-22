use glam::{vec2};
use rand::{Rng, thread_rng};
use crate::{App, CollisionTag, Monster, Player, Position, WangTerrain};
use crate::components::DesiredVelocity;

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
    /// Just wander to a random point on a map
    /// * **player spotted far** -> go to Anxious state
    /// * **point reached** -> update point and start over
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
    pub(crate) fn update_blackboard(&mut self) {
        if let Some((_, (_, position))) = self.world.query::<(&Player, &Position)>().iter().next() {
            self.blackboard.player_position = *position;
        }
    }

    fn with_wang_data(&self, mut foo: impl FnMut(&WangTerrain)) {
        if let Some((_, (wang_data,))) = self.world.query::<(&WangTerrain,)>().iter().next() {
            foo(wang_data)
        }
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

    pub(crate) fn update_ai(&self, dt: f32) {
        const PLAYER_SPOT_DIST: f32 = 192.0;
        const PLAYER_SPOT_NEAR_DIST: f32 = 128.0;
        const UNCERTAIN_SECONDS: f32 = 2.0;

        let player_position: glam::Vec2 = self.blackboard.player_position.into();

        for (_, data) in self.world.query::<(&Monster, &Position, &mut DesiredVelocity, &mut MobState)>().iter() {
            let (_, pos, desired_velocity, state) = data;
            let p: glam::Vec2 = (*pos).into();
            match state {
                MobState::Wandering { destination, time } => {
                    let dest = (*destination).into();
                    *time -= dt;
                    if p.distance_squared(dest) < 1024.0 || *time < 0.01 {
                        let mut rng = thread_rng();
                        *time = rng.gen_range(3.0..6.0);
                        let rnd_t = rng.gen_range(0.4 ..= 0.9);
                        let pt = super::utils::get_point_on_golden_ratio_disk(rnd_t);
                        let delta = vec2(pt.0, pt.1) * 256.0;
                        let collisions_nearby = self.get_collisions_nearby(p.x, p.y);

                        *destination = match super::collision::cast_circle(
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
                    } else {
                        let dir = (dest - p).normalize_or_zero();
                        desired_velocity.x = dir.x;
                        desired_velocity.y = dir.y;
                    }
                },
                MobState::Anxious{ uncertainty } => {

                },
                MobState::Angry => {

                },
                MobState::Fight(FightPhase) => {

                }
            }
        }

        for (_, (monster, pos, desired_velocity)) in self.world.query::<(&Monster, &mut Position, &mut DesiredVelocity)>().iter() {
            self.with_wang_data(|wang_data|{
                *pos = super::collision::move_position_towards(
                    *pos,
                    vec2(desired_velocity.x * monster.speed() * dt, desired_velocity.y * monster.speed() * dt),
                    CollisionTag::All,
                    wang_data
                );
            })
        }
    }
}