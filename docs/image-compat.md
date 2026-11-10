# Image-rs-style API, stage 1

The goal is functional compatibility with common image-rs workflows while
keeping paintcore's existing internal implementation. This is not Rust source,
type, trait, or pixel-for-pixel compatibility with the `image` crate. No `image`
dependency is added.

## Existing capabilities and the gap

Image loading already exists in `image::draw_image`, `decode_image_layer`, and
`draw_image_fit_screen`. `draw_image` already returns decoder warnings; the two
convenience helpers discard them. `load_from_memory` is another workflow entry
point using the same callback, not a new decoder or new format support.

Before this change, paintcore had no public encoding/writer helper of its own.
The WASM application's `image_encoder` and `image_encoder_select_canvas` already
encoded JPEG through WML2, returning a sentinel on error. The new
`encode_to_vec(format)` and `write_to(writer, format)` expose format selection
and propagated errors to paintcore callers. Callers can pass a file to `write_to`;
there is still no path-based `save` method or extension-based output policy.
The checked owned raster, native reader/path helpers, and processing methods are
an optional convenience boundary over existing internals, not missing processing
or decoding engines.

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
there is no guarantee of identical pixels to image-rs resamplers. Exact resizing
uses a zero-origin affine transform and independent horizontal/vertical scales.
It always uses the selected sampler, unlike the legacy uniform `Affine::resize`
helper's automatic pixel-mixing path below 0.5 scale.

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

### WML2 0.0.43 follow-up

The workspace now requires published WML2 0.0.43, including the public error and
warning accessors from #106. Existing codec feature selection is unchanged.
The compatibility API adds `load_from_memory_with_limits`, native
`read_with_limits`, and `open_with_limits`. Native `read` and `open` delegate with
`DecodeLimits::default()`. Existing `load_from_memory` retains its original
`draw_image` reader path, including its low-level error behavior. All existing
entry points keep default budgets, signatures, first-frame behavior, metadata,
warnings, and boxed errors. `DecodeLimits` is WML2's existing type, not a parallel
paintcore policy. Each call scopes the budget independently, including after failure.

The new `load_from_memory_with_limits` path borrows a `Cursor<&[u8]>` through
the same `bin-rs 0.0.10`
`StreamReader` adapter already used by WML2. This avoids an input-sized reader
copy and still decodes directly into the existing paintcore Layer callback.
`bin-rs` becomes a direct dependency for this boundary, not a new codec or drawing
engine. Native readers continue to stream; path format detection uses contents.

For normal inputs the new memory API preserves pixels, metadata and warnings.
Its borrowed `StreamReader` can produce different low-level truncation errors
from the existing API's copying `BytesReader`: for example, a truncated BMP
header returns `std::io::Error` with `UnexpectedEof` in the new API, while
`load_from_memory` continues to return `Other`. Error text can also differ.
WML2 error kinds and decoder warnings are not remapped.

`ImgError`, `ImgErrorKind`, `DecodeLimitExceeded`, `ImgWarning`, and `ImgWarnings`
are re-exported from this module so callers can inspect the original types:

```rust
use paintcore::image_compat::{load_from_memory_with_limits, DecodeLimits,
    DecodeLimitExceeded, ImgError};
let limits = DecodeLimits { input_bytes: 1024 * 1024, ..DecodeLimits::default() };
match load_from_memory_with_limits(bytes, limits) {
    Ok(decoded) => {
        if let Some(warnings) = decoded.warnings {
            for warning in warnings.iter() { eprintln!("{warning}"); }
        }
    }
    Err(error) => {
        if let Some(image_error) = error.downcast_ref::<ImgError>() {
            eprintln!("{:?}", image_error.kind());
        }
        if let Some(limit) = error.source()
            .and_then(|source| source.downcast_ref::<DecodeLimitExceeded>()) {
            eprintln!("{}: {} > {}", limit.resource(), limit.requested(), limit.limit());
        }
    }
}
```

Not every boxed error is an `ImgError`, and not every codec-specific bound uses
`DecodeLimitExceeded`. Legacy `OutOfMemory` includes policy-limit failures as well
as other failures. The limit's resource label is human-readable, not a stable
machine identifier. Warnings retain order and individual display values but have
no common typed category/severity contract. Neither errors nor warnings promise
whole-file validation. Limits are independent budgets, not a total allocation cap.
Use `DecodeLimits::unlimited()` only as an explicit caller policy.


## Validation and next stages

Regression tests cover checked/raw ownership, invalid dimensions, PNG alpha and
hidden RGB, recoverable warnings, metadata separation, original error types,
writer failures, native readers/paths, default codec roundtrips, and equality to
the existing processing routines. Test both native paintcore and the WASM
workspace, including optional font features. CI completion is not a delivery gate.

Work on checked crop/views is tracked in PR #2, and custom decode limits and
WML2 diagnostics in PR #4. Further stages add more operation adapters.
Add Gray/RGB/RGBA16 and explicit color conversion separately before HDR/F32.
Keep codecs in WML2 and UI/document/undo concerns outside this compatibility API.

## Local verification, 2026-11-08

Windows x86_64, Rust/Cargo 1.99.0; isolated checkout based on
`f4b4bd9776c35230b674ff3be674d0760137471e`:

- Baseline WML2 0.0.23: 94 tests and 2 doctests passed.
- Published WML2 0.0.42, `cargo test -p paintcore --no-default-features`:
  110 tests and 4 doctests passed.
- Published WML2 0.0.42, `cargo test -p paintcore --all-features`:
  119 tests and 4 doctests passed; 6 existing external-font tests ignored.
- `cargo check --workspace --features font,svg-font --target wasm32-unknown-unknown`
  passed. This is compilation, not a browser/runtime test.
- Initial local WML2 PR #106 patch at `60bade9c3a08a1ded3543c709cbf4975b462a906`
  and merged main `0ff8ac0d97f419c4ed0fc3c00a8b658ad14cb3b6`:
  the same all-features 117 tests and 4 doctests passed, with 6 ignored.
  No local path/Git override is committed.
- After the resize correction, a local patch of integrated WML2 main
  `865ea616a1f8085273bcce697f4032342e5145e1` (including #104 and #106)
  passed 119 tests and 4 doctests, with 6 ignored, plus the supported WASM check.
- Independent review found that the first resize adapter used the general
  affine helper's center pivot. The 1x1 red to 2x2 nearest-neighbor test reproduced
  three transparent pixels before the fix and now passes with four red pixels.
  Added independent rectangular pixel-block, reduction, and identity checks;
  uniform 0.5/1/2 scales also match the existing zero-origin `Affine::resize`.
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
