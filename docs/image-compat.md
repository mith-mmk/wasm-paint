# Image-rs-style API, stage 1

The goal is functional compatibility with common image-rs workflows while
keeping paintcore's existing internal implementation. This is not Rust source,
type, trait, or pixel-for-pixel compatibility with the `image` crate. No `image`
dependency is added.

## First boundary

`paintcore::image_compat` adds:

- `RgbaImage::new`, checked `from_raw`/`from_layer`, dimensions and byte access.
- Ownership transfers to/from existing `Layer` without copying pixels.
- `load_from_memory`, native `read` and `open`, returning `DecodedImage` with
  public `image` and `warnings` fields. This deliberately differs from image-rs
  so recoverable decoder diagnostics are not silently lost.
- `encode_to_vec` and `write_to`, using WML2 encoders. `write_to` encodes before
  touching the writer, does not flush/close it, and preserves I/O error types.
- `resize_exact`, `grayscale`, and `brighten`, delegating to paintcore's existing
  affine, filter, and color routines. Inputs are unchanged; results are owned.

Storage is packed straight RGBA8. Grayscale remains an RGBA8 image with neutral
RGB and unchanged alpha, not a new grayscale storage type. Empty buffers are
representable, but resizing an empty image or to empty dimensions is an error.
Resize dimensions must fit the existing affine engine's signed coordinate range.
Custom Lanczos radii are accepted from 1 through 64; zero/oversized radii and
non-finite custom bicubic parameters are rejected by this adapter before sampling.
Interpolation uses paintcore's semantics, including its alpha-aware sampling;
there is no guarantee of identical pixels to image-rs resamplers.

Decoding uses the existing Layer callback and returns only the first image/frame.
Use Canvas and the WML2 callbacks for animation. Source metadata remains available
for inspection on decoded rasters. Processing results do not inherit potentially
stale EXIF dimensions/orientation or source color metadata. Encoding embeds no
source metadata implicitly. Explicit metadata/color management is a later stage.

The initial encoder bridge copies pixels into the existing WML2 ImageBuffer.
Borrowed encoding, a save-path policy, crop/views, generic pixel types, image-rs
trait adapters, high precision, and animation iterators are not claimed here.

## WML2 0.0.23 to 0.0.42

The workspace dependency is updated to the published 0.0.42 release. Both
paintcore and wasm-paint inherit it. Existing default features and legacy
callback signatures remain in use. The MSRV remains Rust 1.91. WebP's backend
changes from webp-rust 0.2.1 to 0.3.2.

Default decoder budgets now apply at the existing image_loader entry point:
512 MiB input, 128 Mi pixels, 512 MiB expanded bytes, 32 MiB metadata, 10,000
frames, and 512 MiB animation bytes. These are independent budgets, not a bound
on total peak process memory. Applications requiring custom budgets should use
WML2's explicit limit APIs until a higher-level options API is added here.

Optional codecs, ImageBufferEx and high-bit-depth features are not enabled by
this version bump. Legacy Canvas/Layer storage is still RGBA8. Upgrading alone
does not make existing drawing routines retain 16-bit or HDR pixels.

The separate WML2 error/warning API PR must be merged and released before using
its unpublished API in a registry-backed paintcore release. This stage can be
tested against that PR with a local Cargo patch without committing a Git/path
dependency or claiming the unreleased API is in 0.0.42.

## Validation and next stages

Regression tests cover checked/raw ownership, invalid dimensions, PNG alpha and
hidden RGB, recoverable warnings, metadata separation, original error types,
writer failures, native readers/paths, default codec roundtrips, and equality to
the existing processing routines. Test both native paintcore and the WASM
workspace, including optional font features. CI completion is not a delivery gate.

Next, expose custom limits and structured error diagnostics after WML2's public
contract is available, then add checked crop/views and more operation adapters.
Add Gray/RGB/RGBA16 and explicit color conversion separately before HDR/F32.
Keep codecs in WML2 and UI/document/undo concerns outside this compatibility API.

## Local verification, 2026-11-08

Windows x86_64, Rust/Cargo 1.99.0; isolated checkout based on
`f4b4bd9776c35230b674ff3be674d0760137471e`:

- Baseline WML2 0.0.23: 94 tests and 2 doctests passed.
- Published WML2 0.0.42, `cargo test -p paintcore --no-default-features`:
  108 tests and 4 doctests passed.
- Published WML2 0.0.42, `cargo test -p paintcore --all-features`:
  117 tests and 4 doctests passed; 6 existing external-font tests ignored.
- `cargo check --workspace --features font,svg-font --target wasm32-unknown-unknown`
  passed. This is compilation, not a browser/runtime test.
- Local WML2 PR #106 patch at `60bade9c3a08a1ded3543c709cbf4975b462a906`:
  the same all-features 117 tests and 4 doctests passed, with 6 ignored.
  No local path/Git override is committed.
- Normal Clippy passed with existing warnings. Changed-file rustfmt and
  `git diff --check` passed. No new diagnostics refer to the new API/tests.

Two wider checks remain failing on the unchanged baseline as well as this change:

- Workspace WASM `--all-features` activates legacy `stdweb 0.4.20`, which fails
  with missing `core`/`wasm_bindgen_initialize` under the resolved toolchain.
- Strict Clippy `-D warnings` fails on pre-existing lints in circle/color/error/
  spline and existing tests. Unrelated drawing routines were not rewritten to
  silence those warnings.

The existing release `paint_bench` completed. Single-run baseline/head timings
were 1.136/1.143 s for UHD gradient, 0.521/0.500 s for transformed pattern, and
0.021/0.021 s for brush stroke. Allocation counts were unchanged at 3/0/330.
This is a smoke measurement, not a statistically controlled performance claim.
No CI completion was awaited.
