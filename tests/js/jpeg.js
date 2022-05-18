import init,{Universe} from "../../pkg/paint.js"
const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d');
let universe;
let width = 1024;
let height =1024;
let drawed = true;
canvas.width = width;
canvas.height = height;
const reader = new FileReader();

reader.onload = (event) => {
  console.timeEnd("reader");
  console.time("buffer");
  let buffer = new Uint8Array(reader.result);
  universe.clear(0x000000);
  console.timeEnd("buffer");

  console.time("decode");
//  start_draw();
  universe.jpegDecoder(buffer,0xf9); 
  console.timeEnd("decode");
  start_draw();
//  drawed = true;
//  universe.drawCanvas(width,height);

//  img = new ImageData(buf, universe.width(), universe.height());
//  ctx.putImageData(img, 0, 0);
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
    console.time("reader");
    reader.readAsArrayBuffer(files[0]);
  }, false);

init().then((wasm) => {
    universe = new Universe(width,height);
    universe.bindCanvas("canvas");
    universe.clear(0x000000);
    universe.drawCanvas(width,height);
    
    fetch('./sample/sample01.jpg')
      .then(res => res.blob())
      .then(blob => blob.arrayBuffer())
      .then(arraybuffer => {
        let buffer = new Uint8Array(arraybuffer);      
        universe.jpegDecoder(buffer,0xf9);
        start_draw();
//        universe.drawCanvas(width,height);
      });

});



function start_draw() {
  universe.drawCanvas(width,height);
  console.log(universe.isAnimation());
  if (universe.isAnimation()) {
    setTimeout(function(){draw();},120/1000);
  }
}


function draw() {
    universe.drawCanvas(width,height);
    let wait = universe.nextFrame();
    if (wait == 0) {wait = 1.0}
    console.log(wait)
    if (universe.isAnimation()) {
      setTimeout(function(){draw();},wait*1000);
    }
}