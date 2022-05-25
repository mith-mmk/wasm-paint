//! A canvas is base on image.
/*
 * canvas.rs  Mith@mmk (C) 2022
 * update 2022/03/13
 */

type Error = Box<dyn std::error::Error>;
use wml2::metadata::Metadata;
use crate::draw::draw_over_screen;
use crate::clear::clear_layter;
use crate::layer::AnimationControl;
use wml2::metadata::DataMap;
use wml2::draw::*;
use wml2::error::ImgError;
use wml2::error::ImgErrorKind;
use std::collections::HashMap;
use crate::layer::Layer;
use crate::draw::*;
use crate::pen::Pen;
use crate::clear::fillrect;

pub trait Screen {
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn reinit(&mut self,width:u32, height:u32);
    fn buffer(&self) -> &[u8];
    fn buffer_mut(&mut self) -> &mut [u8];
    fn clear(&mut self);
    fn clear_with_color(&mut self,color: u32);

    fn alpha(&self) -> Option<u8>;
    fn set_alpha(&mut self,alpha:u8);

}

struct AnimationLayer{
    pub x:i32,
    pub y:i32,
    layer: Layer,
    layers: Vec<Layer>,
    frame_no: usize,
    enable: bool,
}

impl AnimationLayer {
    pub fn new(label:String,width:u32,height:u32) -> Self {
        let layer = Layer::new(label.to_string(),width,height);
        let layers = vec![layer];
        let layer = Layer::new(label.to_string(),width,height);

        Self {
            x: 0,
            y: 0,
            layer: layer,
            layers,
            frame_no:0,
            enable: true,
        }
    }

    pub fn set_frame_no(&mut self,no: usize) -> usize{
        if self.layers.len() >= no {
            self.frame_no = 0;
        } else {
            self.frame_no = no;
        }
        self.frame_no
    }

    pub fn layer(&mut self) -> &Layer {
        &self.layers[self.frame_no]
    }

    pub fn layer_mut(&mut self) -> &mut Layer {
        &mut self.layers[self.frame_no]
    }

    pub fn wait(&mut self) -> u64 {
        if let Some(control) = &self.layers[self.frame_no].control {
            control.await_time
        } else {
            0
        }
    }

    pub fn next(&mut self) -> usize {
        let control = self.layers[self.frame_no].control.as_ref();
        if let Some(control) = control {
            if let Some(dispose) = &control.dispose_option {
                match dispose {
                    NextDispose::Previous => {
                        self.layers[self.frame_no].set_disable();
                    },
                    NextDispose::Background => {
                        for i in 0..self.frame_no {
                            self.layers[i].set_disable();
                        }
                    },
                    _ => {
                    }
                }
            }
        }

        self.frame_no += 1;
        if self.frame_no >= self.layers.len() {
            self.reset();
        }
        self.layers[self.frame_no].set_enable();
        self.frame_no
    }

    pub fn reset(&mut self) {
        for i in 1..self.layers.len() {
            self.layers[i].set_disable();
        }       
        self.set_frame_no(0);
        self.layers[self.frame_no].set_enable();
    }

    pub fn truncate(&mut self) {
        self.frame_no = 0;
        self.layers.truncate(1);
    }

    pub fn combined_layer(&mut self) -> &mut Layer {
        let layers = &mut self.layers;
        if layers.len() == 1 {
            return &mut layers[0]
        }

        clear_layter(&mut self.layer);
        for layer in layers {
            let x = layer.x;
            let y = layer.y;
            if layer.enable() {
                if let Some(control) = &layer.control {
                    if let Some(blend) = &control.blend {
                        match blend {
                            NextBlend::Source => {
                                draw_over_screen_with_alpha(layer,&mut self.layer,x,y);
                            },
                            _ => {
                                draw_over_screen(layer,&mut self.layer,x,y);
                            }
                        }
                    } else {
                        draw_over_screen(layer,&mut self.layer,x,y);
                    }
                } else {
                    draw_over_screen(layer,&mut self.layer,x,y);
                }
            }
        }
        &mut self.layer
    }

    pub fn add_frame(&mut self,label:String,width:u32,height:u32,start_x:i32,start_y:i32) {
        let mut layer = Layer::new(label, width, height);
        layer.x = start_x;
        layer.y = start_y;
        layer.enable = false;
        self.frame_no = self.layers.len();
        self.layers.push(layer);
    }

    pub fn z_index(&self) -> i32 {
        self.layers[0].z_index
    }

    pub fn set_z_index(&mut self,z_index:i32) {
        self.layers[0].set_z_index(z_index);
    }

    pub fn set_enable(&mut self) {
        self.enable = true;
    }
    pub fn set_disable(&mut self) {
        self.enable = false;
    }

    pub fn enable(&self) -> bool {
        self.enable
    }
}


struct PackedLayers {
    layers: HashMap<String,AnimationLayer>,
    sorted: Vec<String>,
}

impl PackedLayers {
    fn new() -> Self {
        Self {
            layers: HashMap::new(),
            sorted: Vec::new(),
        }
    }

    fn sort(&mut self) {
        let mut vector:Vec<(&String,i32)> = self.layers.iter().map(|(k, v)| (k,v.z_index())).collect();
        vector.sort_by(|a, b| a.1.cmp(&b.1));
        let sorted:Vec<String> = vector.iter().map(|x| x.0.to_string()).collect();
        self.sorted = sorted;
    }


    fn set_z_index(&mut self,label:String,z_index:i32) -> Result<(),Error> {
        match self.layers.get_mut(&label) {
            Some(layer) => {
                layer.set_z_index(z_index);
                self.sort();
                Ok(())
            },
            _ => {
                return Err(Box::new(crate::error::Error{message:"No exist Layer".to_string()}))
            },
        }
    }

    fn add(&mut self,label:String,width :u32,height: u32,x: i32,y :i32) -> Result<(),Error> {
        match self.layers.get(&label) {
            Some(..) => {
                return Err(Box::new(crate::error::Error{message:"Exist Layer name".to_string()}))
            },
            _ => {},

        };
        let mut layer = AnimationLayer::new(label.clone(), width, height);
        layer.set_z_index(self.layers.len() as i32);
        layer.x = x;
        layer.y = y;
        self.layers.insert(label.clone(),layer);
        self.sorted.push(label);
        self.sort();
        Ok(())
    }

    fn add_frame(&mut self,label:String,width :u32,height: u32,x: i32,y :i32) -> Result<(),Error> {
        match self.layers.get_mut(&label) {
            Some(layer) => {
                layer.add_frame(label,width,height,x,y);
            },
            _ => {
                return Err(Box::new(crate::error::Error{message:"Layer is none".to_string()}))
            },

        };
        Ok(())
    }

    fn set_frame_no(&mut self,label:String,no:usize) -> usize{
        let layer = self.layers.get_mut(&label);
        if let Some(layer) = layer {
            layer.set_frame_no(no)
        } else {
            0
        }
    }

    fn frame_last_no(&mut self,label:String) -> Option<usize>{
        let layer = self.layers.get_mut(&label);
        if let Some(layer) = layer {
            if layer.layers.len() >= 1{
                Some(layer.layers.len() - 1)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get(&self,label:String) -> Option<&Layer> {
        let layer = self.layers.get(&label);
        if let Some(frame) = layer {
            Some(&frame.layers[0])
        } else {
            None
        }
    }

    fn get_combined_layer(&mut self,label:String) -> Option<&mut Layer> {
        let layer = self.layers.get_mut(&label);
        if let Some(layer) = layer {
            Some(layer.combined_layer())
        } else {
            None
        }
    }

    fn get_mut(&mut self,label:String) -> Option<&mut Layer> {
        let layer = self.layers.get_mut(&label);
        if let Some(frame) = layer {
            Some(&mut frame.layers[0])
        } else {
            None
        }
    }

    fn get_frame(&mut self,label:String) -> Option<&Layer> {
        let layer = self.layers.get_mut(&label);
        if let Some(frame) = layer {
            Some(frame.layer())
        } else {
            None
        }
    }

    fn get_frame_mut(&mut self,label:String) -> Option<&mut Layer> {
        let layer = self.layers.get_mut(&label);
        if let Some(frame) = layer {
            Some(frame.layer_mut())
        } else {
            None
        }
    }

    fn sorted(&self) -> &Vec<String> {
        &self.sorted
    }

    fn len(&self) -> usize {
        self.layers.len()
    }

    fn truncate(&mut self,label:String) {
        let layer = self.layers.get_mut(&label);
        if let Some(frame) = layer {
            frame.truncate();
        }
    }

    fn clear(&mut self) {
        for key in &self.sorted().clone() {
            let layer = &mut self.get_mut(key.to_string()).unwrap();
            layer.clear();
        }
    }

    fn clear_layer(&mut self,label:String) -> Result<(),Error>  {
        match self.get_mut(label) {
            Some(layer) => {
                layer.clear();
                Ok(())
            },
            _ => {
                Err(Box::new(crate::error::Error{message:"No exist Layer name".to_string()}))
            },
        }        
    }

    fn set_enable(&mut self,label:String) -> Result<(),Error> {
        match self.layers.get_mut(&label) {
            Some(layer) => {
                layer.set_enable();
                Ok(())
            },
            _ => {
                Err(Box::new(crate::error::Error{message:"No exist Layer name".to_string()}))
            },
        }        
    }

    fn set_disable(&mut self,label:String) -> Result<(),Error> {
        match self.layers.get_mut(&label) {
            Some(layer) => {
                layer.set_disable();
                Ok(())
            },
            _ => {
                Err(Box::new(crate::error::Error{message:"No exist Layer name".to_string()}))
            },
        }        
    }

    fn enable(&self,label:String) -> Result<bool,Error> {
        match self.layers.get(&label) {
            Some(layer) => {
                Ok(layer.enable())
            },
            _ => {
                Err(Box::new(crate::error::Error{message:"No exist Layer name".to_string()}))
            },
        }        
    }

    fn remove(&mut self,label:String) {
        self.layers.remove(&label);
        let mut sorted:Vec<String> = Vec::new();
        for key in &self.sorted {
            if key != &label {
                sorted.push(key.to_string())
            }
        }
        self.sorted = sorted;
        self.sort();
    }

    fn set_next(&mut self,label:String) -> Result<usize,Error> {
        match self.layers.get_mut(&label) {
            Some(layer) => {
                layer.next();
                Ok(layer.frame_no)
            },
            _ => {
                Err(Box::new(crate::error::Error{message:"No exist Layer name".to_string()}))
            },
        }       
    }

    fn wait(&mut self,label:String) -> Result<u64,Error> {
        match self.layers.get_mut(&label) {
            Some(layer) => {
                Ok(layer.wait())
            },
            _ => {
                Err(Box::new(crate::error::Error{message:"No exist Layer name".to_string()}))
            },
        }       
    }

}

pub struct Canvas {
    canvas: Layer,
    color: u32,
    background_color: u32,
    pen: Pen,
    layers: PackedLayers,
    current_layer: Option<String>,
    frame_no:usize,
    loop_count: Option<u32>,
    fnverbose: fn(&str,Option<VerboseOptions>) -> Result<Option<CallbackResponse>,Error>,
    draw_width: u32,
    draw_height: u32,
    use_canvas_alpha: bool,
    canvas_alpha: u8,
    metadata: Option<HashMap<String,DataMap>>,
    is_animation:bool,
}

pub(crate) fn default_verbose(_ :&str,_: Option<VerboseOptions>) -> Result<Option<CallbackResponse>, Error>{
    Ok(None)
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        let mut this = Self::empty();
        if width == 0 || width >= 0x8000000 || height == 0 || height >= 0x8000000 {
            return this
        }

        this.canvas = Layer::new("_".to_string(),width,height);
        this
    }

    fn empty() -> Self {
        Self {
            canvas:Layer::new("_".to_string(),0,0),
            color: 0x00ffffff,
            background_color: 0,
            use_canvas_alpha: false,
            canvas_alpha: 0xff,
            pen: Pen::new(1, 1, vec![255]),
            layers: PackedLayers::new(),
            current_layer: None,
            frame_no: 0,
            loop_count: None,
            fnverbose: default_verbose,
            draw_width: 0,
            draw_height: 0,
            metadata: None,
            is_animation:false,
        }
    }

    pub fn new_in(buffer: Vec<u8>,width: u32, height: u32) -> Self {
        let mut this = Self::empty();
        if width == 0 || width >= 0x8000000 || height == 0 || height >= 0x8000000 {
            return this
        }
        this.canvas = Layer::new_in("_".to_string(),buffer,0,0);

        this
    }

    /// for WebAssembly
    pub fn canvas(&self) -> *const u8 {
        self.canvas.buffer.as_ptr()
    }

    pub fn layers_len(&self) -> usize {
        self.layers.len()
    }

    /// for Wml2
    pub fn set_verbose(&mut self,verbose:fn(&str,Option<VerboseOptions>) -> Result<Option<CallbackResponse>,Error>) {
        self.fnverbose = verbose;
    }
    
    pub fn set_buckground_color(&mut self,color: u32) {
        self.background_color = color;
    }

    pub fn background_color(&self) -> u32 {
        self.background_color
    }

    pub fn color(&self) -> u32 {
        self.color.clone()
    }

    pub fn set_color(&mut self,color: u32) {
        self.color = color;
    }

    pub fn set_pen(&mut self,pen :Pen) {
        self.pen = pen;
    }

    pub fn pen(&self) -> Pen{
        self.pen.clone()
    }

    pub fn set_current(&mut self,current:String) {
        self.current_layer = Some(current);
    }

    pub fn current(&self) -> String {
        if let Some(current) = &self.current_layer {
            current.to_string()
        } else {
            "_".to_string()
        }
    }

    pub fn add_frame(&mut self,label:String,width:u32,height:u32,x:i32,y:i32) -> Result<(),Error> {
        self.layers.add_frame(label.clone(),width,height,x,y)?;
        Ok(())
    }

    pub fn add_layer(&mut self,label:String,width:u32,height:u32,x:i32,y:i32) -> Result<(),Error> {
        self.layers.add(label.clone(),width,height,x,y)?;
        Ok(())
    }

    pub fn set_layer_alpha(&mut self,label:String,alpha:u8) -> Result<(),Error> {
        if let Some(layer) = self.layers.get_mut(label) {
            layer.set_alpha(alpha);
            Ok(())
        }  else {
            Err(Box::new(crate::error::Error{message:"No exist Layer name".to_string()}))
        }
    }

    pub fn set_z_index(&mut self,label:String,z_index:i32) -> Result<(),Error> {
        self.layers.set_z_index(label,z_index)
    }

    pub fn layer(&mut self,label:String) -> Option<&Layer> {
        self.layers.get(label)
    }

    pub fn frame(&mut self,label:String) -> Option<&Layer> {
        self.layers.get_frame(label)
    }

    pub fn layer_mut(&mut self,label:String) -> Option<&mut Layer> {
        self.layers.get_mut(label)
    }
    pub fn frame_mut(&mut self,label:String) -> Option<&mut Layer> {
        self.layers.get_frame_mut(label)
    }

    pub fn set_enable(&mut self,label:String) -> Result<(),Error> {
        self.layers.set_enable(label)
    }

    pub fn set_disable(&mut self,label:String) -> Result<(),Error> {
        self.layers.set_disable(label)
    }

    pub fn enable(&self,label:String) -> Result<bool,Error> {
        self.layers.enable(label)
    }

    pub fn combine(&mut self) {
        let background_color = &self.background_color();
        let canvas = &mut self.canvas;
        fillrect(canvas,*background_color);
        let sorted = self.layers.sorted().clone();
        for label in sorted {
            match self.layers.get_combined_layer(label.clone()) {
                Some(layer) => {
                    let (x,y) = layer.pos();
                    if layer.enable() {
                        draw_over_screen_with_alpha(layer,canvas,x,y);
                    }
                },
                _ => {}
            }
        }
    }

    pub fn clear_layer(&mut self,label:String) -> Result<(),Error> {
        self.layers.clear_layer(label)
    }

    pub fn delete_layer(&mut self,label:String){ 
        if let Some(..) = self.layers.get_mut(label.clone()) {
           self.layers.remove(label);
        }
    }

    pub fn set_pos(&mut self,label:String,x:i32,y:i32) {
        if let Some(layer) = self.layers.get_mut(label) {
                layer.set_pos(x,y);
        }
    }

    pub fn move_pos(&mut self,label:String,dx:i32,dy:i32) {
        if let Some(layer) = self.layers.get_mut(label) {
                layer.move_pos(dx,dy);
        }
    }

    pub fn pos(&mut self,label:String) -> Option<(i32,i32)> {
        if let Some(layer) = self.layers.get_mut(label) {
            Some(layer.pos())
        } else {
            None
        }
    }

    pub fn get_layer_alpha(&mut self,label:String) -> Result<Option<u8>,Error>  { 
        if let Some(layer) = self.layers.get_mut(label) {
            Ok(layer.alpha())
        } else {
            Err(Box::new(crate::error::Error{message:"No exist Layer name".to_string()}))
        }
    }

    pub fn set_next(&mut self,label:String) -> Result<usize,Error> {
        self.layers.set_next(label)  
    }

    pub fn wait(&mut self,label:String) -> Result<u64,Error> {
        self.layers.wait(label)
    }

    pub fn is_animation(&self) -> bool {
        self.is_animation
    }

    
    pub fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }
}

impl Screen for Canvas {

    fn width(&self) -> u32 {
        self.canvas.width.clone()
    }

    fn height(&self) -> u32 {
        self.canvas.height.clone()
    }

    fn reinit(&mut self,width: u32, height: u32) {
        self.canvas.width = width;
        self.canvas.height = height;
        let buffersize = width as usize * height as usize * 4;
        self.canvas.buffer = vec![0;buffersize];
    }

    fn buffer(&self) -> &[u8] {
        &self.canvas.buffer
    }

    fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.canvas.buffer
    }

    fn clear(&mut self) {
        self.clear_with_color(self.background_color);
        self.layers.clear();
    }

    fn clear_with_color(&mut self,color: u32) {
        fillrect(self,color);  
    }

    fn alpha(&self) -> Option<u8> {
        if self.use_canvas_alpha {
            Some(self.canvas_alpha)
        } else {
            None
        }
    }

    fn set_alpha(&mut self,alpha: u8) {
        self.canvas_alpha = alpha;
        self.use_canvas_alpha = true;
    }


}

impl DrawCallback for Canvas {
    fn init(&mut self, width: usize, height: usize,opt: Option<InitOptions>) -> Result<Option<CallbackResponse>, Error> {
        if width <= 0 || height <= 0 {
            return Err(Box::new(ImgError::new_const(ImgErrorKind::SizeZero,"image size zero or minus".to_string())))
        }

        if let Some(option) = opt {
            self.loop_count = Some(option.loop_count);
            self.frame_no = 0;
            if let Some(background) = &option.background {
                let background_color = 
                    (background.red as u32) << 16 | 
                    (background.green as u32) << 8 | 
                    (background.blue as u32);
                self.background_color = background_color;
            }
            self.is_animation = option.animation;
        } else {
            self.loop_count = Some(0);
            self.frame_no = 0;
            self.is_animation = false;
        }

        if self.width() == 0 || self.height() == 0 {
            let buffersize = width as usize * height as usize * 4;
            self.canvas.width = width as u32;
            self.canvas.height = height as u32;
            self.canvas.buffer = (0..buffersize).map(|_| 0).collect();
        }

        match &self.current_layer.clone() {
            None => {

                self.draw_width = width as u32;
                self.draw_height = height as u32;
            },
            Some(label) => {
                self.layers.truncate(label.to_string());
                let mut layer = self.layer_mut(label.to_string()).unwrap();
                if layer.width == 0 || layer.height == 0 {
                    layer.width = width as u32;
                    layer.height = height as u32;
                    let buffersize = width as usize * height as usize * 4;
                    layer.buffer =  (0..buffersize).map(|_| 0).collect();
                }
            }
        }
        Ok(None)
    }

    fn draw(&mut self, start_x: usize, start_y: usize, width: usize, height: usize, data: &[u8],_: Option<DrawOptions>)
                -> Result<Option<CallbackResponse>,Error>  {
        let layer;
        let current_layer = &self.current_layer.clone();
        match current_layer {
            Some(label) => {
                if self.frame_no == 0 {
                    layer = self.layer_mut(label.to_string()).unwrap();
                } else {
                    layer = self.frame_mut(label.to_string()).unwrap();
                }
            },
            _ => {
                layer = self.canvas.get_mut();
            }
        }


        let self_width = layer.width() as usize;
        let self_height = layer.height() as usize;

        let buffer =  &mut layer.buffer_mut();
        if start_x >= self_width || start_y >= self_height {return Ok(None);}
        let w = if self_width < width + start_x {self_width - start_x} else { width };
        let h = if self_height < height + start_y {self_height - start_y} else { height };
        for y in 0..h {
            let scanline_src =  y * width * 4;
            let scanline_dest= (start_y + y) * self_width * 4;
            for x in 0..w {
                let offset_src = scanline_src + x * 4;
                let offset_dest = scanline_dest + (x + start_x) * 4;
                if offset_src + 3 >= data.len() {
                    return Err(Box::new(ImgError::new_const(ImgErrorKind::OutboundIndex,"decoder buffer in draw".to_string())))
                }
                buffer[offset_dest    ] = data[offset_src];
                buffer[offset_dest + 1] = data[offset_src + 1];
                buffer[offset_dest + 2] = data[offset_src + 2];
                buffer[offset_dest + 3] = data[offset_src + 3];
            }
        }
        Ok(None)
    }

    fn terminate(&mut self,_: Option<TerminateOptions>) -> Result<Option<CallbackResponse>, Error> {
        self.frame_no = 0;
        Ok(None)
    }

    fn next(&mut self, opt: Option<NextOptions>) -> Result<Option<CallbackResponse>, Error> {
   
        let current_layer = &self.current_layer.clone();
        if let Some(label) = current_layer {
            if let Some(opt) = opt {
                let (width,height,start_x,start_y);
                if let Some(rect) = opt.image_rect {
                    width = rect.width as u32;
                    height = rect.height as u32;
                    start_x = rect.start_x;
                    start_y = rect.start_y;
                } else {
                    width = self.layer(label.to_string()).unwrap().width();
                    height = self.layer(label.to_string()).unwrap().height();
                    start_x = 0;
                    start_y = 0;
                }
                let await_time = opt.await_time;
                let dispose_option = opt.dispose_option;
                let blend = opt.blend;
                self.add_frame(label.to_string(), width, height, start_x, start_y)?;
                let control = AnimationControl{
                    await_time,
                    dispose_option,
                    blend,
                };
                self.frame_mut(label.to_string()).unwrap().control = Some(control);
                self.frame_no = self.layers.frame_last_no(label.to_string()).unwrap_or(0);
                Ok(Some(CallbackResponse::cont()))
            } else {
                Ok(Some(CallbackResponse::abort()))
            }
        } else {
            Ok(Some(CallbackResponse::abort()))
        }
    }

    fn verbose(&mut self, str: &str,_: Option<VerboseOptions>) -> Result<Option<CallbackResponse>, Error> { 
        return (self.fnverbose)(str,None);
    }

    fn set_metadata(&mut self,key: &str, value: DataMap) -> Result<Option<CallbackResponse>, Error> { 
        let hashmap = if let Some(ref mut hashmap) = self.metadata {
            hashmap
        } else {
            self.metadata = Some(HashMap::new());
            self.metadata.as_mut().unwrap()
        };
        hashmap.insert(key.to_string(), value);

        return Ok(None)
    }
}

