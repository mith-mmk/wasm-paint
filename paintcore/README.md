# paintcore

[![crates.io](https://img.shields.io/crates/v/paintcore)](https://crates.io/crates/paintcore)
[![docs.rs](https://img.shields.io/docsrs/paintcore)](https://docs.rs/paintcore)

`paintcore` is a standalone Pure Rust raster drawing and image-processing engine. It does not
depend on a GPU backend or an external rasterizer. The existing `u32`, `Pen`, `Canvas`, `Screen`,
`Layer`, path, text, and math APIs remain available alongside the typed painting APIs.

## Install

```toml
[dependencies]
paintcore = "0.0.35"
```

Version 0.0.35 uses WML2 0.0.45 and optional fontcore 0.0.22. Callers sharing
these crates' public types with paintcore must align their direct dependency
versions, or use paintcore re-exports where available; these type identities
can differ from prior releases. Reader adapters use bin-rs 0.1.0.

## Features

- RGBA `Color` with explicit ARGB, RGB, and RGBA integer conversion methods
- premultiplied-alpha Porter-Duff compositing, global opacity, blend modes, and clip masks
- `Paint::Solid`, linear/radial/sweep gradients, and shared immutable image patterns
- gradient stop preparation, Pad/Repeat/Reflect spread, user-space/object-bounds units,
  affine transforms, and sRGB/linear-sRGB interpolation
- Repeat/Mirror/Clamp/Decal pattern tiling, nearest/bilinear sampling, and affine transforms
- checked 8-bit `Mask` allocation, rectangle/path masks, boolean operations, grow/shrink,
  feather, and Paint-based mask filling
- configurable flood masks with RGB/RGBA tolerance and 4/8 connectivity
- public path fill/stroke mask generation, fill rules, caps, joins, miter limits, and dashes
- stateful distance-spaced brushes with pressure mapping, opacity, flow, rotation, and seeded
  scatter; circle, rectangle, legacy `Pen`, and arbitrary Mask tips are supported
- affine transforms, layers and animation, filters, image loading through `wml2`, and optional
  OpenType/color-font rendering through `fontcore`

Feature flags:

- `font`: OpenType font support, including raster and COLR layers
- `svg-font`: OpenType SVG glyph support (verified with Noto Color Emoji)

## OpenType SVG glyphs

With `svg-font`, `draw_text_with_options`, family fallback, and mapped layouts
carry the font's SVG placement through to drawing. The sideways and mixed
vertical helpers ending in `_with_svg_context` retain placement for those
orientations. A detached raw SVG run requires explicit placement through
`draw_glyphs_with_svg_transforms`. Use `draw_svg_document` for standalone SVG.

The renderer supports a bounded subset, including elliptical paths, gradients,
local references, and opacity groups. Features such as filters, masks,
external references, and general CSS return errors.

## Vertical text (fontcore 0.0.22)

Enable `font` to use the published fontcore 0.0.22 dependency, shared with the
Wasm crate. Existing horizontal text calls keep their default direction.

```rust,no_run
# #[cfg(feature = "font")]
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use paintcore::prelude::*;

let face = load_font_from_buffer(&std::fs::read("font.ttf")?)?;
let options = FontOptions::new(&face).with_font_size(36.0);
let mut canvas = Layer::tmp(400, 800);

// Standard vertical flow; newlines move to the next column on the left.
draw_text_with_options(
    &mut canvas, "縦書き", options.clone().with_vertical_flow(),
    300.0, 30.0, 0xff20_2020,
)?;

// Opt-in single-face mixed orientation: upright CJK and clockwise Latin groups.
let engine = face.engine().with_options(options);
let run = layout_text_vertical_mixed_ltr("日本語ABC 123", &engine)?;
draw_glyphs(&mut canvas, &run, 200.0, 30.0, 0xff20_2020)?;
# Ok(())
# }
# #[cfg(not(feature = "font"))]
# fn main() {}
```

`layout_text_sideways_ltr` rotates an entire horizontal LTR line clockwise.
Both helpers take a borrowed `FontEngine`, preserve its options, and return a
paintcore `GlyphRun`. Positions, paths, bounds and advances are already resolved
in y-down pixel coordinates. Do not rotate them again. Drawing offsets are the
logical origin, not necessarily the top-left ink corner; `glyph_run_bounds`
returns ink bounds and skips spaces/no-ink glyphs.

Mixed/sideways modes deliberately retain fontcore's limits: one face and one
line, current-color unclipped fill outlines only. Tabs/line-break controls return
`InvalidInput`; unsupported color, raster or raw SVG layers return `Unsupported`
without a partial run. Mixed orientation is an explicit Unicode policy, not
full CSS layout: no BiDi, family fallback, multiline mixing, tate-chu-yoko or
rotated fallback for `Tr` characters without an alternate. Existing
`draw_text_decorations` remains horizontal-only.

Render a local font to PNG (top: horizontal; left to right: standard vertical, mixed,
sideways). No font is downloaded or redistributed by the example:

```text
cargo run -p paintcore --features font --example vertical_text -- FONT OUTPUT.png
```

## Image-rs-style workflows

`paintcore::image_compat` is an additive functional-compatibility API backed by
the existing paintcore routines and WML2 0.0.45. It is not a drop-in replacement
for image-rs and does not depend on `image`. The first stage provides checked
RGBA8 storage, memory/reader/path decoding with retained warnings, memory/writer
encoding, exact-size affine resampling, grayscale and brightness adjustment.
Checked pixel borrowing/iteration, zero-copy read-only rectangular views, and
owned crops are available since 0.0.32. Bounds are checked without silent clipping;
cropping preserves raw RGBA8 samples and leaves the source unchanged.

```rust
use paintcore::image_compat::{load_from_memory, ImageFormat, RgbaImage};

let image = RgbaImage::from_raw(1, 1, vec![255, 0, 0, 128])?;
let png = image.encode_to_vec(ImageFormat::Png)?;
let decoded = load_from_memory(&png)?;
if let Some(warnings) = decoded.warnings {
    eprintln!("{warnings}");
}
let output = decoded.image.brighten(16)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

See [the staged compatibility contract](../docs/image-compat.md) for semantics,
WML2 upgrade notes, and the next stages. Existing drawing APIs are unchanged.

## Color and alpha

`Screen::buffer()` stores straight-alpha bytes in `[R, G, B, A]` order. Compositing converts to
premultiplied alpha for calculation and returns straight RGBA, so translucent destinations retain
the correct alpha. New typed APIs never guess integer channel order:

```rust
use paintcore::prelude::*;

let red = Color::from_argb_u32(0x80ff0000);
let green = Color::from_rgb_u32(0x00ff00);
let blue = Color::from_rgba_u32(0x0000ffff);
```

Legacy numeric drawing calls keep their historical color interpretation for compatibility.

## Paint example

```rust
use paintcore::prelude::*;

let mut layer = Layer::new("art".into(), 640, 480);
let paint = Paint::LinearGradient(LinearGradient {
    start: (0.0, 0.0),
    end: (640.0, 0.0),
    stops: vec![
        ColorStop::new(0.0, Color::rgb(255, 96, 64)),
        ColorStop::new(1.0, Color::rgb(64, 96, 255)),
    ],
    spread: SpreadMode::Pad,
    units: PaintUnits::UserSpaceOnUse,
    transform: PaintTransform::IDENTITY,
    interpolation: GradientInterpolation::LinearSrgb,
});

fill_paint_rect(
    &mut layer,
    0,
    0,
    640,
    480,
    &paint,
    DrawOptions::default(),
);
```

`Paint::prepare` validates and stable-sorts gradient stops before rasterization. Pixel loops do not
clone or sort stops and do not allocate per pixel. `fill_mask`, `fill_path`, `stroke_path`, and
brush dabs all share the same Paint sampling and compositing path.

## Flood fill and brushes

`flood_mask(reference, x, y, options)` only discovers a region. Apply it to the destination with
`fill_mask`, which also permits a different reference surface from the painted surface. Existing
`fill` and `fill_with_alpha` remain exact-color, four-connected Solid-paint compatibility APIs.

`BrushStroke::push_sample` emits dabs at distance-based intervals. The same geometric sample path
therefore produces the same dab positions regardless of pointer-event frequency. Scatter uses the
seed supplied to `BrushStroke::new`.

## Performance smoke measurement

The example below measures UHD gradient/pattern fills and a long brush stroke while counting heap
allocations. Paint preparation may allocate once per operation; sampling and compositing do not
allocate per pixel.

```text
cargo run -p paintcore --example paint_bench --release
```

## Data and asset scope

- Raster buffers use four bytes per pixel in RGBA order.
- Image format support is provided by the `wml2` dependency.
- No production fonts, sample images, or web-paint assets are bundled in the crate
  package. Small MIT synthetic fonts in `tests/fixtures` support deterministic tests.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

## Decode limits and diagnostics (since 0.0.32)

The image workflow uses WML2 0.0.45. `image_compat` offers
`load_from_memory_with_limits` plus native `read_with_limits` and `open_with_limits`.
They accept WML2's re-exported `DecodeLimits`; existing calls keep default budgets.
Only the new memory helper uses a borrowed input reader; `load_from_memory`
keeps its original reader and low-level error behavior.
Decode results preserve individual warnings (`ImgWarnings::iter`), and errors
retain their original type, `ImgError::kind`, and `DecodeLimitExceeded` source.
See [the compatibility notes](../docs/image-compat.md) for semantics and limits.


## Mapped text and caret integration (`font`)

The additive `mapped_text` module uses fontcore 0.0.21. Existing `path::GlyphRun`
and `layout_text` APIs remain unchanged. `MappedTextLayout` retains an immutable
paintcore render run **and** the original fontcore layout/mappings, avoiding the
loss of source ranges during the old render-only conversion.

- `shape` / `from_shaped`: single-direction mapped text (including pre-shaped
  composite-font results); `shape_family`: family fallback with face provenance.
- `shape_hard_lines`: explicit LF/CRLF lines, including empty/trailing lines.
- `shape_bidi`: bounded horizontal BiDi paragraphs.
- `shape_vertical_mixed_ltr`: fontcore's pre-transformed vertical mixed run.
- `placed(x, y)`: use one baseline translation for both `draw`, `hit_test` and
  `caret_positions`. Results retain UTF-8 position, affinity, quality and, where
  applicable, line/group index. The render cache is built once.

`source()` provides the original source objects. Single/vertical source ranges
are global; hard-line ranges inside `shaped()` are line-local (add content start),
and BiDi ranges inside a visual run are run-local (add run byte-range start).
Caret results are always global in the original UTF-8 string. Do not treat these
indices as JS UTF-16 offsets or glyph indices. Source envelopes are conservative,
not exact scalar causality. A boundary may have multiple visual carets.

Limitations are inherited rather than hidden: ligature-internal carets are
approximate; soft wrapping, some cross-run joining and
upper-layout family fallback are unsupported. Mixed vertical is a single-face,
single-column API. Nearest-caret testing is not glyph ink containment. The adapter
supports translation only; zoom/rotation must be handled consistently by the
caller. Non-finite or overflowing query coordinates return errors.

This is a library integration, not a canvas editor or IME implementation. No
paintcore release/version bump is included in this change.
