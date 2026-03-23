import * as fps from './fps.js';

const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d');
const summary = document.getElementById('summary');

const width = 1664;
const height = 1024;
canvas.width = width;
canvas.height = height;

if (window.Worker) {
  workerInit();
} else {
  alert('Worker is not support.');
}

function workerInit() {
  const pixelWorker = new Worker('js/glpyhs-worker.js', { type: 'module' });
  let fpsLoopStarted = false;

  pixelWorker.postMessage({ command: 'init', width, height });

  pixelWorker.onmessage = (ev) => {
    const data = ev.data;
    if (data.message == null) {
      return;
    }

    switch (data.message) {
      case 'init':
        if (data.image != null) {
          ctx.putImageData(data.image, 0, 0);
        }
        pixelWorker.postMessage({ command: 'run' });
        if (!fpsLoopStarted) {
          fpsLoopStarted = true;
          const draw = () => {
            fps.fps.render();
            window.requestAnimationFrame(draw);
          };
          draw();
        }
        break;
      case 'run':
        if (data.image != null) {
          ctx.putImageData(data.image, 0, 0);
        }
        if (summary != null && data.summary != null) {
          const info = data.summary;
          summary.textContent =
            `source=${info.source} glyphs=${info.count} rows=${info.rows} scale=${info.scale} usedHeight=${info.usedHeight}`;
        }
        break;
      case 'error':
        if (summary != null) {
          summary.textContent = data.error;
        }
        console.error(data.error);
        break;
      default:
        break;
    }
  };
}
