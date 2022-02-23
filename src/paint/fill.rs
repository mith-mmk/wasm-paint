use super::super::paint::utils::*;
use super::super::paint::line::*;
use super::super::paint::canvas::Canvas;

pub struct ScanStack {
    pub sx :u32, 
    pub sy :u32,  
}

impl ScanStack {
    pub fn new(sx: u32,sy: u32) -> Self{
        Self {
            sx,
            sy,
        }
    }
}

fn scan_line(canvas: &mut Canvas,lx:u32, rx: u32,y :u32,stacks :&mut Vec<ScanStack>,base_color: u32) {
    let mut x = lx;
    while x <= rx {
        while x <= rx && pick(canvas, x, y) != base_color {
            x = x + 1;
        }

        if pick(canvas, x, y) != base_color {
            return;
        }

        while x <= rx && pick(canvas, x, y) == base_color {
            x = x + 1;
        }

        stacks.push(ScanStack::new(x - 1, y));
    }
}

pub fn fill ( canvas: &mut Canvas, sx: i32, sy: i32, paint_color: u32) {
    if sx < 0 || sx >= canvas.width() as i32 || sy < 0 || sy >= canvas.height() as i32 {return}
    let mut stacks :Vec<ScanStack> = Vec::new();
    stacks.push(ScanStack::new(sx as u32, sy as u32));
    let base_color = pick(canvas, sx as  u32, sy as u32);
    if base_color == paint_color & 0xffffff {return}
    let mut index :usize = 0;
    while stacks.len() > index {
        let stack = &stacks[index];
        index = index + 1;
        let (sx,sy) = (stack.sx,stack.sy);
        let current_color = pick(canvas,sx,sy);
        // current point
        if current_color != base_color {
            continue;
        }
    
        let ly = stack.sy;
        let (mut lx,mut rx) = (stack.sx, stack.sx);

        // left scan
        loop {
            if lx == 0 {break};
            if pick(canvas,lx - 1,ly) != base_color {
                break;
            }
            lx = lx - 1;
        }

        // right scan
        loop {
            if rx + 1 >= canvas.width() {
                break;
            }
            if pick(canvas,rx + 1,ly) != base_color {
                break;
            }
            rx = rx + 1;
        }

        // draw line
        line(canvas,lx as i32 ,ly as i32,rx as i32 ,ly as i32, paint_color & 0xffffff);

        if ly + 1 < canvas.height() {
            scan_line(canvas, lx, rx, ly + 1, &mut stacks, base_color);
        }

        if ly >= 1 {
            scan_line(canvas, lx, rx, ly - 1, &mut stacks, base_color);
        }
    }
}


