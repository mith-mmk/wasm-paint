use super::super::paint::canvas::Canvas;
use core::cmp::max;
use core::cmp::min;

pub fn color_taple(color: u32) -> (u8,u8,u8,u8) {
    let alpha: u8 = ((color  >> 24) & 0xff)  as u8; 
    let red: u8 = ((color  >> 16) & 0xff)  as u8; 
    let green: u8  = ((color >> 8) & 0xff) as u8; 
    let blue: u8 = ((color >> 0) & 0xff) as u8; 
    (red,green,blue,alpha)
}

pub fn pick_taple(canvas: &mut Canvas,x :u32,y: u32) ->  (u8,u8,u8,u8) {
    let buf = &canvas.buffer;
    let pos :usize= (y * canvas.width() * 4 + (x * 4)) as usize;

    let r = buf[pos];
    let g = buf[pos + 1];
    let b = buf[pos + 2];
    let a = buf[pos + 3];
    (r,g,b,a)
}

pub fn pick(canvas: &mut Canvas,x :u32,y: u32) -> u32 {
    let (r,g,b,_) = pick_taple(canvas, x, y);
    let color :u32 = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
    color
}

pub fn normalization_points(canvas: &Canvas,x0: i32,y0 :i32,x1 :i32,y1 :i32 ) -> (u32,u32,u32,u32) {
    let width = canvas.width();
    let height = canvas.height();
    let sx = { let x = min(x0,x1); if x < 0 { 0 as u32 } else { x as u32 }};
    let ex = { let x = max(x0,x1); if x >= width as i32 { width - 1 } else { x as u32 }};
    let sy = { let y = min(y0,y1); if y < 0 { 0 as u32 } else { y as u32 }};
    let ey = { let y = max(y0,y1); if y >= height as i32 { height - 1 } else { y as u32 }};

    (sx,sy,ex,ey)
}
