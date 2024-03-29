//! spline module draws curve quadratic curve,bezier curve.

use crate::point::*;
use crate::line::*;
use crate::canvas::*;

/// draw quadratic curve
/// Parameter a changes half circle, ellipse, parabola or hyperbola
/// Circle or ellipse is a = -2.0
pub fn quadratic_curve(screen:&mut dyn Screen,p:Vec<(f32,f32)>,a:f32,color: u32) {
    quadratic_curve_with_alpha(screen,p,a,color,0xff,false,None)
}

/// with_alpha is with alpha channel,antialias flag,draw size (only is_antialias = true)
pub fn quadratic_curve_with_alpha(screen:&mut dyn Screen,p:Vec<(f32,f32)>,a:f32,color: u32,alpha: u8,is_antialias:bool,size: Option<f32>) {
    let s = if let Some(_s) = size {
       _s
    } else { 1.0 };
    if p.len() == 0 {
        return
    }
    if p.len() == 1 {
        if is_antialias {
            return point_antialias(screen,p[0].0,p[0].1,color,alpha,s)
        } else {
            return point_with_alpha(screen,p[0].0 as i32,p[0].1 as i32,color,alpha)
        }
    }
    if p.len() == 2 {
        if is_antialias {
            return line_antialias(screen,p[0].0 ,p[0].1 ,p[1].0 ,p[1].1 ,color,alpha,s)
        } else {
            return line_with_alpha(screen,p[0].0 as i32,p[0].1 as i32,p[1].0 as i32,p[1].1 as i32,color,alpha)
        }
    }

    for i in 0..p.len() -2 {
        // also worst case 
        let dt = (p[i].0 - p[i+1].0).abs() + (p[i+1].0 - p[i+2].0).abs()
               + (p[i].1 - p[i+1].1).abs() + (p[i+1].1 - p[i+2].1).abs();

        let mut pp = p[i];
        for ti in 0..dt as usize + 1 {
            let t = ti as f32 / dt;
            let s1 = (1.0  -  t) * (1.0  -  t - t);
            let s2 = (a + 4.0) * t * (1.0 - t);
            let s3 = t * (t + t - 1.0);
            let sn = 1.0 / (s1 + s2 + s3);
            let x = (s1 * p[i].0 + s2* p[i+1].0 + s3 * p[i+2].0) * sn;
            let y = (s1 * p[i].1 + s2* p[i+1].1 + s3 * p[i+2].1) * sn;
            if pp.0 as i32 == x as i32 && pp.1 as i32 == y as i32 {
                pp = (x,y);
                continue;
            }
            if is_antialias {
                line_antialias(screen,pp.0 ,pp.1 , x , y , color, alpha,s);
            } else {
                line_with_alpha(screen,pp.0 as i32,pp.1 as i32, x as i32, y as  i32, color, alpha);
            }
            pp = (x,y);
        }
    }
}

fn pascal_triangle(n:usize) -> Vec::<i32>{
    if n == 0 {return vec![1]}
    if n == 1 {return vec![1,1]}
    let mut p :Vec<Vec<i32>> = vec![vec![0,1,0]];

    for i in 1..n+1 {
        let mut row : Vec<i32> = Vec::new();
        row.push(0);
         for j in 0..i+1 {
            row.push(p[i-1][j] + p[i-1][i-j]);
        }
        row.push(0);
        p.push(row);
    }
    let ret = p.pop().unwrap();
    ret[1..ret.len()-1].to_vec()
}

/// draw n bezier curve
/// - p = [x,y].to_vec() -> point
/// - p = [[x0,y0],[x1,y1]].to_vec() Linear Bézier curves = Strait line
/// - p = [[x0,y0],[x1,y1],[x2,y2]].to_vec() Quadratic Bézier curve
/// - p = [[x0,y0],[x1,y1],[x2,y2],[x3,y3]].to_vec() Cubic Bézier curve
///     and Poly Bézier curves
pub fn bezier_curve(screen:&mut dyn Screen,p:Vec<(f32,f32)>,color: u32) {
    bezier_curve_with_alpha(screen,p,color,0xff,false,None)
}

pub fn bezier_curve_with_alpha(screen:&mut dyn Screen,p:Vec<(f32,f32)>,color: u32,alpha: u8,is_antialias:bool,size:Option<f32>) {
    let s = if let Some(_s) = size {
        _s
     } else { 1.0 };
    let n = p.len() - 1;
    if p.len() < 1 {
        return
    }

    let mut dt = 0.0;
    for i in 0..n{
        dt += (p[i].0 - p[i+1].0).abs() + (p[i].1 - p[i+1].1).abs();
    }

    let cn = pascal_triangle(n);

    let mut pp = (p[0].0,p[0].1);
    for ti in 0..dt as usize + 1 {
        let t = ti as f32 / dt as f32;
        let mut bx = 0.0;
        let mut by = 0.0;

        for i in 0..n+1 {
            let c = cn[i] as f32;
            let j = t.powi(i as i32) * (1.0- t).powi((n - i) as i32);
            bx += c * j * p[i].0;
            by += c * j * p[i].1;
        }

        if pp.0 as i32 == bx as i32 && pp.1 as i32 == by as i32 {
            pp = (bx,by);
            continue;
        }
        if is_antialias {
            line_antialias(screen,pp.0 ,pp.1 ,bx ,by ,color,alpha,s)
        } else {
            line_with_alpha(screen,pp.0 as i32,pp.1 as i32, bx as i32, by as  i32, color,alpha);
        }
        pp = (bx,by);
    }
}