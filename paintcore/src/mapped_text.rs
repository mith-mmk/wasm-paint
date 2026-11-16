//! Metadata-preserving text rendering and caret queries (requires `font`).
//!
//! A layout retains its original fontcore data and a read-only paintcore render
//! run. Rendering and queries share the same baseline translation. These APIs
//! do not implement editing, IME, soft wrapping or arbitrary canvas transforms.
//! ```
//! # fn example(engine: &fontcore::FontEngine<'_>, screen: &mut dyn paintcore::canvas::Screen)
//! # -> Result<(), Box<dyn std::error::Error>> {
//! use paintcore::mapped_text::{MappedTextLayout, TextPosition, CaretAffinity};
//! let layout = MappedTextLayout::shape(engine, "fi")?;
//! let placed = layout.placed(100.0, 80.0)?;
//! placed.draw(screen, 0xff000000)?;
//! let caret = placed.hit_test(108.0, 80.0)?;
//! let positions = placed.caret_positions(TextPosition::new(1, CaretAffinity::Downstream))?;
//! # let _ = (caret, positions);
//! # Ok(()) }
//! ```
//!
//! UTF-8 positions, affinity, approximation and Unsupported errors are inherited
//! from fontcore 0.0.19. Source envelopes are not exact scalar causality.
use crate::{
    path::{self, GlyphRun},
    prelude::Screen,
};
pub use fontcore::{BidiBaseDirection, CaretAffinity, CaretQuality, GlyphSource, TextPosition};
use std::io::{Error, ErrorKind};

/// The original layout, retained without flattening its source metadata.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum TextLayoutSource {
    Single(fontcore::ShapedText),
    HardLines(fontcore::HardLineLayout),
    Bidi(fontcore::BidiParagraphLayout),
    VerticalMixed(fontcore::VerticalMappedLayout),
}

/// A caret in screen coordinates. Line/group identity is available where relevant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextCaret {
    position: TextPosition,
    x: f32,
    y: f32,
    quality: CaretQuality,
    line_index: Option<usize>,
    group_index: Option<usize>,
}
impl TextCaret {
    pub fn text_position(self) -> TextPosition {
        self.position
    }
    pub fn x(self) -> f32 {
        self.x
    }
    pub fn y(self) -> f32 {
        self.y
    }
    pub fn quality(self) -> CaretQuality {
        self.quality
    }
    pub fn line_index(self) -> Option<usize> {
        self.line_index
    }
    pub fn group_index(self) -> Option<usize> {
        self.group_index
    }
}

/// Immutable render data and its original source mappings. Conversion is done
/// once, not on every draw. No additional rotation is applied to mixed vertical
/// glyphs: fontcore has already transformed their outlines and positions.
#[derive(Clone)]
pub struct MappedTextLayout {
    source: TextLayoutSource,
    glyphs: GlyphRun,
    single_carets: Option<fontcore::CaretLayout>,
}
impl MappedTextLayout {
    pub fn from_shaped(shaped: fontcore::ShapedText) -> Result<Self, Error> {
        let carets = fontcore::CaretLayout::new(&shaped)?;
        Ok(Self {
            glyphs: shaped.glyph_run().clone().into(),
            source: TextLayoutSource::Single(shaped),
            single_carets: Some(carets),
        })
    }
    pub fn from_hard_lines(layout: fontcore::HardLineLayout) -> Self {
        let mut glyphs = GlyphRun::default();
        for line in layout.lines() {
            append_run(
                &mut glyphs,
                line.shaped().glyph_run(),
                line.origin_x(),
                line.origin_y(),
            );
        }
        Self {
            source: TextLayoutSource::HardLines(layout),
            glyphs,
            single_carets: None,
        }
    }
    pub fn from_bidi(layout: fontcore::BidiParagraphLayout) -> Self {
        let mut glyphs = GlyphRun::default();
        for line in layout.lines() {
            for run in line.visual_runs() {
                append_run(
                    &mut glyphs,
                    run.shaped().glyph_run(),
                    run.origin_x(),
                    run.origin_y(),
                );
            }
        }
        Self {
            source: TextLayoutSource::Bidi(layout),
            glyphs,
            single_carets: None,
        }
    }
    pub fn from_vertical_mixed(layout: fontcore::VerticalMappedLayout) -> Self {
        Self {
            glyphs: layout.glyph_run().clone().into(),
            source: TextLayoutSource::VerticalMixed(layout),
            single_carets: None,
        }
    }
    pub fn shape(engine: &fontcore::FontEngine<'_>, text: &str) -> Result<Self, Error> {
        Self::from_shaped(engine.shape_mapped(text)?)
    }
    /// Single-direction family fallback; provenance is retained in `source()`.
    pub fn shape_family<'a>(
        family: &'a fontcore::FontFamily,
        text: &str,
        options: fontcore::FontOptions<'a>,
    ) -> Result<Self, Error> {
        Self::from_shaped(family.shape_mapped(text, options)?)
    }
    pub fn shape_hard_lines(engine: &fontcore::FontEngine<'_>, text: &str) -> Result<Self, Error> {
        Ok(Self::from_hard_lines(fontcore::HardLineLayout::shape(
            engine, text,
        )?))
    }
    pub fn shape_bidi(
        engine: &fontcore::FontEngine<'_>,
        text: &str,
        base: BidiBaseDirection,
    ) -> Result<Self, Error> {
        Ok(Self::from_bidi(fontcore::BidiParagraphLayout::shape(
            engine, text, base,
        )?))
    }
    pub fn shape_vertical_mixed_ltr(
        engine: &fontcore::FontEngine<'_>,
        text: &str,
    ) -> Result<Self, Error> {
        Ok(Self::from_vertical_mixed(
            engine.shape_vertical_mixed_ltr_mapped(text)?,
        ))
    }
    pub fn source(&self) -> &TextLayoutSource {
        &self.source
    }
    pub fn glyph_run(&self) -> &GlyphRun {
        &self.glyphs
    }
    pub fn text(&self) -> &str {
        match &self.source {
            TextLayoutSource::Single(x) => x.text(),
            TextLayoutSource::HardLines(x) => x.text(),
            TextLayoutSource::Bidi(x) => x.text(),
            TextLayoutSource::VerticalMixed(x) => x.text(),
        }
    }
    /// Place the baseline origin in screen coordinates. This is translation only;
    /// callers with a zoomed/rotated canvas must inverse-transform input first.
    pub fn placed(&self, x: f32, y: f32) -> Result<PlacedText<'_>, Error> {
        finite(x, y)?;
        for glyph in &self.glyphs.glyphs {
            finite(glyph.x + x, glyph.y + y)?;
        }
        Ok(PlacedText { layout: self, x, y })
    }
}

/// A borrowed placement guarantees drawing and hit testing use the same origin.
#[derive(Clone, Copy)]
pub struct PlacedText<'a> {
    layout: &'a MappedTextLayout,
    x: f32,
    y: f32,
}
impl PlacedText<'_> {
    pub fn draw(
        &self,
        screen: &mut dyn Screen,
        color: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        path::draw_glyphs(screen, &self.layout.glyphs, self.x, self.y, color)
    }
    pub fn caret_positions(&self, position: TextPosition) -> Result<Vec<TextCaret>, Error> {
        let carets = match &self.layout.source {
            TextLayoutSource::Single(_) => self
                .layout
                .single_carets
                .as_ref()
                .expect("single layout invariant")
                .caret_positions(position)?
                .into_iter()
                .map(TextCaret::from)
                .collect(),
            TextLayoutSource::HardLines(x) => x
                .caret_positions(position)?
                .into_iter()
                .map(TextCaret::from)
                .collect(),
            TextLayoutSource::Bidi(x) => x
                .caret_positions(position)?
                .into_iter()
                .map(TextCaret::from)
                .collect(),
            TextLayoutSource::VerticalMixed(x) => x
                .caret_positions(position)?
                .into_iter()
                .map(TextCaret::from)
                .collect(),
        };
        let carets: Vec<TextCaret> = carets;
        carets.into_iter().map(|c| self.translate(c)).collect()
    }
    /// Nearest caret, not ink containment. Off-layout coordinates clamp according
    /// to fontcore's baseline/inline rules; non-finite or overflowing input fails.
    pub fn hit_test(&self, x: f32, y: f32) -> Result<TextCaret, Error> {
        finite(x, y)?;
        let (x, y) = (x - self.x, y - self.y);
        finite(x, y)?;
        let caret = match &self.layout.source {
            TextLayoutSource::Single(_) => self
                .layout
                .single_carets
                .as_ref()
                .expect("single layout invariant")
                .hit_test(x, y)?
                .into(),
            TextLayoutSource::HardLines(l) => l.hit_test(x, y)?.into(),
            TextLayoutSource::Bidi(l) => l.hit_test(x, y)?.into(),
            TextLayoutSource::VerticalMixed(l) => l.hit_test(x, y)?.into(),
        };
        self.translate(caret)
    }
    fn translate(&self, mut caret: TextCaret) -> Result<TextCaret, Error> {
        caret.x += self.x;
        caret.y += self.y;
        finite(caret.x, caret.y)?;
        Ok(caret)
    }
}
fn finite(x: f32, y: f32) -> Result<(), Error> {
    if x.is_finite() && y.is_finite() {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::InvalidInput,
            "text coordinates must remain finite",
        ))
    }
}
fn append_run(target: &mut GlyphRun, run: &fontcore::GlyphRun, x: f32, y: f32) {
    target
        .glyphs
        .extend(run.glyphs.iter().cloned().map(|mut g| {
            g.x += x;
            g.y += y;
            g.into()
        }));
}
macro_rules! caret_conversion {
    ($ty:ty, $line:expr, $group:expr) => {
        impl From<$ty> for TextCaret {
            fn from(c: $ty) -> Self {
                Self {
                    position: c.text_position(),
                    x: c.x(),
                    y: c.y(),
                    quality: c.quality(),
                    line_index: ($line)(c),
                    group_index: ($group)(c),
                }
            }
        }
    };
}
caret_conversion!(fontcore::CaretPosition, |_| None, |_| None);
caret_conversion!(
    fontcore::LineCaretPosition,
    |c: fontcore::LineCaretPosition| Some(c.line_index()),
    |_| None
);
caret_conversion!(
    fontcore::BidiCaretPosition,
    |c: fontcore::BidiCaretPosition| Some(c.line_index()),
    |_| None
);
caret_conversion!(
    fontcore::VerticalCaretPosition,
    |_| None,
    |c: fontcore::VerticalCaretPosition| Some(c.group_index())
);
