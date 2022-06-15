use retro_blit::rendering::blittable::{BlitBuilder, Flip};
use retro_blit::rendering::BlittableSurface;
use retro_blit::window::{RetroBlitContext, ContextHandler, WindowMode};

const PICTURE_BYTES: &[u8] = include_bytes!("spritesheet.im256");

#[derive(Copy, Clone)]
pub enum Tile {
    Water,
    Grass,
    Dirt,
    Sand,
    Wheat
}
impl Tile {
    fn to_tile_coords(self) -> (usize, usize) {
        match self {
            Tile::Water => (24, 0),
            Tile::Grass => (48, 0),
            Tile::Dirt => (0, 32),
            Tile::Sand => (24, 32),
            Tile::Wheat => (48, 32)
        }
    }
}

struct MyGame {
    offset: i32,
    palette: Vec<[u8; 3]>,
    sprite_sheet: BlittableSurface,
    level: [[Tile; 5]; 5]
}
impl MyGame {
    pub fn new() -> Self {
        let (palette, pic) = retro_blit::format_loaders::im_256::Image::load_from(PICTURE_BYTES).unwrap();
        let mut surface = BlittableSurface::from(&pic);
        surface.set_color_key(Some(0));
        Self {
            offset: 0,
            palette,
            sprite_sheet: surface,
            level: [
                [Tile::Water, Tile::Water, Tile::Water, Tile::Water, Tile::Water],
                [Tile::Water, Tile::Water, Tile::Sand, Tile::Sand, Tile::Grass],
                [Tile::Grass, Tile::Grass, Tile::Grass, Tile::Grass, Tile::Wheat],
                [Tile::Grass, Tile::Wheat, Tile::Wheat, Tile::Wheat, Tile::Dirt],
                [Tile::Water, Tile::Grass, Tile::Sand, Tile::Dirt, Tile::Dirt]
            ]
        }
    }
}

impl ContextHandler for MyGame {
    fn get_window_mode(&self) -> WindowMode {
        WindowMode::Mode13
    }

    fn init(&mut self, data: &mut RetroBlitContext) {
        for i in 0..self.palette.len() {
            data.set_palette(i as _, self.palette[i]);
        }
    }

    fn update(&mut self, data: &mut RetroBlitContext) {
        data.clear(1);

        for j in 0..5 {
            let y_coord = 44 + (j as i32) * 20;
            for i in 0..5 {
                let x_coord = 100 + (i as i32) * 24;
                let (tx, ty) = self.level[j][i].to_tile_coords();
                BlitBuilder::create(data, &self.sprite_sheet)
                    .with_source_subrect(tx, ty, 24, 32)
                    .with_dest_pos(x_coord, y_coord)
                    .blit();
            }
        }

        BlitBuilder::create(data, &self.sprite_sheet)
            .with_source_subrect(0, 0, 24, 20)
            .with_dest_pos(-24 + (self.offset % 344), 100-30)
            .blit();

        BlitBuilder::create(data, &self.sprite_sheet)
            .with_source_subrect(0, 0, 24, 20)
            .with_dest_pos(320 - (self.offset % 344), 100+10)
            .with_flip(Flip::Horizontally)
            .blit();

        BlitBuilder::create(data, &self.sprite_sheet)
            .with_source_subrect(0, 0, 24, 20)
            .with_dest_pos(160-36, -20 + (self.offset % 220))
            .blit();

        BlitBuilder::create(data, &self.sprite_sheet)
            .with_source_subrect(0, 0, 24, 20)
            .with_dest_pos(160+12, 200 - (self.offset % 220))
            .with_flip(Flip::Vertically)
            .blit();

        self.offset += 1;
    }
}


fn main() {
    retro_blit::window::start(MyGame::new());
}