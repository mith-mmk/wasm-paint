import init, { Universe } from "../../wasm-paint/pkg/paint.js";

const canvas = document.getElementById("canvas");
const context = canvas.getContext("2d");
const status = document.getElementById("status");
const results = document.getElementById("results");

const pattern = new Uint8Array([
  255, 64, 64, 255, 64, 255, 64, 255,
  64, 64, 255, 255, 255, 255, 255, 128,
]);

try {
  await init();
  const universe = new Universe(canvas.width, canvas.height);
  universe.fillPatternRect(8, 8, 144, 144, 2, 2, pattern, "repeat", "repeat", "nearest", 16, 16, 8, 8);
  universe.fillPatternRect(168, 8, 144, 144, 2, 2, pattern, "mirror", "mirror", "bilinear", 20, 20, 168, 8);
  universe.combine();
  const image = universe.getImageData(0);
  context.putImageData(image, 0, 0);

  const offset = (16 * image.width + 16) * 4;
  const first = Array.from(image.data.slice(offset, offset + 4));
  const pass = first[0] > first[1] && first[0] > first[2] && first[3] === 255;
  status.textContent = pass ? "PASS" : "FAIL";
  status.dataset.result = pass ? "pass" : "fail";
  results.textContent = `first tile rgba=${first.join(",")}`;
} catch (error) {
  status.textContent = "FAIL";
  status.dataset.result = "fail";
  results.textContent = String(error?.stack ?? error);
}
