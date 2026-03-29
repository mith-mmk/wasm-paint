const BUILD_ID = 'font-worker-20260329-1';

let universe;
let fontSource = '';
let rendererInfo = '';

async function loadWasm() {
  return import(`../../wasm-paint/pkg/paint.js?${BUILD_ID}`);
}

async function workerInit(width, height) {
  const { default: init, Universe } = await loadWasm();
  await init();
  universe = new Universe(width, height);
  if (typeof universe.hasFontFeature === 'function' && !universe.hasFontFeature()) {
    throw new Error('wasm-paint must be built with --features font');
  }
  rendererInfo =
    typeof universe.glyphRendererInfo === 'function'
      ? universe.glyphRendererInfo()
      : 'renderer info unavailable';
  universe.clear(0xffffff);
  const image = universe.getImageData(0);
  postMessage({ message: 'init', image, rendererInfo, buildId: BUILD_ID });
}

function renderText(text, fontSize) {
  if (universe == null) {
    return;
  }

  universe.clear(0xffffff);
  universe.drawText(text, 32, 96, fontSize, 0x111111);
  universe.combine();

  const lineCount = text === '' ? 0 : text.split(/\r?\n/).length;
  const image = universe.getImageData(0);
  postMessage({
    message: 'render',
    image,
    summary: {
      fontSource,
      charCount: Array.from(text).length,
      lineCount,
      fontSize,
      rendererInfo,
      buildId: BUILD_ID,
    },
  });
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
      case 'loadFont':
        if (universe == null) {
          throw new Error('worker is not ready');
        }
        universe.loadFont(new Uint8Array(data.buffer));
        fontSource = data.source ?? 'buffer';
        postMessage({ message: 'fontLoaded', source: fontSource });
        break;
      case 'render':
        if (universe == null) {
          throw new Error('worker is not ready');
        }
        renderText(data.text ?? '', Number(data.fontSize ?? 64));
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
