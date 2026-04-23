//! spline module draws curve quadratic curve,bezier curve.

use crate::canvas::*;
use crate::line::*;
use crate::point::*;

const CURVE_STEP_PIXELS: f32 = 6.0;
const CURVE_MAX_STEPS: usize = 1024;

fn distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    (dx * dx + dy * dy).sqrt()
}

fn polyline_length(points: &[(f32, f32)]) -> f32 {
    points
        .windows(2)
        .map(|points| distance(points[0], points[1]))
        .sum()
}

fn curve_steps(points: &[(f32, f32)]) -> usize {
    let length = polyline_length(points);
    ((length / CURVE_STEP_PIXELS).ceil() as usize)
        .max(1)
        .min(CURVE_MAX_STEPS)
}

fn draw_curve_segment(
    screen: &mut dyn Screen,
    previous: &mut (f32, f32),
    point: (f32, f32),
    color: u32,
    alpha: u8,
    is_antialias: bool,
    size: f32,
) {
    if previous.0 as i32 == point.0 as i32 && previous.1 as i32 == point.1 as i32 {
        *previous = point;
        return;
    }
    if is_antialias {
        line_antialias(
            screen, previous.0, previous.1, point.0, point.1, color, alpha, size,
        );
    } else {
        line_with_alpha(
            screen,
            previous.0 as i32,
            previous.1 as i32,
            point.0 as i32,
            point.1 as i32,
            color,
            alpha,
        );
    }
    *previous = point;
}

/// draw quadratic curve
/// Parameter a changes half circle, ellipse, parabola or hyperbola
/// Circle or ellipse is a = -2.0
pub fn quadratic_curve(screen: &mut dyn Screen, p: Vec<(f32, f32)>, a: f32, color: u32) {
    quadratic_curve_with_alpha(screen, p, a, color, 0xff, false, None)
}

/// with_alpha is with alpha channel,antialias flag,draw size (only is_antialias = true)
pub fn quadratic_curve_with_alpha(
    screen: &mut dyn Screen,
    p: Vec<(f32, f32)>,
    a: f32,
    color: u32,
    alpha: u8,
    is_antialias: bool,
    size: Option<f32>,
) {
    let s = if let Some(_s) = size { _s } else { 1.0 };
    if p.is_empty() {
        return;
    }
    if p.len() == 1 {
        if is_antialias {
            return point_antialias(screen, p[0].0, p[0].1, color, alpha, s);
        } else {
            return point_with_alpha(screen, p[0].0 as i32, p[0].1 as i32, color, alpha);
        }
    }
    if p.len() == 2 {
        if is_antialias {
            return line_antialias(screen, p[0].0, p[0].1, p[1].0, p[1].1, color, alpha, s);
        } else {
            return line_with_alpha(
                screen,
                p[0].0 as i32,
                p[0].1 as i32,
                p[1].0 as i32,
                p[1].1 as i32,
                color,
                alpha,
            );
        }
    }

    for i in 0..p.len() - 2 {
        // also worst case
        let dt = (p[i].0 - p[i + 1].0).abs()
            + (p[i + 1].0 - p[i + 2].0).abs()
            + (p[i].1 - p[i + 1].1).abs()
            + (p[i + 1].1 - p[i + 2].1).abs();

        let mut pp = p[i];
        for ti in 0..dt as usize + 1 {
            let t = ti as f32 / dt;
            let s1 = (1.0 - t) * (1.0 - t - t);
            let s2 = (a + 4.0) * t * (1.0 - t);
            let s3 = t * (t + t - 1.0);
            let sn = 1.0 / (s1 + s2 + s3);
            let x = (s1 * p[i].0 + s2 * p[i + 1].0 + s3 * p[i + 2].0) * sn;
            let y = (s1 * p[i].1 + s2 * p[i + 1].1 + s3 * p[i + 2].1) * sn;
            if pp.0 as i32 == x as i32 && pp.1 as i32 == y as i32 {
                pp = (x, y);
                continue;
            }
            if is_antialias {
                line_antialias(screen, pp.0, pp.1, x, y, color, alpha, s);
            } else {
                line_with_alpha(
                    screen,
                    pp.0 as i32,
                    pp.1 as i32,
                    x as i32,
                    y as i32,
                    color,
                    alpha,
                );
            }
            pp = (x, y);
        }
    }
}

fn pascal_triangle(n: usize) -> Vec<i32> {
    if n == 0 {
        return vec![1];
    }
    if n == 1 {
        return vec![1, 1];
    }
    let mut p: Vec<Vec<i32>> = vec![vec![0, 1, 0]];

    for i in 1..n + 1 {
        let mut row: Vec<i32> = Vec::new();
        row.push(0);
        for j in 0..i + 1 {
            row.push(p[i - 1][j] + p[i - 1][i - j]);
        }
        row.push(0);
        p.push(row);
    }
    let ret = p.pop().unwrap();
    ret[1..ret.len() - 1].to_vec()
}

fn draw_quadratic_bezier(
    screen: &mut dyn Screen,
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    color: u32,
    alpha: u8,
    is_antialias: bool,
    size: f32,
) {
    let steps = curve_steps(&[p0, p1, p2]);
    let dt = 1.0 / steps as f32;
    let dt_sq = dt * dt;

    let ax = p0.0 - 2.0 * p1.0 + p2.0;
    let ay = p0.1 - 2.0 * p1.1 + p2.1;
    let bx = 2.0 * (p1.0 - p0.0);
    let by = 2.0 * (p1.1 - p0.1);

    let mut point = p0;
    let mut delta = (ax * dt_sq + bx * dt, ay * dt_sq + by * dt);
    let delta_delta = (2.0 * ax * dt_sq, 2.0 * ay * dt_sq);
    let mut previous = p0;

    for _ in 1..steps {
        point = (point.0 + delta.0, point.1 + delta.1);
        delta = (delta.0 + delta_delta.0, delta.1 + delta_delta.1);
        draw_curve_segment(
            screen,
            &mut previous,
            point,
            color,
            alpha,
            is_antialias,
            size,
        );
    }
    draw_curve_segment(screen, &mut previous, p2, color, alpha, is_antialias, size);
}

fn draw_cubic_bezier(
    screen: &mut dyn Screen,
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    color: u32,
    alpha: u8,
    is_antialias: bool,
    size: f32,
) {
    let steps = curve_steps(&[p0, p1, p2, p3]);
    let dt = 1.0 / steps as f32;

    let ax = -p0.0 + 3.0 * p1.0 - 3.0 * p2.0 + p3.0;
    let ay = -p0.1 + 3.0 * p1.1 - 3.0 * p2.1 + p3.1;
    let bx = 3.0 * p0.0 - 6.0 * p1.0 + 3.0 * p2.0;
    let by = 3.0 * p0.1 - 6.0 * p1.1 + 3.0 * p2.1;
    let cx = 3.0 * (p1.0 - p0.0);
    let cy = 3.0 * (p1.1 - p0.1);

    let dt_sq = dt * dt;
    let dt_cu = dt_sq * dt;
    let mut point = p0;
    let mut delta = (
        ax * dt_cu + bx * dt_sq + cx * dt,
        ay * dt_cu + by * dt_sq + cy * dt,
    );
    let mut delta_delta = (
        6.0 * ax * dt_cu + 2.0 * bx * dt_sq,
        6.0 * ay * dt_cu + 2.0 * by * dt_sq,
    );
    let delta_delta_delta = (6.0 * ax * dt_cu, 6.0 * ay * dt_cu);
    let mut previous = p0;

    for _ in 1..steps {
        point = (point.0 + delta.0, point.1 + delta.1);
        delta = (delta.0 + delta_delta.0, delta.1 + delta_delta.1);
        delta_delta = (
            delta_delta.0 + delta_delta_delta.0,
            delta_delta.1 + delta_delta_delta.1,
        );
        draw_curve_segment(
            screen,
            &mut previous,
            point,
            color,
            alpha,
            is_antialias,
            size,
        );
    }
    draw_curve_segment(screen, &mut previous, p3, color, alpha, is_antialias, size);
}

/// draw n bezier curve
/// - p = [x,y].to_vec() -> point
/// - p = [[x0,y0],[x1,y1]].to_vec() Linear Bézier curves = Strait line
/// - p = [[x0,y0],[x1,y1],[x2,y2]].to_vec() Quadratic Bézier curve
/// - p = [[x0,y0],[x1,y1],[x2,y2],[x3,y3]].to_vec() Cubic Bézier curve
///     and Poly Bézier curves
pub fn bezier_curve(screen: &mut dyn Screen, p: Vec<(f32, f32)>, color: u32) {
    bezier_curve_with_alpha(screen, p, color, 0xff, false, None)
}

pub fn bezier_curve_with_alpha(
    screen: &mut dyn Screen,
    p: Vec<(f32, f32)>,
    color: u32,
    alpha: u8,
    is_antialias: bool,
    size: Option<f32>,
) {
    let s = if let Some(_s) = size { _s } else { 1.0 };
    if p.is_empty() {
        return;
    }
    let n = p.len() - 1;
    if p.len() == 1 {
        if is_antialias {
            return point_antialias(screen, p[0].0, p[0].1, color, alpha, s);
        } else {
            return point_with_alpha(screen, p[0].0 as i32, p[0].1 as i32, color, alpha);
        }
    }
    if p.len() == 2 {
        if is_antialias {
            return line_antialias(screen, p[0].0, p[0].1, p[1].0, p[1].1, color, alpha, s);
        } else {
            return line_with_alpha(
                screen,
                p[0].0 as i32,
                p[0].1 as i32,
                p[1].0 as i32,
                p[1].1 as i32,
                color,
                alpha,
            );
        }
    }
    if p.len() == 3 {
        return draw_quadratic_bezier(screen, p[0], p[1], p[2], color, alpha, is_antialias, s);
    }
    if p.len() == 4 {
        return draw_cubic_bezier(
            screen,
            p[0],
            p[1],
            p[2],
            p[3],
            color,
            alpha,
            is_antialias,
            s,
        );
    }

    let mut dt = 0.0;
    for i in 0..n {
        dt += (p[i].0 - p[i + 1].0).abs() + (p[i].1 - p[i + 1].1).abs();
    }

    let cn = pascal_triangle(n);

    let mut pp = (p[0].0, p[0].1);
    for ti in 0..dt as usize + 1 {
        let t = ti as f32 / dt;
        let mut bx = 0.0;
        let mut by = 0.0;

        for i in 0..n + 1 {
            let c = cn[i] as f32;
            let j = t.powi(i as i32) * (1.0 - t).powi((n - i) as i32);
            bx += c * j * p[i].0;
            by += c * j * p[i].1;
        }

        if pp.0 as i32 == bx as i32 && pp.1 as i32 == by as i32 {
            pp = (bx, by);
            continue;
        }
        if is_antialias {
            line_antialias(screen, pp.0, pp.1, bx, by, color, alpha, s)
        } else {
            line_with_alpha(
                screen,
                pp.0 as i32,
                pp.1 as i32,
                bx as i32,
                by as i32,
                color,
                alpha,
            );
        }
        pp = (bx, by);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::Canvas;

    #[test]
    fn bezier_curve_accepts_empty_points() {
        let mut canvas = Canvas::new(8, 8);
        bezier_curve(&mut canvas, Vec::new(), 0xffff_ffff);
    }

    #[test]
    fn bezier_curve_draws_quadratic_and_cubic_endpoints() {
        let mut canvas = Canvas::new(32, 32);
        bezier_curve(
            &mut canvas,
            vec![(2.0, 2.0), (12.0, 24.0), (24.0, 2.0)],
            0xffff_ffff,
        );
        bezier_curve(
            &mut canvas,
            vec![(2.0, 28.0), (8.0, 8.0), (22.0, 8.0), (28.0, 28.0)],
            0xffff_ffff,
        );

        assert!(canvas.buffer().chunks_exact(4).any(|pixel| pixel[3] != 0));
    }
}
