# Vertical text integration verification

Verified on Windows x86_64 on 2026-11-10, using installed Rust 1.91.0 for
builds/tests, stable Rust 1.99.0 for formatting/Clippy, and wasm-bindgen 0.2.129
for generated web bindings. No toolchain or application was installed.

## Passed

- `cargo +1.91.0 test -p paintcore --all-features --offline`: 131 passed,
  6 existing external-font tests ignored.
- `cargo +1.91.0 test -p paintcore --no-default-features --offline`: 116 passed.
- Native workspace tests with `--features font,svg-font`: 135 passed,
  6 existing external-font tests ignored.
- Six deterministic vertical integration tests use included MIT synthetic fonts:
  horizontal compatibility, vertical metrics/origins, newline columns, exactly
  one clockwise rotation, mixed CJK/Latin/marks, actual pixel rendering, preserved
  errors, and no-ink bounds. A seventh bounds regression runs without `font`.
- `cargo +1.91.0 check --workspace --features font,svg-font --target
  wasm32-unknown-unknown --offline` and default native workspace check.
- A real Wasm build and matching web bindings; `cargo tree` shows one published
  fontcore 0.0.14 shared by both crates.
- `node --test tests/web-paint/font-text.test.cjs
  tests/web-paint/font-text-wasm.test.cjs tests/web-paint/paint-tool.test.cjs`:
  30 passed with `PAINT_TEST_FONT` set to an installed BIZ UD Gothic font.
  This includes 23 helper/lifecycle tests, one actual Wasm pixel/error test, and
  six existing component history/export tests. No production font is committed.
- JavaScript syntax checks, `cargo +stable fmt --all -- --check`, PowerShell
  test-launcher syntax/required-binding checks, and `git diff --check`.
- `cargo package -p paintcore --allow-dirty --offline` verifies the package;
  checking the extracted package with `font` also passes. The normalized package
  depends on crates.io fontcore 0.0.14, not a local path or Git branch.
- Documentation build with `--all-features --no-deps`.
- Native PNG example inspected with horizontal, standard vertical, mixed and
  sideways output using BIZ UD Gothic.
- Browser: standalone UI loads, direction controls work, missing-font errors
  appear without discarding the draft.

Review caught and fixed a font-selection race (old font used while replacement
was loading) and a Wasm buffer lifetime problem (ImageData now owns copied bytes
before the temporary engine is freed). Both have regression tests.

## Existing baseline failures

- Strict Clippy `-D warnings` reports seven library diagnostics in unchanged
  circle/color/error code. The same seven reproduce on main 8164fc9; the normal
  Clippy command completes with warnings. A library-test diagnostic also remains
  in unchanged spline test code.
- Workspace Wasm `--all-features` enables legacy `stdweb 0.4.20`, which fails
  with E0433/E0425. The same errors reproduce on main 8164fc9. The requested
  `font,svg-font` configuration passes without enabling `stdweb`.

## Browser verification still pending

The connected browser requires its local-file permission before the selected
font can be loaded through the file picker. End-to-end browser font selection,
placement, draft correction, cancel, undo/redo and PNG download have not yet been
fully verified. The real Wasm and component tests above do not replace that check.

The initial verification above was recorded for Draft PR #3 before merge or release. No CI completion is claimed.
