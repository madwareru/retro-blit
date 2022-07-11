use rand::Rng;
use retro_blit::{
    format_loaders::im_256,
    window::{RetroBlitContext, WindowMode},
    rendering::deformed_rendering::Vertex,
    rendering::tessellation::PathTessellator,
    window::KeyCode
};
use retro_blit::audio::SoundHandle;
use retro_blit::rendering::BlittableSurface;
use retro_blit::rendering::fonts::tri_spaced::Font;
use retro_blit::window::KeyMods;
use crate::components::{Asteroid, Position, SpatialHandle, Velocity};
use crate::constants::{PLAYER_POINTS, PLAYER_SCRAP_POINTS, ROCKY_ASTEROID_POINTS, ROUND_ASTEROID_POINTS, SQUARE_ASTEROID_POINTS, STAR_SKY_SPRITE_BYTES};

mod constants;
mod components;
mod subsystems;

pub struct Sounds {
    pub background_music: SoundHandle,
    pub laser_shot: SoundHandle,
    pub player_explode: SoundHandle,
    pub asteroid_explode: SoundHandle
}

pub struct DemoGame {
    pub sounds: Sounds,
    pub player_hp: u8,
    pub player_entity: Option<hecs::Entity>,
    pub bump_allocator: bumpalo::Bump,
    pub spatial_map: flat_spatial::DenseGrid<hecs::Entity>,
    pub player_vertices: Vec<Vertex>,
    pub player_indices: Vec<u16>,
    pub player_scrap_vertices: Vec<Vertex>,
    pub player_scrap_indices: Vec<u16>,
    pub round_asteroid_vertices: Vec<Vertex>,
    pub round_asteroid_indices: Vec<u16>,
    pub rocky_asteroid_vertices: Vec<Vertex>,
    pub rocky_asteroid_indices: Vec<u16>,
    pub square_asteroid_vertices: Vec<Vertex>,
    pub square_asteroid_indices: Vec<u16>,
    pub ecs_world: hecs::World,
    pub palette: Vec<[u8; 3]>,
    pub star_sky_sprite: BlittableSurface,
    pub font: Font,
    pub flicker_dt_accumulated: f32,
    pub music_handle: Option<usize>,
    pub has_sounds: bool,
    pub mute_sounds: bool,
    volume: f32
}

impl retro_blit::window::ContextHandler for DemoGame {
    fn get_window_title(&self) -> &'static str { "asteroids game" }

    fn get_window_mode(&self) -> WindowMode { WindowMode::ModeX }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        PathTessellator::new().tessellate_polyline_fill(
            &mut self.player_vertices,
            &mut self.player_indices,
            &PLAYER_POINTS
        );

        PathTessellator::new().tessellate_polyline_fill(
            &mut self.player_scrap_vertices,
            &mut self.player_scrap_indices,
            &PLAYER_SCRAP_POINTS
        );

        PathTessellator::new().tessellate_polyline_fill(
            &mut self.square_asteroid_vertices,
            &mut self.square_asteroid_indices,
            &SQUARE_ASTEROID_POINTS
        );

        PathTessellator::new().tessellate_polyline_fill(
            &mut self.round_asteroid_vertices,
            &mut self.round_asteroid_indices,
            &ROUND_ASTEROID_POINTS
        );

        PathTessellator::new().tessellate_polyline_fill(
            &mut self.rocky_asteroid_vertices,
            &mut self.rocky_asteroid_indices,
            &ROCKY_ASTEROID_POINTS
        );

        for (idx, &palette_color) in self.palette.iter().enumerate() {
            ctx.set_palette(idx as u8, palette_color);
        }

        if ctx.init_audio() {
            self.music_handle = ctx.play_sound(self.sounds.background_music.clone());
            self.has_sounds = true;
        }

        self.start_new_game();
    }

    fn on_key_up(&mut self, ctx: &mut RetroBlitContext, key_code: KeyCode, _key_mods: KeyMods) {
        match key_code {
            KeyCode::M => {
                self.mute_sounds = !self.mute_sounds;
                update_playback_volume(ctx, self.mute_sounds, self.volume);
            },
            KeyCode::Minus => {
                self.volume = (self.volume - 0.1).clamp(0.0, 1.0);
                update_playback_volume(ctx, self.mute_sounds, self.volume);
            },
            KeyCode::Equal => {
                self.volume = (self.volume + 0.1).clamp(0.0, 1.0);
                update_playback_volume(ctx, self.mute_sounds, self.volume);
            }
            _ => ()
        }

        fn update_playback_volume(ctx: &mut RetroBlitContext, mute_sounds: bool, volume: f32) {
            if mute_sounds {
                ctx.set_global_playback_volume(0.0);
            } else {
                ctx.set_global_playback_volume(volume);
            }
        }
    }

    fn update(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        if let Some(music_handle) = self.music_handle {
            if self.has_sounds && !ctx.playback_in_progress(music_handle) {
                self.music_handle = ctx.play_sound(self.sounds.background_music.clone());
            }
        }

        self.update_star_sky(ctx, dt);
        self.update_bullet_collisions(ctx);
        self.update_player_collisions(ctx);
        self.update_object_positions(dt);
        self.update_space_partitioning();
        self.update_life_spans(dt);
        self.update_fire_cool_downs(dt);
        self.update_revive_cool_down(dt);
        self.update_player_controls(ctx, dt);
        self.update_player_fire(ctx);

        if (self.game_lost() || self.game_won()) && ctx.is_key_pressed(KeyCode::Enter) {
            self.start_new_game();
        }

        self.render(ctx);
    }
}

impl DemoGame {
    pub fn new() -> Self {
        let (palette, star_sky_sprite) =
            im_256::Image::load_from(STAR_SKY_SPRITE_BYTES).unwrap();
        let font = Font::default_font_small().unwrap();
        Self {
            sounds: Sounds {
                background_music: SoundHandle::from_memory(
                    // Music by Trevor Lentz
                    // https://opengameart.org/content/hero-immortal
                    include_bytes!("assets/background_music.mp3")
                ).unwrap(),
                laser_shot: SoundHandle::from_memory(include_bytes!("assets/laser_shot.wav")).unwrap(),
                player_explode: SoundHandle::from_memory(include_bytes!("assets/player_explode.wav")).unwrap(),
                asteroid_explode: SoundHandle::from_memory(include_bytes!("assets/asteroid_explode.wav")).unwrap()
            },
            player_hp: 0,
            player_entity: None,
            bump_allocator: bumpalo::Bump::new(),
            spatial_map: flat_spatial::DenseGrid::new(32),
            player_vertices: Vec::new(),
            player_indices: Vec::new(),
            player_scrap_vertices: Vec::new(),
            player_scrap_indices: Vec::new(),
            round_asteroid_vertices: Vec::new(),
            round_asteroid_indices: Vec::new(),
            rocky_asteroid_vertices: Vec::new(),
            rocky_asteroid_indices: Vec::new(),
            square_asteroid_vertices: Vec::new(),
            square_asteroid_indices: Vec::new(),
            ecs_world: hecs::World::new(),
            palette,
            star_sky_sprite,
            font,
            flicker_dt_accumulated: 0.0,
            music_handle: None,
            has_sounds: false,
            mute_sounds: false,
            volume: 1.0
        }
    }

    pub fn kill_entity(&mut self, entity: hecs::Entity, spatial_handle: SpatialHandle) {
        self.spatial_map.remove(spatial_handle.handle);
        self.ecs_world.despawn(entity).unwrap();
    }

    pub fn game_won(&mut self) -> bool {
        self.ecs_world.query::<(&Asteroid,)>().iter().count() == 0
    }

    pub fn game_lost(&mut self) -> bool {
        self.player_hp == 0
    }

    pub fn make_bullet_segment(position: &Position, velocity: &Velocity) -> [(i16, i16); 2] {
        let p0 = (position.x as i16, position.y as i16);
        let p1 = (
            (position.x + velocity.x / 85.0) as i16,
            (position.y + velocity.y / 85.0) as i16
        );
        [p0, p1]
    }

    pub fn start_new_game(&mut self) {
        self.player_hp = 5;

        self.ecs_world.clear();

        self.spawn_new_player();

        let mut rng = rand::thread_rng();
        for _ in 0..5 {
            self.spawn_asteroid(
                Position {
                    x: rng.gen_range(0..320) as f32,
                    y: rng.gen_range(0..240) as f32
                },
                1.0 + rng.gen::<f32>(),
                1
            );
        }
    }
}

fn main() {
    retro_blit::window::start(DemoGame::new());
}