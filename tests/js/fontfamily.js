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
const FONT_CHUNK_SIZE = 256 * 1024;
const BUILD_ID =
  new URL(import.meta.url).searchParams.get('v') ?? 'fontfamily-20260329-2';

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

function listedFontUrls() {
  return fontUrlsInput.value
    .split(/\r?\n/)
    .map((value) => value.trim())
    .filter((value) => value !== '');
}

async function collectFamilyFaceSources() {
  const faces = [];
  const files = Array.from(fontFilesInput.files ?? []);
  for (const [index, file] of files.entries()) {
    faces.push({
      faceId: `file-${index}-${file.name}`,
      source: file.name,
      kind: 'file',
      file,
    });
  }

  const urls = listedFontUrls();
  for (const [index, url] of urls.entries()) {
    faces.push({
      faceId: `url-${index}`,
      source: url,
      kind: 'url',
      url,
    });
  }

  return faces;
}

function familyToken(familyName, faces) {
  return `${familyName}::${faces.map((face) => face.source).join('|')}`;
}

function inferFaceDescriptor(face) {
  const label = face.source.split(/[\\/]/).pop() ?? face.source;
  const baseName = label.replace(/\.[^.]+$/, '');
  const normalized = baseName.toLowerCase();

  let fontWeight = 400;
  if (/(black|heavy)/.test(normalized)) {
    fontWeight = 900;
  } else if (/(extra[-_ ]?bold|ultra[-_ ]?bold)/.test(normalized)) {
    fontWeight = 800;
  } else if (/(semi[-_ ]?bold|demi[-_ ]?bold)/.test(normalized)) {
    fontWeight = 600;
  } else if (/bold/.test(normalized)) {
    fontWeight = 700;
  } else if (/medium/.test(normalized)) {
    fontWeight = 500;
  } else if (/(extra[-_ ]?light|ultra[-_ ]?light)/.test(normalized)) {
    fontWeight = 200;
  } else if (/(light|thin|hair)/.test(normalized)) {
    fontWeight = 300;
  }

  let fontStyle = 'normal';
  if (/oblique/.test(normalized)) {
    fontStyle = 'oblique';
  } else if (/italic/.test(normalized)) {
    fontStyle = 'italic';
  }

  let fontStretch = 1;
  if (/(condensed|narrow)/.test(normalized)) {
    fontStretch = 0.875;
  } else if (/(expanded|extended|wide)/.test(normalized)) {
    fontStretch = 1.125;
  }

  return {
    fontName: baseName,
    fontWeight,
    fontStyle,
    fontStretch,
  };
}

function transferChunk(message, buffer) {
  pixelWorker.postMessage(
    {
      ...message,
      buffer,
    },
    [buffer],
  );
}

async function fetchContentLength(url) {
  const head = await fetch(url, { method: 'HEAD' });
  if (!head.ok) {
    return null;
  }
  const value = head.headers.get('content-length');
  if (value == null) {
    return null;
  }
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
}

async function streamUrlFace(face) {
  const response = await fetch(face.url);
  if (!response.ok) {
    throw new Error(`failed to fetch font: ${response.status} ${response.statusText}`);
  }

  let totalSize = Number.parseInt(response.headers.get('content-length') ?? '', 10);
  if (!Number.isFinite(totalSize) || totalSize <= 0) {
    totalSize = await fetchContentLength(face.url);
  }
  if (!Number.isFinite(totalSize) || totalSize <= 0) {
    throw new Error(`content-length is required for chunked font loading: ${face.url}`);
  }
  if (response.body == null) {
    throw new Error(`streaming response body is unavailable: ${face.url}`);
  }

  pixelWorker.postMessage({
    command: 'beginFamilyFace',
    faceId: face.faceId,
    totalSize,
    ...inferFaceDescriptor(face),
  });

  const reader = response.body.getReader();
  let offset = 0;
  while (true) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    const chunk = value.buffer.slice(value.byteOffset, value.byteOffset + value.byteLength);
    transferChunk(
      {
        command: 'appendFamilyChunk',
        faceId: face.faceId,
        offset,
      },
      chunk,
    );
    offset += value.byteLength;
  }
  if (offset !== totalSize) {
    throw new Error(`font download ended early for ${face.source}: ${offset}/${totalSize} bytes`);
  }

  pixelWorker.postMessage({
    command: 'finalizeFamilyFace',
    faceId: face.faceId,
  });
}

async function streamFileFace(face) {
  const totalSize = face.file.size;
  pixelWorker.postMessage({
    command: 'beginFamilyFace',
    faceId: face.faceId,
    totalSize,
    ...inferFaceDescriptor(face),
  });

  let offset = 0;
  while (offset < totalSize) {
    const end = Math.min(offset + FONT_CHUNK_SIZE, totalSize);
    const chunk = await face.file.slice(offset, end).arrayBuffer();
    transferChunk(
      {
        command: 'appendFamilyChunk',
        faceId: face.faceId,
        offset,
      },
      chunk,
    );
    offset = end;
  }

  pixelWorker.postMessage({
    command: 'finalizeFamilyFace',
    faceId: face.faceId,
  });
}

async function streamFamilyFace(face) {
  if (face.kind === 'file') {
    await streamFileFace(face);
    return;
  }
  await streamUrlFace(face);
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

  const faces = await collectFamilyFaceSources();
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
  pixelWorker.postMessage({
    command: 'loadFamily',
    familyName,
  });
  for (const face of faces) {
    setSummary(`loading ${face.source}`);
    await streamFamilyFace(face);
  }
  pixelWorker.postMessage({ command: 'finishFamily' });
  lastFamilyToken = token;
}

function workerInit() {
  pixelWorker = new Worker(`js/fontfamily-worker.js?v=${encodeURIComponent(BUILD_ID)}`, {
    type: 'module',
  });

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
