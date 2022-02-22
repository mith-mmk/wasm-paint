import init,{Universe} from "../../pkg/paint.js";  // Universeは要インポート wasm.Universeでは動かない

let universe;
let buffersize;
let buf;
let img;
let memory;

let sx = 255 ,sy = 255;
let ex = 511, ey = 511; 
let mode = 0;
let step = 1;

console.log("Hello Worker!");

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

onmessage = function(ev) {
    const data = ev.data;
    if(data.command != null) {
        switch(data.command) {
            case 'init':
                workerInit(data.width,data.height);
                break;
            case 'run':
                if (universe == null) return;
                universe.line(sx,sy,ex,ey,
                            Math.random() * 0xffffff,
                );
                if ( mode == 0 ) {
                    ex -= step;
                    if (ex < 0) {
                        ex = 0;
                        mode = 1;
                    }
                } else if (mode == 1) {
                    ey -= step;
                    if (ey < 0) {
                        mode = 2;
                    }
                } else if (mode == 2) {
                    ex += step;
                    if (ex >= 512 ) {
                        ex = 511;
                        mode = 3;
                    }

                } else if (mode == 3) {
                    ey += step;
                    if (ey >= 512 ) {
                        ey = 511;
                        mode = 0;
                    }
                } else {
                    postMessage({message: 'end'});
                    break;
                }
                postMessage({message: 'run'});
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