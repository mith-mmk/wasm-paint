/*
 *  line.rs (C)2022 Mith@mmk 
 *  Create 2022/02/21
 *  Update 2022/02/27
 *  Update 2022/03/13
 */

use super::utils::color_taple;
use super::canvas::*;
use super::pen::point_pen;


// use for line only _point function
fn _point (canvas: &mut Canvas, x: i32, y: i32, r :u8, g :u8, b :u8, a :u8) {
    if x < 0 || y < 0 || x >= canvas.width() as i32 || y >= canvas.height() as i32 || a == 0 {
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

// line no antialias (Bresenham's line algorithm)
pub fn line ( canvas: &mut Canvas, x0: i32, y0: i32, x1: i32, y1: i32 , color: u32) {
    let dx = (x0 - x1).abs();
    let dy = (y0 - y1).abs();

    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_y = if y0 < y1 { 1 } else { -1 };

    let mut err = dx - dy;

    let mut x = x0;
    let mut y = y0;

    loop {
        point_pen(canvas, x as i32, y as i32,color);

        if x == x1 && y == y1 {
            break;
        }
        
        let err2 = err * 2;

        if err2 > -dy  {
            err = err - dy;
            x = x + step_x;
        }

        if err2 < dx  {
            err = err + dx;
            y = y + step_y;
        }

    }
}

pub fn line_with_pen ( canvas: &mut Canvas, x0: i32, y0: i32, x1: i32, y1: i32 , color: u32) {
    let (red, green, blue, _) = color_taple(color);

    let dx = (x0 - x1).abs();
    let dy = (y0 - y1).abs();

    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_y = if y0 < y1 { 1 } else { -1 };

    let mut err = dx - dy;

    let mut x = x0;
    let mut y = y0;

    loop {
        _point(canvas, x as i32, y as i32, red, green, blue, 0xff);

        if x == x1 && y == y1 {
            break;
        }
        
        let err2 = err * 2;

        if err2 > -dy  {
            err = err - dy;
            x = x + step_x;
        }

        if err2 < dx  {
            err = err + dx;
            y = y + step_y;
        }

    }
}