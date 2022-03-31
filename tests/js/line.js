import * as fps from './fps.js';

const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d');
const method = document.getElementById('flag');
let command = "";
method.addEventListener('change', (ev) => {
  command = method.value;
});


let width = 512;
let height =512;
canvas.width = width;
canvas.height = height;
let img;
const PixelWorker = new Worker('js/line-worker.js', { type: 'module' });
if (window.Worker) {
  workerInit();  
} else {
  alert("Worker is not support.")
}
console.log("init");

function workerInit() {
  PixelWorker.postMessage({command: 'init',width: width,height: height});

  PixelWorker.onmessage = (ev) => {
    const data = ev.data;
    if(data.message == null)return;
    switch (data.message) {
      case 'init':
        img = data.image;
        ctx.putImageData(img, 0, 0);
        PixelWorker.postMessage({command: 'run',method: command});
//        draw();
      break;
      case 'run': // run loop
        PixelWorker.postMessage({command: 'run',method: command});
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

setTimeout(function(){draw();},1000/60);

let i = 0;
function draw() {
  setTimeout(function(){draw();},1000/60);
  if(img == null) return;
  PixelWorker.postMessage({command: 'get'});
  PixelWorker.postMessage({command: 'run',tilde: i++ / 16 * Math.PI});
  if (i > 16) i = 0;
}
