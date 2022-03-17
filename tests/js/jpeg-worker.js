import init,{Universe} from "../../pkg/paint.js"
let universe;
let buffersize;
let buf;
let img;
let memory;
const reader = new FileReader();

reader.onload = (event) => {
  console.time("buffer");
  let buffer = new Uint8Array(reader.result);
  universe.input_buffer_set_length(buffer.length);
  let ibuf = new Uint8Array(memory.buffer,universe.input_buffer(), buffer.length);
  ibuf.set(buffer);
  postMessage({message: 'loadstart'});
  universe.clear(0x000000);
  console.timeEnd("buffer");
  postMessage({message: 'get', image:img});

  console.time("decode");
  universe.jpeg_decoder(buffer,0xf9); 
  console.timeEnd("decode");
  postMessage({message: 'get', image:img});
  postMessage({message: 'loadend'});
//  img = new ImageData(buf, universe.width(), universe.height());
//  ctx.putImageData(img, 0, 0);
};

function workerInit(width, height) {
    init()
    .then((wasm) => {
        memory = wasm.memory; // 共有メモリーに必要
        universe = Universe.newOnWorker(width,height);
        buffersize = width * height * 4;
        buf = new Uint8ClampedArray(memory.buffer,universe.output_buffer(), buffersize);
        universe.clear(0x000000);
        img = new ImageData(buf, width, height);
        postMessage({message: 'init', image: img});
    });
}

onmessage = function(ev) {
    const data = ev.data;
    if(data.command != null) {
        switch(data.command) {
            case 'init':
                workerInit(data.width,data.height);
                postMessage({message: 'get', image:img});
                break;
            case 'loadUrl' :
                console.log(data.url);
                fetch(data.url)
                    .then(res => res.blob())
                    .then(blob => blob.arrayBuffer())
                    .then(arraybuffer => {
                        postMessage({message: 'loadstart'});
                        let buffer = new Uint8Array(arraybuffer);      
                        universe.input_buffer_set_length(buffer.length);
                        let ibuf = new Uint8Array(memory.buffer,universe.input_buffer(), buffer.length);
                        ibuf.set(buffer);    
                        universe.jpeg_decoder(buffer,0);
                        postMessage({message: 'loadend'});
                });

                break;
            case 'loadFile' :
                reader.readAsArrayBuffer(data.url);
                break;
            case 'get':
                if (universe == null) return;
//                img = new ImageData(buf, universe.width(), universe.height());
                postMessage({message: 'get', image:img});
                break;
            case 'clear':
                if (universe == null) return;
                universe.clear(0);
                break;
            default:
                break;
        }
    }
}
