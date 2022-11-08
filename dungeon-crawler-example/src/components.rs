use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use flat_spatial::grid::GridHandle;
use glam::{Vec2, vec2};
use hecs::{CommandBuffer, Entity};
use rand::{Rng, thread_rng};

#[derive(Copy, Clone)]
pub struct Projectile<TCast: CastInfo, TProjectileBehaviour: ProjectileBehaviour<TCast>>{
    pub caster: Entity,
    pub behaviour: TProjectileBehaviour,
    pub(crate) _phantom_data: PhantomData<TCast>
}

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
            Monster::Toad => 62.0,
            Monster::Kobold => 56.0,
            Monster::Rat => 50.0,
            Monster::Skeleton => 60.0,
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

pub trait CastInfo: Copy + Send + Sync + 'static {
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

pub trait CastStateImpl<TCast: CastInfo>: Copy + Sync + Send + 'static {
    fn new() -> Self;
    fn update(&mut self, dt: f32) -> bool;
    fn try_cast(&mut self) -> bool;
    fn get_anim_info(self) -> Self;
}

pub trait ProjectileBehaviour<TCast: CastInfo>: Copy + Sync + Send + 'static {
    fn collide(position: Position, cast: TCast, cb: &mut CommandBuffer);
    fn make_particle(x: f32, y: f32) -> Particle;
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

pub struct FreezeSpellBlast;

pub trait PeriodicStatus: Copy + Send + Sync + 'static {
    fn update(&mut self, dt: f32) -> bool;
    fn on_status_off(e: Entity, cb: &mut CommandBuffer) {
        cb.remove::<(Self,)>(e);
    }
}

macro_rules! derive_periodic_status(
    ($status_type:ident) => {
        impl PeriodicStatus for $status_type{
            fn update(&mut self, dt: f32) -> bool {
                let v: &mut f32 = self.deref_mut();
                if *v <= 0.0 {
                    false
                } else {
                    *v -= dt;
                    true
                }
            }
        }

        impl Deref for $status_type {
            type Target = f32;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $status_type {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    }
);

#[derive(Copy, Clone)]
pub struct FreezeStun(pub f32);

derive_periodic_status!(FreezeStun);

#[derive(Copy, Clone)]
pub struct DamageTint(pub f32);

derive_periodic_status!(DamageTint);

#[derive(Copy, Clone)]
pub struct MonsterCorpseGhost {
    pub monster: Monster,
    pub life_time: f32,
    pub frozen: bool
}

impl PeriodicStatus for MonsterCorpseGhost {
    fn update(&mut self, dt: f32) -> bool {
        self.life_time -= dt;
        self.life_time > 0.0
    }
    fn on_status_off(e: Entity, cb: &mut CommandBuffer) {
        cb.despawn(e);
    }
}

#[derive(Copy, Clone)]
pub struct Particle {
    pub color_id: u8,
    pub life_time: f32,
    pub x: f32,
    pub y: f32,
    pub h: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub velocity_h: f32
}

impl PeriodicStatus for Particle {
    fn update(&mut self, dt: f32) -> bool {
        self.life_time -= dt;
        if self.life_time <= 0.0 {
            false
        } else {
            self.x += self.velocity_x * dt;
            self.y += self.velocity_y * dt;
            self.h += self.velocity_h * dt;
            true
        }
    }
    fn on_status_off(e: Entity, cb: &mut CommandBuffer) {
        cb.despawn(e);
    }
}

#[derive(Copy, Clone)]
pub struct FreezeSpellProjectile;

impl ProjectileBehaviour<FreezeSpellCast> for FreezeSpellProjectile {
    fn collide(position: Position, cast: FreezeSpellCast, cb: &mut CommandBuffer) {
        cb.spawn(
            (
                FreezeSpellBlast,
                position,
                cast
            )
        );
    }

    fn make_particle(x: f32, y: f32) -> Particle {
        let mut rng = thread_rng();
        Particle {
            color_id: 35,
            life_time: 0.6,
            x: x + rng.gen_range(-3.0..=3.0),
            y: y + rng.gen_range(-3.0..=3.0),
            h: - 12.0 + rng.gen_range(-3.0..=3.0),
            velocity_x: rng.gen_range(-3.0..=3.0),
            velocity_y: rng.gen_range(-3.0..=3.0),
            velocity_h: rng.gen_range(-3.0..=3.0)
        }
    }
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