import init, { Universe } from "../../wasm-paint/pkg/paint.js";

const canvas = document.getElementById("canvas");
const context = canvas.getContext("2d");
const status = document.getElementById("status");
const results = document.getElementById("results");

const pixelAt = (image, x, y) => {
  const offset = (y * image.width + x) * 4;
  return Array.from(image.data.slice(offset, offset + 4));
};

try {
  await init();
  const universe = new Universe(canvas.width, canvas.height);
  universe.fillLinearGradientRect(8, 8, 112, 144, 8, 0, 120, 0, 0xffff0000, 0xff0000ff, "pad", false);
  universe.fillRadialGradientRect(136, 8, 112, 144, 192, 80, 56, 0xffffffff, 0xff0080ff, "reflect");
  universe.fillSweepGradientRect(264, 8, 112, 144, 320, 80, -Math.PI, Math.PI, 0xffffcc00, 0xff6633ff, "pad");
  universe.combine();
  const image = universe.getImageData(0);
  context.putImageData(image, 0, 0);

  const left = pixelAt(image, 24, 80);
  const right = pixelAt(image, 104, 80);
  const radialCenter = pixelAt(image, 192, 80);
  const pass = left[0] > left[2]
    && right[2] > right[0]
    && radialCenter[0] > 245
    && radialCenter[1] > 245
    && radialCenter[2] > 245;
  status.textContent = pass ? "PASS" : "FAIL";
  status.dataset.result = pass ? "pass" : "fail";
  results.textContent = `linear-left=${left}\nlinear-right=${right}\nradial-center=${radialCenter}`;
} catch (error) {
  status.textContent = "FAIL";
  status.dataset.result = "fail";
  results.textContent = String(error?.stack ?? error);
}
