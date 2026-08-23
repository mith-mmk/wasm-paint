//! Paint colors and sources shared by raster drawing operations.

/// An explicit, non-premultiplied RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);

    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::rgba(red, green, blue, 0xff)
    }

    /// Creates a color from the legacy paintcore `0xAARRGGBB` representation.
    pub const fn from_argb_u32(color: u32) -> Self {
        Self::rgba(
            ((color >> 16) & 0xff) as u8,
            ((color >> 8) & 0xff) as u8,
            (color & 0xff) as u8,
            ((color >> 24) & 0xff) as u8,
        )
    }

    /// Creates an opaque color from the legacy paintcore `0xRRGGBB` representation.
    pub const fn from_rgb_u32(color: u32) -> Self {
        Self::rgb(
            ((color >> 16) & 0xff) as u8,
            ((color >> 8) & 0xff) as u8,
            (color & 0xff) as u8,
        )
    }

    /// Creates a color from `0xRRGGBBAA`.
    pub const fn from_rgba_u32(color: u32) -> Self {
        Self::rgba(
            ((color >> 24) & 0xff) as u8,
            ((color >> 16) & 0xff) as u8,
            ((color >> 8) & 0xff) as u8,
            (color & 0xff) as u8,
        )
    }

    pub const fn to_argb_u32(self) -> u32 {
        ((self.alpha as u32) << 24)
            | ((self.red as u32) << 16)
            | ((self.green as u32) << 8)
            | self.blue as u32
    }
}

use std::cmp::Ordering;
use std::sync::Arc;

use crate::canvas::Screen;
use crate::composite::{blend_pixel, DrawOptions};
use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaintTransform(pub [f32; 6]);

impl PaintTransform {
    pub const IDENTITY: Self = Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    pub const fn new(matrix: [f32; 6]) -> Self {
        Self(matrix)
    }

    pub fn multiply(self, other: Self) -> Self {
        let [a0, b0, c0, d0, e0, f0] = self.0;
        let [a1, b1, c1, d1, e1, f1] = other.0;
        Self([
            a0 * a1 + c0 * b1,
            b0 * a1 + d0 * b1,
            a0 * c1 + c0 * d1,
            b0 * c1 + d0 * d1,
            a0 * e1 + c0 * f1 + e0,
            b0 * e1 + d0 * f1 + f0,
        ])
    }

    pub fn inverse_point(self, x: f32, y: f32) -> Option<(f32, f32)> {
        let [a, b, c, d, e, f] = self.0;
        let determinant = a * d - b * c;
        if !determinant.is_finite() || determinant.abs() <= f32::EPSILON {
            return None;
        }
        let px = x - e;
        let py = y - f;
        Some((
            (d * px - c * py) / determinant,
            (-b * px + a * py) / determinant,
        ))
    }
}

impl Default for PaintTransform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaintBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl PaintBounds {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn object_transform(self) -> PaintTransform {
        PaintTransform([self.width, 0.0, 0.0, self.height, self.x, self.y])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpreadMode {
    #[default]
    Pad,
    Repeat,
    Reflect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaintUnits {
    ObjectBoundingBox,
    #[default]
    UserSpaceOnUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GradientInterpolation {
    #[default]
    Srgb,
    LinearSrgb,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorStop {
    pub offset: f32,
    pub color: Color,
}

impl ColorStop {
    pub const fn new(offset: f32, color: Color) -> Self {
        Self { offset, color }
    }
}

#[derive(Debug, Clone)]
pub struct LinearGradient {
    pub start: (f32, f32),
    pub end: (f32, f32),
    pub stops: Vec<ColorStop>,
    pub spread: SpreadMode,
    pub units: PaintUnits,
    pub transform: PaintTransform,
    pub interpolation: GradientInterpolation,
}

#[derive(Debug, Clone)]
pub struct RadialGradient {
    pub center: (f32, f32),
    pub radius: f32,
    pub focal: (f32, f32),
    pub focal_radius: f32,
    pub stops: Vec<ColorStop>,
    pub spread: SpreadMode,
    pub units: PaintUnits,
    pub transform: PaintTransform,
    pub interpolation: GradientInterpolation,
}

#[derive(Debug, Clone)]
pub struct SweepGradient {
    pub center: (f32, f32),
    pub start_angle: f32,
    pub end_angle: f32,
    pub stops: Vec<ColorStop>,
    pub spread: SpreadMode,
    pub units: PaintUnits,
    pub transform: PaintTransform,
    pub interpolation: GradientInterpolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TileMode {
    #[default]
    Repeat,
    Mirror,
    Clamp,
    Decal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SamplingMode {
    #[default]
    Nearest,
    Bilinear,
}

#[derive(Debug, Clone)]
pub struct PatternImage {
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
}

impl PatternImage {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, Error> {
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| Error {
                message: "pattern image size overflow".to_string(),
            })?;
        if width == 0 || height == 0 || pixels.len() != expected {
            return Err(Error {
                message: "pattern image buffer size mismatch".to_string(),
            });
        }
        Ok(Self {
            width,
            height,
            pixels: pixels.into(),
        })
    }

    pub const fn width(&self) -> u32 {
        self.width
    }
    pub const fn height(&self) -> u32 {
        self.height
    }
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub image: PatternImage,
    pub tile_x: TileMode,
    pub tile_y: TileMode,
    pub sampling: SamplingMode,
    pub transform: PaintTransform,
}

/// A reusable source of color for drawing operations.
#[derive(Debug, Clone)]
pub enum Paint {
    Solid(Color),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
    SweepGradient(SweepGradient),
    Pattern(Pattern),
}

#[derive(Debug, Clone)]
struct PreparedGradient {
    stops: Arc<[ColorStop]>,
    spread: SpreadMode,
    interpolation: GradientInterpolation,
}

#[derive(Debug, Clone)]
pub struct PreparedPaint(PreparedPaintKind);

#[derive(Debug, Clone)]
enum PreparedPaintKind {
    Solid(Color),
    Linear {
        gradient: LinearGradient,
        prepared: PreparedGradient,
        transform: PaintTransform,
    },
    Radial {
        gradient: RadialGradient,
        prepared: PreparedGradient,
        transform: PaintTransform,
    },
    Sweep {
        gradient: SweepGradient,
        prepared: PreparedGradient,
        transform: PaintTransform,
    },
    Pattern(Pattern),
}

impl Paint {
    pub const fn solid(color: Color) -> Self {
        Self::Solid(color)
    }

    pub fn prepare(&self, bounds: PaintBounds) -> PreparedPaint {
        match self {
            Self::Solid(color) => PreparedPaint(PreparedPaintKind::Solid(*color)),
            Self::LinearGradient(gradient) => PreparedPaint(PreparedPaintKind::Linear {
                gradient: gradient.clone(),
                prepared: prepare_gradient(
                    &gradient.stops,
                    gradient.spread,
                    gradient.interpolation,
                ),
                transform: unit_transform(gradient.units, bounds, gradient.transform),
            }),
            Self::RadialGradient(gradient) => PreparedPaint(PreparedPaintKind::Radial {
                gradient: gradient.clone(),
                prepared: prepare_gradient(
                    &gradient.stops,
                    gradient.spread,
                    gradient.interpolation,
                ),
                transform: unit_transform(gradient.units, bounds, gradient.transform),
            }),
            Self::SweepGradient(gradient) => PreparedPaint(PreparedPaintKind::Sweep {
                gradient: gradient.clone(),
                prepared: prepare_gradient(
                    &gradient.stops,
                    gradient.spread,
                    gradient.interpolation,
                ),
                transform: unit_transform(gradient.units, bounds, gradient.transform),
            }),
            Self::Pattern(pattern) => PreparedPaint(PreparedPaintKind::Pattern(pattern.clone())),
        }
    }
}

fn unit_transform(
    units: PaintUnits,
    bounds: PaintBounds,
    transform: PaintTransform,
) -> PaintTransform {
    match units {
        PaintUnits::UserSpaceOnUse => transform,
        PaintUnits::ObjectBoundingBox => bounds.object_transform().multiply(transform),
    }
}

fn prepare_gradient(
    stops: &[ColorStop],
    spread: SpreadMode,
    interpolation: GradientInterpolation,
) -> PreparedGradient {
    let mut stops: Vec<ColorStop> = stops
        .iter()
        .copied()
        .filter(|stop| stop.offset.is_finite())
        .map(|mut stop| {
            stop.offset = stop.offset.clamp(0.0, 1.0);
            stop
        })
        .collect();
    stops.sort_by(|left, right| {
        left.offset
            .partial_cmp(&right.offset)
            .unwrap_or(Ordering::Equal)
    });
    PreparedGradient {
        stops: stops.into(),
        spread,
        interpolation,
    }
}

#[inline]
fn spread_value(value: f32, spread: SpreadMode) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    match spread {
        SpreadMode::Pad => value.clamp(0.0, 1.0),
        SpreadMode::Repeat => value.rem_euclid(1.0),
        SpreadMode::Reflect => {
            let period = value.rem_euclid(2.0);
            if period <= 1.0 {
                period
            } else {
                2.0 - period
            }
        }
    }
}

#[inline]
fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
fn linear_to_srgb(value: f32) -> f32 {
    if value <= 0.003_130_8 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

fn interpolate_color(
    start: Color,
    end: Color,
    t: f32,
    interpolation: GradientInterpolation,
) -> Color {
    let t = t.clamp(0.0, 1.0);
    let start_alpha = start.alpha as f32 / 255.0;
    let end_alpha = end.alpha as f32 / 255.0;
    let alpha = start_alpha + (end_alpha - start_alpha) * t;
    let mut channels = [0u8; 3];
    for (index, (start_channel, end_channel)) in [start.red, start.green, start.blue]
        .into_iter()
        .zip([end.red, end.green, end.blue])
        .enumerate()
    {
        let mut start_value = start_channel as f32 / 255.0;
        let mut end_value = end_channel as f32 / 255.0;
        if interpolation == GradientInterpolation::LinearSrgb {
            start_value = srgb_to_linear(start_value);
            end_value = srgb_to_linear(end_value);
        }
        let premultiplied =
            start_value * start_alpha + (end_value * end_alpha - start_value * start_alpha) * t;
        let mut value = if alpha <= f32::EPSILON {
            0.0
        } else {
            premultiplied / alpha
        };
        if interpolation == GradientInterpolation::LinearSrgb {
            value = linear_to_srgb(value);
        }
        channels[index] = (value * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    Color::rgba(
        channels[0],
        channels[1],
        channels[2],
        (alpha * 255.0).round() as u8,
    )
}

fn sample_stops(gradient: &PreparedGradient, value: f32) -> Color {
    let stops = gradient.stops.as_ref();
    if stops.is_empty() {
        return Color::TRANSPARENT;
    }
    if stops.len() == 1 {
        return stops[0].color;
    }
    let value = spread_value(value, gradient.spread);
    if value < stops[0].offset {
        return stops[0].color;
    }
    for pair in stops.windows(2) {
        if value > pair[1].offset {
            continue;
        }
        let span = pair[1].offset - pair[0].offset;
        if span <= f32::EPSILON {
            return pair[1].color;
        }
        return interpolate_color(
            pair[0].color,
            pair[1].color,
            (value - pair[0].offset) / span,
            gradient.interpolation,
        );
    }
    stops[stops.len() - 1].color
}

fn radial_parameter(gradient: &RadialGradient, x: f32, y: f32) -> f32 {
    let px = x - gradient.focal.0;
    let py = y - gradient.focal.1;
    let dcx = gradient.center.0 - gradient.focal.0;
    let dcy = gradient.center.1 - gradient.focal.1;
    let dr = gradient.radius - gradient.focal_radius;
    let a = dcx * dcx + dcy * dcy - dr * dr;
    let b = -2.0 * (px * dcx + py * dcy + gradient.focal_radius * dr);
    let c = px * px + py * py - gradient.focal_radius * gradient.focal_radius;
    if a.abs() <= f32::EPSILON {
        return if b.abs() <= f32::EPSILON { 0.0 } else { -c / b };
    }
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return 0.0;
    }
    let root = discriminant.sqrt();
    let first = (-b - root) / (2.0 * a);
    let second = (-b + root) / (2.0 * a);
    match (first >= 0.0, second >= 0.0) {
        (true, true) => first.min(second),
        (true, false) => first,
        (false, true) => second,
        (false, false) => first.max(second),
    }
}

fn tile_coordinate(value: i32, size: u32, mode: TileMode) -> Option<u32> {
    let size = size as i32;
    match mode {
        TileMode::Repeat => Some(value.rem_euclid(size) as u32),
        TileMode::Mirror => {
            let value = value.rem_euclid(size * 2);
            Some(if value < size {
                value
            } else {
                size * 2 - value - 1
            } as u32)
        }
        TileMode::Clamp => Some(value.clamp(0, size - 1) as u32),
        TileMode::Decal => (value >= 0 && value < size).then_some(value as u32),
    }
}

fn pattern_pixel(pattern: &Pattern, x: i32, y: i32) -> Color {
    let Some(x) = tile_coordinate(x, pattern.image.width, pattern.tile_x) else {
        return Color::TRANSPARENT;
    };
    let Some(y) = tile_coordinate(y, pattern.image.height, pattern.tile_y) else {
        return Color::TRANSPARENT;
    };
    let offset = ((y * pattern.image.width + x) * 4) as usize;
    Color::rgba(
        pattern.image.pixels[offset],
        pattern.image.pixels[offset + 1],
        pattern.image.pixels[offset + 2],
        pattern.image.pixels[offset + 3],
    )
}

fn sample_pattern(pattern: &Pattern, x: f32, y: f32) -> Color {
    let Some((x, y)) = pattern.transform.inverse_point(x, y) else {
        return Color::TRANSPARENT;
    };
    match pattern.sampling {
        SamplingMode::Nearest => pattern_pixel(pattern, x.floor() as i32, y.floor() as i32),
        SamplingMode::Bilinear => {
            let base_x = (x - 0.5).floor();
            let base_y = (y - 0.5).floor();
            let tx = (x - 0.5) - base_x;
            let ty = (y - 0.5) - base_y;
            let samples = [
                (
                    pattern_pixel(pattern, base_x as i32, base_y as i32),
                    (1.0 - tx) * (1.0 - ty),
                ),
                (
                    pattern_pixel(pattern, base_x as i32 + 1, base_y as i32),
                    tx * (1.0 - ty),
                ),
                (
                    pattern_pixel(pattern, base_x as i32, base_y as i32 + 1),
                    (1.0 - tx) * ty,
                ),
                (
                    pattern_pixel(pattern, base_x as i32 + 1, base_y as i32 + 1),
                    tx * ty,
                ),
            ];
            let alpha: f32 = samples
                .iter()
                .map(|(color, weight)| color.alpha as f32 * weight)
                .sum();
            if alpha <= f32::EPSILON {
                return Color::TRANSPARENT;
            }
            let channel = |get: fn(Color) -> u8| -> u8 {
                let premultiplied: f32 = samples
                    .iter()
                    .map(|(color, weight)| get(*color) as f32 * color.alpha as f32 * weight)
                    .sum();
                (premultiplied / alpha).round().clamp(0.0, 255.0) as u8
            };
            Color::rgba(
                channel(|c| c.red),
                channel(|c| c.green),
                channel(|c| c.blue),
                alpha.round().clamp(0.0, 255.0) as u8,
            )
        }
    }
}

impl PreparedPaint {
    pub fn sample(&self, x: f32, y: f32) -> Color {
        match &self.0 {
            PreparedPaintKind::Solid(color) => *color,
            PreparedPaintKind::Linear {
                gradient,
                prepared,
                transform,
            } => {
                let Some((x, y)) = transform.inverse_point(x, y) else {
                    return Color::TRANSPARENT;
                };
                let dx = gradient.end.0 - gradient.start.0;
                let dy = gradient.end.1 - gradient.start.1;
                let denominator = dx * dx + dy * dy;
                let value = if denominator <= f32::EPSILON {
                    0.0
                } else {
                    ((x - gradient.start.0) * dx + (y - gradient.start.1) * dy) / denominator
                };
                sample_stops(prepared, value)
            }
            PreparedPaintKind::Radial {
                gradient,
                prepared,
                transform,
            } => {
                let Some((x, y)) = transform.inverse_point(x, y) else {
                    return Color::TRANSPARENT;
                };
                sample_stops(prepared, radial_parameter(gradient, x, y))
            }
            PreparedPaintKind::Sweep {
                gradient,
                prepared,
                transform,
            } => {
                let Some((x, y)) = transform.inverse_point(x, y) else {
                    return Color::TRANSPARENT;
                };
                let angle = (y - gradient.center.1).atan2(x - gradient.center.0);
                let span = gradient.end_angle - gradient.start_angle;
                let value = if span.abs() <= f32::EPSILON {
                    0.0
                } else {
                    (angle - gradient.start_angle) / span
                };
                sample_stops(prepared, value)
            }
            PreparedPaintKind::Pattern(pattern) => sample_pattern(pattern, x, y),
        }
    }
}

pub fn fill_paint_rect(
    screen: &mut dyn Screen,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    paint: &Paint,
    options: DrawOptions,
) {
    let bounds = PaintBounds::new(x as f32, y as f32, width as f32, height as f32);
    let prepared = paint.prepare(bounds);
    let end_x = x.saturating_add(width.min(i32::MAX as u32) as i32);
    let end_y = y.saturating_add(height.min(i32::MAX as u32) as i32);
    for py in y..end_y {
        for px in x..end_x {
            blend_pixel(
                screen,
                px,
                py,
                prepared.sample(px as f32 + 0.5, py as f32 + 0.5),
                1.0,
                options,
            );
        }
    }
}
