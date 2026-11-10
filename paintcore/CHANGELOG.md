# Changelog

## 0.0.31 - 2026-11-10

- Update the registry dependencies to WML2 0.0.44 and optional fontcore 0.0.15,
  integrating bin-rs 0.0.11 at their reader boundaries.
- Keep the paintcore API and feature flags unchanged. Rust 1.91 remains the
  minimum supported version. Consumers passing fontcore types across crate
  boundaries should use fontcore 0.0.15 alongside paintcore 0.0.31.

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
