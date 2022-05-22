import init,{Universe} from "../../pkg/paint.js"

const canvas = document.getElementById('canvas');
const canvas2 = document.getElementById('canvas2');
const width = canvas.width;
const height = canvas.width
canvas2.width = width;
canvas2.height = height;

const filter_name = document.getElementById('filter_name');

  
filter_name.addEventListener('change', (ev) => {
    filter();
});

let universe;
function filter() {
  const filter = filter_name.value;
  universe.clearSelectCanvas(1);
  universe.filter(0,1,filter);
  drawer();
}

const reader = new FileReader();
reader.onloadend = (event) => {
  let buffer = new Uint8Array(reader.result);
  universe.clear(0x000000);
  universe.image

  universe.imageLoader(buffer,1); 
  universe.drawCanvas(width,height);
  filter();
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
        universe.imageLoader(buffer,1); 
        universe.drawCanvas(width,height);
        filter();
      });

});


function drawer() {
  universe.drawCanvas(width,height);
  universe.drawCanvas2(width,height);
}