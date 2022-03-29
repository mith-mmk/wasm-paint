use crate::paint::clear::fillrect_with_alpha;
use super::canvas::*;


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
        }
    }
    pub fn move_layer(&mut self,dx:i32,dy:i32) {
        self.x += dx;
        self.y += dy;
    }

}

impl Screen for Layer {
    fn width(&self) -> u32 {
        self.width.clone()
    }

    fn height(&self) -> u32 {
        self.height.clone()
    }


    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn buffer_as_mut(&mut self) -> &mut [u8] {
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