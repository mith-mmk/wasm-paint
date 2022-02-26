use core::f32::consts::PI;
use super::super::paint::line::*;
use super::super::paint::canvas::Canvas;


// pentagram (五芒星)
pub fn pentagram(canvas: &mut Canvas,ox: i32,oy: i32,r : f32,tilde :f32, color: u32){ 
    polygram(canvas,5,2,ox,oy,r,tilde,color);
}

// hexagram (六芒星)
pub fn hexagram(canvas: &mut Canvas,ox: i32,oy: i32,r : f32,tilde :f32, color: u32){ 
    polygram(canvas,6,2,ox,oy,r,tilde,color);
}

// Reglar Pollygon (正多角形)
pub fn reglar_polygon(canvas: &mut Canvas,p: u32,ox: i32,oy: i32,r : f32,tilde :f32, color: u32) {
    polygram(canvas,p,1,ox,oy,r,tilde,color);
}

// 中点(ox,oy) 半径r の円に内接する、Schläfli symbol {p/q}角形を傾き(tilde)で指定したcolorで描画する。

pub fn polygram(canvas: &mut Canvas,p: u32,q: u32,ox: i32,oy: i32,r : f32,tilde :f32, color: u32){
    if r < 0.0 || p <= 2 {return};
 
    let angle = 2.0 * PI  / p as f32; // = 72.0 dgree


    let mut x = vec![0; p as usize];
    let mut y = vec![0; p as usize];

    // point x0  = r * sin (0) , y0 = - r * cos (0) (0,-r)
    for i in 0..p as usize {
        x[i] =  ox + (r * (i as f32 * angle + tilde).sin()).floor() as i32;
        y[i] =  oy - (r * (i as f32 * angle + tilde).cos()).floor() as i32;    
    }

    for i in 0..p as usize {
        let s = i as usize;
        let e = (i + q as usize) % p as usize;
        line(canvas,x[s],y[s],x[e],y[e],color);
    }
}
