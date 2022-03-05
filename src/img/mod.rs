pub mod io;
pub mod jpeg;
pub mod tiff;
pub mod error;
pub mod util;

use self::util::*;


#[allow(unused)]
pub trait DefaultCallback {
    fn init (&mut self,width: usize,height: usize){

    }
    fn draw (&mut self,start_x: usize,start_y: usize,width: usize,height: usize,data: &[u8]){

    } 
    fn next (&mut self,option: Vec<u8>){

    }

    fn terminate (&mut self){

    }
}

pub struct ImageBuffer {
    width: usize,
    height: usize,
    pub buffer: Option<Vec<u8>>,
}

impl ImageBuffer {
    pub fn new () -> Self {
        Self {
            width: 0,
            height: 0,
            buffer: None,
        }
    }
}

impl DefaultCallback for ImageBuffer {
    fn init (self: &mut Self,width: usize,height: usize) {
        let buffersize = width * height * 4;
        self.width = width;
        self.height = height;
        self.buffer = Some((0 .. buffersize).map(|_| 0).collect());
        debug_println!("{}x{} Buffer",width,height);
    }

    fn draw(self: &mut Self,start_x: usize, start_y: usize, width: usize, height: usize, data: &[u8]) {
        if start_x >= self.width || start_y >= self.height {return;}
        let w = if self.width < width + start_x {self.width - start_x} else { width };
        let h = if self.height < height + start_y {self.height - start_y} else { height };
        for y in 0..h {
            let scanline =  y * width * 4;
            for x in 0..w {
                let offset = scanline + x * 4;
                print!("{:>02x} ",&data[offset]);
                print!("{:>02x} ",&data[offset+1]);
                print!("{:>02x}  ",&data[offset+2]);
            }
            println!("");
        }

    }

    fn terminate(&mut self) {  }
    fn next(&mut self, _: Vec<u8>) {  }
}

pub struct DecodeOptions {
    pub debug_flag: usize,
    pub callback: ImageBuffer,
}