# OpenType SVG extraction → paintcore probe

For the subsequent bounded renderer and typed-coordinate E2E path, see
[RENDERER.md](RENDERER.md). The baseline failure measurements below are historical.

This is a **failure-reproduction harness**, not a claim of SVG support. It uses
fontcore's public load → shape → GlyphRun conversion → paintcore draw path.
The synthetic font cmap maps A to glyph 5; VS16 selects the color presentation.

## Reproduce

Use Rust 1.91+, Python with fontTools, CairoSVG and Pillow, and a sibling fontcore
checkout at `6c4aa44882e12c45776166c16b36283ce6786587` (or the newer fail-closed
follow-up). The artwork is read from that checkout, not downloaded by the scripts.

```sh
FONTCORE=$(realpath ../fontcore)
python3 tools/prepare_svg_probe.py "$FONTCORE" docs/svg-probe/output
cargo run -p paintcore --features svg-font --example svg_font_probe \
  --config "patch.crates-io.fontcore.path='$FONTCORE'" -- \
  docs/svg-probe/output/noto.ttf docs/svg-probe/output/noto 0 \
  > docs/svg-probe/output/noto.json
cargo run -p paintcore --features svg-font --example svg_font_probe \
  --config "patch.crates-io.fontcore.path='$FONTCORE'" -- \
  docs/svg-probe/output/twemoji.ttf docs/svg-probe/output/twemoji 212.5 \
  > docs/svg-probe/output/twemoji.json
python3 tools/compare_svg_probe.py docs/svg-probe/output
cargo test -p paintcore --all-features
```

Noto uses the standalone grinning-face artwork, **not an official Noto SVG font**.
Twemoji uses glyph 5's SVG document extracted from TwitterColorEmoji-SVGinOT.ttf
v15.1.0, rewrapped in a minimal synthetic SFNT. This isolates the rendering
boundary; it does not test original font loading, cmap selection, or shaping.
Viewports are 0 0 128 128 and 0 -1700 2048 2048 respectively, at 256×256 pixels.
CairoSVG references are visual diagnostics, not exact antialiasing goldens.

Artwork attribution and licenses remain in fontcore's
`tests/fixtures/svg-extraction/README.md`: Noto copyright Google, Apache-2.0;
Twemoji artwork Twitter, Jason Sofonia, Justine De Caires, Brad Erickson,
CC-BY-4.0. Generated artwork/fonts are ignored, not redistributed here.

## Observed results, 2026-11-17

With fontcore 6c4aa448 and paintcore main 91e45e30 before this patch:

| Fixture | Path / raw SVG layers | draw result | nontransparent pixels | reference |
|---|---:|---|---:|---:|
| Noto | 1 / 0 | Ok, incomplete orange crescent | 2,117 | 42,060 |
| Twemoji | 0 / 1 | Ok, **silent no-op** | 0 | 21,126 |

Noto has 10 source paths. Its legacy path conversion drops unsupported path
commands without forcing a raw fallback. Exact RGBA differences from the
reference: Noto 40,788 pixels; Twemoji 21,126 pixels. Neither renders correctly.

This patch makes Twemoji return `io::ErrorKind::Unsupported`, preserving the
whole destination. It also rejects mixed Path + raw SVG glyphs and runs with a
later raw SVG glyph before drawing anything. Filtering raw layers out is only a
probe diagnostic, **not** an application workaround. Noto's path-only partial
success requires the separate fontcore safety fix; paintcore cannot infer missing
source paths from a path-only GlyphRun.

## Next implementation boundary

- fontcore must retain raw fallback whenever conversion is incomplete.
- Path approximation and full raw SVG must become explicit **alternatives** (or
  be selected once), not two layers painted consecutively.
- A real SVG backend needs path grammar, inherited styles/transforms, defs/use
  dependency resolution, viewport/baseline handling and gradients for this
  corpus. Clip/mask/opacity require explicit support or rejection, not omission.
- Compare extraction output against the original SVG and rendered pixels against
  references; visible nonempty pixels alone never establish success.

The three `svg_unsupported` tests protect the safety boundary without introducing
a renderer dependency or changing the public command representation. Path and
raster-only runs retain their existing behavior. Raw SVG errors are intentional
behavior changes from the previous false-success stub.
