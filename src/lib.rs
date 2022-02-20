mod utils;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn rand_u32(range: u32) -> u32 {
    return ( random() * (range as f64)) as u32;
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    input_buffer: Vec<u8>,
    output_buffer: Vec<u8>,
}

const GrayscaleWeights: [(f64,f64,f64);6] = [
    (0.299_f64, 0.587_f64, 0.114_f64),   //JPEG,BT.601
    (0.2126_f64, 0.7152_f64, 0.0722_f64), // BT.709
    (0.3333333_f64,0.3333334_f64,0.3333333_f64), // Avarage
    (1.0_f64,0.0_f64,0.0_f64), // Red
    (0.0_f64,1.0_f64,0.0_f64), // Green
    (0.0_f64,0.0_f64,1.0_f64), // Blue
];

#[wasm_bindgen]
impl Universe {

    pub fn new (width: u32, height: u32) -> Universe {
        let buffersize = width * height * 4;
        let input_buffer = (0..buffersize)
                .map(|_| {0})
                .collect();
        let output_buffer = (0..buffersize)
                .map(|_| {0})
                .collect();
        Universe {
            width,
            height,
            input_buffer,
            output_buffer,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }


    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn input_buffer(&self) -> *const u8 {
        self.input_buffer.as_ptr()
    }

    pub fn output_buffer(&self) -> *const u8 {
        self.output_buffer.as_ptr()
    }

    pub fn clear(&mut self, color: u32){
        self.fillrect(0, 0, self.width, self.height, color);
    }

    fn color_taple(&mut self, color: u32) -> (u8,u8,u8,u8) {
        let alpha: u8 = ((color  >> 24) & 0xff)  as u8; 
        let red: u8 = ((color  >> 16) & 0xff)  as u8; 
        let green: u8  = ((color >> 8) & 0xff) as u8; 
        let blue: u8 = ((color >> 0) & 0xff) as u8; 
        (red,green,blue,alpha)
    }
    
    pub fn fillrect(&mut self,startx :u32, starty :u32, width: u32, height: u32,color: u32) {
        let endx0 = startx + width;
        let endy0 = starty + height;
        let buf = &mut self.output_buffer;
        // Color model u32 LE (ARGB)  -> u8 BGRA
        let red: u8 = ((color  >> 16) & 0xff)  as u8; 
        let green: u8  = ((color >> 8) & 0xff) as u8; 
        let blue: u8 = ((color >> 0) & 0xff) as u8; 
        let alpha: u8 = 0xff;

        for y in starty..endy0 {
            let offset = y * width * 4;
            for x  in startx..endx0 {
                let pos :usize= (offset + (x * 4)) as usize;

                buf[pos] = red;
                buf[pos + 1] = green;
                buf[pos + 2] = blue;
                buf[pos + 3] = alpha;
            }
        }

    }

    fn _point (&mut self, x: i64, y: i64, red :u8, green :u8, blue :u8, alpha :u8) {
        if x < 0 || y < 0 || x >= self.width as i64 || y >= self.height() as i64 || alpha == 0 {
            return;
        }
        let buf = &mut self.output_buffer;
        let pos :usize= (y as u32 * self.width * 4 + (x as u32 * 4)) as usize;
        let a = alpha as f32 / 255_f32;
        let r = (buf[pos] as f32 * (1.0 - a) + red as f32 * a) as u8;
        let g = (buf[pos + 1] as f32 * (1.0 - a) + green as f32 * a) as u8;
        let b = (buf[pos + 2]  as f32* (1.0 - a) + blue as f32 * a) as u8;

        buf[pos] = r;
        buf[pos + 1] = g;
        buf[pos + 2] = b;
        buf[pos + 3] = 0xff;
    }

    pub fn point (&mut self, x: i32, y: i32, color: u32) {
        let (red, green, blue, _) = self.color_taple(color);
        self._point(x as i64, y as i64, red, green, blue, 0xff);
    }

    pub fn point_antialias(&mut self, x: f64, y: f64, color: u32,s: f64) {
        if s <= 0.0 {return};
        let (red, green, blue, _) = self.color_taple(color);
        let alpha = 1.0_f64;
                                                    // x = 4.5 y = 5.0 s = 4.5
        let sx :f64 = (x + 0.5 - s / 2.0).floor();  // sx : 2
        let sy :f64 = (y + 0.5 - s / 2.0).floor();  // sy : 3
        let ex :f64 = (x + 0.5 + s / 2.0).ceil();   // ex : 8
        let ey :f64 = (y + 0.5 + s / 2.0).ceil();   // ey : 8

        let dx0 = 1.0_f64 - ((x + 0.5 - s / 2.0) - sx);  // dx0 = 0.25
        let dy0 = 1.0_f64 - ((y + 0.5 - s / 2.0) - sy);   // dy0 = 0.75
        let dx1 = 1.0_f64 - (ex - (x + 0.5 + s / 2.0));   // dx1 = 0.25
        let dy1 = 1.0_f64 - (ey - (y+ 0.5 + s / 2.0));  // dy1 = 0.75
        /*
          00        0y       10
         (dx0,dy0)   (1 ,dy0)  (dx1, dy0)
                  +----+
        (dx0, 1)   |(1,1)| (dx1 ,1)
        x0        |     | x1
                  +----+
                 (1, dy0) 1y  (dx1, dy1) 11
        (dx0,dy1) 01
        */

        let weight00 = (dx0 * dy0 * alpha * 255.0_f64).round() as u8;
        let weight01 = (dx0 * dy1 * alpha * 255.0_f64).round() as u8;
        let weight10 = (dx1 * dy0 * alpha * 255.0_f64).round() as u8;
        let weight11 = (dx1 * dy1 * alpha * 255.0_f64).round() as u8;
        let weight0y = (dy0 * alpha * 255.0_f64).round() as u8;
        let weight1y = (dy1 * alpha * 255.0_f64).round() as u8;
        let weightx0 = (dx0 * alpha * 255.0_f64).round() as u8;
        let weightx1 = (dx1 * alpha * 255.0_f64).round() as u8;

        let px = sx as i64;
        let py = sy as i64;
        let rx = ex as i64;
        let ry = ey as i64;
/*
        log (&format!("{} {} {} {}, {} {} {} {}",px,py,rx,ry,dx0,dy0,dx1,dy1));
        log (&format!("{} {} {}\n{} {} {}\n {} {} {}\n",
            weight00,weight0y,weight01,weightx0,255,weightx1,weight10,weight1y,weight11));
*/
        self._point(px, py, red, green, blue, weight00);
        self._point(rx, py, red, green, blue, weight10);
        self._point(px, ry, red, green, blue, weight01);
        self._point(rx ,ry, red, green, blue, weight11);

        if py + 1 < ry {
            for qy in py + 1 .. ry {
                self._point(px, qy, red, green, blue, weightx0);
            }
            for qy in py + 1 .. ry {
                self._point(rx, qy, red, green, blue, weightx1);
            }
        }


        if px + 1 < rx {
            for qx in px + 1 .. rx {
                self._point(qx, py, red, green, blue, weight0y);
                if py + 1 < ry {
                    for qy in py + 1 .. ry {
                        self._point(qx, qy, red, green, blue, 0xff);
                    }
                }
                self._point(qx, ry, red, green, blue, weight1y);
            }
        }
    }


    pub fn to_grayscale(&mut self, t: usize) {
        let height = self.height;
        let width = self.width;
        let ibuf = &self.input_buffer;
        let buf = &mut self.output_buffer;
        let (wred, wgreen, wblue)  = GrayscaleWeights[t];
        for y in 0..height {
            let offset = y * width * 4;
            for x  in 0..width {
                let pos = (offset + (x * 4)) as usize;
                let blue = ibuf[pos + 2] as f64;
                let green  = ibuf[pos + 1] as f64;
                let red = ibuf[pos] as f64;

                let gray =  (wred * red + wgreen * green  + wblue * blue).round() as u8;
                buf[pos] = gray;     // Red
                buf[pos + 1] = gray; // Green
                buf[pos + 2] = gray; // Blue
                buf[pos + 3] = 0xff; // alpha
            }
        }
    }
}
