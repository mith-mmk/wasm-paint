pub mod io;
pub mod jpeg;
pub mod tiff;
pub mod error;
pub mod util;

use crate::log;
use crate::img::error::ImgError;
use crate::img::error::ErrorKind;
use core::any::Any;
use self::util::*;

/* Dynamic Select Callback System */

pub type Dynamic = (dyn Any + Send + Sync);
pub type FnInit = fn(&mut Dynamic,usize,usize) -> Result<Option<isize>,ImgError>;
pub type FnDraw = fn(&mut Dynamic,usize,usize,usize,usize,&[u8]) -> Result<Option<isize>,ImgError>;
pub type FnNext = fn(&mut Dynamic,Vec<u8>) -> Result<Option<isize>,ImgError>;
pub type FnTerminate = fn(&mut Dynamic) -> Result<Option<isize>,ImgError>;

pub trait DrawCallback {
    fn init(any: &mut Dynamic,width: usize,height: usize) -> Result<Option<isize>,ImgError>;
    fn draw(any: &mut Dynamic,start_x: usize, start_y: usize, width: usize, height: usize, data: &[u8])
             -> Result<Option<isize>,ImgError>;
    fn terminate(any: &mut (dyn Any + Send + Sync)) -> Result<Option<isize>,ImgError>;
    fn next(any: &mut (dyn Any + Send + Sync), _: Vec<u8>) -> Result<Option<isize>,ImgError>;
}

#[allow(unused)]
pub struct ImageBuffer {
    pub width: usize,
    pub height: usize,
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

#[allow(unused)]
pub struct Callback {
    init: FnInit,
    draw: FnDraw,
    next: FnNext,
    terminate: FnTerminate,
}

#[allow(unused)]
impl Callback {
    pub fn new() -> Self {
        Self {
            init: Self::default_init,
            draw: Self::default_draw,
            next: Self::default_next,
            terminate: Self::default_terminate,
        }
    }

    pub fn set_init(self:&mut Self,init :FnInit) {
        self.init = init;
    }
    pub fn set_draw(self:&mut Self,draw :FnDraw) {
        self.draw = draw;
    }
    pub fn set_next(self:&mut Self,next :FnNext) {
        self.next = next;
    }
    pub fn set_terminate(self:&mut Self,terminate :FnTerminate) {
        self.terminate = terminate;
    }


    fn default_init(any: &mut Dynamic,width: usize,height: usize) -> Result<Option<isize>,ImgError> {
        let own = any.downcast_mut::<ImageBuffer>().ok_or(ImgError::Simple(ErrorKind::IlligalCallback))?;
        let buffersize = width * height * 4;
        own.width = width;
        own.height = height;
        own.buffer = Some((0 .. buffersize).map(|_| 0).collect());
        Ok(None)
    }

    fn default_draw(any: &mut Dynamic,start_x: usize, start_y: usize, width: usize, height: usize, data: &[u8])
    -> Result<Option<isize>,ImgError>  {
        let own = any.downcast_mut::<ImageBuffer>().ok_or(ImgError::Simple(ErrorKind::IlligalCallback))?;
        let mut buffer =  own.buffer.as_deref_mut().unwrap();
        if start_x >= own.width || start_y >= own.height {return Ok(None);}
        let w = if own.width < width + start_x {own.width - start_x} else { width };
        let h = if own.height < height + start_y {own.height - start_y} else { height };
        for y in 0..h {
            let scanline_src =  y * width * 4;
            let scanline_dest= (start_y + y) * own.width * 4;
            for x in 0..w {
                let offset_src = scanline_src + x * 4;
                let offset_dest = scanline_dest + (x + start_x) * 4;
                buffer[offset_dest    ] = data[offset_src];
                buffer[offset_dest + 1] = data[offset_src + 1];
                buffer[offset_dest + 2] = data[offset_src + 2];
                buffer[offset_dest + 3] = data[offset_src + 3];

            }
        }
        Ok(None)
    }

    fn default_terminate(any: &mut Dynamic) -> Result<Option<isize>,ImgError> {
        Ok(None)
    }

    fn default_next(any: &mut Dynamic, _: Vec<u8>) -> Result<Option<isize>,ImgError> {
        Ok(None) 
    }
}

pub struct DecodeOptions<'a> {
    pub debug_flag: usize,
    pub drawer: &'a mut Dynamic,
    pub callback: Callback,
}