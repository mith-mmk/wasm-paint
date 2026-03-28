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

function loadFamily(request) {
  if (universe == null) {
    throw new Error('worker is not ready');
  }
  if (!Array.isArray(request.faces) || request.faces.length === 0) {
    throw new Error('family requires at least one face');
  }

  universe.resetFontFamily(request.familyName);
  for (const face of request.faces) {
    universe.addFontToFamily(new Uint8Array(face.buffer));
  }
  familyName = request.familyName;
  faceCount = universe.fontFamilyFaceCount();
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
        loadFamily(data);
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
