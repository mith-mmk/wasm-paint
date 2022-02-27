import * as fps from './fps.js';

const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d');

let width = 512;
let height =512;
canvas.width = width;
canvas.height = height;
let img;

if (window.Worker) {
  workerInit();  
} else {
  alert("Worker is not support.")
}
console.log("init");

function workerInit() {
  const PixelWorker = new Worker('js/point-worker.js', { type: 'module' });
  PixelWorker.postMessage({command: 'init',width: width,height: height});

  PixelWorker.onmessage = (ev) => {
    const data = ev.data;
    if(data.message == null)return;
    switch (data.message) {
      case 'init':
        img = data.image;
        ctx.putImageData(img, 0, 0);
        PixelWorker.postMessage({command: 'run'});
        function draw() { // draw loop
          fps.fps.render();          
          PixelWorker.postMessage({command: 'get'});
          window.requestAnimationFrame(draw);
        }
        draw();
      break;
      case 'run': // run loop
        PixelWorker.postMessage({command: 'run'});
        break;
      case 'get':
        img = data.image;
        ctx.putImageData(img, 0, 0);
        break;
        case 'end':

          break;
        default:
        break;
    }
  }
}

