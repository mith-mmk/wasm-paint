import init,{Universe} from "../../pkg/paint.js";  // Universeは要インポート wasm.Universeでは動かない

let universe;
let img;


function workerInit(width, height) {
    init()
    .then((wasm) => {
        universe = new Universe(width,height);
        universe.clear(0x000000);
        img = universe.getImageData(0);
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
                const sx = Math.random() * universe.getWidth() - 1; 
                const sy = Math.random() * universe.getHeight() - 1;
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
