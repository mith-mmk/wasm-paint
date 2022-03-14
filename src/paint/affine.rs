/*
 * affine.rs  Mith@mmk (C) 2022
 * create 2022/03/13
 */

use core::cmp::max;
use core::cmp::min;
use std::cmp::Ordering;
use core::f32::consts::PI;
use super::canvas::*;

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

    /* not implement Scaling Routine */
    pub fn _conversion(self:&mut Self,input_canvas: &Canvas,output_canvas:&mut Canvas) {
        let min_x = 0;
        let max_x = output_canvas.width() as i32;
        let min_y = 0;
        let max_y = output_canvas.height() as i32;

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

        for y in 0..input_canvas.height() as usize {
            let input_base_line = input_canvas.width() as usize * 4 * y;
            for x in 0..input_canvas.width() as usize {
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
                output_canvas.buffer[output_offset    ] = input_canvas.buffer[offset    ];
                output_canvas.buffer[output_offset + 1] = input_canvas.buffer[offset + 1];
                output_canvas.buffer[output_offset + 2] = input_canvas.buffer[offset + 2];
                output_canvas.buffer[output_offset + 3] = input_canvas.buffer[offset + 3]; 
            }
        }
    }

    pub fn conversion(self:&mut Self,input_canvas: &Canvas,output_canvas:&mut Canvas) {
        // 関数の拡張を考えたインプリメント

        let start_x = 0 as f32;
        let start_y = 0 as f32;
        let end_x = input_canvas.width() as f32 - start_x - 1.0;
        let end_y = input_canvas.height() as f32 - start_y - 1.0;

        let out_start_x = 0;
        let out_start_y = 0;
        let out_width = output_canvas.width() as i32;
        let out_height = output_canvas.height() as i32;
        let out_end_x = out_width - out_start_x - 1;
        let out_end_y = out_height - out_start_x - 1;

        let ox = (out_width / 2) as f32;
        let oy = (out_height / 2) as f32;

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

        // 順番に並び替える 
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


        // stage 0 y0..y1 (x0,y0) - (x1,y1) &  (x0,y0) - (x2,y2)
        // stage 1 y1..y2 (x0,y0) - (x2,x2) &  (x1,y1) - (x3,y3) 
        // stage 2 y2..y3 (x1,y1) - (x3,y3) &  (x2,y2) - (x3,y3)

        let t = x0 * y1 - x1 * y0;

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

            let d0 = (xy0.0 as f32 - xy1.0 as f32) / (xy0.1  as f32 - xy1.1 as f32);
            let d1 = (xy2.0 as f32 - xy3.0 as f32) / (xy2.1  as f32 - xy3.1 as f32);

            for y in sy..ey {
                // (x0,y0) - (x1,y1) &  (x2,y2) - (x3,y3)
                let (mut sx,mut ex) = if xy0.1 == xy1.1 {
                        (min(xy0.0 ,xy1.0),max(xy0.0,xy1.0)+1)
                } else {
                        let x0 = (d0 * (y  - xy0.1) as f32) as i32 + xy0.0 as i32;
                        let x1 = (d1 * (y  - xy2.1) as f32) as i32 + xy2.0 as i32;
                        (min(x0,x1),max(x0,x1)+1)
                };
                let output_base_line = output_canvas.width() as usize * 4 * y as usize;
                if sx < out_start_x { sx = out_start_x;}
                if ex > out_end_x {ex = out_end_x;}
                for x in sx..ex {
                    // 逆アフィン変換
                    let xx = (  y0 * (x as f32 - ox - x2) - y1 * ( y as f32 - oy -y2)) / t + ox;
                    let yy = (- x0 * (x as f32 - ox - x2) + x1 * ( y as f32 - oy -y2)) / t + oy;
                    // ニアレストネイバー法
                    let xx = xx.round() as i32;
                    let yy = yy.round() as i32;
                    if xx < start_x as i32 || xx >= end_x as i32 || yy < start_y as i32 || yy >= end_x as i32 {continue;}
                    let output_offset = output_base_line + x as usize * 4;
                    let input_offset = (yy as usize * input_canvas.width() as usize + xx as usize) * 4;

                    output_canvas.buffer[output_offset    ] = input_canvas.buffer[input_offset    ];
                    output_canvas.buffer[output_offset + 1] = input_canvas.buffer[input_offset + 1];
                    output_canvas.buffer[output_offset + 2] = input_canvas.buffer[input_offset + 2];
                    output_canvas.buffer[output_offset + 3] = input_canvas.buffer[input_offset + 3];


                }    
            }
        }
    }

        
    /*
     * 画像補完
     * 
     * 1. 回転時の座標ずれの補正
     *
     * 座標系を中央(ox,oy)に移動させ、最後に戻す
     *　|1　0 -ox||a00 a01 a02||x||1  0  ox|   |a00x+a01y+a02-ox|1 0 ox|  |a00*(x-ox) + a01(y-oy) + a02 + ox|
     *　|0  1 -oy||a10 a11 a12||y||0  1  oy| = |a10x+a11y+a12-oy|0 1 oy|= |a10*(x-ox) + a11(y-oy) + a12 + oy|
     *　|0　0   1||  0  0    1||1||0　0   1|   |     1          |0 0  1|  |               1                 |
     *
     * 2.色補間簡略化の戦略
     * 
     * 2.1 計算法
     * 
     * 最初に頂点のみを計算し、直線を計算し、その中にある点をアフィン逆変換し補完する。最初に四点求めることで計算量を減らす
     * 矩形かつ変形で捻れないのが前提。
     * 
     *  (x,y) = A-1 (X,Y)
     *  |x - ox|           1         | a11   -a01|| X - ox - a02|
     *  |y - oy| = ----------------  |-a10    a00|| Y - oy - a12|
     *              a00a11 - a01a10 = t
     *
     *  x = 1/t *[  a11 * (X - ox - a02) - a01 * ( Y - oy -a12)] + ox
     *  y = 1/t *[- a01 * (X - ox - a02) + a00 * ( Y - oy -a12)] + oy 　/
     * 
     * 2.2 探索法
     * 
     * L1 (X0,Y0) - (X1,X1) L2 (X1,Y1) - (X2,X2)
     * L3 (X2,Y2) - (X3,X3) L4 (X3,X3) - (X0,X0) に囲まれた空間内の点か判定するため、
     * 
     * min(Y) - max(Y) の点Y' のときの X0' - X1' を計算する
     * 
     * まずY0≦Y1≦Y2≦Y3にソートする。
     * 
     * s1  (X0 - X1)/(Y0 - Y1) Y  (X0 - X2)/(Y0 - Y2) Y 　Xを計算  Y0 ≦ y < Y1 を評価
     * s2. (X3 - X1)/(Y3 - Y1) Y  (X0 - X2)/(Y0 - Y2) Y   Xを計算  Y1 ≦ y ≦ Y2 を評価
     * s3. (X3 - X1)/(Y3 - Y1) Y  (X3 - X2)/(Y3 - Y2) Y   Xを計算  Y2 < y ≦ Y3 を評価
     * 
     * 正確には　[(X0 - X1)/(Y0 - Y1)] * (Y - Y0) + X0  dy = 0 の場合は x0 .. x1 を一回だけ回す
     *
     * 2.3 拡大・縮小アルゴリズムの選定
     * 拡大か縮小かは　以下で計算　mx √(a00^2 + a01^2) my √a(10^2 + a11^2)  >1なら拡大　< 1なら縮小
     * 0.5までは縮小アルゴリズムは想定しなくて良い
     * 
     * ニアレストネイバー法　最短の座標から色を取得　縮小も同じ
     * 
     * バイリニア法　(x,y)(x+1,y)(x,y+1)(x+1,y+1)の　4ドットの加重平均
     * 　<0.5 の時の縮小アルゴリズムは 1/mx x 1/my ドットの平均
     * 　両方が混じる場合は、ミックス
     * 
     * バイキュービック法 16ドットにフィルタ　縮小もバイキュービック
     *  x' - x = dとしたとき (xは整数) つまり計算に用いるのは(x - 1, x, x+1, x+2)(y-1,..,y+2) の16ドット
     *  (α + 2)*d^3  - (α + 3)*d^2 + α                 d ≦ 1.0 
     *  α*d^3 - 5α*d^2 - 8α*d - 4α              1.0 <  d  < 2.0 
     *  0                                              d ≧ 2.0
     * 
     * 　α = -0.5～-0.75, -1 
     * 
     * Lanzcos法 縮小もLanzcos　中身はフーリエ変換
     * 　sincと言う関数が出てくる 中身はsin　sinc(x) = sin(πx)/πx 
     *  
     * 計算式 α = 3の場合、16ドットで計算(Lanzcos3) 
     * 1                                d = 0
     * α*sin(dπ)sin(dπ/α)/π^2*x^2       0 < |d| < α
     * 0                                d > α
     * 
     * 高速化戦略： sinの計算量をいかに減らすか
     * 
     * 
     * 2.4アルゴリズム使用時の注意
     * 　端のドットの扱い。 0とするか、(0,0)を補完するかで結果が変わる。
     *
     */
}