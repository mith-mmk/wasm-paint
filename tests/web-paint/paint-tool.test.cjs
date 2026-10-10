// Exercise component history with real WASM pixels and minimal DOM/ImageData
// shims. The encoder retains raw RGBA bytes to check snapshot timing, not PNGs.
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const vm = require("node:vm");
const { pathToFileURL } = require("node:url");
const { test } = require("node:test");

class ImageDataShim {
  constructor(data, width, height) {
    this.data = new Uint8ClampedArray(data);
    this.width = width;
    this.height = height;
  }
}

class ElementShim {
  constructor() {
    this.handlers = {};
    this.dataset = {};
    this.style = { setProperty() {} };
    this.width = this.height = this.clientWidth = this.clientHeight = 8;
    this.clientLeft = this.clientTop = 0;
  }
  addEventListener(type, fn) { this.handlers[type] = fn; }
  setAttribute() {}
  toggleAttribute() {}
  getAttribute(name) { return name === "width" || name === "height" ? "8" : null; }
  hasAttribute() { return false; }
  append() {}
  replaceChildren() {}
  dispatchEvent() {}
  setPointerCapture() {}
  getBoundingClientRect() { return { left: 0, top: 0 }; }
  getContext() { return { putImageData: image => { this.image = image; } }; }
  toBlob(callback) { callback(new Blob([this.image.data])); }
  attachShadow() {
    this.nodes = new Map();
    return this.shadowRoot = {
      append() {},
      querySelector: name => {
        if (!this.nodes.has(name)) this.nodes.set(name, new ElementShim());
        return this.nodes.get(name);
      },
    };
  }
}

const runtime = (async () => {
  global.ImageData = ImageDataShim;
  const packageDir = path.resolve(__dirname, "../../wasm-paint/pkg");
  const bindings = await import(pathToFileURL(path.join(packageDir, "paint.js")));
  bindings.initSync({ module: fs.readFileSync(path.join(packageDir, "paint_bg.wasm")) });
  return bindings;
})();

async function createTool(historyAvailable = true) {
  const { Universe } = await runtime;
  let Tool;
  const context = {
    Universe: class {
      constructor(...args) {
        const universe = new Universe(...args);
        if (!historyAvailable) universe.getLayerImageData = undefined;
        return universe;
      }
    },
    init: async () => {},
    HTMLElement: ElementShim,
    document: {
      createElement: name => name === "template"
        ? { content: { cloneNode() { return {}; } } }
        : new ElementShim(),
    },
    customElements: { get() {}, define(name, value) { Tool = value; } },
    CustomEvent: class {},
    Blob,
    console,
    performance,
    requestAnimationFrame() {},
    crypto: require("node:crypto").webcrypto,
  };
  const source = fs.readFileSync(path.join(__dirname, "paint-tool.js"), "utf8")
    .replace(/^import .*\n/, "")
    .replace("export function shapePathCommands", "function shapePathCommands");
  vm.runInNewContext(source, context);
  const paint = new Tool();
  paint.connectedCallback();
  await paint.ready;
  return paint;
}

const stroke = { size: 2, color: "#ff0000", points: [{ x: 4, y: 4 }] };
const pixels = async blob => Buffer.from(await blob.arrayBuffer());

test("export commits the active stroke before capturing its history ID", async () => {
  const paint = await createTool();
  const canvas = paint.shadowRoot.querySelector("canvas");
  const pointer = {
    pointerType: "mouse", button: 0, pointerId: 1,
    clientX: 4, clientY: 4, pressure: 0.5, timeStamp: 1,
    preventDefault() {},
  };
  canvas.handlers.pointerdown(pointer);
  const saved = await paint.exportImageSnapshot();
  canvas.handlers.pointerup(pointer);
  assert.equal((await paint.getState()).historyId, saved.state.historyId);
  await paint.undo();
  assert.notEqual((await paint.getState()).historyId, saved.state.historyId);
  assert.notDeepEqual(await pixels(await paint.exportImage()), await pixels(saved.blob));
  await paint.redo();
  assert.equal((await paint.getState()).historyId, saved.state.historyId);
  assert.deepEqual(await pixels(await paint.exportImage()), await pixels(saved.blob));
});

for (const historyAvailable of [true, false]) {
  test(`edits during encoding retain the captured revision (history=${historyAvailable})`, async () => {
    const paint = await createTool(historyAvailable);
    await paint.drawStroke(stroke);
    const before = await paint.getState();
    const canvas = paint.shadowRoot.querySelector("canvas");
    let finishEncoding;
    let started;
    const encodingStarted = new Promise(resolve => { started = resolve; });
    canvas.toBlob = callback => {
      const blob = new Blob([canvas.image.data]);
      finishEncoding = () => callback(blob);
      started();
    };
    const exporting = paint.exportImageSnapshot();
    await encodingStarted;
    await paint.clearCanvas();
    finishEncoding();
    const saved = await exporting;
    assert.equal(saved.state.revision, before.revision);
    assert.equal(saved.state.historyId, before.historyId);
    assert.notEqual(saved.state.revision, (await paint.getState()).revision);
    assert.ok((await pixels(saved.blob)).some(value => value !== 0));
  });

  test(`file saving keeps edits during encoding dirty (history=${historyAvailable})`, async () => {
    const paint = await createTool(historyAvailable);
    await paint.drawStroke(stroke);
    const before = await paint.getState();
    const canvas = paint.shadowRoot.querySelector("canvas");
    let finishEncoding;
    let started;
    const encodingStarted = new Promise(resolve => { started = resolve; });
    canvas.toBlob = callback => {
      const blob = new Blob([canvas.image.data]);
      finishEncoding = () => callback(blob);
      started();
    };
    const context = {
      paint, state: before, changeRevision: before.revision,
      requestWritePermission: async () => {},
      writePng: async (handle, blob) => { context.written = blob; },
      setDocumentName() {}, setSaveBusy() {}, message() {},
      setDirty: value => { context.dirty = value; },
      reportSaveError: error => { throw error; },
    };
    const uiSource = fs.readFileSync(path.join(__dirname, "external-ui.js"), "utf8");
    const finishSave = uiSource.slice(uiSource.indexOf("function finishPngSave("), uiSource.indexOf("function reportSaveError("));
    const save = uiSource.slice(uiSource.indexOf("async function saveToCurrentHandle("), uiSource.indexOf("function startSave("));
    vm.createContext(context);
    vm.runInContext(`${finishSave}\n${save}`, context);
    const saving = context.saveToCurrentHandle({ name: "drawing.png" });
    await encodingStarted;
    await paint.clearCanvas();
    context.state = await paint.getState();
    context.changeRevision = context.state.revision;
    finishEncoding();
    await saving;
    assert.equal(context.savedHistoryId, before.historyId);
    assert.equal(context.savedRevision, before.revision);
    assert.equal(context.dirty, true);
    assert.ok((await pixels(context.written)).some(value => value !== 0));
  });
}

test("export commits a pending opacity preview", async () => {
  const paint = await createTool();
  await paint.drawStroke(stroke);
  const before = await paint.getState();
  await paint.setLayerOpacity("main", 0.5, false);
  const saved = await paint.exportImageSnapshot();
  assert.notEqual(saved.state.historyId, before.historyId);
  await paint.undo();
  assert.equal((await paint.getState()).layers[0].opacity, 1);
  assert.notDeepEqual(await pixels(await paint.exportImage()), await pixels(saved.blob));
});
