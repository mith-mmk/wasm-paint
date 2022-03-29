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
                if (universe == null) return;
                universe.pointAntialias(
                            Math.random() * universe.getWidth(), 
                            Math.random() * universe.getHeight(),
                            Math.random() * 0xffffff,
                            1.0
                );
                postMessage({message: 'run'});
                break;
            case 'get':
                if (universe == null) return;
                postMessage({message: 'get', image:img});
                break;
            default:
                break;
        }
    }
}
