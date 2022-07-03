use retro_blit::rendering::blittable::BlitBuilder;
use retro_blit::rendering::bresenham::LineRasterizer;
use retro_blit::rendering::deformed_rendering::TriangleRasterizer;
use retro_blit::rendering::fonts::font_align::{HorizontalAlignment, VerticalAlignment};
use retro_blit::rendering::fonts::tri_spaced::TextDrawer;
use retro_blit::rendering::shapes::fill_rectangle;
use retro_blit::rendering::transform::Transform;
use retro_blit::window::{RetroBlitContext, ScrollDirection, ScrollKind};
use crate::DemoGame;
use crate::constants::*;
use crate::components::*;

impl DemoGame {
    pub fn update_star_sky(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        self.flicker_dt_accumulated += dt;
        while self.flicker_dt_accumulated >= STAR_FLICKER_PACE {
            self.flicker_dt_accumulated -= STAR_FLICKER_PACE;
            ctx.scroll_palette(
                ScrollKind::Range { start_idx: 2, len: 34 },
                ScrollDirection::Forward
            );
            ctx.scroll_palette(
                ScrollKind::Range { start_idx: 36, len: 44 },
                ScrollDirection::Forward
            )
        }
    }

    pub fn render(&mut self, ctx: &mut RetroBlitContext) {
        { // blit star sky
            BlitBuilder::create(ctx, &self.star_sky_sprite).blit();
        }

        { // draw player bullets
            for (_, (_, position, velocity)) in self.ecs_world
                .query::<(&Bullet, &Position, &Velocity)>()
                .iter() {
                let [p0, p1] = Self::make_bullet_segment(position, velocity);
                LineRasterizer::create(ctx)
                    .from(p0)
                    .to(p1)
                    .rasterize(PLAYER_COLOR);
            }
        }

        { // draw player
            for (_, (_, pos, rotation)) in self.ecs_world
                .query::<(&Player, &Position, &Rotation)>()
                .iter() {
                TriangleRasterizer::create(ctx)
                    .with_transform(
                        Transform::from_angle_and_translation(
                            rotation.angle,
                            pos.x as i16,
                            pos.y as i16
                        )
                    )
                    .rasterize_with_color(
                        PLAYER_COLOR,
                        &self.player_vertices,
                        &self.player_indices
                    );
            }
        }

        { // draw player scrap
            for (_, (_, pos, rotation)) in self.ecs_world
                .query::<(&PlayerScrap, &Position, &Rotation)>()
                .iter() {
                TriangleRasterizer::create(ctx)
                    .with_transform(
                        Transform::from_angle_and_translation(
                            rotation.angle,
                            pos.x as i16,
                            pos.y as i16
                        )
                    )
                    .rasterize_with_color(
                        PLAYER_COLOR,
                        &self.player_scrap_vertices,
                        &self.player_scrap_indices
                    );
            }
        }

        { // draw asteroids
            for (_, (&Asteroid { kind, size, .. }, pos, rotation)) in self.ecs_world
                .query::<(&Asteroid, &Position, &Rotation)>()
                .iter() {
                let (vertices, indices) = match kind {
                    AsteroidKind::Round => (
                        &self.round_asteroid_vertices,
                        &self.round_asteroid_indices
                    ),
                    AsteroidKind::Rocky => (
                        &self.rocky_asteroid_vertices,
                        &self.rocky_asteroid_indices
                    ),
                    AsteroidKind::Square => (
                        &self.square_asteroid_vertices,
                        &self.square_asteroid_indices
                    )
                };
                let color = get_asteroid_color(kind);
                TriangleRasterizer::create(ctx)
                    .with_transform(
                        Transform::from_angle_translation_scale(
                            rotation.angle,
                            (pos.x as i16, pos.y as i16),
                            (size, size)
                        )
                    )
                    .rasterize_with_color(
                        color,
                        vertices,
                        indices
                    );
            }
        }

        { // draw lives indicator
            let lives_text = format!("Lives: {}", self.player_hp);

            fill_rectangle(ctx, 136, 0, 48, 12, 85);
            self.font.draw_text_in_box(
                ctx,
                136, 0,
                48, 12,
                HorizontalAlignment::Center,
                VerticalAlignment::Center,
                &lives_text,
                Some(PLAYER_COLOR)
            );

            if self.game_lost() {
                self.font.draw_text_in_box(
                    ctx,
                    0, 0,
                    320, 240,
                    HorizontalAlignment::Center,
                    VerticalAlignment::Center,
                    "Game over. Press enter to restart",
                    Some(PLAYER_COLOR)
                );
            } else if self.game_won() {
                self.font.draw_text_in_box(
                    ctx,
                    0, 0,
                    320, 240,
                    HorizontalAlignment::Center,
                    VerticalAlignment::Center,
                    "You win! Press enter to restart",
                    Some(PLAYER_COLOR)
                );
            }
        }
    }
}

fn get_asteroid_color(kind: AsteroidKind) -> u8 {
    ASTEROID_COLORS[match kind {
        AsteroidKind::Round => 0,
        AsteroidKind::Rocky => 1,
        AsteroidKind::Square => 2
    }]
}