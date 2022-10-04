use jfa_cpu::MatrixJfa;
use retro_blit::rendering::blittable::BufferProvider;

const WANG_MASK_BYTES: &[u8] = include_bytes!("wang_mask.im256");
const VORONOI_DOTS_BYTES: &[u8] = include_bytes!("voronoi_dots.im256");
const STALAGMITE_BYTES: &[u8] = include_bytes!("stalagmite.im256");
const STALACTITE_BYTES: &[u8] = include_bytes!("stalactite.im256");

const TILE_SIZE: usize = 64;

pub type Tile = Vec<f32>;

pub struct TerrainTiles {
    wang_tiles: Vec<Tile>,
    terrain_tiles: Vec<Tile>,
    stalagmite_tile: Tile,
    stalactite_tile: Tile
}

impl TerrainTiles {
    pub fn load(jfa: &mut jfa_cpu::MatrixJfa) -> Self {
        let mut wang_tiles = Vec::with_capacity(16);
        {
            let (_, wang_mask) = retro_blit::format_loaders::im_256::Image
            ::load_from(WANG_MASK_BYTES)
                .unwrap();
            let wang_buffer = wang_mask.get_buffer();

            wang_tiles.push(vec![0.0; 4096]);
            for i in 0..15 {
                let x = i % 4;
                let y = i / 4;
                let start_i = x * TILE_SIZE;
                let start_j = y * TILE_SIZE;
                let jfa = jfa.calc::<TILE_SIZE, TILE_SIZE>(
                    (0..4096)
                        .into_iter()
                        .filter_map(|idx| {
                            let (i, j) = (idx % TILE_SIZE, idx / TILE_SIZE);
                            let idx = start_i + i + (start_j + j) * 256;
                            let col = unsafe { wang_buffer.get_unchecked(idx) };
                            if *col == 0 {
                                None
                            } else {
                                Some((i, j))
                            }
                        })
                );
                wang_tiles.push(
                    jfa
                        .iter()
                        .enumerate()
                        .map(|(idx, &nearest_coord)| {
                            let x = idx % TILE_SIZE;
                            let y = idx / TILE_SIZE;
                            let dx = x as f32 - nearest_coord.0 as f32;
                            let dy = y as f32 - nearest_coord.1 as f32;
                            let distance = (dx * dx + dy * dy).sqrt();
                            let distance = super::utils::smooth_step(0.0, 12.0, distance);
                            1.0 - distance
                        })
                        .collect::<Tile>()
                );
            }
        }

        let mut terrain_tiles = Vec::with_capacity(4);
        {
            let (_, voronoi_dots) = retro_blit::format_loaders::im_256::Image
            ::load_from(VORONOI_DOTS_BYTES)
                .unwrap();
            let voronoi_buffer = voronoi_dots.get_buffer();

            for i in 0..4 {
                let x = i % 2;
                let y = i / 2;
                let start_i = x * TILE_SIZE;
                let start_j = y * TILE_SIZE;
                let jfa = jfa.calc::<TILE_SIZE, TILE_SIZE>(
                    (0..4096)
                        .into_iter()
                        .filter_map(|idx| {
                            let (i, j) = (idx % TILE_SIZE, idx / TILE_SIZE);
                            let idx = start_i + i + (start_j + j) * 256;
                            let col = unsafe { voronoi_buffer.get_unchecked(idx) };
                            if *col == 0 {
                                None
                            } else {
                                Some((i, j))
                            }
                        })
                );
                terrain_tiles.push(
                    jfa
                        .iter()
                        .enumerate()
                        .map(|(idx, &nearest_coord)| {
                            let x = idx % TILE_SIZE;
                            let y = idx / TILE_SIZE;
                            let dx = x as f32 - nearest_coord.0 as f32;
                            let dy = y as f32 - nearest_coord.1 as f32;
                            let noise = 1.0 - ((dx * dx + dy * dy) / 128.0).clamp(0.0, 1.0);
                            0.3 + 0.1 * noise.powf(0.5)
                        })
                        .collect::<Tile>()
                );
            }
        }

        let stalagmite_tile = {
            let (_, stalagmite) = retro_blit::format_loaders::im_256::Image
            ::load_from(STALAGMITE_BYTES)
                .unwrap();
            Self::load_single_tile(jfa, stalagmite.get_buffer())
        };

        let stalactite_tile = {
            let (_, stalactite) = retro_blit::format_loaders::im_256::Image
            ::load_from(STALACTITE_BYTES)
                .unwrap();
            Self::load_single_tile(jfa, stalactite.get_buffer())
        };

        Self {
            wang_tiles,
            terrain_tiles,
            stalagmite_tile,
            stalactite_tile
        }
    }

    fn load_single_tile(jfa: &mut MatrixJfa, buffer: &[u8]) -> Tile {
        let jfa_result = jfa.calc::<TILE_SIZE, TILE_SIZE>(
            (0..4096)
                .into_iter()
                .filter_map(|idx| {
                    let (i, j) = (idx % TILE_SIZE, idx / TILE_SIZE);
                    let col = unsafe { buffer.get_unchecked(idx) };
                    if *col == 0 {
                        None
                    } else {
                        Some((i, j))
                    }
                })
        );
        jfa_result
            .iter()
            .enumerate()
            .map(|(idx, &nearest_coord)| {
                let x = idx % TILE_SIZE;
                let y = idx / TILE_SIZE;
                let dx = x as f32 - nearest_coord.0 as f32;
                let dy = y as f32 - nearest_coord.1 as f32;
                1.0 - ((dx * dx + dy * dy).sqrt() * 20.0 / 255.0).clamp(0.0, 1.0)
            })
            .collect::<Tile>()
    }

    pub fn sample_tile(&self, tile_info: super::components::TileInfo, x_coord: f32, y_coord: f32) -> f32 {
        let x = (x_coord.fract() * TILE_SIZE as f32) as usize;
        let y = (y_coord.fract() * TILE_SIZE as f32) as usize;
        let idx = y * TILE_SIZE + x;
        match tile_info {
            super::components::TileInfo::Wang(wang_id) => self.wang_tiles[wang_id][idx],
            super::components::TileInfo::Terrain(terrain_id) => self.terrain_tiles[terrain_id][idx],
            super::components::TileInfo::Stalagmite => self.stalagmite_tile[idx],
            super::components::TileInfo::Stalactite => self.stalactite_tile[idx],
        }
    }
}