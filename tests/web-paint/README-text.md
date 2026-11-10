# Text in web-paint-ui.html

Horizontal text keeps the existing browser/system-font Canvas renderer.
The direction selector adds **vertical standard**, **vertical mixed LTR**, and
**sideways LTR**, rendered by fontcore 0.0.14 through Wasm, without CSS or Canvas
re-rotation. Select a local TTF/OTF/TTC/WOFF/WOFF2 font. Its bytes stay in the
page; no network upload is performed. The selected font face, not the browser
font menu or synthetic bold/italic, determines vertical text appearance.

Click the canvas, enter text, and check the actual Wasm placement preview.
The click is the conservative ink top-left anchor. Center/bottom alignment
uses the vertical ink extent. Vertical flow has no automatic wrapping; mixed
and sideways are single-face, single-line LTR outline APIs. Tabs, line breaks,
unsupported color/SVG/raster layers, and other fontcore errors are surfaced.
Standard vertical mode uses fontcore vertical flow, including its newline rules.
The editing textarea stays horizontal deliberately; the preview shows output.

Ctrl+Enter or Place creates an image layer using the existing commit/history
path. Esc/cancel leaves the document unchanged. Errors preserve the draft for
correction. After placement text is rasterized, not editable text; undo removes
the new layer and redo restores it, as with existing horizontal text. PNG export
includes the committed layer. Font selections are not persisted in PNG files.

Build Wasm with `font` (optionally `svg-font`), using the wasm-bindgen CLI version
matching Cargo.lock. Existing `drawText`, `drawTextFamily`, and inspection APIs
are unchanged. Additive APIs are `drawTextFlow(text,x,y,size,argb,flow)` and
`measureTextFlow(text,size,flow)` (four conservative ink bounds, or empty).
Supported flow identifiers: `horizontal`, `vertical`, `mixed`, `sideways`.
Drawing coordinates use the shaped origin; UI adjusts bounds once for anchoring.

Tests: `node --test tests/web-paint/font-text.test.cjs` (no generated bindings
needed), and `node --test tests/web-paint/paint-tool.test.cjs` after generating
`wasm-paint/pkg` web bindings. Serve the repository over HTTP, open
`tests/web-paint-ui.html`, select a Japanese font, and compare 日本語ABC123（） in
each direction. Check no-font/invalid-font/multiline errors, draft correction,
cancel, commit, undo/redo and PNG export. No font assets are redistributed here.

The UI limits font files to 64MiB, text to 4096 UTF-16 code units, and vertical
render surfaces to 16M pixels (each side at most 8192). Pending font replacements
block placement, and failed reads require re-selection instead of using stale bytes.
The entire multiline ink block is left-anchored at the click; adding a column
on the left can therefore shift the first column right within that block.
For real Wasm pixel tests, set PAINT_TEST_FONT to a local Japanese font path and
run `node --test tests/web-paint/font-text-wasm.test.cjs` after generating bindings.
