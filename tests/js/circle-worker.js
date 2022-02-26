import init,{Universe} from "../../pkg/paint.js";  // Universeは要インポート wasm.Universeでは動かない

let universe;
let buffersize;
let buf;
let img;
let memory;


function workerInit(width, height) {
    init()
    .then((wasm) => {
        memory = wasm.memory; // 共有メモリーに必要
        universe = Universe.new(width,height);
        buffersize = width * height * 4;
        buf = new Uint8ClampedArray(memory.buffer,universe.output_buffer(), buffersize);
        universe.clear(0x000000);
        img = new ImageData(buf, width, height);
        postMessage({message: 'init', image: img});
    });
}

let width,height;

onmessage = function(ev) {
    const data = ev.data;
    if(data.command != null) {
        switch(data.command) {
            case 'init':
                workerInit(data.width,data.height);
                width = data.width;
                height = data.height;
                break;
            case 'run':
                if (universe == null) return;
                let tilde =0;
                if (data.tilde) {
                    tilde = data.tilde;
                }
                universe.ellipse(
                            Math.random() * width,
                            Math.random() * height,
                            Math.random() * width / 4 + 1,
                            Math.random() * height / 4 + 1,
                            data.tilde,
                            Math.random() * 0xffffff
                );
//                postMessage({message: 'run'});
                break;
            case 'get':
                if (universe == null) return;
                img = new ImageData(buf, universe.width(), universe.height());
                postMessage({message: 'get', image:img});
                break;
            default:
                break;
        }
    }
}
