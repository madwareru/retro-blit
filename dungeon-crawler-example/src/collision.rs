use smallvec::SmallVec;
use retro_blit::math_utils::collision_queries::SegmentCircleCastQuery;
use crate::{HeightMapEntry, MapData, Position, WangTerrain, WangTerrainEntry};

const SKIN: f32 = 2.5;
const RADIUS: f32 = 24.0;
const MINIMAL_DISTANCE: f32 = 0.001;
const MOVE_ITERATIONS: u8 = 8;

#[derive(Copy, Clone, PartialEq)]
pub enum CollisionTag {
    Water,
    Wall,
    All
}

impl Default for CollisionTag {
    fn default() -> Self {
        CollisionTag::Wall
    }
}

#[derive(Copy, Clone, Default)]
pub struct CollisionRegion {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub tag: CollisionTag
}

pub type CollisionVec = SmallVec<[CollisionRegion; 18]>;

pub fn populate_collisions(
    collision_vec: &mut CollisionVec,
    wang_entry: &WangTerrainEntry,
    x_offset: f32,
    y_offset: f32
) {
    let mut water_wang = 0b0000;
    let mut wall_wang = 0b0000;
    match wang_entry.bottom.north_east {
        HeightMapEntry::Water => water_wang += 0b0001,
        HeightMapEntry::Floor => {}
        HeightMapEntry::Wall => wall_wang += 0b0001
    }
    match wang_entry.bottom.north_west {
        HeightMapEntry::Water => water_wang += 0b0010,
        HeightMapEntry::Floor => {}
        HeightMapEntry::Wall => wall_wang += 0b0010
    }
    match wang_entry.bottom.south_east {
        HeightMapEntry::Water => water_wang += 0b0100,
        HeightMapEntry::Floor => {}
        HeightMapEntry::Wall => wall_wang += 0b0100
    }
    match wang_entry.bottom.south_west {
        HeightMapEntry::Water => water_wang += 0b1000,
        HeightMapEntry::Floor => {}
        HeightMapEntry::Wall => wall_wang += 0b1000
    }
    match wall_wang {
        0b0001 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Wall
                }
            );
        },
        0b1110 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Wall
                }
            );
        },

        0b0010 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset,
                    x1: x_offset,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Wall
                }
            );
        },
        0b1101 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset,
                    x1: x_offset,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Wall
                }
            );
        },

        0b0100 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset + 64.0,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Wall
                }
            );
        },
        0b1011 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset + 64.0,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Wall
                }
            );
        },


        0b1000 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset,
                    y0: y_offset + 32.0,
                    x1: x_offset + 32.0,
                    y1: y_offset + 64.0,
                    tag: CollisionTag::Wall
                }
            );
        },
        0b0111 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset,
                    y0: y_offset + 32.0,
                    x1: x_offset + 32.0,
                    y1: y_offset + 64.0,
                    tag: CollisionTag::Wall
                }
            );
        },

        0b0101 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset,
                    x1: x_offset + 32.0,
                    y1: y_offset + 64.0,
                    tag: CollisionTag::Wall
                }
            );
        },
        0b1010 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset,
                    x1: x_offset + 32.0,
                    y1: y_offset + 64.0,
                    tag: CollisionTag::Wall
                }
            );
        },
        0b0011 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset,
                    y0: y_offset + 32.0,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Wall
                }
            );
        },
        0b1100 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset,
                    y0: y_offset + 32.0,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Wall
                }
            );
        },

        0b0110 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Wall
                }
            );
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset,
                    y0: y_offset + 32.0,
                    x1: x_offset + 32.0,
                    y1: y_offset + 64.0,
                    tag: CollisionTag::Wall
                }
            );
        },
        0b1001 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset,
                    y0: y_offset + 32.0,
                    x1: x_offset + 32.0,
                    y1: y_offset,
                    tag: CollisionTag::Wall
                }
            );
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset + 64.0,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Wall
                }
            );
        },
        _ => ()
    }
    match water_wang {
        0b0001 if wall_wang != 0b1110 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Water
                }
            );
        },
        0b1110 if wall_wang != 0b0001  => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Water
                }
            );
        },
        0b0010 if wall_wang != 0b1101  => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset,
                    x1: x_offset,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Water
                }
            );
        },
        0b1101 if wall_wang != 0b0010  => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset,
                    x1: x_offset,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Water
                }
            );
        },
        0b0100 if wall_wang != 0b1011  => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset + 64.0,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Water
                }
            );
        },
        0b1011 if wall_wang != 0b0100  => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset + 64.0,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Water
                }
            );
        },
        0b1000 if wall_wang != 0b0111  => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset,
                    y0: y_offset + 32.0,
                    x1: x_offset + 32.0,
                    y1: y_offset + 64.0,
                    tag: CollisionTag::Water
                }
            );
        },
        0b0111 if wall_wang != 0b1000  => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset,
                    y0: y_offset + 32.0,
                    x1: x_offset + 32.0,
                    y1: y_offset + 64.0,
                    tag: CollisionTag::Water
                }
            );
        },
        0b0101 if wall_wang != 0b1010  => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset,
                    x1: x_offset + 32.0,
                    y1: y_offset + 64.0,
                    tag: CollisionTag::Water
                }
            );
        },
        0b1010 if wall_wang != 0b0101  => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset,
                    x1: x_offset + 32.0,
                    y1: y_offset + 64.0,
                    tag: CollisionTag::Water
                }
            );
        },
        0b0011 if wall_wang != 0b1100 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset,
                    y0: y_offset + 32.0,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Water
                }
            );
        },
        0b1100 if wall_wang != 0b0011 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset,
                    y0: y_offset + 32.0,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Water
                }
            );
        },
        0b0110 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Water
                }
            );
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset,
                    y0: y_offset + 32.0,
                    x1: x_offset + 32.0,
                    y1: y_offset + 64.0,
                    tag: CollisionTag::Water
                }
            );
        },
        0b1001 => {
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset,
                    y0: y_offset + 32.0,
                    x1: x_offset + 32.0,
                    y1: y_offset,
                    tag: CollisionTag::Water
                }
            );
            collision_vec.push(
                CollisionRegion {
                    x0: x_offset + 32.0,
                    y0: y_offset + 64.0,
                    x1: x_offset + 64.0,
                    y1: y_offset + 32.0,
                    tag: CollisionTag::Water
                }
            );
        },
        _ => ()
    }
}

pub fn cast_circle(
    collisions: &CollisionVec,
    origin: glam::Vec2,
    p_dir: glam::Vec2,
    tag: CollisionTag
) -> Option<(f32, glam::Vec2)> {
    let mut t = None;
    for collision in collisions.iter() {
        if tag != CollisionTag::All && collision.tag != tag {
            continue;
        }
        match (
            t,
            SegmentCircleCastQuery::circle_cast_segment(
                origin,
                p_dir,
                RADIUS + SKIN,
                [
                    glam::vec2(collision.x0, collision.y0),
                    glam::vec2(collision.x1, collision.y1)
                ]
            )
        ) {
            (None, next) => t = next,
            (Some(
                (old_t, _)),
                Some((new_t, norm))
            ) if new_t < old_t => t = Some((new_t, norm)),
            _ => ()
        }
    }

    t.map(|(t, normal)| (t - SKIN * 2.0, normal))
}

pub fn move_position_towards(
    pos: Position,
    direction: glam::Vec2,
    collision_tag: CollisionTag,
    terrain_tiles_data: &WangTerrain
) -> (Position, bool) {
    let mut distance_to_go = direction.length();
    let mut current_dir = direction.normalize_or_zero();
    let mut current_pos = glam::vec2(pos.x, pos.y);

    let mut collision_vec = CollisionVec::new();
    let mut ii = (pos.x / 64.0) as usize;
    let mut jj = (pos.y / 64.0) as usize;

    let mut collided = false;

    populate_collisions_data(&mut collision_vec, ii, jj, &terrain_tiles_data);

    for _ in 0..MOVE_ITERATIONS {
        if distance_to_go < MINIMAL_DISTANCE {
            break;
        }

        let new_ii = (current_pos.x / 64.0) as usize;
        let new_jj = (current_pos.y / 64.0) as usize;
        if ii != new_ii || jj != new_jj {
            collision_vec.clear();
            ii = new_ii;
            jj = new_jj;

            populate_collisions_data(&mut collision_vec, ii, jj, &terrain_tiles_data);
        }

        distance_to_go = match cast_circle(
            &collision_vec,
            current_pos,
            current_dir,
            collision_tag
        ) {
            None =>  {
                current_pos += current_dir * distance_to_go;
                0.0
            },
            Some((distance, _)) if distance >= distance_to_go =>  {
                current_pos += current_dir * distance_to_go;
                0.0
            },
            Some((direct_distance, normal)) => {
                collided = true;
                let rest_distance = distance_to_go - direct_distance;
                current_pos += current_dir * direct_distance;

                current_dir = {
                    let dir_rest = current_dir * rest_distance;
                    let norm_proj = normal * dir_rest.dot(normal);
                    dir_rest - norm_proj
                };

                let distance = current_dir.length();
                current_dir = current_dir.normalize_or_zero();

                distance
            }
        }
    }

    (Position { x: current_pos.x, y: current_pos.y }, collided)
}

pub fn populate_collisions_data_from_position(
    collision_vec: &mut SmallVec<[CollisionRegion; 18]>,
    x: f32,
    y: f32,
    terrain_tiles_data: &WangTerrain
) {
    populate_collisions_data(
        collision_vec,
        (x / 64.0) as usize, (y / 64.0) as usize,
        terrain_tiles_data
    );
}

fn populate_collisions_data(
    collision_vec: &mut SmallVec<[CollisionRegion; 18]>,
    ii: usize,
    jj: usize,
    terrain_tiles_data: &WangTerrain
) {
    for j in (if jj > 0 { jj - 1 } else { jj })..=(if jj < MapData::WIDTH - 2 { jj + 1 } else { jj }) {
        for i in (if ii > 0 { ii - 1 } else { ii })..=(if ii < MapData::WIDTH - 2 { ii + 1 } else { ii }) {
            let idx = j * (MapData::WIDTH - 1) + i;
            populate_collisions(
                collision_vec,
                &terrain_tiles_data.tiles[idx],
                i as f32 * 64.0,
                j as f32 * 64.0
            );
        }
    }
}