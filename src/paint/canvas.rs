/*
 * canvas.rs  Mith@mmk (C) 2022
 * update 2022/03/13
 */

use crate::img::error::ImgError::SimpleAddMessage;
use crate::img::error::ImgError;
use crate::img::error::*;
use crate::img::DrawCallback;
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
    fnverbose: fn(&str) -> Result<Option<isize>,ImgError>,
    draw_width: u32,
    draw_height: u32,
}

fn default_verbose(_ :&str) -> Result<Option<isize>, ImgError>{
    Ok(None)
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
                fnverbose: default_verbose,
                draw_width: 0,
                draw_height: 0,
            }
        }
        let buffersize = width * height * 4;
        let buffer = (0..buffersize)
            .map(|_| {0})
            .collect();
        let pen = Pen::new(1, 1, vec![255]);
        let color = 0xfffff;
        let background_color = 0;
        let fnverbose = default_verbose;

        Self {
            buffer,
            width,
            height,
            color,
            background_color,
            pen,
            fnverbose,
            draw_width: 0,
            draw_height: 0,
        }
    }

    pub fn new_in (buffer: Vec<u8>,width: u32, height: u32) -> Self {
        if width == 0 || width >= 0x8000000 || height == 0 || height >= 0x8000000 {
            return Self {
                buffer: Vec::new(),
                width: 0,
                height: 0,
                color: 0x00ffffff,
                background_color: 0,
                pen: Pen::new(1, 1, vec![255]),
                fnverbose: default_verbose,
                draw_width: 0,
                draw_height: 0,
            }
        }
        let pen = Pen::new(1, 1, vec![255]);
        let color = 0xfffff;
        let background_color = 0;
        let fnverbose = default_verbose;

        Self {
            buffer,
            width,
            height,
            color,
            background_color,
            pen,
            fnverbose,
            draw_width: 0,
            draw_height: 0,
        }
    }
    pub fn set_pen(&mut self,pen :Pen) {
        self.pen = pen;
    }

    pub fn pen(&self) -> &Pen{
        &self.pen
    }

    pub fn set_verbose(&mut self,verbose:fn(&str) -> Result<Option<isize>,ImgError>) {
        self.fnverbose = verbose;
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


impl DrawCallback for Canvas {
    fn init(&mut self, width: usize, height: usize) -> Result<Option<isize>, ImgError> {
        self.draw_width = width as u32;
        self.draw_height = height as u32;
        Ok(None)
    }

    fn draw(&mut self, start_x: usize, start_y: usize, width: usize, height: usize, data: &[u8])
                -> Result<Option<isize>,ImgError>  {
        let self_width = self.width as usize;
        let self_height = self.height as usize;

        let buffer =  &mut self.buffer;
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
                    return Err(SimpleAddMessage(ErrorKind::OutboundIndex,format!("decoder buffer in draw {}",data.len())))
                }
                buffer[offset_dest    ] = data[offset_src];
                buffer[offset_dest + 1] = data[offset_src + 1];
                buffer[offset_dest + 2] = data[offset_src + 2];
                buffer[offset_dest + 3] = data[offset_src + 3];
            }
        }
        Ok(None)
    }

    fn terminate(&mut self) -> Result<Option<isize>, ImgError> {
        Ok(None)
    }

    fn next(&mut self, _: std::vec::Vec<u8>) -> Result<Option<isize>, ImgError> {
        Ok(None)
    }

    fn verbose(&mut self, str: &str) -> Result<Option<isize>, ImgError> { 
        return (self.fnverbose)(str);
    }
}