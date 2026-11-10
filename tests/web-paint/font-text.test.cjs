const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { test } = require("node:test");
const source = fs.readFileSync(path.join(__dirname, "font-text.js"), "utf8")
  .replace(/^import .*\r?\n/, "").replace("export function", "function");
function setup(failure) {
  const calls = [];
  const backing = new Uint8ClampedArray([18, 52, 86, 255]);
  class Universe {
    constructor(w, h) { calls.push(["new", w, h]); }
    loadFont(bytes) { calls.push(["load", bytes.length]); if (failure === "load") throw "bad font"; }
    measureTextFlow(...args) { calls.push(["measure", ...args]); if (failure === "shape") throw "unsupported newline"; return failure === "empty" ? [] : [-5, -10, 25, 90]; }
    drawTextFlow(...args) { calls.push(["draw", ...args]); }
    getLayerImageData(layer) { calls.push(["image", layer]); return { width: 200, height: 300, data: backing }; }
    free() { calls.push(["free"]); backing.fill(0); }
  }
  const context = vm.createContext({ Universe, ImageData: class { constructor(data, width, height) { Object.assign(this, { data, width, height }); } } });
  vm.runInContext(source, context);
  return { render: context.fontTextImageData, calls };
}
const input = { width: 200, height: 300, fontBytes: new Uint8Array([1]), text: "日本ABC", fontSize: 32, color: "#123456", flow: "mixed", x: 20, y: 150 };
for (const [align, expectedY] of [["left", 160], ["center", 110], ["right", 60]]) {
  test(`vertical ${align} alignment applies bounds exactly once`, () => {
    const { render, calls } = setup();
    assert.equal(render({ ...input, align }).height, 300);
    assert.deepEqual(calls.find(c => c[0] === "draw"), ["draw", input.text, 25, expectedY, 32, 0xff123456, "mixed"]);
    assert.deepEqual(calls.at(-1), ["free"]);
    assert.deepEqual(calls.find(c => c[0] === "image"), ["image", "main"]);
  });
}
for (const flow of ["vertical", "mixed", "sideways"]) {
  test(`passes ${flow} unchanged to fontcore without rotating canvas`, () => {
    const { render, calls } = setup(); render({ ...input, flow });
    assert.equal(calls.find(c => c[0] === "draw").at(-1), flow);
  });
}
for (const failure of ["load", "shape", "empty"]) {
  test(`${failure} failure frees isolated engine and never draws`, () => {
    const { render, calls } = setup(failure);
    assert.throws(() => render(input));
    assert.equal(calls.some(c => c[0] === "draw"), false);
    assert.deepEqual(calls.at(-1), ["free"]);
  });
}
test("missing font does not allocate an engine", () => {
  const { render, calls } = setup();
  assert.throws(() => render({ ...input, fontBytes: undefined }), /フォント/);
  assert.equal(calls.length, 0);
});

// A failed commit must preserve the editable draft and restore its controls.
const ui = fs.readFileSync(path.join(__dirname, "external-ui.js"), "utf8");
const commitSource = ui.slice(ui.indexOf("async function commitTextDraft()"), ui.indexOf("async function selectEyedropperColor()"));
for (const fail of [true, false]) {
  test(`text commit lifecycle, failure=${fail}`, async () => {
    const elements = new Map([ ["#text-value", { value: "日本ABC" }], ["#text-cancel", {}], ["#text-apply", {}], ["#text-editor", {}] ]);
    const draft = { style: { flow: "mixed" } };
    let loaded = 0;
    const context = vm.createContext({ textDraft: draft, textCommitPromise: undefined, textFontBytes: input.fontBytes, textFontLoading: false,
      $: key => elements.get(key), textImageBlob: async () => { if (fail) throw new Error("unsupported"); return {}; },
      paint: { loadImage: async () => { loaded++; return "text"; } }, thumbnails: new Map(),
      URL: { createObjectURL: () => "blob:test" }, renderLayers() {}, message() {}, cancelTextDraft() {} });
    vm.runInContext(commitSource, context);
    if (fail) await assert.rejects(context.commitTextDraft(), /unsupported/);
    else await context.commitTextDraft();
    assert.equal(context.textDraft, fail ? draft : undefined);
    assert.equal(loaded, fail ? 0 : 1);
    assert.equal(elements.get("#text-value").disabled, false);
    assert.equal(elements.get("#text-cancel").disabled, false);
    assert.equal(elements.get("#text-apply").disabled, false);
    assert.equal(context.textCommitPromise, undefined);
  });
}

test("pending font replacement blocks commit before reading old bytes", async () => {
  const context = vm.createContext({ textDraft: { style: { flow: "mixed" } }, textCommitPromise: undefined, textFontLoading: true });
  vm.runInContext(commitSource, context);
  await assert.rejects(context.commitTextDraft(), /読み込み中/);
  assert.ok(context.textDraft);
});

for (const invalid of [{ width: 0 }, { width: 8192, height: 8192 }, { x: NaN }, { fontSize: -1 }, { flow: "unknown" }, { text: "a".repeat(4097) }]) {
  test(`invalid input fails before allocating: ${Object.keys(invalid).join()}`, () => {
    const { render, calls } = setup();
    assert.throws(() => render({ ...input, ...invalid }));
    assert.equal(calls.length, 0);
  });
}

test("returned pixels own memory after engine.free corrupts backing storage", () => {
  const { render } = setup();
  assert.deepEqual(Array.from(render(input).data), [18, 52, 86, 255]);
});

const selectionStart = ui.indexOf('  file.addEventListener("change", () => run(async () => {');
const selectionBody = ui.slice(selectionStart + '  file.addEventListener("change", () => run(async () => {'.length, ui.indexOf('  }));', selectionStart));
function selectionContext(file) {
  const context = vm.createContext({ file: { files: [file] }, fileLabel: {}, fontSelection: 0,
    textFontBytes: new Uint8Array([9]), textFontLoading: false, textFontName: "old", syncTextDraftStyle() {}, Uint8Array });
  vm.runInContext(`async function chooseFont() { ${selectionBody} }`, context);
  return context;
}
test("font replacement clears old bytes while loading and ignores late stale reads", async () => {
  let completeA;
  const context = selectionContext({ name: "A", size: 1, arrayBuffer: () => new Promise(resolve => { completeA = resolve; }) });
  const a = context.chooseFont();
  assert.equal(context.textFontBytes, undefined);
  assert.equal(context.textFontLoading, true);
  context.file.files = [{ name: "B", size: 1, arrayBuffer: async () => new Uint8Array([2]).buffer }];
  await context.chooseFont();
  completeA(new Uint8Array([1]).buffer);
  await a;
  assert.equal(context.textFontName, "B");
  assert.deepEqual(Array.from(context.textFontBytes), [2]);
  assert.equal(context.textFontLoading, false);
});
test("font read failure clears loading and permits re-selection", async () => {
  const context = selectionContext({ name: "bad", size: 1, arrayBuffer: async () => { throw new Error("read failed"); } });
  await assert.rejects(context.chooseFont(), /read failed/);
  assert.equal(context.textFontBytes, undefined);
  assert.equal(context.textFontLoading, false);
  context.file.files = [{ name: "valid", size: 1, arrayBuffer: async () => new Uint8Array([3]).buffer }];
  await context.chooseFont();
  assert.equal(context.textFontName, "valid");
});
test("oversized font is rejected before arrayBuffer", async () => {
  let read = false;
  const context = selectionContext({ name: "large", size: 65 * 1024 * 1024, arrayBuffer: async () => { read = true; } });
  await assert.rejects(context.chooseFont(), /64MiB/);
  assert.equal(read, false);
  assert.equal(context.textFontLoading, false);
});
