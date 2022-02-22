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
  const PixelWorker = new Worker('js/rect-worker.js', { type: 'module' });
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
        console.log('get');
        img = data.image;
        ctx.putImageData(img, 0, 0);
        break;
      default:
        break;
    }
  }

  setTimeout(function(){draw();},100);

  function draw() {
    setTimeout(function(){draw();},100);
    if(img == null) return;
    PixelWorker.postMessage({command: 'get'});
  }
}