mod utils;
pub mod paint;
use crate::paint::line::line;
use wasm_bindgen::prelude::*;
use crate::paint::point::point_antialias;
use crate::paint::canvas::Canvas;
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


pub struct rnd{
    seed:  u64,
}

impl rnd {
    pub fn new() -> Self{
        let seed: u64 =  instant::Instant::now().elapsed().as_nanos() as u64;
        Self {
            seed,
        }
    }

    pub fn get_u32(&mut self,range: u32) -> u32 {
        let mut seed = self.seed;
        // xorshift64
        seed = seed ^ (seed >> 13);
        seed = seed ^ (seed << 11);
        self.seed = seed ^ (seed >> 8);
    
        (seed % range as u64) as u32

    }
}

fn rand_u32(range: u32) -> u32 {
    ( random() * (range as f64)) as u32
}

#[wasm_bindgen]
pub struct Universe {
    canvas: Canvas,
}

#[wasm_bindgen]
impl Universe {

    pub fn new (width: u32, height: u32) -> Universe {
        let canvas = Canvas::new(width, height);
        Universe {
            canvas,
        }
    }
/* Wrapper */
    pub fn clear(&mut self,color :u32) {
        self.canvas.set_buckground_color(color);
        self.canvas.clear();
    }

    pub fn point_antialias(&mut self, x: f64, y: f64, color: u32,s: f64) {
        point_antialias(&mut self.canvas,x,y,color,s);
    }

    pub fn line(&mut self,sx :i32, sy :i32, ey: i32, ex: i32,color: u32) {
        line(&mut self.canvas,sx,sy,ex,ey,color);
    }

    pub fn output_buffer(&mut self) -> *const u8 {
        self.canvas.canvas()
    }

    pub fn width(&self) -> u32 {
        self.canvas.width()
    }

    pub fn height(&self) -> u32 {
        self.canvas.height()
    }

}
