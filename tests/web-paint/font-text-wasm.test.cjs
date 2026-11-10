const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");
const { test } = require("node:test");
const fontPath = process.env.PAINT_TEST_FONT;
class ImageDataShim {
  constructor(data, width, height) { this.data = new Uint8ClampedArray(data); this.width = width; this.height = height; }
}
const runtime = async () => {
  global.ImageData = ImageDataShim;
  const dir = path.resolve(__dirname, "../../wasm-paint/pkg");
  const bindings = await import(pathToFileURL(path.join(dir, "paint.js")));
  bindings.initSync({ module: fs.readFileSync(path.join(dir, "paint_bg.wasm")) });
  return bindings;
};
test("real fontcore Wasm flows, errors and isolated pixels", { skip: !fontPath && "Set PAINT_TEST_FONT to a local Japanese font" }, async () => {
  const { Universe } = await runtime();
  const { fontTextImageData } = await import(pathToFileURL(path.join(__dirname, "font-text.js")));
  const bytes = fs.readFileSync(fontPath);
  const input = { width: 640, height: 480, fontBytes: bytes, text: "日本語ABC123（）", fontSize: 32, color: "#123456", x: 32, y: 32 };
  const images = [];
  for (const flow of ["vertical", "mixed", "sideways"]) {
    const image = fontTextImageData({ ...input, flow });
    let painted = 0, minX = 640, minY = 480, maxX = 0, maxY = 0;
    for (let y = 0; y < 480; y++) for (let x = 0; x < 640; x++) {
      if (image.data[(y * 640 + x) * 4 + 3]) { painted++; minX = Math.min(minX, x); minY = Math.min(minY, y); maxX = Math.max(maxX, x); maxY = Math.max(maxY, y); }
    }
    assert.ok(painted > 100, `${flow} produces ink`);
    assert.ok(minX >= 31 && minY >= 31, `${flow} respects adjusted origin`);
    assert.ok(maxY - minY > maxX - minX, `${flow} is vertical`);
    images.push(image.data);
  }
  assert.notDeepEqual(images[1], images[2], "mixed and sideways differ");
  for (const flow of ["mixed", "sideways"]) {
    assert.throws(() => fontTextImageData({ ...input, flow, text: "日本\nABC" }));
    assert.throws(() => fontTextImageData({ ...input, flow, text: "日本\tABC" }));
  }
  assert.throws(() => fontTextImageData({ ...input, flow: "mixed", fontBytes: new Uint8Array([0, 1]) }));
  const engine = new Universe(100, 100);
  try {
    engine.loadFont(bytes);
    assert.throws(() => engine.drawTextFlow("ABC", 1, 1, 20, 0xff000000, "bad"));
    assert.throws(() => engine.drawTextFlow("ABC", NaN, 1, 20, 0xff000000, "mixed"));
    assert.ok(engine.getLayerImageData("main").data.every(v => v === 0), "failures leave art unchanged");
    engine.drawText("ABC", 1, 40, 20, 0xff000000);
    assert.ok(engine.getLayerImageData("main").data.some(v => v !== 0), "legacy horizontal API remains functional");
  } finally { engine.free(); }
});
