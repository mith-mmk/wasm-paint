use super::super::paint::canvas::Canvas;

// add 2022/02/22
pub fn clear(canvas: &mut Canvas) {
    fillrect(canvas, canvas.background_color());
}

pub fn fillrect(canvas: &mut Canvas, color: u32){
    let width = canvas.width();
    let height = canvas.height();
    let buf = &mut canvas.buffer;
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

