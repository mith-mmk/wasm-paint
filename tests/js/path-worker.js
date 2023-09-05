import init,{Universe} from "../../wasm-paint/pkg/paint.js"

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
                universe.drawPath("M 100 100 Q 200 100 300 300 L 100 200 Z M 120 120 L 250 250 L 120 180 Z",0xff0000);
                postMessage({message: 'run'});
                break;
            case 'get':
                if (universe == null) return;
                universe.combine();
                img = universe.getImageData(0);
                postMessage({message: 'get', image:img});
                break;
            default:
                break;
        }
    }
}
