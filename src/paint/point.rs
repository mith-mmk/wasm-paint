use super::super::paint::canvas::Canvas;
use super::super::paint::utils::*;

fn _point (canvas: &mut Canvas, x: i32, y: i32, red :u8, green :u8, blue :u8, alpha :u8) {
    if x < 0 || y < 0 || x >= canvas.width() as i32 || y >= canvas.height() as i32 || alpha == 0 {
        return;
    }
    let width = canvas.width();
    let buf = &mut canvas.buffer;
    let pos :usize= (y as u32 * width * 4 + (x as u32 * 4)) as usize;
    let a = alpha as f32 / 255_f32;
    let r = (buf[pos] as f32 * (1.0 - a) + red as f32 * a) as u8;
    let g = (buf[pos + 1] as f32 * (1.0 - a) + green as f32 * a) as u8;
    let b = (buf[pos + 2]  as f32* (1.0 - a) + blue as f32 * a) as u8;

    buf[pos] = r;
    buf[pos + 1] = g;
    buf[pos + 2] = b;
    buf[pos + 3] = 0xff;
}

pub fn point ( canvas: &mut Canvas, x: i32, y: i32, color: u32) {
    let (red, green, blue, _) = color_taple(color);
    _point(canvas, x as i32, y as i32, red, green, blue, 0xff);
}

pub fn point_antialias(canvas: &mut Canvas, x: f32, y: f32, color: u32,s: f32) {
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

    let weight00 = (dx0 * dy0 * alpha * 255.0_f32).round() as u8;
    let weight01 = (dx0 * dy1 * alpha * 255.0_f32).round() as u8;
    let weight10 = (dx1 * dy0 * alpha * 255.0_f32).round() as u8;
    let weight11 = (dx1 * dy1 * alpha * 255.0_f32).round() as u8;
    let weight0y = (dy0 * alpha * 255.0_f32).round() as u8;
    let weight1y = (dy1 * alpha * 255.0_f32).round() as u8;
    let weightx0 = (dx0 * alpha * 255.0_f32).round() as u8;
    let weightx1 = (dx1 * alpha * 255.0_f32).round() as u8;

    let px = sx as i32;
    let py = sy as i32;
    let rx = ex as i32;
    let ry = ey as i32;
/*
    log (&format!("{} {} {} {}, {} {} {} {}",px,py,rx,ry,dx0,dy0,dx1,dy1));
    log (&format!("{} {} {}\n{} {} {}\n {} {} {}\n",
        weight00,weight0y,weight01,weightx0,255,weightx1,weight10,weight1y,weight11));
*/
    _point(canvas, px, py, red, green, blue, weight00);
    _point(canvas, rx, py, red, green, blue, weight10);
    _point(canvas, px, ry, red, green, blue, weight01);
    _point(canvas, rx ,ry, red, green, blue, weight11);

    if py + 1 < ry {
        for qy in py + 1 .. ry {
            _point(canvas, px, qy, red, green, blue, weightx0);
        }
        for qy in py + 1 .. ry {
            _point(canvas, rx, qy, red, green, blue, weightx1);
        }
    }


    if px + 1 < rx {
        for qx in px + 1 .. rx {
            _point(canvas, qx, py, red, green, blue, weight0y);
            if py + 1 < ry {
                for qy in py + 1 .. ry {
                    _point(canvas, qx, qy, red, green, blue, 0xff);
                }
            }
            _point(canvas, qx, ry, red, green, blue, weight1y);
        }
    }
}
