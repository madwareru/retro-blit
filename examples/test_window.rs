use retro_blit::rendering::blittable::{BlitBuilder};
use retro_blit::rendering::BlittableSurface;
use retro_blit::rendering::bresenham::{LineStripRasterizer};
use retro_blit::rendering::deformed_rendering::{TexturedVertex, TriangleRasterizer, Vertex};
use retro_blit::rendering::fonts::font_align::{HorizontalAlignment, VerticalAlignment};
use retro_blit::rendering::fonts::tri_spaced::{Font, TextDrawer};
use retro_blit::rendering::tessellation::PathTessellator;
use retro_blit::rendering::transform::Transform;
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
    offset: usize,
    palette: Vec<[u8; 3]>,
    sprite_sheet: BlittableSurface,
    font: Font,
    level: [[Tile; 5]; 5],
    button_pressed: bool,
    poly_line_positions: Vec<(i32, i32)>,
    poly_line_vertices: Vec<Vertex>,
    poly_line_indices: Vec<u16>
}
impl MyGame {
    pub fn new() -> Self {
        let (palette, sprite_sheet) = retro_blit::format_loaders::im_256::Image::load_from(PICTURE_BYTES).unwrap();
        let font = Font::default_font_small().unwrap();
        let mut poly_line_vertices = Vec::new();
        let mut poly_line_indices = Vec::new();

        let poly_line_positions = vec![
            (-30, -40),
            (70, 0),
            (-30, 40),
            (00, 0)
        ];

        PathTessellator::new().tessellate_polyline_fill(
            &mut poly_line_vertices,
            &mut poly_line_indices,
            &poly_line_positions
        );
        Self {
            offset: 4045,
            poly_line_positions,
            poly_line_vertices,
            poly_line_indices,
            palette,
            sprite_sheet,
            font,
            level: [
                [Tile::Water, Tile::Water, Tile::Water, Tile::Water, Tile::Water],
                [Tile::Water, Tile::Water, Tile::Sand, Tile::Sand, Tile::Grass],
                [Tile::Grass, Tile::Grass, Tile::Grass, Tile::Grass, Tile::Wheat],
                [Tile::Grass, Tile::Wheat, Tile::Wheat, Tile::Wheat, Tile::Dirt],
                [Tile::Water, Tile::Grass, Tile::Sand, Tile::Dirt, Tile::Dirt]
            ],
            button_pressed: false
        }
    }
}

impl ContextHandler for MyGame {
    fn get_window_title(&self) -> &'static str {
        "test_window"
    }

    fn get_window_mode(&self) -> WindowMode {
        WindowMode::Mode13
    }

    fn on_mouse_down(&mut self, _ctx: &mut RetroBlitContext, button_number: u8) {
        if button_number == 0 {
            self.button_pressed = true;
        }
    }

    fn on_mouse_up(&mut self, _ctx: &mut RetroBlitContext, button_number: u8) {
        self.offset += 1;
        if button_number == 0 {
            self.button_pressed = false;
        }
    }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        for i in 0..self.palette.len() {
            ctx.set_palette(i as _, self.palette[i]);
        }
    }

    fn update(&mut self, ctx: &mut RetroBlitContext) {
        ctx.clear(1);

        let sprite_sheet_with_color_key = self
            .sprite_sheet
            .with_color_key(0);

        for j in 0..5 {
            let y_coord = 44 + (j as i32) * 20;
            for i in 0..5 {
                let x_coord = 100 + (i as i32) * 24;
                let (tx, ty) = self.level[j][i].to_tile_coords();
                BlitBuilder::create(ctx, &sprite_sheet_with_color_key)
                    .with_source_subrect(tx, ty, 24, 32)
                    .with_dest_pos(x_coord, y_coord)
                    .blit();
            }
        }

        self.font.draw_text_in_box(
            ctx,
            0, 0,
            120, 100,
            HorizontalAlignment::Left,
            VerticalAlignment::Top,
            "Hello word.\nOops, a typo! We wanted to say a \"world\"!\n Please don't be too angry, will you?",
            Some(40)
        );

        let (mouse_x, mouse_y) = ctx.get_mouse_pos();

        let offset_as_a_turtle = self.offset * 2;

        let vertices = [
            TexturedVertex { position: (24, 20), uv: (23, 19) },
            TexturedVertex { position: (-24, 20), uv: (0, 19) },
            TexturedVertex { position: (-24, -20), uv: (0, 0) },
            TexturedVertex { position: (24, -20), uv: (23, 0) },
        ];

        TriangleRasterizer::create(ctx)
            .with_rotation((offset_as_a_turtle as f32 / 3.0).to_radians())
            .with_translation((64, 64))
            .rasterize_with_color(
                37,
                &self.poly_line_vertices,
                &self.poly_line_indices
            );

        LineStripRasterizer::create(ctx)
            .with_color(31)
            .with_rotation((offset_as_a_turtle as f32 / 3.0).to_radians())
            .with_translation((64, 64))
            .rasterize_slice(true, &self.poly_line_positions);

        {
            let transform = Transform::from_angle_translation_scale(
                (offset_as_a_turtle as f32).to_radians(),
                (160, 100),
                (
                    2.0 + (offset_as_a_turtle as f32 / 150.0).cos(),
                    0.5 + (offset_as_a_turtle as f32 / 137.0).sin()
                )
            );

            TriangleRasterizer::create(ctx)
                .with_transform(transform)
                .rasterize_with_surface(
                    &sprite_sheet_with_color_key,
                    &vertices,
                    &[0, 1, 2, 0, 2, 3]
                );
        }

        if self.button_pressed {
            BlitBuilder::create(ctx, &(self.sprite_sheet.with_color_key_blink(0, 40)))
                .with_source_subrect(0, 0, 24, 20)
                .with_dest_pos(mouse_x as _, mouse_y as _)
                .blit();
        } else {
            BlitBuilder::create(ctx, &sprite_sheet_with_color_key)
                .with_source_subrect(0, 0, 24, 20)
                .with_dest_pos(mouse_x as _, mouse_y as _)
                .blit();
        }
        self.offset += 1;
    }
}


fn main() {
    retro_blit::window::start(MyGame::new());
}