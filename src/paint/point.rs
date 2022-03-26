/*  point.rs
 *  create 2022/02/20
 *  update 2022/02/28  internal point method change
 */

use super::canvas::Screen;
use super::utils::color_taple;

fn _point (screen: &mut dyn Screen, x: i32, y: i32, red :u8, green :u8, blue :u8, alpha :u8, weight :f32) {
    if x < 0 || y < 0 || x >= screen.width() as i32 || y >= screen.height() as i32 || weight == 0.0 {
        return;
    }
    let a = weight;
    let width = screen.width();
    let buf = &mut screen.buffer_as_mut();
    let pos :usize= (y as u32 * width * 4 + (x as u32 * 4)) as usize;

    if a == 1.0 {
        buf[pos] = red;
        buf[pos + 1] = green;
        buf[pos + 2] = blue;
        buf[pos + 3] = alpha;
        return;
    }

    let r = (buf[pos] as f32 * (1.0 - a) + red as f32 * a) as u8;
    let g = (buf[pos + 1] as f32 * (1.0 - a) + green as f32 * a) as u8;
    let b = (buf[pos + 2]  as f32* (1.0 - a) + blue as f32 * a) as u8;

    buf[pos] = r;
    buf[pos + 1] = g;
    buf[pos + 2] = b;
    buf[pos + 3] = alpha;
}

pub fn point ( screen: &mut dyn Screen, x: i32, y: i32, color: u32) {
    let (red, green, blue, _) = color_taple(color);
    _point(screen, x as i32, y as i32, red, green, blue, 0xff, 1.0);
}

pub fn point_with_alpha ( screen: &mut dyn Screen, x: i32, y: i32, color: u32) {
    let (red, green, blue, alpha) = color_taple(color);
    _point(screen, x as i32, y as i32, red, green, blue, alpha, 1.0);
}

pub fn point_with_weight ( screen: &mut dyn Screen, x: i32, y: i32, color: u32, weight: f32) {
    let (red, green, blue, _) = color_taple(color);
    _point(screen, x as i32, y as i32, red, green, blue, 0xff, weight);
}

pub fn point_with_weight_from_alpha ( screen: &mut dyn Screen, x: i32, y: i32, color: u32) {
    let (red, green, blue, alpha) = color_taple(color);
    _point(screen, x as i32, y as i32, red, green, blue, 0xff, alpha as f32 / 255.0);
}

pub fn point_antialias(screen: &mut dyn Screen, x: f32, y: f32, color: u32,s: f32) {
    if s <= 0.0 {return};
    let (red, green, blue, _) = color_taple(color);
    let alpha = 1.0_f32;
                                                // x = 4.5 y = 5.0 s = 4.5
    let sx :f32 = (x + 0.5 - s / 2.0).floor();  // sx : 2
    let sy :f32 = (y + 0.5 - s / 2.0).floor();  // sy : 3
    let ex :f32 = (x + 0.5 + s / 2.0).ceil();   // ex : 8
    let ey :f32 = (y + 0.5 + s / 2.0).ceil();   // ey : 8

    let dx0 = 1.0_f32 - ((x + 0.5 - s / 2.0) - sx);  // dx0 = 0.25
    let dy0 = 1.0_f32 - ((y + 0.5 - s / 2.0) - sy);   // dy0 = 0.75
    let dx1 = 1.0_f32 - (ex - (x + 0.5 + s / 2.0));   // dx1 = 0.25
    let dy1 = 1.0_f32 - (ey - (y+ 0.5 + s / 2.0));  // dy1 = 0.75
    /*
      00        0y       10
     (dx0,dy0)   (1 ,dy0)  (dx1, dy0)
              +----+
    (dx0, 1)   |(1,1)| (dx1 ,1)
    x0        |     | x1
              +----+
             (1, dy0) 1y  (dx1, dy1) 11
    (dx0,dy1) 01
    */

    let weight00 = dx0 * dy0 * alpha;
    let weight01 = dx0 * dy1 * alpha;
    let weight10 = dx1 * dy0 * alpha;
    let weight11 = dx1 * dy1 * alpha;
    let weight0y = dy0 * alpha;
    let weight1y = dy1 * alpha;
    let weightx0 = dx0 * alpha;
    let weightx1 = dx1 * alpha;

    let px = sx as i32;
    let py = sy as i32;
    let rx = ex as i32;
    let ry = ey as i32;
/*
    log (&format!("{} {} {} {}, {} {} {} {}",px,py,rx,ry,dx0,dy0,dx1,dy1));
    log (&format!("{} {} {}\n{} {} {}\n {} {} {}\n",
        weight00,weight0y,weight01,weightx0,255,weightx1,weight10,weight1y,weight11));
*/
    _point(screen, px, py, red, green, blue, 0xff, weight00);
    _point(screen, rx, py, red, green, blue, 0xff, weight10);
    _point(screen, px, ry, red, green, blue, 0xff, weight01);
    _point(screen, rx ,ry, red, green, blue, 0xff, weight11);

    if py + 1 < ry {
        for qy in py + 1 .. ry {
            _point(screen, px, qy, red, green, blue, 0xff, weightx0);
        }
        for qy in py + 1 .. ry {
            _point(screen, rx, qy, red, green, blue, 0xff, weightx1);
        }
    }


    if px + 1 < rx {
        for qx in px + 1 .. rx {
            _point(screen, qx, py, red, green, blue, 0xff, weight0y);
            if py + 1 < ry {
                for qy in py + 1 .. ry {
                    _point(screen, qx, qy, red, green, blue, 0xff, 1.0);
                }
            }
            _point(screen, qx, ry, red, green, blue, 0xff, weight1y);
        }
    }
}
