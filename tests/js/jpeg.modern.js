const canvas = document.getElementById('canvas');
const verbose = document.getElementById('wasm_verbose');

if (!(canvas instanceof HTMLCanvasElement)) {
  throw new Error('#canvas was not found');
}

if (typeof Worker === 'undefined') {
  throw new Error('Worker is not supported in this browser');
}

if (typeof canvas.transferControlToOffscreen !== 'function') {
  throw new Error('OffscreenCanvas is not supported in this browser');
}

const worker = new Worker('js/jpeg-worker.modern.js', { type: 'module' });
const offscreen = canvas.transferControlToOffscreen();

worker.postMessage({
  type: 'test',
  text: 'test'
});


worker.postMessage(
  {
    type: 'init',
    canvas: offscreen,
    width: canvas.clientWidth || canvas.width,
    height: canvas.clientHeight || canvas.height,
    devicePixelRatio: window.devicePixelRatio || 1,
  },
  [offscreen],
);


worker.onmessage = (event) => {
  const data = event.data;
  console.log(data);
  switch (data.type) {
    case 'ready':
      setStatus('ready');
      break;
    case 'decoded':
      setStatus(`decoded: ${data.width}x${data.height} (${data.decodeMs.toFixed(1)} ms)`);
      break;
    case 'cleared':
      setStatus('cleared');
      break;
    case 'error':
      setStatus(`error: ${data.detail || data.message}`);
      console.error(data.detail || data.message);
      break;
    default:
      break;
  }
};

window.addEventListener('resize', () => {
  worker.postMessage({
    type: 'resize',
    width: canvas.clientWidth || canvas.width,
    height: canvas.clientHeight || canvas.height,
    devicePixelRatio: window.devicePixelRatio || 1,
  });
});

canvas.addEventListener('dragover', (event) => {
  event.preventDefault();
});

canvas.addEventListener('drop', (event) => {
  event.preventDefault();

  const file = event.dataTransfer?.files?.[0];
  if (!file) {
    return;
  }

  setStatus(`loading: ${file.name}`);
  worker.postMessage({ type: 'decode-file', file });
});

function setStatus(message) {
  verbose.textContent = message;
}
