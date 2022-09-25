use crate::state::TerrainTile;

pub struct DualGridTileDataInfo {
    pub wang_tiles: [&'static [DualGridTileData]; 16]
}

const NORTH_EAST: usize = 0b0001;
const NORTH_WEST: usize = 0b0010;
const SOUTH_EAST: usize = 0b0100;
const SOUTH_WEST: usize = 0b1000;

const TERRAIN_DATA_INFO: [DualGridTileDataInfo; 5] = [
    DualGridTileDataInfo { // Rocks
        wang_tiles: [
            &[], //0b0000
            &[], //0b0001
            &[], //0b0010
            &[], //0b0100
            &[], //0b1000
            &[], //0b0011
            &[], //0b0110
            &[], //0b1100
            &[], //0b1001
            &[], //0b1010
            &[], //0b0101
            &[], //0b1110
            &[], //0b1101
            &[], //0b1011
            &[], //0b0111
            &[], //0b1111
        ]
    },
    DualGridTileDataInfo { // Dirt
        wang_tiles: [
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
        ]
    },
    DualGridTileDataInfo { // Grass
        wang_tiles: [
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
        ]
    },
    DualGridTileDataInfo { // Sand
        wang_tiles: [
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
        ]
    },
    DualGridTileDataInfo { // Water
        wang_tiles: [
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
        ]
    }
];

#[derive(Copy, Clone)]
pub struct DualGridTileData(pub u8, pub u8);

pub struct DualGridLayer {
    tiles: [[DualGridTileData; 128]; 64]
}

pub struct TerrainData {
    tiles: [[TerrainTile; 129]; 65],
    dual_grid_layers: [DualGridLayer; 5]
}