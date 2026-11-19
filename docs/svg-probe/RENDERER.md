# Bounded SVG renderer

Current verified status: 21 of 23 pinned Noto sample assets pass reference
comparison, with two explicit DTD rejections. The published fontcore dependency
also enables single-face text drawing with real OpenType SVG fonts. Earlier
sections preserve the incremental implementation history.

`svg-font` enables a private SVG adapter in paintcore. fontcore still owns font
parsing, document extraction and glyph metrics. The adapter translates SVG into
existing paintcore path/paint primitives; it does not add a public command format
or a new project crate.

The first target is the Noto grinning-face source used by the extraction probe:
filled paths with relative/absolute M/L/H/V/C/S/Q/T/Z, inherited fill/fill-rule,
affine transforms and user-space linear/radial gradients. Noto's transformed
radial gradient and relative smooth cubic paths exercise this boundary.

This is **not general SVG support**. Arcs, stroke painting, CSS/stylesheets, use,
clip/mask/filter, group opacity, bitmap/external resources and nested viewports
remain explicitly unsupported. Unsupported rendering features must return an
error rather than disappear. The parser bounds input size, nesting and geometry;
it does not resolve network or filesystem resources.

Raw SVG is a full-document alternative to approximate Path layers, not an extra
layer. When raw SVG is present, the adapter uses it instead of the approximations.
Ambiguous multiple raw documents or bitmap/raw mixtures fail. The entire run's
SVG is parsed before pixels are changed; this does not make unrelated bitmap
errors transactional. No parser/dependency types appear in public APIs; only the small explicit affine-transform wrapper is exposed.

## Dependencies and licensing

Only `svg-font` enables `roxmltree 0.20` and `svgtypes 0.15`. Both crates declare
`MIT OR Apache-2.0` (order reversed in svgtypes). The manifests and bundled license
texts from crates.io were inspected. roxmltree parses XML without DTD resolution;
svgtypes tokenizes path/transform/color syntax. Rasterization remains paintcore's.
The fontcore parser was read for boundary context, not copied wholesale.

Artwork used by the optional probe stays under its original licenses; see
[the probe instructions](README.md). Generated images/fonts are not committed.

## Initial standalone reference check (2026-11-17)

The unmodified Noto `emoji_u1f600.svg` from the fixture corpus renders its complete
face through `svg_document_probe`: eyes, mouth, teeth, tongue, outer shading and
radial fill were visually checked. At 256×256, paintcore has 42,077 nontransparent
pixels versus CairoSVG's 42,060. Exact RGBA differs in 2,394 pixels; whole-image
mean absolute channel error is 0.1381 on the 0–255 scale. This is a measured
comparison, not a universal conformance tolerance. Antialiasing differs.

```sh
cargo run -p paintcore --features svg-font --example svg_document_probe -- \
  ../fontcore/tests/fixtures/svg-extraction/noto-emoji_u1f600.svg \
  docs/svg-probe/output/noto-direct.png 2 0
```

## Typed font integration, not legacy bounds

Legacy `SvgGlyphLayer.view_box_*` describes layout bounds (advance/ascender), not
an SVG viewport. Neither those fields nor SVG data attributes can provide the
font-unit scale safely. Existing `draw_glyph`/`draw_glyphs` therefore still reject
raw SVG. The new `SvgRenderTransform` is an explicit affine matrix from root
viewport units to positioned-run pixels, including glyph origin and layer offsets.
`draw_glyphs_with_svg_transforms` requires one optional entry per glyph and never
adds those origins twice. `draw_svg_document` provides a standalone entry point.

The additive fontcore `shape_with_svg_context` companion API supplies typed
per-layer matrices without changing existing `shape`, public layer fields, or SVG
strings. The optional harness targets a caller-supplied fontcore checkout rather
than changing paintcore's normal published dependency:

```sh
python3 tools/run_svg_context_probe.py ../fontcore docs/svg-probe/output
python3 tools/compare_svg_probe.py docs/svg-probe/output --check
```

Both cases pass the extraction → shape/context → raw SVG adapter → paintcore
pixel path. Noto matches the standalone results above. Twemoji's copyright-symbol
glyph produces 21,130 nontransparent pixels versus 21,126 reference pixels, with
RGBA MAE 0.1612/255 and 2,457 exact-difference pixels. The optional corpus gate is
MAE ≤1/255 and coverage difference ≤256 pixels for each of these two 256px
fixtures; it is not a universal SVG conformance test. Unsupported SVG remains a
pre-draw error. Neither successful example establishes full emoji corpus support.

Verified fontcore context revision:
`4c0ee63b1b0d54d1c447e1bc24de7c5d7e2b6e05`.

## Second stage: fixed 20-artwork Noto sample

`tools/noto-corpus.json` pins each input SHA-256 at the same Noto revision. These
are 20 representative standalone assets wrapped into synthetic SVG fonts, **not
the entire Noto corpus**. First-stage E2E passed two; the expanded renderer passes
15 image comparisons. Supported sample MAE is below 0.24/255. The rest remain
explicitly unsupported: tulip/thumbs-up clip/use, UFO element opacity, dog/rocket
DTD. A comparison failure or accidental success on an unsupported fixture fails
the corpus gate; merely producing some pixels never counts as support.

```sh
python3 tools/run_noto_corpus.py ../fontcore docs/svg-probe/output --download --check
```

The source SVG bytes are embedded unchanged in the synthetic font. CairoSVG's
reference alone uses Python ElementTree canonicalization of checksum-verified
sources to expand Adobe's fixed internal namespace entities; Cairo's unsafe mode
is never enabled. DTD remains rejected by fontcore/paintcore. Asset/license details
are in the manifest; generated SVG/fonts/PNGs stay in ignored output.

Additional support: circle/ellipse/rect (no rounded corners), line/polyline/polygon,
stroke width/cap/join/miter under similarity transforms, gradient href inheritance
(local, bounded and cycle-checked), stop opacity, and definitions outside `defs`.
Known Adobe metadata is accepted only where rendering-inert in this subset.
Nonuniform/skewed strokes, clip/mask/filter, use, element/group opacity, arcs and
external references still fail. No public path-command fields were expanded;
private prepared layers carry full stroke style.

SVG colors now retain exact ARGB alpha, avoiding the legacy glyph converter's
RGB/RGBA guessing. This matters for fully transparent gradient stops and is
limited to the new SVG path; existing glyph behavior stays unchanged. The new path
also shifts gradient coordinates together with geometry for screen offsets.

## Third stage: clip/use and leaf opacity

The fixed sample now passes **18/20** reference comparisons. Tulip and thumbs-up
use a user-space `clipPath` containing one local `use` of a path; UFO uses leaf
opacity. All 18 have RGBA MAE below 0.33/255 at 256px. Dog/rocket still fail
explicitly because their source documents contain DTD declarations. This is a
sample result, not a statement that all Noto Emoji or all SVG features work.

Group clipping is intersected with inherited clips in the coordinate system where
the clip was declared, not transformed again by descendants. Clip masks follow
screen offsets. Clip stacks/geometry are bounded. Supported clip definitions have
one shape or one local one-hop `use` reference; `clip-rule` is preserved.

Leaf opacity is supported when the element paints only fill or only stroke.
Group opacity and simultaneous fill+stroke opacity below one remain rejected,
because correct overlap needs isolated compositing. Other remaining limitations:
objectBoundingBox/multi-child/nested clip definitions, recursive/external/general
painted `use`, masks/filters, CSS, text/images, arcs, rounded rects, and nonuniform
strokes. Unsupported rendering features fail during preparation before drawing
any glyph in the run. Existing bitmap/decode failures are not transactional.

Fontcore integration revision for the expanded sample:
`7d04cb51acfc2ade5a0ce5a2e57401ae16243027`.

## Fourth stage: isolated opacity and painted local use

General local `use` of shapes, groups and other `use` elements now expands with
inherited styles and correct instance transforms. External references, symbols,
nested SVG viewports and unsupported content still fail explicitly. Expansion
checks cycles and limits use depth to 32, tree depth to 128, visits to 16,384,
paint nodes to 4,096 and geometry/gradient commands to 100,000 (including hidden
geometry). Clip definitions retain the narrower third-stage subset.

Group opacity, element fill+stroke opacity and clipped groups use isolated RGBA
surfaces, then composite once. This follows SVG's object/group compositing model:
https://www.w3.org/TR/SVG11/masking.html . No public command contract or production
rendering crate was added. Preparation and scratch allocation happen before any
SVG/run drawing. Isolation is limited to depth 16, 64 MiB simultaneous scratch,
and 64 million cumulative full-canvas group pixels. These conservative limits
can reject large canvases; bounded subregion surfaces remain future work.

The sample is now **21 rendered / 23 inputs**: glasses, test tube and petri dish
add real Noto group-opacity coverage; DTD dog/rocket remain rejected. All rendered
Noto samples have RGBA MAE below 0.33/255 against CairoSVG. A separate scan of 100
assets found these three group-opacity examples but no general painted `use`;
this does not prove absence throughout Noto. See `noto-group-discovery.json`.

Seven authored semantic fixtures cover overlapping/nested groups, fill+stroke,
clipping and transformed/styled/nested `use`. The independent primary reference
is pinned resvg 0.45.1 (test-only adapter, no production dependency). All pass:
RGBA MAE at most 0.199/255, coverage delta at most 76 pixels at 256px. CairoSVG
is also measured but differs on fill+stroke opacity, nested use opacity and
use/clip translation; these disagreements are preserved in the results, not
silently normalized away. Six fixtures additionally pass synthetic-SFNT →
fontcore extraction → typed context → paintcore comparison. The overlap fixture
has both legacy Path and SVG alternatives and matches exactly, verifying no
duplicate painting. The seventh use/clip case is standalone renderer coverage.

```sh
python3 tools/run_svg_semantics.py docs/svg-probe/output/semantics --check
```

Remaining unsupported features include masks/filters, CSS, text/images, path
arcs, rounded rects, nonuniform strokes, objectBoundingBox/multi-child/nested
clip definitions and external resources. Unsupported features remain errors,
not partial rendering. Fontcore integration SHA remains
`7d04cb51acfc2ade5a0ce5a2e57401ae16243027`.


## Elliptical path arcs

Paths and clip paths accept absolute `A` and relative `a`, including implicit
repetitions. SVG radius correction, x-axis rotation and both flags are applied
in local coordinates. Identical endpoints omit the segment; a zero radius
produces a line. Curves are lowered into at most eight cubic segments of up to
45° before applying the element transform. Each emitted segment counts toward
the existing limits. Malformed or out-of-bounds geometry still aborts SVG
preflight without drawing. No public path or glyph API changed.

With the FontReader picosvg scoping fix
(`19697f9588899627426be2b28aaa14127d267e19`), all 13 picosvg runs from the [real-font check](#real-font-check)
produced pixels. The three runs newly enabled by arcs were compared at 256 px
against headless Chrome, using the same scoped SVG document, text origin
(16, 288), and viewport-to-pixel scale (256/1200) on a 660×384 canvas:

| Sample | Paint pixels (alpha > 0) | Chrome pixels (alpha > 0) | Alpha > 127 overlap (IoU) | Premultiplied RGBA MAE (0–255) |
| --- | ---: | ---: | ---: | ---: |
| 🛸 | 33,342 | 33,286 | 99.73% | 0.0721 |
| 🧪 | 18,780 | 18,759 | 99.85% | 0.0355 |
| 👩‍🚀 | 40,976 | 40,919 | 99.86% | 0.0997 |

The other 10 runs already drew with the scoping fix before arc support, and
some use existing Path fallback rather than raw SVG. These are sample checks,
not full picosvg or OpenType SVG conformance.

## Published dependency and ordinary text APIs

The workspace uses registry `fontcore 0.0.20`; no Git or local dependency
override is needed. With the `svg-font` feature, native
`draw_text_with_options` and Wasm `Universe.drawText` /
`Universe.drawTextFlow` (`horizontal` and `vertical`) take the typed
placement from fontcore for a loaded single face. Callers still supply the normal
text origin; the SVG glyph origin and layer offset are not applied a second time.
An unsupported SVG stops the run before changing the destination. Raw
`draw_glyphs`, family fallback, mapped layouts and mixed/sideways flow do not
acquire raw SVG placement through this integration.

### Real-font check

These are **unmodified fonts**, not synthetic SFNTs. Fixture source:
[Google Fonts color-fonts, revision e830e042](https://github.com/googlefonts/color-fonts/tree/e830e042b0e93fd5b0197a7fe9f0398f6d87f467/fonts).
No upstream binaries are committed here.

| Font | SHA-256 |
| --- | --- |
| `noto-untouchedsvg.ttf` | `90eca50e0696d71b1d8ac2c6a635d7972e747759e70d47a10b4db896490c9d3f` |
| `noto-untouchedsvgz.ttf` | `0cd71a72d01a9f3064c9e60a75af68ae04a1f3e12a0dde839faff4fdeeabcb5e` |
| `noto-picosvg.ttf` | `51f4a7aef6b813f467b4966bfb8a80bc4963815ea25caaa7d26534a336bbcfce` |

The untouched SVG and SVGZ at 256 px produced identical native pixels in the
13 sampled runs: 😀 with and without VS16, 🙂, 🌈, 🌷, 👍, 🛸, 🧪, 🐶, 🚀, 👓, 👩‍🚀
and 😀👍. The dog/rocket SVGs embedded by this upstream build are not the
DTD-bearing standalone inputs described above. The optimized picosvg font is
not interchangeable: 6 of the 13 runs rendered, and 7 stopped in fontcore SVG
extraction with `Unsupported` rather than drawing incomplete fallback art. The
large picosvg document covering 😀 and several other sampled glyphs has 73,943
local references and exceeds fontcore's 65,536 whole-document reference limit.

On Chrome 157, the ordinary Wasm APIs rendered both orientations of 😀👍 through
`ImageData` onto a Canvas: 76,615 nontransparent pixels in each, with horizontal
ink bounds (34, 93)–(535, 332) at origin (16, 288) and vertical bounds
(298, 31)–(531, 520) at origin (280, 220). The horizontal APIs and both untouched
font encodings agreed pixel-for-pixel. A 👩‍🚀 ZWJ run produced 40,972 visible pixels
within one glyph width. A synthetic run with an unsupported filter verified that
existing pixels were unchanged on error. These are targeted integration and
positioning checks, not full-font conformance.

To reproduce the core pixel checks after building web bindings with `svg-font`:

```sh
PAINT_TEST_SVG_FONT=/path/to/noto-untouchedsvg.ttf \
PAINT_TEST_SVGZ_FONT=/path/to/noto-untouchedsvgz.ttf \
node --test tests/web-paint/font-text-wasm.test.cjs
```

With only the repository's synthetic fixture, set `PAINT_TEST_SVG_WASM=1`
instead. Native SVG text tests need no external assets.
