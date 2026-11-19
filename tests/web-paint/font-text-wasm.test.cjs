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

const svgFontPath = process.env.PAINT_TEST_SVG_FONT;
const svgzFontPath = process.env.PAINT_TEST_SVGZ_FONT;
const testSvgWasm = svgFontPath || process.env.PAINT_TEST_SVG_WASM === "1";
const svgFixture = path.resolve(__dirname, "../../paintcore/tests/fixtures/svg-text.ttf");

test("real Wasm single-face SVG text preserves placement and rejects an unsupported run", { skip: !testSvgWasm && "Build Wasm with svg-font and set PAINT_TEST_SVG_WASM=1" }, async () => {
  const { Universe } = await runtime();
  const engine = new Universe(76, 36);
  try {
    engine.loadFont(fs.readFileSync(svgFixture));
    engine.drawText("A\uFE0FA\uFE0F", 5, 7, 64, 0xff000000);
    const before = Uint8ClampedArray.from(engine.getLayerImageData("main").data);
    for (let y = 0; y < 36; y++) for (let x = 0; x < 76; x++) {
      const inArt = ((x >= 9 && x < 25) || (x >= 41 && x < 57)) && y >= 15 && y < 31;
      const i = (y * 76 + x) * 4;
      const want = inArt ? [34, 119, 204, 128] : [0, 0, 0, 0];
      assert.ok(want.every((v, j) => before[i + j] === v), `pixel (${x}, ${y})`);
    }
    assert.throws(() => engine.drawText("A\uFE0FB\uFE0F", 0, 0, 64, 0xff000000));
    assert.deepEqual(engine.getLayerImageData("main").data, before, "a later unsupported glyph leaves the destination unchanged");
  } finally { engine.free(); }
});

test("Google Fonts Noto OpenType SVG renders in ordinary Wasm text APIs", { skip: !svgFontPath && "Set PAINT_TEST_SVG_FONT to noto-untouchedsvg.ttf" }, async () => {
  const { Universe } = await runtime();
  const render = (font, flow) => {
    const engine = new Universe(660, 900);
    try {
      engine.loadFont(font);
      const text = "😀👍";
      if (flow) engine.drawTextFlow(text, 16, 288, 256, 0xff000000, flow);
      else engine.drawText(text, 16, 288, 256, 0xff000000);
      return Uint8ClampedArray.from(engine.getLayerImageData("main").data);
    } finally { engine.free(); }
  };
  const stats = (data) => {
    let count = 0, color = 0, minX = 660, minY = 900, maxX = 0, maxY = 0;
    for (let i = 0; i < data.length; i += 4) {
      if (!data[i + 3]) continue;
      const x = (i / 4) % 660, y = Math.floor(i / 4 / 660);
      count++;
      if (data[i] !== data[i + 1] || data[i + 1] !== data[i + 2]) color++;
      minX = Math.min(minX, x);
      minY = Math.min(minY, y);
      maxX = Math.max(maxX, x);
      maxY = Math.max(maxY, y);
    }
    return { count, color, width: maxX - minX + 1, height: maxY - minY + 1 };
  };
  const plain = fs.readFileSync(svgFontPath);
  const horizontal = render(plain);
  const h = stats(horizontal);
  assert.ok(h.count > 75000 && h.color > 70000 && h.width > 490, "both glyphs produce substantial colored ink");
  assert.deepEqual(render(plain, "horizontal"), horizontal, "legacy and flow APIs agree");
  const vertical = stats(render(plain, "vertical"));
  assert.ok(vertical.count > 70000 && vertical.height > vertical.width, "upright vertical SVG uses vertical glyph positions");
  if (svgzFontPath) assert.deepEqual(render(fs.readFileSync(svgzFontPath)), horizontal, "same artwork in SVG and SVGZ renders identically");
});
