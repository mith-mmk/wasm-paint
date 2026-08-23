import init, { Universe } from "../../wasm-paint/pkg/paint.js";

const canvas = document.getElementById("canvas");
const context = canvas.getContext("2d");
const status = document.getElementById("status");
const results = document.getElementById("results");

try {
  await init();
  const universe = new Universe(canvas.width, canvas.height);
  universe.fillRectComposite(16, 32, 48, 80, 0xff303030, 1, "normal");
  universe.fillRectComposite(64, 32, 48, 80, 0xff343030, 1, "normal");
  const tolerant = universe.createFloodMask(24, 48, 4, false, false);

  universe.fillRectComposite(160, 48, 24, 24, 0xff606060, 1, "normal");
  universe.fillRectComposite(184, 48, 24, 24, 0xff202020, 1, "normal");
  universe.fillRectComposite(160, 72, 24, 24, 0xff202020, 1, "normal");
  universe.fillRectComposite(184, 72, 24, 24, 0xff606060, 1, "normal");
  const four = universe.createFloodMask(168, 56, 0, false, false);
  const eight = universe.createFloodMask(168, 56, 0, false, true);

  universe.fillRectComposite(264, 48, 40, 64, 0x403399cc, 1, "normal");
  universe.fillRectComposite(304, 48, 40, 64, 0xc03399cc, 1, "normal");
  const rgba = universe.createFloodMask(280, 64, 0, true, false);

  const tolerancePass = universe.maskCoverage(tolerant, 96, 64) === 255;
  const connectivityPass = universe.maskCoverage(four, 192, 80) === 0
    && universe.maskCoverage(eight, 192, 80) === 255;
  const alphaPass = universe.maskCoverage(rgba, 320, 64) === 0;

  universe.fillMaskSolid(tolerant, 0xffff8844);
  universe.fillMaskSolid(eight, 0xff44dd88);
  universe.fillMaskSolid(rgba, 0xffaa55ff);
  universe.combine();
  const image = universe.getImageData(0);
  context.putImageData(image, 0, 0);

  const pass = tolerancePass && connectivityPass && alphaPass;
  status.textContent = pass ? "PASS" : "FAIL";
  status.dataset.result = pass ? "pass" : "fail";
  results.textContent = `tolerance=${tolerancePass}\n4-vs-8=${connectivityPass}\nRGB-vs-RGBA=${alphaPass}`;
} catch (error) {
  status.textContent = "FAIL";
  status.dataset.result = "fail";
  results.textContent = String(error?.stack ?? error);
}
