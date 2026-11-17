# First bounded SVG renderer

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
