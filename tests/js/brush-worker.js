import init, { Universe } from "../../wasm-paint/pkg/paint.js";

let universe;
let brush;

self.addEventListener("message", async ({ data }) => {
  if (data.type === "init") {
    await init();
    universe = new Universe(data.width, data.height);
    brush = universe.createBrush(
      "rectangle", 12, 5, 30, 0.9, 0.65, 0.18, 0, 0.18, 1, 0.6, 0, 2026,
    );
    self.postMessage({ type: "ready" });
    return;
  }
  if (data.type !== "draw" || !universe) return;

  const pattern = new Uint8Array([
    255, 255, 255, 255, 20, 40, 80, 255,
    20, 40, 80, 255, 255, 255, 255, 255,
  ]);
  let totalDabs = 0;
  for (const [index, sample] of data.samples.entries()) {
    if (index < data.samples.length / 2) {
      totalDabs += universe.brushSampleLinearGradient(
        brush, sample.x, sample.y, sample.pressure, sample.rotation, sample.timestamp,
        0, 0, 256, 0, 0xffff3355, 0xff3355ff,
      );
    } else {
      totalDabs += universe.brushSamplePattern(
        brush, sample.x, sample.y, sample.pressure, sample.rotation, sample.timestamp,
        2, 2, pattern, "repeat", "bilinear", 7,
      );
    }
  }
  universe.combine();
  const image = universe.getImageData(0);
  const pixels = new Uint8ClampedArray(image.data);
  self.postMessage({
    type: "frame",
    width: image.width,
    height: image.height,
    pixels: pixels.buffer,
    samples: data.samples.length,
    totalDabs,
  }, [pixels.buffer]);
});
