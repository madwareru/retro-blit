use std::collections::{HashMap, HashSet};
use retro_blit::rendering::blittable::BufferProvider;
use crate::ai::MobState;
use crate::components::{Angle, DesiredVelocity, FreezeSpellCastState, HP, MeleeCastState, MP, Player, Position, WangHeightMapEntry, WangTerrain, WangTerrainEntry};
use crate::{CastStateImpl, FreezeSpellCast, MeleeCast};

#[derive(Copy, Clone, PartialEq)]
pub enum HeightMapEntry { Water, Floor, Wall }

pub struct MapData {
    height_map: Vec<HeightMapEntry>,
    monsters: HashMap<[u16; 2], super::components::Monster>,
    potions: HashMap<[u16; 2], super::components::Potion>,
    terrain_props: HashMap<[u16; 2], super::components::TerrainProp>,
    player_entry_point: [u16; 2]
}

impl MapData {
    pub const WIDTH: usize = 183;
    pub const HEIGHT: usize = 183;

    const WATER_ID: u8 = 1;
    const WALL_ID: u8 = 2;
    const FLOOR_ID: u8 = 3;
    const STALAGMITE_ID: u8 = 4;
    const WATER_STALAGMITE_ID: u8 = 5;
    const STALACTITE_ID: u8 = 6;
    const MANA_POTION_ID: u8 = 9;

    const PLAYER_ENTRY_POINT_ID: u8 = 21;

    const HEALTH_POTION_ID: u8 = 26;

    const KOBOLD_MONSTER_ID: u8 = 28;
    const RAT_MONSTER_ID: u8 = 29;
    const SKELETON_MONSTER_ID: u8 = 30;
    const TOAD_MONSTER_ID: u8 = 31;

    pub fn load(bytes: &[u8]) -> Self {
        let (_, image_data) = retro_blit
            ::format_loaders
            ::im_256
            ::Image
            ::load_from(bytes)
            .unwrap();
        let buffer = image_data.get_buffer();

        let mut terrain_props = HashMap::new();
        let mut potions = HashMap::new();
        let mut monsters = HashMap::new();
        let mut height_map = Vec::with_capacity(Self::WIDTH * Self::HEIGHT);
        let mut player_entry_point = [(Self::WIDTH / 2) as u16, (Self::HEIGHT / 2) as u16];

        for idx in 0..buffer.len() {
            let x = idx % Self::WIDTH;
            let y = idx / Self::WIDTH;
            let height_map_entry = match buffer[idx] {
                Self::WATER_ID => HeightMapEntry::Water,
                Self::WALL_ID => HeightMapEntry::Wall,
                Self::FLOOR_ID => HeightMapEntry::Floor,
                Self::STALAGMITE_ID => {
                    terrain_props.insert([x as u16, y as u16], super::components::TerrainProp::Stalagmite);
                    HeightMapEntry::Floor
                },
                Self::WATER_STALAGMITE_ID => {
                    terrain_props.insert([x as u16, y as u16], super::components::TerrainProp::Stalagmite);
                    HeightMapEntry::Water
                },
                Self::STALACTITE_ID => {
                    terrain_props.insert([x as u16, y as u16], super::components::TerrainProp::Stalactite);
                    HeightMapEntry::Floor
                },
                Self::MANA_POTION_ID => {
                    potions.insert([x as u16, y as u16], super::components::Potion::Mana);
                    HeightMapEntry::Floor
                },
                Self::HEALTH_POTION_ID => {
                    potions.insert([x as u16, y as u16], super::components::Potion::Health);
                    HeightMapEntry::Floor
                },
                Self::PLAYER_ENTRY_POINT_ID => {
                    player_entry_point = [x as u16, y as u16];
                    HeightMapEntry::Floor
                },
                Self::KOBOLD_MONSTER_ID => {
                    monsters.insert([x as u16, y as u16], super::components::Monster::Kobold);
                    HeightMapEntry::Floor
                },
                Self::RAT_MONSTER_ID => {
                    monsters.insert([x as u16, y as u16], super::components::Monster::Rat);
                    HeightMapEntry::Floor
                },
                Self::TOAD_MONSTER_ID => {
                    monsters.insert([x as u16, y as u16], super::components::Monster::Toad);
                    HeightMapEntry::Floor
                },
                Self::SKELETON_MONSTER_ID => {
                    monsters.insert([x as u16, y as u16], super::components::Monster::Skeleton);
                    HeightMapEntry::Floor
                },
                _ => panic!("found unknown id! {}", buffer[idx])
            };
            height_map.push(height_map_entry);
        }

        Self { height_map, monsters, potions, terrain_props, player_entry_point }
    }

    pub fn populate_world(&self, world: &mut hecs::World) {
        let mut wang_terrain = WangTerrain {
            tiles: Vec::with_capacity((MapData::WIDTH-1) * (MapData::HEIGHT-1)),
            props: HashMap::new(),
            seen_tiles: HashSet::new()
        };
        for j in 0..MapData::HEIGHT-1 {
            for i in 0..MapData::WIDTH-1 {
                let idx_north_west = j * MapData::WIDTH + i;
                let idx_north_east = idx_north_west + 1;
                let idx_south_west = idx_north_west + MapData::WIDTH;
                let idx_south_east = idx_south_west + 1;

                let bottom = WangHeightMapEntry {
                    north_east: self.height_map[idx_north_east],
                    north_west: self.height_map[idx_north_west],
                    south_east: self.height_map[idx_south_east],
                    south_west: self.height_map[idx_south_west]
                };

                let top = WangHeightMapEntry {
                    north_east: match bottom.north_east {
                        HeightMapEntry::Water => HeightMapEntry::Floor,
                        _ => bottom.north_east
                    },
                    north_west: match bottom.north_west {
                        HeightMapEntry::Water => HeightMapEntry::Floor,
                        _ => bottom.north_west
                    },
                    south_east: match bottom.south_east {
                        HeightMapEntry::Water => HeightMapEntry::Floor,
                        _ => bottom.south_east
                    },
                    south_west: match bottom.south_west {
                        HeightMapEntry::Water => HeightMapEntry::Floor,
                        _ => bottom.south_west
                    },
                };
                let terrain_id = 0;
                wang_terrain.tiles.push(WangTerrainEntry { terrain_id, bottom, top });
            }
        }
        for (&pos, &prop) in self.terrain_props.iter() {
            wang_terrain.props.insert(pos, prop);
        }
        world.spawn((wang_terrain,));

        for (&pos, &potion) in self.potions.iter() {
            let position = Position { x: pos[0] as f32 * 64.0, y: pos[1] as f32 * 64.0 };
            world.spawn((position, potion));
        }

        for (&pos, &monster) in self.monsters.iter() {
            let position = Position { x: pos[0] as f32 * 64.0, y: pos[1] as f32 * 64.0 };
            let desired_velocity = DesiredVelocity {
                x: 0.0,
                y: 0.0
            };
            world.spawn((
                monster,
                position,
                desired_velocity,
                HP(monster.max_hp()),
                MobState::Wandering { destination: position, time: 0.0 }
            ));
        }

        let player_position = Position {
            x: self.player_entry_point[0] as f32 * 64.0,
            y: self.player_entry_point[1] as f32 * 64.0
        };
        world.spawn((
            Player,
            player_position,
            HP(100),
            MP(100),
            Angle(0.0),
            FreezeSpellCast {
                duration: 4.0,
                blast_range: 128.0
            },
            FreezeSpellCastState::new(),
            MeleeCast {
                cast_angle: 45.0f32.to_radians(),
                cast_distance: 48.0,
                cast_damage: 10
            },
            MeleeCastState::new()
        ));
    }
}