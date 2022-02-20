mod utils;
pub mod paint;
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

fn rand_u32(range: u32) -> u32 {

    return ( random() * (range as f64)) as u32;
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
