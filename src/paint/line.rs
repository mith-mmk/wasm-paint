use crate::paint::utils::color_taple;
use crate::Canvas;

fn _point (canvas: &mut Canvas, x: i64, y: i64, r :u8, g :u8, b :u8, a :u8) {
    if x < 0 || y < 0 || x >= canvas.width() as i64 || y >= canvas.height() as i64 || a == 0 {
        return;
    }
    let width = canvas.width();
    let buf = &mut canvas.buffer;
    let pos :usize= (y as u32 * width * 4 + (x as u32 * 4)) as usize;

    buf[pos] = r;
    buf[pos + 1] = g;
    buf[pos + 2] = b;
    buf[pos + 3] = 0xff;
}

pub fn line ( canvas: &mut Canvas, x0: i32, y0: i32, x1: i32, y1: i32 , color: u32) {
    let (red, green, blue, _) = color_taple(color);
    let sx;
    let ex;
    let sy;
    let ey;
    if x0 > x1 {
        sx = x1;
        ex = x0;
        sy = y1;
        ey = y0;

    } else {
        sx = x0;
        ex = x1;
        sy = y0;
        ey = y1;
    }
    if ex == sx {
        let x = sx;
        if sy <= ey {
            for y in sy..ey + 1 {
                _point(canvas, x as i64, y as i64, red, green, blue, 0xff);
            }
        } else {
            for y in ey..sy + 1 {
                _point(canvas, x as i64, y as i64, red, green, blue, 0xff);
            }
        }
    } else if ey == sy {
        for x in sx..ex + 1 {
            _point(canvas, x as i64, sy as i64, red, green, blue, 0xff);
        }
    } else {
        let delta = (ey -sy) as f32 / (ex - sx) as f32;

        if -1.0 < delta && delta < 1.0 {
            let mut fy = sy as f32;
            for x in sx..ex + 1 {
                let y = fy.round() as i32;
                _point(canvas, x as i64, y as i64, red, green, blue, 0xff);
                fy = fy + delta;
            }
        } else {
            let mut fx = sx as f32;
            let mut y = sy;
            let dy;
            let delta;
            if sy <= ey {
                dy = 1;
                delta = (ex - sx) as f32 / (ey -sy) as f32 ;
            } else {
                delta = - (ex - sx) as f32 / (ey -sy) as f32 ;
                dy = -1;
            }
            while y != ey {
                let x = fx.round() as i32;
                _point(canvas, x as i64, y as i64, red, green, blue, 0xff);
                fx = fx + delta;
                y = y + dy;
            }
            _point(canvas, ex as i64, ey as i64, red, green, blue, 0xff);

        }
    }
}