# paintcore

`paintcore` is a standalone Pure Rust raster drawing and image-processing engine. It does not
depend on a GPU backend or an external rasterizer. The existing `u32`, `Pen`, `Canvas`, `Screen`,
`Layer`, path, text, and math APIs remain available alongside the typed painting APIs.

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
