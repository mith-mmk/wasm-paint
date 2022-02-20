use crate::paint::pen::Pen;
use crate::paint::clear::fillrect;

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

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_pen(&mut self,pen :Pen) {
        self.pen = pen;
    }

    pub fn pen(&self) -> &Pen{
        &self.pen
    }

    // for WebAssembly
    pub fn canvas(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    pub fn set_buckground_color(&mut self,color: u32) {
        self.background_color = color;
    }

    pub fn background_color(&self) -> u32 {
        self.background_color
    }

    pub fn color(&self) -> u32 {
        self.color
    }

    pub fn set_color(&mut self,color: u32) {
        self.color = color;
    }

    pub fn clear(&mut self) {
        self.clear_with_color(self.background_color);
    }

    pub fn clear_with_color(&mut self,color: u32) {
        fillrect(self,color);  
    }

}

