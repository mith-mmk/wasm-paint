mod utils;
pub mod paint;
use wml2::draw::*;
use wml2::error::ImgError;
use wml2::jpeg::decoder::decode as jpeg_decoder;
use std::sync::{Arc,RwLock};
use crate::paint::affine::{Affine,InterpolationAlgorithm};
use crate::paint::circle::*;
use crate::paint::fill::fill;
use crate::paint::polygram::*;
use crate::paint::rect::rect;
use crate::paint::line::line;
use crate::paint::point::point_antialias;
use crate::paint::canvas::{Canvas,Screen};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn initialization() {
    utils::set_panic_hook();
}


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

fn write_log(str: &str,_: Option<VerboseOptions>) -> Result<Option<CallbackResponse>,ImgError> {
    if web_sys::window().is_some() {
        let window = web_sys::window().unwrap();
        if window.document().is_some() {
            let document = window.document().unwrap();
            if document.get_element_by_id("wasm_verbose").is_some() {
                let elmid = document.get_element_by_id("wasm_verbose").unwrap();
                if elmid.dyn_ref::<HtmlElement>().is_some() {
                    let elm = elmid.dyn_ref::<HtmlElement>().unwrap();
                    elm.set_inner_text(str);
                    return Ok(None)
                }
            }
        }
    }
    log(str);
    Ok(None)
}

/*
 * If you use Struct in Vec,you happen bollow error for function call multiple same Sturct.
 * 
 * It is specification of Rust,
 * 
 * ex1) let result = foo(s[0] ,s[0] ); -> borrow error
 * ex2) let result = foo(s[i] ,s[j] ); -> borrow error
 * 
 * Workaround:
 * 
 * Wrap Arc(Rc) + Mutex or RWLock
 * 
 * 解説：
 * StructをVecに保存した場合、複数の同じ構造体をわたす、関数を呼び出すときにborrowエラーが発生する。
 * Rustの仕様である。Rustは同時参照を認めていないので、特定出来ない場合コンパイラがエラーを出す。
 * 
 * 例1）let result = foo(s[0] ,s[0] ); はborrowエラーになる
 * 例2）let result = foo(s[i] ,s[j] ); はborrowエラーになる i==jになる可能性があるかららしい。
 * 
 * 回避するにはArcでラップする。書き換える場合(mut)は、その前にMutexやRWLockでラップする。
 * なお、このアプリケーションはマルチスレッドではないのでLock Errorが出る可能性が低いので特にチェックしていない。
 * （先に弾いている）
 * そもそもwasm_bindgen 0.3はマルチスレッドに対応していない。
 * 
 * borrow エラー対策はいくつかあり
 * 　- 参照するだけなら、構造体を丸ごとコピーしてしまえば良い。コストが低い場合はこれで処理している（Arcでラップする方が重い）
 * 　- 処理順を入れかえるだけでも回避出来る場合がある
 * 
 * ちなみにArcから構造体を取り出す方法が面倒すぎる。Rustのサンプル読んでも解りにくい。
 * 
 */

#[wasm_bindgen]
pub struct Universe {
    canvas:  Canvas,
    on_worker: bool,
    input_buffer: Vec<u8>,
    append_canvas: Vec<Arc<RwLock<Canvas>>>,
}

#[wasm_bindgen]
impl Universe {

    #[wasm_bindgen(js_name = new)]
    pub fn new (width: u32, height: u32) -> Universe {
        let canvas = Canvas::new(width, height);
        Universe {
            canvas,
            on_worker: false,
            input_buffer: Vec::new(),
            append_canvas: Vec::new(),
        }
    }

    #[wasm_bindgen(js_name = newOnWorker)]
    pub fn new_on_worker (width: u32, height: u32) -> Universe {
        let canvas = Canvas::new(width, height);
        Universe {
            canvas,
            on_worker: true,
            input_buffer: Vec::new(),
            append_canvas: Vec::new(),
        }
    }

    pub fn append_canvas(&mut self, width: u32, height: u32) -> usize {
        let canvas = Canvas::new(width, height);
        self.append_canvas.push(Arc::new(RwLock::new(canvas)));
        self.append_canvas.len()
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

    pub fn clear_with_number(&mut self,number :i32) {
        if number > self.append_canvas.len() as i32 {
            return
        }
        if number == 0 {
            self.clear(0xcccccc);
        }
        log(&format!("{}",number));
        let number = (number as i32 - 1_i32) as u32;
        log(&format!("{}",number));
        self.append_canvas[number as usize].as_ref().write().unwrap().set_buckground_color(0);
        self.append_canvas[number as usize].as_ref().write().unwrap().clear();
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

    pub fn buffer_with_number(&mut self,number:usize) -> *const u8 {
        if number == 0 {return self.canvas.canvas()};
        let canvas = &*self.append_canvas[number - 1].write().unwrap();
        canvas.canvas()
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

    pub fn affine_test2(&mut self,canvas_in:usize,canvas_out:usize,no: usize,interpolation:usize) {
        let mut affine = Affine::new();

        match no {
            0 => {
                affine.invert_xy();
            },
            1 => {
                affine.rotate_by_dgree(30.0);
            },
            2 => {
                affine.scale(1.0/3.0,1.0/3.0);
            },
            3 => {
                affine.scale(4.5,4.5);
            },
            4 => {
                affine.translation(20.0,20.0)
            },
            5 => {
                affine.invert_xy();
                affine.rotate_by_dgree(30.0);
                affine.scale(1.0/3.0,1.0/3.0);
                affine.scale(4.5,4.5);
                affine.translation(20.0,20.0)
            }
            _ => {
                
            }

        }

        let algorithom = match interpolation {
            0 => {
                InterpolationAlgorithm::NearestNeighber
            },
            1 => {
                InterpolationAlgorithm::Bilinear
            },
            2 => {
                InterpolationAlgorithm::Bicubic
            },
            3 => {
                InterpolationAlgorithm::Lanzcos3
            }

            _ => {
                InterpolationAlgorithm::BicubicAlpha(Some(-1.0))
            }

        };

        if canvas_in == 0 {
            let output_canvas = &mut *self.append_canvas[canvas_out - 1].write().unwrap();
            affine.conversion(&self.canvas,output_canvas,algorithom);
//            affine.conversion(&self.canvas,output_canvas,InterpolationAlgorithm::Bilinear);
//            affine.conversion(&self.canvas,output_canvas,InterpolationAlgorithm::Bicubic(Some(-0.5)));
        } else if canvas_out == 0 {
            let input_canvas = & *self.append_canvas[canvas_in - 1].read().unwrap();
            affine.conversion(input_canvas,&mut self.canvas,algorithom);
        } else {
            let input_canvas = & *self.append_canvas[canvas_in - 1].read().unwrap();
            let output_canvas = &mut *self.append_canvas[canvas_out - 1].write().unwrap();
            affine.conversion(input_canvas,output_canvas,algorithom);
        }

    }
    pub fn affine_test(&mut self,canvas_in:usize,canvas_out:usize) {
        let mut affine = Affine::new();
        affine.invert_xy();
        affine.scale(5.3,5.3);
        affine.rotate_by_dgree(12.0);

        if canvas_in == 0 {
            let output_canvas = &mut *self.append_canvas[canvas_out - 1].write().unwrap();
            affine.conversion(&self.canvas,output_canvas,InterpolationAlgorithm::Lanzcos(Some(3)));
//            affine.conversion(&self.canvas,output_canvas,InterpolationAlgorithm::Bilinear);
//            affine.conversion(&self.canvas,output_canvas,InterpolationAlgorithm::Bicubic(Some(-0.5)));
        } else if canvas_out == 0 {
            let input_canvas = & *self.append_canvas[canvas_in - 1].read().unwrap();
            affine.conversion(input_canvas,&mut self.canvas,InterpolationAlgorithm::Bilinear);
        } else {
            let input_canvas = & *self.append_canvas[canvas_in - 1].read().unwrap();
            let output_canvas = &mut *self.append_canvas[canvas_out - 1].write().unwrap();
            affine.conversion(input_canvas,output_canvas,InterpolationAlgorithm::Bilinear);
        }
    }
    pub fn image_decoder(&mut self,buffer: &[u8],verbose:usize) {
        let r = crate::paint::image::draw_image(&mut self.canvas,buffer,verbose);
        match r {
            Err(error) => {
                log(&format!("{:?}",error));
            }
            _ => {},
        }
    }

    pub fn jpeg_decoder(&mut self,buffer: &[u8],verbose:usize) {
        self.jpeg_decoder_select_canvas(buffer,verbose,0);
    }

    pub fn jpeg_decoder_select_canvas(&mut self,buffer: &[u8],verbose:usize,number:usize) {
        if number > self.append_canvas.len() { return }

        if number != 0 {
            let mut option = DecodeOptions{
                debug_flag: 0,
                drawer: &mut *self.append_canvas[number - 1].write().unwrap(),
            };
            let r = jpeg_decoder(buffer, &mut option);
            match r {
                Err(error) => {
                    log(&format!("{:?}",error));
                },
                Ok(s) => {
                    if let Some(warning) = s {                        
                        log(&format!("{:?}",warning));
                    }
                }
            }
        } else {
            let canvas =&mut self.canvas;
            if !self.on_worker {
                canvas.set_verbose(write_log);
            }
            let mut option = DecodeOptions{
                debug_flag: verbose,
                drawer: canvas,
            };
        
            let r = jpeg_decoder(buffer, &mut option);
            match r {
                Err(error) => {
                    if self.on_worker {
                        log(&format!("{:?}",error));
                    } else {
                        alert(&format!("{:?}",error));
                    }
                },
                Ok(s) => {
                    if let Some(warning) = s {                        
                        log(&format!("{:?}",warning));
                    }
                }
            }
        }
    }
}
