const canvas = document.getElementById("canvas");
const context = canvas.getContext("2d");
const status = document.getElementById("status");
const results = document.getElementById("results");
const worker = new Worker("./js/brush-worker.js", { type: "module" });

worker.addEventListener("message", ({ data }) => {
  if (data.type === "ready") {
    const samples = Array.from({ length: 33 }, (_, index) => ({
      x: 24 + index * 14,
      y: 96 + Math.sin(index * 0.45) * 44,
      pressure: 0.2 + 0.8 * (index / 32),
      rotation: index * 0.12,
      timestamp: index * 8,
    }));
    worker.postMessage({ type: "draw", samples });
    return;
  }
  if (data.type === "frame") {
    const pixels = new Uint8ClampedArray(data.pixels);
    context.putImageData(new ImageData(pixels, data.width, data.height), 0, 0);
    let painted = 0;
    for (let offset = 0; offset < pixels.length; offset += 4) {
      if (pixels[offset] !== 0 || pixels[offset + 1] !== 0 || pixels[offset + 2] !== 0) {
        painted += 1;
      }
    }
    const pass = data.totalDabs > data.samples && painted > 500;
    status.textContent = pass ? "PASS" : "FAIL";
    status.dataset.result = pass ? "pass" : "fail";
    results.textContent = `samples=${data.samples}\ndabs=${data.totalDabs}\npaintedPixels=${painted}\nspacing=0.18 pressure=size+opacity flow=0.65 rotation+scatter seed=2026`;
  }
});

worker.addEventListener("error", (error) => {
  status.textContent = "FAIL";
  status.dataset.result = "fail";
  results.textContent = error.message;
});

worker.postMessage({ type: "init", width: canvas.width, height: canvas.height });
