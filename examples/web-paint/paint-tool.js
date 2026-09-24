import init, { Universe } from "../../wasm-paint/pkg/paint.js";

const wasmReady = init();
let nextInstanceId = 1;

const template = document.createElement("template");
template.innerHTML = `
  <style>
    :host {
      display: block;
      color: #18212f;
      font: 14px/1.4 system-ui, sans-serif;
      --paint-accent: #315fce;
      --paint-border: #d5dbe5;
      --paint-muted: #647084;
    }
    :host([controls="external"]) .toolbar,
    :host([controls="external"]) .layers,
    :host([controls="external"]) .status { display: none; }
    * { box-sizing: border-box; }
    .shell { display: grid; gap: 12px; }
    .toolbar, .layer-tools { display: flex; align-items: center; flex-wrap: wrap; gap: 8px; }
    button, input { font: inherit; }
    button {
      min-height: 34px;
      padding: 6px 10px;
      color: inherit;
      background: white;
      border: 1px solid var(--paint-border);
      border-radius: 7px;
      cursor: pointer;
    }
    button:hover { border-color: var(--paint-accent); }
    button[aria-pressed="true"] { color: white; background: var(--paint-accent); border-color: var(--paint-accent); }
    button:focus-visible, input:focus-visible { outline: 2px solid var(--paint-accent); outline-offset: 2px; }
    input[type="color"] { width: 38px; height: 34px; padding: 2px; border: 1px solid var(--paint-border); border-radius: 7px; background: white; }
    input[type="range"] { accent-color: var(--paint-accent); }
    .control { display: inline-flex; align-items: center; gap: 7px; white-space: nowrap; }
    .size-value, .opacity-value { min-width: 3ch; color: var(--paint-muted); text-align: right; }
    .workspace { overflow: auto; padding: 12px; background: #eef1f5; border: 1px solid var(--paint-border); border-radius: 10px; }
    canvas {
      display: block;
      width: min(100%, var(--paint-width));
      height: auto;
      margin: 0 auto;
      background-color: #fff;
      background-image: linear-gradient(45deg, #e5e8ed 25%, transparent 25%), linear-gradient(-45deg, #e5e8ed 25%, transparent 25%), linear-gradient(45deg, transparent 75%, #e5e8ed 75%), linear-gradient(-45deg, transparent 75%, #e5e8ed 75%);
      background-size: 18px 18px;
      background-position: 0 0, 0 9px, 9px -9px, -9px 0;
      border: 1px solid #bcc5d2;
      box-shadow: 0 2px 8px #1620331c;
      touch-action: none;
      cursor: crosshair;
    }
    .layers { display: grid; gap: 8px; }
    .layer-list { display: grid; gap: 5px; margin: 0; padding: 0; list-style: none; }
    .layer-row { display: flex; align-items: center; gap: 7px; min-width: 0; }
    .layer-select { flex: 1; overflow: hidden; text-align: left; text-overflow: ellipsis; }
    .layer-select[aria-current="true"] { border-color: var(--paint-accent); box-shadow: inset 3px 0 var(--paint-accent); }
    .visibility { min-width: 70px; }
    .opacity-control { display: flex; align-items: center; gap: 8px; max-width: 360px; }
    .opacity-control input { flex: 1; }
    .status { min-height: 1.4em; color: var(--paint-muted); }
    .status[data-error="true"] { color: #a32222; }
    .visually-hidden { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0; }
    @media (max-width: 520px) {
      .workspace { padding: 5px; }
      .toolbar { gap: 6px; }
      button { padding: 6px 8px; }
    }
  </style>
  <div class="shell">
    <div class="toolbar" role="toolbar" aria-label="Paint tools">
      <label class="control" title="Brush color">
        <span class="visually-hidden">Brush color</span>
        <input class="color" type="color" value="#20304a" aria-label="Brush color">
      </label>
      <label class="control">
        <span>Size</span>
        <input class="size" type="range" min="1" max="100" value="12" aria-label="Brush size">
        <output class="size-value">12</output>
      </label>
      <button class="tool" type="button" aria-pressed="false">Eraser</button>
      <button class="import" type="button">Open image</button>
      <input class="file visually-hidden" type="file" accept="image/*" aria-label="Choose an image file">
      <button class="export" type="button">Download PNG</button>
    </div>
    <div class="workspace"><canvas aria-label="Drawing canvas"></canvas></div>
    <section class="layers" aria-label="Layers">
      <div class="layer-tools">
        <strong>Layers</strong>
        <button class="add-layer" type="button">Add layer</button>
        <button class="clear-layer" type="button">Clear selected layer</button>
      </div>
      <ul class="layer-list"></ul>
      <label class="opacity-control">
        <span>Selected layer opacity</span>
        <input class="opacity" type="range" min="0" max="100" value="100" aria-label="Selected layer opacity">
        <output class="opacity-value">100%</output>
      </label>
    </section>
    <div class="status" role="status" aria-live="polite">Loading paint engine…</div>
  </div>
`;

class WasmPaintTool extends HTMLElement {
  static get observedAttributes() { return ["webmcp", "controls"]; }

  #universe;
  #ready;
  #canvas;
  #context;
  #status;
  #layers = [];
  #selectedLayer = "main";
  #nextLayerNumber = 1;
  #brushSeed = 1;
  #brushColor = "#20304a";
  #brushSize = 12;
  #eraserEnabled = false;
  #pointerStroke;
  #renderQueued = false;
  #webmcpController;
  #instancePrefix = `wasmpaint-${nextInstanceId++}-${globalThis.crypto?.randomUUID?.().replaceAll("-", "") ?? Math.random().toString(36).slice(2)}`;

  connectedCallback() {
    if (this.shadowRoot) {
      this.#ready?.then(() => this.#syncWebMcp());
      return;
    }
    this.attachShadow({ mode: "open" }).append(template.content.cloneNode(true));
    this.#canvas = this.shadowRoot.querySelector("canvas");
    this.#status = this.shadowRoot.querySelector(".status");
    this.#bindUi();
    this.#ready = this.#initialize();
  }

  disconnectedCallback() {
    this.#webmcpController?.abort();
    this.#webmcpController = undefined;
  }

  attributeChangedCallback(name) {
    if (name === "webmcp" && this.isConnected) this.#syncWebMcp();
  }

  get ready() { return this.#ready ?? Promise.resolve(); }

  async getState() {
    await this.ready;
    if (!this.#universe) throw new Error("The paint engine did not initialize.");
    return this.#stateSnapshot();
  }

  async setBrushColor(color) {
    await this.ready;
    if (typeof color !== "string" || !/^#[\da-f]{6}$/i.test(color)) throw new TypeError("Brush color must be a six-digit hex color.");
    this.#brushColor = color.toLowerCase();
    this.#syncControlInputs();
    this.#dispatchStateChange();
  }

  async setBrushSize(size) {
    await this.ready;
    if (!Number.isFinite(size) || size < 1 || size > 100) throw new RangeError("Brush size must be between 1 and 100.");
    this.#brushSize = Math.round(size);
    this.#syncControlInputs();
    this.#dispatchStateChange();
  }

  async setEraserEnabled(enabled) {
    await this.ready;
    if (typeof enabled !== "boolean") throw new TypeError("Eraser state must be a boolean.");
    this.#eraserEnabled = enabled;
    this.#syncControlInputs();
    this.#dispatchStateChange();
  }

  async drawStroke(stroke = {}) {
    await this.ready;
    const { eraser = this.#eraserEnabled, ...settings } = stroke;
    if (typeof eraser !== "boolean") throw new TypeError("Stroke eraser setting must be a boolean.");
    this.#drawStroke({ color: this.#brushColor, size: this.#brushSize, ...settings }, "api-draw", eraser);
  }

  async addLayer(name) {
    await this.ready;
    const label = this.#createLayer("Layer", name);
    this.#renderLayers();
    this.#commitEdit("add-layer");
    return label;
  }

  async selectLayer(name) {
    await this.ready;
    this.#selectLayer(name);
    return this.#stateSnapshot();
  }

  async setLayerVisibility(name, visible) {
    await this.ready;
    if (typeof visible !== "boolean") throw new TypeError("Layer visibility must be a boolean.");
    const layer = this.#findLayer(name);
    layer.visible = visible;
    if (visible) this.#universe.setEnable(name);
    else this.#universe.setDisable(name);
    this.#renderLayers();
    this.#commitEdit("layer-visibility");
  }

  async setLayerOpacity(name, opacity) {
    await this.ready;
    if (!Number.isFinite(opacity) || opacity < 0 || opacity > 1) throw new RangeError("Layer opacity must be between 0 and 1.");
    this.#setLayerOpacity(name, opacity * 100);
    this.#renderLayers();
  }

  async clearLayer(name = this.#selectedLayer) {
    await this.ready;
    this.#clearLayer(name);
  }

  async clearCanvas() {
    await this.ready;
    for (const layer of this.#layers) this.#universe.clearLayer(layer.name);
    this.#renderNow();
    this.#commitEdit("clear-canvas");
  }

  async loadImage(blob) {
    await this.ready;
    if (!(blob instanceof Blob)) throw new TypeError("loadImage expects a Blob.");
    if (!this.#universe) throw new Error("The paint engine did not initialize.");

    const bitmap = await createImageBitmap(blob);
    try {
      const source = document.createElement("canvas");
      source.width = bitmap.width;
      source.height = bitmap.height;
      const sourceContext = source.getContext("2d", { alpha: true });
      sourceContext.drawImage(bitmap, 0, 0);
      const png = await new Promise((resolve, reject) => {
        source.toBlob((result) => result ? resolve(result) : reject(new Error("Could not encode the image for import.")), "image/png");
      });
      const bytes = new Uint8Array(await png.arrayBuffer());
      const label = this.#createLayer("Image");
      this.#universe.imageLoader(bytes, 1);
      this.#selectedLayer = label;
      this.#renderLayers();
      this.#renderNow();
      this.#commitEdit("load-image");
      this.#setStatus(`Loaded image into ${label}.`);
      return label;
    } finally {
      bitmap.close?.();
    }
  }

  async exportImage() {
    await this.ready;
    if (!this.#universe) throw new Error("The paint engine did not initialize.");
    this.#renderNow();
    return new Promise((resolve, reject) => {
      this.#canvas.toBlob((blob) => blob ? resolve(blob) : reject(new Error("Could not export the canvas as PNG.")), "image/png");
    });
  }

  async #initialize() {
    try {
      await wasmReady;
      const width = this.#dimensionAttribute("width", 640);
      const height = this.#dimensionAttribute("height", 480);
      this.#canvas.width = width;
      this.#canvas.height = height;
      this.style.setProperty("--paint-width", `${width}px`);
      this.#context = this.#canvas.getContext("2d", { alpha: true });
      this.#universe = new Universe(width, height);
      this.#layers = [{ name: "main", visible: true, opacity: 255 }];
      this.#renderLayers();
      this.#syncControlInputs();
      this.#renderNow();
      this.#setStatus(`Ready · ${width} × ${height}`);
      this.#syncWebMcp();
    } catch (error) {
      this.#setStatus(`Could not initialize paint: ${error.message}`, true);
      throw error;
    }
  }

  #dimensionAttribute(name, fallback) {
    const raw = this.getAttribute(name);
    if (!raw || !/^\d+$/.test(raw)) return fallback;
    const value = Number(raw);
    return Number.isSafeInteger(value) && value > 0 ? Math.min(value, 8192) : fallback;
  }

  #bindUi() {
    const root = this.shadowRoot;
    root.querySelector(".tool").addEventListener("click", (event) => {
      const button = event.currentTarget;
      this.#eraserEnabled = button.getAttribute("aria-pressed") !== "true";
      this.#syncControlInputs();
      this.#dispatchStateChange();
    });
    root.querySelector(".color").addEventListener("input", (event) => {
      this.#brushColor = event.currentTarget.value.toLowerCase();
      this.#dispatchStateChange();
    });
    root.querySelector(".size").addEventListener("input", (event) => {
      this.#brushSize = Number(event.currentTarget.value);
      this.#syncControlInputs();
      this.#dispatchStateChange();
    });
    root.querySelector(".import").addEventListener("click", () => root.querySelector(".file").click());
    root.querySelector(".file").addEventListener("change", async (event) => {
      const input = event.currentTarget;
      const [file] = input.files ?? [];
      if (!file) return;
      try { await this.loadImage(file); }
      catch (error) { this.#setStatus(`Image import failed: ${error.message}`, true); }
      finally { input.value = ""; }
    });
    root.querySelector(".export").addEventListener("click", async () => {
      try {
        const blob = await this.exportImage();
        this.#download(blob);
        this.#setStatus("PNG downloaded.");
      } catch (error) { this.#setStatus(`PNG export failed: ${error.message}`, true); }
    });
    root.querySelector(".add-layer").addEventListener("click", () => {
      this.addLayer().catch((error) => this.#setStatus(`Could not add layer: ${error.message}`, true));
    });
    root.querySelector(".clear-layer").addEventListener("click", () => {
      this.#clearLayer(this.#selectedLayer);
    });
    root.querySelector(".opacity").addEventListener("input", (event) => {
      const percent = Number(event.currentTarget.value);
      root.querySelector(".opacity-value").value = `${percent}%`;
      this.#setLayerOpacity(this.#selectedLayer, percent, false);
    });
    root.querySelector(".opacity").addEventListener("change", () => this.#commitEdit("layer-opacity"));
    this.#canvas.addEventListener("pointerdown", (event) => this.#pointerDown(event));
    this.#canvas.addEventListener("pointermove", (event) => this.#pointerMove(event));
    this.#canvas.addEventListener("pointerup", (event) => this.#finishPointerStroke(event, "draw"));
    this.#canvas.addEventListener("pointercancel", (event) => this.#finishPointerStroke(event, "draw"));
    this.#canvas.addEventListener("lostpointercapture", (event) => this.#finishPointerStroke(event, "draw"));
    root.querySelector(".layer-list").addEventListener("click", (event) => {
      const select = event.target.closest("[data-select-layer]");
      if (select) {
        this.selectLayer(select.dataset.selectLayer).catch((error) => this.#setStatus(error.message, true));
        return;
      }
      const visibility = event.target.closest("[data-toggle-visibility]");
      if (visibility) {
        const layer = this.#findLayer(visibility.dataset.toggleVisibility);
        this.setLayerVisibility(layer.name, !layer.visible).catch((error) => this.#setStatus(error.message, true));
      }
    });
  }

  #pointerDown(event) {
    if (!this.#universe || (event.pointerType === "mouse" && event.button !== 0)) return;
    event.preventDefault();
    this.#canvas.setPointerCapture(event.pointerId);
    const brushId = this.#universe.createBrush("circle", 9, 9, this.#brushSize, 1, 1, 0.2, 0, 0, 0, 0, 0, this.#brushSeed++);
    this.#pointerStroke = { pointerId: event.pointerId, brushId, eraser: this.#eraserEnabled };
    this.#samplePointer(event);
  }

  #pointerMove(event) {
    if (!this.#pointerStroke || this.#pointerStroke.pointerId !== event.pointerId) return;
    event.preventDefault();
    const samples = event.getCoalescedEvents?.() ?? [event];
    for (const sample of samples) this.#samplePointer(sample);
  }

  #samplePointer(event) {
    const point = this.#canvasPoint(event);
    const stroke = this.#pointerStroke;
    const pressure = event.pressure > 0 ? event.pressure : 0.5;
    const rotation = (event.twist ?? 0) * Math.PI / 180;
    if (stroke.eraser) {
      this.#universe.brushSampleEraser(stroke.brushId, point.x, point.y, pressure, event.tiltX ?? 0, event.tiltY ?? 0, rotation, event.timeStamp);
    } else {
      const rgb = Number.parseInt(this.#brushColor.slice(1), 16);
      const argb = ((0xff << 24) | rgb) >>> 0;
      this.#universe.brushSampleSolid(stroke.brushId, point.x, point.y, pressure, event.tiltX ?? 0, event.tiltY ?? 0, rotation, event.timeStamp, argb);
    }
    this.#queueRender();
  }

  #canvasPoint(event) {
    const rect = this.#canvas.getBoundingClientRect();
    const contentX = event.clientX - rect.left - this.#canvas.clientLeft;
    const contentY = event.clientY - rect.top - this.#canvas.clientTop;
    return {
      x: Math.min(this.#canvas.width, Math.max(0, contentX * this.#canvas.width / this.#canvas.clientWidth)),
      y: Math.min(this.#canvas.height, Math.max(0, contentY * this.#canvas.height / this.#canvas.clientHeight)),
    };
  }

  #finishPointerStroke(event, source) {
    if (!this.#pointerStroke || this.#pointerStroke.pointerId !== event.pointerId) return;
    const { brushId } = this.#pointerStroke;
    this.#pointerStroke = undefined;
    this.#universe.destroyBrush(brushId);
    this.#renderNow();
    this.#commitEdit(source === "draw" ? "draw" : source);
  }

  #queueRender() {
    if (this.#renderQueued) return;
    this.#renderQueued = true;
    requestAnimationFrame(() => {
      this.#renderQueued = false;
      this.#renderNow();
    });
  }

  #renderNow() {
    if (!this.#universe || !this.#context) return;
    this.#context.putImageData(this.#universe.getImageDataTransparent(0), 0, 0);
  }

  #renderLayers() {
    if (!this.#universe || !this.shadowRoot) return;
    const list = this.shadowRoot.querySelector(".layer-list");
    list.replaceChildren();
    for (const layer of [...this.#layers].reverse()) {
      const row = document.createElement("li");
      row.className = "layer-row";
      const select = document.createElement("button");
      select.type = "button";
      select.className = "layer-select";
      select.dataset.selectLayer = layer.name;
      select.setAttribute("aria-current", String(layer.name === this.#selectedLayer));
      select.textContent = layer.name;
      const visibility = document.createElement("button");
      visibility.type = "button";
      visibility.className = "visibility";
      visibility.dataset.toggleVisibility = layer.name;
      visibility.setAttribute("aria-pressed", String(layer.visible));
      visibility.textContent = layer.visible ? "Visible" : "Hidden";
      visibility.setAttribute("aria-label", `${layer.visible ? "Hide" : "Show"} ${layer.name}`);
      row.append(select, visibility);
      list.append(row);
    }
    const layer = this.#findLayer(this.#selectedLayer);
    const opacity = this.shadowRoot.querySelector(".opacity");
    opacity.value = String(Math.round(layer.opacity * 100 / 255));
    this.shadowRoot.querySelector(".opacity-value").value = `${opacity.value}%`;
  }

  #createLayer(prefix, requestedName) {
    let label = typeof requestedName === "string" ? requestedName.trim() : "";
    if (label && (label.length > 80 || /[\u0000-\u001f\u007f]/.test(label) || label === "__base__")) {
      throw new TypeError("Layer names must be 1–80 printable characters and cannot be __base__.");
    }
    if (label && this.#layers.some((layer) => layer.name === label)) throw new Error(`Layer already exists: ${label}`);
    while (!label) {
      const candidate = `${prefix} ${this.#nextLayerNumber++}`;
      if (!this.#layers.some((layer) => layer.name === candidate)) label = candidate;
    }
    this.#universe.addLayer(label, this.#canvas.width, this.#canvas.height);
    this.#universe.setCurrentLayer(label);
    this.#layers.push({ name: label, visible: true, opacity: 255 });
    this.#selectedLayer = label;
    return label;
  }

  #findLayer(name) {
    const layer = this.#layers.find((item) => item.name === name);
    if (!layer) throw new Error(`Unknown layer: ${name}`);
    return layer;
  }

  #selectLayer(name) {
    this.#findLayer(name);
    this.#universe.setCurrentLayer(name);
    this.#selectedLayer = name;
    this.#renderLayers();
    this.#dispatchStateChange();
  }

  #setLayerOpacity(name, percent, commit = true) {
    const layer = this.#findLayer(name);
    const value = Math.round(Math.max(0, Math.min(100, percent)) * 255 / 100);
    layer.opacity = value;
    this.#universe.setLayerAlpha(name, value);
    this.#renderNow();
    if (commit) this.#commitEdit("layer-opacity");
  }

  #clearLayer(name) {
    this.#findLayer(name);
    this.#universe.clearLayer(name);
    this.#renderNow();
    this.#commitEdit("clear-layer");
    this.#setStatus(`Cleared ${name}.`);
  }

  #drawStroke({ color = this.#brushColor, size = this.#brushSize, points = [] }, source = "draw", eraser = this.#eraserEnabled) {
    if (typeof eraser !== "boolean") throw new TypeError("Stroke eraser setting must be a boolean.");
    if (!/^#[\da-f]{6}$/i.test(color)) throw new TypeError("color must be a six-digit hex color.");
    if (!Number.isFinite(size) || size < 1 || size > 100) throw new RangeError("size must be between 1 and 100.");
    if (!Array.isArray(points) || points.length < 1 || points.length > 2048) throw new RangeError("points must contain between 1 and 2048 points.");
    for (const point of points) {
      if (!Number.isFinite(point?.x) || !Number.isFinite(point?.y)) throw new TypeError("Each point needs finite x and y coordinates.");
    }
    const rgb = Number.parseInt(color.slice(1), 16);
    const argb = ((0xff << 24) | rgb) >>> 0;
    const brushId = this.#universe.createBrush("circle", 9, 9, size, 1, 1, 0.2, 0, 0, 0, 0, 0, this.#brushSeed++);
    try {
      for (const point of points) {
        const x = Math.max(0, Math.min(this.#canvas.width, point.x));
        const y = Math.max(0, Math.min(this.#canvas.height, point.y));
        const pressure = Number.isFinite(point.pressure) ? Math.max(0.05, Math.min(1, point.pressure)) : 1;
        if (eraser) {
          this.#universe.brushSampleEraser(brushId, x, y, pressure, 0, 0, 0, performance.now());
        } else {
          this.#universe.brushSampleSolid(brushId, x, y, pressure, 0, 0, 0, performance.now(), argb);
        }
      }
    } finally {
      this.#universe.destroyBrush(brushId);
    }
    this.#renderNow();
    this.#commitEdit(source);
  }

  #commitEdit(source) {
    this.#renderNow();
    this.dispatchEvent(new CustomEvent("paint-change", {
      bubbles: true,
      composed: true,
      detail: {
        source,
        width: this.#canvas.width,
        height: this.#canvas.height,
        layers: this.#layers.map(({ name, visible, opacity }) => ({ name, visible, opacity: opacity / 255 })),
        selectedLayer: this.#selectedLayer,
        brushColor: this.#brushColor,
        brushSize: this.#brushSize,
        eraserEnabled: this.#eraserEnabled,
      },
    }));
    this.#dispatchStateChange();
  }

  #setStatus(message, error = false) {
    if (!this.#status) return;
    this.#status.textContent = message;
    this.#status.toggleAttribute("data-error", error);
  }

  #syncControlInputs() {
    if (!this.shadowRoot) return;
    this.shadowRoot.querySelector(".color").value = this.#brushColor;
    this.shadowRoot.querySelector(".size").value = String(this.#brushSize);
    this.shadowRoot.querySelector(".size-value").value = String(this.#brushSize);
    this.shadowRoot.querySelector(".tool").setAttribute("aria-pressed", String(this.#eraserEnabled));
    this.#canvas.style.cursor = this.#eraserEnabled ? "cell" : "crosshair";
  }

  #stateSnapshot() {
    return {
      width: this.#canvas.width,
      height: this.#canvas.height,
      selectedLayer: this.#selectedLayer,
      layers: this.#layers.map(({ name, visible, opacity }) => ({ name, visible, opacity: opacity / 255 })),
      brushColor: this.#brushColor,
      brushSize: this.#brushSize,
      eraserEnabled: this.#eraserEnabled,
    };
  }

  #dispatchStateChange() {
    if (!this.#universe) return;
    this.dispatchEvent(new CustomEvent("paint-state-change", {
      bubbles: true,
      composed: true,
      detail: this.#stateSnapshot(),
    }));
  }

  #download(blob) {
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = "paint.png";
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }

  #webmcpName(action) { return `${this.#instancePrefix}-${action}`; }

  #syncWebMcp() {
    if (!this.hasAttribute("webmcp")) {
      this.#webmcpController?.abort();
      this.#webmcpController = undefined;
      return;
    }
    const modelContext = this.ownerDocument?.defaultView?.document?.modelContext;
    if (!modelContext || typeof modelContext.registerTool !== "function" || this.#webmcpController) return;
    const controller = new AbortController();
    this.#webmcpController = controller;
    const state = () => JSON.stringify(this.#stateSnapshot());
    const tools = [
      {
        name: this.#webmcpName("get-state"),
        description: "Get canvas size, selected layer, layer visibility, and opacity.",
        inputSchema: { type: "object", properties: {}, additionalProperties: false },
        annotations: { readOnlyHint: true },
        execute: async () => { await this.ready; return state(); },
      },
      {
        name: this.#webmcpName("draw-stroke"),
        description: "Draw a stroke on the selected layer using canvas pixel coordinates.",
        inputSchema: {
          type: "object",
          properties: {
            color: { type: "string", pattern: "^#[0-9a-fA-F]{6}$" },
            size: { type: "number", minimum: 1, maximum: 100 },
            eraser: { type: "boolean", description: "Erase with transparent DestinationOut compositing instead of drawing color." },
            points: { type: "array", minItems: 1, maxItems: 2048, items: { type: "object", properties: { x: { type: "number" }, y: { type: "number" }, pressure: { type: "number", minimum: 0, maximum: 1 } }, required: ["x", "y"], additionalProperties: false } },
          },
          required: ["color", "size", "points"],
          additionalProperties: false,
        },
        execute: async (input) => { await this.ready; this.#drawStroke(input, "webmcp-draw", input.eraser ?? this.#eraserEnabled); return input.eraser ?? this.#eraserEnabled ? "Erased stroke from the selected layer." : "Stroke drawn on the selected layer."; },
      },
      {
        name: this.#webmcpName("add-layer"),
        description: "Add an empty layer and select it.",
        inputSchema: { type: "object", properties: {}, additionalProperties: false },
        execute: async () => { await this.ready; const name = this.#createLayer("Layer"); this.#renderLayers(); this.#commitEdit("webmcp-add-layer"); return `Added ${name}.`; },
      },
      {
        name: this.#webmcpName("select-layer"),
        description: "Select an existing paint layer by its name.",
        inputSchema: { type: "object", properties: { name: { type: "string" } }, required: ["name"], additionalProperties: false },
        execute: async ({ name }) => { await this.ready; this.#selectLayer(name); return `Selected ${name}.`; },
      },
      {
        name: this.#webmcpName("set-layer-visibility"),
        description: "Show or hide a layer by its name.",
        inputSchema: { type: "object", properties: { name: { type: "string" }, visible: { type: "boolean" } }, required: ["name", "visible"], additionalProperties: false },
        execute: async ({ name, visible }) => {
          await this.ready;
          const layer = this.#findLayer(name);
          layer.visible = visible;
          if (visible) this.#universe.setEnable(name); else this.#universe.setDisable(name);
          this.#renderLayers(); this.#renderNow(); this.#commitEdit("webmcp-layer-visibility");
          return `${name} is ${visible ? "visible" : "hidden"}.`;
        },
      },
      {
        name: this.#webmcpName("set-layer-opacity"),
        description: "Set a layer opacity from 0 (transparent) to 1 (opaque).",
        inputSchema: { type: "object", properties: { name: { type: "string" }, opacity: { type: "number", minimum: 0, maximum: 1 } }, required: ["name", "opacity"], additionalProperties: false },
        execute: async ({ name, opacity }) => { await this.ready; this.#setLayerOpacity(name, opacity * 100); return `Set ${name} opacity to ${opacity}.`; },
      },
      {
        name: this.#webmcpName("clear-layer"),
        description: "Erase all pixels from one named layer.",
        inputSchema: { type: "object", properties: { name: { type: "string" } }, required: ["name"], additionalProperties: false },
        annotations: { consequentialHint: true },
        execute: async ({ name }) => { await this.ready; this.#clearLayer(name); return `Cleared ${name}.`; },
      },
      {
        name: this.#webmcpName("clear-canvas"),
        description: "Erase every layer in this paint canvas.",
        inputSchema: { type: "object", properties: {}, additionalProperties: false },
        annotations: { consequentialHint: true },
        execute: async () => {
          await this.ready;
          for (const layer of this.#layers) this.#universe.clearLayer(layer.name);
          this.#renderNow(); this.#commitEdit("webmcp-clear-canvas");
          return "Cleared all canvas layers.";
        },
      },
      {
        name: this.#webmcpName("download-png"),
        description: "Export the current canvas as a PNG file download.",
        inputSchema: { type: "object", properties: {}, additionalProperties: false },
        execute: async () => { await this.ready; const blob = await this.exportImage(); this.#download(blob); return "Started PNG download."; },
      },
    ];
    Promise.all(tools.map((tool) => Promise.resolve().then(() => modelContext.registerTool(tool, { signal: controller.signal }))))
      .catch((error) => {
        if (!controller.signal.aborted) this.#setStatus(`WebMCP registration failed: ${error.message}`, true);
        controller.abort();
        if (this.#webmcpController === controller) this.#webmcpController = undefined;
      });
  }
}

if (!customElements.get("wasm-paint-tool")) {
  customElements.define("wasm-paint-tool", WasmPaintTool);
}
