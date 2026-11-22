# Changelog

## 0.0.35 - 2026-11-22

- Update optional `fontcore` dependency to `0.0.22`.
- Add `MappedWrappedLineLayout` (`shape`, `shape_family`, `shape_composite_ltr`)
  and `draw_mapped_wrapped_lines` in `mapped_text` for width-constrained greedy
  line wrapping with caret and hit-testing preservation across single-face,
  `FontFamily`, and `CompositeFont` sources.
- Extend `MappedHardLineLayout` with `shape_family` and `shape_composite_ltr`.

WML2 remains at 0.0.45 and the minimum Rust version stays at 1.91.
The wasm-paint wrapper is not published as part of this release.

## 0.0.34 - 2026-11-19

- Add opt-in OpenType SVG glyph rendering through published fontcore 0.0.21.
  Supported SVGs include paths with elliptical arcs, user-space gradients,
  local use and clips, and isolated opacity groups. Unsupported SVG is rejected
  before drawing the run.
- Carry typed SVG placement through ordinary text, family fallback, mapped
  layouts, and mixed vertical and sideways text. Detached raw SVG runs still
  require an explicit transform. Existing outline layout APIs remain available.
- Retain the original fontcore layouts in the mapped-text adapter for caret
  and hit testing while reusing the converted paintcore run for drawing.

WML2 remains at 0.0.45 and the minimum Rust version stays at 1.91.
The wasm-paint wrapper is not published as part of this release.

## 0.0.33 - 2026-11-11

- Update registry dependencies to bin-rs 0.1.0, WML2 0.0.45, and optional
  fontcore 0.0.16, aligning reader adapters across the dependency graph.
- No paintcore implementation or signatures are edited. Feature flags and the
  Rust 1.91 minimum remain unchanged. Callers sharing WML2 or fontcore types
  must align their direct dependencies, or use paintcore re-exports where
  available, because these dependency updates change public Rust type identities.

The wasm-paint wrapper is not published as part of this release.

## 0.0.32 - 2026-11-11

- Add checked RGBA8 pixel access and mutation, exact-size pixel iterators,
  zero-copy read-only rectangular views, and independent owned crops. Bounds
  are checked without clipping; crops preserve hidden RGB at zero alpha and
  omit source metadata.
- Add explicit decode-limit memory, native reader, and path helpers, and
  re-export the original WML2 error, warning, and limit types. Limits are
  independent budgets, not a total peak-memory cap.
- Use bin-rs 0.0.11 directly for reader adapters while retaining WML2 0.0.44
  and optional fontcore 0.0.15. The legacy `load_from_memory` reader path and
  truncated-input error behavior remain unchanged.

These APIs are additive. Feature flags and the Rust 1.91 minimum are unchanged.
The wasm-paint wrapper is not published as part of this release.

## 0.0.31 - 2026-11-10

- Update the registry dependencies to WML2 0.0.44 and optional fontcore 0.0.15,
  integrating bin-rs 0.0.11 at their reader boundaries.
- No paintcore implementation or signatures are otherwise edited; feature flags
  and the Rust 1.91 minimum remain unchanged. The dependency updates change
  public Rust type identities. Callers sharing WML2 or fontcore types must align
  direct dependencies to WML2 0.0.44 and fontcore 0.0.15, or use paintcore
  re-exports where available.

This release does not include the independent RGBA view work in PR #2 or the
other feature work in PR #4. The wasm-paint wrapper is not published as part of
this release.

## 0.0.30 - 2026-11-10

- Add image-rs-style RGBA8 load, save, resize, rotate, flip, and Layer conversion
  workflows through WML2 0.0.42. This is functional convenience, not image-rs
  source or type compatibility.
- Use published fontcore 0.0.14 for optional font support. Add sideways and mixed
  vertical layout helpers and re-export the font engine and direction types.
  Existing horizontal APIs remain available.
- Ignore no-ink glyphs when computing text ink bounds.
- Add deterministic synthetic-font tests and a vertical text rendering example.

Rust 1.91 is the minimum supported version. Consumers passing fontcore types
across crate boundaries should use fontcore 0.0.14 alongside paintcore 0.0.30.
Mixed/sideways layout is single-face, single-line LTR outline rendering, not
complete CSS text layout, BiDi, font fallback, or tate-chu-yoko.

The repository's web-paint UI also adds local font selection and vertical text
preview/placement. These web assets are not part of the paintcore crate. Browser
end-to-end file selection and export remain unverified pending local-file
permission; native, Wasm, and JavaScript automated checks cover the release.
The independent RGBA view work in PR #2 is not included in this release.
