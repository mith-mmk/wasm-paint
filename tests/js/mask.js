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
  const left = universe.createRectMask(16, 24, 112, 96);
  const right = universe.createRectMask(72, 48, 112, 96);
  const union = universe.maskUnion(left, right);
  const feathered = universe.maskFeather(union, 6);
  universe.fillMaskLinearGradient(feathered, 16, 0, 184, 0, 0xffff3355, 0xff3355ff, "pad");

  const intersection = universe.maskIntersect(left, right);
  const shrunk = universe.maskShrink(intersection, 4);
  const tile = new Uint8Array([
    255, 255, 255, 220, 32, 32, 32, 220,
    32, 32, 32, 220, 255, 255, 255, 220,
  ]);
  universe.fillMaskPattern(shrunk, 2, 2, tile, "repeat", "nearest", 8, Math.PI / 8, 0, 0);

  const difference = universe.maskDifference(union, intersection);
  const grown = universe.maskGrow(difference, 2);
  const outsideBeforeInvert = universe.maskCoverage(left, 300, 80);
  universe.maskInvert(left);
  const outsideAfterInvert = universe.maskCoverage(left, 300, 80);
  const featherCoverage = universe.maskCoverage(feathered, 13, 48);
  const grownCoverage = universe.maskCoverage(grown, 14, 24);

  universe.combine();
  const image = universe.getImageData(0);
  context.putImageData(image, 0, 0);
  const gradientPixel = pixelAt(image, 32, 64);
  const patternPixel = pixelAt(image, 96, 80);
  const pass = outsideBeforeInvert === 0
    && outsideAfterInvert === 255
    && featherCoverage > 0
    && grownCoverage > 0
    && gradientPixel[3] > 0
    && patternPixel[3] > 0;
  status.textContent = pass ? "PASS" : "FAIL";
  status.dataset.result = pass ? "pass" : "fail";
  results.textContent = `feather=${featherCoverage}\ngrow=${grownCoverage}\ngradient=${gradientPixel}\npattern=${patternPixel}`;
} catch (error) {
  status.textContent = "FAIL";
  status.dataset.result = "fail";
  results.textContent = String(error?.stack ?? error);
}
