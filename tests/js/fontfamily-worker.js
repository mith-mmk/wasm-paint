import init, { Universe } from "../../wasm-paint/pkg/paint.js";

let universe;
let familyName = '';
let faceCount = 0;

async function workerInit(width, height) {
  await init();
  universe = new Universe(width, height);
  if (typeof universe.hasFontFeature === 'function' && !universe.hasFontFeature()) {
    throw new Error('wasm-paint must be built with --features font');
  }
  universe.clear(0xffffff);
  const image = universe.getImageData(0);
  postMessage({ message: 'init', image });
}

function renderText(request) {
  if (universe == null) {
    return;
  }

  universe.clear(0xffffff);
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
  universe.combine();

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
    },
  });
}

function beginFamily(request) {
  if (universe == null) {
    throw new Error('worker is not ready');
  }
  universe.resetFontFamily(request.familyName);
  familyName = request.familyName;
  faceCount = 0;
}

function beginFamilyFace(request) {
  if (universe == null) {
    throw new Error('worker is not ready');
  }
  universe.beginFontFamilyFace(
    request.faceId,
    Number(request.totalSize),
    request.fontName ?? undefined,
    Number(request.fontWeight ?? 400),
    request.fontStyle ?? 'normal',
    Number(request.fontStretch ?? 1),
  );
}

function appendFamilyChunk(request) {
  if (universe == null) {
    throw new Error('worker is not ready');
  }
  universe.appendFontFamilyChunk(
    request.faceId,
    Number(request.offset ?? 0),
    new Uint8Array(request.buffer),
  );
}

function finalizeFamilyFace(request) {
  if (universe == null) {
    throw new Error('worker is not ready');
  }
  universe.finalizeFontFamilyFace(request.faceId);
  faceCount = universe.fontFamilyFaceCount();
}

function finishFamily() {
  postMessage({ message: 'familyLoaded', familyName, faceCount });
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
      error: error instanceof Error ? error.message : String(error),
    });
  }
};
