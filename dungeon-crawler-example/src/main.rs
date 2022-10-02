use retro_blit::rendering::blittable::{BlitBuilder, BufferProviderMut};
use retro_blit::rendering::BlittableSurface;
use retro_blit::window::{ContextHandler, RetroBlitContext, WindowMode};
use crate::terrain_tiles_data::TerrainTiles;

const BAYER_LOOKUP: [u8; 16] = [
    00, 08, 02, 10,
    12, 04, 14, 06,
    03, 11, 01, 09,
    15, 07, 13, 05
];

const MAP_BYTES: &[u8] = include_bytes!("map.im256");
const GRAPHICS_BYTES: &[u8] = include_bytes!("dungeon_crawler.im256");

mod terrain_tiles_data;
mod map_data;
mod components;

pub struct App {
    time: f32,
    terrain_tiles: TerrainTiles,
    palette: Vec<[u8; 3]>,
    graphics: BlittableSurface,
    world: hecs::World
}

impl App {
    pub fn new() -> Self {
        let mut jfa = jfa_cpu::MatrixJfa::new();
        let terrain_tiles = TerrainTiles::load(&mut jfa);
        let mut world = hecs::World::new();
        let map_data = map_data::MapData::load(MAP_BYTES);
        let (palette, graphics) = retro_blit::format_loaders::im_256::Image
            ::load_from(GRAPHICS_BYTES)
            .unwrap();
        map_data.populate_world(&mut world);
        Self {
            time: 0.0,
            terrain_tiles,
            palette,
            graphics,
            world
        }
    }
}

impl ContextHandler for App {
    fn get_window_title(&self) -> &'static str {
        "dungeon crawler example"
    }

    fn get_window_mode(&self) -> WindowMode {
        WindowMode::Mode160x120
    }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        let mut offset = 0;
        let darkest_blue_idx = 2;
        let darkest_blue = self.palette[darkest_blue_idx];
        for i in 0..4 {
            for &pal_color in self.palette.iter() {
                let warm_overlay = [
                    if pal_color[0] < 225 { pal_color[0] + 30 } else { 255 },
                    if pal_color[1] < 238 { pal_color[1] + 17 } else { 255 },
                    pal_color[2]
                ];
                let darken = [
                    ((pal_color[0] as u16 * 70) / 100) as u8,
                    ((pal_color[1] as u16 * 70) / 100) as u8,
                    ((pal_color[2] as u16 * 90) / 100) as u8
                ];
                ctx.set_palette(
                    offset,
                    match i {
                        0 => warm_overlay,
                        1 => pal_color,
                        2 => darken,
                        _ => {
                            [
                                ((darkest_blue[0] as u16 + darken[0] as u16) / 2) as u8,
                                ((darkest_blue[1] as u16 + darken[1] as u16) / 2) as u8,
                                ((darkest_blue[2] as u16 + darken[2] as u16) / 2) as u8
                            ]
                        }
                    }
                );
                if offset < 255 { offset += 1; }
            }
        }
    }

    fn update(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        self.time += dt;

        let tint = ((1.0 + self.time.cos()) / 2.0).clamp(0.0, 1.0);
        let tint = tint * 4.05;

        let tint_offset = tint as u8;
        let tint_t = (tint.fract() * 16.0) as u8;

        ctx.clear(66);

        let sprite_sheet_with_color_key = self
            .graphics
            .with_color_key(0);

        BlitBuilder::create(ctx, &sprite_sheet_with_color_key)
            .with_source_subrect(120, 0, 160, 120)
            .with_dest_subrect(0, 0, 160, 120)
            .blit();

        let buffer = ctx.get_buffer_mut();
        for j in 0..160 {
            for i in 0..120 {
                let idx = j * 120 + i;
                if buffer[idx] == 66 {
                    continue;
                }
                let lookup_idx = (j % 4) * 4 + i % 4;
                let bayer = BAYER_LOOKUP[lookup_idx];

                if tint_offset >= 4 {
                    buffer[idx] = 66;
                } else {
                    let ix = buffer[idx] + tint_offset * 64;
                    let next_ix = if tint_offset == 3 {
                        66
                    } else {
                        ix + 64
                    };
                    buffer[idx] = if tint_t <= bayer { ix } else { next_ix};
                }
            }
        }
    }
}

fn main() {
    retro_blit::window::start(App::new());
}
