use std::collections::{HashMap, HashSet};
use glam::{Vec2, vec2};

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