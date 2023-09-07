use crate::{line, prelude::Screen, spline};

#[derive(Debug, Clone)]
pub enum Command {
    Line(f32, f32),
    MoveTo(f32, f32),
    Bezier((f32, f32), (f32, f32)),
    CubicBezier((f32, f32), (f32, f32), (f32, f32)),
    Close,
}

pub fn draw(screen: &mut dyn Screen, commands: &Vec<Command>, color: u32) {
    let mut current_point = (0.0, 0.0);
    let mut start_point = (0.0, 0.0);
    for command in commands.iter() {
        match command {
            Command::Line(ex, ey) => {
                let x0 = current_point.0 as i32;
                let y0 = current_point.1 as i32;
                let x1 = *ex as i32;
                let y1 = *ey as i32;
                line::line(screen, x0, y0, x1, y1, color);
                current_point = (*ex, *ey);
            }
            Command::MoveTo(x, f) => {
                current_point = (*x, *f);
                start_point = (*x, *f);
            }
            Command::Bezier(control, end) => {
                let mut points = Vec::new();
                points.push(current_point);
                points.push(*control);
                points.push(*end);
                spline::bezier_curve(screen, points, color);
                current_point = *end;
            }
            Command::CubicBezier(control1, control2, end) => {
                let mut points = Vec::new();
                points.push(current_point);
                points.push(*control1);
                points.push(*control2);
                points.push(*end);
                spline::bezier_curve(screen, points, color);
                current_point = *end;
            }

            Command::Close => {
                let x0 = current_point.0 as i32;
                let y0 = current_point.1 as i32;
                let x1 = start_point.0 as i32;
                let y1 = start_point.1 as i32;
                line::line(screen, x0, y0, x1, y1, color);
            }
        }
    }
}
