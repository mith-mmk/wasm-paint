/*
 * affine.rs  Mith@mmk (C) 2022
 * create 2022/03/13
 */


use core::cmp::max;
use core::cmp::min;
use std::cmp::Ordering;
use core::f32::consts::PI;
use super::canvas::*;

pub enum InterpolationAlgorithm {
    NearestNeighber,
    Bilinear,
    Bicubic,
    BicubicAlpha(Option<f32>),
    Lanzcos3,
    Lanzcos(Option<usize>),
}

pub struct Affine {
    affine: [[f32;3];3],    // 3*3
}

impl Affine {
    pub fn new() -> Self {
        let affine = [
            [1.0,0.0,0.0],
            [0.0,1.0,0.0],
            [0.0,0.0,1.0]];
        Self {
            affine,
        }
    }

    fn matrix(self:&mut Self,f: &[[f32;3];3]) {
        let affin = self.affine;
        let mut result:[[f32;3];3] = [[0.0,0.0,0.0],[0.0,0.0,0.0],[0.0,0.0,0.0]];
        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = affin[i][0] * f[0][j]
                             + affin[i][1] * f[1][j]
                             + affin[i][2] * f[2][j];
            }
        }
        self.affine = result;
    }

    pub fn translation(self:&mut Self,x:f32,y:f32) {
        self.matrix(&[[1.0 ,0.0 ,  x],
                      [0.0 ,1.0 ,  y],
                      [0.0 ,0.0 ,1.0]]);
    }

    pub fn invert_x(self:&mut Self) {
        self.matrix(&[[-1.0 , 0.0 , 0.0],
                      [ 0.0 , 1.0 , 0.0],
                      [ 0.0 , 0.0 , 1.0]]);
    }

    pub fn invert_y(self:&mut Self) {
        self.matrix(&[[ 1.0 ,  0.0 , 0.0],
                      [ 0.0 , -1.0 , 0.0],
                      [ 0.0 ,  0.0 , 1.0]]);
    }

    pub fn invert_xy(self:&mut Self) {
        self.matrix(&[[-1.0 , 0.0 , 0.0],
                      [ 0.0 ,-1.0 , 0.0],
                      [ 0.0 , 0.0 , 1.0]]);
    }

    pub fn scale(self:&mut Self,x:f32,y:f32) {
        self.matrix(&[[   x ,0.0 , 0.0],
                      [ 0.0 ,  y , 0.0],
                      [ 0.0 ,0.0 , 1.0]]);
    }

    pub fn rotate_by_dgree(self:&mut Self,theta:f32) {
        let theta = PI * theta / 180.0;
        self.rotate(theta);
    }

    pub fn rotate(self:&mut Self,theta:f32) {
        let c = theta.cos();
        let s = theta.sin();
        self.matrix(&[[   c , -s , 0.0],
                      [   s ,  c , 0.0],
                      [ 0.0 ,0.0 , 1.0]]);    
    }

    pub fn shear(self:&mut Self,x:f32,y:f32) {
        self.matrix(&[[ 1.0 ,  y , 0.0],
                      [   x ,1.0 , 0.0],
                      [ 0.0 ,0.0 , 1.0]]);    
    }

    #[inline]
    fn sinc (x:f32) -> f32 {
        (x * PI).sin() / (x * PI)
    }

    /* not implement Scaling Routine */
    pub fn _conversion(self:&mut Self,input_screen: &dyn Screen,output_screen:&mut dyn Screen) {
        let min_x = 0;
        let max_x = output_screen.width() as i32;
        let min_y = 0;
        let max_y = output_screen.height() as i32;

        let input_buffer = input_screen.buffer();
        let output_buffer: &mut [u8] = output_screen.buffer_as_mut();

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
                if xx < min_x || xx >= max_x {continue} // Out of bound

                let yy_ = x_ as f32 * y0 + y_ as f32 * y1 + y2;
                let yy = (yy_ + oy as f32).round() as i32;

                if yy < min_y || yy >= max_y {continue} // Out of bound


                let output_offset = ((yy * max_x + xx) *4) as usize;
                output_buffer[output_offset    ] = input_buffer[offset    ];
                output_buffer[output_offset + 1] = input_buffer[offset + 1];
                output_buffer[output_offset + 2] = input_buffer[offset + 2];
                output_buffer[output_offset + 3] = input_buffer[offset + 3]; 
            }
        }
    }

    pub fn conversion(self:&mut Self,input_screen: &Canvas,output_screen:&mut Canvas,algorithm:InterpolationAlgorithm) {

        let start_x = 0 as f32;
        let start_y = 0 as f32;

        let out_start_x = 0;
        let out_start_y = 0;
        let out_width = output_screen.width() as i32;
        let out_height = output_screen.height() as i32;

        self.conversion_with_area(input_screen,output_screen,
                start_x as f32,start_y,input_screen.width() as f32,input_screen.height() as f32,
                out_start_x,out_start_y,out_width as i32,out_height as i32,
                algorithm);
    }

    pub fn conversion_with_area(self:&mut Self,input_screen: &Canvas,output_screen:&mut Canvas,
                start_x :f32,start_y:f32 ,width: f32,height: f32,
                out_start_x :i32,out_start_y:i32 ,out_width: i32,out_height: i32,
                algorithm:InterpolationAlgorithm) {
        let end_x = width - start_x - 1.0;
        let end_y = height - start_y - 1.0;
        let out_end_x = out_width - out_start_x - 1;
        let out_end_y = out_height - out_start_y - 1;

        let ox = (out_width / 2) as f32;
        let oy = (out_height / 2) as f32;

        let mut alpha = -0.5;   // -0.5 - -1.0
        let mut lanzcos_n = 3;
        match algorithm {
            InterpolationAlgorithm::BicubicAlpha(a) =>{
                if a.is_some() {
                    alpha = a.unwrap()
                }
            },
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

        let mut xy = [(0_i32,0_i32);4];
        let x = start_x - ox;
        let y = start_y - oy;
        xy[0] = ((x * x0 + y * x1 + x2 + ox) as i32 ,(x * y0 + y * y1 + y2 +oy) as i32);
        let x = end_x - ox;
        let y = start_y - oy ;
        xy[1] = ((x * x0 + y * x1 + x2 + ox) as i32 ,(x * y0 + y * y1 + y2 +oy) as i32);
        let x = start_x - ox;
        let y = end_y - oy;
        xy[2] = ((x * x0 + y * x1 + x2 + ox) as i32 ,(x * y0 + y * y1 + y2 +oy) as i32);
        let x = end_x - ox;
        let y = end_y - oy;
        xy[3] = ((x * x0 + y * x1 + x2 + ox) as i32 ,(x * y0 + y * y1 + y2 +oy) as i32);

        xy.sort_by(|a, b| 
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
            });

        // pre-calc inverse affine transformation
        // x =  y1 * X - x0 * Y  + x1 * y2 - x2 * y1
        // y = -y0 * X + x1 * X  + x2 * y0 - x0 * x2

        let t = x0 * y1 - x1 * y0;
        let ix0 =  y1;
        let ix1 = -x1;
        let ix2 =  x1 * y2 - x2 * y1; 
        let iy0 = -y0;
        let iy1 =  x0;
        let iy2 =  x2 * y0 - x0 * x2;

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
                },
                1 => { 
                    sy = xy[1].1;
                    ey = xy[2].1;
                    xy0 = xy[1];
                    xy1 = xy[3];
                    xy2 = xy[0];
                    xy3 = xy[2];                 
                },
                2 => {
                    sy = xy[2].1;
                    ey = xy[3].1;
                    xy0 = xy[1];
                    xy1 = xy[3];
                    xy2 = xy[2];
                    xy3 = xy[3];                 
                },
                _ => {
                    sy = 0;
                    ey = 0;
                    xy0 = (0,0);
                    xy1 = (0,0);
                    xy2 = (0,0);
                    xy3 = (0,0);
                }
            }

            if sy < out_start_y { sy = out_start_y;}
            if ey > out_end_y {ey = out_end_y;}

            let d0 = if xy0.1 != xy1.1 {(xy0.0 as f32 - xy1.0 as f32) / (xy0.1  as f32 - xy1.1 as f32)} else {0.0};
            let d1 = if xy2.1 != xy3.1 {(xy2.0 as f32 - xy3.0 as f32) / (xy2.1  as f32 - xy3.1 as f32)} else {0.0};

            for y in sy..ey {
                // (x0,y0) - (x1,y1) &  (x2,y2) - (x3,y3)
                let (mut sx,mut ex) = if xy0.1 == xy1.1 {
                    (min(xy0.0 ,xy1.0),max(xy0.0,xy1.0)+1)
                } else {
                    if xy2.1 == xy3.1 {
                        (min(xy2.0 ,xy3.0),max(xy2.0,xy3.0)+1)
                    } else {
                        let x0 = (d0 * (y  - xy0.1) as f32) as i32 + xy0.0 as i32;
                        let x1 = (d1 * (y  - xy2.1) as f32) as i32 + xy2.0 as i32;
                        (min(x0,x1),max(x0,x1)+1)
                    }
                };
                let output_base_line = output_screen.width() as usize * 4 * y as usize;
                if sx < out_start_x { sx = out_start_x;}
                if ex > out_end_x {ex = out_end_x;}

                for x in sx..ex {
                    // inverse affine transformation from output image integer position
                    let xx = (ix0 * (x as f32 - ox) + ix1 * ( y as f32 - oy) + ix2 ) / t + ox;
                    let yy = (iy0 * (x as f32 - ox) + iy1 * ( y as f32 - oy) + iy2 ) / t + oy;
                    if xx < start_x || xx >= end_x || yy < start_y || yy >= end_x {continue;}
                    let output_offset = output_base_line + x as usize * 4;
                    let input_offset = (yy as usize * input_screen.width() as usize + xx as usize) * 4;
                    match algorithm {
                        InterpolationAlgorithm::NearestNeighber => {
                            output_screen.buffer[output_offset    ] = input_screen.buffer[input_offset    ];
                            output_screen.buffer[output_offset + 1] = input_screen.buffer[input_offset + 1];
                            output_screen.buffer[output_offset + 2] = input_screen.buffer[input_offset + 2];
                            output_screen.buffer[output_offset + 3] = input_screen.buffer[input_offset + 3];
                        },
                        InterpolationAlgorithm::Bilinear => {
                            let dx = xx - xx.floor();
                            let dy = yy - yy.floor();
                            let xx = xx.floor() as i32;
                            let yy = yy.floor() as i32;

                            let nx = if xx + 1 > end_x as i32 {0} else {4};
                            let ny = if yy + 1 > end_y as i32 {0} else {input_screen.width() as usize * 4};

                            let r   =(input_screen.buffer[input_offset         ] as f32 * (1.0-dx) * (1.0-dy)
                                    + input_screen.buffer[input_offset     + nx] as f32 * dx * (1.0-dy)
                                    + input_screen.buffer[input_offset     + ny] as f32 * (1.0-dx) * dy
                                    + input_screen.buffer[input_offset     + nx + ny] as f32 * dx * dy) as i32;
                            let g   =(input_screen.buffer[input_offset + 1     ] as f32 * (1.0-dx) * (1.0-dy)
                                    + input_screen.buffer[input_offset + 1 + nx] as f32 * dx * (1.0-dy)
                                    + input_screen.buffer[input_offset + 1 + ny] as f32 * (1.0-dx) * dy
                                    + input_screen.buffer[input_offset + 1 + nx + ny] as f32 * dx * dy) as i32;
                            let b   =(input_screen.buffer[input_offset + 2     ] as f32 * (1.0-dx) * (1.0-dy)
                                    + input_screen.buffer[input_offset + 2 + nx] as f32 * dx * (1.0-dy)
                                    + input_screen.buffer[input_offset + 2 + ny] as f32 * (1.0-dx) * dy
                                    + input_screen.buffer[input_offset + 2 + nx + ny] as f32 * dx * dy) as i32; 
                            let a   =(input_screen.buffer[input_offset + 3     ] as f32 * (1.0-dx) * (1.0-dy)
                                    + input_screen.buffer[input_offset + 3 + nx] as f32 * dx * (1.0-dy)
                                    + input_screen.buffer[input_offset + 3 + ny] as f32 * (1.0-dx) * dy
                                    + input_screen.buffer[input_offset + 3 + nx + ny] as f32 * dx * dy) as i32;
                                                       
                            output_screen.buffer[output_offset    ] = r.clamp(0,255) as u8;
                            output_screen.buffer[output_offset + 1] = g.clamp(0,255) as u8;
                            output_screen.buffer[output_offset + 2] = b.clamp(0,255) as u8;
                            output_screen.buffer[output_offset + 3] = a.clamp(0,255) as u8;
                        },
                        InterpolationAlgorithm::Bicubic | InterpolationAlgorithm::BicubicAlpha(_)  => {
                            let dx = xx - xx.floor();
                            let dy = yy - yy.floor();
                            let xx = xx.floor() as i32;
                            let yy = yy.floor() as i32;

                            let mut color = [0.0;4];

                            for _y in 0..4 {
                                let dy = _y as f32 - dy - 1.0 ;
                                let dy = dy.abs();
                                let wy = if dy <= 1.0 {
                                    (alpha + 2.0) * dy.powi(3) - (alpha + 3.0) * dy.powi(2) + 1.0
                                } else if dy < 2.0 {
                                    alpha * dy.powi(3) - 5.0 * alpha * dy.powi(2) + 8.0 * alpha * dy - 4.0 * alpha
                                } else {0.0};

                                let jy = _y - 1;  
                                let baseoffset = if yy + jy < start_y as i32
                                     {start_y as isize * input_screen.width() as isize * 4 }
                                else if yy + jy >= end_y as i32 { end_y as isize * input_screen.width() as isize * 4}
                                else { ((yy + jy) as isize * input_screen.width() as isize) * 4 };

                                for _x in 0..4 {
                                    let dx = _x as f32 - dx - 1.0;
                                    let dx = dx.abs();
                                    let jx = _x - 1;
                                    let offset = if xx + jx <= start_x as i32 { baseoffset + start_x as isize * 4 }
                                        else if xx + jx >= end_x as i32 { baseoffset + end_x as isize * 4}
                                        else { baseoffset + (xx + jx) as isize * 4 };
                                    let wx = if dx <= 1.0 {
                                        (alpha + 2.0) * dx.powi(3) - (alpha + 3.0) * dx.powi(2) + 1.0
                                    } else if dx < 2.0 {
                                        alpha * dx.powi(3) - 5.0 * alpha * dx.powi(2) + 8.0 * alpha * dx - 4.0 * alpha
                                    } else {0.0};

                                    let w = wx * wy;

                                    for i in 0..3 {
                                        color[i] += w * input_screen.buffer[offset as usize + i] as f32;
                                    }
                                }
                            }
                            
                            output_screen.buffer[output_offset    ] = (color[0] as i32).clamp(0,255) as u8;
                            output_screen.buffer[output_offset + 1] = (color[1] as i32).clamp(0,255) as u8;
                            output_screen.buffer[output_offset + 2] = (color[2] as i32).clamp(0,255) as u8;
                            output_screen.buffer[output_offset + 3] = 0xff;
                        },
                        InterpolationAlgorithm::Lanzcos3 | InterpolationAlgorithm::Lanzcos(_) => {
                            let dx = xx - xx.floor();
                            let dy = yy - yy.floor();
                            let xx = xx.floor() as i32;
                            let yy = yy.floor() as i32;
                            let n = lanzcos_n as i32;

                            let mut color = [0.0;4];


                            for _y in 0..2 * n {
                                let jy = _y - n + 1;  
                                let dy = (jy as f32 - dy).abs();
                                let wy = if dy == 0.0 { 1.0 }
                                         else if dy < n as f32 { Self::sinc(dy) * Self::sinc(dy / n as f32) }
                                         else { 0.0 };
                                let baseoffset = if yy + jy < start_y as i32
                                     {start_y as isize * input_screen.width() as isize * 4 }
                                else if yy + jy > end_y as i32 { end_y  as isize * input_screen.width() as isize * 4}
                                else { ((yy + jy) as isize * input_screen.width() as isize) * 4 };

                                for _x in 0..2 * n {
                                    let jx = _x - n + 1;  
                                    let dx = (jx as f32 - dx).abs();
                                    let wx = if dx == 0.0 { 1.0 }
                                             else if dx < n as f32 { Self::sinc(dx) * Self::sinc(dx / n as f32) }
                                             else {0.0};
                                    let offset = if xx + jx <= start_x as i32 { baseoffset + start_x as isize * 4 }
                                        else if xx + jx >= end_x as i32 { baseoffset + end_x as isize * 4}
                                        else { baseoffset + (xx + jx) as isize * 4 };

                                    let w = wx * wy;
                                    for i in 0..3 {
                                        color[i] += w * input_screen.buffer[offset as usize + i] as f32;
                                    }
                                }
                            }
                            
                            output_screen.buffer[output_offset    ] = (color[0] as i32).clamp(0,255) as u8;
                            output_screen.buffer[output_offset + 1] = (color[1] as i32).clamp(0,255) as u8;
                            output_screen.buffer[output_offset + 2] = (color[2] as i32).clamp(0,255) as u8;
                            output_screen.buffer[output_offset + 3] = 0xff;
                        },
                    }
                }    
            }
        }


    }


}