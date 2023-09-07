//! Affine trasration
//!
//! usage
//!
//! ```
//! paint::paint::affine;
//!
//! let affine = Affine::new();
//! let affine.translation(20,20); //  x +20 ,y +20
//! let affine.invertXY(); // reverse up-down and right-left
//! let affine.scale(3.0,3.0); // image scale x 3.0
//! affineconversion(input_screen,output_screen,InterpolationAlgorithm::Bilinear);  // conversion with Bilinear algorithm
//! ```
//!
/*
 * affine.rs  Mith@mmk (C) 2022
 * create 2022/03/13
 */

use crate::canvas::*;
use crate::image::ImageAlign;
use core::cmp::max;
use core::cmp::min;
use core::f32::consts::PI;
use std::cmp::Ordering;

#[derive(Clone)]
pub enum InterpolationAlgorithm {
    NearestNeighber,
    Bilinear,
    Bicubic,
    BicubicAlpha(Option<f32>),
    Lanzcos3,
    Lanzcos(Option<usize>),
}

pub struct Affine {
    affine: [[f32; 3]; 3], // 3*3
}

impl Affine {
    pub fn new() -> Self {
        let affine = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        Self { affine }
    }

    fn matrix(self: &mut Self, f: &[[f32; 3]; 3]) {
        let affin = self.affine;
        let mut result: [[f32; 3]; 3] = [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        for i in 0..3 {
            for j in 0..3 {
                result[i][j] =
                    affin[i][0] * f[0][j] + affin[i][1] * f[1][j] + affin[i][2] * f[2][j];
            }
        }
        self.affine = result;
    }

    /// transration image (x,y) position
    pub fn translation(self: &mut Self, x: f32, y: f32) {
        self.matrix(&[[1.0, 0.0, x], [0.0, 1.0, y], [0.0, 0.0, 1.0]]);
    }

    /// reverse image right-left
    pub fn invert_x(self: &mut Self) {
        self.matrix(&[[-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    }

    /// reverse image up-down
    pub fn invert_y(self: &mut Self) {
        self.matrix(&[[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]]);
    }

    /// reverse image right-left and up-down
    pub fn invert_xy(self: &mut Self) {
        self.matrix(&[[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]]);
    }

    /// change scale x X x Y
    pub fn scale(self: &mut Self, x: f32, y: f32) {
        self.matrix(&[[x, 0.0, 0.0], [0.0, y, 0.0], [0.0, 0.0, 1.0]]);
    }

    /// rotate by dgree
    pub fn rotate_by_dgree(self: &mut Self, theta: f32) {
        let theta = PI * theta / 180.0;
        self.rotate(theta);
    }

    /// rotate by radian
    pub fn rotate(self: &mut Self, theta: f32) {
        let c = theta.cos();
        let s = theta.sin();
        self.matrix(&[[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]);
    }

    /// skew
    pub fn skew_x(self: &mut Self, theta: f32) {
        let s = theta.tan();
        self.matrix(&[[1.0, 0.0, 0.0], [s, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    }

    pub fn skew_y(self: &mut Self, theta: f32) {
        let s = theta.tan();
        self.matrix(&[[1.0, s, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    }

    pub fn skew_x_by_degree(self: &mut Self, theta: f32) {
        let theta = PI * theta / 180.0;
        self.skew_x(theta);
    }

    pub fn skew_y_by_degree(self: &mut Self, theta: f32) {
        let theta = PI * theta / 180.0;
        self.skew_y(theta);
    }

    /// shear
    pub fn shear(self: &mut Self, x: f32, y: f32) {
        self.matrix(&[[1.0, y, 0.0], [x, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    }

    #[inline]
    fn sinc(x: f32) -> f32 {
        (x * PI).sin() / (x * PI)
    }

    /// conversion no implement interpolations
    pub fn _conversion(self: &mut Self, input_screen: &dyn Screen, output_screen: &mut dyn Screen) {
        let min_x = 0;
        let max_x = output_screen.width() as i32;
        let min_y = 0;
        let max_y = output_screen.height() as i32;

        let input_buffer = input_screen.buffer();
        let output_buffer: &mut [u8] = output_screen.buffer_mut();

        let ox = max_x / 2;
        let oy = max_y / 2;

        /*
         * |X|   |a00 a01 a02||x|         X = a00 * x + a01 * y + a02
         * |Y| = |a10 a11 a12||y|         Y = a10 * x + a11 * y + a12
         * |Z|   |a20 a21 a22||1|         _  do not use
         */

        let x0 = self.affine[0][0];
        let x1 = self.affine[0][1];
        let x2 = self.affine[0][2];
        let y0 = self.affine[1][0];
        let y1 = self.affine[1][1];
        let y2 = self.affine[1][2];

        for y in 0..input_screen.height() as usize {
            let input_base_line = input_screen.width() as usize * 4 * y;
            for x in 0..input_screen.width() as usize {
                let offset = input_base_line + x * 4;
                let x_ = x as i32 - ox;
                let y_ = y as i32 - oy;

                let xx_ = x_ as f32 * x0 + y_ as f32 * x1 + x2;
                let xx = (xx_ + ox as f32).round() as i32;
                if xx < min_x || xx >= max_x {
                    continue;
                } // Out of bound

                let yy_ = x_ as f32 * y0 + y_ as f32 * y1 + y2;
                let yy = (yy_ + oy as f32).round() as i32;

                if yy < min_y || yy >= max_y {
                    continue;
                } // Out of bound

                let output_offset = ((yy * max_x + xx) * 4) as usize;
                output_buffer[output_offset] = input_buffer[offset];
                output_buffer[output_offset + 1] = input_buffer[offset + 1];
                output_buffer[output_offset + 2] = input_buffer[offset + 2];
                output_buffer[output_offset + 3] = input_buffer[offset + 3];
            }
        }
    }

    /// conversion implement interpolations
    pub fn conversion(
        self: &mut Self,
        input_screen: &dyn Screen,
        output_screen: &mut dyn Screen,
        algorithm: InterpolationAlgorithm,
    ) {
        let start_x = 0 as f32;
        let start_y = 0 as f32;

        let out_start_x = 0;
        let out_start_y = 0;
        let out_width = output_screen.width() as i32;
        let out_height = output_screen.height() as i32;

        self.conversion_with_area(
            input_screen,
            output_screen,
            start_x as f32,
            start_y,
            input_screen.width() as f32,
            input_screen.height() as f32,
            out_start_x,
            out_start_y,
            out_width as i32,
            out_height as i32,
            algorithm,
        );
    }

    pub fn conversion_with_area_center(
        self: &mut Self,
        input_screen: &dyn Screen,
        output_screen: &mut dyn Screen,
        start_x: f32,
        start_y: f32,
        width: f32,
        height: f32,
        out_start_x: i32,
        out_start_y: i32,
        out_width: i32,
        out_height: i32,
        ox: f32,
        oy: f32,
        algorithm: InterpolationAlgorithm,
    ) {
        let end_x = width - start_x - 1.0;
        let end_y = height - start_y - 1.0;
        let out_end_x = out_width - out_start_x - 1;
        let out_end_y = out_height - out_start_y - 1;

        let output_screen_width = &output_screen.width();
        let output_buffer = output_screen.buffer_mut();
        let input_buffer = input_screen.buffer();

        let mut alpha = -0.5; // -0.5 - -1.0
        let mut lanzcos_n = 3;
        match algorithm {
            InterpolationAlgorithm::BicubicAlpha(a) => {
                if a.is_some() {
                    alpha = a.unwrap()
                }
            }
            InterpolationAlgorithm::Lanzcos(n) => {
                if n.is_some() {
                    lanzcos_n = n.unwrap();
                }
            }

            _ => {}
        }

        /*
         * |X|   |a00 a01 a02||x|         X = a00 * x + a01 * y + a02
         * |Y| = |a10 a11 a12||y|         Y = a10 * x + a11 * y + a12
         * |Z|   |a20 a21 a22||1|         _  do not use
         */

        let x0 = self.affine[0][0];
        let x1 = self.affine[0][1];
        let x2 = self.affine[0][2];
        let y0 = self.affine[1][0];
        let y1 = self.affine[1][1];
        let y2 = self.affine[1][2];

        // calc rectangle 4x affine trans

        let mut xy = [(0_i32, 0_i32); 4];
        let x = start_x - ox;
        let y = start_y - oy;
        xy[0] = (
            (x * x0 + y * x1 + x2 + ox) as i32,
            (x * y0 + y * y1 + y2 + oy) as i32,
        );
        let x = end_x - ox;
        let y = start_y - oy;
        xy[1] = (
            (x * x0 + y * x1 + x2 + ox) as i32,
            (x * y0 + y * y1 + y2 + oy) as i32,
        );
        let x = start_x - ox;
        let y = end_y - oy;
        xy[2] = (
            (x * x0 + y * x1 + x2 + ox) as i32,
            (x * y0 + y * y1 + y2 + oy) as i32,
        );
        let x = end_x - ox;
        let y = end_y - oy;
        xy[3] = (
            (x * x0 + y * x1 + x2 + ox) as i32,
            (x * y0 + y * y1 + y2 + oy) as i32,
        );

        xy.sort_by(|a, b| {
            if a.1 == b.1 {
                if a.0 < b.0 {
                    Ordering::Less
                } else if a.0 > b.0 {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            } else if a.1 < b.1 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });

        // pre-calc inverse affine transformation
        // x =  y1 * X - x0 * Y  +  x1 * y2 - y1 * x2
        // y = -y0 * X + x1 * X  +  y0 * x2 - x0 * y2

        let t = x0 * y1 - x1 * y0;
        let ix0 = y1;
        let ix1 = -x1;
        let ix2 = x1 * y2 - y1 * x2;
        let iy0 = -y0;
        let iy1 = x0;
        let iy2 = y0 * x2 - x0 * y2;

        // stage 0 y0..y1 (x0,y0) - (x1,y1) &  (x0,y0) - (x2,y2)
        // stage 1 y1..y2 (x0,y0) - (x2,x2) &  (x1,y1) - (x3,y3)
        // stage 2 y2..y3 (x1,y1) - (x3,y3) &  (x2,y2) - (x3,y3)
        for stage in 0..3 {
            let mut sy;
            let mut ey;
            let xy0;
            let xy1;
            let xy2;
            let xy3;
            match stage {
                0 => {
                    sy = xy[0].1;
                    ey = xy[1].1;
                    xy0 = xy[0];
                    xy1 = xy[1];
                    xy2 = xy[0];
                    xy3 = xy[2];
                }
                1 => {
                    sy = xy[1].1;
                    ey = xy[2].1;
                    xy0 = xy[1];
                    xy1 = xy[3];
                    xy2 = xy[0];
                    xy3 = xy[2];
                }
                2 => {
                    sy = xy[2].1;
                    ey = xy[3].1;
                    xy0 = xy[1];
                    xy1 = xy[3];
                    xy2 = xy[2];
                    xy3 = xy[3];
                }
                _ => {
                    sy = 0;
                    ey = 0;
                    xy0 = (0, 0);
                    xy1 = (0, 0);
                    xy2 = (0, 0);
                    xy3 = (0, 0);
                }
            }

            if sy < out_start_y {
                sy = out_start_y;
            }
            if ey > out_end_y {
                ey = out_end_y;
            }

            let d0 = if xy0.1 != xy1.1 {
                (xy0.0 as f32 - xy1.0 as f32) / (xy0.1 as f32 - xy1.1 as f32)
            } else {
                0.0
            };
            let d1 = if xy2.1 != xy3.1 {
                (xy2.0 as f32 - xy3.0 as f32) / (xy2.1 as f32 - xy3.1 as f32)
            } else {
                0.0
            };

            for y in sy..ey {
                // (x0,y0) - (x1,y1) &  (x2,y2) - (x3,y3)
                let (mut sx, mut ex) = if xy0.1 == xy1.1 {
                    (min(xy0.0, xy1.0), max(xy0.0, xy1.0) + 1)
                } else {
                    if xy2.1 == xy3.1 {
                        (min(xy2.0, xy3.0), max(xy2.0, xy3.0) + 1)
                    } else {
                        let x0 = (d0 * (y - xy0.1) as f32) as i32 + xy0.0 as i32;
                        let x1 = (d1 * (y - xy2.1) as f32) as i32 + xy2.0 as i32;
                        (min(x0, x1), max(x0, x1) + 1)
                    }
                };
                let output_base_line = *output_screen_width as usize * 4 * y as usize;
                if sx < out_start_x {
                    sx = out_start_x;
                }
                if ex > out_end_x {
                    ex = out_end_x;
                }

                for x in sx..ex {
                    // inverse affine transformation from output image integer position
                    let xx = (ix0 * (x as f32 - ox) + ix1 * (y as f32 - oy) + ix2) / t + ox;
                    let yy = (iy0 * (x as f32 - ox) + iy1 * (y as f32 - oy) + iy2) / t + oy;

                    if xx < start_x || xx >= end_x || yy < start_y || yy >= end_y {
                        continue;
                    }
                    let output_offset = output_base_line + x as usize * 4;
                    let input_offset =
                        (yy as usize * input_screen.width() as usize + xx as usize) * 4;
                    match algorithm {
                        InterpolationAlgorithm::NearestNeighber => {
                            output_buffer[output_offset] = input_buffer[input_offset];
                            output_buffer[output_offset + 1] = input_buffer[input_offset + 1];
                            output_buffer[output_offset + 2] = input_buffer[input_offset + 2];
                            output_buffer[output_offset + 3] = input_buffer[input_offset + 3];
                        }
                        InterpolationAlgorithm::Bilinear => {
                            let dx = xx - xx.floor();
                            let dy = yy - yy.floor();
                            let xx = xx.floor() as i32;
                            let yy = yy.floor() as i32;

                            let nx = if xx + 1 > end_x as i32 { 0 } else { 4 };
                            let ny = if yy + 1 > end_y as i32 {
                                0
                            } else {
                                input_screen.width() as usize * 4
                            };

                            let r = (input_buffer[input_offset] as f32 * (1.0 - dx) * (1.0 - dy)
                                + input_buffer[input_offset + nx] as f32 * dx * (1.0 - dy)
                                + input_buffer[input_offset + ny] as f32 * (1.0 - dx) * dy
                                + input_buffer[input_offset + nx + ny] as f32 * dx * dy)
                                as i32;
                            let g =
                                (input_buffer[input_offset + 1] as f32 * (1.0 - dx) * (1.0 - dy)
                                    + input_buffer[input_offset + 1 + nx] as f32 * dx * (1.0 - dy)
                                    + input_buffer[input_offset + 1 + ny] as f32 * (1.0 - dx) * dy
                                    + input_buffer[input_offset + 1 + nx + ny] as f32 * dx * dy)
                                    as i32;
                            let b =
                                (input_buffer[input_offset + 2] as f32 * (1.0 - dx) * (1.0 - dy)
                                    + input_buffer[input_offset + 2 + nx] as f32 * dx * (1.0 - dy)
                                    + input_buffer[input_offset + 2 + ny] as f32 * (1.0 - dx) * dy
                                    + input_buffer[input_offset + 2 + nx + ny] as f32 * dx * dy)
                                    as i32;
                            let a =
                                (input_buffer[input_offset + 3] as f32 * (1.0 - dx) * (1.0 - dy)
                                    + input_buffer[input_offset + 3 + nx] as f32 * dx * (1.0 - dy)
                                    + input_buffer[input_offset + 3 + ny] as f32 * (1.0 - dx) * dy
                                    + input_buffer[input_offset + 3 + nx + ny] as f32 * dx * dy)
                                    as i32;

                            output_buffer[output_offset] = r.clamp(0, 255) as u8;
                            output_buffer[output_offset + 1] = g.clamp(0, 255) as u8;
                            output_buffer[output_offset + 2] = b.clamp(0, 255) as u8;
                            output_buffer[output_offset + 3] = a.clamp(0, 255) as u8;
                        }
                        InterpolationAlgorithm::Bicubic
                        | InterpolationAlgorithm::BicubicAlpha(_) => {
                            let dx = xx - xx.floor();
                            let dy = yy - yy.floor();
                            let xx = xx.floor() as i32;
                            let yy = yy.floor() as i32;

                            let mut color = [0.0; 4];

                            for _y in 0..4 {
                                let dy = _y as f32 - dy - 1.0;
                                let dy = dy.abs();
                                let wy = if dy <= 1.0 {
                                    (alpha + 2.0) * dy.powi(3) - (alpha + 3.0) * dy.powi(2) + 1.0
                                } else if dy < 2.0 {
                                    alpha * dy.powi(3) - 5.0 * alpha * dy.powi(2) + 8.0 * alpha * dy
                                        - 4.0 * alpha
                                } else {
                                    0.0
                                };

                                let jy = _y - 1;
                                let baseoffset = if yy + jy < start_y as i32 {
                                    start_y as isize * input_screen.width() as isize * 4
                                } else if yy + jy >= end_y as i32 {
                                    end_y as isize * input_screen.width() as isize * 4
                                } else {
                                    ((yy + jy) as isize * input_screen.width() as isize) * 4
                                };

                                for _x in 0..4 {
                                    let dx = _x as f32 - dx - 1.0;
                                    let dx = dx.abs();
                                    let jx = _x - 1;
                                    let offset = if xx + jx <= start_x as i32 {
                                        baseoffset + start_x as isize * 4
                                    } else if xx + jx >= end_x as i32 {
                                        baseoffset + end_x as isize * 4
                                    } else {
                                        baseoffset + (xx + jx) as isize * 4
                                    };
                                    let wx = if dx <= 1.0 {
                                        (alpha + 2.0) * dx.powi(3) - (alpha + 3.0) * dx.powi(2)
                                            + 1.0
                                    } else if dx < 2.0 {
                                        alpha * dx.powi(3) - 5.0 * alpha * dx.powi(2)
                                            + 8.0 * alpha * dx
                                            - 4.0 * alpha
                                    } else {
                                        0.0
                                    };

                                    let w = wx * wy;

                                    for i in 0..3 {
                                        color[i] += w * input_buffer[offset as usize + i] as f32;
                                    }
                                }
                            }

                            output_buffer[output_offset] = (color[0] as i32).clamp(0, 255) as u8;
                            output_buffer[output_offset + 1] =
                                (color[1] as i32).clamp(0, 255) as u8;
                            output_buffer[output_offset + 2] =
                                (color[2] as i32).clamp(0, 255) as u8;
                            output_buffer[output_offset + 3] = 0xff;
                        }
                        InterpolationAlgorithm::Lanzcos3 | InterpolationAlgorithm::Lanzcos(_) => {
                            let dx = xx - xx.floor();
                            let dy = yy - yy.floor();
                            let xx = xx.floor() as i32;
                            let yy = yy.floor() as i32;
                            let n = lanzcos_n as i32;

                            let mut color = [0.0; 4];

                            for _y in 0..2 * n {
                                let jy = _y - n + 1;
                                let dy = (jy as f32 - dy).abs();
                                let wy = if dy == 0.0 {
                                    1.0
                                } else if dy < n as f32 {
                                    Self::sinc(dy) * Self::sinc(dy / n as f32)
                                } else {
                                    0.0
                                };
                                let baseoffset = if yy + jy < start_y as i32 {
                                    start_y as isize * input_screen.width() as isize * 4
                                } else if yy + jy > end_y as i32 {
                                    end_y as isize * input_screen.width() as isize * 4
                                } else {
                                    ((yy + jy) as isize * input_screen.width() as isize) * 4
                                };

                                for _x in 0..2 * n {
                                    let jx = _x - n + 1;
                                    let dx = (jx as f32 - dx).abs();
                                    let wx = if dx == 0.0 {
                                        1.0
                                    } else if dx < n as f32 {
                                        Self::sinc(dx) * Self::sinc(dx / n as f32)
                                    } else {
                                        0.0
                                    };
                                    let offset = if xx + jx <= start_x as i32 {
                                        baseoffset + start_x as isize * 4
                                    } else if xx + jx >= end_x as i32 {
                                        baseoffset + end_x as isize * 4
                                    } else {
                                        baseoffset + (xx + jx) as isize * 4
                                    };

                                    let w = wx * wy;
                                    for i in 0..3 {
                                        color[i] += w * input_buffer[offset as usize + i] as f32;
                                    }
                                }
                            }

                            output_buffer[output_offset] = (color[0] as i32).clamp(0, 255) as u8;
                            output_buffer[output_offset + 1] =
                                (color[1] as i32).clamp(0, 255) as u8;
                            output_buffer[output_offset + 2] =
                                (color[2] as i32).clamp(0, 255) as u8;
                            output_buffer[output_offset + 3] = 0xff;
                        }
                    }
                }
            }
        }
    }

    pub fn conversion_with_area(
        self: &mut Self,
        input_screen: &dyn Screen,
        output_screen: &mut dyn Screen,
        start_x: f32,
        start_y: f32,
        width: f32,
        height: f32,
        out_start_x: i32,
        out_start_y: i32,
        out_width: i32,
        out_height: i32,
        algorithm: InterpolationAlgorithm,
    ) {
        let ox = (out_width / 2) as f32;
        let oy = (out_height / 2) as f32;

        self.conversion_with_area_center(
            input_screen,
            output_screen,
            start_x as f32,
            start_y,
            width,
            height,
            out_start_x,
            out_start_y,
            out_width as i32,
            out_height as i32,
            ox,
            oy,
            algorithm,
        )
    }

    // only scale < 0.5
    fn pixel_mixing(
        input_screen: &dyn Screen,
        output_screen: &mut dyn Screen,
        scale_x: f32,
        scale_y: f32,
        ox: f32,
        oy: f32,
    ) {
        let ox = ox as u32;
        let oy = oy as u32;
        /*
        *
        *
        * x0,y0  xn,y0   x0,y0 = w(0,0) ,xn,y0 = w(n,0) x0,yn = w(0,n) xn,yn = w(n,n)
        *   +----+       x0,y1 .. x0,yn-1 = w(0,1)  xn,y1 .. xn,yn-1 w(0,n)
        *   |    |       x1,y0 .. xn-1,y0 = w(1,0)  x1,yn .. xn-1,yn w(n,0)
        *   +----+       other w(1,1)
        * x0,yn  xn,yn  w(0,0) = err_x0 * err_y0, w(n,0) = err_xn  * erry0 ...
        *               w(1,1) = (1.0 - Σ w(i,n) - Σ w(n,i) - Σ w(0,i) - Σ w(i,0) - w(0,0) ... - w(n,n)) / (n-2)(n-2)
        *                      = (1.0 - w(1,0) * n - w(1,n) * n - w(0,1) * n - w(n,1) * n
        *                          + w(0,0)  w(0,n) w(n,0) + w(n,m) ) / (n-2)(n-2)
        *                    (n > 2)
        *  d = 1.0 / scale
        *
        *  nx,ny  > 2 → scale < 0.5
        *     nx = d ([d] = d) , n = d +1 p(x) = p([x]) or d + 2 ( p(x) != p([x]) )
        *     ny = d ([d] = d) , n = d +1 p(y) = p([y]) or d + 2 ( p(y) != p([y]) )
        *
        *  P(X,Y) * scale  bx = X * d  by = Y * d , ex = (X + 1) * d, ey = (Y + 1) * d
        *
        *   err_y0 = 1.0 - (bx - [bx]);
        *   err_yn = ex - [ex];

        *   err_y0 = 1.0 - (by - [by]);
        *   err_yn = ey - [ey];
        *
        *  S = err_x0 * err_y0 + err_x0 * err_yn + err_xn * err_y0 + err_xn * err_yn
        *       + err_x0 *(n-2) + err_xn * (n-2) + err_y0 * (n-2) + err_yn * (n-2) + (n-2)(n-2)
        *    = scale * scale
        * ∴
        *   w(i,j) = 1 / S ( 2 <=i <= n - 1 ,2 <= j <= n - 1)
        *   w(0,0) = err_x0 * err_y0 / S
        *   w(n,0) = err_xn * err_y0 / S
        *   w(0,n) = err_x0 * err_yn / S
        *   w(n,n) = err_xn * err_yn / S
        *   w(0,j) = err_x0 / S
        *   w(i,0) = err_y0 / S
        *   w(n,j) = err_xn / S
        *   w(i,n) = err_yn / S
        */

        //        let d = (1.0 / scale).ceil() as usize;
        let d_x = 1.0 / scale_x;
        let d_y = 1.0 / scale_y;
        //        let weight = 1.0 / d as f32 * 1.0 / d as f32;
        let weight = scale_x * scale_y;
        let ey = output_screen.height() - oy;
        let ex = output_screen.width() - ox;
        for y in oy..ey {
            let output_base_line = output_screen.width() as usize * 4 * y as usize;
            let mut dest_offset = output_base_line + ox as usize * 4;

            let by = (y - oy) as f32 / scale_y;
            let base_y = by as usize;
            let end_y = by + d_y;
            let err_y0 = 1.0 - (by - base_y as f32);
            let err_y1 = end_y - end_y.floor();
            let dy = end_y as usize - base_y + 1;

            for x in ox..ex {
                let bx = (x - ox) as f32 / scale_x;
                let base_x = bx as usize;
                let end_x = bx + d_x;
                let err_x0 = 1.0 - (bx - base_x as f32);
                let err_x1 = end_x - end_x.floor();
                let dx = end_x as usize - base_x + 1;

                let mut red = 0.0;
                let mut green = 0.0;
                let mut blue = 0.0;
                let mut alpha = 0.0;
                for yy_ in 0..dy {
                    let mut yy = yy_ + base_y;
                    if yy >= input_screen.height() as usize {
                        yy = input_screen.height() as usize - 1;
                    }
                    let input_base_line = input_screen.width() as usize * 4 * yy as usize;
                    let err_y = if yy_ == 0 {
                        err_y0
                    } else if yy_ == dy - 1 {
                        err_y1
                    } else {
                        1.0
                    };
                    for xx_ in 0..dx {
                        let mut xx = xx_ + base_x;
                        if xx >= input_screen.width() as usize {
                            xx = input_screen.width() as usize - 1;
                        }
                        let err_x = if xx_ == 0 {
                            err_x0
                        } else if xx_ == dx - 1 {
                            err_x1
                        } else {
                            1.0
                        };
                        let src_offset = input_base_line + xx * 4;
                        let weight = weight * err_x * err_y;
                        red += input_screen.buffer()[src_offset] as f32 * weight;
                        green += input_screen.buffer()[src_offset + 1] as f32 * weight;
                        blue += input_screen.buffer()[src_offset + 2] as f32 * weight;
                        alpha += input_screen.buffer()[src_offset + 3] as f32 * weight;
                    }
                }

                output_screen.buffer_mut()[dest_offset] = (red as u32).clamp(0, 255) as u8;
                output_screen.buffer_mut()[dest_offset + 1] = (green as u32).clamp(0, 255) as u8;
                output_screen.buffer_mut()[dest_offset + 2] = (blue as u32).clamp(0, 255) as u8;
                output_screen.buffer_mut()[dest_offset + 3] = (alpha as u32).clamp(0, 255) as u8;
                dest_offset += 4;
            }
        }
    }

    pub fn resize(
        input_screen: &dyn Screen,
        output_screen: &mut dyn Screen,
        scale: f32,
        algorithm: InterpolationAlgorithm,
        align: ImageAlign,
    ) {
        let mut affine = Affine::new();
        let output_width = output_screen.width() as i32;
        let output_height = output_screen.height() as i32;

        let cx = (output_screen.width() as f32 - input_screen.width() as f32 * scale) / 2.0;
        let cy = (output_screen.height() as f32 - input_screen.height() as f32 * scale) / 2.0;

        let (ox, oy);

        match align {
            ImageAlign::Default | ImageAlign::LeftUp => {
                ox = 0.0;
                oy = 0.0;
            }
            ImageAlign::Center => {
                ox = cx;
                oy = cy;
            }
            ImageAlign::Up => {
                ox = cx;
                oy = 0.0;
            }
            ImageAlign::Bottom => {
                ox = cx;
                oy = cy * 2.0;
            }
            ImageAlign::Left => {
                ox = 0.0;
                oy = cy;
            }
            ImageAlign::LeftBottom => {
                ox = 0.0;
                oy = cy * 2.0;
            }
            ImageAlign::Right => {
                ox = cx * 2.0;
                oy = cy;
            }
            ImageAlign::RightUp => {
                ox = 0.0;
                oy = cy;
            }
            ImageAlign::RightBottom => {
                ox = 0.0;
                oy = cy * 2.0;
            }
        }

        if scale < 0.5 {
            Self::pixel_mixing(input_screen, output_screen, scale, scale, ox, oy);
            return;
        }

        affine.translation(ox, oy);
        affine.scale(scale, scale);
        affine.conversion_with_area_center(
            input_screen,
            output_screen,
            0.0,
            0.0,
            input_screen.width() as f32,
            input_screen.height() as f32,
            0,
            0,
            output_width,
            output_height,
            0.0,
            0.0,
            algorithm,
        )
    }
}
