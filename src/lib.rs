mod utils;
pub mod paint;
pub mod img;
use crate::paint::image::draw_image;
use crate::paint::circle::*;
use crate::paint::fill::fill;
use crate::paint::polygram::*;
use crate::paint::rect::rect;
use crate::paint::line::line;
use crate::paint::point::point_antialias;
use crate::paint::canvas::Canvas;

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


pub struct Rnd{
    seed:  u64,
}

impl Rnd {
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

fn _rand_u32(range: u32) -> u32 {
    ( random() * (range as f64)) as u32
}

#[wasm_bindgen]
pub struct Universe {
    canvas:  Canvas,
    input_buffer: Vec<u8>,
}

#[wasm_bindgen]
impl Universe {

    pub fn new (width: u32, height: u32) -> Universe {
        let canvas = Canvas::new(width, height);
        Universe {
            canvas,
            input_buffer: Vec::new(),
        }
    }

    pub fn input_buffer(&mut self) -> *const u8 {
        self.input_buffer.as_ptr()
    }

    pub fn input_buffer_set_length(&mut self,size : u32) -> *const u8 {
        self.input_buffer = (0..size)
            .map(|_| {0})
            .collect();
        log(&format!("Get Buffer {}",self.input_buffer.len()));
        self.input_buffer.as_ptr()
    }

/* Wrappers */
    pub fn clear(&mut self,color :u32) {
        self.canvas.set_buckground_color(color);
        self.canvas.clear();
    }

    pub fn point_antialias(&mut self, x: f32, y: f32, color: u32,s: f32) {
        point_antialias(&mut self.canvas,x,y,color,s);
    }

    pub fn line(&mut self,sx :i32, sy :i32, ey: i32, ex: i32,color: u32) {
        line(&mut self.canvas,sx,sy,ex,ey,color);
    }

    pub fn rect(&mut self,sx :i32, sy :i32, ey: i32, ex: i32,color: u32) {
        rect(&mut self.canvas,sx,sy,ex,ey,color);
    }

    pub fn pentagram(&mut self,ox :i32, oy: i32, r: f32,tilde: f32,color: u32) {
        pentagram(&mut self.canvas,ox, oy, r,tilde,color);
    }

    pub fn polygram(&mut self,p :u32,q :u32,ox :i32, oy: i32, r: f32,tilde: f32,color: u32) {
        polygram(&mut self.canvas,p,q,ox, oy, r,tilde,color);
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

    pub fn fill(&mut self, sx: i32, sy: i32, color: u32) {
        fill(&mut self.canvas, sx, sy, color);
    }

    pub fn circle(&mut self,ox :i32, oy: i32, r: i32,color:u32){
        circle(&mut self.canvas, ox, oy, r, color);
    }

    pub fn ellipse(&mut self,ox :i32, oy: i32, rx: i32, ry: i32,tilde : f32,color:u32){
        ellipse(&mut self.canvas, ox, oy, rx, ry, tilde, color);
    }

    pub fn jpeg_decoder(&mut self,buffer: &[u8]) {
        draw_image(&mut self.canvas,buffer);
    }
}
