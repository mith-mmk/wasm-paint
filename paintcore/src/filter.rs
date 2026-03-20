use crate::canvas::*;
use crate::grayscale::to_grayscale;
use crate::layer::Layer;
use std::{collections::HashMap, io::{Error, ErrorKind}};

#[derive(Clone)]
enum FilterType {
    Copy,
    Median,
    Erode,
    Dilate,
    Sharpness,
    Blur,
    Average,
    Smooth,
    Sharpen,
    Shadow,
    Canny,
    Edges,
    EdgeX,
    EdgeY,
    Gaussian,
    Laplacian,
    Laplacian8,
    Emboss,
    Outline,
    Grayscale,
    RGBCustom(Kernel),
    LUMCustom(Kernel),
    Unknown,
}

impl From<&str> for FilterType {
    fn from(s: &str) -> Self {
        match s {
            "copy" => FilterType::Copy,
            "median" => FilterType::Median,
            "erode" => FilterType::Erode,
            "dilate" => FilterType::Dilate,
            "sharpness" => FilterType::Sharpness,
            "blur" => FilterType::Blur,
            "average" => FilterType::Average,
            "smooth" => FilterType::Smooth,
            "sharpen" => FilterType::Sharpen,
            "shadow" => FilterType::Shadow,
            "canny" => FilterType::Canny,
            "edge" => FilterType::Edges,
            "edgeX" => FilterType::EdgeX,
            "edgeY" => FilterType::EdgeY,
            "gaussian" => FilterType::Gaussian,
            "laplacian" => FilterType::Laplacian,
            "laplacian8" => FilterType::Laplacian8,
            "emboss" => FilterType::Emboss,
            "outline" => FilterType::Outline,
            "grayscale" => FilterType::Grayscale,
            "custom" => FilterType::RGBCustom(Kernel::empty()),
            "lumCustom" => FilterType::LUMCustom(Kernel::empty()),
            _ => FilterType::Unknown,
        }
    }
}

#[derive(Clone)]
pub struct Kernel {
    pub width: usize,
    pub height: usize,
    pub matrix: Vec<Vec<f32>>,
    pub already_nomarized: bool,
}

impl Kernel {
    pub fn new(matrix: [[f32; 3]; 3]) -> Self {
        let matrix = vec![matrix[0].to_vec(), matrix[1].to_vec(), matrix[2].to_vec()];
        Self::from(3, 3, matrix, false)
    }

    pub fn new_already_normalized(matrix: [[f32; 3]; 3]) -> Self {
        let matrix = vec![matrix[0].to_vec(), matrix[1].to_vec(), matrix[2].to_vec()];
        Self::from(3, 3, matrix, true)
    }

    pub fn empty() -> Self {
        let matrix = vec![
            vec![0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0],
            vec![0.0, 0.0, 0.0],
        ];
        Self::from(3, 3, matrix, false)
    }
    pub fn from(width: usize, height: usize, matrix: Vec<Vec<f32>>,already_nomarized: bool) -> Self {
        Self {
            width,
            height,
            matrix,
            already_nomarized,
        }
    }

    pub fn gaussian_kernel(size: usize, sigma: f32) -> Result<Self,Error> {
        if size % 2 == 0 {
            return Err(Error::new(ErrorKind::Other,"not support even kernel size"))
        }
        let mut matrix = vec![vec![0.0;size];size];
        let k = (size /2) as i32;
        let mut sum = 0.0;
        for y in -k..=k {
            for x in -k..=k {
                let v = (-((x*x +y*y) as f32) / (2.0 * sigma * sigma)).exp();
                matrix[(y + k) as usize][(x + k ) as usize] = v;
                sum += v;
            }
        }
        for y in 0..size {
            for x in 0..size {
               matrix[y][x] /= sum;
            }
        }
        Ok(Kernel::from(size, size, matrix, true))
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

pub fn lum_filter(src: &dyn Screen, dest: &mut dyn Screen, kernel: &Kernel) -> Result<(), Error> {
    if dest.width() == 0 || dest.height() == 0 {
        dest.reinit(src.width(), src.height());
    }
    let dest_height = dest.height() as usize;
    let dest_width = dest.width() as usize;

    let matrix = &kernel.matrix;
    let u0 = (kernel.height / 2) as i32;
    let v0 = (kernel.width / 2) as i32;
    let coeff = if kernel.already_nomarized {
        1.0
    } else {
        let mut coeff = 0.0;
        for u in 0..kernel.height {
            for v in 0..kernel.width {
                coeff += matrix[v][u];
            }
        }
        coeff
    };
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
    Ok(())
}

pub fn edge_filter(src: &dyn Screen, dest: &mut dyn Screen, kernel: &Kernel) -> Result<(), Error> {
    if dest.width() == 0 || dest.height() == 0 {
        dest.reinit(src.width(), src.height());
    }
    let dest_height = dest.height() as usize;
    let dest_width = dest.width() as usize;

    let matrix = &kernel.matrix;
    let u0 = (kernel.height / 2) as i32;
    let v0 = (kernel.width / 2) as i32;
    let coeff = if kernel.already_nomarized {
        1.0
    } else {
        let mut coeff = 0.0;
        for u in 0..kernel.height {
            for v in 0..kernel.width {
                coeff += matrix[v][u];
            }
        }
        coeff
    };
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
            let l = ((l / coeff).round() as i16 * 255).clamp(0, 255) as u8;

            dest_buffer[offset + x * 4] = l;
            dest_buffer[offset + x * 4 + 1] = l;
            dest_buffer[offset + x * 4 + 2] = l;
            dest_buffer[offset + x * 4 + 3] = a;
        }
    }
    Ok(())
}

pub fn grayscale(src: &dyn Screen, dest: &mut dyn Screen) -> Result<(), Error> {
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
    Ok(())
}

pub fn rgb_filter(src: &dyn Screen, dest: &mut dyn Screen, kernel: &Kernel) -> Result<(), Error> {
    if dest.width() == 0 || dest.height() == 0 {
        dest.reinit(src.width(), src.height());
    }
    let dest_height = dest.height() as usize;
    let dest_width = dest.width() as usize;

    let matrix = &kernel.matrix;
    let u0 = (kernel.height / 2) as i32;
    let v0 = (kernel.width / 2) as i32;
    let coeff = if kernel.already_nomarized {
        1.0
    } else {
        let mut coeff = 0.0;
        for u in 0..kernel.height {
            for v in 0..kernel.width {
                coeff += matrix[v][u];
            }
        }
        coeff
    };
    let src_buffer = src.buffer();
    let dest_buffer = dest.buffer_mut();
    let src_width = src.width() as usize;
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
                    * src_width
                    * 4;
                for v in 0..kernel.width {
                    let vv = (x as i32 + v as i32 - v0).clamp(0, src_width as i32 - 1) as usize * 4;
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
    Ok(())
}

pub fn sharpness(src: &dyn Screen, dest: &mut dyn Screen) -> Result<(), Error> {
    let matrix = [[-1.0, -1.0, -1.0], [-1.0, 10.0, -1.0], [-1.0, -1.0, -1.0]];
    lum_filter(src, dest, &Kernel::new(matrix))
}

pub fn blur(src: &dyn Screen, dest: &mut dyn Screen) -> Result<(), Error> {
    let matrix = [[1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0]];
    lum_filter(src, dest, &Kernel::new(matrix))
}

pub fn copy_to(src: &dyn Screen, dest: &mut dyn Screen) -> Result<(), Error> {
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
            dest_buffer[offset + x * 4] = src_buffer[offset + x * 4];
            dest_buffer[offset + x * 4 + 1] = src_buffer[offset + x * 4 + 1];
            dest_buffer[offset + x * 4 + 2] = src_buffer[offset + x * 4 + 2];
            dest_buffer[offset + x * 4 + 3] = src_buffer[offset + x * 4 + 3];
        }
    }
    Ok(())
}

pub fn combine(src1: &dyn Screen, src2: &dyn Screen, dest: &mut dyn Screen) -> Result<(), Error> {
    // √(src1^2 + src2^2) arctan (src2/src1)
    if dest.width() == 0 || dest.height() == 0 {
        dest.reinit(src1.width(), src1.height());
    }
    let dest_height = dest.height() as usize;
    let dest_width = dest.width() as usize;
    let src1_buffer = src1.buffer();
    let src2_buffer = src2.buffer();
    let dest_buffer = dest.buffer_mut();
    for y in 0..src1.height() as usize {
        let offset = y * src1.width() as usize * 4;
        if y >= dest_height {
            break;
        }
        for x in 0..src1.width() as usize {
            if x >= dest_width {
                break;
            }
            let r1 = src1_buffer[offset + x * 4] as f32;
            let g1 = src1_buffer[offset + x * 4 + 1] as f32;
            let b1 = src1_buffer[offset + x * 4 + 2] as f32;

            let r2 = src2_buffer[offset + x * 4] as f32;
            let g2 = src2_buffer[offset + x * 4 + 1] as f32;
            let b2 = src2_buffer[offset + x * 4 + 2] as f32;

            let r = ((r1 * r1 + r2 * r2).sqrt().round() as i32).clamp(0, 255) as u8;
            let g = ((g1 * g1 + g2 * g2).sqrt().round() as i32).clamp(0, 255) as u8;
            let b = ((b1 * b1 + b2 * b2).sqrt().round() as i32).clamp(0, 255) as u8;
            let a = src1_buffer[offset + x * 4 + 3];

            dest_buffer[offset + x * 4] = r;
            dest_buffer[offset + x * 4 + 1] = g;
            dest_buffer[offset + x * 4 + 2] = b;
            dest_buffer[offset + x * 4 + 3] = a;
        }
    }
    Ok(())
}

pub fn ranking_with_size(
    src: &dyn Screen,
    dest: &mut dyn Screen,
    rank: usize,
    size: usize,
) -> Result<(), Error> {
    if size <= 2 || size % 2 == 0 {
        return Err(Error::new(ErrorKind::Other, "not support kernel size"));
    }
    let size = size as i32;
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
            let mut l = vec![(0.0, 0, 0, 0); (size * size) as usize];
            for u in 0..size {
                let uu = (y as i32 + u - 1).clamp(0, src.height() as i32 - 1) as usize
                    * src.width() as usize
                    * 4;
                for v in 0..size {
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
    Ok(())
}

pub fn ranking(src: &dyn Screen, dest: &mut dyn Screen, rank: usize) -> Result<(), Error> {
    let size: i32 = 3;
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
            let mut l = vec![(0.0, 0, 0, 0); (size * size) as usize];
            for u in 0..size {
                let uu = (y as i32 + u - 1).clamp(0, src.height() as i32 - 1) as usize
                    * src.width() as usize
                    * 4;
                for v in 0..size {
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
    Ok(())
}

#[derive(Clone)]
struct Grad {
    pub mag: f32,
    pub dir: u8, // 0,1,2,3 → 0°,45°,90°,135°
}

fn calc_grad(gx: Vec<f32>, gy: Vec<f32>, w: usize, h: usize) -> Vec<Grad> {
    let mut out = vec![Grad { mag: 0.0, dir: 0 }; w as usize * h as usize];


    for y in 0..h {
        let stride = (y * w) as usize;

        for x in 0..w as usize {
            let offset = stride + x;
            let g_x = gx[offset] as f32;
            let g_y = gy[offset] as f32;

            let mag = (g_x * g_x + g_y * g_y).sqrt();

            let angle = g_y.atan2(g_x) * 180.0 / std::f32::consts::PI;

            let dir = if (-22.5..22.5).contains(&angle) || angle >= 157.5 || angle <= -157.5 {
                0 // →
            } else if (22.5..67.5).contains(&angle) || (-157.5..-112.5).contains(&angle) {
                1 // ／
            } else if (67.5..112.5).contains(&angle) || (-112.5..-67.5).contains(&angle) {
                2 // ↑
            } else {
                3 // ＼
            };

            out[y as usize * w as usize + x as usize] = Grad { mag, dir };
        }
    }

    out
}

fn double_threshold(src: &[f32], low: f32, high: f32) -> Vec<u8> {
    let mut out = vec![0u8; src.len()];

    for i in 0..src.len() {
        out[i] = if src[i] >= high {
            2 // strong
        } else if src[i] >= low {
            1 // weak
        } else {
            0
        };
    }
    out
}

fn non_max_suppression(grad: &[Grad], w: usize, h: usize) -> Vec<f32> {
    let mut out = vec![0.0; w * h];

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let i = y * w + x;
            let g = &grad[i];

            let (a, b) = match g.dir {
                0 => (grad[i - 1].mag, grad[i + 1].mag),
                1 => (grad[i - w - 1].mag, grad[i + w + 1].mag),
                2 => (grad[i - w].mag, grad[i + w].mag),
                _ => (grad[i - w + 1].mag, grad[i + w - 1].mag),
            };

            if g.mag >= a && g.mag >= b {
                out[i] = g.mag;
            }
        }
    }

    out
}

fn hysteresis(src: &mut [u8], w: usize, h: usize) {
    let mut stack = Vec::new();

    for i in 0..src.len() {
        if src[i] == 2 {
            stack.push(i);

            while let Some(idx) = stack.pop() {
                let x = idx % w;
                let y = idx / w;

                for ny in y.saturating_sub(1)..=(y + 1).min(h - 1) {
                    for nx in x.saturating_sub(1)..=(x + 1).min(w - 1) {
                        let ni = ny * w + nx;
                        if src[ni] == 1 {
                            src[ni] = 2;
                            stack.push(ni);
                        }
                    }
                }
            }
        }
    }

    // weakを消す
    for v in src.iter_mut() {
        if *v != 2 {
            *v = 0;
        }
    }
}

fn edge_filter_f32(
    src: &dyn Screen,
) -> (Vec<f32>, Vec<f32>) {
    let w = src.width() as usize;
    let h = src.height() as usize;

    let buf = src.buffer();

    let mut gx = vec![0.0f32; w * h];
    let mut gy = vec![0.0f32; w * h];

    // Sobel kernel
    let kx = [
        [ 1.0,  0.0, -1.0],
        [ 2.0,  0.0, -2.0],
        [ 1.0,  0.0, -1.0],
    ];
    let ky = [
        [ 1.0,  2.0,  1.0],
        [ 0.0,  0.0,  0.0],
        [-1.0, -2.0, -1.0],
    ];

    for y in 1..h-1 {
        for x in 1..w-1 {
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;

            for ky_i in 0..3 {
                for kx_i in 0..3 {
                    let px = x + kx_i - 1;
                    let py = y + ky_i - 1;

                    let offset = (py * w + px) * 4;

                    // グレースケール化（ここ重要）
                    let r = buf[offset] as f32;
                    let g = buf[offset + 1] as f32;
                    let b = buf[offset + 2] as f32;
                    let gray = 0.299*r + 0.587*g + 0.114*b;

                    sum_x += gray * kx[ky_i][kx_i];
                    sum_y += gray * ky[ky_i][kx_i];
                }
            }

            let i = y * w + x;
            gx[i] = sum_x;
            gy[i] = sum_y;
        }
    }

    (gx, gy)
}

pub fn canny(src: &dyn Screen, dest: &mut dyn Screen) -> Result<(), Error> {
    let src_width = src.width() as usize;
    let src_height = src.height() as usize;
    // ガウシアン
    let mut tmp_gaussian = Layer::tmp(src.width(), src.height());
    lum_filter(
        src,
        &mut tmp_gaussian,
        &Kernel::gaussian_kernel(3, 1.0).unwrap()
    )?;
    let (gx, gy) = edge_filter_f32(src);
    let grad = calc_grad(gx, gy, src_width, src_height);
    let nms = non_max_suppression(&grad, src_width, src_height);
    let mut th = double_threshold(&nms, 20.0, 50.0);
    hysteresis(&mut th, src_width, src_height);

    // 書き戻し
    let buffer = dest.buffer_mut();
    for y in 0..src_height {
        let stride = y * src_width * 4;
        for x in 0..src_width {
            let offset = stride + x * 4;
            let v = if th[y * src_width + x] == 2 { 255 } else { 0 };
            buffer[offset] = v; // R
            buffer[offset + 1] = v; // G
            buffer[offset + 2] = v; //B
            buffer[offset + 3] = 255; // A
        }
    }
    Ok(())
}

pub fn filter(src: &dyn Screen, dest: &mut dyn Screen, filter_name: &str) -> Result<(), Error> {
    let filter_type = FilterType::from(filter_name);
    _filter(src, dest, filter_type,None)
}

pub fn filter_with_option(src: &dyn Screen, dest: &mut dyn Screen, filter_name: &str, options:Option<HashMap<&str, f32>>) -> Result<(), Error> {
    let filter_type = FilterType::from(filter_name);
    _filter(src, dest, filter_type, options)
}

fn _filter(src: &dyn Screen, dest: &mut dyn Screen, filter_type: FilterType, options:Option<HashMap<&str, f32>>) -> Result<(), Error> {
    let result = match filter_type {
        FilterType::Copy => copy_to(src, dest),
        FilterType::Median => ranking(src, dest, 5),
        FilterType::Erode => ranking(src, dest, 0),
        FilterType::Dilate => ranking(src, dest, 8),
        FilterType::Sharpness => {
            let matrix = [[-1.0, -1.0, -1.0], [-1.0, 10.0, -1.0], [-1.0, -1.0, -1.0]];
            lum_filter(src, dest, &Kernel::new(matrix))
        }
        FilterType::Blur => {
            let matrix = [[1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0]];
            lum_filter(src, dest, &Kernel::new(matrix))
        }
        FilterType::Average => {
            let matrix = [[1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0]];
            rgb_filter(src, dest, &Kernel::new(matrix))
        }
        FilterType::Smooth => {
            let matrix = [[1.0, 1.0, 1.0], [1.0, 4.0, 1.0], [1.0, 1.0, 1.0]];
            lum_filter(src, dest, &Kernel::new(matrix))
        }
        FilterType::Sharpen => {
            let matrix = [[-1.0, -1.0, -1.0], [-1.0, 12.0, -1.0], [-1.0, -1.0, -1.0]];
            lum_filter(src, dest, &Kernel::new(matrix))
        }
        FilterType::Shadow => {
            let matrix = [[1.0, 2.0, 1.0], [0.0, 1.0, 0.0], [-1.0, -2.0, -1.0]];
            lum_filter(src, dest, &Kernel::new(matrix))
        }
        FilterType::Canny => canny(src, dest),
        FilterType::Edges => {
            let matrix_a = [[1.0, 2.0, 1.0], [0.0, 0.0, 0.0], [-1.0, -2.0, -1.0]];
            let matrix_b = [[1.0, 0.0, -1.0], [2.0, 0.0, -2.0], [1.0, 0.0, -1.0]];
            // ガウシアン
            let mut tmp_gaussian = Layer::tmp(src.width(), src.height());
            lum_filter(
                src,
                &mut tmp_gaussian,
                &Kernel::gaussian_kernel(3, 1.0).unwrap()
            )?;
            let mut tmp_a = Layer::tmp(src.width(), src.height());
            let mut tmp_b = Layer::tmp(src.width(), src.height());
            // SobelX
            edge_filter(
                &tmp_gaussian as &dyn Screen,
                &mut tmp_a,
                &Kernel::new_already_normalized(matrix_a),
            )?;
            // SobelY
            edge_filter(
                &tmp_gaussian as &dyn Screen,
                &mut tmp_b,
                &Kernel::new_already_normalized(matrix_b),
            )?;
            // combine
            combine(&tmp_a as &dyn Screen, &tmp_b as &dyn Screen, dest)
        }
        FilterType::EdgeX => {
            // Sobel X
            let matrix = [[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]];
            edge_filter(src, dest, &Kernel::new_already_normalized(matrix))
        }
        FilterType::EdgeY => {
            // Sobel Y
            let matrix = [[-1.0, -2.0, -1.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]];
            edge_filter(src, dest, &Kernel::new_already_normalized(matrix))
        }
        FilterType::Gaussian => {
            let kernel = if let Some(options) = options {
                let sigma: f32 = if let Some(sigma) = options.get("sigma") {
                    sigma.clone()
                } else {
                    1.0
                };
                let size = if let Some(size) = options.get("size") {
                    size.clone() as usize
                } else {
                    3
                };
                Kernel::gaussian_kernel(size, sigma)?
            } else {
               let matrix = [[1.0, 2.0, 1.0], [2.0, 4.0, 2.0], [1.0, 2.0, 1.0]];
               Kernel::new(matrix)
            };
            lum_filter(src, dest, &kernel)
        }
        FilterType::Laplacian => {
            let matrix = [[0.0, 1.0, 0.0], [1.0, -4.0, 1.0], [0.0, 1.0, 0.0]];
            edge_filter(src, dest, &Kernel::new_already_normalized(matrix))
        }
        FilterType::Laplacian8 => {
            let matrix = [[1.0, 1.0, 1.0], [1.0, -8.0, 1.0], [1.0, 1.0, 1.0]];
            edge_filter(src, dest, &Kernel::new_already_normalized(matrix))
        }
        FilterType::Emboss => {
            let matrix = [[-2.0, -1.0, 0.0], [-1.0, 1.0, 1.0], [0.0, 1.0, 2.0]];
            lum_filter(src, dest, &Kernel::new_already_normalized(matrix))
        }
        FilterType::Outline => {
            let mut tmp_gaussian = Layer::tmp(src.width(), src.height());
            lum_filter(
                src,
                &mut tmp_gaussian,
                &Kernel::new([[1.0, 2.0, 1.0], [2.0, 4.0, 2.0], [1.0, 2.0, 1.0]]),
            )?;
            let matrix = [[-1.0, -1.0, -1.0], [-1.0, 8.0, -1.0], [-1.0, -1.0, -1.0]];
            edge_filter(&tmp_gaussian, dest, &Kernel::new_already_normalized(matrix))
        }
        FilterType::Grayscale => {
            to_grayscale(src, dest, 0);
            Ok(())
        }
        FilterType::RGBCustom(kernel) => rgb_filter(src, dest, &kernel),
        FilterType::LUMCustom(kernel) => lum_filter(src, dest, &kernel),
        FilterType::Unknown => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unknown filter",
        )),
    };
    result
}
/*

平均 (Box blur)	[[1,1,1],[1,1,1],[1,1,1]] / 9	均等平滑化
Gaussian (σ≈1)	[[1,2,1],[2,4,2],[1,2,1]] / 16	ノイズ除去＋自然なぼかし
Sharpen	[[0,-1,0],[-1,5,-1],[0,-1,0]]	輪郭強調
強Sharpen	[[-1,-1,-1],[-1,9,-1],[-1,-1,-1]]	より強めのシャープ
Edge (Sobel X)	[[-1,0,1],[-2,0,2],[-1,0,1]]	垂直エッジ検出
Edge (Sobel Y)	[[-1,-2,-1],[0,0,0],[1,2,1]]	水平エッジ検出
Edge (Laplacian)	[[0,1,0],[1,-4,1],[0,1,0]]	方向性を持たない輪郭検出
Emboss	[[-2,-1,0],[-1,1,1],[0,1,2]]	浮き出し効果
Outline	[[-1,-1,-1],[-1,8,-1],[-1,-1,-1]]	輪郭のみ抽出
 */

pub fn filters(
    src: &dyn Screen,
    dest: &mut dyn Screen,
    filter_names: Vec<String>,
) -> Result<(), Error> {
    let mut intermediate_src = Layer::tmp(src.width(), src.height());
    let mut intermediate_dest = Layer::tmp(src.width(), src.height());
    copy_to(src, &mut intermediate_src)?;

    for (i, filter_name) in filter_names.iter().enumerate() {
        if i % 2 == 0 {
            filter(&intermediate_src, &mut intermediate_dest, filter_name)?;
        } else {
            filter(&intermediate_dest, &mut intermediate_src, filter_name)?;
        }
    }

    if filter_names.len() % 2 == 0 {
        copy_to(&intermediate_src, dest)?;
    } else {
        copy_to(&intermediate_dest, dest)?;
    }

    Ok(())
}

#[test]
fn gaussian_test() {
    let kernel = Kernel::gaussian_kernel(3, 1.0).unwrap();

    let mut sum = 0.0;
    for row in &kernel.matrix {
        for v in row {
            sum += *v;
        }
    }

    println!("sum = {}", sum);

    // 中心
    println!("center = {}", kernel.matrix[1][1]);

    // 角
    println!("corner = {}", kernel.matrix[0][0]);
}