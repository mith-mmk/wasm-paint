use super::utils::*;
use super::canvas::*;


pub fn rect(canvas: &mut Canvas,x0: i32,y0: i32,x1: i32,y1: i32, color: u32){ 
    let (sx,sy,ex,ey) = normalization_points(&canvas,x0,y0,x1,y1);

    let width = canvas.width();

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