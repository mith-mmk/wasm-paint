const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d');

const width = canvas.width;
const height = canvas.height;
const PixelWorker = new Worker('js/jpeg-worker.js', { type: 'module' })


// Drag and Drop
canvas.addEventListener('dragover', (ev) => {
    ev.stopPropagation();
    ev.preventDefault();
    canvas.style.border = 'solid 10px #e1e7f0';
  }, false);

canvas.addEventListener('drop', (ev) => {
    ev.stopPropagation();
    ev.preventDefault();
    canvas.style.border = '';
    const files = ev.dataTransfer.files; 
    if (!files[0].type.match(/image\/*/)) {
      return;
    }
    if (files.length > 1) return alert('Illigal Operation.Multi Files Select.');

    PixelWorker.postMessage({command: 'loadFile',url: files[0]});

  }, false);


if (window.Worker) {
  workerInit();  
} else {
  alert("Worker is not support.")
}




function workerInit() {
  let img;
  PixelWorker.postMessage({command: 'init',width: width,height: height});

  PixelWorker.onmessage = (ev) => {
    const data = ev.data;
    if(data.message == null)return;
    switch (data.message) {
      case 'init':
        img = data.image;
        PixelWorker.postMessage({command: 'loadUrl',url: '../sample/sample01.jpg'});
      break;
      case 'loadstart':
        start_draw();
      break;
      case 'loadend':
        PixelWorker.postMessage({command: 'get'});
        end_draw();
      break;
      case 'get':
        img = data.image;
        if (img != null) ctx.putImageData(img, 0, 0);
        break;
      default:
        break;
    }
  }

  let drawed = true;
  function start_draw() {
    drawed = false;
    setTimeout(function(){draw();},1000/120);
  }

  function end_draw() {
    drawed = true;
  }
  
  let count = 1; 

  function draw() {
    if(img == null) return;
    ctx.putImageData(img, 0, 0);
    if(!drawed) {
      setTimeout(function(){draw();},1000 / 120);

    } else {
      count = 1;
    }
  }
}