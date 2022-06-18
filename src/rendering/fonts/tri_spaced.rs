use std::collections::HashMap;
use maplit::hashmap;
use crate::rendering::blittable::{BlitBuilder};
use crate::rendering::{BlittableSurface};
use crate::rendering::fonts::font_align::{HorizontalAlignment, VerticalAlignment};
use crate::window::RetroBlitContext;

const DEFAULT_TRISPACED_FONT_BYTES: &[u8] = include_bytes!("default_trispaced_font.im256");
const DEFAULT_TRISPACED_FONT_SMALL_BYTES: &[u8] = include_bytes!("default_trispaced_font_small.im256");

#[derive(Copy, Clone)]
#[repr(usize)]
pub enum GlyphWidth {
    Narrow = 1,
    Normal = 2,
    Wide = 3
}

impl std::ops::Mul<usize> for GlyphWidth {
    type Output = usize;

    fn mul(self, rhs: usize) -> Self::Output {
        self as usize * rhs
    }
}

#[derive(Copy, Clone)]
pub struct GlyphInfo {
    /// offset in x steps of a font info,
    /// e.g. in pixels it would correspond to x_offset * glyph_grid_step_x
    pub x_offset: usize,

    /// offset in y steps of a font info,
    /// e.g. in pixels it would correspond to y_offset * glyph_grid_step_y
    pub y_offset: usize,

    /// in pixels it would correspond to (1|2|3) * glyph_grid_step_x
    pub width: GlyphWidth
}

#[derive(Copy, Clone)]
pub struct GlyphMetrics {
    pub x_pos: usize,
    pub y_pos: usize,
    pub width: usize
}

pub struct FontInfo {
    pub upper_cap_offset: usize,
    pub base_line_offset: usize,
    pub glyph_grid_step_x: usize,
    pub glyph_grid_step_y: usize,
    pub default_glyph_info: GlyphInfo,
    pub font_mapping: HashMap<char, GlyphInfo>
}

impl FontInfo {
    pub fn font_height(&self) -> usize {
        self.base_line_offset - self.upper_cap_offset
    }

    pub fn get_glyph_metrics(&self, chr: char) -> GlyphMetrics {
        let mapping = self.font_mapping
            .get(&chr)
            .map(|it| *it)
            .unwrap_or(self.default_glyph_info);

        GlyphMetrics {
            x_pos: mapping.x_offset * self.glyph_grid_step_x,
            y_pos: mapping.y_offset * self.glyph_grid_step_y,
            width: mapping.width * self.glyph_grid_step_x
        }
    }

    pub fn measure_word_width(&self, s: &str) -> usize {
        s.chars()
            .map(|it| self.get_glyph_metrics(it).width)
            .sum()
    }
}

pub struct Font {
    font_info: FontInfo,
    surface: BlittableSurface,
    arena: bumpalo::Bump,
}

pub trait TextDrawer<Destination> {

    fn draw_text(
        &self, destination: &mut Destination,
        x: i32, y: i32, text: &str,
        color_tint_idx: Option<u8>
    );

    fn draw_text_in_box(
        &self, destination: &mut Destination,
        x: i32, y: i32,
        box_width: usize, box_height: usize,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
        text: &str,
        color_tint_idx: Option<u8>
    );
}

macro_rules! impl_text_drawer {
    ($dest_type: ident) => {
        impl TextDrawer<$dest_type> for Font {
            fn draw_text(&self, destination: &mut $dest_type, x: i32, y: i32, text: &str, color_tint_idx: Option<u8>) {
                let height = self.font_info.glyph_grid_step_y;
                let mut current_y = y - (self.font_info.upper_cap_offset as i32);
                let mut current_x = x;
                for c in text.chars() {
                    if c.is_ascii_whitespace() {
                        if c == ' ' {
                            current_x += self.font_info.glyph_grid_step_x as i32;
                        } else if c == '\n' {
                            current_x = x;
                            current_y += height as i32;
                        }
                        continue;
                    }

                    let GlyphMetrics {
                        x_pos, y_pos,
                        width
                    } = self.font_info.get_glyph_metrics(c);

                    match color_tint_idx {
                        None => {
                            BlitBuilder::create(destination, &self.surface.with_color_key(0))
                                .with_dest_pos(current_x, current_y)
                                .with_source_subrect(x_pos, y_pos, width, height)
                                .blit();
                        },
                        Some(idx) => {
                            BlitBuilder::create(destination, &self.surface.with_color_key_blink(0, idx))
                                .with_dest_pos(current_x, current_y)
                                .with_source_subrect(x_pos, y_pos, width, height)
                                .blit();
                        }
                    }

                    current_x += width as i32;
                }
            }

            fn draw_text_in_box(&self, destination: &mut $dest_type, x: i32, y: i32, box_width: usize, box_height: usize, horizontal_alignment: HorizontalAlignment, vertical_alignment: VerticalAlignment, text: &str, color_tint_idx: Option<u8>) {
                struct LineInfo {
                    word_count: usize,
                    empty_space: i32
                }

                let mut line_words = bumpalo::collections::Vec::new_in(&self.arena);
                let mut line_info_vec = bumpalo::collections::Vec::new_in(&self.arena);

                for line in text.lines() {
                    line_words.clear();
                    for word in line.split_ascii_whitespace() {
                        line_words.push(word);
                    }

                    let mut current_x = 0;
                    let mut current_words = 0;
                    for word in line_words.iter() {
                        let new_width = self.font_info.measure_word_width(*word);

                        let next_x = if current_x == 0 {
                            current_x + new_width
                        } else {
                            current_x + new_width + self.font_info.glyph_grid_step_x
                        };

                        if next_x > box_width {
                            if current_words == 0 {
                                line_info_vec.push(LineInfo { word_count: 1, empty_space: box_width as i32 - next_x as i32 });
                                current_x = 0;
                            } else {
                                line_info_vec.push(
                                    LineInfo {
                                        word_count: current_words,
                                        empty_space: box_width as i32 - current_x as i32
                                    }
                                );
                                current_x = new_width;
                                current_words = 1;
                            }
                            continue;
                        }
                        current_x = next_x;
                        current_words += 1;
                    }

                    if current_words > 0 {
                        line_info_vec.push(
                            LineInfo {
                                word_count: current_words,
                                empty_space: box_width as i32 - current_x as i32
                            }
                        );
                    }
                }

                let mut words = text.split_ascii_whitespace();

                let height = self.font_info.glyph_grid_step_y as i32;
                let result_height = self.font_info.glyph_grid_step_y * line_info_vec.len() -
                    (self.font_info.glyph_grid_step_y - self.font_info.base_line_offset);
                let mut current_y = y + match vertical_alignment {
                    VerticalAlignment::Top => 0,
                    VerticalAlignment::Center =>
                        (box_height / 2) as i32 -
                            (result_height / 2) as i32,
                    VerticalAlignment::Bottom =>
                        box_height as i32 - result_height as i32
                };
                for LineInfo{ word_count, empty_space } in line_info_vec.iter() {
                    let mut current_x = x + match horizontal_alignment {
                        HorizontalAlignment::Left => 0,
                        HorizontalAlignment::Center => *empty_space / 2,
                        HorizontalAlignment::Right => *empty_space
                    };
                    for i in 0..*word_count {
                        if let Some(word) = words.next() {
                            if i != 0 {
                                current_x += self.font_info.glyph_grid_step_x as i32;
                            }
                            self.draw_text(destination, current_x, current_y, word, color_tint_idx);
                            current_x += self.font_info.measure_word_width(word) as i32;
                        }
                    }
                    current_y += height;
                }
            }
        }
    }
}

impl_text_drawer!(RetroBlitContext);
impl_text_drawer!(BlittableSurface);

impl Font {
    pub fn new(font_info: FontInfo, surface: BlittableSurface) -> Self {
        Self {
            font_info,
            surface,
            arena: bumpalo::Bump::new()
        }
    }

    pub fn default_font_small() -> std::io::Result<Self> {
        let (_, surface) = crate::format_loaders::im_256::Image::load_from(DEFAULT_TRISPACED_FONT_SMALL_BYTES)?;
        let font_info = FontInfo {
            upper_cap_offset: 1,
            base_line_offset: 9,
            glyph_grid_step_x: 3,
            glyph_grid_step_y: 12,
            default_glyph_info: GlyphInfo {
                x_offset: 14,
                y_offset: 2,
                width: GlyphWidth::Normal
            },
            font_mapping: hashmap!{
                'a' => GlyphInfo {
                    x_offset: 0,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'b' => GlyphInfo {
                    x_offset: 2,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'c' => GlyphInfo {
                    x_offset: 4,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'd' => GlyphInfo {
                    x_offset: 6,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'e' => GlyphInfo {
                    x_offset: 8,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'f' => GlyphInfo {
                    x_offset: 10,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'g' => GlyphInfo {
                    x_offset: 12,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'h' => GlyphInfo {
                    x_offset: 14,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'i' => GlyphInfo {
                    x_offset: 16,
                    y_offset: 0,
                    width: GlyphWidth::Narrow
                },
                'j' => GlyphInfo {
                    x_offset: 17,
                    y_offset: 0,
                    width: GlyphWidth::Narrow
                },
                'k' => GlyphInfo {
                    x_offset: 18,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'l' => GlyphInfo {
                    x_offset: 20,
                    y_offset: 0,
                    width: GlyphWidth::Narrow
                },
                'm' => GlyphInfo {
                    x_offset: 21,
                    y_offset: 0,
                    width: GlyphWidth::Wide
                },
                'n' => GlyphInfo {
                    x_offset: 24,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'o' => GlyphInfo {
                    x_offset: 26,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'p' => GlyphInfo {
                    x_offset: 28,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'q' => GlyphInfo {
                    x_offset: 30,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'r' => GlyphInfo {
                    x_offset: 32,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                's' => GlyphInfo {
                    x_offset: 34,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                't' => GlyphInfo {
                    x_offset: 36,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'u' => GlyphInfo {
                    x_offset: 38,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'v' => GlyphInfo {
                    x_offset: 40,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'w' => GlyphInfo {
                    x_offset: 42,
                    y_offset: 0,
                    width: GlyphWidth::Wide
                },
                'x' => GlyphInfo {
                    x_offset: 45,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'y' => GlyphInfo {
                    x_offset: 47,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'z' => GlyphInfo {
                    x_offset: 49,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                '.' => GlyphInfo {
                    x_offset: 51,
                    y_offset: 0,
                    width: GlyphWidth::Narrow
                },
                ',' => GlyphInfo {
                    x_offset: 52,
                    y_offset: 0,
                    width: GlyphWidth::Narrow
                },
                ':' => GlyphInfo {
                    x_offset: 53,
                    y_offset: 0,
                    width: GlyphWidth::Narrow
                },
                '!' => GlyphInfo {
                    x_offset: 54,
                    y_offset: 0,
                    width: GlyphWidth::Narrow
                },
                ';' => GlyphInfo {
                    x_offset: 55,
                    y_offset: 0,
                    width: GlyphWidth::Narrow
                },
                '"' => GlyphInfo {
                    x_offset: 56,
                    y_offset: 0,
                    width: GlyphWidth::Narrow
                },
                '\'' => GlyphInfo {
                    x_offset: 57,
                    y_offset: 0,
                    width: GlyphWidth::Narrow
                },
                '`' => GlyphInfo {
                    x_offset: 58,
                    y_offset: 0,
                    width: GlyphWidth::Narrow
                },
                '(' => GlyphInfo {
                    x_offset: 59,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                ')' => GlyphInfo {
                    x_offset: 61,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                '|' => GlyphInfo {
                    x_offset: 63,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'A' => GlyphInfo {
                    x_offset: 0,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'B' => GlyphInfo {
                    x_offset: 2,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'C' => GlyphInfo {
                    x_offset: 4,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'D' => GlyphInfo {
                    x_offset: 6,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'E' => GlyphInfo {
                    x_offset: 8,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'F' => GlyphInfo {
                    x_offset: 10,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'G' => GlyphInfo {
                    x_offset: 12,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'H' => GlyphInfo {
                    x_offset: 14,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'I' => GlyphInfo {
                    x_offset: 16,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'J' => GlyphInfo {
                    x_offset: 18,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'K' => GlyphInfo {
                    x_offset: 20,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'L' => GlyphInfo {
                    x_offset: 22,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'M' => GlyphInfo {
                    x_offset: 24,
                    y_offset: 1,
                    width: GlyphWidth::Wide
                },
                'N' => GlyphInfo {
                    x_offset: 27,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'O' => GlyphInfo {
                    x_offset: 29,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'P' => GlyphInfo {
                    x_offset: 31,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'Q' => GlyphInfo {
                    x_offset: 33,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'R' => GlyphInfo {
                    x_offset: 35,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'S' => GlyphInfo {
                    x_offset: 37,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'T' => GlyphInfo {
                    x_offset: 39,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'U' => GlyphInfo {
                    x_offset: 41,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'V' => GlyphInfo {
                    x_offset: 43,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'W' => GlyphInfo {
                    x_offset: 45,
                    y_offset: 1,
                    width: GlyphWidth::Wide
                },
                'X' => GlyphInfo {
                    x_offset: 48,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'Y' => GlyphInfo {
                    x_offset: 50,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'Z' => GlyphInfo {
                    x_offset: 52,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                '-' => GlyphInfo {
                    x_offset: 54,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                '+' => GlyphInfo {
                    x_offset: 56,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                '*' => GlyphInfo {
                    x_offset: 58,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                '/' => GlyphInfo {
                    x_offset: 60,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                '\\' => GlyphInfo {
                    x_offset: 62,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                '0' => GlyphInfo {
                    x_offset: 0,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '1' => GlyphInfo {
                    x_offset: 2,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '2' => GlyphInfo {
                    x_offset: 4,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '3' => GlyphInfo {
                    x_offset: 6,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '4' => GlyphInfo {
                    x_offset: 8,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '5' => GlyphInfo {
                    x_offset: 10,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '6' => GlyphInfo {
                    x_offset: 12,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '7' => GlyphInfo {
                    x_offset: 14,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '8' => GlyphInfo {
                    x_offset: 16,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '9' => GlyphInfo {
                    x_offset: 18,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '[' => GlyphInfo {
                    x_offset: 20,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                ']' => GlyphInfo {
                    x_offset: 22,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '{' => GlyphInfo {
                    x_offset: 24,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '}' => GlyphInfo {
                    x_offset: 26,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '>' => GlyphInfo {
                    x_offset: 28,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '<' => GlyphInfo {
                    x_offset: 30,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '~' => GlyphInfo {
                    x_offset: 32,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '=' => GlyphInfo {
                    x_offset: 34,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '%' => GlyphInfo {
                    x_offset: 36,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '@' => GlyphInfo {
                    x_offset: 38,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '&' => GlyphInfo {
                    x_offset: 40,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '_' => GlyphInfo {
                    x_offset: 42,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '#' => GlyphInfo {
                    x_offset: 44,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '$' => GlyphInfo {
                    x_offset: 46,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '№' => GlyphInfo {
                    x_offset: 48,
                    y_offset: 2,
                    width: GlyphWidth::Wide
                },
                '?' => GlyphInfo {
                    x_offset: 51,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '^' => GlyphInfo {
                    x_offset: 51,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
            }
        };
        Ok(Self::new(font_info, surface))
    }

    pub fn default_font() -> std::io::Result<Self> {
        let (_, surface) = crate::format_loaders::im_256::Image::load_from(DEFAULT_TRISPACED_FONT_BYTES)?;
        let font_info = FontInfo {
            upper_cap_offset: 1,
            base_line_offset: 11,
            glyph_grid_step_x: 5,
            glyph_grid_step_y: 16,
            default_glyph_info: GlyphInfo {
                x_offset: 4,
                y_offset: 3,
                width: GlyphWidth::Normal
            },
            font_mapping: hashmap!{
                'a' => GlyphInfo {
                    x_offset: 0,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'b' => GlyphInfo {
                    x_offset: 2,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'c' => GlyphInfo {
                    x_offset: 4,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'd' => GlyphInfo {
                    x_offset: 6,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'e' => GlyphInfo {
                    x_offset: 8,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'f' => GlyphInfo {
                    x_offset: 10,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'g' => GlyphInfo {
                    x_offset: 12,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'h' => GlyphInfo {
                    x_offset: 14,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'i' => GlyphInfo {
                    x_offset: 16,
                    y_offset: 0,
                    width: GlyphWidth::Narrow
                },
                'j' => GlyphInfo {
                    x_offset: 17,
                    y_offset: 0,
                    width: GlyphWidth::Narrow
                },
                'k' => GlyphInfo {
                    x_offset: 18,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'l' => GlyphInfo {
                    x_offset: 20,
                    y_offset: 0,
                    width: GlyphWidth::Narrow
                },
                'm' => GlyphInfo {
                    x_offset: 21,
                    y_offset: 0,
                    width: GlyphWidth::Wide
                },
                'n' => GlyphInfo {
                    x_offset: 24,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'o' => GlyphInfo {
                    x_offset: 26,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'p' => GlyphInfo {
                    x_offset: 28,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'q' => GlyphInfo {
                    x_offset: 30,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'r' => GlyphInfo {
                    x_offset: 32,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                's' => GlyphInfo {
                    x_offset: 34,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                't' => GlyphInfo {
                    x_offset: 36,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'u' => GlyphInfo {
                    x_offset: 38,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'v' => GlyphInfo {
                    x_offset: 40,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'w' => GlyphInfo {
                    x_offset: 42,
                    y_offset: 0,
                    width: GlyphWidth::Wide
                },
                'x' => GlyphInfo {
                    x_offset: 45,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'y' => GlyphInfo {
                    x_offset: 47,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                'z' => GlyphInfo {
                    x_offset: 49,
                    y_offset: 0,
                    width: GlyphWidth::Normal
                },
                '.' => GlyphInfo {
                    x_offset: 51,
                    y_offset: 0,
                    width: GlyphWidth::Narrow
                },
                'A' => GlyphInfo {
                    x_offset: 0,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'B' => GlyphInfo {
                    x_offset: 2,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'C' => GlyphInfo {
                    x_offset: 4,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'D' => GlyphInfo {
                    x_offset: 6,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'E' => GlyphInfo {
                    x_offset: 8,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'F' => GlyphInfo {
                    x_offset: 10,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'G' => GlyphInfo {
                    x_offset: 12,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'H' => GlyphInfo {
                    x_offset: 14,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'I' => GlyphInfo {
                    x_offset: 16,
                    y_offset: 1,
                    width: GlyphWidth::Narrow
                },
                'J' => GlyphInfo {
                    x_offset: 17,
                    y_offset: 1,
                    width: GlyphWidth::Narrow
                },
                'K' => GlyphInfo {
                    x_offset: 18,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'L' => GlyphInfo {
                    x_offset: 20,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'M' => GlyphInfo {
                    x_offset: 22,
                    y_offset: 1,
                    width: GlyphWidth::Wide
                },
                'N' => GlyphInfo {
                    x_offset: 25,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'O' => GlyphInfo {
                    x_offset: 27,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'P' => GlyphInfo {
                    x_offset: 29,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'Q' => GlyphInfo {
                    x_offset: 31,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'R' => GlyphInfo {
                    x_offset: 33,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'S' => GlyphInfo {
                    x_offset: 35,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'T' => GlyphInfo {
                    x_offset: 37,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'U' => GlyphInfo {
                    x_offset: 39,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'V' => GlyphInfo {
                    x_offset: 41,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'W' => GlyphInfo {
                    x_offset: 43,
                    y_offset: 1,
                    width: GlyphWidth::Wide
                },
                'X' => GlyphInfo {
                    x_offset: 46,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'Y' => GlyphInfo {
                    x_offset: 48,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                'Z' => GlyphInfo {
                    x_offset: 50,
                    y_offset: 1,
                    width: GlyphWidth::Normal
                },
                '0' => GlyphInfo {
                    x_offset: 0,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '1' => GlyphInfo {
                    x_offset: 2,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '2' => GlyphInfo {
                    x_offset: 4,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '3' => GlyphInfo {
                    x_offset: 6,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '4' => GlyphInfo {
                    x_offset: 8,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '5' => GlyphInfo {
                    x_offset: 10,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '6' => GlyphInfo {
                    x_offset: 12,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '7' => GlyphInfo {
                    x_offset: 14,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '8' => GlyphInfo {
                    x_offset: 16,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '9' => GlyphInfo {
                    x_offset: 18,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '"' => GlyphInfo {
                    x_offset: 20,
                    y_offset: 2,
                    width: GlyphWidth::Narrow
                },
                '\'' => GlyphInfo {
                    x_offset: 21,
                    y_offset: 2,
                    width: GlyphWidth::Narrow
                },
                '`' => GlyphInfo {
                    x_offset: 22,
                    y_offset: 2,
                    width: GlyphWidth::Narrow
                },
                ':' => GlyphInfo {
                    x_offset: 23,
                    y_offset: 2,
                    width: GlyphWidth::Narrow
                },
                '!' => GlyphInfo {
                    x_offset: 24,
                    y_offset: 2,
                    width: GlyphWidth::Narrow
                },
                ';' => GlyphInfo {
                    x_offset: 25,
                    y_offset: 2,
                    width: GlyphWidth::Narrow
                },
                '-' => GlyphInfo {
                    x_offset: 26,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '+' => GlyphInfo {
                    x_offset: 28,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '*' => GlyphInfo {
                    x_offset: 30,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '\\' => GlyphInfo {
                    x_offset: 32,
                    y_offset: 2,
                    width: GlyphWidth::Narrow
                },
                '/' => GlyphInfo {
                    x_offset: 33,
                    y_offset: 2,
                    width: GlyphWidth::Narrow
                },
                '(' => GlyphInfo {
                    x_offset: 34,
                    y_offset: 2,
                    width: GlyphWidth::Narrow
                },
                ')' => GlyphInfo {
                    x_offset: 35,
                    y_offset: 2,
                    width: GlyphWidth::Narrow
                },
                '[' => GlyphInfo {
                    x_offset: 36,
                    y_offset: 2,
                    width: GlyphWidth::Narrow
                },
                ']' => GlyphInfo {
                    x_offset: 37,
                    y_offset: 2,
                    width: GlyphWidth::Narrow
                },
                '{' => GlyphInfo {
                    x_offset: 38,
                    y_offset: 2,
                    width: GlyphWidth::Narrow
                },
                '}' => GlyphInfo {
                    x_offset: 39,
                    y_offset: 2,
                    width: GlyphWidth::Narrow
                },
                '|' => GlyphInfo {
                    x_offset: 40,
                    y_offset: 2,
                    width: GlyphWidth::Narrow
                },
                '>' => GlyphInfo {
                    x_offset: 41,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '<' => GlyphInfo {
                    x_offset: 43,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '~' => GlyphInfo {
                    x_offset: 45,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '=' => GlyphInfo {
                    x_offset: 47,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '%' => GlyphInfo {
                    x_offset: 49,
                    y_offset: 2,
                    width: GlyphWidth::Normal
                },
                '@' => GlyphInfo {
                    x_offset: 0,
                    y_offset: 3,
                    width: GlyphWidth::Normal
                },
                '&' => GlyphInfo {
                    x_offset: 2,
                    y_offset: 3,
                    width: GlyphWidth::Normal
                },
                '_' => GlyphInfo {
                    x_offset: 4,
                    y_offset: 3,
                    width: GlyphWidth::Normal
                },
                '#' => GlyphInfo {
                    x_offset: 6,
                    y_offset: 3,
                    width: GlyphWidth::Normal
                },
                '$' => GlyphInfo {
                    x_offset: 8,
                    y_offset: 3,
                    width: GlyphWidth::Normal
                },
                '№' => GlyphInfo {
                    x_offset: 10,
                    y_offset: 3,
                    width: GlyphWidth::Wide
                },
                ',' => GlyphInfo {
                    x_offset: 13,
                    y_offset: 3,
                    width: GlyphWidth::Narrow
                },
                '?' => GlyphInfo {
                    x_offset: 14,
                    y_offset: 3,
                    width: GlyphWidth::Normal
                },
                '^' => GlyphInfo {
                    x_offset: 16,
                    y_offset: 3,
                    width: GlyphWidth::Normal
                },
            }
        };
        Ok(Self::new(font_info, surface))
    }
}