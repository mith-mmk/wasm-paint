import init,{Universe} from "../../wasm-paint/pkg/paint.js"

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

function filters() {
  //const filters = ///
  universe.clearSelectCanvas(1);
  //universe.filters(0,1,filters);
  drawer();
}

const reader = new FileReader();
reader.onloadend = (event) => {
  let buffer = new Uint8Array(reader.result);
  universe.clear(0x000000);
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
  if (files.length > 1) return alert('Illigal Operation.Multi Files Select.');

  console.log("load start");
  reader.readAsArrayBuffer(files[0]);
}, false);

const loadFile = document.getElementById('upload');
loadFile.addEventListener('change', (ev) =>{
  const files = loadFile.files;
  if (files.length > 1) return alert('Illigal Operation.Multi Files Select.');
  console.log("load start");
  console.time("reader");
    reader.readAsArrayBuffer(files[0]);
});


const save = document.getElementById('saver');
save.addEventListener('click', (ev) => {
  const data = universe.imageEncoderSelectCanvas(1, 0);
  if (!(data instanceof Uint8Array) || data.length <= 1) {
    alert('保存データの作成に失敗しました');
    return;
  }

  const blob = new Blob([data], { type: 'image/jpeg' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = 'filter-output.jpg';
  document.body.appendChild(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
});

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
