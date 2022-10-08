use smallvec::SmallVec;
use crate::{HeightMapEntry, WangTerrainEntry};

#[derive(Copy, Clone)]
pub enum CollisionTag {
    Water,
    Wall
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

pub type CollisionVec = SmallVec<[CollisionRegion; 16]>;

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