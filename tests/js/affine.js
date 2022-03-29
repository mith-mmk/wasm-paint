import init,{Universe} from "../../pkg/paint.js"

const canvas = document.getElementById('canvas');
const canvas2 = document.getElementById('canvas2');
let universe;
let drawed = true;
const width = canvas.width;
const height = canvas.width
canvas2.width = width;
canvas2.height = height;
const reader = new FileReader();
reader.onloadend = (event) => {
  let buffer = new Uint8Array(reader.result);
  universe.clear(0x000000);

  universe.jpeg_decoder(buffer,0); 
  universe.drawCanvas(width,height);
  universe.affine_test(0,1);
  universe.drawSelectCanvas2(width,height,1);
};
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

    console.log("load start");
    reader.readAsArrayBuffer(files[0]);
  }, false);

init().then((wasm) => {
    universe = new Universe(width,height);
    universe.appendCanvas(width,height);
    universe.bindCanvas("canvas");
    universe.bindCanvas2("canvas2");
    universe.clear(0x000000);
    universe.clearSelectCanvas(1);
    universe.drawCanvas(width,height);
    universe.drawSelectCanvas2(width,height,1);
    fetch('./sample/sample02.jpg')
      .then(res => res.blob())
      .then(blob => blob.arrayBuffer())
      .then(arraybuffer => {
        let buffer = new Uint8Array(arraybuffer);      
        universe.jpegDecoderSelectCanvas(buffer,0x0); 
        universe.drawCanvas(width,height);
        universe.affineTest2(0,1);
        universe.drawSelectCanvas2(width,height,1);
      });

});
function start_draw() {
  setTimeout(function(){draw();},1000 / 120);
  drawed = false;  
}

function draw() {
    if(drawed) return;
    setTimeout(function(){draw();},1000 / 120);
    universe.drawCanvas(width,height);
    universe.drawCanvas2(width,height);
  }
