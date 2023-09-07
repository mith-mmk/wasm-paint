//! fill a closed shape.
/*
 * fill.rs  Mith@mmk (C) 2022
 *
 */

use crate::canvas::*;
use crate::line::*;
use crate::utils::*;

pub struct ScanStack {
    pub sx: u32,
    pub sy: u32,
}

impl ScanStack {
    pub fn new(sx: u32, sy: u32) -> Self {
        Self { sx, sy }
    }
}

fn scan_line(
    screan: &mut dyn Screen,
    lx: u32,
    rx: u32,
    y: u32,
    stacks: &mut Vec<ScanStack>,
    base_color: u32,
) {
    let mut x = lx;
    while x <= rx {
        while x <= rx && pick(screan, x, y) != base_color {
            x = x + 1;
        }

        if pick(screan, x, y) != base_color {
            return;
        }

        while x <= rx && pick(screan, x, y) == base_color {
            x = x + 1;
        }

        stacks.push(ScanStack::new(x - 1, y));
    }
}

pub fn fill(screan: &mut dyn Screen, sx: i32, sy: i32, paint_color: u32) {
    fill_with_alpha(screan, sx, sy, paint_color, 0xff);
}

pub fn fill_with_alpha(screan: &mut dyn Screen, sx: i32, sy: i32, paint_color: u32, alpha: u8) {
    if sx < 0 || sx >= screan.width() as i32 || sy < 0 || sy >= screan.height() as i32 {
        return;
    }
    let mut stacks: Vec<ScanStack> = Vec::new();
    stacks.push(ScanStack::new(sx as u32, sy as u32));
    let base_color = pick(screan, sx as u32, sy as u32);
    if base_color == paint_color & 0xffffff {
        return;
    }

    while let Some(stack) = stacks.pop() {
        let (sx, sy) = (stack.sx, stack.sy);
        let current_color = pick(screan, sx, sy);
        // current point
        if current_color != base_color {
            continue;
        }

        let ly = stack.sy;
        let (mut lx, mut rx) = (stack.sx, stack.sx);

        // left scan
        loop {
            if lx == 0 {
                break;
            };
            if pick(screan, lx - 1, ly) != base_color {
                break;
            }
            lx = lx - 1;
        }

        // right scan
        loop {
            if rx + 1 >= screan.width() {
                break;
            }
            if pick(screan, rx + 1, ly) != base_color {
                break;
            }
            rx = rx + 1;
        }

        // draw line
        line_with_alpha(
            screan,
            lx as i32,
            ly as i32,
            rx as i32,
            ly as i32,
            paint_color & 0xffffff,
            alpha,
        );

        if ly + 1 < screan.height() {
            scan_line(screan, lx, rx, ly + 1, &mut stacks, base_color);
        }

        if ly >= 1 {
            scan_line(screan, lx, rx, ly - 1, &mut stacks, base_color);
        }
    }
}
