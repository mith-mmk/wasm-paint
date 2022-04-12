import init,{Universe} from "../../pkg/paint.js"

const canvas = document.getElementById('canvas');
const canvas2 = document.getElementById('canvas2');
const width = canvas.width;
const height = canvas.width
canvas2.width = width;
canvas2.height = height;


let raws_id = "option_raws";
let line_id ="line_";
let method_id = "method_";
let value1_id = "value1_";
let value2_id = "value2_";
let add_id = "add_";
let del_id = "delete_";
let current = 0;
let last = 0;

let method = document.getElementById(method_id + current);
let value1 = document.getElementById(value1_id + current);
let value2 = document.getElementById(value2_id + current);
let add = document.getElementById(add_id + current);
let del = document.getElementById(del_id + current);


const interpolation = document.getElementById('interpolation');

function getElements(current) {
  method = document.getElementById(method_id + current);
  value1 = document.getElementById(value1_id + current);
  value2 = document.getElementById(value2_id + current);
  add = document.getElementById(add_id + current);
  del = document.getElementById(del_id + current);
}

function appdendEvents() {
  method.addEventListener('change', (ev) => {
    let id = ev.target.id;
    current = Number(id.replace(method_id,""));
    getElements(current);
    setDisplay();
    affine();
  });
  
  interpolation.addEventListener('change', (ev) => {
    affine();
  });
  
  
  value1.addEventListener('change', (ev) => {
    affine();
  });
  
  value2.addEventListener('change', (ev) => {
    affine();
  });

  add.addEventListener('click', (ev) => {
    let id = ev.target.id;
    let number= Number(id.replace(add_id,""));
    addElements(number);
  });

  if (del != null) {
    del.addEventListener('click', (ev) => {
      let id = ev.target.id;
      let number= Number(id.replace(del_id,""));
      delElements(number);
    });
  }
}

function delElements(number) {
  let current = document.getElementById(line_id + number);
  current.remove();
  affine();
}

function addElements(number) {
  let prev = document.getElementById(line_id + number);
  last ++;
  let next = document.createElement('p');
  next.id = line_id + last;

  next.innerHTML =  
  `
  <select id="${method_id}${last}" class="narrow">
  <option value="-1">----</option>
  <option value="0">上下左右反転</option>
  <option value="1">回転 30°</option>
  <option value="2">縮小・拡大</option>
  <option value="3">アスペクト比</option>
  <option value="4">シフト</option>
  <option value="5">平行四辺形(X)</option>
  <option value="6">平行四辺形(Y)</option>
  </select>
  <input type="text" id="${value1_id}${last}" size="3">
  <input type="text" id="${value2_id}${last}" size="3">
  <button id="${add_id}${last}">追加</button>
  <button id="${del_id}${last}">削除</button>`;
  prev.after(next);
  getElements(last);
  appdendEvents(last);
  setDisplay();
  affine();
}

function setDisplay() {
  let m = Number(method.value);
  switch (m) {
    case 0:
      value1.style.display = 'none';
      value2.style.display = 'none';
      break;
    case 1: // rotate
      value1.style.display = 'inline';
      value2.style.display = 'none';
      value1.value = 30.0;
      break;
    case 2: // scale 1
      value1.style.display = 'inline';
      value2.style.display = 'none';
      value1.value = 1/3;
      break;
    case 3: // scale 2
      value1.style.display = 'inline';
      value2.style.display = 'inline';
      value1.value = 4.5;
      value2.value = 4.5;
      break;
    case 4: // shift
      value1.style.display = 'inline';
      value2.style.display = 'inline';
      value1.value = 20;
      value2.value = 20;
      break;
    case 5: //skey_x;
      value1.style.display = 'inline';
      value2.style.display = 'none';
      value1.value = 20;
      break;
    case 6: //skey_y;
      value1.style.display = 'inline';
      value2.style.display = 'none';
      value1.value = -50;
    break;
    default:
      value1.style.display = 'none';
      value2.style.display = 'none';
    break;
  }
}

let universe;
function affine() {
  let m = method.value;
  const interop = interpolation.value;
  let raws = document.getElementById(raws_id);
  let child_nodes = raws.childElementCount;

  universe.affineNew();

  for(let i=0; i<child_nodes; i++) {
    let id = raws.children[i].id;
    let current =  Number(id.replace(line_id,""));
    console.log(current);
    getElements(current);
    m = Number(method.value);
    console.log(m);
    switch (m) {
      case 0:
        console.log(0);
        universe.affineAdd(0,0,0);
        break;
      case 1: // rotate
        rotate_value = Number(value1.value);
        universe.affineAdd(1,rotate_value,0);

        break;
      case 2: // scale 1
        scale_value1 = Number(value1.value);
        scale_value2 = Number(value1.value);
        universe.affineAdd(2,scale_value1,scale_value2);

        break;
      case 3: // scale 2
        scale_value1 = Number(value1.value);
        scale_value2 = Number(value2.value);
        universe.affineAdd(3,scale_value1,scale_value2);

        break;
      case 4: // shift
        shift_value1 = Number(value1.value);
        shift_value2 = Number(value2.value);
        universe.affineAdd(4,shift_value1,shift_value2);

        break;
      case 5: //skey_x;
        skew_x = Number(value1.value);
        universe.affineAdd(5,skew_x);

        break;
      case 6: //skey_y;
        skew_y = Number(value1.value);
        universe.affineAdd(6,skew_y);

      break;
    }
  }

  universe.clearSelectCanvas(1);
  universe.affineRun(0,1,interop);
  drawer();
}

let rotate_value = 30;
let scale_value1 = 1.0;
let scale_value2 = 1.0;
let shift_value1 = 0;
let shift_value2 = 0;
let skew_x = 0;
let skew_y = 0;

appdendEvents(0);

const reader = new FileReader();
reader.onloadend = (event) => {
  let buffer = new Uint8Array(reader.result);
  universe.clear(0x000000);
  universe.image

  universe.imageLoader(buffer,1); 
  universe.drawCanvas(width,height);
  affine();
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
        affine();
      });

});


function drawer() {
  universe.drawCanvas(width,height);
  universe.drawCanvas2(width,height);
}