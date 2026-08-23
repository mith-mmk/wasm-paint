//! Alpha compositing and artistic blend modes.

use crate::{canvas::Screen, paint::Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompositeOp {
    Clear,
    Source,
    Destination,
    #[default]
    SourceOver,
    DestinationOver,
    SourceIn,
    DestinationIn,
    SourceOut,
    DestinationOut,
    SourceAtop,
    DestinationAtop,
    Xor,
    Lighter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
}

impl BlendMode {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "normal" | "source-over" => Some(Self::Normal),
            "multiply" => Some(Self::Multiply),
            "screen" => Some(Self::Screen),
            "overlay" => Some(Self::Overlay),
            "darken" => Some(Self::Darken),
            "lighten" => Some(Self::Lighten),
            "color-dodge" | "colordodge" => Some(Self::ColorDodge),
            "color-burn" | "colorburn" => Some(Self::ColorBurn),
            "hard-light" | "hardlight" => Some(Self::HardLight),
            "soft-light" | "softlight" => Some(Self::SoftLight),
            "difference" => Some(Self::Difference),
            "exclusion" => Some(Self::Exclusion),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DrawOptions {
    pub opacity: f32,
    pub composite_op: CompositeOp,
    pub blend_mode: BlendMode,
}

impl Default for DrawOptions {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            composite_op: CompositeOp::SourceOver,
            blend_mode: BlendMode::Normal,
        }
    }
}

#[inline]
fn blend_channel(backdrop: f32, source: f32, mode: BlendMode) -> f32 {
    match mode {
        BlendMode::Normal => source,
        BlendMode::Multiply => backdrop * source,
        BlendMode::Screen => backdrop + source - backdrop * source,
        BlendMode::Overlay => {
            if backdrop <= 0.5 {
                2.0 * backdrop * source
            } else {
                1.0 - 2.0 * (1.0 - backdrop) * (1.0 - source)
            }
        }
        BlendMode::Darken => backdrop.min(source),
        BlendMode::Lighten => backdrop.max(source),
        BlendMode::ColorDodge => {
            if source >= 1.0 {
                1.0
            } else {
                (backdrop / (1.0 - source)).min(1.0)
            }
        }
        BlendMode::ColorBurn => {
            if source <= 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - backdrop) / source).min(1.0)
            }
        }
        BlendMode::HardLight => {
            if source <= 0.5 {
                2.0 * backdrop * source
            } else {
                1.0 - 2.0 * (1.0 - backdrop) * (1.0 - source)
            }
        }
        BlendMode::SoftLight => {
            if source <= 0.5 {
                backdrop - (1.0 - 2.0 * source) * backdrop * (1.0 - backdrop)
            } else {
                let d = if backdrop <= 0.25 {
                    ((16.0 * backdrop - 12.0) * backdrop + 4.0) * backdrop
                } else {
                    backdrop.sqrt()
                };
                backdrop + (2.0 * source - 1.0) * (d - backdrop)
            }
        }
        BlendMode::Difference => (backdrop - source).abs(),
        BlendMode::Exclusion => backdrop + source - 2.0 * backdrop * source,
    }
}

#[inline]
fn factors(op: CompositeOp, source_alpha: f32, dest_alpha: f32) -> (f32, f32) {
    match op {
        CompositeOp::Clear => (0.0, 0.0),
        CompositeOp::Source => (1.0, 0.0),
        CompositeOp::Destination => (0.0, 1.0),
        CompositeOp::SourceOver => (1.0, 1.0 - source_alpha),
        CompositeOp::DestinationOver => (1.0 - dest_alpha, 1.0),
        CompositeOp::SourceIn => (dest_alpha, 0.0),
        CompositeOp::DestinationIn => (0.0, source_alpha),
        CompositeOp::SourceOut => (1.0 - dest_alpha, 0.0),
        CompositeOp::DestinationOut => (0.0, 1.0 - source_alpha),
        CompositeOp::SourceAtop => (dest_alpha, 1.0 - source_alpha),
        CompositeOp::DestinationAtop => (1.0 - dest_alpha, source_alpha),
        CompositeOp::Xor => (1.0 - dest_alpha, 1.0 - source_alpha),
        CompositeOp::Lighter => (1.0, 1.0),
    }
}

/// Composites one non-premultiplied RGBA pixel and returns non-premultiplied RGBA.
pub fn composite_pixel(dest: Color, source: Color, coverage: f32, options: DrawOptions) -> Color {
    let source_alpha =
        (source.alpha as f32 / 255.0) * coverage.clamp(0.0, 1.0) * options.opacity.clamp(0.0, 1.0);
    let dest_alpha = dest.alpha as f32 / 255.0;

    if options.composite_op == CompositeOp::SourceOver && options.blend_mode != BlendMode::Normal {
        let source_channels = [source.red, source.green, source.blue].map(|v| v as f32 / 255.0);
        let dest_channels = [dest.red, dest.green, dest.blue].map(|v| v as f32 / 255.0);
        let out_alpha = source_alpha + dest_alpha - source_alpha * dest_alpha;
        if out_alpha <= f32::EPSILON {
            return Color::TRANSPARENT;
        }
        let mut out = [0u8; 3];
        for index in 0..3 {
            let blended = blend_channel(
                dest_channels[index],
                source_channels[index],
                options.blend_mode,
            );
            let premultiplied = source_alpha * (1.0 - dest_alpha) * source_channels[index]
                + dest_alpha * (1.0 - source_alpha) * dest_channels[index]
                + source_alpha * dest_alpha * blended;
            out[index] = ((premultiplied / out_alpha) * 255.0)
                .round()
                .clamp(0.0, 255.0) as u8;
        }
        return Color::rgba(
            out[0],
            out[1],
            out[2],
            (out_alpha * 255.0).round().clamp(0.0, 255.0) as u8,
        );
    }

    let (source_factor, dest_factor) = factors(options.composite_op, source_alpha, dest_alpha);
    let out_alpha = (source_alpha * source_factor + dest_alpha * dest_factor).clamp(0.0, 1.0);
    if out_alpha <= f32::EPSILON {
        return Color::TRANSPARENT;
    }

    let source_channels = [source.red, source.green, source.blue].map(|v| v as f32 / 255.0);
    let dest_channels = [dest.red, dest.green, dest.blue].map(|v| v as f32 / 255.0);
    let mut out = [0u8; 3];
    for index in 0..3 {
        let premultiplied = source_channels[index] * source_alpha * source_factor
            + dest_channels[index] * dest_alpha * dest_factor;
        out[index] = ((premultiplied / out_alpha) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
    }
    Color::rgba(
        out[0],
        out[1],
        out[2],
        (out_alpha * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

pub fn blend_pixel(
    screen: &mut dyn Screen,
    x: i32,
    y: i32,
    source: Color,
    coverage: f32,
    options: DrawOptions,
) {
    if x < 0 || y < 0 || x >= screen.width() as i32 || y >= screen.height() as i32 {
        return;
    }
    let offset = ((y as u32 * screen.width() + x as u32) * 4) as usize;
    let buffer = screen.buffer_mut();
    let dest = Color::rgba(
        buffer[offset],
        buffer[offset + 1],
        buffer[offset + 2],
        buffer[offset + 3],
    );
    let out = composite_pixel(dest, source, coverage, options);
    buffer[offset] = out.red;
    buffer[offset + 1] = out.green;
    buffer[offset + 2] = out.blue;
    buffer[offset + 3] = out.alpha;
}

pub fn composite_screen(
    source: &dyn Screen,
    dest: &mut dyn Screen,
    dx: i32,
    dy: i32,
    options: DrawOptions,
) {
    let mut options = options;
    options.opacity *= source.alpha().unwrap_or(0xff) as f32 / 255.0;
    let source_buffer = source.buffer();
    for source_y in 0..source.height() as i32 {
        let dest_y = source_y + dy;
        if dest_y < 0 || dest_y >= dest.height() as i32 {
            continue;
        }
        for source_x in 0..source.width() as i32 {
            let dest_x = source_x + dx;
            if dest_x < 0 || dest_x >= dest.width() as i32 {
                continue;
            }
            let offset = ((source_y as u32 * source.width() + source_x as u32) * 4) as usize;
            blend_pixel(
                dest,
                dest_x,
                dest_y,
                Color::rgba(
                    source_buffer[offset],
                    source_buffer[offset + 1],
                    source_buffer[offset + 2],
                    source_buffer[offset + 3],
                ),
                1.0,
                options,
            );
        }
    }
}

pub fn fill_rect_with_options(
    screen: &mut dyn Screen,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    color: Color,
    options: DrawOptions,
) {
    let end_y = y.saturating_add(height.min(i32::MAX as u32) as i32);
    let end_x = x.saturating_add(width.min(i32::MAX as u32) as i32);
    for py in y..end_y {
        for px in x..end_x {
            blend_pixel(screen, px, py, color, 1.0, options);
        }
    }
}
