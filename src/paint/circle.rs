use crate::log;
use crate::paint::point::point;
use super::super::paint::canvas::Canvas;

use core::f32::consts::PI;

/*
  (rx,0) , (0,ry) を通る楕円を
  a x**2 + b y**2 = R**2 に変形
  a rx ** 2 = R ** 2 
  b ry ** 2 = R ** 2
  rx ** 2 + ry ** 2 = R ** 2
  a = R ** 2 / rx ** 2
  b = R ** 2 / ry ** 2  

  a x ** 2 + b y ** 2  == R **2
 */


pub fn arc (canvas: &mut Canvas,ox: i32,oy: i32,rx :f32,ry: f32,t0: f32,t1: f32,tilde : f32  ,color: u32) {
    if rx <= 0.0 || ry <= 0.0 {return;}

    log(&format!("{} {} {} {} {} ",ox,oy,rx,ry,tilde));
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
    let rpow2 = rx * rx + ry * ry;
    let a = rpow2 / rx.powi(2);
    let b = rpow2 / ry.powi(2);

    let mut fx: f32 = 0.0;
    let mut fy: f32 = ry;
    let mut x: i32 = 0;
    let mut y: i32 = ry.round() as i32;

    while fx <= rx  && fy >= 0.0 {

    // 0<= θ < PI/2
        let mut theta = x as f32 / rx * PI * 0.5; // calc θ
        let mut thetam = theta - 2.0 * PI ; 

        if (ts <= theta && theta <= te) || (ts <= thetam && thetam <= te) {
            point(canvas, ox + x, oy - y, color);
        }

    // PI/2 <= θ < PI
        theta =  PI - x as f32 / rx * PI * 0.5;
        thetam = theta - 2.0 * PI ; 

        if (ts <= theta && theta <= te) || (ts <= thetam && thetam <= te) {
            point(canvas, ox + x, oy + y, color);
        }
    // PI <= θ < 3PI/2
        theta =  PI * 1.0 +  x as f32 / rx * 0.5;
        thetam = theta - 2.0 * PI ; 

        if (ts <= theta && theta <= te) || (ts <= thetam && thetam <= te) {
            point(canvas, ox - x, oy + y, color);
        }
    // 3PI/2 <= θ < 2PI
        theta =  PI * 2.0 -  x as f32 / rx * PI * 0.5;
        thetam = theta - 2.0 * PI ; 

        if (ts <= theta && theta <= te) || (ts <= thetam && thetam <= te) {
            point(canvas, ox - x, oy - y, color);
        }
// next
        let dx = fx - x as f32;
        if dx >= 0.0 {
            x = x + 1;
            fx = x as f32;
            fy = ((rpow2 - a * fx.powi(2)) / b).sqrt();
        }
        let dy = (y as f32) - fy;
        if dy >= 0.0 {
            y = y - 1;
            fy = y as f32;
            fx = ((rpow2 - b * fy.powi(2)) / a).sqrt();
        }
    }
}


pub fn circle (canvas :&mut Canvas,ox: i32,oy: i32,r: f32 ,color: u32) {
    arc (canvas, ox, oy, r ,r ,0.0 ,2.0 * PI, 0.0, color)
}

pub fn ellipse (canvas :&mut Canvas,ox: i32,oy: i32,rx : f32,ry : f32,tilde : f32 ,color: u32) {
    arc (canvas, ox, oy, rx ,ry ,0.0 ,PI * 2.0,tilde, color)
}
