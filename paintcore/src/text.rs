use crate::canvas::Screen;
use crate::line::line_antialias;
use crate::path::{GlyphBounds, GlyphRun};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDecoration {
    Underline,
    Overline,
    LineThrough,
}

#[derive(Debug, Clone, Copy)]
pub struct TextDecorationOptions {
    pub stroke_width: f32,
    pub underline_offset: f32,
    pub overline_offset: f32,
    pub line_through_ratio: f32,
}

impl Default for TextDecorationOptions {
    fn default() -> Self {
        Self {
            stroke_width: 1.0,
            underline_offset: 2.0,
            overline_offset: 1.0,
            line_through_ratio: 0.5,
        }
    }
}

pub fn glyph_run_bounds(run: &GlyphRun, offset_x: f32, offset_y: f32) -> Option<GlyphBounds> {
    let mut bounds: Option<GlyphBounds> = None;
    for glyph in &run.glyphs {
        let glyph_bounds = glyph.glyph.metrics.bounds?;
        let current = GlyphBounds {
            min_x: offset_x + glyph.x + glyph_bounds.min_x,
            min_y: offset_y + glyph.y + glyph_bounds.min_y,
            max_x: offset_x + glyph.x + glyph_bounds.max_x,
            max_y: offset_y + glyph.y + glyph_bounds.max_y,
        };
        bounds = Some(match bounds {
            Some(bounds) => GlyphBounds {
                min_x: bounds.min_x.min(current.min_x),
                min_y: bounds.min_y.min(current.min_y),
                max_x: bounds.max_x.max(current.max_x),
                max_y: bounds.max_y.max(current.max_y),
            },
            None => current,
        });
    }
    bounds
}

pub fn draw_text_decorations(
    screen: &mut dyn Screen,
    run: &GlyphRun,
    offset_x: f32,
    offset_y: f32,
    color: u32,
    decorations: &[TextDecoration],
    options: TextDecorationOptions,
) {
    let Some(bounds) = glyph_run_bounds(run, offset_x, offset_y) else {
        return;
    };
    let x0 = bounds.min_x.floor();
    let x1 = bounds.max_x.ceil();
    for decoration in decorations {
        let y = match decoration {
            TextDecoration::Underline => bounds.max_y + options.underline_offset,
            TextDecoration::Overline => bounds.min_y - options.overline_offset,
            TextDecoration::LineThrough => {
                bounds.min_y + (bounds.max_y - bounds.min_y) * options.line_through_ratio
            }
        };
        line_antialias(screen, x0, y, x1, y, color, 0xff, options.stroke_width);
    }
}
