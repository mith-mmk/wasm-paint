/*
 * canvas.rs  Mith@mmk (C) 2022
 * update 2022/03/13
 */

use super::pen::Pen;
use super::clear::fillrect;

pub trait Screen {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    
    // for WebAssembly
    fn canvas(&self) -> *const u8;
    fn set_buckground_color(&mut self,color: u32);
    fn background_color(&self) -> u32;
    fn color(&self) -> u32;
    fn set_color(&mut self,color: u32);
    fn clear(&mut self);
    fn clear_with_color(&mut self,color: u32);
}

pub struct Canvas {
    pub buffer: Vec<u8>,
    width: u32,
    height: u32,
    color: u32,
    background_color: u32,
    pen: Pen,
}

impl Canvas {
    pub fn new (width: u32, height: u32) -> Self {
        if width == 0 || width >= 0x8000000 || height == 0 || height >= 0x8000000 {
            return Self {
                buffer: Vec::new(),
                width: 0,
                height: 0,
                color: 0x00ffffff,
                background_color: 0,
                pen: Pen::new(1, 1, vec![255]),
            }
        }
        let buffersize = width * height * 4;
        let buffer = (0..buffersize)
            .map(|_| {0})
            .collect();
        let pen = Pen::new(1, 1, vec![255]);
        let color = 0xfffff;
        let background_color = 0;


        Self {
            buffer,
            width,
            height,
            color,
            background_color,
            pen,
        }
    }

    pub fn set_pen(&mut self,pen :Pen) {
        self.pen = pen;
    }

    pub fn pen(&self) -> &Pen{
        &self.pen
    }

}

impl Screen for Canvas {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    // for WebAssembly
    fn canvas(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    fn set_buckground_color(&mut self,color: u32) {
        self.background_color = color;
    }

    fn background_color(&self) -> u32 {
        self.background_color
    }

    fn color(&self) -> u32 {
        self.color
    }

    fn set_color(&mut self,color: u32) {
        self.color = color;
    }

    fn clear(&mut self) {
        self.clear_with_color(self.background_color);
    }

    fn clear_with_color(&mut self,color: u32) {
        fillrect(self,color);  
    }
}

