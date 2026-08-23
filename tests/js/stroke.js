import init, { Universe } from "../../wasm-paint/pkg/paint.js";

const canvas = document.getElementById("canvas");
const context = canvas.getContext("2d");
const status = document.getElementById("status");
const results = document.getElementById("results");

const alphaAt = (image, x, y) => image.data[(y * image.width + x) * 4 + 3];

try {
  await init();
  const universe = new Universe(canvas.width, canvas.height);
  universe.strokeStyledPath("M 20 32 L 104 32", 0xffff4455, 10, "butt", "miter", 4, [], 0);
  universe.strokeStyledPath("M 20 72 L 104 72", 0xff44cc77, 10, "round", "round", 4, [], 0);
  universe.strokeStyledPath("M 20 112 L 104 112", 0xff4488ff, 10, "square", "bevel", 4, [], 0);

  universe.strokeStyledPath("M 144 120 L 192 32 L 240 120", 0xffffcc44, 14, "butt", "miter", 10, [], 0);
  universe.strokeStyledPath("M 152 128 L 192 64 L 232 128", 0xffcc44ff, 8, "butt", "bevel", 1, [], 0);

  universe.strokeStyledPath("M 272 40 L 368 40", 0xffffffff, 8, "butt", "miter", 4, [12, 8], 0);
  universe.strokeStyledPath("M 272 80 L 368 80", 0xff44ddff, 8, "round", "round", 4, [4, 10], 5);
  universe.combine();
  const image = universe.getImageData(0);
  context.putImageData(image, 0, 0);

  const buttOutside = alphaAt(image, 16, 32);
  const roundOutside = alphaAt(image, 16, 72);
  const squareOutside = alphaAt(image, 16, 112);
  const dashOn = alphaAt(image, 276, 40);
  const dashOff = alphaAt(image, 288, 40);
  const pass = buttOutside === 0
    && roundOutside > 0
    && squareOutside > 0
    && dashOn > 0
    && dashOff === 0;
  status.textContent = pass ? "PASS" : "FAIL";
  status.dataset.result = pass ? "pass" : "fail";
  results.textContent = `caps=${buttOutside}/${roundOutside}/${squareOutside}\ndash=${dashOn}/${dashOff}\njoin=miter + bevel`;
} catch (error) {
  status.textContent = "FAIL";
  status.dataset.result = "fail";
  results.textContent = String(error?.stack ?? error);
}
