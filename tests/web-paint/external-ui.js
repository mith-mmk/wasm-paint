import { shapePathCommands } from "./paint-tool.js";

const $ = (selector) => document.querySelector(selector);
const paint = $("wasm-paint-tool");
const thumbnails = new Map();
const editedLayers = new Set();
let state,
  activeTool = "pencil",
  zoom = 100,
  fitMode = true,
  hue = 52,
  changeRevision = 0,
  savedHistoryId,
  savedRevision = 0,
  saveMarkerInitialized = false,
  dirty = false,
  currentFileHandle,
  saveInProgress = false,
  eyedropperReturn,
  shapeDrag,
  shapeKind = "rectangle",
  shapeFilled = false,
  lastSample;
let ready = false,
  sampleLoading = false;
const tools = [
  ["pencil", "鉛筆"],
  ["brush", "ブラシ"],
  ["eraser", "消しゴム"],
  ["fill", "塗りつぶし"],
  ["eyedropper", "スポイト"],
  ["text", "文字", true],
  ["shapes", "図形"],
];

function message(text, error = false) {
  $("#status").textContent = text;
  $("#status").dataset.error = String(error);
}
function documentStem(value) {
  return String(value ?? "")
    .trim()
    .replace(/\.(?:png|jpe?g|gif|webp|bmp|avif|svg|tiff?)$/iu, "") || "無題";
}
function updateDirtyIndicator() {
  const name = $("#document-name");
  name.dataset.dirty = String(dirty);
  name.setAttribute("aria-label", dirty ? `${name.textContent}、未保存の変更あり` : name.textContent);
}
function setDocumentName(value) {
  const name = documentStem(value);
  $("#document-name").textContent = name;
  document.title = `${name} — ペイント`;
  updateDirtyIndicator();
}
function setDirty(value) {
  dirty = value;
  updateDirtyIndicator();
}
async function run(action) {
  try {
    await action();
  } catch (error) {
    message(error.message, true);
  }
}
function icon(name) {
  const img = document.createElement("img");
  img.src = new URL(`./assets/icons/${name}.svg`, import.meta.url).href;
  img.alt = "";
  return img;
}
for (const target of [$("#ribbon-tools"), $("#side-tools")]) {
  for (const [id, label, unavailable] of tools) {
    const button = document.createElement("button");
    button.className = "tool-button";
    button.dataset.tool = id;
    button.disabled = Boolean(unavailable);
    button.title = unavailable ? `${label}は未対応です` : label;
    button.setAttribute("aria-pressed", String(id === activeTool));
    button.append(icon(id), document.createTextNode(label));
    target.append(button);
  }
}
const colors = [
  "#252525",
  "#858585",
  "#b6b6b6",
  "#ffffff",
  "#ff553e",
  "#ffad30",
  "#ffe331",
  "#80c83c",
  "#209347",
  "#38d9ec",
  "#49b7f3",
  "#2485f5",
  "#204fa6",
  "#8048e6",
  "#c880ef",
  "#f3a1ed",
  "#86534f",
  "#92652c",
];
for (const color of colors) {
  const button = document.createElement("button");
  button.className = "swatch";
  button.style.backgroundColor = color;
  button.dataset.color = color;
  button.title = color.toUpperCase();
  button.setAttribute("aria-label", `色 ${color.toUpperCase()}`);
  $("#palette").append(button);
}

function renderLayers() {
  // Retain row nodes while dragging a slider or navigating with the keyboard.
  const list = $("#layers");
  for (const layer of [...state.layers].reverse()) {
    let row = [...list.children].find(
      (item) => item.dataset.layer === layer.name,
    );
    if (!row) {
      row = document.createElement("div");
      row.className = "layer-row";
      row.dataset.layer = layer.name;
      const visibility = document.createElement("button");
      visibility.className = "layer-visibility";
      visibility.addEventListener("click", () =>
        run(() => {
          const current = state.layers.find((item) => item.name === layer.name);
          return paint.setLayerVisibility(layer.name, !current.visible);
        }),
      );
      const select = document.createElement("button");
      select.className = "layer-select";
      select.title = layer.name;
      select.setAttribute("aria-label", `レイヤー ${layer.name} を選択`);
      const thumb = document.createElement("span");
      thumb.className = "layer-thumb";
      const label = document.createElement("span");
      label.textContent = layer.name === "main" ? "背景" : layer.name;
      select.append(thumb, label);
      select.addEventListener("click", () =>
        run(() => paint.selectLayer(layer.name)),
      );
      const opacity = document.createElement("input");
      Object.assign(opacity, {
        type: "range",
        min: "0",
        max: "100",
        value: "100",
      });
      opacity.setAttribute("aria-label", `${layer.name} の不透明度`);
      opacity.addEventListener("input", () =>
        run(() =>
          paint.setLayerOpacity(layer.name, Number(opacity.value) / 100, false),
        ),
      );
      opacity.addEventListener("change", () =>
        run(() => paint.setLayerOpacity(layer.name, Number(opacity.value) / 100)),
      );
      row.append(visibility, select, opacity, document.createElement("output"));
      list.prepend(row);
    }
    row.classList.toggle("selected", layer.name === state.selectedLayer);
    row
      .querySelector(".layer-select")
      .setAttribute("aria-pressed", String(layer.name === state.selectedLayer));
    const visibility = row.querySelector(".layer-visibility");
    visibility.setAttribute(
      "aria-label",
      `${layer.name} を${layer.visible ? "非表示" : "表示"}`,
    );
    visibility.setAttribute("aria-pressed", String(layer.visible));
    visibility.replaceChildren(icon(layer.visible ? "eye" : "eye-off"));
    const percent = Math.round(layer.opacity * 100);
    row.querySelector("input").value = percent;
    row.querySelector("output").value = `${percent}%`;
    const thumb = row.querySelector(".layer-thumb");
    const source = thumbnails.get(layer.name);
    if (source) {
      if (thumb.dataset.source !== source) {
        const img = document.createElement("img");
        Object.assign(img, { src: source, alt: "", className: "layer-thumb" });
        thumb.replaceChildren(img);
        thumb.dataset.source = source;
      }
    } else {
      thumb.replaceChildren(
        ...(editedLayers.has(layer.name) ? [icon("brush")] : []),
      );
      thumb.title = editedLayers.has(layer.name)
        ? "描画済み（プレビュー未対応）"
        : "透明なレイヤー";
      delete thumb.dataset.source;
    }
  }
}

function update(next) {
  state = next;
  if (!saveMarkerInitialized) {
    savedHistoryId = state.historyId;
    savedRevision = changeRevision;
    saveMarkerInitialized = true;
  }
  const isAtSavedState = state.historyId !== null && savedHistoryId !== null
    ? state.historyId === savedHistoryId
    : changeRevision === savedRevision;
  setDirty(!isAtSavedState);
  document.querySelectorAll('[data-action="undo"]').forEach((button) => {
    button.disabled = !state.canUndo;
  });
  document.querySelectorAll('[data-action="redo"]').forEach((button) => {
    button.disabled = !state.canRedo;
  });
  if (state.eraserEnabled) activeTool = "eraser";
  else if (activeTool === "eraser") activeTool = "pencil";
  $("#color").value = state.brushColor;
  if (document.activeElement !== $("#hex"))
    $("#hex").value = state.brushColor.toUpperCase();
  $("#size").value = state.brushSize;
  if (document.activeElement !== $("#size-value"))
    $("#size-value").value = state.brushSize;
  const selected = state.layers.find(
    (layer) => layer.name === state.selectedLayer,
  );
  $("#opacity").value = Math.round(selected.opacity * 100);
  $("#opacity-value").value = `${$("#opacity").value}%`;
  document
    .querySelectorAll("[data-tool]")
    .forEach((button) =>
      button.setAttribute(
        "aria-pressed",
        String(button.dataset.tool === activeTool),
      ),
    );
  document
    .querySelectorAll("[data-color]")
    .forEach((button) =>
      button.setAttribute(
        "aria-pressed",
        String(button.dataset.color === state.brushColor),
      ),
    );
  renderLayers();
  drawColorWheel();
}
paint.addEventListener("paint-state-change", (event) => update(event.detail));
paint.addEventListener("paint-change", (event) => {
  const source = event.detail.source;
  changeRevision += 1;
  setDirty(true);
  if (source === "undo" || source === "redo") {
    for (const name of event.detail.changedLayers ?? []) {
      if (thumbnails.has(name)) URL.revokeObjectURL(thumbnails.get(name));
      thumbnails.delete(name);
    }
    editedLayers.clear();
    for (const name of event.detail.nonEmptyLayers ?? []) editedLayers.add(name);
    if (state) renderLayers();
    message("未保存の変更があります");
    return;
  }
  if (
    [
      "draw",
      "api-draw",
      "fill",
      "shape",
      "clear-layer",
      "clear-canvas",
      "webmcp-clear-canvas",
    ].includes(source)
  ) {
    const name = event.detail.selectedLayer;
    if (source.endsWith("clear-canvas")) {
      for (const url of thumbnails.values()) URL.revokeObjectURL(url);
      thumbnails.clear();
      editedLayers.clear();
    } else {
      if (thumbnails.has(name)) URL.revokeObjectURL(thumbnails.get(name));
      thumbnails.delete(name);
      if (source === "clear-layer") editedLayers.delete(name);
      else editedLayers.add(name);
    }
    if (state) renderLayers();
  }
  message("未保存の変更があります");
});

function paintCanvas() {
  return paint.shadowRoot?.querySelector("canvas");
}
function canvasFromEvent(event) {
  return event.composedPath().find((node) => node instanceof HTMLCanvasElement);
}
function sampleAt(canvas, x, y) {
  const px = Math.max(0, Math.min(canvas.width - 1, x));
  const py = Math.max(0, Math.min(canvas.height - 1, y));
  const [r, g, b, alpha] = canvas.getContext("2d").getImageData(px, py, 1, 1).data;
  const hex = `#${[r, g, b].map((value) => value.toString(16).padStart(2, "0")).join("")}`;
  lastSample = { x: px, y: py, hex, alpha };
  $("#eyedropper-swatch").classList.toggle("has-color", alpha > 0);
  $("#eyedropper-swatch").style.backgroundColor = alpha
    ? `rgba(${r}, ${g}, ${b}, ${alpha / 255})`
    : "transparent";
  $("#eyedropper-hex").textContent = alpha ? hex.toUpperCase() : "透明";
  $("#eyedropper-details").textContent = `位置 ${px + 1}, ${py + 1} · 不透明度 ${Math.round((alpha / 255) * 100)}%`;
  $("#eyedropper-preview").hidden = false;
  return lastSample;
}
function sampleFromPointer(canvas, event) {
  const point = pointFromPointer(canvas, event);
  return sampleAt(canvas, point.x, point.y);
}
function pointFromPointer(canvas, event) {
  const rect = canvas.getBoundingClientRect();
  const contentX = event.clientX - rect.left - canvas.clientLeft;
  const contentY = event.clientY - rect.top - canvas.clientTop;
  return {
    x: Math.max(0, Math.min(canvas.width - 1, Math.floor((contentX * canvas.width) / canvas.clientWidth))),
    y: Math.max(0, Math.min(canvas.height - 1, Math.floor((contentY * canvas.height) / canvas.clientHeight))),
  };
}
function clearShapePreview() {
  const preview = $("#shape-preview");
  preview.getContext("2d").clearRect(0, 0, preview.width, preview.height);
  preview.hidden = true;
}
function renderShapePreview() {
  if (!shapeDrag) return;
  const preview = $("#shape-preview");
  const stage = $(".canvas-stage");
  const canvasRect = shapeDrag.canvas.getBoundingClientRect();
  const stageRect = stage.getBoundingClientRect();
  preview.width = shapeDrag.canvas.width;
  preview.height = shapeDrag.canvas.height;
  preview.style.left = `${canvasRect.left - stageRect.left}px`;
  preview.style.top = `${canvasRect.top - stageRect.top}px`;
  preview.style.width = `${canvasRect.width}px`;
  preview.style.height = `${canvasRect.height}px`;
  preview.hidden = false;
  const context = preview.getContext("2d");
  context.clearRect(0, 0, preview.width, preview.height);
  const path = new Path2D(shapePathCommands(
    shapeDrag.kind,
    shapeDrag.start.x,
    shapeDrag.start.y,
    shapeDrag.end.x,
    shapeDrag.end.y,
  ));
  if (shapeDrag.filled) {
    context.globalAlpha = 0.25;
    context.fillStyle = shapeDrag.color;
    context.fill(path);
    context.globalAlpha = 1;
  }
  context.strokeStyle = shapeDrag.color;
  context.lineWidth = shapeDrag.size;
  context.lineCap = "round";
  context.lineJoin = "round";
  context.stroke(path);
}
function finishShapeDrag(event) {
  if (!shapeDrag || shapeDrag.pointerId !== event.pointerId) return;
  const drag = shapeDrag;
  if (event.type === "pointerup") drag.end = pointFromPointer(drag.canvas, event);
  shapeDrag = undefined;
  clearShapePreview();
  if (event.type !== "pointerup") return;
  event.preventDefault();
  run(async () => {
    const changed = await paint.drawShape(drag.kind, drag.start, drag.end, {
      color: drag.color,
      size: drag.size,
      filled: drag.filled,
    });
    message(changed ? "図形を描画しました" : "図形の大きさが足りません");
  });
}
function focusToolButton(tool) {
  [...document.querySelectorAll("[data-tool]")]
    .find((button) => button.dataset.tool === tool && !button.disabled && button.getClientRects().length)
    ?.focus();
}
async function leaveEyedropper({ focus = true, status = "" } = {}) {
  if (!eyedropperReturn) return;
  const previous = eyedropperReturn;
  eyedropperReturn = undefined;
  lastSample = undefined;
  activeTool = previous.tool;
  $("#eyedropper-preview").hidden = true;
  paint.removeAttribute("data-eyedropper");
  const canvas = paintCanvas();
  if (canvas) {
    if (previous.tabindex === null) canvas.removeAttribute("tabindex");
    else canvas.setAttribute("tabindex", previous.tabindex);
    if (previous.ariaLabel === null) canvas.removeAttribute("aria-label");
    else canvas.setAttribute("aria-label", previous.ariaLabel);
    canvas.removeAttribute("aria-keyshortcuts");
  }
  await paint.setEraserEnabled(previous.tool === "eraser");
  await paint.setBrushSize(previous.size);
  if (focus) focusToolButton(previous.tool);
  if (status) message(status);
}
async function enterEyedropper() {
  const canvas = paintCanvas();
  if (!canvas || !state) return;
  eyedropperReturn = {
    tool: activeTool,
    size: state.brushSize,
    tabindex: canvas.getAttribute("tabindex"),
    ariaLabel: canvas.getAttribute("aria-label"),
  };
  activeTool = "eyedropper";
  paint.dataset.eyedropper = "true";
  canvas.tabIndex = 0;
  canvas.setAttribute("aria-label", "スポイト。矢印キーで色を確認し、Enterで取得、Escapeで取消");
  canvas.setAttribute("aria-keyshortcuts", "ArrowUp ArrowDown ArrowLeft ArrowRight Enter Escape");
  await paint.setEraserEnabled(false);
  sampleAt(canvas, Math.floor(canvas.width / 2), Math.floor(canvas.height / 2));
  canvas.focus({ preventScroll: true });
  message("色を確認中 · クリックまたはEnterで取得 · Escで取消");
}
async function chooseTool(tool) {
  if (tool === "eyedropper") {
    if (eyedropperReturn)
      await leaveEyedropper({ status: "スポイトをキャンセルしました" });
    else await enterEyedropper();
    return;
  }
  if (eyedropperReturn) await leaveEyedropper({ focus: false });
  activeTool = tool;
  document.querySelectorAll("[data-tool]").forEach((button) => {
    button.setAttribute("aria-pressed", String(button.dataset.tool === tool));
  });
  await paint.setEraserEnabled(tool === "eraser");
  if (tool === "pencil") await paint.setBrushSize(5);
  if (tool === "brush") await paint.setBrushSize(18);
  if (tool === "eraser") await paint.setBrushSize(24);
  if (tool === "shapes" && !$('[data-panel="shapes"]').classList.contains("active"))
    $('[data-panel="shapes"]').click();
  message(`${tools.find(([id]) => id === tool)[1]}を選択`);
}

function updateShapePanelControls() {
  document.querySelectorAll("[data-shape-kind]").forEach((button) => {
    button.setAttribute("aria-pressed", String(button.dataset.shapeKind === shapeKind));
  });
  if (shapeKind === "line") shapeFilled = false;
  document.querySelectorAll("[data-shape-fill]").forEach((button) => {
    const filled = button.dataset.shapeFill === "true";
    button.disabled = filled && shapeKind === "line";
    button.setAttribute("aria-pressed", String(filled === shapeFilled));
  });
}

function renderShapePanel(target) {
  target.classList.add("shape-panel");
  const gallery = document.createElement("div");
  gallery.className = "shape-gallery";
  const galleryTitle = document.createElement("span");
  galleryTitle.className = "shape-group-title";
  galleryTitle.textContent = "図形の種類";
  const types = document.createElement("div");
  types.className = "shape-types";
  types.setAttribute("role", "group");
  types.setAttribute("aria-label", "図形の種類");
  for (const [kind, label, path] of [
    ["line", "直線", "M5 27 27 5"],
    ["rectangle", "四角形", "M5 6 H27 V27 H5 Z"],
    ["ellipse", "楕円", "M27 16 A11 10 0 1 1 5 16 A11 10 0 1 1 27 16 Z"],
    ["roundedRectangle", "角丸四角", "M10 6 H22 Q27 6 27 11 V22 Q27 27 22 27 H10 Q5 27 5 22 V11 Q5 6 10 6 Z"],
    ["triangle", "三角形", "M16 5 28 27 H4 Z"],
    ["diamond", "ひし形", "M16 4 29 16 16 28 3 16 Z"],
    ["pentagon", "五角形", "M16 4 28 13 23 28 H9 L4 13 Z"],
    ["hexagon", "六角形", "M10 5 H22 L29 16 22 27 H10 L3 16 Z"],
    ["star", "星形", "M16 3 20 12 H29 L22 18 25 28 16 22 7 28 10 18 3 12 H12 Z"],
    ["arrow", "矢印", "M4 12 H18 V5 L29 16 18 27 V20 H4 Z"],
  ]) {
    const button = document.createElement("button");
    button.type = "button";
    button.dataset.shapeKind = kind;
    button.title = label;
    button.setAttribute("aria-label", label);
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("viewBox", "0 0 32 32");
    svg.setAttribute("aria-hidden", "true");
    const outline = document.createElementNS("http://www.w3.org/2000/svg", "path");
    outline.setAttribute("d", path);
    svg.append(outline);
    const name = document.createElement("span");
    name.textContent = label;
    button.append(svg, name);
    types.append(button);
  }
  gallery.append(galleryTitle, types);
  const style = document.createElement("div");
  style.className = "shape-style";
  const styleTitle = document.createElement("span");
  styleTitle.className = "shape-group-title";
  styleTitle.textContent = "描画スタイル";
  const styleOptions = document.createElement("div");
  styleOptions.className = "shape-style-options";
  styleOptions.setAttribute("role", "group");
  styleOptions.setAttribute("aria-label", "描画スタイル");
  for (const [filled, label] of [[false, "輪郭のみ"], [true, "塗りつぶし"]]) {
    const button = document.createElement("button");
    button.type = "button";
    button.dataset.shapeFill = String(filled);
    button.textContent = label;
    styleOptions.append(button);
  }
  style.append(styleTitle, styleOptions);
  target.append(gallery, style);
  updateShapePanelControls();
}
async function selectEyedropperColor() {
  if (!lastSample || !lastSample.alpha) {
    message("透明なピクセルです。色のある場所を選択してください", true);
    return;
  }
  const { hex, alpha } = lastSample;
  await paint.setBrushColor(hex);
  await leaveEyedropper({ focus: true });
  message(
    alpha === 255
      ? `描画色を${hex.toUpperCase()}に変更しました`
      : `描画色を${hex.toUpperCase()}に変更しました · ピクセルの不透明度は引き継がれません`,
  );
}
function setZoom(value, fitted = false) {
  fitMode = fitted;
  zoom = Math.max(
    25,
    Math.min(200, fitted ? Math.floor(value) : Math.round(value)),
  );
  paint.style.width = `${(640 * zoom) / 100}px`;
  $("#zoom").value = zoom;
  $("#zoom-value").textContent = `${zoom}%`;
}
function fitCanvas() {
  const area = $(".canvas-area");
  const padding =
    parseFloat(getComputedStyle($(".canvas-stage")).paddingLeft) * 2;
  setZoom(
    Math.min(
      (area.clientWidth - padding) / 640,
      (area.clientHeight - padding) / 480,
    ) * 100,
    true,
  );
}
new ResizeObserver(() => {
  if (fitMode) fitCanvas();
}).observe($(".canvas-area"));

async function importImage(blob, title) {
  // Fit a reference image to the fixed backing canvas; preserve aspect ratio.
  const bitmap = await createImageBitmap(blob);
  let fitted;
  try {
    const canvas = document.createElement("canvas");
    canvas.width = 640;
    canvas.height = 480;
    const scale = Math.min(640 / bitmap.width, 480 / bitmap.height);
    canvas
      .getContext("2d")
      .drawImage(
        bitmap,
        (640 - bitmap.width * scale) / 2,
        (480 - bitmap.height * scale) / 2,
        bitmap.width * scale,
        bitmap.height * scale,
      );
    fitted = await new Promise((resolve, reject) =>
      canvas.toBlob(
        (value) =>
          value
            ? resolve(value)
            : reject(new Error("画像を読み込めませんでした")),
        "image/png",
      ),
    );
  } finally {
    bitmap.close();
  }
  const layer = await paint.loadImage(fitted);
  const url = URL.createObjectURL(fitted);
  thumbnails.set(layer, url);
  renderLayers();
  if (!currentFileHandle) setDocumentName(title);
  message("画像を新しいレイヤーに読み込みました");
}
function suggestedPngName() {
  return `${documentStem($("#document-name").textContent)}.png`;
}
function setSaveBusy(busy) {
  saveInProgress = busy;
  $("#document-name").setAttribute("aria-busy", String(busy));
  document
    .querySelectorAll('[data-action="save"], [data-action="save-as"], [data-action="download"]')
    .forEach((button) => (button.disabled = busy));
}
function openDownloadDialog() {
  const dialog = $("#save-dialog");
  if (dialog.open || saveInProgress) return;
  dialog.returnValue = "cancel";
  $("#save-filename").value = documentStem($("#document-name").textContent);
  dialog.showModal();
  $("#save-filename").select();
}
function downloadFilename(value) {
  return `${documentStem(value)}.png`;
}
async function writePng(handle, blob) {
  const writable = await handle.createWritable();
  let closed = false;
  try {
    await writable.write(blob);
    await writable.close();
    closed = true;
  } finally {
    if (!closed) await writable.abort?.().catch(() => {});
  }
}
async function requestWritePermission(handle) {
  if (typeof handle.queryPermission !== "function") return;
  let permission = await handle.queryPermission({ mode: "readwrite" });
  if (permission !== "granted" && typeof handle.requestPermission === "function")
    permission = await handle.requestPermission({ mode: "readwrite" });
  if (permission !== "granted")
    throw new Error("ファイルへの書き込みが許可されませんでした");
}
function finishPngSave(handle, savedMarker, blob) {
  currentFileHandle = handle;
  setDocumentName(handle.name);
  savedHistoryId = savedMarker.historyId;
  savedRevision = savedMarker.revision;
  const unchanged = state.historyId !== null && savedHistoryId !== null
    ? state.historyId === savedHistoryId
    : changeRevision === savedRevision;
  setDirty(!unchanged);
  message(
    unchanged
      ? `${handle.name} を保存しました · ${blob.size.toLocaleString()} bytes`
      : `${handle.name} を保存しました · 保存中に加えた変更は未保存です`,
  );
}
function reportSaveError(error) {
  if (error?.name === "AbortError") {
    message(dirty ? "保存をキャンセルしました · 未保存の変更があります" : "保存をキャンセルしました");
  } else {
    message(`保存できませんでした: ${error?.message ?? error}${dirty ? " · 未保存の変更があります" : ""}`, true);
  }
}
async function saveToNewHandle(pickerRequest) {
  try {
    const handle = await pickerRequest;
    if (!/\.png$/i.test(handle.name))
      throw new Error("保存ファイル名には .png を指定してください");
    const savedMarker = { historyId: state.historyId, revision: changeRevision };
    const blob = await paint.exportImage();
    await writePng(handle, blob);
    finishPngSave(handle, savedMarker, blob);
  } catch (error) {
    reportSaveError(error);
  } finally {
    setSaveBusy(false);
  }
}
function startSaveAs() {
  if (saveInProgress || $("#save-dialog").open) return;
  if (typeof window.showSaveFilePicker !== "function") {
    openDownloadDialog();
    return;
  }
  setSaveBusy(true);
  message("保存先を選択してください");
  let pickerRequest;
  try {
    pickerRequest = window.showSaveFilePicker({
      suggestedName: suggestedPngName(),
      types: [
        {
          description: "PNG画像",
          accept: { "image/png": [".png"] },
        },
      ],
      excludeAcceptAllOption: true,
    });
  } catch (error) {
    reportSaveError(error);
    setSaveBusy(false);
    return;
  }
  void saveToNewHandle(pickerRequest);
}
async function saveToCurrentHandle(handle) {
  try {
    await requestWritePermission(handle);
    const savedMarker = { historyId: state.historyId, revision: changeRevision };
    const blob = await paint.exportImage();
    await writePng(handle, blob);
    finishPngSave(handle, savedMarker, blob);
  } catch (error) {
    reportSaveError(error);
  } finally {
    setSaveBusy(false);
  }
}
function startSave() {
  if (saveInProgress || $("#save-dialog").open) return;
  if (!currentFileHandle) {
    startSaveAs();
    return;
  }
  const handle = currentFileHandle;
  setSaveBusy(true);
  message("保存中…");
  void saveToCurrentHandle(handle);
}
async function downloadPngFromDialog() {
  if (saveInProgress) return;
  const filename = downloadFilename($("#save-filename").value);
  setSaveBusy(true);
  message("PNGを準備しています…");
  try {
    const blob = await paint.exportImage();
    const url = URL.createObjectURL(blob);
    const link = Object.assign(document.createElement("a"), {
      href: url,
      download: filename,
    });
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 5000);
    message(
      `${filename} のダウンロードを開始しました。保存完了はブラウザーで確認してください${dirty ? " · 未保存の変更があります" : ""}`,
    );
  } catch (error) {
    reportSaveError(error);
  } finally {
    setSaveBusy(false);
  }
}
const actions = {
  undo: () => paint.undo(),
  redo: () => paint.redo(),
  import: () => $("#image").click(),
  add: () => paint.addLayer(),
  clear: () => {
    const dialog = $("#clear-dialog");
    if (dialog.open) return;
    dialog.returnValue = "cancel";
    dialog.showModal();
  },
  fit: fitCanvas,
  actual: () => setZoom(100),
  "zoom-in": () => setZoom(zoom + 10),
  "zoom-out": () => setZoom(zoom - 10),
  sample: async () => {
    if (sampleLoading) return;
    sampleLoading = true;
    try {
      message("サンプルを読み込み中…");
      const response = await fetch(new URL("./assets/meadow.png", import.meta.url));
      if (!response.ok) throw new Error("サンプル画像を読み込めませんでした");
      await importImage(await response.blob(), "草原のスケッチ");
    } finally {
      sampleLoading = false;
    }
  },
  save: startSave,
  "save-as": startSaveAs,
  download: openDownloadDialog,
};
document.addEventListener("click", (event) => {
  const button = event.target.closest("button");
  if (button?.dataset.action) {
    document
      .querySelectorAll(".menu[open]")
      .forEach((menu) => (menu.open = false));
    if (ready) run(actions[button.dataset.action]);
  }
  if (button?.dataset.tool && ready) run(() => chooseTool(button.dataset.tool));
  if (button?.dataset.shapeKind) {
    shapeKind = button.dataset.shapeKind;
    updateShapePanelControls();
  }
  if (button?.dataset.shapeFill && !button.disabled) {
    shapeFilled = button.dataset.shapeFill === "true";
    updateShapePanelControls();
  }
  if (button?.dataset.color && ready)
    run(() => paint.setBrushColor(button.dataset.color));
  if (!event.target.closest(".menu"))
    document
      .querySelectorAll(".menu[open]")
      .forEach((menu) => (menu.open = false));
});
document.addEventListener("keydown", (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
    event.preventDefault();
    if (!ready || saveInProgress) return;
    if (event.shiftKey) startSaveAs();
    else startSave();
    return;
  }
  const editableTarget = event.target instanceof HTMLElement && (
    event.target.isContentEditable ||
    ["INPUT", "TEXTAREA", "SELECT"].includes(event.target.tagName)
  );
  if ((event.ctrlKey || event.metaKey) && !editableTarget) {
    const key = event.key.toLowerCase();
    if (key === "z" || key === "y") {
      event.preventDefault();
      if (!ready) return;
      run(() => key === "y" || event.shiftKey ? paint.redo() : paint.undo());
      return;
    }
  }
  if (
    eyedropperReturn &&
    event.key === "Escape" &&
    !$("#save-dialog").open &&
    !$("#clear-dialog").open
  ) {
    event.preventDefault();
    run(() => leaveEyedropper({ status: "スポイトをキャンセルしました" }));
    return;
  }
  const canvas = canvasFromEvent(event);
  if (eyedropperReturn && canvas) {
    if (["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(event.key)) {
      event.preventDefault();
      const step = event.shiftKey ? 10 : 1;
      const x = lastSample?.x ?? Math.floor(canvas.width / 2);
      const y = lastSample?.y ?? Math.floor(canvas.height / 2);
      sampleAt(
        canvas,
        x + (event.key === "ArrowRight" ? step : event.key === "ArrowLeft" ? -step : 0),
        y + (event.key === "ArrowDown" ? step : event.key === "ArrowUp" ? -step : 0),
      );
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      run(selectEyedropperColor);
      return;
    }
  }
  if (event.key === "Escape")
    document
      .querySelectorAll(".menu[open]")
      .forEach((menu) => (menu.open = false));
});
$("#clear-dialog").addEventListener("close", () => {
  if ($("#clear-dialog").returnValue === "clear")
    run(() => paint.clearLayer(state.selectedLayer));
});
$("#save-dialog").addEventListener("close", () => {
  if ($("#save-dialog").returnValue === "download")
    run(downloadPngFromDialog);
  else
    message(dirty ? "保存をキャンセルしました · 未保存の変更があります" : "保存をキャンセルしました");
});
$("#save-cancel").addEventListener("click", () => $("#save-dialog").close("cancel"));
$("#image").addEventListener("change", (event) =>
  run(async () => {
    const input = event.currentTarget;
    const [file] = input.files ?? [];
    if (!file) return;
    try {
      await importImage(file, file.name);
    } finally {
      input.value = "";
    }
  }),
);
$("#color").addEventListener("input", (event) =>
  run(() => paint.setBrushColor(event.target.value)),
);
$("#hex").addEventListener("change", (event) =>
  run(async () => {
    const input = event.target;
    if (!/^#[\da-f]{6}$/i.test(input.value)) {
      input.value = state.brushColor.toUpperCase();
      throw new Error("色は #RRGGBB の形式で入力してください");
    }
    await paint.setBrushColor(input.value);
  }),
);
$("#white").addEventListener("click", () =>
  run(() => paint.setBrushColor("#ffffff")),
);
function setBrushSizeFromInput(event, { editing = false } = {}) {
  return run(async () => {
    const input = event.target;
    const value = Number(input.value);
    if (!Number.isFinite(value) || value < 1 || value > 100) {
      if (editing) return;
      input.value = state.brushSize;
      throw new Error("ブラシの大きさは1〜100で指定してください");
    }
    await paint.setBrushSize(value);
  });
}
$("#size").addEventListener("input", setBrushSizeFromInput);
$("#size-value").addEventListener("input", (event) =>
  setBrushSizeFromInput(event, { editing: true }),
);
$("#size-value").addEventListener("change", (event) =>
  setBrushSizeFromInput(event),
);
$("#opacity").addEventListener("input", (event) =>
  run(() =>
    paint.setLayerOpacity(
      state.selectedLayer,
      Number(event.target.value) / 100,
      false,
    ),
  ),
);
$("#opacity").addEventListener("change", (event) =>
  run(() =>
    paint.setLayerOpacity(
      state.selectedLayer,
      Number(event.target.value) / 100,
    ),
  ),
);
$("#zoom").addEventListener("input", (event) =>
  setZoom(Number(event.target.value)),
);
document.querySelectorAll(".collapse").forEach((button) =>
  button.addEventListener("click", () => {
    const collapsed = button.closest(".panel").classList.toggle("collapsed");
    button.setAttribute("aria-expanded", String(!collapsed));
  }),
);
document.querySelectorAll("[data-panel]").forEach((button) =>
  button.addEventListener("click", () => {
    document.querySelectorAll("[data-panel]").forEach((tab) => {
      tab.classList.toggle("active", tab === button);
      tab.setAttribute("aria-pressed", String(tab === button));
    });
    const panel = button.dataset.panel;
    $(".ribbon").classList.toggle("shapes-active", panel === "shapes");
    $("#ribbon-tools").hidden = ["canvas", "color", "shapes"].includes(panel);
    $("#brush-settings").hidden = ["canvas", "color"].includes(panel);
    const target = $("#context-actions");
    target.hidden = ["home", "draw"].includes(panel);
    target.classList.remove("shape-panel");
    target.replaceChildren();
    if (panel === "canvas") {
      for (const [action, label] of [
        ["import", "画像を開く"],
        ["save", "保存"],
        ["save-as", "名前を付けて保存"],
        ["download", "PNGをダウンロード…"],
        ["sample", "サンプルを開く"],
        ["fit", "画面に合わせる"],
        ["clear", "レイヤーをクリア…"],
      ]) {
        const item = document.createElement("button");
        item.dataset.action = action;
        item.textContent = label;
        target.append(item);
      }
    }
    if (panel === "color") {
      const text = document.createElement("p");
      text.textContent =
        "右のパレット・色相環で描画色を選べます。スポイトは色をプレビューし、クリックまたはEnterで取得します。Escで元のツールに戻ります。";
      target.append(text);
      const item = document.createElement("button");
      item.dataset.tool = "eyedropper";
      item.textContent = "スポイト";
      target.append(item);
      $(".color-panel").classList.remove("collapsed");
      $(".color-panel .collapse").setAttribute("aria-expanded", "true");
    }
    if (panel === "shapes") {
      renderShapePanel(target);
      if (ready && activeTool !== "shapes") run(() => chooseTool("shapes"));
    }
  }),
);

// A functional HSV color picker, computed from color values rather than an image.
function hsv(h, s, v) {
  const f = (n) => {
    const k = (n + h / 60) % 6;
    return Math.round(255 * v * (1 - s * Math.max(0, Math.min(k, 4 - k, 1))));
  };
  return [f(5), f(3), f(1)];
}
function currentHsv() {
  const [r, g, b] = state.brushColor
    .slice(1)
    .match(/../g)
    .map((value) => parseInt(value, 16) / 255);
  const max = Math.max(r, g, b),
    min = Math.min(r, g, b),
    delta = max - min;
  if (delta)
    hue =
      ((max === r
        ? (g - b) / delta
        : max === g
          ? (b - r) / delta + 2
          : (r - g) / delta + 4) *
        60 +
        360) %
      360;
  return [max ? delta / max : 0, max];
}
function drawColorWheel() {
  if (!state) return;
  const [s, v] = currentHsv();
  const ctx = $("#color-wheel").getContext("2d");
  const pixels = ctx.createImageData(240, 240);
  for (let y = 0; y < 240; y++)
    for (let x = 0; x < 240; x++) {
      const distance = Math.hypot(x - 120, y - 120);
      let rgb;
      if (distance >= 88 && distance <= 112)
        rgb = hsv(
          ((Math.atan2(y - 120, x - 120) * 180) / Math.PI + 450) % 360,
          1,
          1,
        );
      else if (x >= 64 && x <= 176 && y >= 64 && y <= 176)
        rgb = hsv(hue, (x - 64) / 112, 1 - (y - 64) / 112);
      if (rgb) pixels.data.set([...rgb, 255], (y * 240 + x) * 4);
    }
  ctx.putImageData(pixels, 0, 0);
  for (const [x, y] of [
    [
      120 + 100 * Math.cos(((hue - 90) * Math.PI) / 180),
      120 + 100 * Math.sin(((hue - 90) * Math.PI) / 180),
    ],
    [64 + s * 112, 64 + (1 - v) * 112],
  ]) {
    ctx.beginPath();
    ctx.arc(x, y, 5, 0, 2 * Math.PI);
    ctx.strokeStyle = "white";
    ctx.lineWidth = 3;
    ctx.stroke();
    ctx.strokeStyle = "#233047";
    ctx.lineWidth = 1;
    ctx.stroke();
  }
}
let wheelMode;
function pickColor(event) {
  if (!state || !wheelMode) return;
  const rect = $("#color-wheel").getBoundingClientRect();
  const x = ((event.clientX - rect.left) * 240) / rect.width,
    y = ((event.clientY - rect.top) * 240) / rect.height;
  let [s, v] = currentHsv();
  if (wheelMode === "hue") {
    hue = ((Math.atan2(y - 120, x - 120) * 180) / Math.PI + 450) % 360;
    s = s || 1;
    v = v || 1;
  } else {
    s = Math.max(0, Math.min(1, (x - 64) / 112));
    v = Math.max(0, Math.min(1, 1 - (y - 64) / 112));
  }
  run(() =>
    paint.setBrushColor(
      `#${hsv(hue, s, v)
        .map((value) => value.toString(16).padStart(2, "0"))
        .join("")}`,
    ),
  );
}
$("#color-wheel").addEventListener("pointerdown", (event) => {
  const rect = event.currentTarget.getBoundingClientRect();
  const x = ((event.clientX - rect.left) * 240) / rect.width,
    y = ((event.clientY - rect.top) * 240) / rect.height;
  const distance = Math.hypot(x - 120, y - 120);
  wheelMode =
    distance >= 88 && distance <= 112
      ? "hue"
      : x >= 64 && x <= 176 && y >= 64 && y <= 176
        ? "sv"
        : undefined;
  if (wheelMode) {
    event.currentTarget.setPointerCapture(event.pointerId);
    pickColor(event);
  }
});
$("#color-wheel").addEventListener("pointermove", pickColor);
for (const type of ["pointerup", "pointercancel", "lostpointercapture"])
  $("#color-wheel").addEventListener(type, () => (wheelMode = undefined));

run(async () => {
  await paint.ready;
  await paint.setBrushSize(5);
  await paint.setBrushColor("#ffe331");
  update(await paint.getState());
  ready = true;
  fitCanvas();
  paint.addEventListener(
    "pointermove",
    (event) => {
      if (shapeDrag && shapeDrag.pointerId === event.pointerId) {
        shapeDrag.end = pointFromPointer(shapeDrag.canvas, event);
        renderShapePreview();
        event.preventDefault();
        return;
      }
      if (!eyedropperReturn) return;
      const canvas = canvasFromEvent(event);
      if (canvas) sampleFromPointer(canvas, event);
    },
    true,
  );
  // Capture before the embedded component begins a stroke.
  paint.addEventListener(
    "pointerdown",
    (event) => {
      const canvas = canvasFromEvent(event);
      if (!canvas) return;
      if (!eyedropperReturn && activeTool !== "fill" && activeTool !== "shapes") return;
      event.stopImmediatePropagation();
      if (event.pointerType === "mouse" && event.button !== 0) return;
      event.preventDefault();
      if (activeTool === "shapes" && !eyedropperReturn) {
        const start = pointFromPointer(canvas, event);
        shapeDrag = {
          pointerId: event.pointerId,
          canvas,
          kind: shapeKind,
          start,
          end: start,
          color: state.brushColor,
          size: state.brushSize,
          filled: shapeFilled && shapeKind !== "line",
        };
        canvas.setPointerCapture(event.pointerId);
        renderShapePreview();
        return;
      }
      if (activeTool === "fill" && !eyedropperReturn) {
        const point = pointFromPointer(canvas, event);
        run(async () => {
          const changed = await paint.fillAt(point.x, point.y, {
            tolerance: Number($("#fill-tolerance").value),
          });
          message(changed ? "塗りつぶしました" : "塗りつぶす範囲に変更はありません");
        });
        return;
      }
      sampleFromPointer(canvas, event);
      if (!lastSample.alpha) {
        message("透明なピクセルです。色のある場所を選択してください", true);
        return;
      }
      run(selectEyedropperColor);
    },
    true,
  );
  for (const type of ["pointerup", "pointercancel", "lostpointercapture"]) {
    paint.addEventListener(type, finishShapeDrag, true);
  }
  $("#fill-tolerance").addEventListener("input", (event) => {
    $("#fill-tolerance-value").value = event.currentTarget.value;
  });
  message("描画できます · ファイルからサンプルも開けます");
});
