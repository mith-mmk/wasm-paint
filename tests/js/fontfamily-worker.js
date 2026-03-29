let universe;
let familyName = '';
let faceCount = 0;
const pendingFaces = new Map();
let initWasm;
let UniverseCtor;
let lastStage = 'startup';
let rendererInfo = '';
const BUILD_ID = new URL(self.location.href).searchParams.get('v') ?? 'fontfamily-worker-20260329-3';

function setStage(stage) {
  lastStage = stage;
}

async function ensureWasmBindings() {
  setStage('import wasm bindings');
  if (initWasm != null && UniverseCtor != null) {
    return;
  }

  const buildId = new URL(self.location.href).searchParams.get('v') ?? '';
  const suffix = buildId === '' ? '' : `?v=${encodeURIComponent(buildId)}`;
  const moduleUrl = new URL(`../../wasm-paint/pkg/paint.js${suffix}`, import.meta.url);
  const wasmUrl = new URL(`../../wasm-paint/pkg/paint_bg.wasm${suffix}`, import.meta.url);
  const pkg = await import(moduleUrl.href);
  initWasm = pkg.default;
  UniverseCtor = pkg.Universe;
  setStage('initialize wasm module');
  await initWasm(wasmUrl);
}

async function workerInit(width, height) {
  setStage('worker init');
  await ensureWasmBindings();
  setStage('construct universe');
  universe = new UniverseCtor(width, height);
  if (typeof universe.hasFontFeature === 'function' && !universe.hasFontFeature()) {
    throw new Error('wasm-paint must be built with --features font');
  }
  rendererInfo =
    typeof universe.glyphRendererInfo === 'function'
      ? universe.glyphRendererInfo()
      : 'renderer info unavailable';
  setStage('clear canvas');
  universe.clear(0xffffff);
  setStage('read initial image');
  const image = universe.getImageData(0);
  postMessage({ message: 'init', image, buildId: BUILD_ID, rendererInfo });
}

function renderText(request) {
  if (universe == null) {
    return;
  }

  const layoutInfo =
    typeof universe.inspectTextFamily === 'function'
      ? universe.inspectTextFamily(
          request.text ?? '',
          Number(request.fontSize ?? 48),
          request.fontName ?? undefined,
          Number(request.fontWeight ?? 400),
          request.fontStyle ?? 'normal',
          Number(request.fontStretch ?? 1),
        )
      : 'layout inspection unavailable';

  setStage('render clear');
  universe.clear(0xffffff);
  setStage('draw text family');
  universe.drawTextFamily(
    request.text ?? '',
    32,
    96,
    Number(request.fontSize ?? 48),
    0x111111,
    request.fontName ?? undefined,
    Number(request.fontWeight ?? 400),
    request.fontStyle ?? 'normal',
    Number(request.fontStretch ?? 1),
  );
  setStage('combine layers');
  universe.combine();

  setStage('read rendered image');
  const image = universe.getImageData(0);
  postMessage({
    message: 'render',
    image,
    summary: {
      familyName,
      faceCount,
      charCount: Array.from(request.text ?? '').length,
      fontSize: Number(request.fontSize ?? 48),
      fontWeight: Number(request.fontWeight ?? 400),
      fontStyle: request.fontStyle ?? 'normal',
      fontStretch: Number(request.fontStretch ?? 1),
      buildId: BUILD_ID,
      rendererInfo,
      layoutInfo,
    },
  });
}

function beginFamily(request) {
  if (universe == null) {
    throw new Error('worker is not ready');
  }
  setStage(`reset family ${request.familyName}`);
  universe.resetFontFamily(request.familyName);
  familyName = request.familyName;
  faceCount = 0;
  pendingFaces.clear();
}

function beginFamilyFace(request) {
  if (universe == null) {
    throw new Error('worker is not ready');
  }
  const totalSize = Number(request.totalSize);
  if (!Number.isFinite(totalSize) || totalSize <= 0) {
    throw new Error('totalSize must be a positive finite number');
  }
  pendingFaces.set(request.faceId, {
    totalSize,
    received: 0,
    buffer: new Uint8Array(totalSize),
    fontName: request.fontName ?? undefined,
    fontWeight: Number(request.fontWeight ?? 400),
    fontStyle: request.fontStyle ?? 'normal',
    fontStretch: Number(request.fontStretch ?? 1),
  });
}

function appendFamilyChunk(request) {
  if (universe == null) {
    throw new Error('worker is not ready');
  }
  const pending = pendingFaces.get(request.faceId);
  if (pending == null) {
    throw new Error(`unknown pending font face: ${request.faceId}`);
  }

  const offset = Number(request.offset ?? 0);
  const chunk = new Uint8Array(request.buffer);
  if (!Number.isInteger(offset) || offset < 0) {
    throw new Error(`invalid chunk offset: ${request.offset}`);
  }
  if (offset !== pending.received) {
    throw new Error(
      `unexpected chunk offset for ${request.faceId}: expected ${pending.received}, got ${offset}`,
    );
  }

  const end = offset + chunk.byteLength;
  if (end > pending.totalSize) {
    throw new Error(
      `chunk is out of range for ${request.faceId}: ${end}/${pending.totalSize}`,
    );
  }

  pending.buffer.set(chunk, offset);
  pending.received = end;
}

function finalizeFamilyFace(request) {
  if (universe == null) {
    throw new Error('worker is not ready');
  }
  const pending = pendingFaces.get(request.faceId);
  if (pending == null) {
    throw new Error(`unknown pending font face: ${request.faceId}`);
  }
  if (pending.received !== pending.totalSize) {
    throw new Error(
      `font ${request.faceId} is incomplete: ${pending.received}/${pending.totalSize} bytes`,
    );
  }

  setStage(`add font to family ${request.faceId}`);
  universe.addFontToFamilyWithDescriptor(
    pending.buffer,
    pending.fontName,
    pending.fontWeight,
    pending.fontStyle,
    pending.fontStretch,
  );
  pendingFaces.delete(request.faceId);
  setStage('count family faces');
  faceCount = universe.fontFamilyFaceCount();
}

function finishFamily() {
  postMessage({ message: 'familyLoaded', familyName, faceCount, buildId: BUILD_ID, rendererInfo });
}

onmessage = async function(ev) {
  const data = ev.data;
  if (data.command == null) {
    return;
  }

  try {
    switch (data.command) {
      case 'init':
        await workerInit(data.width, data.height);
        break;
      case 'loadFamily':
        beginFamily(data);
        break;
      case 'beginFamilyFace':
        beginFamilyFace(data);
        break;
      case 'appendFamilyChunk':
        appendFamilyChunk(data);
        break;
      case 'finalizeFamilyFace':
        finalizeFamilyFace(data);
        break;
      case 'finishFamily':
        finishFamily();
        break;
      case 'render':
        renderText(data);
        break;
      default:
        break;
    }
  } catch (error) {
    postMessage({
      message: 'error',
      error: `[${lastStage}] ${error instanceof Error ? error.message : String(error)}`,
    });
  }
};
