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

let width = 512;
let height =512;
let p = 3, q = 1, radius = 0.0;

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
                if(q==1) {
                    universe.clear(0x000000);
                }
                universe.polygram(
                            p,
                            q,
                            255,
                            255,
                            200,
                            radius ,
                            0xffffff,
                );
                if (++q >= p / 2) {
                    p ++;
                    q = 1;
                }
                if ( p > 8 ) {
                    p = 3;
                    q = 1;
                    radius += Math.PI / 3.0 ;
                }
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
