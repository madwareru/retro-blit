use hecs::World;
use retro_blit::rendering::blittable::{BlitBuilder, BufferProvider, BufferProviderMut, SizedSurface};
use retro_blit::rendering::BlittableSurface;
use retro_blit::rendering::bresenham::{BresenhamCircleDrawer, LineRasterizer};
use retro_blit::rendering::fonts::font_align::{HorizontalAlignment, VerticalAlignment};
use retro_blit::rendering::fonts::tri_spaced::{Font, TextDrawer};
use retro_blit::window::{ContextHandler, KeyCode, KeyMod, KeyMods, RetroBlitContext, ScrollDirection, ScrollKind, WindowMode};
use crate::components::{Angle, HP, MP, Player, Position, TerrainProp, TileInfo, WangHeightMapEntry, WangTerrain, WangTerrainEntry};
use crate::map_data::{HeightMapEntry, MapData};
use crate::terrain_tiles_data::TerrainTiles;

const BAYER_LOOKUP: [u8; 16] = [
    00, 08, 02, 10,
    12, 04, 14, 06,
    03, 11, 01, 09,
    15, 07, 13, 05
];

const MAP_BYTES: &[u8] = include_bytes!("map.im256");
const GRAPHICS_BYTES: &[u8] = include_bytes!("dungeon_crawler.im256");
const DARKEST_BLUE_IDX: usize = 0x02;

const PIXELS_PER_METER: f32 = 64.0;
const VIEW_RANGE: f32 = 14.0;

const NEAR: f32 = 0.05 * PIXELS_PER_METER;
const FAR: f32 = PIXELS_PER_METER * VIEW_RANGE;
const FOV_SLOPE: f32 = 0.7;

mod terrain_tiles_data;
mod map_data;
mod components;
mod utils;

pub enum AppOverlayState {
    Entry,
    NoOverlay,
    HelpContent,
    MinimapView
}

pub struct AppFlags {
    pub texture_terrain: bool,
    pub terrain_rendering_step: f32
}

pub struct App {
    scroll_timer: f32,
    flags: AppFlags,
    terrain_tiles: TerrainTiles,
    palette: Vec<[u8; 3]>,
    graphics: BlittableSurface,
    depth_buffer: Vec<f32>,
    font: Font,
    overlay_state: AppOverlayState,
    world: World,
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
        let mut depth_buffer = Vec::with_capacity(160 * 120);
        for j in 0..120 {
            let depth = 1.0 - ((48.0 - j as f32).abs() / 48.0).clamp(0.0, 1.0);
            for _ in 0..160 {
                depth_buffer.push(depth);
            }
        }
        map_data.populate_world(&mut world);
        let font = Font::default_font_small().unwrap();
        Self {
            scroll_timer: 0.0,
            terrain_tiles,
            palette,
            graphics,
            depth_buffer,
            flags: AppFlags {
                texture_terrain: true,
                terrain_rendering_step: 1.0 / 256.0
            },
            font,
            overlay_state: AppOverlayState::Entry,
            world,
        }
    }

    fn fade(&mut self, ctx: &mut RetroBlitContext) {
        let darkest_blue = DARKEST_BLUE_IDX as u8 + 64;
        let buffer = ctx.get_buffer_mut();
        for j in 0..96 {
            for i in 0..160 {
                let idx = j * 160 + i;

                if buffer[idx] == darkest_blue {
                    continue;
                }

                let tint = self.depth_buffer[idx];
                let tint = tint * 4.05;

                let tint_offset = tint as u8;
                let tint_t = (tint.fract() * 16.0) as u8;

                let lookup_idx = (j % 4) * 4 + i % 4;
                let bayer = BAYER_LOOKUP[lookup_idx];

                if tint_offset >= 4 {
                    buffer[idx] = darkest_blue;
                } else {
                    let ix = buffer[idx] + tint_offset * 64;
                    let next_ix = if tint_offset == 3 {
                        darkest_blue
                    } else {
                        ix + 64
                    };
                    buffer[idx] = if tint_t <= bayer { ix } else { next_ix };
                }
            }
        }
    }

    fn draw_hud(&mut self, ctx: &mut RetroBlitContext) {
        let sprite_sheet_with_color_key = self
            .graphics
            .with_color_key(0);

        BlitBuilder::create(ctx, &sprite_sheet_with_color_key)
            .with_source_subrect(120, 0, 160, 120)
            .with_dest_pos(0, 0)
            .blit();

        if let Some((_, (_, &HP(hp), &MP(mp)))) = self.world.query::<(&Player, &HP, &MP)>().iter().next() {
            let hp_offset = match () {
                _ if hp > 66 => 0,
                _ if hp > 33 => 24,
                _ if hp > 0 => 48,
                _ => 72
            };

            BlitBuilder::create(ctx, &sprite_sheet_with_color_key)
                .with_source_subrect(hp_offset, 24, 24, 24)
                .with_dest_pos(68, 94)
                .blit();

            if hp > 0 {
                let hp_height = (24 * hp) / 100;
                BlitBuilder::create(ctx, &sprite_sheet_with_color_key)
                    .with_source_subrect(61, 76, 6, hp_height as usize)
                    .with_dest_pos(59, 94 + 24 - hp_height as i16)
                    .blit();
            }

            if mp > 0 {
                let mp_height = (24 * mp) / 100;
                BlitBuilder::create(ctx, &sprite_sheet_with_color_key)
                    .with_source_subrect(69, 76, 6, mp_height as usize)
                    .with_dest_pos(95, 94 + 24 - mp_height as i16)
                    .blit();
            }
        }
    }

    fn render(&mut self, ctx: &mut RetroBlitContext) {
        ctx.clear(66);

        self.clear_depth_buffer();

        self.render_terrain(ctx);

        self.fade(ctx);

        self.draw_overlays(ctx);

        self.draw_hud(ctx);
    }

    fn clear_depth_buffer(&mut self) {
        for p in self.depth_buffer.iter_mut() {
            *p = 1.0;
        }
    }

    fn render_terrain(&mut self, ctx: &mut RetroBlitContext) {
        let mut trapezoid_coords;
        if let Some((_, data)) = self.world.query::<(&Player, &Position, &Angle)>().iter().next() {
            let (_, &Position { x, y }, &Angle(angle)) = data;

            let angle = angle.to_radians();

            trapezoid_coords = [
                rotate((FOV_SLOPE * NEAR, NEAR), angle),
                rotate((-FOV_SLOPE * NEAR, NEAR), angle),
                rotate((-FOV_SLOPE * FAR, FAR), angle),
                rotate((FOV_SLOPE * FAR, FAR), angle)
            ];

            for p in trapezoid_coords.iter_mut() {
                *p = (p.0 + x as f32, y as f32 - p.1);
            }
        } else {
            return;
        }

        if let Some((_, (wang_terrain, ))) = self.world.query::<(&mut WangTerrain, )>().iter().next() {
            for i in 0..160 {
                let t = i as f32 / 159.0;
                let uv_up = (
                    trapezoid_coords[2].0 * (1.0 - t) + trapezoid_coords[3].0 * t,
                    trapezoid_coords[2].1 * (1.0 - t) + trapezoid_coords[3].1 * t
                );
                let uv_bottom = (
                    trapezoid_coords[0].0 * (1.0 - t) + trapezoid_coords[1].0 * t,
                    trapezoid_coords[0].1 * (1.0 - t) + trapezoid_coords[1].1 * t
                );

                let mut max_h = 0;
                let mut bottom_pix = 96;

                let mut max_h_top = 96;
                let mut bottom_pix_top = 0;

                let mut t = 0.0;
                let delta_t = self.flags.terrain_rendering_step;
                while t <= 1.0 {
                    {
                        let t = t * t;
                        let point = (
                            uv_bottom.0 * (1.0 - t) + uv_up.0 * t,
                            uv_bottom.1 * (1.0 - t) + uv_up.1 * t,
                        );
                        let cell_coord = (point.0 / 64.0, point.1 / 64.0);
                        let cell_remainder = (cell_coord.0.fract(), cell_coord.1.fract());
                        let cell_coord = (cell_coord.0 as i32, cell_coord.1 as i32);

                        let dual_cell_coord = ((point.0 + 32.0) / 64.0, (point.1 + 32.0) / 64.0);
                        let dual_cell_remainder = (dual_cell_coord.0.fract(), dual_cell_coord.1.fract());
                        let dual_cell_coord = (dual_cell_coord.0 as i32, dual_cell_coord.1 as i32);

                        let in_range = (0..(MapData::WIDTH as i32 - 1)).contains(&cell_coord.0) &&
                            (0..(MapData::HEIGHT as i32 - 1)).contains(&cell_coord.1);

                        let dual_in_range = (0..(MapData::WIDTH as i32)).contains(&dual_cell_coord.0) &&
                            (0..(MapData::HEIGHT as i32)).contains(&dual_cell_coord.1);

                        let wang_terrain_entry = if !in_range
                        {
                            None
                        } else {
                            wang_terrain.seen_tiles.insert([cell_coord.0 as u16, cell_coord.1 as u16]);
                            let idx = (MapData::WIDTH - 1) * cell_coord.1 as usize + cell_coord.0 as usize;
                            Some(wang_terrain.tiles[idx])
                        }.unwrap_or(WangTerrainEntry {
                            terrain_id: 0,
                            top: WangHeightMapEntry {
                                north_east: HeightMapEntry::Wall,
                                north_west: HeightMapEntry::Wall,
                                south_east: HeightMapEntry::Wall,
                                south_west: HeightMapEntry::Wall,
                            },
                            bottom: WangHeightMapEntry {
                                north_east: HeightMapEntry::Wall,
                                north_west: HeightMapEntry::Wall,
                                south_east: HeightMapEntry::Wall,
                                south_west: HeightMapEntry::Wall,
                            },
                        });

                        let water_pix = self.graphics.get_buffer()[
                            (cell_remainder.0 * 24.0) as usize + 72 +
                                self.graphics.get_width() * (48 + (cell_remainder.1 * 24.0) as usize)
                            ];

                        let floor_pix = if self.flags.texture_terrain {
                            self.graphics.get_buffer()[
                                (cell_remainder.0 * 24.0) as usize + 24 +
                                    self.graphics.get_width() * (48 + (cell_remainder.1 * 24.0) as usize)
                                ]
                        } else {
                            17
                        };

                        let terrain_h = self.terrain_tiles.sample_tile(
                            TileInfo::Terrain(wang_terrain_entry.terrain_id),
                            cell_remainder.0,
                            cell_remainder.1,
                        );

                        let mut terrain_bottom = terrain_h;
                        {
                            let mut wang_id = 0;
                            if wang_terrain_entry.bottom.north_east == HeightMapEntry::Wall {
                                wang_id += 0b0001;
                            }
                            if wang_terrain_entry.bottom.north_west == HeightMapEntry::Wall {
                                wang_id += 0b0010;
                            }
                            if wang_terrain_entry.bottom.south_east == HeightMapEntry::Wall {
                                wang_id += 0b0100;
                            }
                            if wang_terrain_entry.bottom.south_west == HeightMapEntry::Wall {
                                wang_id += 0b1000;
                            }
                            terrain_bottom += (1.0 - terrain_bottom) *
                                self.terrain_tiles.sample_tile(
                                    TileInfo::Wang(wang_id),
                                    cell_remainder.0,
                                    cell_remainder.1,
                                );

                            wang_id = 0;
                            if wang_terrain_entry.bottom.north_east == HeightMapEntry::Water {
                                wang_id += 0b0001;
                            }
                            if wang_terrain_entry.bottom.north_west == HeightMapEntry::Water {
                                wang_id += 0b0010;
                            }
                            if wang_terrain_entry.bottom.south_east == HeightMapEntry::Water {
                                wang_id += 0b0100;
                            }
                            if wang_terrain_entry.bottom.south_west == HeightMapEntry::Water {
                                wang_id += 0b1000;
                            }
                            terrain_bottom += -terrain_bottom *
                                self.terrain_tiles.sample_tile(
                                    TileInfo::Wang(wang_id),
                                    cell_remainder.0,
                                    cell_remainder.1,
                                );
                        }
                        if dual_in_range {
                            if let Some(TerrainProp::Stalagmite) = wang_terrain.props.get(&[dual_cell_coord.0 as u16, dual_cell_coord.1 as u16]) {
                                terrain_bottom = utils::lerp(
                                    terrain_bottom,
                                    if terrain_bottom < 0.3 { 0.4} else {0.75},
                                    self.terrain_tiles.sample_tile(
                                    TileInfo::Stalagmite,
                                    dual_cell_remainder.0,
                                    dual_cell_remainder.1,
                                ));
                            }
                        }

                        let mut terrain_top = terrain_h - 0.2;
                        {
                            let mut wang_id = 0;
                            if wang_terrain_entry.top.north_east == HeightMapEntry::Wall {
                                wang_id += 0b0001;
                            }
                            if wang_terrain_entry.top.north_west == HeightMapEntry::Wall {
                                wang_id += 0b0010;
                            }
                            if wang_terrain_entry.top.south_east == HeightMapEntry::Wall {
                                wang_id += 0b0100;
                            }
                            if wang_terrain_entry.top.south_west == HeightMapEntry::Wall {
                                wang_id += 0b1000;
                            }
                            terrain_top += (1.0 - terrain_top) *
                                self.terrain_tiles.sample_tile(
                                    TileInfo::Wang(wang_id),
                                    cell_remainder.0,
                                    cell_remainder.1,
                                );

                            wang_id = 0;
                            if wang_terrain_entry.top.north_east == HeightMapEntry::Water {
                                wang_id += 0b0001;
                            }
                            if wang_terrain_entry.top.north_west == HeightMapEntry::Water {
                                wang_id += 0b0010;
                            }
                            if wang_terrain_entry.top.south_east == HeightMapEntry::Water {
                                wang_id += 0b0100;
                            }
                            if wang_terrain_entry.top.south_west == HeightMapEntry::Water {
                                wang_id += 0b1000;
                            }
                            terrain_top += -terrain_top *
                                self.terrain_tiles.sample_tile(
                                    TileInfo::Wang(wang_id),
                                    cell_remainder.0,
                                    cell_remainder.1,
                                );
                        }
                        if dual_in_range {
                            if let Some(TerrainProp::Stalactite) = wang_terrain.props.get(&[dual_cell_coord.0 as u16, dual_cell_coord.1 as u16]) {
                                terrain_top = utils::lerp(terrain_top, 0.55, self.terrain_tiles.sample_tile(
                                    TileInfo::Stalactite,
                                    dual_cell_remainder.0,
                                    dual_cell_remainder.1,
                                ));
                            }
                        }

                        { // render_bottom
                            if terrain_bottom > 0.25 {
                                let h = -64.0 + 128.0 * (terrain_bottom - 0.3);
                                let corr = NEAR * (1.0 - t) + FAR * t;
                                let h = 48.0 + h / (corr * FOV_SLOPE / PIXELS_PER_METER);

                                let h = h.clamp(0.0, 96.0) as usize;
                                if h > max_h {
                                    for _ in max_h..h {
                                        let idx = i + 160 * bottom_pix;
                                        if self.depth_buffer[idx] > t {
                                            self.depth_buffer[idx] = t;
                                            ctx.get_buffer_mut()[idx] = floor_pix;
                                        }
                                        if bottom_pix > 0 { bottom_pix -= 1; }
                                    }
                                    max_h = h;
                                }
                            } else {
                                let h = -64.0 + 128.0 * (-0.05);
                                let corr = NEAR * (1.0 - t) + FAR * t;
                                let h = 48.0 + h / (corr * FOV_SLOPE / PIXELS_PER_METER);

                                let h = h.clamp(0.0, 96.0) as usize;
                                if h > max_h {
                                    for _ in max_h..h {
                                        let idx = i + 160 * bottom_pix;
                                        if self.depth_buffer[idx] > t {
                                            self.depth_buffer[idx] = t;
                                            ctx.get_buffer_mut()[idx] = water_pix;
                                        }
                                        if bottom_pix > 0 { bottom_pix -= 1; }
                                    }
                                    max_h = h;
                                }
                            }
                        }

                        { // render top
                            let h = -64.0 + 128.0 * terrain_top;
                            let corr = NEAR * (1.0 - t) + FAR * t;
                            let h = 48.0 + h / (corr * FOV_SLOPE / PIXELS_PER_METER);

                            let h = 96 - h.clamp(0.0, 96.0) as usize;
                            if h < max_h_top {
                                for _ in h..max_h_top {
                                    let idx = i + 160 * bottom_pix_top;
                                    if self.depth_buffer[idx] > t {
                                        self.depth_buffer[idx] = t;
                                        ctx.get_buffer_mut()[idx] = floor_pix;
                                    }
                                    bottom_pix_top += 1;
                                }
                                max_h_top = h;
                            }
                        }
                    }
                    t += delta_t;
                }
            }
        }
    }
    fn draw_overlays(&self, ctx: &mut RetroBlitContext) {
        match self.overlay_state {
            AppOverlayState::Entry => {
                self.font.draw_text_in_box(
                    ctx,
                    0, 0,
                    160, 96,
                    HorizontalAlignment::Left,
                    VerticalAlignment::Top,
                    "F1 for help!",
                    Some(12)
                );
            },
            AppOverlayState::NoOverlay => {},
            AppOverlayState::HelpContent => {
                self.font.draw_text_in_box(
                    ctx,
                    0, -2,
                    160, 96,
                    HorizontalAlignment::Center,
                    VerticalAlignment::Center,

                    r##"Arrows: Movement
Shift: Strafe
Return: Cast a magic
0: Toggle terrain texturing
-/=: Tweak terrain quality
F1: Toggle help
Tab: Toggle map
Esc: Quit game
                    "##,
                    Some(12)
                );
            }
            AppOverlayState::MinimapView => {
                self.render_minimap(ctx);
            }
        }

    }

    fn update_input(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        let (strafe_speed, turn_speed) = match (ctx.is_key_pressed(KeyCode::Left), ctx.is_key_pressed(KeyCode::Right)) {
            (true, false) => {
                if ctx.is_key_mod_pressed(KeyMod::Shift) {
                    (180.0, 0.0)
                } else {
                    (0.0, -60.0)
                }
            }
            (false, true) => {
                if ctx.is_key_mod_pressed(KeyMod::Shift) {
                    (-180.0, 0.0)
                } else {
                    (0.0, 60.0)
                }
            }
            _ => (0.0, 0.0)
        };

        let movement_speed = match (ctx.is_key_pressed(KeyCode::Down), ctx.is_key_pressed(KeyCode::Up)) {
            (true, false) => {
                -180.0
            }
            (false, true) => {
                180.0
            }
            _ => 0.0
        };

        for (_, player_data) in self.world.query_mut::<(&mut Player, &mut Position, &mut Angle)>() {
            let (_, pos, angle) = player_data;
            angle.0 += turn_speed * dt;
            let angle = angle.0.to_radians();
            let (s, c) = (angle.sin() * dt, angle.cos() * dt);
            let speed_x = movement_speed * s - strafe_speed * c;
            let speed_y = - movement_speed * c - strafe_speed * s;
            pos.x = pos.x + speed_x;
            pos.y = pos.y + speed_y;
        }
    }

    fn update_palette_scrolling(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        self.scroll_timer += dt;

        while self.scroll_timer > 0.2 {
            self.scroll_timer -= 0.2;
            ctx.scroll_palette(ScrollKind::Range { start_idx: 26, len: 6 }, ScrollDirection::Forward);
            ctx.scroll_palette(ScrollKind::Range { start_idx: 26 + 64, len: 6 }, ScrollDirection::Forward);
            ctx.scroll_palette(ScrollKind::Range { start_idx: 26 + 128, len: 6 }, ScrollDirection::Forward);
            ctx.scroll_palette(ScrollKind::Range { start_idx: 26 + 192, len: 6 }, ScrollDirection::Forward);
        }
    }
    fn render_minimap(&self, ctx: &mut RetroBlitContext) {
        let sprite_sheet_with_color_key = self
            .graphics
            .with_color_key(0);

        let start_x;
        let start_y;
        let angle;

        if let Some((_, data)) = self.world.query::<(&Player, &Position, &Angle)>().iter().next() {
            let (_, &Position { x, y }, &Angle(a)) = data;

            angle = a.to_radians();

            let (remapped_x, remapped_y) = (x / 16.0, y / 16.0);
            start_x = -(remapped_x as i32);
            start_y = -(remapped_y as i32);
        } else {
            return;
        }

        if let Some((_, (wang_terrain, ))) = self.world.query::<(&WangTerrain, )>().iter().next() {
            for j in 0..MapData::HEIGHT-1 {
                for i in 0..MapData::WIDTH-1 {
                    let x = start_x + i as i32 * 4;
                    let y = start_y + j as i32 * 4;
                    let idx = j * (MapData::WIDTH-1) + i;
                    if !wang_terrain.seen_tiles.contains(&[i as u16, j as u16]) {
                        continue;
                    }

                    let tile = wang_terrain.tiles[idx].bottom;

                    let mut water_wang = 0;
                    let mut wall_wang = 0;

                    match tile.north_east {
                        HeightMapEntry::Water => {
                            water_wang += 0b0001;
                        }
                        HeightMapEntry::Floor => {}
                        HeightMapEntry::Wall => {
                            wall_wang += 0b0001;}
                    }
                    match tile.north_west {
                        HeightMapEntry::Water => {
                            water_wang += 0b0010;
                        }
                        HeightMapEntry::Floor => {}
                        HeightMapEntry::Wall => {
                            wall_wang += 0b0010;}
                    }
                    match tile.south_east {
                        HeightMapEntry::Water => {
                            water_wang += 0b0100;
                        }
                        HeightMapEntry::Floor => {}
                        HeightMapEntry::Wall => {
                            wall_wang += 0b0100;}
                    }
                    match tile.south_west {
                        HeightMapEntry::Water => {
                            water_wang += 0b1000;
                        }
                        HeightMapEntry::Floor => {}
                        HeightMapEntry::Wall => {
                            wall_wang += 0b1000;}
                    }

                    if water_wang != 0 {
                        let i = water_wang % 4;
                        let j = water_wang / 4;
                        BlitBuilder::create(ctx, &sprite_sheet_with_color_key)
                            .with_source_subrect(80 + i * 4, 80 + j * 4, 4, 4)
                            .with_dest_pos(80 + x as i16, 48 + y as i16)
                            .blit();
                    }

                    if wall_wang != 0 {
                        let i = wall_wang % 4;
                        let j = wall_wang / 4;
                        BlitBuilder::create(ctx, &sprite_sheet_with_color_key)
                            .with_source_subrect(96 + i * 4, 80 + j * 4, 4, 4)
                            .with_dest_pos(80 + x as i16, 48 + y as i16)
                            .blit();
                    }

                    BresenhamCircleDrawer::create(ctx)
                        .with_position((80, 48))
                        .with_radius(4)
                        .draw(12);

                    let view_vec = (-6.0 * angle.sin(), 6.0 * angle.cos());

                    LineRasterizer::create(ctx)
                        .from((80, 48))
                        .to(((80.0 - view_vec.0) as _, (48.0 - view_vec.1) as _))
                        .rasterize(12);
                }
            }
        }
    }
}

impl ContextHandler for App {
    fn get_window_title(&self) -> &'static str { "dungeon crawler example" }

    fn get_window_mode(&self) -> WindowMode { WindowMode::Mode160x120 }

    fn on_key_up(&mut self, ctx: &mut RetroBlitContext, key_code: KeyCode, _key_mods: KeyMods) {
        match key_code {
            KeyCode::Key0 => {
                self.flags.texture_terrain = !self.flags.texture_terrain;
            },
            KeyCode::Minus => {
                self.flags.terrain_rendering_step = (self.flags.terrain_rendering_step * 2.0)
                    .clamp(1.0 / 4096.0, 1.0 / 8.0);
            },
            KeyCode::Equal => {
                self.flags.terrain_rendering_step = (self.flags.terrain_rendering_step / 2.0)
                    .clamp(1.0 / 4096.0, 1.0 / 8.0);
            },
            KeyCode::F1 => {
                self.overlay_state = match self.overlay_state {
                    AppOverlayState::HelpContent => AppOverlayState::NoOverlay,
                    _ => AppOverlayState::HelpContent
                };
            },
            KeyCode::Tab => {
                self.overlay_state = match self.overlay_state {
                    AppOverlayState::MinimapView => AppOverlayState::NoOverlay,
                    _ => AppOverlayState::MinimapView
                };
            },
            KeyCode::Escape => {
                ctx.quit();
            }
            _ => ()
        }
    }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        let mut offset = 0;
        let darkest_blue = self.palette[DARKEST_BLUE_IDX];
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
                    },
                );
                if offset < 255 { offset += 1; }
            }
        }
    }

    fn update(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        self.update_palette_scrolling(ctx, dt);
        self.update_input(ctx, dt);
        self.render(ctx);
    }
}

fn main() {
    retro_blit::window::start(App::new());
}

fn rotate(p: (f32, f32), angle: f32) -> (f32, f32) {
    let sin_cos = (angle.sin(), angle.cos());
    (
        p.0 * sin_cos.1 + p.1 * sin_cos.0,
        -p.0 * sin_cos.0 + p.1 * sin_cos.1
    )
}