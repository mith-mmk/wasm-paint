use crate::canvas::Screen;
use std::io::Error;

#[inline]
fn clamp_u8(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

#[inline]
fn rgb_to_ycbcr(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32;
    let g = g as f32;
    let b = b as f32;
    let y = 0.299 * r + 0.587 * g + 0.114 * b;
    let cb = 128.0 - 0.168_736 * r - 0.331_264 * g + 0.5 * b;
    let cr = 128.0 + 0.5 * r - 0.418_688 * g - 0.081_312 * b;
    (y, cb, cr)
}

#[inline]
fn ycbcr_to_rgb(y: f32, cb: f32, cr: f32) -> (u8, u8, u8) {
    let cb = cb - 128.0;
    let cr = cr - 128.0;
    let r = y + 1.402 * cr;
    let g = y - 0.344_136 * cb - 0.714_136 * cr;
    let b = y + 1.772 * cb;
    (clamp_u8(r), clamp_u8(g), clamp_u8(b))
}

#[inline]
fn luminance(r: u8, g: u8, b: u8) -> f32 {
    0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32
}

fn map_pixels<F>(src: &dyn Screen, dest: &mut dyn Screen, mut f: F) -> Result<(), Error>
where
    F: FnMut(u8, u8, u8, u8) -> (u8, u8, u8, u8),
{
    if dest.width() == 0 || dest.height() == 0 {
        dest.reinit(src.width(), src.height());
    }

    let src_width = src.width() as usize;
    let src_height = src.height() as usize;
    let dest_width = dest.width() as usize;
    let dest_height = dest.height() as usize;
    let src_buffer = src.buffer();
    let dest_buffer = dest.buffer_mut();

    for y in 0..src_height.min(dest_height) {
        for x in 0..src_width.min(dest_width) {
            let src_i = (y * src_width + x) * 4;
            let dest_i = (y * dest_width + x) * 4;
            let (r, g, b, a) = f(
                src_buffer[src_i],
                src_buffer[src_i + 1],
                src_buffer[src_i + 2],
                src_buffer[src_i + 3],
            );
            dest_buffer[dest_i] = r;
            dest_buffer[dest_i + 1] = g;
            dest_buffer[dest_i + 2] = b;
            dest_buffer[dest_i + 3] = a;
        }
    }

    Ok(())
}

pub fn gamma_control(src: &dyn Screen, dest: &mut dyn Screen, gamma: f32) -> Result<(), Error> {
    let gamma = if gamma <= 0.0 { 1.0 } else { gamma };
    let inv_gamma = 1.0 / gamma;
    map_pixels(src, dest, |r, g, b, a| {
        let curve = |v: u8| 255.0 * ((v as f32 / 255.0).powf(inv_gamma));
        (
            clamp_u8(curve(r)),
            clamp_u8(curve(g)),
            clamp_u8(curve(b)),
            a,
        )
    })
}

pub fn color_curve(
    src: &dyn Screen,
    dest: &mut dyn Screen,
    shadows: f32,
    midtones: f32,
    highlights: f32,
) -> Result<(), Error> {
    map_pixels(src, dest, |r, g, b, a| {
        let curve = |v: u8| {
            let t = v as f32 / 255.0;
            let low = (1.0 - t) * (1.0 - t) * shadows;
            let mid = 4.0 * t * (1.0 - t) * midtones;
            let high = t * t * highlights;
            clamp_u8(v as f32 + low + mid + high)
        };
        (curve(r), curve(g), curve(b), a)
    })
}

pub fn invert_crcb(src: &dyn Screen, dest: &mut dyn Screen) -> Result<(), Error> {
    map_pixels(src, dest, |r, g, b, a| {
        let (y, cb, cr) = rgb_to_ycbcr(r, g, b);
        let (r, g, b) = ycbcr_to_rgb(y, 255.0 - cb, 255.0 - cr);
        (r, g, b, a)
    })
}

pub fn saturation_control(
    src: &dyn Screen,
    dest: &mut dyn Screen,
    saturation: f32,
) -> Result<(), Error> {
    map_pixels(src, dest, |r, g, b, a| {
        let y = luminance(r, g, b);
        let adjust = |v: u8| y + (v as f32 - y) * saturation;
        (
            clamp_u8(adjust(r)),
            clamp_u8(adjust(g)),
            clamp_u8(adjust(b)),
            a,
        )
    })
}

pub fn brightness_control(
    src: &dyn Screen,
    dest: &mut dyn Screen,
    brightness: f32,
) -> Result<(), Error> {
    map_pixels(src, dest, |r, g, b, a| {
        (
            clamp_u8(r as f32 + brightness),
            clamp_u8(g as f32 + brightness),
            clamp_u8(b as f32 + brightness),
            a,
        )
    })
}

pub fn auto_brightness(src: &dyn Screen, dest: &mut dyn Screen) -> Result<(), Error> {
    let mut min_y = 255.0f32;
    let mut max_y = 0.0f32;
    for px in src.buffer().chunks_exact(4) {
        let y = luminance(px[0], px[1], px[2]);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }

    if (max_y - min_y).abs() < f32::EPSILON {
        return map_pixels(src, dest, |r, g, b, a| (r, g, b, a));
    }

    let scale = 255.0 / (max_y - min_y);
    map_pixels(src, dest, |r, g, b, a| {
        let adjust = |v: u8| (v as f32 - min_y) * scale;
        (
            clamp_u8(adjust(r)),
            clamp_u8(adjust(g)),
            clamp_u8(adjust(b)),
            a,
        )
    })
}
