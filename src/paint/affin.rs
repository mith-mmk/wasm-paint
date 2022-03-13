/*
 * affine.rs  Mith@mmk (C) 2022
 * update 2022/03/13
 */

use core::f32::consts::PI;
use super::canvas::*;

pub struct Affin {
    affin: [[f32;3];3],    // 3*3
}

impl Affin {
    fn new() -> Self {
        let affin = [
            [1.0,0.0,0.0],
            [0.0,1.0,0.0],
            [0.0,0.0,1.0]];
        Self {
            affin,
        }
    }

    fn matrix(self:&mut Self,f: &[[f32;3];3]) {
        let affin = self.affin;
        let mut result:[[f32;3];3] = [[0.0,0.0,0.0],[0.0,0.0,0.0],[0.0,0.0,0.0]];
        for i in 1..3 {
            for j in 1..3 {
                result[i][j] = affin[i][0] * f[0][j] 
                             + affin[i][1] * f[1][j]
                             + affin[i][2] * f[2][j];
            }
        }
        self.affin = result;
    }

    pub fn translation(self:&mut Self,x:f32,y:f32) {
        self.matrix(&[[1.0 ,0.0 ,  x],
                      [0.0 ,1.0 ,  y],
                      [0.0 ,0.0 ,1.0]]);
    }

    pub fn invert_x(self:&mut Self,width:u32) {
        let w = width as f32;
        self.matrix(&[[-1.0 ,0.0 , w  ],
                      [ 0.0 ,1.0 , 0.0],
                      [ 0.0 ,0.0 , 1.0]]);
    }

    pub fn invert_y(self:&mut Self,height:u32) {
        let h = height as f32;
        self.matrix(&[[-1.0 ,0.0 , 0.0],
                      [ 0.0 ,1.0 ,   h],
                      [ 0.0 ,0.0 , 1.0]]);
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

    /* not implement Scaling Routine */
    pub fn conversion(self:&mut Self,input_canvas: &Canvas,output_canvas:&mut Canvas) {
        let min_x = 0;
        let max_x = output_canvas.width() as i32;
        let min_y = 0;
        let max_y = output_canvas.height() as i32;

        // |X|   |a00 a01 a02||x|         X = a00 * x + a01 * y + a02
        // |Y| = |a10 a11 a12||y|         Y = a10 * x + a11 * y + a12
        // |Z|   |a20 a21 a22||1|         _  do not use

        let x0 = self.affin[0][0];
        let x1 = self.affin[0][1];
        let x2 = self.affin[0][2];
        let y0 = self.affin[1][0];
        let y1 = self.affin[1][1];
        let y2 = self.affin[1][2];

        for y in 0..input_canvas.height() as usize {
            let input_base_line = input_canvas.width() as usize * 4 * y;
            for x in 0..input_canvas.width() as usize {
                let offset = input_base_line + x * 4;
                let xx = (x as f32 * x0 + y as f32 * x1 + x2).round() as i32;
                if xx < min_x || xx >= max_x {continue} // Out of bound
                let yy = (x as f32 * y0 + y as f32 * y1 + y2).round() as i32;
                if yy < min_y || yy >= max_y {continue} // Out of bound
                let output_offset = ((yy * max_x + xx) *4) as usize;
                output_canvas.buffer[output_offset    ] = input_canvas.buffer[offset    ]; 
                output_canvas.buffer[output_offset + 1] = input_canvas.buffer[offset + 1]; 
                output_canvas.buffer[output_offset + 2] = input_canvas.buffer[offset + 2]; 
                output_canvas.buffer[output_offset + 3] = input_canvas.buffer[offset + 3]; 
            }
        }
    }
}