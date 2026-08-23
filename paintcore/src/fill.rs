//! fill a closed shape.
/*
 * fill.rs  Mith@mmk (C) 2022
 *
 */

use crate::canvas::*;
use crate::line::*;
use crate::mask::Mask;
use crate::utils::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FloodColorMode {
    #[default]
    Rgb,
    Rgba,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FloodConnectivity {
    #[default]
    Four,
    Eight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FloodOptions {
    pub tolerance: u8,
    pub color_mode: FloodColorMode,
    pub connectivity: FloodConnectivity,
}

fn rgba_at(screen: &dyn Screen, x: u32, y: u32) -> Option<[u8; 4]> {
    if x >= screen.width() || y >= screen.height() {
        return None;
    }
    let offset = usize::try_from(y)
        .ok()?
        .checked_mul(usize::try_from(screen.width()).ok()?)?
        .checked_add(usize::try_from(x).ok()?)?
        .checked_mul(4)?;
    let pixel = screen.buffer().get(offset..offset.checked_add(4)?)?;
    Some([pixel[0], pixel[1], pixel[2], pixel[3]])
}

fn flood_matches(candidate: [u8; 4], reference: [u8; 4], options: FloodOptions) -> bool {
    let channels = if options.color_mode == FloodColorMode::Rgba {
        4
    } else {
        3
    };
    candidate[..channels]
        .iter()
        .zip(&reference[..channels])
        .all(|(&candidate, &reference)| candidate.abs_diff(reference) <= options.tolerance)
}

/// Finds a flood region without modifying the reference surface.
pub fn flood_mask(
    reference: &dyn Screen,
    start_x: i32,
    start_y: i32,
    options: FloodOptions,
) -> Mask {
    let mut mask = Mask::new(reference.width(), reference.height());
    if start_x < 0
        || start_y < 0
        || start_x >= reference.width() as i32
        || start_y >= reference.height() as i32
        || mask.is_empty()
    {
        return mask;
    }
    let Some(seed) = rgba_at(reference, start_x as u32, start_y as u32) else {
        return mask;
    };
    let mut queue = VecDeque::new();
    queue.push_back((start_x, start_y));
    mask.set(start_x, start_y, 255);

    const FOUR: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    const EIGHT: [(i32, i32); 8] = [
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
    ];
    let neighbors: &[(i32, i32)] = if options.connectivity == FloodConnectivity::Eight {
        &EIGHT
    } else {
        &FOUR
    };

    while let Some((x, y)) = queue.pop_front() {
        for &(dx, dy) in neighbors {
            let next_x = x.saturating_add(dx);
            let next_y = y.saturating_add(dy);
            if next_x < 0
                || next_y < 0
                || next_x >= reference.width() as i32
                || next_y >= reference.height() as i32
                || mask.get(next_x, next_y) != 0
            {
                continue;
            }
            let Some(candidate) = rgba_at(reference, next_x as u32, next_y as u32) else {
                continue;
            };
            if flood_matches(candidate, seed, options) {
                mask.set(next_x, next_y, 255);
                queue.push_back((next_x, next_y));
            }
        }
    }
    mask
}

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
            x += 1;
        }

        if x > rx {
            return;
        }

        if pick(screan, x, y) != base_color {
            return;
        }

        while x <= rx && pick(screan, x, y) == base_color {
            x += 1;
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
            lx -= 1;
        }

        // right scan
        loop {
            if rx + 1 >= screan.width() {
                break;
            }
            if pick(screan, rx + 1, ly) != base_color {
                break;
            }
            rx += 1;
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
