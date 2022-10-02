use std::collections::HashMap;

#[derive(Copy, Clone)]
pub struct Player;

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

pub struct WangTerrainEntry {
    pub terrain_id: usize,
    pub top: WangHeightMapEntry,
    pub bottom: WangHeightMapEntry,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32, pub y: i32,
}

pub struct WangTerrain {
    pub tiles: Vec<WangTerrainEntry>,
    pub props: HashMap<Position, TerrainProp>
}