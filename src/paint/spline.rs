
use crate::paint::line::line_with_alpha;
use crate::Screen;
use crate::line;
use crate::paint::point::point;
use crate::Canvas;

pub fn quadratic_curve(screen:&mut dyn Screen,p:Vec<(f32,f32)>,a:f32,color: u32) {
    quadratic_curve_with_alpha(screen,p,a,color,0xff)
}

pub fn quadratic_curve_with_alpha(screen:&mut dyn Screen,p:Vec<(f32,f32)>,a:f32,color: u32,alpha: u8) {
    if p.len() == 0 {
        return
    }
    if p.len() == 1 {
        return point(screen,p[0].0 as i32,p[0].1 as i32,color)
    }
    if p.len() == 2 {
        return line(screen,p[0].0 as i32,p[0].1 as i32,p[1].0 as i32,p[1].1 as i32,color)
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
            line_with_alpha(screen,pp.0 as i32,pp.1 as i32, x as i32, y as  i32, color, alpha);
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

pub fn bezier_curve(screen:&mut Canvas,p:Vec<(f32,f32)>,color: u32) {
    bezier_curve_with_alpha(screen,p,color,0xff)
}

pub fn bezier_curve_with_alpha(screen:&mut Canvas,p:Vec<(f32,f32)>,color: u32,alpha: u8) {
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
        line_with_alpha(screen,pp.0 as i32,pp.1 as i32, bx as i32, by as  i32, color,alpha);
        pp = (bx,by);
    }
}