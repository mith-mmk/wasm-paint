//! A canvas is base on image.
/*
 * canvas.rs  Mith@mmk (C) 2022
 * update 2022/03/13
 */
extern crate wml2;
type Error = Box<dyn std::error::Error>;
use std::collections::HashMap;
use wml2::draw::*;
use wml2::error::ImgError;
use wml2::error::ImgErrorKind;
use super::layer::Layer;
use super::draw::draw_over_screen_with_alpha;
use super::pen::Pen;
use super::clear::fillrect;

pub trait Screen {
    fn width(&self) -> u32;
    fn height(&self) -> u32;

    fn buffer(&self) -> &[u8];
    fn buffer_mut(&mut self) -> &mut [u8];
    fn clear(&mut self);
    fn clear_with_color(&mut self,color: u32);

    fn alpha(&self) -> Option<u8>;
    fn set_alpha(&mut self,alpha:u8);

}

struct PackedLayers {
    layers: HashMap<String,Layer>,
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
        let mut vector:Vec<(&String,i32)> = self.layers.iter().map(|(k, v)| (k,v.z_index)).collect();
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
                return Err(Box::new(super::error::Error{message:"No exist Layer".to_string()}))
            },
        }
    }

    fn add(&mut self,label:String,width :u32,height: u32,x: i32,y :i32) -> Result<(),Error> {
        match self.layers.get(&label) {
            Some(..) => {
                return Err(Box::new(super::error::Error{message:"Exist Layer name".to_string()}))
            },
            _ => {},

        };
        let mut layer = Layer::new(label.clone(), width, height);
        layer.z_index = self.layers.len() as i32;
        layer.x = x;
        layer.y = y;
        self.layers.insert(label.clone(),layer);
        self.sorted.push(label);
        self.sort();
        Ok(())
    }

    fn get(&self,label:String) -> Option<&Layer> {
        self.layers.get(&label)
    }

    fn get_mut(&mut self,label:String) -> Option<&mut Layer> {
        self.layers.get_mut(&label)
    }

    fn sorted(&self) -> &Vec<String> {
        &self.sorted
    }

    fn len(&self) -> usize {
        self.layers.len()
    }

    fn clear(&mut self) {
        for key in &self.sorted {
            let layer = self.layers.get_mut(key).unwrap();
            layer.clear();
        }
    }

    fn clear_layer(&mut self,label:String) -> Result<(),Error>  {
        match self.layers.get_mut(&label) {
            Some(layer) => {
                layer.clear();
                Ok(())
            },
            _ => {
                Err(Box::new(super::error::Error{message:"No exist Layer name".to_string()}))
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
                Err(Box::new(super::error::Error{message:"No exist Layer name".to_string()}))
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
                Err(Box::new(super::error::Error{message:"No exist Layer name".to_string()}))
            },
        }        
    }

    fn enable(&self,label:String) -> Result<bool,Error> {
        match self.layers.get(&label) {
            Some(layer) => {
                Ok(layer.enable())
            },
            _ => {
                Err(Box::new(super::error::Error{message:"No exist Layer name".to_string()}))
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

}

pub struct Canvas {
    canvas: Layer,
    color: u32,
    background_color: u32,
    pen: Pen,
    layers: PackedLayers,
    fnverbose: fn(&str,Option<VerboseOptions>) -> Result<Option<CallbackResponse>,Error>,
    draw_width: u32,
    draw_height: u32,
    use_canvas_alpha: bool,
    canvas_alpha: u8,
}

pub(crate) fn default_verbose(_ :&str,_: Option<VerboseOptions>) -> Result<Option<CallbackResponse>, Error>{
    Ok(None)
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        if width == 0 || width >= 0x8000000 || height == 0 || height >= 0x8000000 {
            return Self::empty()
        }
        let pen = Pen::new(1, 1, vec![255]);
        let color = 0xfffff;
        let background_color = 0;
        let fnverbose = default_verbose;

        Self {
            canvas:Layer::new("_".to_string(),width,height),
            color,
            background_color,
            use_canvas_alpha: false,
            canvas_alpha: 0xff,
            pen,
            layers: PackedLayers::new(),
            fnverbose,
            draw_width: 0,
            draw_height: 0,
        }
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
            fnverbose: default_verbose,
            draw_width: 0,
            draw_height: 0,
        }
    }

    pub fn new_in(buffer: Vec<u8>,width: u32, height: u32) -> Self {
        if width == 0 || width >= 0x8000000 || height == 0 || height >= 0x8000000 {
            return Self::empty()
        }
        let pen = Pen::new(1, 1, vec![255]);
        let color = 0xfffff;
        let background_color = 0;
        let fnverbose = default_verbose;

        Self {
            canvas:Layer::new_in("_".to_string(),buffer,0,0),
            color,
            background_color,
            use_canvas_alpha: false,
            canvas_alpha: 0xff,
            layers: PackedLayers::new(),

            pen,
            fnverbose,
            draw_width: 0,
            draw_height: 0,
        }
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

    pub fn add_layer(&mut self,label:String,width:u32,height:u32,x:i32,y:i32) -> Result<(),Error> {
        self.layers.add(label.clone(),width,height,x,y)?;
        Ok(())
    }

    pub fn set_layer_alpha(&mut self,label:String,alpha:u8) -> Result<(),Error> {
        if let Some(layer) = self.layers.get_mut(label) {
            layer.set_alpha(alpha);
            Ok(())
        }  else {
            Err(Box::new(super::error::Error{message:"No exist Layer name".to_string()}))
        }
    }

    pub fn set_z_index(&mut self,label:String,z_index:i32) -> Result<(),Error> {
        self.layers.set_z_index(label,z_index)
    }

    pub fn layer(&mut self,label:String) -> Option<&Layer> {
        self.layers.get(label)
    }

    pub fn layer_mut(&mut self,label:String) -> Option<&mut Layer> {
        self.layers.get_mut(label)
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
        let sorted = self.layers.sorted();
        for label in sorted {
            match self.layers.get(label.clone()) {
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
            Err(Box::new(super::error::Error{message:"No exist Layer name".to_string()}))
        }
    }
}

impl Screen for Canvas {

    fn width(&self) -> u32 {
        self.canvas.width.clone()
    }

    fn height(&self) -> u32 {
        self.canvas.height.clone()
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
    fn init(&mut self, width: usize, height: usize,_: Option<InitOptions>) -> Result<Option<CallbackResponse>, Error> {
        if width <= 0 || height <= 0 {
            return Err(Box::new(ImgError::new_const(ImgErrorKind::SizeZero,"image size zero or minus".to_string())))
        }
        if self.width() == 0 || self.height() == 0 {
            let buffersize = width as usize * height as usize * 4;
            self.canvas.buffer = (0..buffersize).map(|_| 0).collect();
        }
        self.draw_width = width as u32;
        self.draw_height = height as u32;
        Ok(None)
    }

    fn draw(&mut self, start_x: usize, start_y: usize, width: usize, height: usize, data: &[u8],_: Option<DrawOptions>)
                -> Result<Option<CallbackResponse>,Error>  {
        let self_width = self.width() as usize;
        let self_height = self.height() as usize;

        let buffer =  &mut self.buffer_mut();
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
        Ok(None)
    }

    fn next(&mut self, _: Option<NextOptions>) -> Result<Option<CallbackResponse>, Error> {
        Ok(Some(CallbackResponse::abort()))
    }

    fn verbose(&mut self, str: &str,_: Option<VerboseOptions>) -> Result<Option<CallbackResponse>, Error> { 
        return (self.fnverbose)(str,None);
    }
}

