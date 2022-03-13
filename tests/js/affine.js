import init,{Universe} from "../../pkg/paint.js"

const canvas = document.getElementById('canvas');
const canvas2 = document.getElementById('canvas2');
const ctx2 = canvas2.getContext('2d');
const ctx = canvas.getContext('2d');
let universe;
let buffersize;
let buf;
let buf2;
let img;
let img2;
let memory;
let drawed = true;
const width = canvas.width;
const height = canvas.width
canvas2.width = width;
canvas2.height = height;
const reader = new FileReader();
reader.onloadend = (event) => {
  let buffer = new Uint8Array(reader.result);
  universe.input_buffer_set_length(buffer.length);
  let ibuf = new Uint8Array(memory.buffer,universe.input_buffer(), buffer.length);
  ibuf.set(buffer);
  universe.clear(0x000000);

  start_draw();
  universe.jpeg_decoder(buffer,0); 
  drawed = true;
  ctx.putImageData(img, 0, 0);
  universe.affine_test(0,1);
  ctx2.putImageData(img2, 0, 0);
//  img = new ImageData(buf, universe.width(), universe.height());
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
    memory = wasm.memory; // 共有メモリーに必要
    universe = Universe.new(width,height);
    universe.append_canvas(width,height);
    buffersize = width * height * 4;
    buf  = new Uint8ClampedArray(memory.buffer,universe.output_buffer(), buffersize);
    img  = new ImageData(buf, width, height);
    universe.clear(0x000000);
    buf2 = new Uint8ClampedArray(memory.buffer,universe.buffer_with_number(1), buffersize);
    img2 = new ImageData(buf2, width, height);
    universe.clear_with_number(1);
    ctx.putImageData(img, 0, 0);
    fetch('./sample/sample02.jpg')
      .then(res => res.blob())
      .then(blob => blob.arrayBuffer())
      .then(arraybuffer => {
        let buffer = new Uint8Array(arraybuffer);      
        universe.input_buffer_set_length(buffer.length);
        let ibuf = new Uint8Array(memory.buffer,universe.input_buffer(), buffer.length);
        ibuf.set(buffer);    
        universe.jpeg_decoder_select_canvas(buffer,0x0); 
        ctx.putImageData(img, 0, 0);
        universe.affine_test(0,1);
        ctx2.putImageData(img2, 0, 0);
      });

});
function start_draw() {
  setTimeout(function(){draw();},1000 / 120);
  drawed = false;  
}

function draw() {
    if(img == null || drawed) return;
    setTimeout(function(){draw();},1000 / 120);
    ctx.putImageData(img, 0, 0);
}
