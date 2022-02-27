/*
 *  Circle.rs
 *  Create 2022/02/26
 *  Update 2022/02/27
 */
//use crate::log;
use super::line::line;
use super::point::*;
use super::canvas::Canvas;

use core::f32::consts::PI;

/*
 *  (rx,0) , (0,ry) を通る楕円を
 * a x**2 + b y**2 = R**2 に変形
 *
 * a = ry ** 2
 * b = rx ** 2
 * R**2 = rx ** 2 + ry ** 2
 *
 * 
 * 誤差値 ただし、E = a x**2 + b y ** 2 - R**2
 * x2 + 2x 1
 * Px  f(x-1,y  ) Ex = E - 2ax + a
 * Pxy f(x-1,y+1) Exy= E - 2ax + 2by + a + b
 * Py  f(x  ,y+1) Ey = E - 2by + b
 * 
 * 点の選択
 * |Ex| < |Exy| and |Ex| < |Ey| -> Px  (1)
 * |Ey| < |Ex| and |Ey| < |Exy| -> Py  (2)
 * |Exy| < |Ex| and |Exy| < |Ey| -> Pxy  (3)
 * 
 * (1)
 *   Ex **2 - Exy ** 2 < 0  -> a( 2x - 1 )( 2E - 2ax + 4by + a + 2b ) < 0    (A)
 *   Ex **2 - Ey ** 2 < 0   -> { a( 2x - 1 ) + b( 2y + 1 ) }( 2E - 2ax + 2by + a + b ) < 0	(B)
 *    Aより  2E - 2ax + 4by + a + 2b < 0 （∵ a > 0 and 2x -1 > 0　∵ x >= 0) (C)
 *    Bより  2E - 2ax + 2by + a + b < 0 （∵ { a( 2x - 1 ) + b( 2y + 1 ) } > 0　∵ x >= 0) (D)
 * 
 * 　（以下省略)
 *   C = 2E - 2ax + 4by + a + 2b
 *   D = 2E - 2ax + 2by + a + b 
 *   E = 2E - 4ax + 2by + 2a + b 
 *
 * 　（以下省略)
 * https://fussy.web.fc2.com/algo/algo2-2.htm
 *
 */

 /* arc (ox,oy)を中点とするθ0 = t0 ,θ1 = t1 を満たす半径rx,ryの楕円弧を描く。　θはラジアン
  * 　ただし θ=0 を時計の12時の位置とし、時計回りとする。ただし、-2π <= θ <= 2π　とする
  *   |t0 - t1| >= 2π の場合、楕円を描き、さらにrx=ryの時、円を描く
  */

pub fn arc (canvas: &mut Canvas,ox: i32,oy: i32,rx :i32,ry: i32,t0: f32,t1: f32 ,color: u32) {
    if rx <= 0 || ry <= 0 {return;}

    /* arc */
    let ts;
    let te;
    if (t0 - t1).abs() >= 2.0 * PI { // a round
        ts = 0.0; te = PI * 2.0;
    } else {
        ts = t0 % (PI * 2.0);
        te = t1 % (PI * 2.0);
    }

    /* ellipse */
    /* rx **2 ,ry **2 が大きいので i64に変換している rx,ryが3万越えることは無いと思うが */
    let a: i64 = (ry as i64).pow(2);
    let b: i64 = (rx as i64).pow(2);
    let rpow2: i64 = a * b;
    let d: i64 = (ry as f64 * (rpow2 as f64).sqrt()) as i64;

    let mut x: i32 = rx;
    let mut y: i32 = 0;

    let mut err1: i64 = -2 * d +     a  + 2 * b;
    let mut err2: i64 = -4 * d + 2 * a +      b;

    while x >= 0 {
    // 0<= θ < PI/2
        let mut theta = (x as f32 / rx as f32).asin(); // calc θ
        let mut thetam = theta - 2.0 * PI ;

        if (ts <= theta && theta <= te) || (ts <= thetam && thetam <= te) { // arc check
            // reverse y axis,shift (ox,oy)
            point(canvas, ox + x, oy - y, color);
        }
    // PI/2 <= θ < PI  この象限はPI -> PI/2 で描画するので反転する
        theta =  PI - (x as f32 / rx as f32).asin();
        thetam = theta - 2.0 * PI ; 

        if (ts <= theta && theta <= te) || (ts <= thetam && thetam <= te) {
            point(canvas, ox + x, oy + y, color);
        }
    // PI <= θ < 3PI/2
        theta =  PI * 1.0 +  (x as f32 / rx as f32).asin();
        thetam = theta - 2.0 * PI ; 

        if (ts <= theta && theta <= te) || (ts <= thetam && thetam <= te) {
            point(canvas, ox - x, oy + y, color);
        }
    // 3PI/2 <= θ < 2PI  この象限は2PI -> 3PI/2 で描画するので反転する
        theta =  PI * 2.0 -  (x as f32 / rx as f32).asin();
        thetam = theta - 2.0 * PI ; 

        if (ts <= theta && theta <= te) || (ts <= thetam && thetam <= te) {
            point(canvas, ox - x, oy - y, color);
        }
// next
        if err1 >= 0 {
            x = x - 1;
            err1 = err1 - 4 * a * x as i64;
            err2 = err2 - 4 * a * x as i64 - 2 * a;
        }

        if err2 < 0 {
            y = y + 1;
            err1 = err1 + 4 * b * y as i64 + 2 * b;
            err2 = err2 + 4 * b * y as i64;
        }
    }
}

/* arc_tilde
 * 　傾く楕円を描く、傾きの計算にアフィン変換を利用し、隙間はlineで補完する（手抜き）
 */
pub fn arc_tilde (canvas: &mut Canvas,ox: i32,oy: i32,rx :f32,ry: f32,t0: f32,t1: f32,tilde : f32  ,color: u32) {
    if rx <= 0.0 || ry <= 0.0 {return;}

    /* arc */
    let ts;
    let te;
    if (t0 - t1).abs() >= 2.0 * PI { // a round
        ts = 0.0; te = PI * 2.0;
    } else {
        ts = t0 % (PI * 2.0);
        te = t1 % (PI * 2.0);
    }

    /* ellipse */
    let a: i64 = (ry as i64).pow(2);
    let b: i64 = (rx as i64).pow(2);
    let rpow2: i64 = a * b;
    let d: i64 = (ry as f64 * (rpow2 as f64).sqrt()) as i64;

    let mut x: i32 = rx.ceil() as i32;
    let mut y: i32 = 0;

    let mut err1: i64 = -2 * d +     a  + 2 * b;
    let mut err2: i64 = -4 * d + 2 * a +      b;

    let mut bx: [i32;4] = [i32::MIN, i32::MIN, i32::MIN, i32::MIN];
    let mut by: [i32;4] = [ 0, 0, 0, 0];

    /* tilde is clockwise */
    let tsin: f32 = tilde.sin();
    let tcos: f32 = tilde.cos();


    while x >= 0 as i32 {
    // 0<= θ < PI/2
        let mut theta = (x as f32 / rx as f32).asin(); // calc θ
        let mut thetam = theta - 2.0 * PI ; 

        let mut i = 0;

        if (ts <= theta && theta <= te) || (ts <= thetam && thetam <= te) { // arc check
            // afin transration
            /*          shift         tilde      rev y
             * |x'|  | 1 0 ox|| cosθ -sinθ  0||1  0  0|
             * |y'| =| 0 1 oy|| sinθ  cosθ  0||0 -1  0||x y 1|
             * |1 |  | 0 0  1||   0      0  1||0  0  1|
             * 
             *  x' = ox + xx , y' = oy + yy
             * |xx|   | cosθ -sinθ 0|
             * |yy| = | sinθ cosθ  0||x -y 1|
             * | 1|   | 0     0    1|
             *
             * xx = x * cosθ + y * sinθ
             * yy = y * sinθ - y * cosθ
             */

             // rotate (tilde), becose the roteate start point is clock number 12.
            let xx =  (x as f32 * tcos + y as f32 * tsin).floor() as i32;
            let yy =  (x as f32 * tsin - y as f32 * tcos).floor() as i32;

            if bx[i] == i32::MIN {
                bx[i] = xx;
                by[i] = yy;
            }

            line(canvas, ox + bx[i], oy + by[i],ox + xx,oy + yy, color);
            bx[i] = xx;
            by[i] = yy;
            i = i + 1;
        } else {
            bx[i] = i32::MIN;
            i = i + 1;
        }
    
        // PI/2 <= θ < PI
        theta =  PI - (x as f32 / rx as f32).asin();
        thetam = theta - 2.0 * PI ; 

        if (ts <= theta && theta <= te) || (ts <= thetam && thetam <= te) {
            // reverce y + rotate (tilde)
            /*          shift         tilde      rev y    rev y
             * |x'|  | 1 0 ox|| cosθ  -sinθ  0||1  0  0||1  0  1|
             * |y'| =| 0 1 oy|| sinθ   cosθ  0||0 -1  0||0 -1  1||x y 1|
             * |1 |  | 0 0  1||  0       0   1||0  0  1||0  0  1|
             * 
             *  x' = ox + xx , y' = oy + yy
             * |xx|   |cosθ -sinθ 0||1 0 0|           | cosθ -sinθ 0|
             * |yy| = |sinθ  cosθ 0||0 1 0||x y 1| =  | sinθ  cosθ 0||x y 1|
             * | 1|   |0     0    1||0 0 1|           | 0     0    1|
             * 
             * xx = x * cosθ - y * sinθ
             * yy = x * sinθ + y * cosθ
             */
            let xx =  (x as f32 * tcos - y as f32 * tsin).floor() as i32;
            let yy =  (x as f32 * tsin + y as f32 * tcos).floor() as i32;

            if bx[i] == i32::MIN {
                bx[i] = xx;
                by[i] = yy;
            }

            line(canvas, ox + bx[i], oy + by[i],ox + xx,oy + yy, color);
            bx[i] = xx;
            by[i] = yy;
            i = i + 1;
        } else {
            bx[i] = i32::MIN;
            i = i + 1;
        }

        // PI <= θ < 3PI/2
        theta =  PI * 1.0 + (x as f32 / rx as f32).asin();
        thetam = theta - 2.0 * PI ; 

        if (ts <= theta && theta <= te) || (ts <= thetam && thetam <= te) {
            // reverce xy + rotate (tilde)
            /*          shift         tilde      rev xy    rev y
             * |x'|  | 1 0 ox|| cosθ  -sinθ  0||-1  0  0||1  0  1|
             * |y'| =| 0 1 oy|| sinθ   cosθ  0|| 0 -1  0||0 -1  1||x y 1|
             * |1 |  | 0 0  1||  0       0   1|| 0  0  1||0  0  1|
             * 
             *  x' = ox + xx , y' = oy + yy
             * |xx|   |cosθ -sinθ 0|            |-cosθ -sinθ 0|
             * |yy| = |sinθ  cosθ 0||-x y 1| =  |-sinθ  cosθ 0||x y 1|
             * | 1|   |0     0    1|            | 0      0    1|
             * 
             * xx = - x * cosθ - y * sinθ
             * yy = - x * sinθ + y * cosθ
             */
            let xx = (-x as f32 * tcos - y as f32 * tsin).floor() as i32;
            let yy = (-x as f32 * tsin + y as f32 * tcos).floor() as i32;

            if bx[i] == i32::MIN {
                bx[i] = xx;
                by[i] = yy;
            }

            line(canvas, ox + bx[i], oy + by[i],ox + xx,oy + yy, color);
            bx[i] = xx;
            by[i] = yy;
            i = i + 1;
        } else {
            bx[i] = i32::MIN;
            i = i + 1;
        }
        // 3PI/2 <= θ < 2PI
        theta =  PI * 2.0 - (x as f32 / rx as f32).asin();
        thetam = theta - 2.0 * PI ; 

        if (ts <= theta && theta <= te) || (ts <= thetam && thetam <= te) {
            // reverce x + rotate (tilde)
            /*          shift         tilde      rev x    rev y
             * |x'|  | 1 0 ox|| cosθ  -sinθ  0||-1  0  0||1  0  1|
             * |y'| =| 0 1 oy|| sinθ   cosθ  0|| 0  1  0||0 -1  1||x y 1|
             * |1 |  | 0 0  1||  0       0   1|| 0  0  1||0  0  1|
             * 
             *  x' = ox + xx , y' = oy + yy
             * |xx|   |cosθ -sinθ 0|             |-cosθ  sinθ 0|
             * |yy| = |sinθ  cosθ 0||-x -y 1| =  |-sinθ -cosθ 0||x y 1|
             * | 1|   |0     0    1|             | 0      0    1|
             * 
             * xx = - x * cosθ + y * sinθ
             * yy = - x * sinθ - y * cosθ
             */
            let xx = (-x as f32 * tcos + y as f32 * tsin).floor() as i32;
            let yy = (-x as f32 * tsin - y as f32 * tcos).floor() as i32;

            if bx[i] == i32::MIN {
                bx[i] = xx;
                by[i] = yy;
            }

            line(canvas, ox + bx[i], oy + by[i],ox + xx,oy + yy, color);
            bx[i] = xx;
            by[i] = yy;
        } else {
            bx[i] = i32::MIN;
        }
// next
        if err1 >= 0 {
            x = x - 1;
            err1 = err1 - 4 * a * x as i64;
            err2 = err2 - 4 * a * x as i64 - 2 * a;
        }

        if err2 < 0 {
            y = y + 1;
            err1 = err1 + 4 * b * y as i64 + 2 * b;
            err2 = err2 + 4 * b * y as i64;
        }
    }
}

pub fn circle (canvas :&mut Canvas,ox: i32,oy: i32,r: i32 ,color: u32) {
    arc (canvas, ox, oy, r ,r ,0.0 ,2.0 * PI, color)
}

pub fn ellipse (canvas :&mut Canvas,ox: i32,oy: i32,rx : i32,ry : i32,tilde: f32,color: u32) {
    if tilde == 0.0 {
        arc(canvas, ox, oy, rx ,ry , 0.0, 2.0 * PI, color);
    } else {
        arc_tilde(canvas, ox, oy, rx as f32, ry as f32, 0.0 * PI, 2.0 * PI, tilde, color);
    }
}
