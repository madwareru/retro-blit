use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use flat_spatial::grid::GridHandle;
use glam::{Vec2, vec2};
use hecs::Entity;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SpatialHandle {
    pub handle: GridHandle
}

#[derive(Copy, Clone)]
pub struct Player;

#[derive(Copy, Clone)]
pub struct HP(pub i32);

#[derive(Copy, Clone)]
pub struct MP(pub i32);

#[derive(Copy, Clone)]
pub struct Angle(pub f32);

#[derive(Copy, Clone)]
pub enum Monster {
    Toad,
    Kobold,
    Rat,
    Skeleton
}

impl Monster {
    pub(crate) fn max_hp(&self) -> i32 {
        match self {
            Monster::Toad => 70,
            Monster::Kobold => 40,
            Monster::Rat => 20,
            Monster::Skeleton => 80
        }
    }

    pub(crate) fn damage(&self) -> i32 {
        match self {
            Monster::Toad => 25,
            Monster::Kobold => 10,
            Monster::Rat => 3,
            Monster::Skeleton => 15
        }
    }

    pub(crate) fn fight_distance(&self) -> f32 {
        match self {
            Monster::Toad => 60.0,
            Monster::Kobold => 54.0,
            Monster::Rat => 48.0,
            Monster::Skeleton => 58.0,
        }
    }

    pub(crate) fn lost_fight_distance(&self) -> f32 {
        match self {
            Monster::Toad => 66.0,
            Monster::Kobold => 60.0,
            Monster::Rat => 52.0,
            Monster::Skeleton => 64.0,
        }
    }

    pub(crate) fn hit_distance(&self) -> f32 {
        match self {
            Monster::Toad => 54.0,
            Monster::Kobold => 45.0,
            Monster::Rat => 32.0,
            Monster::Skeleton => 50.0,
        }
    }

    pub(crate) fn speed(&self) -> f32 {
        match self {
            Monster::Toad => 24.0 * 3.0,
            Monster::Kobold => 36.0 * 3.0,
            Monster::Rat => 48.0 * 3.0,
            Monster::Skeleton => 18.0 * 3.0
        }
    }
}

pub trait CastInfo: Copy + Send + Sync {
    fn cool_down_duration() -> f32;
    fn cast_duration() -> f32;
}

#[derive(Copy, Clone)]
pub enum CastState<TCast: CastInfo> {
    NoCast(PhantomData<TCast>),
    PreCast { t: f32 },
    Cast {t: f32},
    CoolDown { t: f32 }
}

pub trait CastStateImpl<TCast: CastInfo>: Copy + Sync + Send {
    fn new() -> Self;
    fn update(&mut self, dt: f32) -> bool;
    fn try_cast(&mut self) -> bool;
    fn get_anim_info(self) -> Self;
}

impl<TCast: CastInfo> CastStateImpl<TCast> for CastState<TCast> {
    fn new() -> Self { Self::NoCast(PhantomData) }

    fn update(&mut self, dt: f32) -> bool {
        match self {
            CastState::PreCast { t } => {
                if *t <= 0.0 {
                    *self = CastState::Cast { t: TCast::cast_duration() };
                    true
                } else {
                    *t -= dt;
                    false
                }
            },
            CastState::Cast { t } => {
                if *t <= 0.0 {
                    *self = CastState::CoolDown { t: TCast::cool_down_duration() };
                    false
                } else {
                    *t -= dt;
                    false
                }
            }
            CastState::CoolDown { t } => {
                if *t <= 0.0 {
                    *self = CastState::NoCast(PhantomData);
                    false
                } else {
                    *t -= dt;
                    false
                }
            },
            _ => false
        }
    }

    fn try_cast(&mut self) -> bool {
        match self {
            CastState::NoCast(_) => {
                *self = Self::PreCast { t: TCast::cast_duration() };
                true
            }
            _ => false
        }
    }

    fn get_anim_info(self) -> Self {
        match self {
            CastState::NoCast(_) => Self::NoCast(PhantomData),
            CastState::PreCast { t } => Self::PreCast {
                t: (TCast::cast_duration() - t) / TCast::cast_duration()
            },
            CastState::Cast { t } => Self::Cast {
                t: (TCast::cast_duration() - t) / TCast::cast_duration()
            },
            CastState::CoolDown { t } => Self::CoolDown {
                t: (TCast::cool_down_duration() - t) / TCast::cool_down_duration()
            },
        }
    }
}

#[derive(Copy, Clone)]
pub struct MeleeCast {
    pub cast_angle: f32,
    pub cast_distance: f32,
    pub cast_damage: i32
}

impl CastInfo for MeleeCast {
    fn cool_down_duration() -> f32 { 0.15 }

    fn cast_duration() -> f32 { 0.1 }
}

pub type MeleeCastState = CastState<MeleeCast>;

#[derive(Copy, Clone)]
pub struct FreezeSpellCast {
    pub duration: f32,
    pub blast_range: f32,
}

impl CastInfo for FreezeSpellCast {
    fn cool_down_duration() -> f32 { 1.3 }

    fn cast_duration() -> f32 { 0.15 }
}


pub type FreezeSpellCastState = CastState<FreezeSpellCast>;

#[derive(Copy, Clone)]
pub enum Potion {
    Health,
    Mana
}

#[derive(Copy, Clone)]
pub enum TerrainProp {
    Stalagmite,
    Stalactite
}

#[derive(Copy, Clone)]
pub enum TileInfo {
    Wang(usize),
    Terrain(usize),
    Stalagmite,
    Stalactite
}

#[derive(Copy, Clone)]
pub struct WangHeightMapEntry {
    pub north_east: super::map_data::HeightMapEntry,
    pub north_west: super::map_data::HeightMapEntry,
    pub south_east: super::map_data::HeightMapEntry,
    pub south_west: super::map_data::HeightMapEntry
}

#[derive(Copy, Clone)]
pub struct WangTerrainEntry {
    pub terrain_id: usize,
    pub top: WangHeightMapEntry,
    pub bottom: WangHeightMapEntry,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Position {
    pub x: f32, pub y: f32,
}

#[derive(Copy, Clone, PartialEq)]
pub struct DesiredVelocity {
    pub x: f32, pub y: f32,
}

impl Into<Vec2> for Position {
    fn into(self) -> Vec2 {
        vec2(self.x, self.y)
    }
}

pub struct WangTerrain {
    pub tiles: Vec<WangTerrainEntry>,
    pub props: HashMap<[u16; 2], TerrainProp>,
    pub seen_tiles: HashSet<[u16; 2]>
}