import * as fps from './fps.js';

const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d');
const summary = document.getElementById('summary');
const familyNameInput = document.getElementById('family-name');
const fontUrlsInput = document.getElementById('font-urls');
const fontFilesInput = document.getElementById('font-files');
const textInput = document.getElementById('font-text');
const fontNameInput = document.getElementById('font-name');
const fontSizeInput = document.getElementById('font-size');
const fontWeightInput = document.getElementById('font-weight');
const fontStyleInput = document.getElementById('font-style');
const fontStretchInput = document.getElementById('font-stretch');
const loadButton = document.getElementById('load-family');
const renderButton = document.getElementById('render-family');

const width = 1280;
const height = 960;

canvas.width = width;
canvas.height = height;

const params = new URLSearchParams(window.location.search);
if (params.has('family')) {
  familyNameInput.value = params.get('family') ?? familyNameInput.value;
}
if (params.has('text')) {
  textInput.value = params.get('text') ?? textInput.value;
}
if (params.has('size')) {
  fontSizeInput.value = params.get('size') ?? fontSizeInput.value;
}
if (params.has('weight')) {
  fontWeightInput.value = params.get('weight') ?? fontWeightInput.value;
}
if (params.has('style')) {
  fontStyleInput.value = params.get('style') ?? fontStyleInput.value;
}
if (params.has('stretch')) {
  fontStretchInput.value = params.get('stretch') ?? fontStretchInput.value;
}
if (params.has('name')) {
  fontNameInput.value = params.get('name') ?? fontNameInput.value;
}

let pixelWorker;
let workerReady = false;
let familyLoaded = false;
let lastFamilyToken = '';

function setSummary(message) {
  if (summary != null) {
    summary.textContent = message;
  }
}

function currentRenderRequest() {
  const fontName = fontNameInput.value.trim();
  return {
    text: textInput.value,
    fontName: fontName === '' ? null : fontName,
    fontSize: Number.parseFloat(fontSizeInput.value || '48'),
    fontWeight: Number.parseInt(fontWeightInput.value || '400', 10),
    fontStyle: fontStyleInput.value,
    fontStretch: Number.parseFloat(fontStretchInput.value || '1'),
  };
}

function postRender() {
  if (!workerReady) {
    setSummary('worker is not ready');
    return;
  }
  if (!familyLoaded) {
    setSummary('load a font family first');
    return;
  }

  pixelWorker.postMessage({
    command: 'render',
    ...currentRenderRequest(),
  });
}

async function loadFacesFromUrls(urls) {
  const faces = [];
  for (const url of urls) {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`failed to fetch font: ${response.status} ${response.statusText}`);
    }
    const buffer = await response.arrayBuffer();
    faces.push({ buffer, source: url });
  }
  return faces;
}

async function collectFamilyFaces() {
  const faces = [];
  const files = Array.from(fontFilesInput.files ?? []);
  for (const file of files) {
    const buffer = await file.arrayBuffer();
    faces.push({ buffer, source: file.name });
  }

  const urls = fontUrlsInput.value
    .split(/\r?\n/)
    .map((value) => value.trim())
    .filter((value) => value !== '');
  const urlFaces = await loadFacesFromUrls(urls);
  faces.push(...urlFaces);

  return faces;
}

function familyToken(familyName, faces) {
  return `${familyName}::${faces.map((face) => face.source).join('|')}`;
}

async function loadSelectedFamily() {
  if (!workerReady) {
    setSummary('worker is not ready');
    return;
  }

  const familyName = familyNameInput.value.trim();
  if (familyName === '') {
    setSummary('family name is required');
    return;
  }

  const faces = await collectFamilyFaces();
  if (faces.length === 0) {
    setSummary('select at least one font file or URL');
    return;
  }

  const token = familyToken(familyName, faces);
  if (token === lastFamilyToken && familyLoaded) {
    postRender();
    return;
  }

  setSummary(`loading family ${familyName}`);
  pixelWorker.postMessage(
    {
      command: 'loadFamily',
      familyName,
      faces,
    },
    faces.map((face) => face.buffer),
  );
  lastFamilyToken = token;
}

function workerInit() {
  pixelWorker = new Worker('js/fontfamily-worker.js', { type: 'module' });

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
        setSummary('worker ready');
        break;
      case 'familyLoaded':
        familyLoaded = true;
        setSummary(`family=${data.familyName} faces=${data.faceCount}`);
        postRender();
        break;
      case 'render':
        if (data.image != null) {
          ctx.putImageData(data.image, 0, 0);
        }
        if (data.summary != null) {
          const info = data.summary;
          setSummary(
            `family=${info.familyName} faces=${info.faceCount} chars=${info.charCount} size=${info.fontSize}px weight=${info.fontWeight} style=${info.fontStyle} stretch=${info.fontStretch}`,
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
  loadSelectedFamily().catch((error) => {
    setSummary(error instanceof Error ? error.message : String(error));
  });
});

renderButton.addEventListener('click', () => {
  if (!familyLoaded) {
    loadSelectedFamily().catch((error) => {
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
