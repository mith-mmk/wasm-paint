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

function workerInit() {
  const PixelWorker = new Worker('js/fill-worker.js', { type: 'module' });
  PixelWorker.postMessage({command: 'init',width: width,height: height});

  PixelWorker.onmessage = (ev) => {
    const data = ev.data;
    if(data.message == null)return;
    switch (data.message) {
      case 'init':
        img = data.image;
        ctx.putImageData(img, 0, 0);
        PixelWorker.postMessage({command: 'run'});
      break;
      case 'run': // run loop
        PixelWorker.postMessage({command: 'run'});
        break;
      case 'get':
        img = data.image;
        ctx.putImageData(img, 0, 0);
        break;
      default:
        break;
    }
  }


  setTimeout(function(){draw();},1000 / 120);

let count = 0; 

  function draw() {
    setTimeout(function(){draw();},1000 / 120);
    if(img == null) return;
    if (count++ < 300) {
      PixelWorker.postMessage({command: 'get'});
      PixelWorker.postMessage({command: 'run'});
    } else {
      count = 0;
      PixelWorker.postMessage({command: 'clear'});
      PixelWorker.postMessage({command: 'run'});
    }
  }
}