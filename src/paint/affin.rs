/*
 * affine.rs  Mith@mmk (C) 2022
 * update 2022/03/13
 */

pub struct Affin {
    affin: [f32;9],    // 3*3
}

impl Affin {
    fn new() -> Self {
        let affin = 
            [1.0,0.0,0.0,
             0.0,1.0,0.0,
             0.0,0.0,1.0];
        Self {
            affin,
        }
    }

    fn matrix(self:&mut Self,f: &[f32]) {
        let affin = self.affin;
        let mut result:[f32;9] = [0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0,0.0];
        for i in 1..3 {
            for j in 1..3 {
                result[j+i*3] = affin[i*3] * f[j] + affin[i*3+1] * f[j+3] + affin[i*3+2] * f[j+6];
            }
        }
        self.affin = result;
    }
}