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

let width = 512;
let height =512;
let x1=1.0,y1=250.0,x2=255.0,y2=400.0,x3=510.0,y3=250.0;
let dy = 150;
let a = -2.0;
let mode = 0;
let p = [
    [0,400],
    [511,100],
    [0,0],
    [200,400]
];



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
                universe.clear(0x000000);
                if(p[2][0] < -511) {
                    p[1][0] = 0;
                    p[2][0] = 511;
                }
                if(p[2][1] < -500) {
                    p[1][1] = 0;
                    p[2][1] = 500;
                    p[3][0] = Math.random() * width;
                    p[3][1] = Math.random() * height;
                    p[0][0] = Math.random() * width;
                    p[0][1] = Math.random() * height;
                }
                y2 = (y1+y3)/2;
                universe.quadratic_curve(
                    x1,y1,
                    x2,y2+dy,
                    x3,y3,
                    a ,
                    0xffffff,
                );
                universe.quadratic_curve(
                    x3,y3,
                    x2,y2-dy,
                    x1,y1,
                    a ,
                    0xffffff,
                );
                universe.point_with_pen(x1,y1,0x7f7f7f);
                universe.point_with_pen(x3,y3,0x7f7f7f);
                universe.point_with_pen(x2,y2+dy,0x7f7f7f);
                universe.point_with_pen(x2,y2-dy,0x7f7f7f);

                universe.line(p[0][0],p[0][1],p[1][0],p[1][1],0x7f0000);
                universe.line(p[3][0],p[3][1],p[1][0],p[1][1],0x7f0000);
                universe.line(p[0][0],p[0][1],p[2][0],p[2][1],0x7f0000);
                universe.line(p[3][0],p[3][1],p[2][0],p[2][1],0x7f0000);

                universe.bezier_curve(
                    p[0][0],p[0][1],
                    p[1][0],p[1][1],
                    p[3][0],p[3][1],
                    0x00ff00,
                );

                universe.bezier_curve3(
                    p[0][0],p[0][1],
                    p[1][0],p[1][1],
                    p[2][0],p[2][1],
                    p[3][0],p[3][1],
                    0xff0000,
                );

                universe.point_with_pen(p[0][0],p[0][1],0x00ffff);
                universe.point_with_pen(p[1][0],p[1][1],0x00ffff);
                universe.point_with_pen(p[2][0],p[2][1],0x00ffff);
                universe.point_with_pen(p[3][0],p[3][1],0x00ffff);


                p[1][0] += 8;
                p[2][0] -= 8;
                p[1][1] += 4;
                p[2][1] -= 4;

//                a += 0.5;
                if(mode == 0) {
                    y1 += 32;
                    y3 -= 32;
                    mode = 1;
                    if(y3 < 0) {
                        mode = 1;
                    }
                } else if (mode == 1) {
                    x1 += 32;
                    x3 -= 32;
                    if (x3 < 0) {
                        mode = 2;
                    }
                } else if (mode == 2){
                    y1 += 32;
                    y3 -= 32;
                    mode = 3;
                } else if (mode == 3) {
                    x1 -= 32;
                    x3 += 32;                    
                    if (x1 < 0) {
                        mode =4;
                    }
                } else if (mode == 4) {
                    y1 += 32;
                    y3 -= 32;
                    mode = 1;
                    if(y3 < 0) {
                        x1=1.0,y1=250.0,x2=255.0,y2=400.0,x3=510.0,y3=250.0;
                    }
                }
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
