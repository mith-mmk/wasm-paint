/*
 * clear.rs  Mith@mmk (C) 2022
 * 
 */

use super::canvas::*;

pub fn clear_canvas(canvas: &mut Canvas) {
    let background_color = canvas.background_color();
    fillrect(canvas, background_color);
}

pub fn clear(screen: &mut dyn Screen) {
    let mut background_color = 0xff000000_u32;
    if let Some(alpha) = screen.alpha() {
        background_color &= (alpha as u32) << 24;
    }
    fillrect(screen, background_color);
}

pub fn fillrect(screen: &mut  dyn Screen, color: u32){
    let width = screen.width();
    let height = screen.height();
    let buf = &mut screen.buffer_as_mut();
    // Color model u32 LE (ARGB)  -> u8 BGRA
    let red: u8 = ((color  >> 16) & 0xff)  as u8; 
    let green: u8  = ((color >> 8) & 0xff) as u8; 
    let blue: u8 = ((color >> 0) & 0xff) as u8; 
    let alpha: u8 = 0xff;

    for y in 0..height {
        let offset = y * width * 4;
        for x  in 0..width {
            let pos :usize= (offset + (x * 4)) as usize;

            buf[pos] = red;
            buf[pos + 1] = green;
            buf[pos + 2] = blue;
            buf[pos + 3] = alpha;
        }
    }
}

