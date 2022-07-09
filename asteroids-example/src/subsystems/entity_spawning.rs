use rand::Rng;
use crate::{
    components::*,
    constants::*,
    DemoGame
};

impl DemoGame {
    pub fn spawn_new_player(&mut self) {
        let position = Position { x: 160.0, y: 120.0 };
        let player_entity = self.ecs_world.spawn((
            Player,
            position,
            Rotation { angle: 0.0 },
            Velocity { x: 0.0, y: 0.0 },
            FireCoolDown(0.0)
        ));
        let handle = self.spatial_map.insert([position.x, position.y], player_entity);
        self.ecs_world.insert(player_entity, (SpatialHandle { handle }, )).unwrap();
        self.player_entity = Some(player_entity);
    }

    pub fn spawn_bullet(&mut self, mut position: Position, angle: f32) {
        let (dir_y, dir_x) = angle.sin_cos();

        position.x += dir_x * FIRE_OFFSET;
        position.y += dir_y * FIRE_OFFSET;

        let velocity = Velocity {
            x: dir_x * BULLET_VELOCITY,
            y: dir_y * BULLET_VELOCITY
        };

        let bullet_entity = self.ecs_world.spawn((
            Bullet,
            position,
            velocity,
            LifeSpan(BULLET_LIFE_SPAN)
        ));
        let handle = self.spatial_map.insert([position.x, position.y], bullet_entity);
        self.ecs_world.insert(bullet_entity, (SpatialHandle { handle }, )).unwrap();
    }

    pub fn spawn_asteroid(&mut self, position: Position, size: f32, generation: i32) {
        let mut rng = rand::thread_rng();
        let asteroid_entity = self.ecs_world.spawn((
            Asteroid {
                kind: match rng.gen_range(0..3) {
                    0 => AsteroidKind::Round,
                    1 => AsteroidKind::Rocky,
                    _ => AsteroidKind::Square,
                },
                size,
                generation
            },
            position,
            Velocity {
                x: (rng.gen::<f32>() - 0.5) * 2f32 * ASTEROID_VELOCITY * generation as f32,
                y: (rng.gen::<f32>() - 0.5) * 2f32 * ASTEROID_VELOCITY * generation as f32
            },
            Rotation { angle: rng.gen::<f32>() }
        ));
        let handle = self.spatial_map.insert([position.x, position.y], asteroid_entity);
        self.ecs_world.insert(asteroid_entity, (SpatialHandle { handle }, )).unwrap();
    }
}