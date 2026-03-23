//! Path and glyph rendering helpers.
//! (c) 2023-2026 by Mith@mmk

type Error = Box<dyn std::error::Error>;

use crate::{
    affine::{Affine, InterpolationAlgorithm},
    draw::draw_over_screen_with_alpha,
    error::Error as PaintError,
    image::{self, ImageAlign},
    layer::Layer,
    line,
    prelude::Screen,
    spline,
    utils::color_taple,
};
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub enum Command {
    Line(f32, f32),
    MoveTo(f32, f32),
    Bezier((f32, f32), (f32, f32)),
    CubicBezier((f32, f32), (f32, f32), (f32, f32)),
    Close,
}

/// Text advance direction resolved by the font parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphFlow {
    Horizontal,
    Vertical,
}

/// Font-level metrics. Keep this on the glyph so mixed fallback fonts can coexist in one run.
#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub flow: GlyphFlow,
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

/// Glyph metrics after the font parser has resolved units and orientation.
#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    pub advance_x: f32,
    pub advance_y: f32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub bounds: Option<GlyphBounds>,
}

impl Default for GlyphMetrics {
    fn default() -> Self {
        Self {
            advance_x: 0.0,
            advance_y: 0.0,
            bearing_x: 0.0,
            bearing_y: 0.0,
            bounds: None,
        }
    }
}

/// Paint for vector glyph layers. `CurrentColor` maps to the color passed into `draw_glyphs`.
#[derive(Debug, Clone, Copy)]
pub enum GlyphPaint {
    Solid(u32),
    CurrentColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

/// Vector glyph layer.
///
/// This is used for normal outline fonts and for SVG emoji after the SVG has been converted
/// into path commands by the font parser.
#[derive(Debug, Clone)]
pub struct PathGlyphLayer {
    pub commands: Vec<Command>,
    pub paint: GlyphPaint,
    pub fill_rule: FillRule,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl PathGlyphLayer {
    pub fn new(commands: Vec<Command>, paint: GlyphPaint) -> Self {
        Self {
            commands,
            paint,
            fill_rule: FillRule::NonZero,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

/// Raster glyph payload.
///
/// `Encoded` is decoded through the existing image loader, which already covers PNG.
#[derive(Debug, Clone)]
pub enum RasterGlyphSource {
    Encoded(Vec<u8>),
    Rgba {
        width: u32,
        height: u32,
        data: Vec<u8>,
    },
}

#[derive(Clone)]
pub struct RasterGlyphLayer {
    pub source: RasterGlyphSource,
    pub offset_x: f32,
    pub offset_y: f32,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub interpolation: InterpolationAlgorithm,
}

impl RasterGlyphLayer {
    pub fn from_encoded(data: Vec<u8>) -> Self {
        Self {
            source: RasterGlyphSource::Encoded(data),
            offset_x: 0.0,
            offset_y: 0.0,
            width: None,
            height: None,
            interpolation: InterpolationAlgorithm::Bilinear,
        }
    }

    pub fn from_rgba(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            source: RasterGlyphSource::Rgba {
                width,
                height,
                data,
            },
            offset_x: 0.0,
            offset_y: 0.0,
            width: None,
            height: None,
            interpolation: InterpolationAlgorithm::Bilinear,
        }
    }
}

/// Extensible glyph layer model.
///
/// - `Path`: monochrome outlines and SVG emoji vector layers.
/// - `Raster`: PNG bitmap emoji and other image-based glyph layers.
#[derive(Clone)]
pub enum GlyphLayer {
    Path(PathGlyphLayer),
    Raster(RasterGlyphLayer),
}

/// A single glyph as prepared by the font parser.
///
/// Coordinates in each layer are already in screen space relative to the glyph origin.
/// Metrics are preserved for layout and future extensions, but drawing only uses the resolved
/// positioned origin plus the layer offsets.
#[derive(Clone)]
pub struct Glyph {
    pub font: Option<FontMetrics>,
    pub metrics: GlyphMetrics,
    pub layers: Vec<GlyphLayer>,
}

impl Glyph {
    pub fn new(layers: Vec<GlyphLayer>) -> Self {
        Self {
            font: None,
            metrics: GlyphMetrics::default(),
            layers,
        }
    }
}

#[derive(Clone)]
pub struct PositionedGlyph {
    pub glyph: Glyph,
    pub x: f32,
    pub y: f32,
}

impl PositionedGlyph {
    pub fn new(glyph: Glyph, x: f32, y: f32) -> Self {
        Self { glyph, x, y }
    }
}

#[derive(Clone, Default)]
pub struct GlyphRun {
    pub glyphs: Vec<PositionedGlyph>,
}

impl GlyphRun {
    pub fn new(glyphs: Vec<PositionedGlyph>) -> Self {
        Self { glyphs }
    }
}

#[derive(Debug, Clone, Copy)]
struct Edge {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
    winding: i32,
}

fn paint_error(message: &str) -> Error {
    Box::new(PaintError {
        message: message.to_string(),
    })
}

fn normalize_paint_color(color: u32) -> u32 {
    if color <= 0x00ff_ffff {
        0xff00_0000 | color
    } else {
        color
    }
}

fn resolve_paint(paint: GlyphPaint, default_color: u32) -> u32 {
    match paint {
        GlyphPaint::Solid(color) => normalize_paint_color(color),
        GlyphPaint::CurrentColor => normalize_paint_color(default_color),
    }
}

fn push_point(points: &mut Vec<(f32, f32)>, point: (f32, f32)) {
    const EPSILON: f32 = 0.001;
    if let Some(last) = points.last() {
        if (last.0 - point.0).abs() <= EPSILON && (last.1 - point.1).abs() <= EPSILON {
            return;
        }
    }
    points.push(point);
}

fn curve_steps(points: &[(f32, f32)]) -> usize {
    let mut length = 0.0;
    for i in 0..points.len().saturating_sub(1) {
        length += (points[i].0 - points[i + 1].0).abs() + (points[i].1 - points[i + 1].1).abs();
    }

    length.ceil().clamp(8.0, 2048.0) as usize
}

fn quadratic_point(start: (f32, f32), control: (f32, f32), end: (f32, f32), t: f32) -> (f32, f32) {
    let mt = 1.0 - t;
    (
        mt * mt * start.0 + 2.0 * mt * t * control.0 + t * t * end.0,
        mt * mt * start.1 + 2.0 * mt * t * control.1 + t * t * end.1,
    )
}

fn cubic_point(
    start: (f32, f32),
    control1: (f32, f32),
    control2: (f32, f32),
    end: (f32, f32),
    t: f32,
) -> (f32, f32) {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;
    (
        mt2 * mt * start.0
            + 3.0 * mt2 * t * control1.0
            + 3.0 * mt * t2 * control2.0
            + t2 * t * end.0,
        mt2 * mt * start.1
            + 3.0 * mt2 * t * control1.1
            + 3.0 * mt * t2 * control2.1
            + t2 * t * end.1,
    )
}

fn flush_contour(contours: &mut Vec<Vec<(f32, f32)>>, contour: &mut Vec<(f32, f32)>) {
    if contour.len() < 2 {
        contour.clear();
        return;
    }
    if contour.first() != contour.last() {
        let first = contour[0];
        contour.push(first);
    }
    contours.push(std::mem::take(contour));
}

fn flatten_commands(commands: &[Command], offset_x: f32, offset_y: f32) -> Vec<Vec<(f32, f32)>> {
    let mut contours = Vec::new();
    let mut contour = Vec::new();
    let mut current_point = None;
    let mut start_point = None;

    for command in commands {
        match command {
            Command::MoveTo(x, y) => {
                flush_contour(&mut contours, &mut contour);
                let point = (x + offset_x, y + offset_y);
                contour.push(point);
                current_point = Some(point);
                start_point = Some(point);
            }
            Command::Line(x, y) => {
                if let Some(current) = current_point {
                    let point = (x + offset_x, y + offset_y);
                    if contour.is_empty() {
                        contour.push(current);
                    }
                    push_point(&mut contour, point);
                    current_point = Some(point);
                }
            }
            Command::Bezier(control, end) => {
                if let Some(start) = current_point {
                    let control = (control.0 + offset_x, control.1 + offset_y);
                    let end = (end.0 + offset_x, end.1 + offset_y);
                    if contour.is_empty() {
                        contour.push(start);
                    }
                    let steps = curve_steps(&[start, control, end]);
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        push_point(&mut contour, quadratic_point(start, control, end, t));
                    }
                    current_point = Some(end);
                }
            }
            Command::CubicBezier(control1, control2, end) => {
                if let Some(start) = current_point {
                    let control1 = (control1.0 + offset_x, control1.1 + offset_y);
                    let control2 = (control2.0 + offset_x, control2.1 + offset_y);
                    let end = (end.0 + offset_x, end.1 + offset_y);
                    if contour.is_empty() {
                        contour.push(start);
                    }
                    let steps = curve_steps(&[start, control1, control2, end]);
                    for i in 1..=steps {
                        let t = i as f32 / steps as f32;
                        push_point(&mut contour, cubic_point(start, control1, control2, end, t));
                    }
                    current_point = Some(end);
                }
            }
            Command::Close => {
                if let Some(start) = start_point {
                    if contour.first() != Some(&start) {
                        contour.insert(0, start);
                    }
                }
                flush_contour(&mut contours, &mut contour);
                current_point = start_point;
                start_point = None;
            }
        }
    }

    flush_contour(&mut contours, &mut contour);
    contours
}

fn contour_bounds(contours: &[Vec<(f32, f32)>]) -> Option<GlyphBounds> {
    let mut iter = contours.iter().flat_map(|contour| contour.iter().copied());
    let first = iter.next()?;

    let mut bounds = GlyphBounds {
        min_x: first.0,
        min_y: first.1,
        max_x: first.0,
        max_y: first.1,
    };

    for (x, y) in iter {
        bounds.min_x = bounds.min_x.min(x);
        bounds.min_y = bounds.min_y.min(y);
        bounds.max_x = bounds.max_x.max(x);
        bounds.max_y = bounds.max_y.max(y);
    }

    Some(bounds)
}

fn contour_edges(contours: &[Vec<(f32, f32)>]) -> Vec<Edge> {
    let mut edges = Vec::new();

    for contour in contours {
        for window in contour.windows(2) {
            let (x0, y0) = window[0];
            let (x1, y1) = window[1];
            if (y0 - y1).abs() <= f32::EPSILON {
                continue;
            }
            let winding = if y1 > y0 { 1 } else { -1 };
            edges.push(Edge {
                x0,
                y0,
                x1,
                y1,
                winding,
            });
        }
    }

    edges
}

fn fill_span(screen: &mut dyn Screen, start_x: f32, end_x: f32, y: i32, color: u32) {
    if start_x >= end_x {
        return;
    }

    let width = screen.width() as i32;
    let start = (start_x - 0.5).ceil() as i32;
    let end = (end_x - 0.5).floor() as i32;
    if width == 0 || start > end {
        return;
    }

    let start = start.clamp(0, width - 1);
    let end = end.clamp(0, width - 1);
    if start > end {
        return;
    }

    let (_, _, _, alpha) = color_taple(color);
    if alpha == 0 {
        return;
    }

    line::line_with_alpha(screen, start, y, end, y, color & 0x00ff_ffff, alpha);
}

fn fill_contours(
    screen: &mut dyn Screen,
    contours: &[Vec<(f32, f32)>],
    color: u32,
    rule: FillRule,
) {
    if screen.width() == 0 || screen.height() == 0 || contours.is_empty() {
        return;
    }

    let Some(bounds) = contour_bounds(contours) else {
        return;
    };

    let edges = contour_edges(contours);
    if edges.is_empty() {
        return;
    }

    let height = screen.height() as i32;
    let start_y = ((bounds.min_y - 0.5).ceil() as i32).clamp(0, height.saturating_sub(1));
    let end_y = ((bounds.max_y - 0.5).floor() as i32).clamp(0, height.saturating_sub(1));

    for y in start_y..=end_y {
        let scan_y = y as f32 + 0.5;
        let mut intersections = Vec::new();

        for edge in &edges {
            let y_min = edge.y0.min(edge.y1);
            let y_max = edge.y0.max(edge.y1);
            if scan_y < y_min || scan_y >= y_max {
                continue;
            }

            let t = (scan_y - edge.y0) / (edge.y1 - edge.y0);
            let x = edge.x0 + (edge.x1 - edge.x0) * t;
            intersections.push((x, edge.winding));
        }

        if intersections.is_empty() {
            continue;
        }

        intersections.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

        match rule {
            FillRule::EvenOdd => {
                let mut i = 0;
                while i + 1 < intersections.len() {
                    fill_span(screen, intersections[i].0, intersections[i + 1].0, y, color);
                    i += 2;
                }
            }
            FillRule::NonZero => {
                let mut grouped: Vec<(f32, i32)> = Vec::with_capacity(intersections.len());
                for (x, delta) in intersections {
                    if let Some(last) = grouped.last_mut() {
                        if (last.0 - x).abs() <= 0.001 {
                            last.1 += delta;
                            continue;
                        }
                    }
                    grouped.push((x, delta));
                }

                let mut winding = 0;
                let mut start_x = None;
                for (x, delta) in grouped {
                    let previous = winding;
                    winding += delta;
                    if previous == 0 && winding != 0 {
                        start_x = Some(x);
                    } else if previous != 0 && winding == 0 {
                        if let Some(start_x) = start_x.take() {
                            fill_span(screen, start_x, x, y, color);
                        }
                    }
                }
            }
        }
    }
}

fn decode_raster(source: &RasterGlyphSource) -> Result<Layer, Error> {
    match source {
        RasterGlyphSource::Encoded(data) => {
            let mut layer = Layer::tmp(0, 0);
            image::draw_image(&mut layer, data, 0)?;
            Ok(layer)
        }
        RasterGlyphSource::Rgba {
            width,
            height,
            data,
        } => {
            let expected = (*width as usize)
                .checked_mul(*height as usize)
                .and_then(|pixels| pixels.checked_mul(4))
                .ok_or_else(|| paint_error("rgba glyph buffer size overflow"))?;

            if data.len() != expected {
                return Err(paint_error("rgba glyph buffer size mismatch"));
            }

            Ok(Layer::new_in(
                "_glyph_raster_".to_string(),
                data.clone(),
                *width,
                *height,
            ))
        }
    }
}

fn scaled_size(
    source_width: u32,
    source_height: u32,
    width: Option<u32>,
    height: Option<u32>,
) -> (u32, u32) {
    match (width, height) {
        (Some(width), Some(height)) => (width.max(1), height.max(1)),
        (Some(width), None) => {
            let height =
                ((source_height as f32 * width as f32) / source_width as f32).round() as u32;
            (width.max(1), height.max(1))
        }
        (None, Some(height)) => {
            let width =
                ((source_width as f32 * height as f32) / source_height as f32).round() as u32;
            (width.max(1), height.max(1))
        }
        (None, None) => (source_width.max(1), source_height.max(1)),
    }
}

fn scale_raster(
    source: &Layer,
    width: u32,
    height: u32,
    interpolation: InterpolationAlgorithm,
) -> Layer {
    if source.width() == width && source.height() == height {
        return Layer::new_in(
            "_glyph_raster_scaled_".to_string(),
            source.buffer().to_vec(),
            source.width(),
            source.height(),
        );
    }

    let mut target = Layer::tmp(width, height);
    let scale_x = width as f32 / source.width() as f32;
    let scale_y = height as f32 / source.height() as f32;

    if (scale_x - scale_y).abs() <= f32::EPSILON {
        Affine::resize(
            source,
            &mut target,
            scale_x,
            interpolation,
            ImageAlign::LeftUp,
        );
        return target;
    }

    let mut affine = Affine::new();
    affine.scale(scale_x, scale_y);
    affine.conversion_with_area_center(
        source,
        &mut target,
        0.0,
        0.0,
        source.width() as f32,
        source.height() as f32,
        0,
        0,
        width as i32,
        height as i32,
        0.0,
        0.0,
        interpolation,
    );

    target
}

fn draw_path_layer(
    screen: &mut dyn Screen,
    layer: &PathGlyphLayer,
    origin_x: f32,
    origin_y: f32,
    default_color: u32,
) {
    let contours = flatten_commands(
        &layer.commands,
        origin_x + layer.offset_x,
        origin_y + layer.offset_y,
    );
    let color = resolve_paint(layer.paint, default_color);
    fill_contours(screen, &contours, color, layer.fill_rule);
}

fn draw_raster_layer(
    screen: &mut dyn Screen,
    layer: &RasterGlyphLayer,
    origin_x: f32,
    origin_y: f32,
) -> Result<(), Error> {
    let source = decode_raster(&layer.source)?;
    let (target_width, target_height) =
        scaled_size(source.width(), source.height(), layer.width, layer.height);
    let raster = scale_raster(
        &source,
        target_width,
        target_height,
        layer.interpolation.clone(),
    );

    draw_over_screen_with_alpha(
        &raster,
        screen,
        (origin_x + layer.offset_x).round() as i32,
        (origin_y + layer.offset_y).round() as i32,
    );
    Ok(())
}

pub fn draw_glyph(
    screen: &mut dyn Screen,
    glyph: &PositionedGlyph,
    offset_x: f32,
    offset_y: f32,
    default_color: u32,
) -> Result<(), Error> {
    let origin_x = glyph.x + offset_x;
    let origin_y = glyph.y + offset_y;

    for layer in &glyph.glyph.layers {
        match layer {
            GlyphLayer::Path(layer) => {
                draw_path_layer(screen, layer, origin_x, origin_y, default_color)
            }
            GlyphLayer::Raster(layer) => draw_raster_layer(screen, layer, origin_x, origin_y)?,
        }
    }

    Ok(())
}

/// Draws parsed glyphs prepared by the font parser.
///
/// The parser is responsible for:
/// - converting font units into pixel space
/// - resolving fallback fonts
/// - converting SVG glyph payloads into `PathGlyphLayer`s
/// - leaving PNG or other bitmap glyph payloads as `RasterGlyphLayer`s
pub fn draw_glyphs(
    screen: &mut dyn Screen,
    glyphs: &GlyphRun,
    offset_x: f32,
    offset_y: f32,
    default_color: u32,
) -> Result<(), Error> {
    for glyph in &glyphs.glyphs {
        draw_glyph(screen, glyph, offset_x, offset_y, default_color)?;
    }
    Ok(())
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
            Command::MoveTo(x, y) => {
                current_point = (*x, *y);
                start_point = (*x, *y);
            }
            Command::Bezier(control, end) => {
                let points = vec![current_point, *control, *end];
                spline::bezier_curve(screen, points, color);
                current_point = *end;
            }
            Command::CubicBezier(control1, control2, end) => {
                let points = vec![current_point, *control1, *control2, *end];
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::Canvas;

    fn rgba(screen: &dyn Screen, x: u32, y: u32) -> [u8; 4] {
        let offset = ((y * screen.width() + x) * 4) as usize;
        [
            screen.buffer()[offset],
            screen.buffer()[offset + 1],
            screen.buffer()[offset + 2],
            screen.buffer()[offset + 3],
        ]
    }

    #[test]
    fn draw_glyphs_fills_nonzero_path_with_hole() {
        let commands = vec![
            Command::MoveTo(1.0, 1.0),
            Command::Line(8.0, 1.0),
            Command::Line(8.0, 8.0),
            Command::Line(1.0, 8.0),
            Command::Close,
            Command::MoveTo(3.0, 3.0),
            Command::Line(3.0, 6.0),
            Command::Line(6.0, 6.0),
            Command::Line(6.0, 3.0),
            Command::Close,
        ];

        let glyph = Glyph::new(vec![GlyphLayer::Path(PathGlyphLayer::new(
            commands,
            GlyphPaint::CurrentColor,
        ))]);
        let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 0.0, 0.0)]);
        let mut canvas = Canvas::new(10, 10);

        draw_glyphs(&mut canvas, &run, 0.0, 0.0, 0xff33_6699).unwrap();

        assert_eq!(rgba(&canvas, 2, 2), [0x33, 0x66, 0x99, 0xff]);
        assert_eq!(rgba(&canvas, 4, 4), [0x00, 0x00, 0x00, 0x00]);
        assert_eq!(rgba(&canvas, 0, 0), [0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn draw_glyphs_draws_rgba_raster_layers() {
        let raster = RasterGlyphLayer {
            source: RasterGlyphSource::Rgba {
                width: 2,
                height: 2,
                data: vec![
                    0xff, 0x00, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0x00, 0xff, 0xff, 0xff,
                    0xff, 0x00, 0x80,
                ],
            },
            offset_x: 1.0,
            offset_y: 2.0,
            width: None,
            height: None,
            interpolation: InterpolationAlgorithm::Bilinear,
        };

        let glyph = Glyph::new(vec![GlyphLayer::Raster(raster)]);
        let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 2.0, 1.0)]);
        let mut canvas = Canvas::new(8, 8);

        draw_glyphs(&mut canvas, &run, 0.0, 0.0, 0xffff_ffff).unwrap();

        assert_eq!(rgba(&canvas, 3, 3), [0xff, 0x00, 0x00, 0xff]);
        assert_eq!(rgba(&canvas, 4, 3), [0x00, 0xff, 0x00, 0xff]);
        assert_eq!(rgba(&canvas, 3, 4), [0x00, 0x00, 0xff, 0xff]);
        assert_eq!(rgba(&canvas, 4, 4), [0x80, 0x80, 0x00, 0xff]);
    }
}
