//! Layer is canvas overlay images.
type Error = Box<dyn std::error::Error>;
use wml2::metadata::DataMap;
use wml2::error::*;
use wml2::draw::*;
use std::collections::HashMap;
use crate::clear::fillrect_with_alpha;
use super::canvas::*;

pub struct AnimationControl {
    pub await_time: u64,
    pub dispose_option: Option<NextDispose>,
    pub blend: Option<NextBlend>,
}

impl AnimationControl {
    pub fn new(await_time:u64) -> Self {
        Self {
            await_time,
            dispose_option:None,
            blend:None,
        }
    }
}

pub struct Layer {
    pub label:String,
    pub(crate) buffer: Vec<u8>,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) z_index: i32,
    pub(crate) use_canvas_alpha: bool,
    pub(crate) canvas_alpha: u8,
    pub(crate) enable: bool,
    pub(crate) control: Option<AnimationControl>,
    fnverbose: fn(&str,Option<VerboseOptions>) -> Result<Option<CallbackResponse>,Error>,
    pub(crate) metadata: Option<HashMap<String,DataMap>>,
}

impl Layer {
    pub fn new(label:String,width:u32,height:u32) -> Self {
        let buffer = (0..(width*height*4) as usize).map(|_| 0).collect();
        Self {
            label,
            buffer,
            x:0,
            y:0,
            width: width,
            height: height,
            z_index: 0,
            use_canvas_alpha: true,
            canvas_alpha: 0xff,
            enable: true,
            control: None,
            fnverbose: super::canvas::default_verbose,
            metadata: None,
        }
    }

    pub fn new_in(label:String,buffer:Vec<u8>,width:u32,height:u32) -> Self {
        Self {
            label,
            buffer,
            x:0,
            y:0,
            width: width,
            height: height,
            z_index: 0,
            use_canvas_alpha: true,
            canvas_alpha: 0xff,
            enable: true,
            control: None,
            fnverbose: super::canvas::default_verbose,
            metadata: None,
        }
    }

    pub fn set_enable(&mut self) {
        self.enable = true
    }

    pub fn set_disable(&mut self) {
        self.enable = false
    }

    pub fn enable(&self) -> bool {
        self.enable
    }

    pub fn set_z_index(&mut self,z_index: i32) {
        self.z_index = z_index;
    }

    pub fn z_index(&mut self) -> i32 {
        self.z_index.clone()
    }

    pub fn set_pos(&mut self,x:i32,y:i32) {
        self.x = x;
        self.y = y;
    }

    pub fn pos(&self) -> (i32,i32) {
        (self.x.clone(),self.y.clone())
    }

    pub fn move_pos(&mut self,dx:i32,dy:i32) {
        self.x += dx;
        self.y += dy;
    }

    pub fn set_verbose(&mut self,verbose:fn(&str,Option<VerboseOptions>) -> Result<Option<CallbackResponse>,Error>) {
        self.fnverbose = verbose;
    }

    pub fn get_mut(&mut self) -> &mut Self {
        self
    }
}

impl Screen for Layer {
    fn width(&self) -> u32 {
        self.width.clone()
    }

    fn height(&self) -> u32 {
        self.height.clone()
    }

    fn reinit(&mut self,width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let buffersize = width as usize * height as usize * 4;
        self.buffer = vec![0;buffersize];
    }

    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn clear(&mut self) {
        fillrect_with_alpha(self,0,0);  
    }

    fn clear_with_color(&mut self,color: u32) {
        fillrect_with_alpha(self,color,0);  
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

impl DrawCallback for Layer {
    fn init(&mut self, width: usize, height: usize,_: Option<InitOptions>) -> Result<Option<CallbackResponse>, Error> {
        if width <= 0 || height <= 0 {
            return Err(Box::new(ImgError::new_const(ImgErrorKind::SizeZero,"image size zero or minus".to_string())))
        }
        if self.width() == 0 || self.height() == 0 {
            let buffersize = width as usize * height as usize * 4;
            self.width = width as u32;
            self.height = height as u32;
            self.buffer = (0..buffersize).map(|_| 0).collect();
        }
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
