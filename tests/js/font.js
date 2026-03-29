import * as fps from './fps.js';

const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d');
const summary = document.getElementById('summary');
const fontUrlInput = document.getElementById('font-url');
const fontFileInput = document.getElementById('font-file');
const textInput = document.getElementById('font-text');
const fontSizeInput = document.getElementById('font-size');
const loadButton = document.getElementById('load-font');
const renderButton = document.getElementById('render-font');

const width = 1280;
const height = 960;
const BUILD_ID = 'font-ui-20260329-1';

canvas.width = width;
canvas.height = height;

const params = new URLSearchParams(window.location.search);
if (params.has('font')) {
  fontUrlInput.value = params.get('font') ?? '';
}
if (params.has('text')) {
  textInput.value = params.get('text') ?? textInput.value;
}
if (params.has('size')) {
  fontSizeInput.value = params.get('size') ?? fontSizeInput.value;
}

let pixelWorker;
let workerReady = false;
let fontLoaded = false;
let lastFontToken = '';

function setSummary(message) {
  if (summary != null) {
    summary.textContent = message;
  }
}

function currentRenderRequest() {
  return {
    text: textInput.value,
    fontSize: Number.parseFloat(fontSizeInput.value || '64'),
  };
}

function postRender() {
  if (!workerReady) {
    setSummary('worker is not ready');
    return;
  }
  if (!fontLoaded) {
    setSummary('load a font first');
    return;
  }

  pixelWorker.postMessage({
    command: 'render',
    ...currentRenderRequest(),
  });
}

async function loadFontFromUrl(url) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`failed to fetch font: ${response.status} ${response.statusText}`);
  }
  const buffer = await response.arrayBuffer();
  pixelWorker.postMessage(
    {
      command: 'loadFont',
      buffer,
      source: url,
    },
    [buffer],
  );
}

async function loadFontFromFile(file) {
  const buffer = await file.arrayBuffer();
  pixelWorker.postMessage(
    {
      command: 'loadFont',
      buffer,
      source: file.name,
    },
    [buffer],
  );
}

async function loadSelectedFont() {
  if (!workerReady) {
    setSummary('worker is not ready');
    return;
  }

  const file = fontFileInput.files != null ? fontFileInput.files[0] : null;
  const url = fontUrlInput.value.trim();

  if (file != null) {
    const token = `file:${file.name}:${file.size}:${file.lastModified}`;
    if (token === lastFontToken && fontLoaded) {
      postRender();
      return;
    }
    setSummary(`loading ${file.name}`);
    await loadFontFromFile(file);
    lastFontToken = token;
    return;
  }

  if (url !== '') {
    const token = `url:${url}`;
    if (token === lastFontToken && fontLoaded) {
      postRender();
      return;
    }
    setSummary(`loading ${url}`);
    await loadFontFromUrl(url);
    lastFontToken = token;
    return;
  }

  setSummary('select a font file or enter a font URL');
}

function workerInit() {
  const workerUrl = new URL(`./font-worker.js?${BUILD_ID}`, import.meta.url);
  pixelWorker = new Worker(workerUrl, { type: 'module' });

  pixelWorker.onmessage = (ev) => {
    const data = ev.data;
    if (data.message == null) {
      return;
    }

    switch (data.message) {
      case 'init':
        workerReady = true;
        if (data.image != null) {
          ctx.putImageData(data.image, 0, 0);
        }
        setSummary(
          `worker ready build=${data.buildId ?? BUILD_ID} renderer=${data.rendererInfo ?? 'unknown'}`,
        );
        if (fontUrlInput.value.trim() !== '') {
          loadSelectedFont().catch((error) => {
            setSummary(error instanceof Error ? error.message : String(error));
          });
        }
        break;
      case 'fontLoaded':
        fontLoaded = true;
        setSummary(`font loaded: ${data.source}`);
        postRender();
        break;
      case 'render':
        if (data.image != null) {
          ctx.putImageData(data.image, 0, 0);
        }
        if (data.summary != null) {
          const info = data.summary;
          setSummary(
            `font=${info.fontSource} chars=${info.charCount} lines=${info.lineCount} size=${info.fontSize}px build=${info.buildId ?? BUILD_ID} renderer=${info.rendererInfo ?? 'unknown'}`,
          );
        }
        break;
      case 'error':
        setSummary(data.error ?? 'unknown error');
        console.error(data.error);
        break;
      default:
        break;
    }
  };

  pixelWorker.postMessage({ command: 'init', width, height });

  const draw = () => {
    fps.fps.render();
    window.requestAnimationFrame(draw);
  };
  draw();
}

loadButton.addEventListener('click', () => {
  loadSelectedFont().catch((error) => {
    setSummary(error instanceof Error ? error.message : String(error));
  });
});

renderButton.addEventListener('click', () => {
  if (!fontLoaded) {
    loadSelectedFont().catch((error) => {
      setSummary(error instanceof Error ? error.message : String(error));
    });
    return;
  }
  postRender();
});

if (window.Worker) {
  workerInit();
} else {
  setSummary('Worker is not support.');
}
