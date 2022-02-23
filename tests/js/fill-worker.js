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

onmessage = function(ev) {
    const data = ev.data;
    if(data.command != null) {
        switch(data.command) {
            case 'init':
                workerInit(data.width,data.height);
                break;
            case 'run':
                const p = Math.random() * 8 + 2;
                const sx = Math.random() * universe.width() - 1; 
                const sy = Math.random() * universe.height() - 1;
                const r = Math.random() * 200 + 2;

                universe.polygram(
                    p,
                    1,
                    sx,
                    sy,
                    r,
                    0 ,
                    Math.random() * 0xffffff,
                );
                universe.fill (sx ,sy , Math.random() * 0xffffff );
                break;
            case 'get':
                if (universe == null) return;
                img = new ImageData(buf, universe.width(), universe.height());
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
