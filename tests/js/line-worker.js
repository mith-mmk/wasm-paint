import init,{Universe} from "../../pkg/paint.js";  // Universeは要インポート wasm.Universeでは動かない

let universe;
let img;

let sx = 255 ,sy = 255;
let ex = 511, ey = 511; 
let mode = 0;
let step = 1;


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
                if (data.method != null && data.method == "antialias") {
                    universe.lineAntialias(sx,sy,ex,ey,
                        Math.random() * 0xffffff,
                    );
                } else {
                    universe.line(sx,sy,ex,ey,
                        Math.random() * 0xffffff,
                    );
                }
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
//                img = new ImageData(buf, universe.width(), universe.height());
                postMessage({message: 'get', image:img});
                break;
            default:
                break;
        }
    }
}
