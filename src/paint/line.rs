/*
 *  line.rs (C)2022 Mith@mmk 
 *  Create 2022/02/21
 *  Update 2022/02/27
 *  Update 2022/03/13
 */

use super::pen::*;
use super::utils::color_taple;
use super::canvas::*;
use super::point::point_for_line;

// line no antialias (Bresenham's line algorithm)
pub fn line ( screen: &mut dyn Screen, x0: i32, y0: i32, x1: i32, y1: i32 , color: u32) {
    let (red, green, blue, _) = color_taple(color);
    let dx = (x0 - x1).abs();
    let dy = (y0 - y1).abs();

    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_y = if y0 < y1 { 1 } else { -1 };

    let mut err = dx - dy;

    let mut x = x0;
    let mut y = y0;

    loop {
        point_for_line (screen, x as i32, y as i32, red, green, blue, 0xff);
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

// line no antialias (Bresenham's line algorithm)
pub fn line_with_alpha ( screen: &mut dyn Screen, x0: i32, y0: i32, x1: i32, y1: i32 , color: u32, alpha: u8) {
    let (red, green, blue, _) = color_taple(color);
    let dx = (x0 - x1).abs();
    let dy = (y0 - y1).abs();

    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_y = if y0 < y1 { 1 } else { -1 };

    let mut err = dx - dy;

    let mut x = x0;
    let mut y = y0;

    loop {
        point_for_line(screen, x as i32, y as i32, red, green, blue, alpha);
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

pub fn line_pen (canvas: &mut Canvas, x0: i32, y0: i32, x1: i32, y1: i32 , color: u32) {
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
pub fn line_with_pen ( screen: &mut dyn Screen, x0: i32, y0: i32, x1: i32, y1: i32 , color: u32,pen: &Pen) {
    let dx = (x0 - x1).abs();
    let dy = (y0 - y1).abs();

    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_y = if y0 < y1 { 1 } else { -1 };

    let mut err = dx - dy;

    let mut x = x0;
    let mut y = y0;

    loop {
        point_with_pen(screen, x as i32, y as i32,color,pen);

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