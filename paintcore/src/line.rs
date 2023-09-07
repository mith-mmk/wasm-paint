//! draw a line.
/*
 *  line.rs (C)2022 Mith@mmk
 *  Create 2022/02/21
 *  Update 2022/02/27
 *  Update 2022/03/13
 */

use crate::canvas::*;
use crate::pen::*;
use crate::point::point_antialias;
use crate::point::point_for_line;
use crate::utils::color_taple;

/// line no antialias (Bresenham's line algorithm)
/// color = RGB888 no include alpha mask
pub fn line(screen: &mut dyn Screen, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    let (red, green, blue, _) = color_taple(color);
    let dx = (x0 - x1).abs();
    let dy = (y0 - y1).abs();

    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_y = if y0 < y1 { 1 } else { -1 };

    let mut err = dx - dy;

    let mut x = x0;
    let mut y = y0;

    loop {
        point_for_line(screen, x as i32, y as i32, red, green, blue, 0xff);
        if x == x1 && y == y1 {
            break;
        }

        let err2 = err * 2;

        if err2 > -dy {
            err = err - dy;
            x = x + step_x;
        }

        if err2 < dx {
            err = err + dx;
            y = y + step_y;
        }
    }
}

// line no antialias (Bresenham's line algorithm)
pub fn line_with_alpha(
    screen: &mut dyn Screen,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: u32,
    alpha: u8,
) {
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

        if err2 > -dy {
            err = err - dy;
            x = x + step_x;
        }

        if err2 < dx {
            err = err + dx;
            y = y + step_y;
        }
    }
}

/// line_antialias uses alternative Xiaolin Wu's line algorithm
pub fn line_antialias(
    screen: &mut dyn Screen,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    color: u32,
    alpha: u8,
    size: f32,
) {
    if x0 == x1 && y0 == y1 {
        point_antialias(screen, x0, y0, color, alpha, size);
        return;
    }

    let dx = x0 - x1;
    let dy = y0 - y1;
    let ddx = dx.abs();
    let ddy = dy.abs();

    let step_x = if ddx > ddy {
        if x0 < x1 {
            1.0
        } else {
            -1.0
        }
    } else {
        if y0 < y1 {
            dx / dy
        } else {
            -dx / dy
        }
    };
    let step_y = if ddx <= ddy {
        if y0 < y1 {
            1.0
        } else {
            -1.0
        }
    } else {
        if x0 < x1 {
            dy / dx
        } else {
            -dy / dx
        }
    };

    let step = if ddx > ddy { true } else { false };

    let (mut x, mx) = if x0 < x1 {
        let x = x0.ceil();
        (x, x - x0)
    } else {
        let x = x0.floor();
        (x, x0 - x)
    };
    let (mut y, my) = if y0 < y1 {
        let y = y0.ceil();
        (y, y - y0)
    } else {
        let y = y0.floor();
        (y, y0 - y)
    };

    if !step {
        x = (dx / dy) * my + x0;
    } else {
        y = (dy / dx) * mx + y0;
    }
    let x1 = if x0 > x1 { x1.ceil() } else { x1.floor() };
    let y1 = if y0 > y1 { y1.ceil() } else { y1.floor() };

    point_antialias(screen, x0, y0, color, alpha, size);
    point_antialias(screen, x1, y1, color, alpha, size);
    if step && (x - x1).abs() <= 1.0 || !step && (y - y1).abs() <= 1.0 {
        return;
    }
    loop {
        point_antialias(screen, x, y, color, alpha, size);
        if step && x == x1 || !step && y == y1 {
            break;
        }
        x += step_x;
        y += step_y;
    }
}

pub fn line_pen(canvas: &mut Canvas, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    let dx = (x0 - x1).abs();
    let dy = (y0 - y1).abs();

    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_y = if y0 < y1 { 1 } else { -1 };

    let mut err = dx - dy;

    let mut x = x0;
    let mut y = y0;

    loop {
        point_pen(canvas, x as i32, y as i32, color);

        if x == x1 && y == y1 {
            break;
        }

        let err2 = err * 2;

        if err2 > -dy {
            err = err - dy;
            x = x + step_x;
        }

        if err2 < dx {
            err = err + dx;
            y = y + step_y;
        }
    }
}
pub fn line_with_pen(
    screen: &mut dyn Screen,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: u32,
    pen: &Pen,
) {
    let dx = (x0 - x1).abs();
    let dy = (y0 - y1).abs();

    let step_x = if x0 < x1 { 1 } else { -1 };
    let step_y = if y0 < y1 { 1 } else { -1 };

    let mut err = dx - dy;

    let mut x = x0;
    let mut y = y0;

    loop {
        point_with_pen(screen, x as i32, y as i32, color, pen);

        if x == x1 && y == y1 {
            break;
        }

        let err2 = err * 2;

        if err2 > -dy {
            err = err - dy;
            x = x + step_x;
        }

        if err2 < dx {
            err = err + dx;
            y = y + step_y;
        }
    }
}
