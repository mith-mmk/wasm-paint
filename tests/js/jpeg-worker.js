import init,{Universe} from "../../pkg/paint.js"
let universe = null;
let img;
const reader = new FileReader();

reader.onload = (event) => {
  console.time("buffer");
  let buffer = new Uint8Array(reader.result);
  postMessage({message: 'loadstart'});
  if (universe != null) {
    universe.clear(0x000000);
    console.timeEnd("buffer");
    postMessage({message: 'get', image:img});

    console.time("decode");
     universe.jpegDecoder(buffer,0xf9); 
    console.timeEnd("decode");
    postMessage({message: 'get', image:img});
    postMessage({message: 'loadend'});
  }
};

function workerInit(width, height) {
    init()
    .then(() => {
        universe = Universe.newOnWorker(width,height);
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
                postMessage({message: 'get', image:img});
                break;
            case 'loadUrl' :
                console.log(data.url);
                fetch(data.url)
                    .then(res => res.blob())
                    .then(blob => blob.arrayBuffer())
                    .then(arraybuffer => {
                        if (universe != null) {
                            postMessage({message: 'loadstart'});
                            let buffer = new Uint8Array(arraybuffer);      
                            universe.imageDecoder(buffer,0);
                            postMessage({message: 'loadend'});
                        }
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
