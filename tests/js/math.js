const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d');
const summary = document.getElementById('summary');
const fontUrlInput = document.getElementById('font-url');
const fontFileInput = document.getElementById('font-file');
const textInput = document.getElementById('math-text');
const fontSizeInput = document.getElementById('font-size');
const loadButton = document.getElementById('load-font');
const renderButton = document.getElementById('render-math');
const smokeButton = document.getElementById('run-smoke');

const width = 960;
const height = 360;
const BUILD_ID = 'math-ui-20260505-1';

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

let universe;
let fontLoaded = false;
let fontSource = '';

function setSummary(message) {
  summary.textContent = message;
}

function countInk(image) {
  let count = 0;
  for (let offset = 0; offset < image.data.length; offset += 4) {
    const red = image.data[offset];
    const green = image.data[offset + 1];
    const blue = image.data[offset + 2];
    const alpha = image.data[offset + 3];
    if (alpha !== 0 && (red !== 255 || green !== 255 || blue !== 255)) {
      count += 1;
    }
  }
  return count;
}

function putFrame() {
  universe.combine();
  const image = universe.getImageData(0);
  ctx.putImageData(image, 0, 0);
  return image;
}

async function initWasm() {
  const { default: init, Universe } = await import(`../../wasm-paint/pkg/paint.js?${BUILD_ID}`);
  await init();
  universe = new Universe(width, height);
  if (typeof universe.hasFontFeature === 'function' && !universe.hasFontFeature()) {
    throw new Error('wasm-paint must be built with --features font');
  }
  if (typeof universe.drawTexLike !== 'function') {
    throw new Error('drawTexLike is not exported');
  }
  universe.clear(0xffffff);
  putFrame();
  setSummary('ready');
}

async function loadFontFromUrl(url) {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`failed to fetch font: ${response.status} ${response.statusText}`);
  }
  const buffer = await response.arrayBuffer();
  universe.loadFont(new Uint8Array(buffer));
  fontLoaded = true;
  fontSource = url;
}

async function loadFontFromFile(file) {
  const buffer = await file.arrayBuffer();
  universe.loadFont(new Uint8Array(buffer));
  fontLoaded = true;
  fontSource = file.name;
}

async function loadSelectedFont() {
  if (universe == null) {
    await initWasm();
  }

  const file = fontFileInput.files != null ? fontFileInput.files[0] : null;
  const url = fontUrlInput.value.trim();
  if (file != null) {
    setSummary(`loading ${file.name}`);
    await loadFontFromFile(file);
    setSummary(`font loaded: ${file.name}`);
    return;
  }
  if (url !== '') {
    setSummary(`loading ${url}`);
    await loadFontFromUrl(url);
    setSummary(`font loaded: ${url}`);
    return;
  }
  throw new Error('select a font file or enter a font URL');
}

async function ensureFontLoaded() {
  if (!fontLoaded) {
    await loadSelectedFont();
  }
}

async function renderMath() {
  if (universe == null) {
    await initWasm();
  }
  await ensureFontLoaded();

  const text = textInput.value;
  const fontSize = Number.parseFloat(fontSizeInput.value || '48');
  universe.clear(0xffffff);
  universe.drawTexLike(text, 32, 120, fontSize, 0x111111);
  const image = putFrame();
  const ink = countInk(image);
  setSummary(`font=${fontSource} chars=${Array.from(text).length} size=${fontSize}px ink=${ink}`);
}

async function runSmokeTest() {
  await renderMath();
  const image = universe.getImageData(0);
  const ink = countInk(image);
  if (ink === 0) {
    throw new Error('drawTexLike rendered no visible pixels');
  }
  setSummary(`${summary.textContent}\nsmoke test passed`);
}

loadButton.addEventListener('click', () => {
  loadSelectedFont().catch((error) => {
    setSummary(error instanceof Error ? error.message : String(error));
  });
});

renderButton.addEventListener('click', () => {
  renderMath().catch((error) => {
    setSummary(error instanceof Error ? error.message : String(error));
  });
});

smokeButton.addEventListener('click', () => {
  runSmokeTest().catch((error) => {
    setSummary(error instanceof Error ? error.message : String(error));
  });
});

initWasm()
  .then(() => {
    if (fontUrlInput.value.trim() !== '') {
      return runSmokeTest();
    }
    return undefined;
  })
  .catch((error) => {
    setSummary(error instanceof Error ? error.message : String(error));
  });
