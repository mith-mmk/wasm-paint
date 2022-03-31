mod utils;
pub mod paint;

type Error = Box<dyn std::error::Error>;

use crate::paint::layer::Layer;
use crate::paint::line::line_antialias;
use web_sys::HtmlElement;
use std::sync::{Arc,RwLock};
use web_sys::ImageData;
use web_sys::CanvasRenderingContext2d;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wml2::draw::*;
use crate::paint::line::line_pen;
use crate::paint::affine::{Affine,InterpolationAlgorithm};
use crate::paint::circle::*;
use crate::paint::fill::fill;
use crate::paint::polygram::*;
use crate::paint::rect::rect;
use crate::paint::line::line;
use crate::paint::point::point_antialias;
use crate::paint::canvas::{Canvas,Screen};
use crate::paint::pen::*;
use crate::paint::spline::*;


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn initialization() {
    utils::set_panic_hook();
}

pub(crate) fn write_log(str: &str,_: Option<VerboseOptions>) -> Result<Option<CallbackResponse>,Error> {
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
    current: String,
    on_worker: bool,
//    input_buffer: Vec<u8>,
    append_canvas: Vec<Arc<RwLock<Canvas>>>,
//    #[cfg(target="web")]
    ctx: Option<CanvasRenderingContext2d>,
    ctx2: Option<CanvasRenderingContext2d>,
}

#[wasm_bindgen]
impl Universe {

    #[wasm_bindgen(constructor)]
    pub fn new (width: u32, height: u32) -> Universe {
        let mut canvas = Canvas::new(width, height);
        let pen = Pen::new(9, 9, vec![
            0x00,0x00,0x00,0x3f,0x7f,0x3f,0x00,0x00,0x00,
            0x00,0x00,0x00,0x7f,0xff,0x7f,0x00,0x00,0x00,
            0x00,0x00,0x00,0x7f,0xff,0x7f,0x00,0x00,0x00,
            0x3f,0x7f,0x7f,0x7f,0x7f,0xff,0x7f,0x7f,0x3f,
            0x7f,0xff,0xff,0xff,0xff,0xff,0xff,0xff,0x7f,
            0x3f,0x7f,0x7f,0x7f,0x7f,0x7f,0x7f,0x7f,0x3f,
            0x00,0x00,0x00,0x00,0x7f,0x7f,0x00,0x00,0x00,
            0x00,0x00,0x00,0x00,0x7f,0x7f,0x00,0x00,0x00,
            0x00,0x00,0x00,0x00,0x7f,0x3f,0x00,0x00,0x00
        ]);

        canvas.set_pen(pen);
        let _ = canvas.add_layer("main".to_string(), width, height, 0, 0);
        Universe {
            canvas,
            current: "main".to_string(),
            on_worker: false,
//            input_buffer: Vec::new(),
            append_canvas: Vec::new(),
 //           #[cfg(target="web")]
            ctx : None,
            ctx2: None,
        }
    }

    #[wasm_bindgen(js_name = newOnWorker)]
    pub fn new_on_worker (width: u32, height: u32) -> Universe {
        let mut universe = Self::new(width,height);
        universe.on_worker = true;
        universe
    }

    #[wasm_bindgen(js_name = appendCanvas)]
    pub fn append_canvas(&mut self, width: u32, height: u32) -> usize {
        let canvas = Canvas::new(width, height);
        self.append_canvas.push(Arc::new(RwLock::new(canvas)));
        self.append_canvas.len()
    }

/* Wrappers */
    fn layer_as_mut(&mut self) -> &mut Layer {
        self.canvas.layer_as_mut(self.current.to_string()).unwrap()    
    }
    
    #[wasm_bindgen(js_name = setEnable)]
    pub fn set_enable(&mut self,label:String) {
        let _ = self.canvas.set_enable(label);
    }

    #[wasm_bindgen(js_name = setLayerAlpha)]
    pub fn set_layer_alpha(&mut self,label:String,alpha:u8) {
        let _ = self.canvas.set_layer_alpha(label,alpha);
    }

    #[wasm_bindgen(js_name = getLayerAlpha)]
    pub fn layer_alpha(&mut self,label:String) -> u8 {
        let r = self.canvas.get_layer_alpha(label);
        if let Ok(alpha) = r {
            if let Some(alpha) = alpha {
                alpha
            } else {
                0xff
            }
        } else {
            0
        }
    }

    #[wasm_bindgen(js_name = setDisable)]
    pub fn set_disable(&mut self,label:String) {
        let _ = self.canvas.set_disable(label);
    }

    #[wasm_bindgen(js_name = getEnable)]
    pub fn enable(&self,label:String) -> bool {
        let r = self.canvas.enable(label);
        if let Ok(enable) = r {
            enable
        } else {
            false
        }
    }

    #[wasm_bindgen(js_name = setCurrentLayer)]
    pub fn set_current(&mut self,label:String) -> String{
        let prev = &self.current.clone();
        self.current = label;
        prev.to_string()
    }

    #[wasm_bindgen(js_name = setPos)]
    pub fn set_pos(&mut self,label:String,x: i32,y: i32) {
        self.canvas.set_pos(label,x,y);
    }

    pub fn clear(&mut self,color :u32) {
        self.canvas.set_buckground_color(color);
        self.canvas.clear();
    }


    #[wasm_bindgen(js_name = clearLayer)]
    pub fn clear_layer(&mut self,label :String) {
        let _ = self.canvas.clear_layer(label);
    }

    #[wasm_bindgen(js_name = layersLength)]
    pub fn layers_len(&self) -> usize {
        self.canvas.layers_len()
    }

    #[wasm_bindgen(js_name = clearSelectCanvas)]
    pub fn clear_with_number(&mut self,number :i32) {
        if number > self.append_canvas.len() as i32 {
            return
        }
        if number == 0 {
            self.clear(0xcccccc);
        }
        let number = (number as i32 - 1_i32) as u32;
        self.append_canvas[number as usize].as_ref().write().unwrap().set_buckground_color(0);
        self.append_canvas[number as usize].as_ref().write().unwrap().clear();
    }

    #[wasm_bindgen(js_name = pointAntialias)]
    pub fn point_antialias(&mut self, x: f32, y: f32, color: u32,alpha: u8) {
        point_antialias(self.layer_as_mut(),x,y,color,alpha,1.0);
    }

    #[wasm_bindgen(js_name = pointWithPen)]
    pub fn point_with_pen(&mut self, x: f32, y: f32, color: u32) {
//        point_pen(&mut self.canvas,x as i32,y as i32,color);
        let pen = self.canvas.pen();
        point_with_pen(self.layer_as_mut(),x as i32,y as i32,color,&pen);
    }

    pub fn line(&mut self,sx :i32, sy :i32, ex: i32, ey: i32,color: u32) {
        line(self.layer_as_mut(),sx,sy,ex,ey,color);
    }

    #[wasm_bindgen(js_name = lineAntialias)]
    pub fn line_antialias(&mut self,sx :f32, sy :f32, ex: f32, ey: f32,color: u32) {
        line_antialias(self.layer_as_mut(),sx,sy,ex,ey,color,0xff,1.0);
    }

    #[wasm_bindgen(js_name = lineWithPen)]
    pub fn line_with_pen(&mut self,sx :i32, sy :i32, ey: i32, ex: i32,color: u32) {
        line_pen(&mut self.canvas,sx,sy,ex,ey,color);
    }

    pub fn rect(&mut self,sx :i32, sy :i32, ey: i32, ex: i32,color: u32) {
        rect(self.layer_as_mut(),sx,sy,ex,ey,color);
    }

    pub fn pentagram(&mut self,ox :i32, oy: i32, r: f32,tilde: f32,color: u32) {
        pentagram(self.layer_as_mut(),ox, oy, r,tilde,color);
    }

    pub fn polygram(&mut self,p :u32,q :u32,ox :i32, oy: i32, r: f32,tilde: f32,color: u32) {
        polygram(self.layer_as_mut(),p,q,ox, oy, r,tilde,color);
    }


    #[wasm_bindgen(js_name = getBuffer)]
    pub fn output_buffer(&mut self) -> *const u8 {
        self.canvas.canvas()
    }

    #[wasm_bindgen(js_name = getBufferSelectCanvas)]
    pub fn buffer_with_number(&mut self,number:usize) -> *const u8 {
        if number == 0 {return self.canvas.canvas()};
        let canvas = &*self.append_canvas[number - 1].write().unwrap();
        canvas.canvas()
    }

    #[wasm_bindgen(js_name = addLayer)]
    pub fn add_layer(&mut self,label:String,width:u32,height:u32)  {
        let ret = self.canvas.add_layer(label, width, height, 0, 0);
        match ret {
            Err(err) => {
                log(&format!("{:?}",err));
            },
            _ => {

            },
        }
    }

    #[wasm_bindgen(js_name = getWidth)]
    pub fn width(&self) -> u32 {
        self.canvas.width().clone()
    }

    #[wasm_bindgen(js_name = getHeight)]
    pub fn height(&self) -> u32 {
        self.canvas.height().clone()
    }

    pub fn fill(&mut self, sx: i32, sy: i32, color: u32) {
        fill(self.layer_as_mut(), sx, sy, color);
    }

    pub fn circle(&mut self,ox :i32, oy: i32, r: i32,color:u32){
        circle(self.layer_as_mut(), ox, oy, r, color);
    }

    pub fn ellipse(&mut self,ox :i32, oy: i32, rx: i32, ry: i32,tilde : f32,color:u32){
        ellipse(self.layer_as_mut(), ox, oy, rx, ry, tilde, color);
    }

    #[wasm_bindgen(js_name = quadraticCurve)]
    pub fn quadratic_curve(&mut self,x1: f32,y1: f32,x2: f32,y2: f32,x3:f32, y3:f32,a:f32,color: u32) {
        let p :[(f32,f32);3]= [(x1,y1),(x2,y2),(x3,y3)];
        quadratic_curve(self.layer_as_mut(),p.to_vec(), a, color);
    }

    #[wasm_bindgen(js_name = quadraticCurveAntialias)]
    pub fn quadratic_curve_antialias(&mut self,x1: f32,y1: f32,x2: f32,y2: f32,x3:f32, y3:f32,a:f32,color: u32,size:f32) {
        let p :[(f32,f32);3]= [(x1,y1),(x2,y2),(x3,y3)]; 
        quadratic_curve_with_alpha(self.layer_as_mut(),p.to_vec(), a, color,0xff,true,Some(size));
    }

    #[wasm_bindgen(js_name = bezierCurve)]
    pub fn bezier_curve(&mut self,x1: f32,y1: f32,x2: f32,y2: f32,x3:f32,y3:f32,color: u32) {
        let p :[(f32,f32);3]= [(x1,y1),(x2,y2),(x3,y3)]; 
        bezier_curve(self.layer_as_mut(),p.to_vec(), color);
    }

    #[wasm_bindgen(js_name = bezierCurveAntialias)]
    pub fn bezier_curve_antialias(&mut self,x1: f32,y1: f32,x2: f32,y2: f32,x3:f32,y3:f32,color: u32,size:f32) {
        let p :[(f32,f32);3]= [(x1,y1),(x2,y2),(x3,y3)]; 
        bezier_curve_with_alpha(self.layer_as_mut(),p.to_vec(), color,0xff,true,Some(size));
    }

    #[wasm_bindgen(js_name = bezierCurve3)]
    pub fn bezier_curve3(&mut self,x1: f32,y1: f32,x2: f32,y2: f32,x3:f32,y3:f32,x4:f32,y4:f32,color: u32) {
        let p :[(f32,f32);4]= [(x1,y1),(x2,y2),(x3,y3),(x4,y4)]; 
        bezier_curve(self.layer_as_mut(),p.to_vec(), color);
    }

    #[wasm_bindgen(js_name = bezierCurve3Antialias)]
    pub fn bezier_curve3_antialias(&mut self,x1: f32,y1: f32,x2: f32,y2: f32,x3:f32,y3:f32,x4:f32,y4:f32,color: u32,size:f32) {
        let p :[(f32,f32);4]= [(x1,y1),(x2,y2),(x3,y3),(x4,y4)]; 
        bezier_curve_with_alpha(self.layer_as_mut(),p.to_vec(), color,0xff,true,Some(size));
    }

    #[wasm_bindgen(js_name = affineTest2)]
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

    #[wasm_bindgen(js_name = affineTest)]
    pub fn affine_test(&mut self,canvas_in:usize,canvas_out:usize) {
        let mut affine = Affine::new();
        affine.invert_xy();
        affine.scale(5.3,5.3);
        affine.rotate_by_dgree(12.0);

        if canvas_in == 0 {
            self.combine();
            let output_canvas = &mut *self.append_canvas[canvas_out - 1].write().unwrap();
            affine.conversion(&self.canvas
                     ,output_canvas,InterpolationAlgorithm::Lanzcos(Some(3)));
//            affine.conversion(&self.canvas,output_canvas,InterpolationAlgorithm::Bilinear);
//            affine.conversion(&self.canvas,output_canvas,InterpolationAlgorithm::Bicubic(Some(-0.5)));
        } else if canvas_out == 0 {
            let input_canvas = & *self.append_canvas[canvas_in - 1].read().unwrap();
            affine.conversion(input_canvas,self.canvas.layer_as_mut(self.current.to_string()).unwrap()
                ,InterpolationAlgorithm::Bilinear);
        } else {
            let input_canvas = & *self.append_canvas[canvas_in - 1].read().unwrap();
            let output_canvas = &mut *self.append_canvas[canvas_out - 1].write().unwrap();
            affine.conversion(input_canvas,output_canvas,InterpolationAlgorithm::Bilinear);
        }
    }

    #[wasm_bindgen(js_name = imageDecoder)]
    pub fn image_decoder(&mut self,buffer: &[u8],verbose:usize) {
        let r = crate::paint::image::draw_image(self.layer_as_mut(),buffer,verbose);
        match r {
            Err(error) => {
                log(&format!("{:?}",error));
            }
            _ => {
  
            },
        }
    }

    #[wasm_bindgen(js_name = jpegDecoder)]
    pub fn jpeg_decoder(&mut self,buffer: &[u8],verbose:usize) {
        self.jpeg_decoder_select_canvas(buffer,verbose,0);
    }

    
    #[wasm_bindgen(js_name = jpegDecoderSelectCanvas)]
    pub fn jpeg_decoder_select_canvas(&mut self,buffer: &[u8],verbose:usize,number:usize) {
        if number > self.append_canvas.len() { return }

        if number != 0 {
            let mut option = DecodeOptions{
                debug_flag: 0,
                drawer: &mut *self.append_canvas[number - 1].write().unwrap(),
            };
            let r = image_loader(buffer, &mut option);
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
            let worker = self.on_worker;
            let canvas = self.layer_as_mut();
            if !worker {
                canvas.set_verbose(write_log);
            }
            let mut option = DecodeOptions{
                debug_flag: verbose,
                drawer: canvas,
            };
        
            let r = image_loader(buffer, &mut option);
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
                    self.combine();
                }
            }
        }
    }

    /// Javascript bindCanvas() is bind rust canvas and Web Canvas.
    /// This function cannnot run on web worker.
    #[wasm_bindgen(js_name = bindCanvas)]
    pub fn bind_canvas(&mut self,canvas:&str) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id(canvas).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        self.ctx = Some(context)
    }

    /// Javascript bindCanvas2() is bind rust canvas and Web Canvas 2nd.
    /// This function cannnot run on web worker.
    #[wasm_bindgen(js_name = bindCanvas2)]
    pub fn bind_canvas2(&mut self,canvas:&str) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id(canvas).unwrap();
        let canvas: web_sys::HtmlCanvasElement = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        self.ctx2 = Some(context)
    }

    /// Javascript drawCanvas() draws binded WebCanvas.
    /// Must call bindCanvas2 before.
    /// This function cannnot run on web worker.
    #[wasm_bindgen(js_name = drawCanvas)]
    pub fn draw_canvas(&mut self,width:u32,height:u32) -> Result<(),JsValue>{
        if let Some(ctx) = &self.ctx {
            self.canvas.combine();
            let clamped = Clamped(self.canvas.buffer());
            let img = ImageData::new_with_u8_clamped_array_and_sh(clamped,width,height)?;
            ctx.put_image_data(&img,0_f64,0_f64)
        } else {
            Err(JsValue::FALSE)
        }
    }

    /// Javascript drawSelectCanvas() draws binded WebCanvas 1st.
    /// A no selects main canvas or append canvases
    /// Must call bindCanvas() before.
    /// This function cannnot run on web worker.
    #[wasm_bindgen(js_name = drawSelectCanvas)]
    pub fn draw_canvas_with_number(&mut self,width:u32,height:u32,no:usize) -> Result<(),JsValue>{
        if let Some(ctx) = &self.ctx {
            if no == 0 {
                let canvas = &mut self.canvas;
                canvas.combine();
                let clamped = Clamped(canvas.buffer());
                let img = ImageData::new_with_u8_clamped_array_and_sh(clamped,width,height)?;
                ctx.put_image_data(&img,0_f64,0_f64)
            } else {
                if self.append_canvas.len() > no {
                    return Err(JsValue::FALSE)
                } 
                let canvas = &mut self.append_canvas[no - 1].write().unwrap();
//                canvas.combine(); no impl
                let clamped = Clamped(canvas.buffer());
                let img = ImageData::new_with_u8_clamped_array_and_sh(clamped,width,height)?;
                ctx.put_image_data(&img,0_f64,0_f64)
            }
        } else {
            Err(JsValue::FALSE)
        }
    }

    #[wasm_bindgen(js_name = getImageData)]
    pub fn get_imagedata(&mut self,no: usize) -> Result<ImageData, JsValue>{
        if no == 0 {
            let width = self.width();
            let height = self.height();
            let canvas = &mut self.canvas;
            canvas.combine();
            let clamped = Clamped(canvas.buffer());
            ImageData::new_with_u8_clamped_array_and_sh(clamped,width,height)
        } else {
            if self.append_canvas.len() > no {
                return Err(JsValue::FALSE)
            } 
            let canvas = &mut self.append_canvas[no - 1].write().unwrap();
    //            canvas.combine(); noimpl
            let clamped = Clamped(canvas.buffer());
            ImageData::new_with_u8_clamped_array_and_sh(clamped,self.width(),self.height())
        }
    }

    pub fn combine(&mut self) {
        self.canvas.combine();
    }


    #[wasm_bindgen(js_name = drawCanvas2)]
    pub fn draw_canvas2(&mut self,width:u32,height:u32) -> Result<(),JsValue>{
        if self.append_canvas.len() == 0 {
            return Err(JsValue::FALSE)
        } 
        if let Some(ctx) = &self.ctx2 {
            let canvas = &mut self.append_canvas[0].write().unwrap();
            let clamped = Clamped(canvas.buffer());
            let img = ImageData::new_with_u8_clamped_array_and_sh(clamped,width,height)?;
            ctx.put_image_data(&img,0_f64,0_f64)
        } else {
            Err(JsValue::FALSE)
        }
    }

    #[wasm_bindgen(js_name = drawSelectCanvas2)]
    pub fn draw_canvas2_with_number(&mut self,width:u32,height:u32,no:usize) -> Result<(),JsValue>{
        if let Some(ctx) = &self.ctx2 {
            if no == 0 {
                let canvas = &mut self.canvas;
                canvas.combine();
                let clamped = Clamped(canvas.buffer());
                let img = ImageData::new_with_u8_clamped_array_and_sh(clamped,width,height)?;
                ctx.put_image_data(&img,0_f64,0_f64)
            } else {
                if self.append_canvas.len() > no {
                    return Err(JsValue::FALSE)
                } 
                let canvas = &mut self.append_canvas[no - 1].write().unwrap();
                //canvas.combine();
                let clamped = Clamped(canvas.buffer());
                let img = ImageData::new_with_u8_clamped_array_and_sh(clamped,width,height)?;
                ctx.put_image_data(&img,0_f64,0_f64)
            }
        } else {
            Err(JsValue::FALSE)
        }
    }

}
