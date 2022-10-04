use std::collections::{HashMap, HashSet};

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

pub struct WangTerrain {
    pub tiles: Vec<WangTerrainEntry>,
    pub props: HashMap<[u16; 2], TerrainProp>,
    pub seen_tiles: HashSet<[u16; 2]>
}