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
use png::{
    ColorType as PngColorType, Decoder as PngDecoder, Transformations as PngTransformations,
};
use std::cmp::Ordering;
use std::io::Cursor;

#[cfg(feature = "font")]
pub use fontcore::commands as commads;
#[cfg(feature = "font")]
pub use fontcore::{
    load_font_from_buffer, FontFaceDescriptor, FontFamily, FontOptions, FontRef, FontStretch,
    FontStyle, FontVariant, FontWeight,
};
#[cfg(feature = "font")]
pub type LoadedFont = fontcore::FontFace;

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
#[derive(Debug, Clone)]
pub enum GlyphPaint {
    Solid(u32),
    CurrentColor,
    LinearGradient(LinearGradientPaint),
    RadialGradient(RadialGradientPaint),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathPaintMode {
    Fill,
    Stroke,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientSpread {
    Pad,
    Repeat,
    Reflect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientUnits {
    ObjectBoundingBox,
    UserSpaceOnUse,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    pub offset: f32,
    pub color: u32,
}

#[derive(Debug, Clone)]
pub struct LinearGradientPaint {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub units: GradientUnits,
    pub transform: [f32; 6],
    pub spread: GradientSpread,
    pub stops: Vec<GradientStop>,
}

#[derive(Debug, Clone)]
pub struct RadialGradientPaint {
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
    pub fx: f32,
    pub fy: f32,
    pub fr: f32,
    pub units: GradientUnits,
    pub transform: [f32; 6],
    pub spread: GradientSpread,
    pub stops: Vec<GradientStop>,
}

/// Vector glyph layer.
///
/// This is used for normal outline fonts and for SVG emoji after the SVG has been converted
/// into path commands by the font parser.
#[derive(Debug, Clone)]
pub struct PathGlyphLayer {
    pub commands: Vec<Command>,
    pub clip_commands: Vec<Command>,
    pub paint: GlyphPaint,
    pub paint_mode: PathPaintMode,
    pub fill_rule: FillRule,
    pub stroke_width: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl PathGlyphLayer {
    pub fn new(commands: Vec<Command>, paint: GlyphPaint) -> Self {
        Self {
            commands,
            clip_commands: Vec::new(),
            paint,
            paint_mode: PathPaintMode::Fill,
            fill_rule: FillRule::NonZero,
            stroke_width: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }

    pub fn stroke(commands: Vec<Command>, paint: GlyphPaint, stroke_width: f32) -> Self {
        Self {
            commands,
            clip_commands: Vec::new(),
            paint,
            paint_mode: PathPaintMode::Stroke,
            fill_rule: FillRule::NonZero,
            stroke_width,
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
}

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

#[cfg(feature = "svg-font")]
#[derive(Debug, Clone)]
pub struct SvgGlyphLayer {
    pub document: String,
    pub view_box_min_x: f32,
    pub view_box_min_y: f32,
    pub view_box_width: f32,
    pub view_box_height: f32,
    pub width: f32,
    pub height: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

/// Extensible glyph layer model.
///
/// - `Path`: monochrome outlines and SVG emoji vector layers.
/// - `Raster`: PNG bitmap emoji and other image-based glyph layers.
#[derive(Clone)]
pub enum GlyphLayer {
    Path(PathGlyphLayer),
    Raster(RasterGlyphLayer),
    #[cfg(feature = "svg-font")]
    Svg(SvgGlyphLayer),
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

#[cfg(feature = "font")]
impl From<fontcore::Command> for Command {
    fn from(command: fontcore::Command) -> Self {
        match command {
            fontcore::Command::Line(x, y) => Self::Line(x, y),
            fontcore::Command::MoveTo(x, y) => Self::MoveTo(x, y),
            fontcore::Command::Bezier(control, end) => Self::Bezier(control, end),
            fontcore::Command::CubicBezier(control1, control2, end) => {
                Self::CubicBezier(control1, control2, end)
            }
            fontcore::Command::Close => Self::Close,
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::GlyphFlow> for GlyphFlow {
    fn from(flow: fontcore::GlyphFlow) -> Self {
        match flow {
            fontcore::GlyphFlow::Horizontal => Self::Horizontal,
            fontcore::GlyphFlow::Vertical => Self::Vertical,
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::FontMetrics> for FontMetrics {
    fn from(metrics: fontcore::FontMetrics) -> Self {
        Self {
            ascent: metrics.ascent,
            descent: metrics.descent,
            line_gap: metrics.line_gap,
            flow: metrics.flow.into(),
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::GlyphBounds> for GlyphBounds {
    fn from(bounds: fontcore::GlyphBounds) -> Self {
        Self {
            min_x: bounds.min_x,
            min_y: bounds.min_y,
            max_x: bounds.max_x,
            max_y: bounds.max_y,
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::GlyphMetrics> for GlyphMetrics {
    fn from(metrics: fontcore::GlyphMetrics) -> Self {
        Self {
            advance_x: metrics.advance_x,
            advance_y: metrics.advance_y,
            bearing_x: metrics.bearing_x,
            bearing_y: metrics.bearing_y,
            bounds: metrics.bounds.map(Into::into),
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::GlyphPaint> for GlyphPaint {
    fn from(paint: fontcore::GlyphPaint) -> Self {
        match paint {
            fontcore::GlyphPaint::Solid(color) => Self::Solid(color),
            fontcore::GlyphPaint::CurrentColor => Self::CurrentColor,
            fontcore::GlyphPaint::LinearGradient(gradient) => Self::LinearGradient(gradient.into()),
            fontcore::GlyphPaint::RadialGradient(gradient) => Self::RadialGradient(gradient.into()),
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::GlyphGradientStop> for GradientStop {
    fn from(stop: fontcore::GlyphGradientStop) -> Self {
        Self {
            offset: stop.offset,
            color: stop.color,
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::GlyphGradientSpread> for GradientSpread {
    fn from(spread: fontcore::GlyphGradientSpread) -> Self {
        match spread {
            fontcore::GlyphGradientSpread::Pad => Self::Pad,
            fontcore::GlyphGradientSpread::Repeat => Self::Repeat,
            fontcore::GlyphGradientSpread::Reflect => Self::Reflect,
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::GlyphGradientUnits> for GradientUnits {
    fn from(units: fontcore::GlyphGradientUnits) -> Self {
        match units {
            fontcore::GlyphGradientUnits::ObjectBoundingBox => Self::ObjectBoundingBox,
            fontcore::GlyphGradientUnits::UserSpaceOnUse => Self::UserSpaceOnUse,
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::GlyphLinearGradient> for LinearGradientPaint {
    fn from(gradient: fontcore::GlyphLinearGradient) -> Self {
        Self {
            x1: gradient.x1,
            y1: gradient.y1,
            x2: gradient.x2,
            y2: gradient.y2,
            units: gradient.units.into(),
            transform: gradient.transform,
            spread: gradient.spread.into(),
            stops: gradient.stops.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::GlyphRadialGradient> for RadialGradientPaint {
    fn from(gradient: fontcore::GlyphRadialGradient) -> Self {
        Self {
            cx: gradient.cx,
            cy: gradient.cy,
            r: gradient.r,
            fx: gradient.fx,
            fy: gradient.fy,
            fr: 0.0,
            units: gradient.units.into(),
            transform: gradient.transform,
            spread: gradient.spread.into(),
            stops: gradient.stops.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::FillRule> for FillRule {
    fn from(rule: fontcore::FillRule) -> Self {
        match rule {
            fontcore::FillRule::NonZero => Self::NonZero,
            fontcore::FillRule::EvenOdd => Self::EvenOdd,
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::PathPaintMode> for PathPaintMode {
    fn from(mode: fontcore::PathPaintMode) -> Self {
        match mode {
            fontcore::PathPaintMode::Fill => Self::Fill,
            fontcore::PathPaintMode::Stroke => Self::Stroke,
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::PathGlyphLayer> for PathGlyphLayer {
    fn from(layer: fontcore::PathGlyphLayer) -> Self {
        Self {
            commands: layer.commands.into_iter().map(Into::into).collect(),
            clip_commands: layer.clip_commands.into_iter().map(Into::into).collect(),
            paint: layer.paint.into(),
            paint_mode: layer.paint_mode.into(),
            fill_rule: layer.fill_rule.into(),
            stroke_width: layer.stroke_width,
            offset_x: layer.offset_x,
            offset_y: layer.offset_y,
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::RasterGlyphSource> for RasterGlyphSource {
    fn from(source: fontcore::RasterGlyphSource) -> Self {
        match source {
            fontcore::RasterGlyphSource::Encoded(data) => Self::Encoded(data),
            fontcore::RasterGlyphSource::Rgba {
                width,
                height,
                data,
            } => Self::Rgba {
                width,
                height,
                data,
            },
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::RasterGlyphLayer> for RasterGlyphLayer {
    fn from(layer: fontcore::RasterGlyphLayer) -> Self {
        Self {
            source: layer.source.into(),
            offset_x: layer.offset_x,
            offset_y: layer.offset_y,
            width: layer.width,
            height: layer.height,
        }
    }
}

#[cfg(all(feature = "font", feature = "svg-font"))]
impl From<fontcore::SvgGlyphLayer> for SvgGlyphLayer {
    fn from(layer: fontcore::SvgGlyphLayer) -> Self {
        Self {
            document: layer.document,
            view_box_min_x: layer.view_box_min_x,
            view_box_min_y: layer.view_box_min_y,
            view_box_width: layer.view_box_width,
            view_box_height: layer.view_box_height,
            width: layer.width,
            height: layer.height,
            offset_x: layer.offset_x,
            offset_y: layer.offset_y,
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::GlyphLayer> for GlyphLayer {
    fn from(layer: fontcore::GlyphLayer) -> Self {
        match layer {
            fontcore::GlyphLayer::Path(path) => Self::Path(path.into()),
            fontcore::GlyphLayer::Raster(raster) => Self::Raster(raster.into()),
            #[cfg(feature = "svg-font")]
            fontcore::GlyphLayer::Svg(svg) => Self::Svg(svg.into()),
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::Glyph> for Glyph {
    fn from(glyph: fontcore::Glyph) -> Self {
        Self {
            font: glyph.font.map(Into::into),
            metrics: glyph.metrics.into(),
            layers: glyph.layers.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::PositionedGlyph> for PositionedGlyph {
    fn from(glyph: fontcore::PositionedGlyph) -> Self {
        Self {
            glyph: glyph.glyph.into(),
            x: glyph.x,
            y: glyph.y,
        }
    }
}

#[cfg(feature = "font")]
impl From<fontcore::GlyphRun> for GlyphRun {
    fn from(run: fontcore::GlyphRun) -> Self {
        Self {
            glyphs: run.glyphs.into_iter().map(Into::into).collect(),
        }
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

#[derive(Debug, Clone)]
struct FlattenedSubpath {
    points: Vec<(f32, f32)>,
}

struct CoverageMask {
    origin_x: i32,
    origin_y: i32,
    width: u32,
    height: u32,
    coverage: Vec<f32>,
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

fn resolve_paint(paint: &GlyphPaint, default_color: u32) -> u32 {
    match paint {
        GlyphPaint::Solid(color) => normalize_solid_color(*color),
        GlyphPaint::CurrentColor => normalize_paint_color(default_color),
        GlyphPaint::LinearGradient(_) | GlyphPaint::RadialGradient(_) => {
            normalize_paint_color(default_color)
        }
    }
}

fn interpolate_channel(start: u8, end: u8, t: f32) -> u8 {
    (start as f32 + (end as f32 - start as f32) * t)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn gradient_spread_t(mut t: f32, spread: GradientSpread) -> f32 {
    match spread {
        GradientSpread::Pad => t.clamp(0.0, 1.0),
        GradientSpread::Repeat => {
            t = t - t.floor();
            if t < 0.0 {
                t + 1.0
            } else {
                t
            }
        }
        GradientSpread::Reflect => {
            let period = t.rem_euclid(2.0);
            if period <= 1.0 {
                period
            } else {
                2.0 - period
            }
        }
    }
}

fn sample_gradient_stops(stops: &[GradientStop], t: f32) -> u32 {
    if stops.is_empty() {
        return 0;
    }
    if stops.len() == 1 {
        return normalize_solid_color(stops[0].color);
    }

    let mut stops: Vec<GradientStop> = stops.to_vec();
    stops.sort_by(|left, right| {
        left.offset
            .partial_cmp(&right.offset)
            .unwrap_or(Ordering::Equal)
    });
    let t = t.clamp(0.0, 1.0);
    if t <= stops[0].offset {
        return normalize_solid_color(stops[0].color);
    }

    for pair in stops.windows(2) {
        let start = pair[0];
        let end = pair[1];
        if t > end.offset {
            continue;
        }

        let start_color = normalize_solid_color(start.color);
        let end_color = normalize_solid_color(end.color);
        let span = (end.offset - start.offset).abs();
        let local_t = if span <= f32::EPSILON {
            0.0
        } else {
            ((t - start.offset) / (end.offset - start.offset)).clamp(0.0, 1.0)
        };

        let start_a = ((start_color >> 24) & 0xff) as u8;
        let start_r = ((start_color >> 16) & 0xff) as u8;
        let start_g = ((start_color >> 8) & 0xff) as u8;
        let start_b = (start_color & 0xff) as u8;
        let end_a = ((end_color >> 24) & 0xff) as u8;
        let end_r = ((end_color >> 16) & 0xff) as u8;
        let end_g = ((end_color >> 8) & 0xff) as u8;
        let end_b = (end_color & 0xff) as u8;

        return ((interpolate_channel(start_a, end_a, local_t) as u32) << 24)
            | ((interpolate_channel(start_r, end_r, local_t) as u32) << 16)
            | ((interpolate_channel(start_g, end_g, local_t) as u32) << 8)
            | interpolate_channel(start_b, end_b, local_t) as u32;
    }

    normalize_solid_color(stops.last().unwrap().color)
}

fn inverse_affine_point(transform: [f32; 6], x: f32, y: f32) -> (f32, f32) {
    let [a, b, c, d, e, f] = transform;
    let det = a * d - b * c;
    if det.abs() <= f32::EPSILON {
        return (x, y);
    }

    let px = x - e;
    let py = y - f;
    ((d * px - c * py) / det, (-b * px + a * py) / det)
}

fn sample_linear_gradient(gradient: &LinearGradientPaint, x: f32, y: f32) -> u32 {
    let (x, y) = inverse_affine_point(gradient.transform, x, y);
    let dx = gradient.x2 - gradient.x1;
    let dy = gradient.y2 - gradient.y1;
    let denom = dx * dx + dy * dy;
    let t = if denom <= f32::EPSILON {
        0.0
    } else {
        ((x - gradient.x1) * dx + (y - gradient.y1) * dy) / denom
    };
    sample_gradient_stops(&gradient.stops, gradient_spread_t(t, gradient.spread))
}

fn solve_radial_gradient_t(gradient: &RadialGradientPaint, x: f32, y: f32) -> f32 {
    let px = x - gradient.fx;
    let py = y - gradient.fy;
    let dcx = gradient.cx - gradient.fx;
    let dcy = gradient.cy - gradient.fy;
    let dr = gradient.r - gradient.fr;

    let a = dcx * dcx + dcy * dcy - dr * dr;
    let b = -2.0 * (px * dcx + py * dcy + gradient.fr * dr);
    let c = px * px + py * py - gradient.fr * gradient.fr;

    if a.abs() <= f32::EPSILON {
        if b.abs() <= f32::EPSILON {
            return 0.0;
        }
        return (-c / b).clamp(-1_000_000.0, 1_000_000.0);
    }

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return 0.0;
    }

    let sqrt_d = discriminant.sqrt();
    let t0 = (-b - sqrt_d) / (2.0 * a);
    let t1 = (-b + sqrt_d) / (2.0 * a);

    match (t0.is_finite(), t1.is_finite()) {
        (true, true) => {
            if t0 >= 0.0 && t1 >= 0.0 {
                t0.min(t1)
            } else if t0 >= 0.0 {
                t0
            } else {
                t1
            }
        }
        (true, false) => t0,
        (false, true) => t1,
        (false, false) => 0.0,
    }
}

fn sample_radial_gradient(gradient: &RadialGradientPaint, x: f32, y: f32) -> u32 {
    let (x, y) = inverse_affine_point(gradient.transform, x, y);
    let t = solve_radial_gradient_t(gradient, x, y);
    sample_gradient_stops(&gradient.stops, gradient_spread_t(t, gradient.spread))
}

fn resolve_paint_at(paint: &GlyphPaint, default_color: u32, x: f32, y: f32) -> u32 {
    match paint {
        GlyphPaint::Solid(_) | GlyphPaint::CurrentColor => resolve_paint(paint, default_color),
        GlyphPaint::LinearGradient(gradient) => sample_linear_gradient(gradient, x, y),
        GlyphPaint::RadialGradient(gradient) => sample_radial_gradient(gradient, x, y),
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

fn point_segment_distance_sq(point: (f32, f32), start: (f32, f32), end: (f32, f32)) -> f32 {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let length_sq = dx * dx + dy * dy;
    if length_sq <= f32::EPSILON {
        let px = point.0 - start.0;
        let py = point.1 - start.1;
        return px * px + py * py;
    }

    let t = (((point.0 - start.0) * dx + (point.1 - start.1) * dy) / length_sq).clamp(0.0, 1.0);
    let projection = (start.0 + dx * t, start.1 + dy * t);
    let px = point.0 - projection.0;
    let py = point.1 - projection.1;
    px * px + py * py
}

// Font outlines already arrive in device space. Estimate a bounded segment count once, then
// sample the curve directly. This avoids recursive half-pixel subdivision on long smooth curves.
const CURVE_FLATNESS_TOLERANCE_SQ: f32 = 0.25;
const CURVE_MAX_SEGMENTS: usize = 64;
const CURVE_SEGMENT_LENGTH: f32 = 4.0;

fn distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    (dx * dx + dy * dy).sqrt()
}

fn curve_segment_count(control_polygon_length: f32, flatness_sq: f32) -> usize {
    let length_steps = (control_polygon_length / CURVE_SEGMENT_LENGTH).ceil() as usize;
    let flatness_steps = (flatness_sq / CURVE_FLATNESS_TOLERANCE_SQ).sqrt().ceil() as usize;
    length_steps
        .max(flatness_steps)
        .max(1)
        .min(CURVE_MAX_SEGMENTS)
}

fn quadratic_point(start: (f32, f32), control: (f32, f32), end: (f32, f32), t: f32) -> (f32, f32) {
    let mt = 1.0 - t;
    let a = mt * mt;
    let b = 2.0 * mt * t;
    let c = t * t;
    (
        a * start.0 + b * control.0 + c * end.0,
        a * start.1 + b * control.1 + c * end.1,
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
    let a = mt * mt * mt;
    let b = 3.0 * mt * mt * t;
    let c = 3.0 * mt * t * t;
    let d = t * t * t;
    (
        a * start.0 + b * control1.0 + c * control2.0 + d * end.0,
        a * start.1 + b * control1.1 + c * control2.1 + d * end.1,
    )
}

fn flatten_quadratic_segment(
    points: &mut Vec<(f32, f32)>,
    start: (f32, f32),
    control: (f32, f32),
    end: (f32, f32),
) {
    let flatness = point_segment_distance_sq(control, start, end);
    let control_polygon_length = distance(start, control) + distance(control, end);
    let segment_count = curve_segment_count(control_polygon_length, flatness);
    for step in 1..segment_count {
        let t = step as f32 / segment_count as f32;
        push_point(points, quadratic_point(start, control, end, t));
    }
    push_point(points, end);
}

fn flatten_cubic_segment(
    points: &mut Vec<(f32, f32)>,
    start: (f32, f32),
    control1: (f32, f32),
    control2: (f32, f32),
    end: (f32, f32),
) {
    let flatness = point_segment_distance_sq(control1, start, end)
        .max(point_segment_distance_sq(control2, start, end));
    let control_polygon_length =
        distance(start, control1) + distance(control1, control2) + distance(control2, end);
    let segment_count = curve_segment_count(control_polygon_length, flatness);
    for step in 1..segment_count {
        let t = step as f32 / segment_count as f32;
        push_point(points, cubic_point(start, control1, control2, end, t));
    }
    push_point(points, end);
}

fn flush_subpath(
    subpaths: &mut Vec<FlattenedSubpath>,
    points: &mut Vec<(f32, f32)>,
    _closed: bool,
) {
    if points.len() < 2 {
        points.clear();
        return;
    }
    subpaths.push(FlattenedSubpath {
        points: std::mem::take(points),
    });
}

fn flatten_commands(commands: &[Command], offset_x: f32, offset_y: f32) -> Vec<FlattenedSubpath> {
    let mut subpaths = Vec::new();
    let mut points = Vec::new();
    let mut current_point = None;
    let mut start_point = None;

    for command in commands {
        match command {
            Command::MoveTo(x, y) => {
                flush_subpath(&mut subpaths, &mut points, false);
                let point = (x + offset_x, y + offset_y);
                points.push(point);
                current_point = Some(point);
                start_point = Some(point);
            }
            Command::Line(x, y) => {
                if let Some(current) = current_point {
                    let point = (x + offset_x, y + offset_y);
                    if points.is_empty() {
                        points.push(current);
                    }
                    push_point(&mut points, point);
                    current_point = Some(point);
                }
            }
            Command::Bezier(control, end) => {
                if let Some(start) = current_point {
                    let control = (control.0 + offset_x, control.1 + offset_y);
                    let end = (end.0 + offset_x, end.1 + offset_y);
                    if points.is_empty() {
                        points.push(start);
                    }
                    flatten_quadratic_segment(&mut points, start, control, end);
                    current_point = Some(end);
                }
            }
            Command::CubicBezier(control1, control2, end) => {
                if let Some(start) = current_point {
                    let control1 = (control1.0 + offset_x, control1.1 + offset_y);
                    let control2 = (control2.0 + offset_x, control2.1 + offset_y);
                    let end = (end.0 + offset_x, end.1 + offset_y);
                    if points.is_empty() {
                        points.push(start);
                    }
                    flatten_cubic_segment(&mut points, start, control1, control2, end);
                    current_point = Some(end);
                }
            }
            Command::Close => {
                if let Some(start) = start_point {
                    if points.is_empty() {
                        points.push(start);
                    }
                    push_point(&mut points, start);
                }
                flush_subpath(&mut subpaths, &mut points, true);
                current_point = start_point;
                start_point = None;
            }
        }
    }

    flush_subpath(&mut subpaths, &mut points, false);
    subpaths
}

fn subpath_bounds(subpaths: &[FlattenedSubpath]) -> Option<GlyphBounds> {
    let mut iter = subpaths
        .iter()
        .flat_map(|subpath| subpath.points.iter().copied());
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

fn subpaths_to_fill_contours(subpaths: &[FlattenedSubpath]) -> Vec<Vec<(f32, f32)>> {
    subpaths
        .iter()
        .filter_map(|subpath| {
            if subpath.points.len() < 2 {
                return None;
            }
            let mut contour = subpath.points.clone();
            if contour.first() != contour.last() {
                contour.push(contour[0]);
            }
            Some(contour)
        })
        .collect()
}

fn stroke_segments(subpaths: &[FlattenedSubpath]) -> Vec<((f32, f32), (f32, f32))> {
    let mut segments = Vec::new();

    for subpath in subpaths {
        for window in subpath.points.windows(2) {
            if window[0] != window[1] {
                segments.push((window[0], window[1]));
            }
        }
    }

    segments
}

// We rasterize vector glyphs in device space, so rounded low-resolution glyphs benefit from
// denser vertical supersampling than generic shapes.
const GLYPH_AA_SUBPIXEL_ROW_COUNT: usize = 32;

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

fn rasterize_fill_coverage(contours: &[Vec<(f32, f32)>], rule: FillRule) -> Option<CoverageMask> {
    let bounds = subpath_bounds(
        &contours
            .iter()
            .cloned()
            .map(|points| FlattenedSubpath { points })
            .collect::<Vec<_>>(),
    )?;
    let (origin_x, origin_y, width, height) = coverage_bounds(&bounds)?;

    let edges = contour_edges(contours);
    if edges.is_empty() {
        return None;
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

    let row_weight = 1.0 / GLYPH_AA_SUBPIXEL_ROW_COUNT as f32;
    let mut coverage = vec![0.0_f32; width as usize * height as usize];

    for y in 0..height as i32 {
        for subpixel_index in 0..GLYPH_AA_SUBPIXEL_ROW_COUNT {
            let scan_y =
                y as f32 + (subpixel_index as f32 + 0.5) / GLYPH_AA_SUBPIXEL_ROW_COUNT as f32;
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

    Some(CoverageMask {
        origin_x,
        origin_y,
        width,
        height,
        coverage,
    })
}

fn rasterize_stroke_coverage(
    subpaths: &[FlattenedSubpath],
    stroke_width: f32,
) -> Option<CoverageMask> {
    let radius = (stroke_width.max(0.0) * 0.5).max(0.5);
    let radius_sq = radius * radius;
    let mut bounds = subpath_bounds(subpaths)?;
    bounds.min_x -= radius;
    bounds.min_y -= radius;
    bounds.max_x += radius;
    bounds.max_y += radius;
    let (origin_x, origin_y, width, height) = coverage_bounds(&bounds)?;
    let segments = stroke_segments(subpaths);
    if segments.is_empty() {
        return None;
    }

    let row_weight = 1.0 / GLYPH_AA_SUBPIXEL_ROW_COUNT as f32;
    let mut coverage = vec![0.0_f32; width as usize * height as usize];

    for y in 0..height as i32 {
        let row_offset = y as usize * width as usize;
        for x in 0..width as i32 {
            let sample_x = origin_x as f32 + x as f32 + 0.5;
            let mut pixel_coverage = 0.0;

            for subpixel_index in 0..GLYPH_AA_SUBPIXEL_ROW_COUNT {
                let sample_y = origin_y as f32
                    + y as f32
                    + (subpixel_index as f32 + 0.5) / GLYPH_AA_SUBPIXEL_ROW_COUNT as f32;

                let covered = segments.iter().any(|(start, end)| {
                    point_segment_distance_sq((sample_x, sample_y), *start, *end) <= radius_sq
                });

                if covered {
                    pixel_coverage += row_weight;
                }
            }

            coverage[row_offset + x as usize] = pixel_coverage;
        }
    }

    Some(CoverageMask {
        origin_x,
        origin_y,
        width,
        height,
        coverage,
    })
}

fn paint_coverage_mask(
    screen: &mut dyn Screen,
    mask: CoverageMask,
    paint: &GlyphPaint,
    default_color: u32,
) {
    if screen.width() == 0 || screen.height() == 0 {
        return;
    }

    for y in 0..mask.height as i32 {
        let row_offset = y as usize * mask.width as usize;
        for x in 0..mask.width as i32 {
            let pixel_coverage = mask.coverage[row_offset + x as usize];
            if pixel_coverage <= 0.0 {
                continue;
            }
            let paint_x = mask.origin_x as f32 + x as f32 + 0.5;
            let paint_y = mask.origin_y as f32 + y as f32 + 0.5;
            let color = resolve_paint_at(paint, default_color, paint_x, paint_y);
            blend_coverage_pixel(
                screen,
                mask.origin_x + x,
                mask.origin_y + y,
                color,
                pixel_coverage,
            );
        }
    }
}

fn mask_coverage_at(mask: &CoverageMask, x: i32, y: i32) -> f32 {
    if x < mask.origin_x
        || y < mask.origin_y
        || x >= mask.origin_x + mask.width as i32
        || y >= mask.origin_y + mask.height as i32
    {
        return 0.0;
    }

    let local_x = (x - mask.origin_x) as usize;
    let local_y = (y - mask.origin_y) as usize;
    mask.coverage[local_y * mask.width as usize + local_x]
}

fn apply_clip_mask(mask: &mut CoverageMask, clip_mask: &CoverageMask) {
    for y in 0..mask.height as i32 {
        let row_offset = y as usize * mask.width as usize;
        let abs_y = mask.origin_y + y;
        for x in 0..mask.width as i32 {
            let abs_x = mask.origin_x + x;
            let clip_coverage = mask_coverage_at(clip_mask, abs_x, abs_y);
            let index = row_offset + x as usize;
            mask.coverage[index] *= clip_coverage;
        }
    }
}

fn clip_mask_from_commands(
    commands: &[Command],
    offset_x: f32,
    offset_y: f32,
) -> Option<CoverageMask> {
    if commands.is_empty() {
        return None;
    }

    let subpaths = flatten_commands(commands, offset_x, offset_y);
    let contours = subpaths_to_fill_contours(&subpaths);
    rasterize_fill_coverage(&contours, FillRule::NonZero)
}

fn decode_raster(source: &RasterGlyphSource) -> Result<Layer, Error> {
    match source {
        RasterGlyphSource::Encoded(data) => {
            let mut layer = Layer::tmp(0, 0);
            match image::draw_image(&mut layer, data, 0) {
                Ok(_) => Ok(layer),
                Err(primary_error) => decode_png_raster(data).or(Err(primary_error)),
            }
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

fn decode_png_raster(data: &[u8]) -> Result<Layer, Error> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if data.len() < PNG_SIGNATURE.len() || &data[..PNG_SIGNATURE.len()] != PNG_SIGNATURE {
        return Err(paint_error("encoded raster glyph is not a PNG"));
    }

    let cursor = Cursor::new(data);
    let mut decoder = PngDecoder::new(cursor);
    decoder.set_transformations(PngTransformations::EXPAND | PngTransformations::STRIP_16);

    let mut reader = decoder
        .read_info()
        .map_err(|error| paint_error(&format!("png glyph decode failed: {}", error)))?;
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|error| paint_error(&format!("png glyph frame decode failed: {}", error)))?;
    let pixels = &buffer[..info.buffer_size()];

    let rgba = match info.color_type {
        PngColorType::Rgba => pixels.to_vec(),
        PngColorType::Rgb => {
            let mut rgba = Vec::with_capacity((info.width * info.height * 4) as usize);
            for chunk in pixels.chunks_exact(3) {
                rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 0xff]);
            }
            rgba
        }
        PngColorType::Grayscale => {
            let mut rgba = Vec::with_capacity((info.width * info.height * 4) as usize);
            for gray in pixels {
                rgba.extend_from_slice(&[*gray, *gray, *gray, 0xff]);
            }
            rgba
        }
        PngColorType::GrayscaleAlpha => {
            let mut rgba = Vec::with_capacity((info.width * info.height * 4) as usize);
            for chunk in pixels.chunks_exact(2) {
                rgba.extend_from_slice(&[chunk[0], chunk[0], chunk[0], chunk[1]]);
            }
            rgba
        }
        PngColorType::Indexed => {
            return Err(paint_error(
                "png glyph decode left indexed data after EXPAND transformation",
            ));
        }
    };

    Ok(Layer::new_in(
        "_glyph_raster_png_".to_string(),
        rgba,
        info.width,
        info.height,
    ))
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
    let clip_mask = clip_mask_from_commands(
        &layer.clip_commands,
        origin_x + layer.offset_x,
        origin_y + layer.offset_y,
    );
    let subpaths = flatten_commands(
        &layer.commands,
        origin_x + layer.offset_x,
        origin_y + layer.offset_y,
    );
    match layer.paint_mode {
        PathPaintMode::Fill => {
            let contours = subpaths_to_fill_contours(&subpaths);
            if let Some(mut mask) = rasterize_fill_coverage(&contours, layer.fill_rule) {
                if let Some(clip_mask) = &clip_mask {
                    apply_clip_mask(&mut mask, clip_mask);
                }
                paint_coverage_mask(screen, mask, &layer.paint, default_color);
            }
        }
        PathPaintMode::Stroke => {
            if let Some(mut mask) = rasterize_stroke_coverage(&subpaths, layer.stroke_width) {
                if let Some(clip_mask) = &clip_mask {
                    apply_clip_mask(&mut mask, clip_mask);
                }
                paint_coverage_mask(screen, mask, &layer.paint, default_color);
            }
        }
    }
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

#[cfg(feature = "svg-font")]
fn draw_svg_layer(
    _screen: &mut dyn Screen,
    _layer: &SvgGlyphLayer,
    _origin_x: f32,
    _origin_y: f32,
) -> Result<(), Error> {
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
            #[cfg(feature = "svg-font")]
            GlyphLayer::Svg(layer) => draw_svg_layer(screen, layer, origin_x, origin_y)?,
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

pub fn glyph_renderer_info() -> String {
    format!(
        "curve_tol_sq={:.10};max_segments={};segment_length={:.4};aa_rows={}",
        CURVE_FLATNESS_TOLERANCE_SQ,
        CURVE_MAX_SEGMENTS,
        CURVE_SEGMENT_LENGTH,
        GLYPH_AA_SUBPIXEL_ROW_COUNT
    )
}

#[cfg(feature = "font")]
pub fn layout_text(text: &str, options: FontOptions<'_>) -> Result<GlyphRun, Error> {
    fontcore::text2commands(text, options)
        .map(Into::into)
        .map_err(|error| Box::new(error) as Error)
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
