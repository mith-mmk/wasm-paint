# paint roadmap

## Completed drawing foundations

- [x] typed Solid, linear/radial/sweep gradient, and pattern Paint
- [x] premultiplied-alpha Porter-Duff compositing, opacity, blend modes, and clip masks
- [x] gradient spread/units/transforms/interpolation and prepared stops
- [x] pattern tile modes, sampling, and transforms
- [x] checked 8-bit masks, boolean/morphology/feather operations, and Paint-based filling
- [x] configurable flood masks using RGB/RGBA tolerance, 4/8 connectivity, and reference surfaces
- [x] public path masks and stroke width/cap/join/miter/dash styles
- [x] distance-spaced pressure brushes with flow, rotation, and deterministic scatter
- [x] OpenType text and color-font support; optional SVG glyph support remains feature-gated

## Next painting stage

- gap closing for flood selections
- automatic multi-layer flood references
- smudge and wet-paint mixing
- bristle and calligraphy brush models
- stroke stabilizer and predictive input smoothing
- tilt-driven brush-tip deformation
- shadows and additional image effects

## Color and precision

- RGBA16F surfaces and high-dynamic-range compositing
- ICC profile conversion and OCIO integration
- expanded histogram, threshold, noise, and mosaic tools

## Backends and performance

- optional SIMD acceleration after scalar conformance tests
- optional GPU backend with CPU output conformance
- worker scheduling and controlled 30/60/120 Hz rendering

## Application-level work (outside paintcore)

- document model and persistence
- undo/redo history
- selection UI and editing handles
- image saver/export workflow
- trimming and borders
