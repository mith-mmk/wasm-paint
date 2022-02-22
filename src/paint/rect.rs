use crate::paint::utils::color_taple;
use core::cmp::max;
use core::cmp::min;
use super::super::Canvas;


pub fn rect(canvas: &mut Canvas,x0: i32,y0: i32,x1: i32,y1: i32, color: u32){ 
    let width = canvas.width();
    let height = canvas.height();

    let sx = { let x = min(x0,x1); if x < 0 { 0 as u32 } else { x as u32 }};
    let ex = { let x = max(x0,x1); if x >= width as i32 { width - 1 } else { x as u32 }};
    let sy = { let y = min(y0,y1); if y < 0 { 0 as u32 } else { y as u32 }};
    let ey = { let y = max(y0,y1); if y >= height as i32 { height - 1 } else { y as u32 }};

    let buf = &mut canvas.buffer;
    // Color model u32 LE (ARGB)  -> u8 BGRA
    let (red,green,blue,_) = color_taple(color);

    for y in sy ..ey + 1 {
        let pos = ((y * width + sx) * 4) as usize;
        buf[pos] = red;
        buf[pos + 1] = green;
        buf[pos + 2] = blue;
        buf[pos + 3] = 0xff;

        let pos = ((y * width + ex) * 4) as usize;
        buf[pos] = red;
        buf[pos + 1] = green;
        buf[pos + 2] = blue;
        buf[pos + 3] = 0xff;
    }

    for x  in sx..ex + 1 {
        let offset = sy * width * 4;
        let pos :usize= (offset + (x * 4)) as usize;
        buf[pos] = red;
        buf[pos + 1] = green;
        buf[pos + 2] = blue;
        buf[pos + 3] = 0xff;
    }

    for x  in sx..ex + 1 {
        let offset = ey * width * 4;
        let pos :usize= (offset + (x * 4)) as usize;
        buf[pos] = red;
        buf[pos + 1] = green;
        buf[pos + 2] = blue;
        buf[pos + 3] = 0xff;
    }
}