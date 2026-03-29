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

#[cfg(feature = "font")]
pub use fontloader::commands as commads;
#[cfg(feature = "font")]
pub use fontloader::{
    load_font_from_buffer, text2commands, Command, FillRule, FontFaceDescriptor, FontFamily,
    FontMetrics, FontOptions, FontRef, FontStretch, FontStyle, FontVariant, FontWeight, Glyph,
    GlyphBounds, GlyphCommands, GlyphFlow, GlyphLayer, GlyphMetrics, GlyphPaint, GlyphRun,
    LoadedFont, PathCommand, PathGlyphLayer, PositionedGlyph, RasterGlyphLayer,
    RasterGlyphSource,
};

#[cfg(not(feature = "font"))]
#[derive(Debug, Clone)]
pub enum Command {
    Line(f32, f32),
    MoveTo(f32, f32),
    Bezier((f32, f32), (f32, f32)),
    CubicBezier((f32, f32), (f32, f32), (f32, f32)),
    Close,
}

/// Text advance direction resolved by the font parser.
#[cfg(not(feature = "font"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlyphFlow {
    Horizontal,
    Vertical,
}

/// Font-level metrics. Keep this on the glyph so mixed fallback fonts can coexist in one run.
#[cfg(not(feature = "font"))]
#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub flow: GlyphFlow,
}

#[cfg(not(feature = "font"))]
#[derive(Debug, Clone, Copy)]
pub struct GlyphBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

/// Glyph metrics after the font parser has resolved units and orientation.
#[cfg(not(feature = "font"))]
#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    pub advance_x: f32,
    pub advance_y: f32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub bounds: Option<GlyphBounds>,
}

#[cfg(not(feature = "font"))]
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
#[cfg(not(feature = "font"))]
#[derive(Debug, Clone, Copy)]
pub enum GlyphPaint {
    Solid(u32),
    CurrentColor,
}

#[cfg(not(feature = "font"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

/// Vector glyph layer.
///
/// This is used for normal outline fonts and for SVG emoji after the SVG has been converted
/// into path commands by the font parser.
#[cfg(not(feature = "font"))]
#[derive(Debug, Clone)]
pub struct PathGlyphLayer {
    pub commands: Vec<Command>,
    pub paint: GlyphPaint,
    pub fill_rule: FillRule,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[cfg(not(feature = "font"))]
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
#[cfg(not(feature = "font"))]
#[derive(Debug, Clone)]
pub enum RasterGlyphSource {
    Encoded(Vec<u8>),
    Rgba {
        width: u32,
        height: u32,
        data: Vec<u8>,
    },
}

#[cfg(not(feature = "font"))]
#[derive(Clone)]
pub struct RasterGlyphLayer {
    pub source: RasterGlyphSource,
    pub offset_x: f32,
    pub offset_y: f32,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[cfg(not(feature = "font"))]
impl RasterGlyphLayer {
    pub fn from_encoded(data: Vec<u8>) -> Self {
        Self {
            source: RasterGlyphSource::Encoded(data),
            offset_x: 0.0,
            offset_y: 0.0,
            width: None,
            height: None,
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
        }
    }
}

/// Extensible glyph layer model.
///
/// - `Path`: monochrome outlines and SVG emoji vector layers.
/// - `Raster`: PNG bitmap emoji and other image-based glyph layers.
#[cfg(not(feature = "font"))]
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
#[cfg(not(feature = "font"))]
#[derive(Clone)]
pub struct Glyph {
    pub font: Option<FontMetrics>,
    pub metrics: GlyphMetrics,
    pub layers: Vec<GlyphLayer>,
}

#[cfg(not(feature = "font"))]
impl Glyph {
    pub fn new(layers: Vec<GlyphLayer>) -> Self {
        Self {
            font: None,
            metrics: GlyphMetrics::default(),
            layers,
        }
    }
}

#[cfg(not(feature = "font"))]
#[derive(Clone)]
pub struct PositionedGlyph {
    pub glyph: Glyph,
    pub x: f32,
    pub y: f32,
}

#[cfg(not(feature = "font"))]
impl PositionedGlyph {
    pub fn new(glyph: Glyph, x: f32, y: f32) -> Self {
        Self { glyph, x, y }
    }
}

#[cfg(not(feature = "font"))]
#[derive(Clone, Default)]
pub struct GlyphRun {
    pub glyphs: Vec<PositionedGlyph>,
}

#[cfg(not(feature = "font"))]
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

fn normalize_solid_color(color: u32) -> u32 {
    let color = normalize_paint_color(color);
    let alpha_hi = ((color >> 24) & 0xff) as u8;
    let alpha_lo = (color & 0xff) as u8;

    // `paintcore` colors are ARGB, but FontReader COLR/CPAL layers currently arrive as RGBA.
    // Keep explicit ARGB input intact and only reinterpret the unambiguous RGBA cases.
    if alpha_hi != 0x00 && alpha_hi != 0xff && (alpha_lo == 0x00 || alpha_lo == 0xff) {
        ((alpha_lo as u32) << 24) | (color >> 8)
    } else {
        color
    }
}

fn resolve_paint(paint: GlyphPaint, default_color: u32) -> u32 {
    match paint {
        GlyphPaint::Solid(color) => normalize_solid_color(color),
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

const GLYPH_AA_SUBPIXEL_ROWS: [f32; 4] = [0.125, 0.375, 0.625, 0.875];

fn coverage_bounds(bounds: &GlyphBounds) -> Option<(i32, i32, u32, u32)> {
    let origin_x = bounds.min_x.floor() as i32;
    let origin_y = bounds.min_y.floor() as i32;
    let max_x = bounds.max_x.ceil() as i32;
    let max_y = bounds.max_y.ceil() as i32;

    let width = max_x.saturating_sub(origin_x) as u32;
    let height = max_y.saturating_sub(origin_y) as u32;
    if width == 0 || height == 0 {
        return None;
    }

    Some((origin_x, origin_y, width, height))
}

fn accumulate_coverage_span(
    coverage: &mut [f32],
    width: u32,
    height: u32,
    start_x: f32,
    end_x: f32,
    y: i32,
    row_weight: f32,
) {
    if start_x >= end_x || y < 0 || y >= height as i32 {
        return;
    }

    let width = width as i32;
    let start = start_x.floor() as i32;
    let end = end_x.ceil() as i32 - 1;
    if start > end {
        return;
    }

    let start = start.clamp(0, width - 1);
    let end = end.clamp(0, width - 1);
    if start > end {
        return;
    }

    let row_offset = y as usize * width as usize;
    for x in start..=end {
        let pixel_start = x as f32;
        let pixel_end = pixel_start + 1.0;
        let overlap = pixel_end.min(end_x) - pixel_start.max(start_x);
        if overlap <= 0.0 {
            continue;
        }

        let index = row_offset + x as usize;
        coverage[index] += overlap.clamp(0.0, 1.0) * row_weight;
    }
}

fn blend_coverage_pixel(screen: &mut dyn Screen, x: i32, y: i32, color: u32, coverage: f32) {
    if x < 0 || y < 0 || x >= screen.width() as i32 || y >= screen.height() as i32 {
        return;
    }

    let coverage = coverage.clamp(0.0, 1.0);
    if coverage <= 0.0 {
        return;
    }

    let (red, green, blue, alpha) = color_taple(color);
    let src_alpha = (alpha as f32 / 255.0) * coverage;
    if src_alpha <= f32::EPSILON {
        return;
    }

    let width = screen.width();
    let pos = (y as u32 * width * 4 + x as u32 * 4) as usize;
    let buf = screen.buffer_mut();
    let dst_alpha = buf[pos + 3] as f32 / 255.0;
    let out_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);
    if out_alpha <= f32::EPSILON {
        return;
    }

    let dst_scale = dst_alpha * (1.0 - src_alpha);
    let red = ((red as f32 * src_alpha + buf[pos] as f32 * dst_scale) / out_alpha)
        .round()
        .clamp(0.0, 255.0) as u8;
    let green = ((green as f32 * src_alpha + buf[pos + 1] as f32 * dst_scale) / out_alpha)
        .round()
        .clamp(0.0, 255.0) as u8;
    let blue = ((blue as f32 * src_alpha + buf[pos + 2] as f32 * dst_scale) / out_alpha)
        .round()
        .clamp(0.0, 255.0) as u8;
    let alpha = (out_alpha * 255.0).round().clamp(0.0, 255.0) as u8;

    buf[pos] = red;
    buf[pos + 1] = green;
    buf[pos + 2] = blue;
    buf[pos + 3] = alpha;
}

fn fill_contours_antialias(
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
    let Some((origin_x, origin_y, width, height)) = coverage_bounds(&bounds) else {
        return;
    };

    let edges = contour_edges(contours);
    if edges.is_empty() {
        return;
    }

    let translated_edges: Vec<Edge> = edges
        .into_iter()
        .map(|edge| Edge {
            x0: edge.x0 - origin_x as f32,
            y0: edge.y0 - origin_y as f32,
            x1: edge.x1 - origin_x as f32,
            y1: edge.y1 - origin_y as f32,
            winding: edge.winding,
        })
        .collect();

    let row_weight = 1.0 / GLYPH_AA_SUBPIXEL_ROWS.len() as f32;
    let mut coverage = vec![0.0_f32; width as usize * height as usize];

    for y in 0..height as i32 {
        for subpixel_y in GLYPH_AA_SUBPIXEL_ROWS {
            let scan_y = y as f32 + subpixel_y;
            let mut intersections = Vec::new();

            for edge in &translated_edges {
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
                        accumulate_coverage_span(
                            &mut coverage,
                            width,
                            height,
                            intersections[i].0,
                            intersections[i + 1].0,
                            y,
                            row_weight,
                        );
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
                                accumulate_coverage_span(
                                    &mut coverage,
                                    width,
                                    height,
                                    start_x,
                                    x,
                                    y,
                                    row_weight,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    for y in 0..height as i32 {
        let row_offset = y as usize * width as usize;
        for x in 0..width as i32 {
            let pixel_coverage = coverage[row_offset + x as usize];
            blend_coverage_pixel(screen, origin_x + x, origin_y + y, color, pixel_coverage);
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

fn raster_interpolation(_layer: &RasterGlyphLayer) -> InterpolationAlgorithm {
    InterpolationAlgorithm::Bilinear
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
    fill_contours_antialias(screen, &contours, color, layer.fill_rule);
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
        raster_interpolation(layer),
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

#[cfg(feature = "font")]
pub fn layout_text(text: &str, options: FontOptions<'_>) -> Result<GlyphRun, Error> {
    text2commands(text, options).map_err(|error| Box::new(error) as Error)
}

#[cfg(feature = "font")]
pub fn draw_text_with_options(
    screen: &mut dyn Screen,
    text: &str,
    options: FontOptions<'_>,
    offset_x: f32,
    offset_y: f32,
    default_color: u32,
) -> Result<GlyphRun, Error> {
    let glyphs = layout_text(text, options)?;
    draw_glyphs(screen, &glyphs, offset_x, offset_y, default_color)?;
    Ok(glyphs)
}

#[cfg(feature = "font")]
pub fn draw_text_with_family<'a>(
    screen: &mut dyn Screen,
    family: &'a FontFamily,
    text: &str,
    mut options: FontOptions<'a>,
    offset_x: f32,
    offset_y: f32,
    default_color: u32,
) -> Result<GlyphRun, Error> {
    options.font = Some(FontRef::Family(family));
    if options.font_family.is_none() {
        options.font_family = Some(family.name());
    }
    draw_text_with_options(screen, text, options, offset_x, offset_y, default_color)
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
    use crate::clear::fillrect;
    #[cfg(feature = "font")]
    use fontloader::load_font_from_buffer;
    #[cfg(feature = "font")]
    use std::path::{Path, PathBuf};

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

    #[test]
    fn draw_glyphs_antialiases_diagonal_edges() {
        let commands = vec![
            Command::MoveTo(1.0, 1.0),
            Command::Line(8.0, 1.0),
            Command::Line(8.0, 8.0),
            Command::Close,
        ];

        let glyph = Glyph::new(vec![GlyphLayer::Path(PathGlyphLayer::new(
            commands,
            GlyphPaint::CurrentColor,
        ))]);
        let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 0.0, 0.0)]);
        let mut canvas = Canvas::new(10, 10);
        fillrect(&mut canvas, 0x00ff_ffff);

        draw_glyphs(&mut canvas, &run, 0.0, 0.0, 0xff00_0000).unwrap();

        let edge = rgba(&canvas, 3, 3);
        assert_eq!(edge[3], 0xff);
        assert!(edge[0] > 0x00 && edge[0] < 0xff);
        assert_eq!(edge[0], edge[1]);
        assert_eq!(edge[1], edge[2]);
    }

    #[test]
    fn draw_glyphs_keeps_argb_path_colors() {
        let commands = vec![
            Command::MoveTo(1.0, 1.0),
            Command::Line(5.0, 1.0),
            Command::Line(5.0, 5.0),
            Command::Line(1.0, 5.0),
            Command::Close,
        ];

        let glyph = Glyph::new(vec![GlyphLayer::Path(PathGlyphLayer::new(
            commands,
            GlyphPaint::Solid(0xff11_2233),
        ))]);
        let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 0.0, 0.0)]);
        let mut canvas = Canvas::new(8, 8);

        draw_glyphs(&mut canvas, &run, 0.0, 0.0, 0xffff_ffff).unwrap();

        assert_eq!(rgba(&canvas, 2, 2), [0x11, 0x22, 0x33, 0xff]);
    }

    #[test]
    fn draw_glyphs_accepts_opaque_rgba_path_colors() {
        let commands = vec![
            Command::MoveTo(1.0, 1.0),
            Command::Line(5.0, 1.0),
            Command::Line(5.0, 5.0),
            Command::Line(1.0, 5.0),
            Command::Close,
        ];

        let glyph = Glyph::new(vec![GlyphLayer::Path(PathGlyphLayer::new(
            commands,
            GlyphPaint::Solid(0x11_22_33_ff),
        ))]);
        let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 0.0, 0.0)]);
        let mut canvas = Canvas::new(8, 8);

        draw_glyphs(&mut canvas, &run, 0.0, 0.0, 0xffff_ffff).unwrap();

        assert_eq!(rgba(&canvas, 2, 2), [0x11, 0x22, 0x33, 0xff]);
    }

    #[cfg(feature = "font")]
    fn workspace_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .to_path_buf()
    }

    #[cfg(feature = "font")]
    fn find_test_font_path(name: &str) -> Option<PathBuf> {
        let root = workspace_root();
        let candidates = [
            root.join("_test-fonts").join(name),
            root.join(".tmp-fonts").join(name),
            root.join(".tmp-font").join(name),
            PathBuf::from(r"C:\Windows\Fonts").join(name),
        ];

        candidates.into_iter().find(|path| path.exists())
    }

    #[cfg(feature = "font")]
    fn load_test_font(name: &str) -> Option<fontloader::LoadedFont> {
        let path = find_test_font_path(name)?;
        let buffer = std::fs::read(&path).ok()?;
        load_font_from_buffer(&buffer).ok()
    }

    #[cfg(feature = "font")]
    fn count_non_white_pixels(screen: &dyn Screen, x0: u32, y0: u32, x1: u32, y1: u32) -> usize {
        let x1 = x1.min(screen.width());
        let y1 = y1.min(screen.height());
        let mut count = 0;

        for y in y0.min(y1)..y1 {
            for x in x0.min(x1)..x1 {
                let pixel = rgba(screen, x, y);
                if pixel[0] != 0xff || pixel[1] != 0xff || pixel[2] != 0xff {
                    count += 1;
                }
            }
        }

        count
    }

    #[cfg(feature = "font")]
    #[test]
    #[ignore = "diagnostic: currently fails on fontloader outline extraction for FiraSans-Black"]
    fn font_reader_fira_black_text2command_still_has_commands() {
        let Some(font) = load_test_font("FiraSans-Black.ttf") else {
            return;
        };

        for ch in ['i', 'j'] {
            let commands = font
                .text2command(&ch.to_string())
                .expect("text2command should succeed");
            assert_eq!(commands.len(), 1, "expected one glyph for {}", ch);
            assert!(
                !commands[0].commands.is_empty(),
                "text2command returned no commands for {}",
                ch
            );
        }
    }

    #[cfg(feature = "font")]
    #[test]
    #[ignore = "diagnostic: currently fails on fontloader glyph_run output for FiraSans-Black"]
    fn font_reader_fira_black_i_and_j_have_outline_layers() {
        let Some(font) = load_test_font("FiraSans-Black.ttf") else {
            return;
        };

        let mut options = fontloader::FontOptions::new(&font);
        options.font_size = 64.0;
        let run = font.text2glyph_run("ij", options).expect("glyph run");

        assert_eq!(run.glyphs.len(), 2, "expected two glyphs for 'ij'");
        for (index, glyph) in run.glyphs.iter().enumerate() {
            let path_layers: Vec<&PathGlyphLayer> = glyph
                .glyph
                .layers
                .iter()
                .filter_map(|layer| match layer {
                    GlyphLayer::Path(path) => Some(path),
                    GlyphLayer::Raster(_) => None,
                })
                .collect();
            assert!(
                !path_layers.is_empty(),
                "font reader returned no outline layers for glyph index {}",
                index
            );
            assert!(
                path_layers.iter().any(|path| !path.commands.is_empty()),
                "font reader returned only empty outline layers for glyph index {}",
                index
            );
            assert!(
                glyph.glyph.metrics.bounds.is_some(),
                "font reader returned no bounds for glyph index {}",
                index
            );
        }
    }

    #[cfg(feature = "font")]
    #[test]
    #[ignore = "diagnostic: currently fails on fontloader glyph_run output for seguiemj"]
    fn font_reader_segoe_emoji_has_colr_path_layers() {
        let Some(font) = load_test_font("seguiemj.ttf") else {
            return;
        };

        let mut options = fontloader::FontOptions::new(&font);
        options.font_size = 64.0;
        let run = font.text2glyph_run("🥺", options).expect("glyph run");

        assert_eq!(run.glyphs.len(), 1, "expected one glyph for emoji");
        let mut solid_layers = 0usize;
        for layer in &run.glyphs[0].glyph.layers {
            if let GlyphLayer::Path(path) = layer {
                if matches!(path.paint, GlyphPaint::Solid(_)) {
                    solid_layers += 1;
                    assert!(
                        !path.commands.is_empty(),
                        "COLR path layer should have commands"
                    );
                }
            }
        }

        assert!(solid_layers > 0, "expected solid COLR path layers");
    }

    #[cfg(feature = "font")]
    #[test]
    fn segoe_emoji_colr_layers_resolve_to_opaque_argb() {
        let Some(font) = load_test_font("seguiemj.ttf") else {
            return;
        };

        let mut options = fontloader::FontOptions::new(&font);
        options.font_size = 64.0;
        let run = font.text2glyph_run("🥺", options).expect("glyph run");

        let mut found_solid = false;
        for glyph in &run.glyphs {
            for layer in &glyph.glyph.layers {
                if let GlyphLayer::Path(path) = layer {
                    if let GlyphPaint::Solid(color) = path.paint {
                        found_solid = true;
                        let resolved = resolve_paint(GlyphPaint::Solid(color), 0xff00_0000);
                        assert_eq!(
                            resolved >> 24,
                            0xff,
                            "COLR layer colors should resolve as opaque ARGB"
                        );
                    }
                }
            }
        }

        assert!(found_solid, "expected at least one solid COLR layer");
    }

    #[cfg(feature = "font")]
    #[test]
    fn draw_text_with_family_renders_cached_face() {
        let Some(font) = load_test_font("FiraSans-Black.ttf") else {
            return;
        };
        let mut family = FontFamily::new("Fira Sans");
        family.add_face(
            FontFaceDescriptor::new("Fira Sans")
                .with_font_name("Fira Sans Black")
                .with_font_weight(FontWeight::BLACK),
            font,
        );

        let mut canvas = Canvas::new(160, 120);
        fillrect(&mut canvas, 0x00ff_ffff);

        let glyphs = draw_text_with_family(
            &mut canvas,
            &family,
            "A",
            FontOptions::from_family(&family)
                .with_font_family("Fira Sans")
                .with_font_weight(FontWeight::BLACK)
                .with_font_size(48.0),
            16.0,
            72.0,
            0xff11_1111,
        )
        .expect("draw text from font family");

        assert_eq!(glyphs.glyphs.len(), 1);
        let ink = count_non_white_pixels(&canvas, 0, 0, canvas.width(), canvas.height());
        assert!(ink > 0, "expected rendered ink from cached family face");
    }

    #[cfg(feature = "font")]
    #[test]
    fn twemoji_sbix_woff2_loads_from_buffer_and_emits_raster_layers() {
        let Some(path) = find_test_font_path("TwemojiMozilla-sbix.woff2") else {
            return;
        };
        let bytes = std::fs::read(&path).expect("read TwemojiMozilla-sbix.woff2");
        let font = load_font_from_buffer(&bytes).expect("load TwemojiMozilla-sbix.woff2");

        let run = font
            .text2glyph_run("😀", fontloader::FontOptions::new(&font).with_font_size(96.0))
            .expect("build glyph run for sbix font");

        assert!(
            !run.glyphs.is_empty(),
            "expected at least one glyph from TwemojiMozilla-sbix.woff2"
        );
        assert!(
            run.glyphs.iter().flat_map(|glyph| glyph.glyph.layers.iter()).any(
                |layer| matches!(layer, GlyphLayer::Raster(_))
            ),
            "expected sbix font to emit raster glyph layers"
        );
    }

    #[cfg(feature = "font")]
    #[test]
    fn twemoji_sbix_woff2_promotes_chunked_face_into_family_cache() {
        let Some(path) = find_test_font_path("TwemojiMozilla-sbix.woff2") else {
            return;
        };
        let bytes = std::fs::read(&path).expect("read TwemojiMozilla-sbix.woff2");

        let mut family = FontFamily::new("Twemoji Mozilla");
        family
            .begin_chunked_face(
                "twemoji-sbix",
                FontFaceDescriptor::new("Twemoji Mozilla")
                    .with_font_name("Twemoji Mozilla sbix")
                    .with_font_weight(FontWeight::NORMAL),
                bytes.len(),
            )
            .expect("begin chunked sbix face");

        let chunk = 256 * 1024;
        let mut offset = 0;
        while offset < bytes.len() {
            let end = (offset + chunk).min(bytes.len());
            family
                .append_chunk("twemoji-sbix", offset, &bytes[offset..end])
                .expect("append sbix chunk");
            offset = end;
        }

        family
            .finalize_chunked_face("twemoji-sbix")
            .expect("finalize chunked sbix face");
        assert_eq!(family.cached_faces_len(), 1);

        let run = family
            .text2glyph_run(
                "😀",
                FontOptions::from_family(&family).with_font_size(96.0),
            )
            .expect("layout sbix glyph from chunked family");
        assert!(
            run.glyphs.iter().flat_map(|glyph| glyph.glyph.layers.iter()).any(
                |layer| matches!(layer, GlyphLayer::Raster(_))
            ),
            "expected chunked sbix family load to keep raster glyph layers"
        );
    }

    #[cfg(feature = "font")]
    #[test]
    #[ignore = "diagnostic: currently fails because FiraSans-Black glyph_run has empty bounds/paths"]
    fn composite_lowercase_glyphs_render_with_visible_ink_when_fira_is_available() {
        let Some(font) = load_test_font("FiraSans-Black.ttf") else {
            return;
        };

        let mut options = fontloader::FontOptions::new(&font);
        options.font_size = 64.0;
        let run = font.text2glyph_run("ij", options).expect("glyph run");

        let mut canvas = Canvas::new(256, 160);
        fillrect(&mut canvas, 0x00ff_ffff);
        draw_glyphs(&mut canvas, &run, 24.0, 96.0, 0xff11_1111).unwrap();

        for (index, glyph) in run.glyphs.iter().enumerate() {
            let bounds = glyph.glyph.metrics.bounds.expect("glyph bounds");
            let min_x = (24.0 + glyph.x + bounds.min_x - 2.0).floor().max(0.0) as u32;
            let max_x = (24.0 + glyph.x + bounds.max_x + 2.0).ceil().max(0.0) as u32;
            let min_y = (96.0 + glyph.y + bounds.min_y - 2.0).floor().max(0.0) as u32;
            let max_y = (96.0 + glyph.y + bounds.max_y + 2.0).ceil().max(0.0) as u32;
            let ink = count_non_white_pixels(&canvas, min_x, min_y, max_x, max_y);
            assert!(
                ink > 0,
                "renderer left no visible ink for glyph index {} in its own bounds",
                index
            );
        }
    }
}
