use super::utils::*;
use super::canvas::*;

pub fn rect(screen: &mut dyn Screen,x0: i32,y0: i32,x1: i32,y1: i32, color: u32){ 
    rect_with_alpha(screen,x0,y0,x1,y1, color,0xff);
}

pub fn rect_with_alpha(screen: &mut dyn Screen,x0: i32,y0: i32,x1: i32,y1: i32, color: u32,alpha: u8){ 
    let (sx,sy,ex,ey) = normalization_points(screen,x0,y0,x1,y1);

    let width = screen.width();

    let buf = &mut screen.buffer_as_mut();
    // Color model u32 LE (ARGB)  -> u8 BGRA
    let (red,green,blue,_) = color_taple(color);

    for y in sy ..ey + 1 {
        let pos = ((y * width + sx) * 4) as usize;
        buf[pos] = red;
        buf[pos + 1] = green;
        buf[pos + 2] = blue;
        buf[pos + 3] = alpha;

        let pos = ((y * width + ex) * 4) as usize;
        buf[pos] = red;
        buf[pos + 1] = green;
        buf[pos + 2] = blue;
        buf[pos + 3] = alpha;
    }

    for x  in sx..ex + 1 {
        let offset = sy * width * 4;
        let pos :usize= (offset + (x * 4)) as usize;
        buf[pos] = red;
        buf[pos + 1] = green;
        buf[pos + 2] = blue;
        buf[pos + 3] = alpha;
    }

    for x  in sx..ex + 1 {
        let offset = ey * width * 4;
        let pos :usize= (offset + (x * 4)) as usize;
        buf[pos] = red;
        buf[pos + 1] = green;
        buf[pos + 2] = blue;
        buf[pos + 3] = alpha;
    }
}