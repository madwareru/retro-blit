pub const STAR_SKY_SPRITE_BYTES: &[u8] = include_bytes!("assets/star_sky.im256");
pub const STAR_FLICKER_PACE: f32 = 0.1;
pub const PLAYER_ANGULAR_SPEED_DEGREES: f32 = 90.0f32;
pub const MAX_PLAYER_VELOCITY: f32 = 70.0;
pub const FIRE_OFFSET: f32 = 18.0;
pub const BULLET_VELOCITY: f32 = 210.0;
pub const ASTEROID_VELOCITY: f32 = 40.0;
pub const BULLET_LIFE_SPAN: f32 = 2.0;
pub const PLAYER_SCRAP_LIFE_SPAN: f32 = 0.6;
pub const PLAYER_THROTTLE: f32 = 65.0;
pub const PLAYER_COLOR: u8 = 80;
pub const PLAYER_REVIVE_TIME: f32 = 2.0;
pub const PLAYER_FIRE_COOL_DOWN: f32 = 0.2;
pub const ASTEROID_COLORS: &[u8] = &[81, 82, 83];
pub const MAX_ASTEROID_GENERATIONS: i32 = 3;
pub const SUB_ASTEROIDS_COUNT: u8 = 3;

// constants to wrap objects around screen borders
pub const MAX_X: f32 = 360.0;
pub const MIN_X: f32 = -40.0;
pub const MAX_Y: f32 = 280.0;
pub const MIN_Y: f32 = -40.0;
pub const X_CORRECTION: f32 = 400.0;
pub const Y_CORRECTION: f32 = 320.0;

pub const PLAYER_POINTS: &[(i16, i16)] = &[
    (-8, 0),
    (0, -18),
    (7, 0),
    (0, -4)
];

pub const PLAYER_SCRAP_POINTS: &[(i16, i16)] = &[
    (-4, 2),
    (0, -2),
    (3, 2)
];

pub const ROUND_ASTEROID_POINTS: &[(i16, i16)] = &[
    (-4, -15),
    (10, -9),
    (13, 6),
    (2, 14),
    (-14, 8),
    (-13, -10)
];

pub const ROCKY_ASTEROID_POINTS: &[(i16, i16)] = &[
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

pub const SQUARE_ASTEROID_POINTS: &[(i16, i16)] = &[
    (-9, -19),
    (16, -11),
    (6, 13),
    (-15, -6)
];