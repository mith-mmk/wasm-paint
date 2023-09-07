use crate::canvas::*;
use crate::grayscale::to_grayscale;
use std::io::Error;

pub struct Kernel {
    pub width: usize,
    pub height: usize,
    pub matrix: Vec<Vec<f32>>,
}

impl Kernel {
    pub fn new(matrix: [[f32; 3]; 3]) -> Self {
        let matrix = vec![matrix[0].to_vec(), matrix[1].to_vec(), matrix[2].to_vec()];
        Self {
            width: 3,
            height: 3,
            matrix,
        }
    }
}

#[inline]
fn rgb_to_yuv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32;
    let g = g as f32;
    let b = b as f32;
    let y = 0.29900 * r + 0.58700 * g + 0.11400 * b;
    let u = -0.16874 * r - 0.33126 * g + 0.50000 * b;
    let v = 0.50000 * r - 0.41869 * g - 0.081 * b;
    (y, u, v)
}

#[inline]
fn rgb_to_y(r: u8, g: u8, b: u8) -> f32 {
    let r = r as f32;
    let g = g as f32;
    let b = b as f32;
    
    0.29900 * r + 0.58700 * g + 0.11400 * b
}

#[inline]
fn yuv_to_rgb(y: f32, u: f32, v: f32) -> (u8, u8, u8) {
    let crr = 1.402;
    let cbg = -0.34414;
    let crg = -0.71414;
    let cbb = 1.772;

    let r = (y + (crr * v)) as i32;
    let g = (y + (cbg * u) + crg * v) as i32;
    let b = (y + (cbb * u)) as i32;

    let r = r.clamp(0, 255) as u8;
    let g = g.clamp(0, 255) as u8;
    let b = b.clamp(0, 255) as u8;

    (r, g, b)
}

pub fn lum_filter(src: &dyn Screen, dest: &mut dyn Screen, kernel: &Kernel) {
    if dest.width() == 0 || dest.height() == 0 {
        dest.reinit(src.width(), src.height());
    }
    let dest_height = dest.height() as usize;
    let dest_width = dest.width() as usize;

    let mut coeff = 0.0;
    let matrix = &kernel.matrix;
    let u0 = (kernel.height + 1) as i32;
    let v0 = (kernel.width + 1) as i32;
    for u in 0..kernel.height {
        for v in 0..kernel.width {
            coeff += matrix[v][u];
        }
    }
    let src_buffer = src.buffer();
    let dest_buffer = dest.buffer_mut();

    for y in 0..src.height() as usize {
        let offset = y * src.width() as usize * 4;
        if y >= dest_height {
            break;
        }
        for x in 0..src.width() as usize {
            if x >= dest_width {
                break;
            }
            let r = src_buffer[offset + x * 4];
            let g = src_buffer[offset + x * 4 + 1];
            let b = src_buffer[offset + x * 4 + 2];
            let a = src_buffer[offset + x * 4 + 3];
            let mut l = 0.0;
            for u in 0..kernel.height {
                let uu = (y as i32 + u as i32 - u0).clamp(0, src.height() as i32 - 1) as usize
                    * src.width() as usize
                    * 4;
                for v in 0..kernel.width {
                    let vv =
                        (x as i32 + v as i32 - v0).clamp(0, src.width() as i32 - 1) as usize * 4;
                    let r = src_buffer[uu + vv];
                    let g = src_buffer[uu + vv + 1];
                    let b = src_buffer[uu + vv + 2];
                    let ln = rgb_to_y(r, g, b);
                    l += ln * matrix[v][u];
                }
            }
            let l = l / coeff;
            let (_, u, v) = rgb_to_yuv(r, g, b);
            let (r, g, b) = yuv_to_rgb(l, u, v);

            dest_buffer[offset + x * 4] = r;
            dest_buffer[offset + x * 4 + 1] = g;
            dest_buffer[offset + x * 4 + 2] = b;
            dest_buffer[offset + x * 4 + 3] = a;
        }
    }
}

pub fn grayscale(src: &dyn Screen, dest: &mut dyn Screen) {
    if dest.width() == 0 || dest.height() == 0 {
        dest.reinit(src.width(), src.height());
    }
    let dest_height = dest.height() as usize;
    let dest_width = dest.width() as usize;

    let src_buffer = src.buffer();
    let dest_buffer = dest.buffer_mut();

    for y in 0..src.height() as usize {
        let offset = y * src.width() as usize * 4;
        if y >= dest_height {
            break;
        }
        for x in 0..src.width() as usize {
            if x >= dest_width {
                break;
            }
            let r = src_buffer[offset + x * 4];
            let g = src_buffer[offset + x * 4 + 1];
            let b = src_buffer[offset + x * 4 + 2];
            let a = src_buffer[offset + x * 4 + 3];
            let (l, _, _) = rgb_to_yuv(r, g, b);
            let l = (l.round() as i16).clamp(0, 255) as u8;

            dest_buffer[offset + x * 4] = l;
            dest_buffer[offset + x * 4 + 1] = l;
            dest_buffer[offset + x * 4 + 2] = l;
            dest_buffer[offset + x * 4 + 3] = a;
        }
    }
}

pub fn rgb_filter(src: &dyn Screen, dest: &mut dyn Screen, kernel: &Kernel) {
    if dest.width() == 0 || dest.height() == 0 {
        dest.reinit(src.width(), src.height());
    }
    let dest_height = dest.height() as usize;
    let dest_width = dest.width() as usize;

    let mut coeff = 0.0;
    let matrix = &kernel.matrix;
    let u0 = (kernel.height + 1) as i32;
    let v0 = (kernel.width + 1) as i32;
    for u in 0..kernel.height {
        for v in 0..kernel.width {
            coeff += matrix[v][u];
        }
    }
    let src_buffer = src.buffer();
    let dest_buffer = dest.buffer_mut();

    for y in 0..src.height() as usize {
        let offset = y * src.width() as usize * 4;
        if y >= dest_height {
            break;
        }
        for x in 0..src.width() as usize {
            if x >= dest_width {
                break;
            }
            let mut r = 0.0;
            let mut g = 0.0;
            let mut b = 0.0;
            let a = src_buffer[offset + x * 4 + 3];
            for u in 0..kernel.height {
                let uu = (y as i32 + u as i32 - u0).clamp(0, src.height() as i32 - 1) as usize
                    * src.width() as usize
                    * 4;
                for v in 0..kernel.width {
                    let vv =
                        (x as i32 + v as i32 - v0).clamp(0, src.width() as i32 - 1) as usize * 4;
                    r += src_buffer[uu + vv] as f32 * matrix[v][u];
                    g += src_buffer[uu + vv + 1] as f32 * matrix[v][u];
                    b += src_buffer[uu + vv + 2] as f32 * matrix[v][u];
                }
            }
            let r = ((r / coeff).round() as i32).clamp(0, 255) as u8;
            let g = ((g / coeff).round() as i32).clamp(0, 255) as u8;
            let b = ((b / coeff).round() as i32).clamp(0, 255) as u8;

            dest_buffer[offset + x * 4] = r;
            dest_buffer[offset + x * 4 + 1] = g;
            dest_buffer[offset + x * 4 + 2] = b;
            dest_buffer[offset + x * 4 + 3] = a;
        }
    }
}

pub fn sharpness(src: &dyn Screen, dest: &mut dyn Screen) {
    let matrix = [[-1.0, -1.0, -1.0], [-1.0, 10.0, -1.0], [-1.0, -1.0, -1.0]];
    lum_filter(src, dest, &Kernel::new(matrix))
}

pub fn blur(src: &dyn Screen, dest: &mut dyn Screen) {
    let matrix = [[1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0]];
    lum_filter(src, dest, &Kernel::new(matrix))
}

pub fn ranking(src: &dyn Screen, dest: &mut dyn Screen, rank: usize) {
    if dest.width() == 0 || dest.height() == 0 {
        dest.reinit(src.width(), src.height());
    }

    let dest_height = dest.height() as usize;
    let dest_width = dest.width() as usize;

    let src_buffer = src.buffer();
    let dest_buffer = dest.buffer_mut();

    for y in 0..src.height() as usize {
        let offset = y * src.width() as usize * 4;
        if y >= dest_height {
            break;
        }
        for x in 0..src.width() as usize {
            if x >= dest_width {
                break;
            }
            let a = src_buffer[offset + x * 4 + 3];
            let mut l = [(0.0, 0, 0, 0); 9];
            for u in 0..3 {
                let uu = (y as i32 + u - 1).clamp(0, src.height() as i32 - 1) as usize
                    * src.width() as usize
                    * 4;
                for v in 0..3 {
                    let vv = (x as i32 + v - 1).clamp(0, src.width() as i32 - 1) as usize * 4;
                    let r = src_buffer[uu + vv];
                    let g = src_buffer[uu + vv + 1];
                    let b = src_buffer[uu + vv + 2];
                    let ln = rgb_to_y(r, g, b);
                    l[u as usize * 3 + v as usize] = (ln, r, g, b);
                }
            }
            let mut l = l.to_vec();
            l.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let l = l[rank];

            dest_buffer[offset + x * 4] = l.1;
            dest_buffer[offset + x * 4 + 1] = l.2;
            dest_buffer[offset + x * 4 + 2] = l.3;
            dest_buffer[offset + x * 4 + 3] = a;
        }
    }
}

pub fn filter(src: &dyn Screen, dest: &mut dyn Screen, filter_name: &str) -> Result<(), Error> {
    match filter_name {
        "median" => ranking(src, dest, 4),
        "erode" => ranking(src, dest, 0),
        "dilate" => ranking(src, dest, 8),
        "sharpness" => sharpness(src, dest),
        "blur" => blur(src, dest),
        "average" => {
            let matrix = [[1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0]];
            rgb_filter(src, dest, &Kernel::new(matrix));
        }

        "smooth" => {
            let matrix = [[1.0, 1.0, 1.0], [1.0, 4.0, 1.0], [1.0, 1.0, 1.0]];
            lum_filter(src, dest, &Kernel::new(matrix));
        }
        "sharpen" => {
            let matrix = [[-1.0, -1.0, -1.0], [-1.0, 12.0, -1.0], [-1.0, -1.0, -1.0]];
            lum_filter(src, dest, &Kernel::new(matrix));
        }
        "shardow" => {
            let matrix = [[1.0, 2.0, 1.0], [0.0, 1.0, 0.0], [-1.0, -2.0, -1.0]];
            lum_filter(src, dest, &Kernel::new(matrix));
        }
        "edges" => {
            let matrix_a = [[1.0, 2.0, 1.0], [0.0, 0.0, 0.0], [-1.0, -2.0, -1.0]];
            let matrix_b = [[1.0, 0.0, -1.0], [2.0, 0.0, -2.0], [1.0, 0.0, -1.0]];
            let mut tmp = Canvas::new(src.width(), src.height());
            lum_filter(src, &mut tmp, &Kernel::new(matrix_a));
            lum_filter(&tmp, dest, &Kernel::new(matrix_b));
        }
        "grayscale" => {
            to_grayscale(src, dest, 0);
        }
        &_ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unknown filter",
            ))
        }
    }
    Ok(())
}
