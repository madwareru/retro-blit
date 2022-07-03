use crate::rendering::blittable::{BufferProviderMut, SizedSurface};

pub fn fill_rectangle(
    dest: &mut (impl BufferProviderMut<u8> + SizedSurface),
    x: i16, y: i16,
    w: u16, h: u16,
    color: u8
) {
    let (dw, dh) = (dest.get_width(), dest.get_height());
    let mut w = w as i16;
    let mut h = h as i16;

    if x < 0 { w += x; }
    if y < 0 { h += y; }
    if w < 0 || h < 0 { return; }

    let x = x.max(0) as usize;
    let y = y.max(0) as usize;
    let w = w as usize;
    let h = h as usize;

    let left = x.min(dw);
    let right = (x + w).min(dw);
    let buffer = dest.get_buffer_mut();

    let mut stride = y * dw;
    for _ in y..(y + h).min(dh) {
        for px in &mut buffer[stride+left..stride+right] {
            *px = color;
        }
        stride += dw;
    }
}