use flat_spatial::grid::GridHandle;
use hecs::Entity;
use rand::Rng;
use retro_blit::{
    format_loaders::im_256,
    rendering::{
        blittable::BlitBuilder,
        BlittableSurface,
        shapes::fill_rectangle,
        fonts::font_align::{HorizontalAlignment, VerticalAlignment},
        fonts::tri_spaced::{Font, TextDrawer}
    },
    window::{RetroBlitContext, ScrollDirection, ScrollKind, WindowMode},
    rendering::deformed_rendering::Vertex,
    rendering::tessellation::PathTessellator,
    rendering::transform::Transform,
    math_utils::collision_queries::{PointInPolyQuery, SegmentPolyIntersectionQuery, PolyIntersectionQuery},
    rendering::bresenham::LineRasterizer,
    rendering::deformed_rendering::TriangleRasterizer,
    window::KeyCode
};

const STAR_SKY_SPRITE_BYTES: &[u8] = include_bytes!("assets/star_sky.im256");
const STAR_FLICKER_PACE: f32 = 0.1;
const PLAYER_ANGULAR_SPEED_DEGREES: f32 = 90.0f32;
const MAX_PLAYER_VELOCITY: f32 = 70.0;
const FIRE_OFFSET: f32 = 18.0;
const BULLET_VELOCITY: f32 = 210.0;
const ASTEROID_VELOCITY: f32 = 40.0;
const BULLET_LIFE_SPAN: f32 = 2.0;
const PLAYER_SCRAP_LIFE_SPAN: f32 = 0.6;
const PLAYER_THROTTLE: f32 = 65.0;
const PLAYER_COLOR: u8 = 80;
const PLAYER_REVIVE_TIME: f32 = 2.0;
const PLAYER_FIRE_COOL_DOWN: f32 = 0.2;
const ASTEROID_COLORS: &[u8] = &[81, 82, 83];
const MAX_ASTEROID_GENERATIONS: i32 = 3;
const SUB_ASTEROIDS_COUNT: u8 = 3;

// constants to wrap objects around screen borders
const MAX_X: f32 = 360.0;
const MIN_X: f32 = -40.0;
const MAX_Y: f32 = 280.0;
const MIN_Y: f32 = -40.0;
const X_CORRECTION: f32 = 400.0;
const Y_CORRECTION: f32 = 320.0;

fn get_asteroid_color(kind: AsteroidKind) -> u8 {
    ASTEROID_COLORS[match kind {
        AsteroidKind::Round => 0,
        AsteroidKind::Rocky => 1,
        AsteroidKind::Square => 2
    }]
}

const PLAYER_POINTS: &[(i16, i16)] = &[
    (-8, 0),
    (0, -18),
    (7, 0),
    (0, -4)
];

const PLAYER_SCRAP_POINTS: &[(i16, i16)] = &[
    (-4, 2),
    (0, -2),
    (3, 2)
];

const ROUND_ASTEROID_POINTS: &[(i16, i16)] = &[
    (-4, -15),
    (10, -9),
    (13, 6),
    (2, 14),
    (-14, 8),
    (-13, -10)
];

const ROCKY_ASTEROID_POINTS: &[(i16, i16)] = &[
    (6, -20),
    (9, -8),
    (18, -8),
    (21, 5),
    (7, 14),
    (0, 8),
    (-9, 14),
    (-19, 4),
    (-9, -7),
    (-13, -16)
];

const SQUARE_ASTEROID_POINTS: &[(i16, i16)] = &[
    (-9, -19),
    (16, -11),
    (6, 13),
    (-15, -6)
];

struct Demo {
    player_hp : u8,
    player_entity: Option<hecs::Entity>,
    bump_allocator: bumpalo::Bump,
    spatial_map: flat_spatial::DenseGrid<hecs::Entity>,
    player_vertices: Vec<Vertex>,
    player_indices: Vec<u16>,
    player_scrap_vertices: Vec<Vertex>,
    player_scrap_indices: Vec<u16>,
    round_asteroid_vertices: Vec<Vertex>,
    round_asteroid_indices: Vec<u16>,
    rocky_asteroid_vertices: Vec<Vertex>,
    rocky_asteroid_indices: Vec<u16>,
    square_asteroid_vertices: Vec<Vertex>,
    square_asteroid_indices: Vec<u16>,
    ecs_world: hecs::World,
    palette: Vec<[u8; 3]>,
    star_sky_sprite: BlittableSurface,
    font: Font,
    flicker_dt_accumulated: f32
}

#[derive(Copy, Clone)]
struct Position {
    x: f32,
    y: f32
}

#[derive(Copy, Clone)]
struct Velocity {
    x: f32,
    y: f32
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct SpatialHandle {
    handle: GridHandle
}

#[derive(Copy, Clone)]
struct Rotation {
    angle: f32
}

#[derive(Copy, Clone)]
struct FireCoolDown(f32);

#[derive(Copy, Clone)]
struct LifeSpan(f32);

#[derive(Copy, Clone)]
struct Bullet;
#[derive(Copy, Clone)]
struct Player;

#[derive(Copy, Clone)]
struct PlayerReviveCountDown {
    time_remaining: f32
}

#[derive(Copy, Clone)]
struct PlayerScrap;
#[derive(Copy, Clone)]
struct Asteroid {
    kind: AsteroidKind,
    size: f32,
    generation: i32
}

#[derive(Copy, Clone)]
enum AsteroidKind {
    Round,
    Rocky,
    Square
}

impl retro_blit::window::ContextHandler for Demo {
    fn get_window_title(&self) -> &'static str { "asteroids game" }

    fn get_window_mode(&self) -> WindowMode { WindowMode::ModeX }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        PathTessellator::new().tessellate_polyline_fill(
            &mut self.player_vertices,
            &mut self.player_indices,
            &PLAYER_POINTS
        );

        PathTessellator::new().tessellate_polyline_fill(
            &mut self.player_scrap_vertices,
            &mut self.player_scrap_indices,
            &PLAYER_SCRAP_POINTS
        );

        PathTessellator::new().tessellate_polyline_fill(
            &mut self.square_asteroid_vertices,
            &mut self.square_asteroid_indices,
            &SQUARE_ASTEROID_POINTS
        );

        PathTessellator::new().tessellate_polyline_fill(
            &mut self.round_asteroid_vertices,
            &mut self.round_asteroid_indices,
            &ROUND_ASTEROID_POINTS
        );

        PathTessellator::new().tessellate_polyline_fill(
            &mut self.rocky_asteroid_vertices,
            &mut self.rocky_asteroid_indices,
            &ROCKY_ASTEROID_POINTS
        );

        for (idx, &palette_color) in self.palette.iter().enumerate() {
            ctx.set_palette(idx as u8, palette_color);
        }

        self.start_new_game();
    }

    fn update(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        self.update_star_sky(ctx, dt);
        self.update_object_positions(dt);
        self.update_space_partitioning();
        self.update_life_spans(dt);
        self.update_bullet_collisions();
        self.update_player_collisions();
        self.update_fire_cool_downs(dt);
        self.update_revive_cool_down(dt);
        self.update_player_controls(ctx, dt);
        self.update_player_fire(ctx);

        if (self.game_lost() || self.game_won()) && ctx.is_key_pressed(KeyCode::Enter) {
            self.start_new_game();
        }

        self.render(ctx);
    }
}

impl Demo {
    pub fn new() -> Self {
        let (palette, star_sky_sprite) =
            im_256::Image::load_from(STAR_SKY_SPRITE_BYTES).unwrap();
        let font = Font::default_font_small().unwrap();
        Self {
            player_hp: 0,
            player_entity: None,
            bump_allocator: bumpalo::Bump::new(),
            spatial_map: flat_spatial::DenseGrid::new(32),
            player_vertices: Vec::new(),
            player_indices: Vec::new(),
            player_scrap_vertices: Vec::new(),
            player_scrap_indices: Vec::new(),
            round_asteroid_vertices: Vec::new(),
            round_asteroid_indices: Vec::new(),
            rocky_asteroid_vertices: Vec::new(),
            rocky_asteroid_indices: Vec::new(),
            square_asteroid_vertices: Vec::new(),
            square_asteroid_indices: Vec::new(),
            ecs_world: hecs::World::new(),
            palette,
            star_sky_sprite,
            font,
            flicker_dt_accumulated: 0.0
        }
    }

    fn update_star_sky(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        self.flicker_dt_accumulated += dt;
        while self.flicker_dt_accumulated >= STAR_FLICKER_PACE {
            self.flicker_dt_accumulated -= STAR_FLICKER_PACE;
            ctx.scroll_palette(
                ScrollKind::Range { start_idx: 2, len: 34 },
                ScrollDirection::Forward
            );
            ctx.scroll_palette(
                ScrollKind::Range { start_idx: 36, len: 44 },
                ScrollDirection::Forward
            )
        }
    }

    fn update_player_controls(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
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

    fn update_object_positions(&mut self, dt: f32) {
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

    fn update_space_partitioning(&mut self) {
        for (_, (position, spatial_handle)) in self.ecs_world
            .query::<(&Position, &SpatialHandle)>()
            .iter() {
            self.spatial_map.set_position(spatial_handle.handle, [position.x, position.y]);
        }
        self.spatial_map.maintain();
    }

    fn update_bullet_collisions(&mut self) {
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
                        32.0
                    ) {
                        if let Some((other_position, &other_entity)) = self.spatial_map.get(other_h) {
                            match (
                                self.ecs_world.get::<Asteroid>(other_entity),
                                self.ecs_world.get::<Rotation>(other_entity)
                            ) {
                                (Ok(asteroid), Ok(rotation)) => {
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

    fn blow_asteroid(&mut self, asteroid_entity: Entity, position: Position, asteroid: Asteroid) {
        if asteroid.generation < MAX_ASTEROID_GENERATIONS {
            for _ in 0..SUB_ASTEROIDS_COUNT {
                self.spawn_asteroid(position, asteroid.size * 0.5, asteroid.generation + 1);
            }
        }
        let spatial_handle = *self.ecs_world.get::<SpatialHandle>(asteroid_entity).unwrap();
        self.kill_entity(asteroid_entity, spatial_handle);
    }

    fn update_player_collisions(&mut self) {
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
                    for (other_position, &other_entity) in self.spatial_map
                        .query_around([player_position.x, player_position.y], 32.0)
                        .filter_map(|it | self.spatial_map.get(it.0)) {
                        match (
                            self.ecs_world.get::<Asteroid>(other_entity),
                            self.ecs_world.get::<Rotation>(other_entity)
                        ) {
                            (Ok(asteroid), Ok(rotation)) => {
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

    fn kill_entity(&mut self, entity: hecs::Entity, spatial_handle: SpatialHandle) {
        self.spatial_map.remove(spatial_handle.handle);
        self.ecs_world.despawn(entity).unwrap();
    }

    fn update_revive_cool_down(&mut self, dt: f32) {
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

    fn spawn_new_player(&mut self) {
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

    fn update_fire_cool_downs(&mut self, dt: f32) {
        for (_, (FireCoolDown(amount),)) in self.ecs_world
            .query_mut::<(&mut FireCoolDown,)>() {
            if *amount <= 0.0 {
                return;
            }
            *amount -= dt;
        }
    }

    fn update_life_spans(&mut self, dt: f32) {
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

    fn update_player_fire(&mut self, ctx: &mut RetroBlitContext) {
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
                self.spawn_bullet(position, angle);
            }
        }
    }

    fn spawn_bullet(&mut self, mut position: Position, angle: f32) {
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

    fn spawn_asteroid(&mut self, position: Position, size: f32, generation: i32) {
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

    fn render(&mut self, ctx: &mut RetroBlitContext) {
        { // blit star sky
            BlitBuilder::create(ctx, &self.star_sky_sprite).blit();
        }

        { // draw player bullets
            for (_, (_, position, velocity)) in self.ecs_world
                .query::<(&Bullet, &Position, &Velocity)>()
                .iter() {
                let [p0, p1] = Self::make_bullet_segment(position, velocity);
                LineRasterizer::create(ctx)
                    .from(p0)
                    .to(p1)
                    .rasterize(PLAYER_COLOR);
            }
        }

        { // draw player
            for (_, (_, pos, rotation)) in self.ecs_world
                .query::<(&Player, &Position, &Rotation)>()
                .iter() {
                TriangleRasterizer::create(ctx)
                    .with_transform(
                        Transform::from_angle_and_translation(
                            rotation.angle,
                            pos.x as i16,
                            pos.y as i16
                        )
                    )
                    .rasterize_with_color(
                        PLAYER_COLOR,
                        &self.player_vertices,
                        &self.player_indices
                    );
            }
        }

        { // draw player scrap
            for (_, (_, pos, rotation)) in self.ecs_world
                .query::<(&PlayerScrap, &Position, &Rotation)>()
                .iter() {
                TriangleRasterizer::create(ctx)
                    .with_transform(
                        Transform::from_angle_and_translation(
                            rotation.angle,
                            pos.x as i16,
                            pos.y as i16
                        )
                    )
                    .rasterize_with_color(
                        PLAYER_COLOR,
                        &self.player_scrap_vertices,
                        &self.player_scrap_indices
                    );
            }
        }

        { // draw asteroids
            for (_, (&Asteroid { kind, size, .. }, pos, rotation)) in self.ecs_world
                .query::<(&Asteroid, &Position, &Rotation)>()
                .iter() {
                let (vertices, indices) = match kind {
                    AsteroidKind::Round => (
                        &self.round_asteroid_vertices,
                        &self.round_asteroid_indices
                    ),
                    AsteroidKind::Rocky => (
                        &self.rocky_asteroid_vertices,
                        &self.rocky_asteroid_indices
                    ),
                    AsteroidKind::Square => (
                        &self.square_asteroid_vertices,
                        &self.square_asteroid_indices
                    )
                };
                let color = get_asteroid_color(kind);
                TriangleRasterizer::create(ctx)
                    .with_transform(
                        Transform::from_angle_translation_scale(
                            rotation.angle,
                            (pos.x as i16, pos.y as i16),
                            (size, size)
                        )
                    )
                    .rasterize_with_color(
                        color,
                        vertices,
                        indices
                    );
            }
        }

        { // draw lives indicator
            let lives_text = format!("Lives: {}", self.player_hp);

            fill_rectangle(ctx, 136, 0, 48, 12, 85);
            self.font.draw_text_in_box(
                ctx,
                136, 0,
                48, 12,
                HorizontalAlignment::Center,
                VerticalAlignment::Center,
                &lives_text,
                Some(PLAYER_COLOR)
            );

            if self.game_lost() {
                self.font.draw_text_in_box(
                    ctx,
                    0, 0,
                    320, 240,
                    HorizontalAlignment::Center,
                    VerticalAlignment::Center,
                    "Game over. Press enter to restart",
                    Some(PLAYER_COLOR)
                );
            } else if self.game_won() {
                self.font.draw_text_in_box(
                    ctx,
                    0, 0,
                    320, 240,
                    HorizontalAlignment::Center,
                    VerticalAlignment::Center,
                    "You win! Press enter to restart",
                    Some(PLAYER_COLOR)
                );
            }
        }
    }

    fn game_won(&mut self) -> bool {
        self.ecs_world.query::<(&Asteroid,)>().iter().count() == 0
    }

    fn game_lost(&mut self) -> bool {
        self.player_hp == 0
    }

    fn make_bullet_segment(position: &Position, velocity: &Velocity) -> [(i16, i16); 2] {
        let p0 = (position.x as i16, position.y as i16);
        let p1 = (
            (position.x + velocity.x / 85.0) as i16,
            (position.y + velocity.y / 85.0) as i16
        );
        [p0, p1]
    }

    fn start_new_game(&mut self) {
        self.player_hp = 5;

        self.ecs_world.clear();

        self.spawn_new_player();

        let mut rng = rand::thread_rng();
        for _ in 0..5 {
            self.spawn_asteroid(
                Position {
                    x: rng.gen_range(0..320) as f32,
                    y: rng.gen_range(0..240) as f32
                },
                1.0 + rng.gen::<f32>(),
                1
            );
        }
    }
}

fn main() {
    retro_blit::window::start(Demo::new());
}