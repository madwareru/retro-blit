use glam::vec2;
use hecs::{CommandBuffer, Entity, World};
use smallvec::SmallVec;
use retro_blit::rendering::blittable::{BlitBuilder, BufferProvider, BufferProviderMut, SizedSurface};
use retro_blit::rendering::BlittableSurface;
use retro_blit::rendering::bresenham::{BresenhamCircleDrawer, LineRasterizer};
use retro_blit::rendering::fonts::font_align::{HorizontalAlignment, VerticalAlignment};
use retro_blit::rendering::fonts::tri_spaced::{Font, TextDrawer};
use retro_blit::window::{ContextHandler, KeyCode, KeyMod, KeyMods, RetroBlitContext, ScrollDirection, ScrollKind, WindowMode};
use crate::ai::Blackboard;
use crate::collision::{CollisionTag, CollisionVec};
use crate::components::*;
use crate::map_data::{HeightMapEntry, MapData};
use crate::terrain_tiles_data::TerrainTiles;

const BAYER_LOOKUP: [f32; 16] = [
    00.0, 08.0, 02.0, 10.0,
    12.0, 04.0, 14.0, 06.0,
    03.0, 11.0, 01.0, 09.0,
    15.0, 07.0, 13.0, 05.0
];

const NOISE_PNG_BYTES: &[u8] = include_bytes!("noise.png");
const MAP_BYTES: &[u8] = include_bytes!("map.im256");
const GRAPHICS_BYTES: &[u8] = include_bytes!("dungeon_crawler.im256");
const DARKEST_BLUE_IDX: usize = 0x02;
const TINT_FADE_OUT_SPEED: f32 = 3.0;

const PIXELS_PER_METER: f32 = 64.0;
const VIEW_RANGE: f32 = 14.0;

const NEAR: f32 = 0.005 * PIXELS_PER_METER;
const FAR: f32 = PIXELS_PER_METER * VIEW_RANGE;

pub(crate) mod systems_base;
pub(crate) mod works;
mod terrain_tiles_data;
mod map_data;
mod components;
mod collision;
mod utils;
mod ai;

pub enum AppOverlayState {
    Entry,
    NoOverlay,
    HelpContent,
    MinimapView,
}

pub enum DimLevel {
    FullWithBlueNoise,
    FullWithDither,
    DimOnly,
}

pub struct AppFlags {
    pub texture_terrain: bool,
    pub terrain_rendering_step: f32,
    pub fov_slope: f32,
    pub dim_level: DimLevel,
}

pub struct HandWaveSate {
    amount: f32,
    t: f32
}

pub enum PaletteState {
    ScrollingWater,
    HpPickupTint { t: f32 },
    MpPickupTint { t: f32 },
    DamageTint { t: f32 },
}

pub struct App {
    scroll_timer: f32,
    flags: AppFlags,
    terrain_tiles: TerrainTiles,
    last_palette: Vec<[u8; 3]>,
    graphics: BlittableSurface,
    depth_buffer: Vec<f32>,
    font: Font,
    overlay_state: AppOverlayState,
    noise_dither_lookup: Vec<f32>,
    blackboard: Blackboard,
    world: World,
    command_buffer: CommandBuffer,
    palette_state: PaletteState,
    spatial_map: flat_spatial::DenseGrid<Entity>,
    hand_wave_state: HandWaveSate
}

fn cast_melee(
    world: &World,
    command_buffer: &mut CommandBuffer,
    spatial_map: &mut flat_spatial::DenseGrid<Entity>,
    cast: MeleeCast,
    caster: Entity,
    position: Position,
    angle: Angle
) {
    let angle = angle.0.to_radians();
    let forward_vec = vec2(angle.sin(), -angle.cos());
    let angle_cos = (cast.cast_angle / 2.0).cos();

    for (_, &other_entity) in spatial_map
        .query_around([position.x, position.y], 128.0)
        .filter_map(|it | spatial_map.get(it.0)) {

        if other_entity == caster {
            continue;
        }

        let mut query = world.query_one::<(&Position, &mut HP)>(other_entity).unwrap();
        let (other_pos, hp) = query.get().unwrap();

        let pos = vec2(position.x, position.y);
        let other_pos = vec2(other_pos.x, other_pos.y);
        let delta = other_pos - pos;
        let distance = delta.length();
        if distance <= cast.cast_distance {
            let delta = delta / distance;
            let proj = delta.dot(forward_vec);
            if (angle_cos..=1.0).contains(&proj) {
                do_damage(other_entity, world, hp, cast.cast_damage, command_buffer);
            }
        }
    }
}

fn do_damage(entity: Entity, world: &World, hp: &mut HP, damage: i32, cb: &mut CommandBuffer) {
    hp.0 = (hp.0 - damage).max(0);
    if hp.0 > 0 {
        if world.get::<DamageTint>(entity).is_err() {
            cb.insert(entity, (DamageTint(0.05),));
        }
    }
}

fn cast_freeze_spell(
    _world: &World,
    command_buffer: &mut CommandBuffer,
    _spatial_map: &mut flat_spatial::DenseGrid<Entity>,
    cast: FreezeSpellCast,
    caster: Entity,
    position: Position,
    angle: Angle
) {
    let angle = angle.0.to_radians();
    let forward_vec = vec2(angle.sin(), -angle.cos()) * 24.0;
    let projectile: Projectile<FreezeSpellCast, FreezeSpellProjectile> = Projectile::make(caster);
    command_buffer.spawn(
        (
            projectile,
            Position{
                x: position.x + forward_vec.x,
                y: position.y + forward_vec.y
            },
            DesiredVelocity {
                x: forward_vec.x * 4.0,
                y: forward_vec.y * 4.0
            },
            cast
        )
    );
}

impl App {
    pub(crate) fn update_pickups(&mut self, ctx: &mut RetroBlitContext) {
        let pos;
        let health;
        let mana_points;
        if let Some((_, (_, position, hp, mp))) = self.world.query::<(&Player, &Position, &HP, &MP)>().iter().next() {
            pos = *position;
            health = hp.0;
            mana_points = mp.0;
        } else {
            return;
        }
        let pos = vec2(pos.x, pos.y);

        let mut new_health = health;
        let mut new_mp = mana_points;

        let mut entities_to_delete: SmallVec<[Entity; 8]> = SmallVec::new();
        for (e, (potion, position)) in self.world.query::<(&Potion, &Position)>().iter() {
            let potion_position = vec2(position.x, position.y);
            if potion_position.distance_squared(pos) <= 256.0 {
                match potion {
                    Potion::Health if new_health < 100 => {
                        new_health = (new_health + 20).min(100);
                        entities_to_delete.push(e);
                    }
                    Potion::Mana if new_mp < 100 => {
                        new_mp = (new_mp + 20).min(100);
                        entities_to_delete.push(e);
                    }
                    _ => ()
                }
            }
        }

        if new_health > health {
            self.set_palette_state(ctx, PaletteState::HpPickupTint { t: 1.0 });
            if let Some((_, (_, hp))) = self.world.query::<(&Player, &mut HP)>().iter().next() {
                *hp = HP(new_health);
            }
        } else if new_mp > mana_points {
            self.set_palette_state(ctx, PaletteState::MpPickupTint { t: 1.0 });
            if let Some((_, (_, mp))) = self.world.query::<(&Player, &mut MP)>().iter().next() {
                *mp = MP(new_mp);
            }
        }

        for e in entities_to_delete.drain(..) {
            self.world.despawn(e).unwrap();
        }
    }
}

impl App {
    pub fn new() -> Self {
        let mut jfa = jfa_cpu::MatrixJfa::new();
        let terrain_tiles = TerrainTiles::load(&mut jfa);
        let mut world = World::new();
        let map_data = MapData::load(MAP_BYTES);
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

        let mut spatial_map = flat_spatial::DenseGrid::new(64);

        map_data.populate_world(&mut world, &mut spatial_map);
        let font = Font::default_font_small().unwrap();

        let noise_img = image::load_from_memory(NOISE_PNG_BYTES)
            .unwrap()
            .to_luma8();

        let noise_dither_lookup = noise_img
            .as_raw()
            .iter()
            .map(|&it| it as f32 / 255.0)
            .collect();

        Self {
            scroll_timer: 0.0,
            terrain_tiles,
            last_palette: palette,
            graphics,
            depth_buffer,
            flags: AppFlags {
                texture_terrain: true,
                terrain_rendering_step: 1.0 / 512.0,
                fov_slope: 1.0,
                dim_level: DimLevel::DimOnly,
            },
            font,
            overlay_state: AppOverlayState::Entry,
            noise_dither_lookup,
            blackboard: Blackboard { player_position: Position { x: 0.0, y: 0.0 } },
            world,
            command_buffer: CommandBuffer::new(),
            palette_state: PaletteState::ScrollingWater,
            spatial_map,
            hand_wave_state: HandWaveSate {
                amount: 0.0,
                t: 0.0
            }
        }
    }

    fn fade(&mut self, ctx: &mut RetroBlitContext) {
        let darkest_blue = DARKEST_BLUE_IDX as u8 + 72;
        let buffer = ctx.get_buffer_mut();
        for j in 0..96 {
            for i in 0..160 {
                let idx = j * 160 + i;

                if buffer[idx] == darkest_blue {
                    continue;
                }

                let tint = self.depth_buffer[idx];
                let tint = tint * 7.9;

                let tint_offset = tint as u8;
                let tint_t = tint.fract();

                let threshold = match self.flags.dim_level {
                    DimLevel::FullWithBlueNoise => {
                        let lookup_idx = (j % 128) * 128 + i % 128;
                        self.noise_dither_lookup[lookup_idx]
                    }
                    _ => {
                        let lookup_idx = (j % 4) * 4 + i % 4;
                        BAYER_LOOKUP[lookup_idx] / 16.0
                    }
                };

                if tint_offset >= 7 {
                    buffer[idx] = darkest_blue;
                } else {
                    let ix = buffer[idx] + tint_offset * 36;
                    let next_ix = if tint_offset == 6 {
                        darkest_blue
                    } else {
                        ix + 36
                    };
                    buffer[idx] = match self.flags.dim_level {
                        DimLevel::FullWithBlueNoise | DimLevel::FullWithDither => {
                            if tint_t <= threshold { ix } else { next_ix }
                        }
                        DimLevel::DimOnly => {
                            ix
                        }
                    };
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
                .with_source_subrect(hp_offset, 96, 24, 24)
                .with_dest_pos(68, 94)
                .blit();

            if hp > 0 {
                let hp_height = (24 * hp) / 100;
                BlitBuilder::create(ctx, &sprite_sheet_with_color_key)
                    .with_source_subrect(96, 96, 6, hp_height as usize)
                    .with_dest_pos(59, 94 + 24 - hp_height as i16)
                    .blit();
            }

            if mp > 0 {
                let mp_height = (24 * mp) / 100;
                BlitBuilder::create(ctx, &sprite_sheet_with_color_key)
                    .with_source_subrect(102, 96, 6, mp_height as usize)
                    .with_dest_pos(95, 94 + 24 - mp_height as i16)
                    .blit();
            }
        }
    }

    fn render(&mut self, ctx: &mut RetroBlitContext) {
        ctx.clear(72);
        self.clear_depth_buffer();
        self.render_terrain(ctx);
        self.render_objects(ctx);
        self.render_particles(ctx);
        self.fade(ctx);
        self.render_hands(ctx);
        self.draw_overlays(ctx);
        self.draw_hud(ctx);
    }

    fn clear_depth_buffer(&mut self) {
        for p in self.depth_buffer.iter_mut() {
            *p = 1.0;
        }
    }

    fn render_terrain(&mut self, ctx: &mut RetroBlitContext) {
        let Some((_, (_, &Position { x, y }, &Angle(angle))))
            = self.world
                .query::<(&Player, &Position, &Angle)>()
                .iter()
                .next() else { return; };

        let trapezoid_coords = gen_trapezoid_coords(x, y, angle.to_radians(), self.flags.fov_slope);

        let mut depth_buffer = std::mem::take(&mut self.depth_buffer);
        self.with_wang_data_mut(|wang_terrain| {
            for i in 0..160 {
                let t = i as f32 / 159.0;

                let uv_up = (
                    utils::lerp(trapezoid_coords[2].0, trapezoid_coords[3].0, t),
                    utils::lerp(trapezoid_coords[2].1, trapezoid_coords[3].1, t)
                );
                let uv_bottom = (
                    utils::lerp(trapezoid_coords[0].0, trapezoid_coords[1].0, t),
                    utils::lerp(trapezoid_coords[0].1, trapezoid_coords[1].1, t)
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
                        let remainder = (cell_coord.0.fract(), cell_coord.1.fract());
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

                        let stride = self.graphics.get_width() * (72 + (remainder.1 * 24.0) as usize);

                        let water_pix = *unsafe{
                            self
                                .graphics
                                .get_buffer()
                                .get_unchecked(stride + (remainder.0 * 24.0) as usize)
                        };
                        let floor_pix = *unsafe{
                            self
                                .graphics
                                .get_buffer()
                                .get_unchecked(stride + (remainder.0 * 24.0) as usize + 24)
                        };

                        let (terrain_bottom, terrain_top) = self.fetch_terrain(
                            wang_terrain,
                            remainder,
                            dual_cell_remainder,
                            dual_cell_coord,
                            dual_in_range,
                            wang_terrain_entry
                        );

                        { // render_bottom
                            if terrain_bottom > 0.25 {
                                let h = -64.0 + 128.0 * (terrain_bottom - 0.3);
                                let h = self.project_height(h, t);

                                let h = h.clamp(0.0, 96.0) as usize;
                                if h > max_h {
                                    for _ in max_h..h {
                                        let idx = i + 160 * bottom_pix;
                                        if depth_buffer[idx] > t {
                                            depth_buffer[idx] = t;
                                            ctx.get_buffer_mut()[idx] = floor_pix;
                                        }
                                        if bottom_pix > 0 { bottom_pix -= 1; }
                                    }
                                    max_h = h;
                                }
                            } else {
                                let h = -64.0 + 128.0 * (-0.05);
                                let h = self.project_height(h, t);

                                let h = h.clamp(0.0, 96.0) as usize;
                                if h > max_h {
                                    for _ in max_h..h {
                                        let idx = i + 160 * bottom_pix;
                                        if depth_buffer[idx] > t {
                                            depth_buffer[idx] = t;
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
                            let h = self.project_height(h, t);

                            let h = 96 - h.clamp(0.0, 96.0) as usize;
                            if h < max_h_top {
                                for _ in h..max_h_top {
                                    let idx = i + 160 * bottom_pix_top;
                                    if depth_buffer[idx] > t {
                                        depth_buffer[idx] = t;
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
        });
        self.depth_buffer = depth_buffer;
    }

    #[inline(always)]
    fn project_height(&self, h: f32, depth: f32) -> f32 {
        48.0 + h * Self::scale_y(depth, self.flags.fov_slope)
    }

    fn fetch_terrain(
        &self,
        wang_terrain: &WangTerrain,
        remainder: (f32, f32),
        dual_cell_remainder: (f32, f32),
        dual_cell_coord: (i32, i32),
        dual_in_range: bool,
        wang_terrain_entry: WangTerrainEntry
    ) -> (f32, f32) {
        let terrain_detail_height = self.terrain_tiles.sample_tile(
            TileInfo::Terrain(wang_terrain_entry.terrain_id),
            remainder.0,
            remainder.1,
        );

        let mut terrain_bottom = terrain_detail_height;
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
                    remainder.0,
                    remainder.1,
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
                    remainder.0,
                    remainder.1,
                );
        }
        if dual_in_range {
            if let Some(TerrainProp::Stalagmite) = wang_terrain.props.get(&[dual_cell_coord.0 as u16, dual_cell_coord.1 as u16]) {
                terrain_bottom = utils::lerp(
                    terrain_bottom,
                    if terrain_bottom < 0.3 { 0.4 } else { 0.75 },
                    self.terrain_tiles.sample_tile(
                        TileInfo::Stalagmite,
                        dual_cell_remainder.0,
                        dual_cell_remainder.1,
                    ));
            }
        }

        let mut terrain_top = terrain_detail_height - 0.2;
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
                    remainder.0,
                    remainder.1,
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
                    remainder.0,
                    remainder.1,
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

        (terrain_bottom, terrain_top)
    }

    #[inline(always)]
    fn scale_y(t: f32, fov_slope: f32) -> f32 {
        let corr = utils::lerp(NEAR, FAR, t);
        1.0 / (corr * fov_slope / PIXELS_PER_METER)
    }

    fn render_hands(&mut self, ctx: &mut RetroBlitContext) {
        let (spell, sword);

        if let Some((_, (_, sp, sw))) = self.world.query::<(
            &Player,
            &FreezeSpellCastState,
            &MeleeCastState)
        >().iter().next() {
            spell = sp.get_anim_info();
            sword = sw.get_anim_info();
        } else {
            return;
        }

        let sprite_sheet_with_color_key = self
            .graphics
            .with_color_key(0);

        let (movement_amount, hand_wave_t) = (self.hand_wave_state.amount.min(1.0), self.hand_wave_state.t);

        let spell_arm_x_anim = movement_amount * hand_wave_t.sin() * 4.0;
        let spell_arm_y_anim = movement_amount * (hand_wave_t * 2.0).sin() * 4.0;

        let sword_arm_x_anim = movement_amount * -(hand_wave_t + 0.35).cos() * 4.0;
        let sword_arm_y_anim = movement_amount * ((hand_wave_t + 0.35) * 2.0).cos() * 4.0;

        let (spell_arm_x, spell_arm_y) = match spell {
            CastState::PreCast { t } => {
                (
                    4 + (24.0 * t) as i16 + (spell_arm_x_anim * (1.0 - t)) as i16,
                    96 - 30 - (14.0 * t) as i16 + (spell_arm_y_anim * (1.0 - t)) as i16
                )
            },
            CastState::Cast { t } => {
                (
                    4 + (24.0 * (1.0 - t)) as i16 + (spell_arm_x_anim * t) as i16,
                    96 - 30 - (14.0 * (1.0 - t)) as i16 + (spell_arm_y_anim * t) as i16
                )
            },
            _ => {
                (4 + spell_arm_x_anim as i16, 96 - 30 + spell_arm_y_anim as i16)
            }
        };

        let (sword_arm_x, sword_arm_y) = match sword {
            CastState::PreCast { t } => {
                (
                    160-52 - (24.0 * t) as i16 + (sword_arm_x_anim * (1.0 - t)) as i16,
                    96 - 30 - (14.0 * t) as i16 + (sword_arm_y_anim * (1.0 - t)) as i16
                )
            },
            CastState::Cast { t } => {
                (
                    160-52 - (24.0 * (1.0 - t)) as i16 + (sword_arm_x_anim * t) as i16,
                    96 - 30 - (14.0 * (1.0 - t)) as i16 + (sword_arm_y_anim * t) as i16
                )
            },
            _ => {
                (160-52 + sword_arm_x_anim as i16, 96 - 30 + sword_arm_y_anim as i16)
            }
        };

        BlitBuilder::create(ctx, &sprite_sheet_with_color_key)
            .with_source_subrect(0, 24, 48, 48)
            .with_dest_pos(spell_arm_x, spell_arm_y)
            .blit();

        BlitBuilder::create(ctx, &sprite_sheet_with_color_key)
            .with_source_subrect(48, 24, 48, 48)
            .with_dest_pos(sword_arm_x, sword_arm_y)
            .blit();
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
                    Some(12),
                );
            }
            AppOverlayState::NoOverlay => {}
            AppOverlayState::HelpContent => {
                self.font.draw_text_in_box(
                    ctx,
                    0, -2,
                    160, 96,
                    HorizontalAlignment::Center,
                    VerticalAlignment::Center,
                    r##"Arrows: Movement
Alt: Strafe
Num keys 0-9: just check out
-/=: Tweak terrain quality
F1: Toggle help
Tab: Toggle map
Esc: Quit game"##,
                    Some(12),
                );
            }
            AppOverlayState::MinimapView => {
                self.render_minimap(ctx);
            }
        }
    }

    fn update_input(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        self.update_player_movement(ctx, dt);
        self.update_player_casting(ctx);
    }

    fn update_player_casting(&mut self, ctx: &mut RetroBlitContext) {
        let (cast_spell_pressed, cast_melee_pressed) = (
            ctx.is_key_pressed(KeyCode::Z),
            ctx.is_key_pressed(KeyCode::X)
        );

        if let Some((_, (_, mp, freeze_spell_cast_state, melee_cast_state))) = self.world.query::<(&Player, &mut MP, &mut FreezeSpellCastState, &mut MeleeCastState)>().iter().next() {
            match cast_spell_pressed {
                true if mp.0 >= 30 => {
                    if freeze_spell_cast_state.try_cast() {
                        mp.0 -= 30;
                    }
                },
                _ => ()
            }
            if cast_melee_pressed {
                melee_cast_state.try_cast();
            }
        }
    }

    fn update_projectiles(&mut self, dt: f32) {
        let spatial = &mut self.spatial_map;
        let cb = &mut self.command_buffer;
        let world = &self.world;

        fn do_work<TProjectile, TCast>
        (
            spatial_map: &mut flat_spatial::DenseGrid<Entity>,
            cb: &mut CommandBuffer,
            world: &World,
            dt: f32
        )
            where
                TProjectile: ProjectileBehaviour<TCast>,
                TCast: CastInfo
        {
            for (proj_entity, (proj, pos, desired_velocity, cast)) in world
                .query::<(&Projectile<TCast, TProjectile>, &mut Position, &DesiredVelocity, &TCast)>()
                .iter() {

                for (_, &other_entity) in spatial_map
                    .query_around([pos.x, pos.y], 24.0)
                    .filter_map(|it | spatial_map.get(it.0)) {

                    if proj.caster == other_entity {
                        continue;
                    }

                    TProjectile::collide(*pos, *cast, cb);
                    cb.despawn(proj_entity);
                    return;
                }

                if let Some((_, (wang_data, ))) = world.query::<(&WangTerrain, )>().iter().next() {
                    let (new_pos, collided) = collision::move_position_towards(
                        *pos,
                        vec2(desired_velocity.x * dt, desired_velocity.y * dt),
                        CollisionTag::Wall,
                        wang_data,
                    );
                    if collided {
                        TProjectile::collide(new_pos, *cast, cb);
                        cb.despawn(proj_entity);
                        return;
                    } else {
                        cb.spawn((TProjectile::make_particle(new_pos.x, new_pos.y),));
                    }
                    *pos = new_pos;
                }
            }
        }

        do_work::<FreezeSpellProjectile, _>(spatial, cb, world, dt);

        self.command_buffer.run_on(&mut self.world)
    }

    fn update_freeze_spell_blasts(&mut self) {
        let spatial = &mut self.spatial_map;
        let cb = &mut self.command_buffer;
        let world = &self.world;

        for (blast_entity, (_, pos, cast)) in world
            .query::<(&FreezeSpellBlast, &Position, &FreezeSpellCast)>()
            .iter() {

            for (_, &other_entity) in spatial
                .query_around([pos.x, pos.y], cast.blast_range)
                .filter_map(|it | spatial.get(it.0)) {

                if world.get::<FreezeStun>(other_entity).is_err() {
                    cb.insert(other_entity, (FreezeStun(cast.duration),));
                }
            }
            cb.despawn(blast_entity);
        }

        self.command_buffer.run_on(&mut self.world)
    }

    fn update_periodic_statuses<TStatus: PeriodicStatus>(&mut self, dt: f32) {
        let cb = &mut self.command_buffer;
        let world = &self.world;

        for (status_entity, (status, )) in world
            .query::<(&mut TStatus, )>()
            .iter() {

            if !status.update(dt) {
                TStatus::on_status_off(status_entity, cb);
            }
        }

        self.command_buffer.run_on(&mut self.world)
    }

    fn update_castings(&mut self, dt: f32) {
        let spatial = &mut self.spatial_map;
        let cb = &mut self.command_buffer;
        let world = &self.world;

        fn do_work<TCastState, TState>
        (
            spatial_map: &mut flat_spatial::DenseGrid<Entity>,
            cb: &mut CommandBuffer,
            world: &World,
            dt: f32,
            foo: impl Fn(
                &World,
                &mut CommandBuffer,
                &mut flat_spatial::DenseGrid<Entity>,
                TState,
                Entity,
                Position,
                Angle
            ) -> ()
        )
        where
            TCastState: CastStateImpl<TState>,
            TState: CastInfo
        {
            for (e, (cast_state, pos, ang, cast)) in world
                .query::<(&mut TCastState, &Position, &Angle, &TState)>()
                .iter() {
                if cast_state.update(dt) {
                    foo(world, cb, spatial_map, *cast, e, *pos, *ang);
                }
            }
        }

        do_work::<FreezeSpellCastState, _>(spatial, cb, world, dt, cast_freeze_spell);
        do_work::<MeleeCastState, _>(spatial, cb, world, dt, cast_melee);

        self.command_buffer.run_on(&mut self.world)
    }

    fn update_player_movement(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        let (strafe_speed, turn_speed): (f32, f32) = match (ctx.is_key_pressed(KeyCode::Left), ctx.is_key_pressed(KeyCode::Right)) {
            (true, false) => {
                if ctx.is_key_mod_pressed(KeyMod::Option) {
                    (180.0, 0.0)
                } else {
                    (0.0, -120.0)
                }
            }
            (false, true) => {
                if ctx.is_key_mod_pressed(KeyMod::Option) {
                    (-180.0, 0.0)
                } else {
                    (0.0, 120.0)
                }
            }
            _ => (0.0, 0.0)
        };

        let movement_speed: f32 = match (ctx.is_key_pressed(KeyCode::Down), ctx.is_key_pressed(KeyCode::Up)) {
            (true, false) => {
                -180.0
            }
            (false, true) => {
                180.0
            }
            _ => 0.0
        };

        let should_move = strafe_speed.abs() > 10.0 || movement_speed.abs() > 10.0;

        if let Some((_, player_data)) = self.world.query::<(&mut Player, &mut Position, &mut Angle, &mut MovementInertial)>().iter().next() {
            let (_, pos, angle, movement_inertial) = player_data;
            angle.0 += turn_speed * dt;
            let angle = angle.0.to_radians();
            let (s, c) = (angle.sin(), angle.cos());

            if should_move {
                let speed_x = movement_speed * s - strafe_speed * c;
                let speed_y = -movement_speed * c - strafe_speed * s;
                movement_inertial.x = if speed_x > 0.0 {
                    (movement_inertial.x + speed_x * dt * 5.0).min(speed_x)
                } else {
                    (movement_inertial.x + speed_x * dt * 5.0).max(speed_x)
                };
                movement_inertial.y = if speed_y > 0.0 {
                    (movement_inertial.y + speed_y * dt * 5.0).min(speed_y)
                } else {
                    (movement_inertial.y + speed_y * dt * 5.0).max(speed_y)
                };
            } else {
                movement_inertial.x -= movement_inertial.x * dt * 5.0;
                movement_inertial.y -= movement_inertial.y * dt * 5.0;
                if movement_inertial.x.abs() < 0.01 {
                    movement_inertial.x = 0.0;
                }
                if movement_inertial.y.abs() < 0.01 {
                    movement_inertial.y = 0.0;
                }
            }

            const SPEED_COEFF: f32 = 180.0 * 180.0;

            self.hand_wave_state.amount = (
                movement_inertial.x * movement_inertial.x +
                movement_inertial.y * movement_inertial.y
            ) / SPEED_COEFF;

            self.hand_wave_state.t += self.hand_wave_state.amount * dt * 4.0;

            self.with_wang_data(|wang_data| {
                let (new_pos, _) = collision::move_position_towards(
                    *pos,
                    glam::vec2(movement_inertial.x * dt, movement_inertial.y * dt),
                    CollisionTag::All,
                    wang_data,
                );
                *pos = new_pos;
            });
        }
    }

    fn update_palette(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        match &mut self.palette_state {
            PaletteState::ScrollingWater => {
                self.scroll_timer += dt;

                while self.scroll_timer > 0.2 {
                    self.scroll_timer -= 0.2;
                    for i in 0..7 {
                        ctx.scroll_palette(
                            ScrollKind::Range { start_idx: 26 + 36 * i, len: 6 },
                            ScrollDirection::Forward,
                        );
                    }
                }
            }
            PaletteState::HpPickupTint { t } => {
                if *t <= 0.0 {
                    for (ix, clr) in self.last_palette.iter().enumerate() {
                        ctx.set_palette(ix as _, *clr);
                    }
                    self.palette_state = PaletteState::ScrollingWater;
                } else {
                    for (ix, clr) in self.last_palette.iter().enumerate() {
                        let r = clr[0] as f32 / 2.0;
                        let g = (clr[1] as f32 + 255.0) / 2.0;
                        let b = (clr[2] as f32 + 63.0) / 2.0;
                        let clr = [
                            utils::lerp(clr[0] as f32, r, *t).clamp(0.0, 255.0) as u8,
                            utils::lerp(clr[1] as f32, g, *t).clamp(0.0, 255.0) as u8,
                            utils::lerp(clr[2] as f32, b, *t).clamp(0.0, 255.0) as u8
                        ];
                        ctx.set_palette(ix as _, clr);
                    }
                    *t -= dt * TINT_FADE_OUT_SPEED;
                }
            }
            PaletteState::MpPickupTint { t } => {
                if *t <= 0.0 {
                    for (ix, clr) in self.last_palette.iter().enumerate() {
                        ctx.set_palette(ix as _, *clr);
                    }
                    self.palette_state = PaletteState::ScrollingWater;
                } else {
                    for (ix, clr) in self.last_palette.iter().enumerate() {
                        let r = (clr[0] as f32 + 63.0) / 2.0;
                        let g = (clr[1] as f32 + 127.0) / 2.0;
                        let b = (clr[2] as f32 + 255.0) / 2.0;
                        let clr = [
                            utils::lerp(clr[0] as f32, r, *t).clamp(0.0, 255.0) as u8,
                            utils::lerp(clr[1] as f32, g, *t).clamp(0.0, 255.0) as u8,
                            utils::lerp(clr[2] as f32, b, *t).clamp(0.0, 255.0) as u8
                        ];
                        ctx.set_palette(ix as _, clr);
                    }
                    *t -= dt * TINT_FADE_OUT_SPEED;
                }
            }
            PaletteState::DamageTint { t } => {
                if *t <= 0.0 {
                    for (ix, clr) in self.last_palette.iter().enumerate() {
                        ctx.set_palette(ix as _, *clr);
                    }
                    self.palette_state = PaletteState::ScrollingWater;
                } else {
                    for (ix, clr) in self.last_palette.iter().enumerate() {
                        let r = (clr[0] as f32 + 255.0) / 2.0;
                        let g = clr[1] as f32 / 2.0;
                        let b = clr[2] as f32 / 2.0;
                        let clr = [
                            utils::lerp(clr[0] as f32, r, *t).clamp(0.0, 255.0) as u8,
                            utils::lerp(clr[1] as f32, g, *t).clamp(0.0, 255.0) as u8,
                            utils::lerp(clr[2] as f32, b, *t).clamp(0.0, 255.0) as u8
                        ];
                        ctx.set_palette(ix as _, clr);
                    }
                    *t -= dt * TINT_FADE_OUT_SPEED;
                }
            }
        }
    }

    fn render_minimap(&self, ctx: &mut RetroBlitContext) {
        let start_x;
        let start_y;
        let angle;

        const DENOMINATOR: f32 = 32.0;

        if let Some((_, data)) = self.world.query::<(&Player, &Position, &Angle)>().iter().next() {
            let (_, &Position { x, y }, &Angle(a)) = data;

            angle = a.to_radians();

            let (remapped_x, remapped_y) = (x / DENOMINATOR, y / DENOMINATOR);
            start_x = -(remapped_x as i32);
            start_y = -(remapped_y as i32);
        } else {
            return;
        }

        let mut collision_vec = CollisionVec::new();

        self.with_wang_data(|wang_terrain| {
            for j in 0..MapData::HEIGHT - 1 {
                for i in 0..MapData::WIDTH - 1 {
                    let idx = j * (MapData::WIDTH - 1) + i;
                    if !wang_terrain.seen_tiles.contains(&[i as u16, j as u16]) {
                        continue;
                    }

                    collision_vec.clear();
                    collision::populate_collisions(
                        &mut collision_vec,
                        &wang_terrain.tiles[idx],
                        i as f32 * 64.0,
                        j as f32 * 64.0,
                    );
                    for collision in collision_vec.iter() {
                        let p0 = (
                            80 + start_x as i16 + (collision.x0 / DENOMINATOR) as i16,
                            48 + start_y as i16 + (collision.y0 / DENOMINATOR) as i16
                        );
                        let p1 = (
                            80 + start_x as i16 + (collision.x1 / DENOMINATOR) as i16,
                            48 + start_y as i16 + (collision.y1 / DENOMINATOR) as i16
                        );
                        LineRasterizer::create(ctx)
                            .from(p0)
                            .to(p1)
                            .rasterize(match collision.tag {
                                CollisionTag::Water => 35,
                                CollisionTag::Wall => 14,
                                CollisionTag::All => 12
                            });
                    }

                    BresenhamCircleDrawer::create(ctx)
                        .with_position((80, 48))
                        .with_radius(2)
                        .draw(12);

                    let view_vec = (4.0 * angle.sin(), -4.0 * angle.cos());

                    LineRasterizer::create(ctx)
                        .from((80, 48))
                        .to(((80.0 + view_vec.0) as _, (48.0 + view_vec.1) as _))
                        .rasterize(12);
                }
            }
        });
    }

    fn render_objects(&mut self, ctx: &mut RetroBlitContext) {
        let Some((_, (_, &Position { x, y }, &Angle(angle)))) =
            self.world
                .query::<(&Player, &Position, &Angle)>()
                .iter()
                .next() else { return; };

        let angle = angle.to_radians();
        let forward = (angle.sin(), -angle.cos());
        let right = (angle.cos(), angle.sin());
        let pos_x = x;
        let pos_y = y;

        for (_, (&potion, &Position { x, y })) in self.world.query::<(&Potion, &Position)>().iter() {
            let d_p = (x - pos_x, y - pos_y);
            let t = utils::dot(d_p, forward);
            if (NEAR..=FAR).contains(&t) {
                let depth = (t - NEAR) / (FAR - NEAR);
                let u = utils::dot(d_p, right) / t / self.flags.fov_slope;

                let x_scale = 40.0 * Self::scale_y(depth, self.flags.fov_slope);
                let up = self.project_height(-24.0, depth);
                let down = self.project_height(56.0, depth);

                let upper = (up).max(0.0) as usize;
                let lower = (down).min(96.0) as usize;

                let u_corr = (u + 1.0) * 79.5;
                let left = u_corr - x_scale;
                let right = u_corr + x_scale;

                if left >= 0.0 || right < 160.0 {
                    for j in upper..lower {
                        let v = ((j as f32 - up) / (down - up)).clamp(0.0, 1.0);
                        for i in left.max(0.0) as usize..right.min(159.0) as usize {
                            let u = ((i as f32 - left) / (right - left)).clamp(0.0, 1.0);
                            let idx = j * 160 + i;

                            let ix = (u * 23.0) as usize + 96;
                            let iy = (v * 23.0) as usize + match potion {
                                Potion::Health => 24,
                                Potion::Mana => 0
                            };

                            let source_idx = self.graphics.get_width() * iy + ix;
                            let color = self.graphics.get_buffer()[source_idx];

                            if color != 0 && self.depth_buffer[idx] > depth {
                                self.depth_buffer[idx] = depth;
                                ctx.get_buffer_mut()[idx] = color;
                            }
                        }
                    }
                }
            }
        }

        for (monster_entity, (&monster, &Position { x, y })) in self.world.query::<(&Monster, &Position)>().iter() {
            let frozen = self.world.get::<FreezeStun>(monster_entity).is_ok();
            let has_damage_tint = self.world.get::<DamageTint>(monster_entity).is_ok();

            let d_p = (x - pos_x, y - pos_y);
            let t = utils::dot(d_p, forward);
            if (NEAR..=FAR).contains(&t) {
                let depth = (t - NEAR) / (FAR - NEAR);
                let u = utils::dot(d_p, right) / t / self.flags.fov_slope;

                let x_scale = 40.0 * Self::scale_y(depth, self.flags.fov_slope);
                let up = self.project_height(-24.0, depth);
                let down = self.project_height(56.0, depth);

                let upper = (up).max(0.0) as usize;
                let lower = (down).min(96.0) as usize;

                let u_corr = (u + 1.0) * 79.5;
                let left = u_corr - x_scale;
                let right = u_corr + x_scale;

                if left >= 0.0 || right < 160.0 {
                    for j in upper..lower {
                        let v = ((j as f32 - up) / (down - up)).clamp(0.0, 1.0);
                        for i in left.max(0.0) as usize..right.min(159.0) as usize {
                            let u = ((i as f32 - left) / (right - left)).clamp(0.0, 1.0);
                            let idx = j * 160 + i;

                            let (ix, iy) = if frozen {
                                let ix = (u * 23.0) as usize + match monster {
                                    Monster::Toad => 48,
                                    Monster::Kobold => 96,
                                    Monster::Rat => 96,
                                    Monster::Skeleton => 72
                                };
                                let iy = (v * 23.0) as usize + match monster {
                                    Monster::Toad => 72,
                                    Monster::Kobold => 72,
                                    Monster::Rat => 48,
                                    Monster::Skeleton => 72
                                };
                                (ix, iy)
                            } else {
                                let ix = (u * 23.0) as usize + match monster {
                                    Monster::Toad => 0,
                                    Monster::Kobold => 24,
                                    Monster::Rat => 48,
                                    Monster::Skeleton => 72
                                };
                                let iy = (v * 23.0) as usize;
                                (ix, iy)
                            };

                            let source_idx = self.graphics.get_width() * iy + ix;
                            let color = self.graphics.get_buffer()[source_idx];

                            if color != 0 && self.depth_buffer[idx] > depth {
                                self.depth_buffer[idx] = depth;
                                ctx.get_buffer_mut()[idx] = if has_damage_tint { 12 } else { color };
                            }
                        }
                    }
                }
            }
        }

        for (_, (&monster, &Position { x, y })) in self.world.query::<(&MonsterCorpseGhost, &Position)>().iter() {
            let frozen = monster.frozen;

            let d_p = (x - pos_x, y - pos_y);
            let t = utils::dot(d_p, forward);
            if (NEAR..=FAR).contains(&t) {
                let depth = (t - NEAR) / (FAR - NEAR);
                let u = utils::dot(d_p, right) / t / self.flags.fov_slope;

                let x_scale = 40.0 * Self::scale_y(depth, self.flags.fov_slope);
                let up = self.project_height(-24.0, depth);
                let down = self.project_height(56.0, depth);

                let upper = (up).max(0.0) as usize;
                let lower = (down).min(96.0) as usize;

                let u_corr = (u + 1.0) * 79.5;
                let left = u_corr - x_scale;
                let right = u_corr + x_scale;

                if left >= 0.0 || right < 160.0 {
                    for j in upper..lower {
                        let v = ((j as f32 - up) / (down - up)).clamp(0.0, 1.0);
                        for i in left.max(0.0) as usize..right.min(159.0) as usize {
                            let u = ((i as f32 - left) / (right - left)).clamp(0.0, 1.0);
                            let idx = j * 160 + i;

                            let (ix, iy) = if frozen {
                                let ix = (u * 23.0) as usize + match monster.monster {
                                    Monster::Toad => 48,
                                    Monster::Kobold => 96,
                                    Monster::Rat => 96,
                                    Monster::Skeleton => 72
                                };
                                let iy = (v * 23.0) as usize + match monster.monster {
                                    Monster::Toad => 72,
                                    Monster::Kobold => 72,
                                    Monster::Rat => 48,
                                    Monster::Skeleton => 72
                                };
                                (ix, iy)
                            } else {
                                let ix = (u * 23.0) as usize + match monster.monster {
                                    Monster::Toad => 0,
                                    Monster::Kobold => 24,
                                    Monster::Rat => 48,
                                    Monster::Skeleton => 72
                                };
                                let iy = (v * 23.0) as usize;
                                (ix, iy)
                            };


                            let source_idx = self.graphics.get_width() * iy + ix;
                            let color = self.graphics.get_buffer()[source_idx];

                            let lookup_idx = (j % 128) * 128 + i % 128;
                            let color = if monster.life_time > self.noise_dither_lookup[lookup_idx] {
                                color
                            } else {
                                0
                            };

                            if color != 0 && self.depth_buffer[idx] > depth {
                                self.depth_buffer[idx] = depth;
                                ctx.get_buffer_mut()[idx] = color;
                            }
                        }
                    }
                }
            }
        }
    }

    fn render_particles(&mut self, ctx: &mut RetroBlitContext) {
        let (forward, right, pos_x, pos_y);
        if let Some((_, data)) = self.world.query::<(&Player, &Position, &Angle)>().iter().next() {
            let (_, &Position { x, y }, &Angle(angle)) = data;
            let angle = angle.to_radians();
            forward = (angle.sin(), -angle.cos());
            right = (angle.cos(), angle.sin());
            pos_x = x;
            pos_y = y;
        } else {
            return;
        }

        for (_, (&Particle { color_id, x, y, h, .. },)) in self.world.query::<(&Particle, )>().iter() {
            let d_p = (x - pos_x, y - pos_y);
            let t = utils::dot(d_p, forward);
            if (NEAR..=FAR).contains(&t) {
                let depth = (t - NEAR) / (FAR - NEAR);
                let u = utils::dot(d_p, right) / t / self.flags.fov_slope;

                let up = self.project_height(-h, depth);

                let upper = (up).max(0.0) as usize;

                let u_corr = (u + 1.0) * 79.5;

                if u_corr >= 0.0 || u_corr < 160.0 {
                    let j = upper;
                    let i = u_corr.clamp(0.0, 159.0) as usize;
                    let idx = j * 160 + i;

                    if idx < self.depth_buffer.len() && self.depth_buffer[idx] > depth {
                        self.depth_buffer[idx] = depth;
                        ctx.get_buffer_mut()[idx] = color_id;
                    }
                }
            }
        }
    }

    pub(crate) fn with_wang_data(&self, mut foo: impl FnMut(&WangTerrain)) {
        if let Some((_, (wang_data, ))) = self.world.query::<(&WangTerrain, )>().iter().next() {
            foo(wang_data)
        }
    }

    pub(crate) fn with_wang_data_mut(&self, mut foo: impl FnMut(&mut WangTerrain)) {
        if let Some((_, (wang_data, ))) = self.world.query::<(&mut WangTerrain, )>().iter().next() {
            foo(wang_data)
        }
    }

    pub(crate) fn set_palette_state(&mut self, ctx: &mut RetroBlitContext, palette_state: PaletteState) {
        match palette_state {
            PaletteState::ScrollingWater => (),
            _ => {
                match self.palette_state {
                    PaletteState::ScrollingWater => {
                        for i in 0..self.last_palette.len() {
                            self.last_palette[i] = ctx.get_palette(i as _);
                        }
                    }
                    _ => ()
                }
            }
        }
        self.palette_state = palette_state;
    }
}

impl ContextHandler for App {
    fn get_window_title(&self) -> &'static str { "dungeon crawler example" }

    fn get_window_mode(&self) -> WindowMode { WindowMode::Mode160x120 }

    fn on_key_up(&mut self, ctx: &mut RetroBlitContext, key_code: KeyCode, _key_mods: KeyMods) {
        match key_code {
            KeyCode::Key1 => {
                self.flags.fov_slope = 0.7;
            }
            KeyCode::Key2 => {
                self.flags.fov_slope = 0.8;
            }
            KeyCode::Key3 => {
                self.flags.fov_slope = 0.9;
            }
            KeyCode::Key4 => {
                self.flags.fov_slope = 1.0;
            }
            KeyCode::Key5 => {
                self.flags.fov_slope = 1.1;
            }
            KeyCode::Key6 => {
                self.flags.fov_slope = 1.2;
            }
            KeyCode::Key7 => {
                self.flags.fov_slope = 1.3;
            }
            KeyCode::Key8 => {
                self.flags.fov_slope = 1.4;
            }
            KeyCode::Key0 => {
                self.flags.texture_terrain = !self.flags.texture_terrain;
            }
            KeyCode::Key9 => {
                self.flags.dim_level = match self.flags.dim_level {
                    DimLevel::FullWithBlueNoise => DimLevel::FullWithDither,
                    DimLevel::FullWithDither => DimLevel::DimOnly,
                    DimLevel::DimOnly => DimLevel::FullWithBlueNoise
                };
            }
            KeyCode::Minus => {
                self.flags.terrain_rendering_step = (self.flags.terrain_rendering_step * 2.0)
                    .clamp(1.0 / 4096.0, 1.0 / 8.0);
            }
            KeyCode::Equal => {
                self.flags.terrain_rendering_step = (self.flags.terrain_rendering_step / 2.0)
                    .clamp(1.0 / 4096.0, 1.0 / 8.0);
            }
            KeyCode::F1 => {
                self.overlay_state = match self.overlay_state {
                    AppOverlayState::HelpContent => AppOverlayState::NoOverlay,
                    _ => AppOverlayState::HelpContent
                };
            }
            KeyCode::Tab => {
                self.overlay_state = match self.overlay_state {
                    AppOverlayState::MinimapView => AppOverlayState::NoOverlay,
                    _ => AppOverlayState::MinimapView
                };
            }
            KeyCode::Escape => {
                ctx.quit();
            }
            _ => ()
        }
    }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        ctx.hide_cursor();

        let mut offset = 0;
        let total_colors = self.last_palette.len() * 7;
        let darkest_blue = self.last_palette[DARKEST_BLUE_IDX];
        for i in 0..7 {
            for &pal_color in self.last_palette.iter() {
                let warm_overlay = [
                    if pal_color[0] < 235 { pal_color[0] + 20 } else { 255 },
                    if pal_color[1] < 246 { pal_color[1] + 9 } else { 255 },
                    pal_color[2]
                ];
                let darken = [
                    ((pal_color[0] as u16 * 80) / 100) as u8,
                    ((pal_color[1] as u16 * 80) / 100) as u8,
                    ((pal_color[2] as u16 * 92) / 100) as u8
                ];
                let near_full_dark = [
                    ((darkest_blue[0] as u16 * 3 + darken[0] as u16) / 4) as u8,
                    ((darkest_blue[1] as u16 * 3 + darken[1] as u16) / 4) as u8,
                    ((darkest_blue[2] as u16 * 3 + darken[2] as u16) / 4) as u8
                ];

                ctx.set_palette(
                    offset,
                    match i {
                        0 => warm_overlay,
                        1 => [
                            ((warm_overlay[0] as u16 + pal_color[0] as u16) / 2) as u8,
                            ((warm_overlay[1] as u16 + pal_color[1] as u16) / 2) as u8,
                            ((warm_overlay[2] as u16 + pal_color[2] as u16) / 2) as u8
                        ],
                        2 => pal_color,
                        3 => [
                            ((darken[0] as u16 + pal_color[0] as u16) / 2) as u8,
                            ((darken[1] as u16 + pal_color[1] as u16) / 2) as u8,
                            ((darken[2] as u16 + pal_color[2] as u16) / 2) as u8
                        ],
                        4 => darken,
                        5 => [
                            ((darken[0] as u16 + near_full_dark[0] as u16) / 2) as u8,
                            ((darken[1] as u16 + near_full_dark[1] as u16) / 2) as u8,
                            ((darken[2] as u16 + near_full_dark[2] as u16) / 2) as u8
                        ],
                        _ => near_full_dark
                    },
                );
                if offset < 255 { offset += 1; }
            }
        }
        self.last_palette.resize(total_colors, [0, 0, 0]);
    }

    fn update(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        self.update_palette(ctx, dt);
        self.update_castings(dt);
        self.update_projectiles(dt);
        self.update_freeze_spell_blasts();
        self.update_periodic_statuses::<FreezeStun>(dt);
        self.update_periodic_statuses::<DamageTint>(dt);
        self.update_periodic_statuses::<MonsterCorpseGhost>(dt);
        self.update_periodic_statuses::<Particle>(dt);
        self.update_input(ctx, dt);
        self.update_blackboard();
        self.maintain_monster_hp();
        self.update_spatial_partition();
        self.update_ai(ctx, dt);
        self.update_pickups(ctx);
        self.render(ctx);
    }
}

fn main() {
    retro_blit::window::start(App::new());
}

#[inline(always)]
fn rotate(p: (f32, f32), angle: f32) -> (f32, f32) {
    let sin_cos = (angle.sin(), angle.cos());
    (
        p.0 * sin_cos.1 + p.1 * sin_cos.0,
        -p.0 * sin_cos.0 + p.1 * sin_cos.1
    )
}

#[inline(always)]
fn gen_trapezoid_coords(x: f32, y: f32, angle: f32, fov_slope: f32) -> [(f32, f32); 4] {
    [
        rotate((fov_slope * NEAR, NEAR), angle),
        rotate((-fov_slope * NEAR, NEAR), angle),
        rotate((-fov_slope * FAR, FAR), angle),
        rotate((fov_slope * FAR, FAR), angle)
    ].map(|p| (p.0 + x as f32, y as f32 - p.1))
}
