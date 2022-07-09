use hecs::Entity;
use rand::Rng;
use retro_blit::{
    math_utils::collision_queries::{
        PointInPolyQuery,
        PolyIntersectionQuery,
        SegmentPolyIntersectionQuery
    },
    rendering::transform::Transform
};
use crate::{
    components::*,
    constants::*,
    DemoGame
};

impl DemoGame {
    pub fn update_bullet_collisions(&mut self) {
        let bump_allocator = std::mem::take(&mut self.bump_allocator);
        self.bump_allocator = {
            {
                let mut hit_asteroids = bumpalo::collections::Vec::new_in(&bump_allocator);
                let mut hit_bullets = bumpalo::collections::Vec::new_in(&bump_allocator);
                for (bullet_entity, (_, position, velocity, &spatial_handle)) in self.ecs_world
                    .query::<(&Bullet, &Position, &Velocity, &SpatialHandle)>()
                    .iter() {
                    let mut found_hits = false;
                    let segment = Self::make_bullet_segment(position, velocity);
                    for (other_h, _) in self.spatial_map.query_around(
                        [position.x, position.y],
                        64.0
                    ) {
                        if let Some((_, &other_entity)) = self.spatial_map.get(other_h) {
                            match (
                                self.ecs_world.get::<Asteroid>(other_entity),
                                self.ecs_world.get::<Position>(other_entity),
                                self.ecs_world.get::<Rotation>(other_entity)
                            ) {
                                (Ok(asteroid), Ok(other_position), Ok(rotation)) => {
                                    let transform = Transform::from_angle_translation_scale(
                                        rotation.angle,
                                        (other_position.x as i16, other_position.y as i16),
                                        (asteroid.size, asteroid.size)
                                    );

                                    let poly = match asteroid.kind {
                                        AsteroidKind::Round => ROUND_ASTEROID_POINTS,
                                        AsteroidKind::Rocky => ROCKY_ASTEROID_POINTS,
                                        AsteroidKind::Square => SQUARE_ASTEROID_POINTS
                                    };
                                    if segment[0].is_in_poly(Some(transform), poly) ||
                                        segment[1].is_in_poly(Some(transform), poly) ||
                                        SegmentPolyIntersectionQuery::is_intersect(segment, Some(transform), poly)
                                    {
                                        if !hit_asteroids.contains(&other_entity) {
                                            hit_asteroids.push(other_entity);
                                            found_hits = true;
                                        }
                                    }
                                },
                                _ => ()
                            }
                        }
                    }
                    if found_hits {
                        hit_bullets.push((bullet_entity, spatial_handle));
                    }
                }
                for asteroid_entity in hit_asteroids.drain(..) {
                    let position = *self.ecs_world.get::<Position>(asteroid_entity).unwrap();
                    let asteroid = *self.ecs_world.get::<Asteroid>(asteroid_entity).unwrap();
                    self.blow_asteroid(asteroid_entity, position, asteroid);
                }
                for (bullet_entity, spatial_handle) in hit_bullets.drain(..) {
                    self.kill_entity(bullet_entity, spatial_handle);
                }
            }
            bump_allocator
        };
    }

    pub fn update_player_collisions(&mut self) {
        let bump_allocator = std::mem::take(&mut self.bump_allocator);
        self.bump_allocator = {
            if let Some(player_entity) = self.player_entity {
                let mut hit_asteroids = bumpalo::collections::Vec::new_in(&bump_allocator);
                let prh = self.ecs_world
                    .query_one_mut::<(&Player, &Position, &Rotation, &SpatialHandle)>(player_entity)
                    .map(|(_, p, r, h)| (*p, *r, *h));
                if let Ok((player_position, player_rotation, player_handle)) = prh {
                    let player_transform = Transform::from_angle_and_translation(
                        player_rotation.angle,
                        player_position.x as i16,
                        player_position.y as i16
                    );
                    for (_, &other_entity) in self.spatial_map
                        .query_around([player_position.x, player_position.y], 64.0)
                        .filter_map(|it | self.spatial_map.get(it.0)) {
                        match (
                            self.ecs_world.get::<Asteroid>(other_entity),
                            self.ecs_world.get::<Position>(other_entity),
                            self.ecs_world.get::<Rotation>(other_entity)
                        ) {
                            (Ok(asteroid), Ok(other_position), Ok(rotation)) => {
                                let asteroid_transform = Transform::from_angle_translation_scale(
                                    rotation.angle,
                                    (other_position.x as i16, other_position.y as i16),
                                    (asteroid.size, asteroid.size)
                                );

                                let poly = match asteroid.kind {
                                    AsteroidKind::Round => ROUND_ASTEROID_POINTS,
                                    AsteroidKind::Rocky => ROCKY_ASTEROID_POINTS,
                                    AsteroidKind::Square => SQUARE_ASTEROID_POINTS
                                };

                                if PolyIntersectionQuery::is_intersect(
                                    Some(player_transform), PLAYER_POINTS,
                                    Some(asteroid_transform), poly
                                ) {
                                    if !hit_asteroids.contains(&other_entity) {
                                        hit_asteroids.push(other_entity);
                                    }
                                }
                            },
                            _ => ()
                        }
                    }
                    if !hit_asteroids.is_empty() {
                        self.kill_entity(player_entity, player_handle);
                        for asteroid_entity in hit_asteroids.drain(..) {
                            let position = *self.ecs_world.get::<Position>(asteroid_entity).unwrap();
                            let asteroid = *self.ecs_world.get::<Asteroid>(asteroid_entity).unwrap();
                            self.blow_asteroid(asteroid_entity, position, asteroid);
                        }
                        for _ in 0..5 {
                            self.spawn_player_scrap(player_position);
                        }
                        if self.player_hp > 1 {
                            self.spawn_player_respawn_countdown();
                        }
                        self.player_hp -= 1;
                        self.player_entity = None;
                    }
                }
            }
            bump_allocator
        };
    }

    fn spawn_player_respawn_countdown(&mut self) {
        self.ecs_world.spawn((
            PlayerReviveCountDown {
                time_remaining: PLAYER_REVIVE_TIME
            },
        ));
    }

    fn spawn_player_scrap(&mut self, position: Position) {
        let mut rng = rand::thread_rng();
        let asteroid_entity = self.ecs_world.spawn((
            PlayerScrap,
            position,
            Velocity {
                x: (rng.gen::<f32>() - 0.5) * ASTEROID_VELOCITY * 4f32,
                y: (rng.gen::<f32>() - 0.5) * ASTEROID_VELOCITY * 4f32
            },
            Rotation { angle: rng.gen::<f32>() },
            LifeSpan(PLAYER_SCRAP_LIFE_SPAN)
        ));
        let handle = self.spatial_map.insert([position.x, position.y], asteroid_entity);
        self.ecs_world.insert(asteroid_entity, (SpatialHandle { handle }, )).unwrap();
    }

    fn blow_asteroid(&mut self, asteroid_entity: Entity, position: Position, asteroid: Asteroid) {
        if asteroid.generation < MAX_ASTEROID_GENERATIONS {
            for _ in 0..SUB_ASTEROIDS_COUNT {
                self.spawn_asteroid(position, asteroid.size * 0.5, asteroid.generation + 1);
            }
        }
        let spatial_handle = *self.ecs_world.get::<SpatialHandle>(asteroid_entity).unwrap();
        self.kill_entity(asteroid_entity, spatial_handle);
    }
}