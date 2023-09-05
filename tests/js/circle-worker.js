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
                
                universe.ellipseAntialias(
                            Math.random() * width,
                            Math.random() * height,
                            Math.random() * width / 4 + 1,
                            Math.random() * height / 4 + 1,
                            data.tilde,
                            Math.random() * 0xffffff,
                            0xff,
                            0.3
                );
                
                universe.circleAntialias(
                    Math.random() * width,
                    Math.random() * height,
                    Math.random() * width / 4 + 1,
                    Math.random() * 0xffffff,
                    0xff,
                    0.3
                );
                
                postMessage({message: 'get', image:img});
//                postMessage({message: 'run'});
                break;
            case 'get':
                if (universe == null) return;
                universe.combine();
                postMessage({message: 'get', image:img});
                break;
            default:
                break;
        }
    }
}
