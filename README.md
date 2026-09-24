# wasm-paint / paintcore

- `paintcore` is a standalone Pure Rust raster drawing and image-processing engine.
- `wasm-paint` exposes the engine to browsers without breaking the existing JavaScript API.

The typed painting API provides Solid, linear/radial/sweep gradient, and shared image-pattern
paints; premultiplied-alpha compositing and blend modes; checked 8-bit masks and configurable flood
selection; styled path strokes; and a distance-spaced pressure brush engine. No `vectorcore`, GPU
backend, or external rasterizer is required.

Buffers are straight `[R, G, B, A]` bytes. New WASM methods with an `argb` parameter accept
`0xAARRGGBB`; legacy numeric APIs retain their historical color contract.

See [paintcore/README.md](paintcore/README.md) for API details and examples.

## Embedded Web Paint Component

`examples/web-paint/` contains a framework-free `<wasm-paint-tool>` Web Component backed by the
existing WASM engine. It provides pointer and touch drawing, an eraser, image import, PNG export,
and basic layer controls inside a Shadow DOM.

Build the generated WASM JavaScript package, then serve the repository root over HTTP:

```sh
cd wasm-paint
wasm-pack build -t web
cd ..
python -m http.server 8000
```

Open <http://localhost:8000/examples/web-paint/> for the demo.

On Windows, run `.\test.ps1` from the repository root to start the local test server. Open
<http://127.0.0.1:8000/examples/web-paint/external-ui.html> to test the standalone UI.

To embed the component in another page, load its module and add the element:

```html
<script type="module" src="/examples/web-paint/paint-tool.js"></script>
<wasm-paint-tool width="640" height="480"></wasm-paint-tool>
```

The `width` and `height` attributes set the backing canvas size; positive integer values are capped
at 8192, and omitted or invalid values default to `640 × 480`. The canvas scales to its available
width while pointer coordinates continue to use canvas pixels. The host page can pass a `Blob` to
create a new image layer, export the visible result
as a PNG `Blob`, and listen for committed edits. Add `controls="external"` to hide the component's
toolbar and layer controls while keeping the canvas and its pointer interaction:

```js
const paint = document.querySelector("wasm-paint-tool");
await paint.setBrushColor("#315fce");
await paint.setBrushSize(18);
await paint.setEraserEnabled(false);
const layer = await paint.addLayer("Annotations");
await paint.drawStroke({ points: [{ x: 80, y: 60 }, { x: 140, y: 100 }] });
await paint.setLayerOpacity(layer, 0.8);
await paint.loadImage(imageBlob);
const pngBlob = await paint.exportImage();
paint.addEventListener("paint-change", (event) => {
  console.log(event.detail.source, event.detail.layers);
});
paint.addEventListener("paint-state-change", (event) => {
  console.log(event.detail.selectedLayer, event.detail.brushSize);
});
```

The host API also includes `getState()`, `selectLayer(name)`, `setLayerVisibility(name, visible)`,
`clearLayer(name)`, and `clearCanvas()`. See
[`examples/web-paint/external-ui.html`](examples/web-paint/external-ui.html) for a standalone page
whose complete toolbar and layer UI are outside the component.

The external UI provides a Japanese ribbon, tool rail, HSV color picker, layer panel, and zoom
controls. Pencil and brush are size presets for the existing brush engine; opacity controls the
selected layer. Image imports fit within the existing 640 × 480 canvas and create a new layer.
The File menu also opens a landscape sample. Selection, fill, text, and shape tools are explicitly
disabled because the component does not yet expose them. The eyedropper previews canvas pixels and
restores the previous tool after sampling or cancellation. Save and Save As use the browser file
picker when available; “PNGをダウンロード…” always opens the named-download flow and keeps the
document marked as changed. Imported image previews are replaced with a drawn-layer icon after editing; live layer
thumbnails are not implemented. The component exposes `workspace` and `canvas` CSS parts for host styling.

Adding the `webmcp` attribute opts that instance into WebMCP tool registration when
`document.modelContext` is available. WebMCP is an evolving proposal; Chrome currently documents
availability through an origin trial or a local development flag. The normal component UI and
JavaScript API do not depend on WebMCP. See the [Chrome WebMCP guide](https://developer.chrome.com/docs/ai/webmcp)
and [WebMCP draft specification](https://webmachinelearning.github.io/webmcp/).

# WebAssembly Test

Color model ABGR uint32LE

Typed paint regression pages:

- `tests/composite.html`
- `tests/gradient.html`
- `tests/pattern.html`
- `tests/mask.html`
- `tests/flood.html`
- `tests/stroke.html`
- `tests/brush.html` (stateful Worker example)

2022/02/20 0.0.1 Pointのみ

2022/02/21 0.0.2 line追加

2022/02/21 0.0.3 rect追加

2022/02/22 0.0.4 polygram追加

2022/02/22 0.0.5 paint/picker追加

2022/02/27 0.0.6 circle/ellipse/arc追加

2022/03/07 0.0.7 jpeg decoder(baselineのみ)追加

2022/03/12 0.0.8 callback systemの変更、jpegの高速化(AANアルゴリズム)

2022/03/13 0.0.9 アフィン変換

2022/03/13 0.0.10 アフィン変換 + 補完アルゴリズム（ニアレストネイバー、バイリニア、バイキュービック、Lanczos

2022/03/17 0.0.11 Graphic Loaderを分離(WML2)

2022/03/26 0.0.12 ベジェ曲線を実装

2022/03/26 0.0.13 Screen Traitを実装

2022/03/27 0.0.14 Canvasの実装変更、Layer実装、with_alpha function実装

2022/03/29 0.0.15 JavaScriptとのバインド関数の変更、input_bufferの削除

2022/03/31 0.0.16 antialias実装(Circle以外)、Layer拡張、Layer combined canvas

2022/04/07 0.0.17 antialias circle/ellipse/arc

2022/04/08 0.0.18 resized image loader/Affine逆変換の計算式のシフトの部分が間違っていたので修正

2022/04/24 0.0.19 WML2 0.0.10で追加されたmetadata を追加

2022/05/01 0.0.20 Animation を追加

2022/05/22 0.0.21 filterを追加

2022/05/25 0.0.22 paintcore分離

2026/03/20 0.0.24 Canvasの実装の変更(破壊的変更)、filterの実装拡張

2026/03/20 0.0.25 Pathの拡張実装 FontAPIの前段階実装

2026/03/24 0.0.26 FontAPIの実装

2026/05/03 0.0.27 Color filter、Math libの実装

2026/05/16 0.0.28 refatoring code

2026/08/09 0.0.29 境界チェックの強化とそれに伴うAPIの拡張

The maintained roadmap is in [todo.md](todo.md).
