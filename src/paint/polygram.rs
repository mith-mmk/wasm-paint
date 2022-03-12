/*
 * polygram.rs  Mith@mmk (C) 2022
 * use MIT License
 */

use core::f32::consts::PI;
use super::line::*;
use super::canvas::Canvas;


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

/*
  一辺A(x0,y0) - B(x1,y1) の長さからrを計算する方法
  a = √{(x0-1)**2 ++ (y0-y1)**2} とおく
  360 / n = xとする。
　内接するOABの三角形を二等分した、△OAHの計算をする
　∠A = x/2  ∠H = 90° ∠O = 90 - x/2

  AO = r
　AH = AB/2 = a/2 = r sin (x/2)
  r = a / 2sin (x/2)
  OH = √{r**2 - (AB/2)**2}

　正三角形 x = 120 より r = a / 2sin60° = a√3/3
  正四角形 x = 90 より r =  a / 2sin45° = a√2 /2
  正五角形 x = 72 より r = a / 2sin36°
  正n角形 x = 360/n より r = a / 2sin(180/n)° = a / 2sin(π/n) (rad)

  x0 = ox とする と oy = y0 - r 
  このとき x1' = x0 + a cos(π/n)  y1' = y0 + a sin(π/n) 

※　 要検証
  tilde = arccos ((x1 - x0)/(x1' - x0))  (y1' <= y1) ただし x1 > x0 のとき tilde > 0
  tilde = arccos ((x1 - x0)/(x1' - x0))  (y1' > y1) 
*/

pub fn _reglar_polygon2(canvas: &mut Canvas,p: u32,x0: i32,y0: i32,x1: i32,y1: i32, color: u32) {
    let a = (((x0 - x1).pow(2) + (y0 - y1).pow(2)) as f32).sqrt();
    let r = a / 2.0 * (PI/p as f32).sin();
    let ox = x0;
    let oy = y0 + r.floor() as i32;
    let dx = a * (PI/p as f32).cos();
    let dy = (y0 - y1) as f32 + (a * (PI/p as f32).sin());
    let tilde = if dy <= 0.0 { ((x1 - x0) as f32 / dx).asin() } else { - ((x1 - x0) as f32/ dx).asin() };
    polygram(canvas,p,1,ox,oy,r,tilde,color);
}
