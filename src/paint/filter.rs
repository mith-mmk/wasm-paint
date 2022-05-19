
use super::canvas::*;


#[inline]
fn rgb_to_yuv(r:u8,g:u8,b:u8) -> (f32,f32,f32) {
    let r = r as f32;
    let g = g as f32;
    let b = b as f32;
    let y =  0.29900 * r + 0.58700 * g + 0.11400 * b;
    let u = -0.16874 * r - 0.33126 * g + 0.50000 * b;
    let v =  0.50000 * r - 0.41869 * g - 0.081   * b;
    (y,u,v)
}

#[inline]
fn rgb_to_y(r:u8,g:u8,b:u8) -> f32 {
    let r = r as f32;
    let g = g as f32;
    let b = b as f32;
    let y =  0.29900 * r + 0.58700 * g + 0.11400 * b;
    y
}


#[inline]
fn yuv_to_rgb(y:f32,u:f32,v:f32) -> (u8,u8,u8) {
    let crr = 1.402;
    let cbg = - 0.34414;
    let crg = - 0.71414;
    let cbb = 1.772;

    let r = (y + (crr * v)) as i32;
    let g = (y + (cbg * u) + crg * v) as i32;
    let b = (y + (cbb * u)) as i32;

    let r = r.clamp(0,255) as u8;
    let g = g.clamp(0,255) as u8;
    let b = b.clamp(0,255) as u8;

    (r,g,b)
}

pub fn lum_filter(src:&dyn Screen,dest: &mut dyn Screen,matrix:&[[f32;3];3]) {
    dest.reinit(src.width(), src.height());
    let mut coeff = 0.0;
    for u in 0..3 {
        for v in 0..3 {
            coeff += matrix[v][u];
        }
    }
    let src_buffer = src.buffer();
    let dest_buffer = dest.buffer_mut();

    for y in 0..src.height() as usize{
        let offset = y * src.width() as usize * 4;
        for x in 0..src.width() as usize {
            let r = src_buffer[offset + x * 4];
            let g = src_buffer[offset + x * 4 + 1];
            let b = src_buffer[offset + x * 4 + 2];
            let a = src_buffer[offset + x * 4 + 3];
            let mut l = 0.0;
            for u in 0..3 {
                let uu = (y  as i32 - u + 2).clamp(0,src.height() as i32) as usize
                         * src.width() as usize * 4;
                for v in 0..3 {
                    let vv = (y  as i32 - v + 2).clamp(0,src.height() as i32) as usize * 4;
                    let r = src_buffer[uu + vv];
                    let g = src_buffer[uu + vv + 1];
                    let b = src_buffer[uu + vv + 2];
                    let ln = rgb_to_y(r, g, b);
                    l += ln * matrix[v as usize][u as usize];
                }
            }
            let l = l / coeff;
            let (_,u,v) = rgb_to_yuv(r, g, b);
            let (r,g,b) = yuv_to_rgb(l, u, v);

            dest_buffer[offset + x * 4] = r;
            dest_buffer[offset + x * 4 + 1] = g;
            dest_buffer[offset + x * 4 + 2] = b;
            dest_buffer[offset + x * 4 + 3] = a;

        }
    }
}

pub fn sharpness(src:&dyn Screen, dest:&mut dyn Screen) {
    let matrix = [
            [-1.0,-1.0,-1.0],
            [-1.0,10.0,-1.0],
            [-1.0,-1.0,-1.0]];
    lum_filter(src,dest,&matrix)
}

pub fn blur(src:&dyn Screen, dest:&mut dyn Screen) {
    let matrix = [
            [ 1.0, 1.0, 1.0],
            [ 1.0, 1.0, 1.0],
            [ 1.0, 1.0, 1.0]];
    lum_filter(src,dest,&matrix)
}


pub fn medien(src:&dyn Screen,dest:&mut dyn Screen){
    dest.reinit(src.width(), src.height());

    let src_buffer = src.buffer();
    let dest_buffer = dest.buffer_mut();

    for y in 0..src.height() as usize{
        let offset = y * src.width() as usize * 4;
        for x in 0..src.width() as usize {
            let a = src_buffer[offset + x * 4 + 3];
            let mut l = [(0.0,0,0,0);9];
            for u in 0..3 {
                let uu = (y  as i32 - u + 2).clamp(0,src.height() as i32) as usize
                         * src.width() as usize * 4;
                for v in 0..3 {
                    let vv = (y  as i32 - v + 2).clamp(0,src.height() as i32) as usize * 4;
                    let r = src_buffer[uu + vv];
                    let g = src_buffer[uu + vv + 1];
                    let b = src_buffer[uu + vv + 2];
                    let ln = rgb_to_y(r, g, b);
                    l[u as usize *3 + v as usize] = (ln,r,g,b);
                }
            }
            let mut l = l.to_vec();
            l.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let l = l[5];

            dest_buffer[offset + x * 4] = l.1;
            dest_buffer[offset + x * 4 + 1] = l.2;
            dest_buffer[offset + x * 4 + 2] = l.3;
            dest_buffer[offset + x * 4 + 3] = a;

        }
    }
}