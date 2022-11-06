use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use glam::{Vec2, vec2};
use hecs::Entity;
use crate::App;

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

pub trait CastImpl: Copy + Send + Sync {
    fn cast(
        self,
        app: &mut App,
        caster: Entity,
        cast_position: Position,
        cast_angle: Angle,
    );

    fn cool_down_duration() -> f32;
    fn pre_cast_duration() -> f32;
}

#[derive(Copy, Clone)]
pub enum CastState<TCast: CastImpl> {
    NoCast(PhantomData<TCast>),
    PreCast { t: f32 },
    CoolDown { t: f32 }
}

pub trait CastStateImpl<TCast: CastImpl>: Copy + Sync + Send {
    fn new() -> Self;
    fn update(&mut self, entity: Entity, pos: Position, ang: Angle, cast: TCast, app: &mut App, dt: f32);
    fn try_cast(&mut self) -> bool;
    fn get_anim_info(self) -> Self;
}

impl<TCast: CastImpl> CastStateImpl<TCast> for CastState<TCast> {
    fn new() -> Self { Self::NoCast(PhantomData) }

    fn update(&mut self, entity: Entity, pos: Position, ang: Angle, cast: TCast, app: &mut App, dt: f32) {
        match self {
            CastState::PreCast { t } => {
                if *t <= 0.0 {
                    cast.cast(app, entity, pos, ang);
                    *self = CastState::CoolDown { t: TCast::cool_down_duration() };
                } else {
                    *t -= dt;
                }
            }
            CastState::CoolDown { t } => {
                if *t <= 0.0 {
                    *self = CastState::NoCast(PhantomData);
                } else {
                    *t -= dt;
                }
            },
            _ => ()
        }
    }

    fn try_cast(&mut self) -> bool {
        match self {
            CastState::NoCast(_) => {
                *self = Self::PreCast { t: TCast::pre_cast_duration() };
                true
            }
            _ => false
        }
    }

    fn get_anim_info(self) -> Self {
        match self {
            CastState::NoCast(_) => Self::NoCast(PhantomData),
            CastState::PreCast { t } => Self::PreCast {
                t: (TCast::pre_cast_duration() - t) / TCast::pre_cast_duration()
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

impl CastImpl for MeleeCast {
    fn cast(
        self,
        app: &mut App,
        caster: Entity,
        cast_position: Position,
        cast_angle: Angle
    ) {
        app.cast_melee(self, caster, cast_position, cast_angle);
    }

    fn cool_down_duration() -> f32 { 0.15 }

    fn pre_cast_duration() -> f32 { 0.1 }
}

pub type MeleeCastState = CastState<MeleeCast>;

#[derive(Copy, Clone)]
pub struct FreezeSpellCast {
    pub duration: f32,
    pub blast_range: f32,
}

impl CastImpl for FreezeSpellCast {
    fn cast(
        self,
        app: &mut App,
        caster: Entity,
        cast_position: Position,
        cast_angle: Angle
    ) {
        app.cast_freeze_spell(self, caster, cast_position, cast_angle);
    }

    fn cool_down_duration() -> f32 { 0.3 }

    fn pre_cast_duration() -> f32 { 0.15 }
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