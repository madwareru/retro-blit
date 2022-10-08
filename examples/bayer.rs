use retro_blit::rendering::blittable::{BufferProviderMut, SizedSurface};
use retro_blit::utility::StopWatch;
use retro_blit::window::{RetroBlitContext, ContextHandler, WindowMode};

const BAYER_LOOKUP: [u8; 16] = [
    00, 08, 02, 10,
    12, 04, 14, 06,
    03, 11, 01, 09,
    15, 07, 13, 05
];

struct App {

}
impl ContextHandler for App {
    fn get_window_title(&self) -> &'static str {
        "bayer"
    }

    fn get_window_mode(&self) -> WindowMode {
        WindowMode::Mode160x120
    }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        let mut idx = 0;
        for j in 0..16 {
            for i in 0..16 {
                let red = 255.0 * (i as f32) / 15.0;
                let green = 255.0 * (j as f32) / 15.0;
                ctx.set_palette(idx, [red as _, green as _, 0]);
                if idx < 255 {
                    idx += 1;
                }
            }
        }
    }

    fn update(&mut self, ctx: &mut RetroBlitContext, _dt: f32) {
        let (w, h) = (ctx.get_width(), ctx.get_height());
        let (mx, my) = ctx.get_mouse_pos();
        {
            let buffer = ctx.get_buffer_mut();
            let _sw = StopWatch::named("bayer");
            for j in 0..h/4 {
                for i in 0..w/4 {
                    let mut lookup_idx = 0;
                    for j in j*4..(j+1) * 4 {
                        for i in i*4..(i+1) * 4 {
                            let idx = j * w + i;
                            let pal_idx = rg_to_pal_id(
                                [i as f32 / w as f32, j as f32 / h as f32],
                                lookup_idx
                            );
                            lookup_idx += 1;
                            unsafe {
                                *buffer.get_unchecked_mut(idx) = if (-5.0 ..= 5.0).contains(&((i as f32) - mx))
                                    && (-5.0 ..= 5.0).contains(&((j as f32) - my)) {
                                    255
                                } else {
                                    pal_idx
                                };
                            }
                        }
                    }
                }
            }
        }
    }
}

#[inline(always)]
fn rg_to_pal_id(rg: [f32; 2], lookup_idx: usize) -> u8 {
    let bayer = BAYER_LOOKUP[lookup_idx];

    let (r_id, g_id) = (rg[0] * 15.0, rg[1] * 15.0);

    let r_id = if (r_id.fract() * 16.0) as u8 <= bayer { r_id as u8 } else { (r_id as u8 + 1).min(15) };
    let g_id = if (g_id.fract() * 16.0) as u8 <= bayer { g_id as u8 } else { (g_id as u8 + 1).min(15) };

    (g_id as u8) * 16 + r_id as u8
}

fn main() {
    retro_blit::window::start(App{})
}