import init, { Universe } from "../../wasm-paint/pkg/paint.js";

const status = document.getElementById("status");
const results = document.getElementById("results");
const canvas = document.getElementById("canvas");
const context = canvas.getContext("2d");

try {
  await init();
  const universe = new Universe(canvas.width, canvas.height);
  universe.fillRectComposite(16, 16, 112, 112, 0xffff0000, 1.0, "normal");
  universe.fillRectComposite(72, 40, 112, 96, 0x800000ff, 1.0, "multiply");
  universe.fillRectComposite(136, 16, 96, 112, 0x8000ff00, 0.8, "screen");
  universe.combine();
  const image = universe.getImageData(0);
  context.putImageData(image, 0, 0);

  const offset = (64 * canvas.width + 96) * 4;
  const pixel = Array.from(image.data.slice(offset, offset + 4));
  const pass = pixel[3] === 255 && pixel[0] > 0 && pixel[2] === 0;
  status.textContent = pass ? "PASS" : "FAIL";
  status.dataset.result = pass ? "pass" : "fail";
  results.textContent = `overlap rgba=${pixel.join(",")}`;
} catch (error) {
  status.textContent = "FAIL";
  status.dataset.result = "fail";
  results.textContent = String(error?.stack ?? error);
}
