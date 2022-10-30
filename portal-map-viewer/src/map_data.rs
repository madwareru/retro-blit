use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{PathBuf};
use serde::Deserialize;
use ron::de::from_reader;

#[derive(Debug, Clone, Deserialize)]
pub struct MapData {
    pub pixels_per_meter: i32,
    pub points: Vec<MapPoint>,
    pub regions: Vec<MapRegion>
}

impl MapData {
    pub fn read_from_path(path: &PathBuf, bytes: &mut Vec<u8>) -> Self {
        bytes.clear();
        let mut file = File::open(path).unwrap();
        file.read_to_end(bytes).unwrap();
        from_reader(&bytes[..]).unwrap()
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct MapPoint(pub f32, pub f32);

#[derive(Debug, Clone, Deserialize)]
pub struct MapRegion {
    pub height: f32,
    pub walls: Vec<usize>,
    pub portals: HashMap<usize, usize>
}