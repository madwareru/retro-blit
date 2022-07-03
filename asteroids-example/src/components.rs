use flat_spatial::grid::GridHandle;

#[derive(Copy, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32
}

#[derive(Copy, Clone)]
pub struct Velocity {
    pub x: f32,
    pub y: f32
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SpatialHandle {
    pub handle: GridHandle
}

#[derive(Copy, Clone)]
pub struct Rotation {
    pub angle: f32
}

#[derive(Copy, Clone)]
pub struct FireCoolDown(pub f32);

#[derive(Copy, Clone)]
pub struct LifeSpan(pub f32);

#[derive(Copy, Clone)]
pub struct Bullet;

#[derive(Copy, Clone)]
pub struct Player;

#[derive(Copy, Clone)]
pub struct PlayerReviveCountDown {
    pub time_remaining: f32
}

#[derive(Copy, Clone)]
pub struct PlayerScrap;

#[derive(Copy, Clone)]
pub struct Asteroid {
    pub kind: AsteroidKind,
    pub size: f32,
    pub generation: i32
}

#[derive(Copy, Clone)]
pub enum AsteroidKind {
    Round,
    Rocky,
    Square
}