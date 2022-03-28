import init,{Universe} from "../../pkg/paint.js"
const canvas = document.getElementById('canvas');
const ctx = canvas.getContext('2d');
let universe;
let buffersize;
let buf;
let img;
let memory;
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
  universe.input_buffer_set_length(buffer.length);
  let ibuf = new Uint8Array(memory.buffer,universe.input_buffer(), buffer.length);
  ibuf.set(buffer);
  universe.clear(0x000000);
  console.timeEnd("buffer");

  console.time("decode");
  start_draw();
  universe.jpeg_decoder(buffer,0xf9); 
  console.timeEnd("decode");
  drawed = true;

//  img = new ImageData(buf, universe.width(), universe.height());
  ctx.putImageData(img, 0, 0);
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
    memory = wasm.memory; // 共有メモリーに必要
    universe = Universe.new(width,height);
    buffersize = width * height * 4;
    buf = new Uint8ClampedArray(memory.buffer,universe.output_buffer(), buffersize);
    universe.clear(0x000000);
    img = new ImageData(buf, width, height);
    ctx.putImageData(img, 0, 0);
    fetch('./sample/sample01.jpg')
      .then(res => res.blob())
      .then(blob => blob.arrayBuffer())
      .then(arraybuffer => {
        let buffer = new Uint8Array(arraybuffer);      
        universe.input_buffer_set_length(buffer.length);
        let ibuf = new Uint8Array(memory.buffer,universe.input_buffer(), buffer.length);
        ibuf.set(buffer);    
        universe.jpeg_decoder(buffer,0xf9); 
//        img = new ImageData(buf, width, height);
        console.log(ctx);
        ctx.putImageData(img, 0, 0);
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
