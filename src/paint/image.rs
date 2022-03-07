use crate::Canvas;
use crate::log;
use crate::img::error::ImgError;
use crate::img::error::ErrorKind;
use crate::img::*;

struct Drawer<'a> {
  pub canvas: &'a mut Canvas,
  pub image_width: usize,
  pub image_height: usize,
}

impl DrawCallback for Drawer<'_> {
  fn init(any: &mut Dynamic,width: usize,height: usize) -> Result<Option<isize>,ImgError> {
    let own = any.downcast_mut::<Drawer>().ok_or(ImgError::Simple(ErrorKind::IlligalCallback))?;
    own.image_width = width as usize;
    own.image_height = height as usize;
    Ok(None)
  }
  
  fn draw(any: &mut Dynamic,start_x: usize, start_y: usize, width: usize, height: usize, data: &[u8])
  -> Result<Option<isize>,ImgError>  {
    let own = any.downcast_mut::<Drawer>().ok_or(ImgError::Simple(ErrorKind::IlligalCallback))?;
  
    if start_x >= own.canvas.width() as usize || start_y >= own.canvas.height() as usize {return Ok(None);}
    let w = if (own.canvas.width() as usize) < width + start_x {
        own.canvas.width() as usize - start_x 
      } else { width };
    let h = if (own.canvas.height() as usize) < height + start_y {
        own.canvas.height() as usize - start_y
      } else { height };

    let dest_width = own.canvas.width() as usize;
    let buffer = &mut own.canvas.buffer;

    for y in 0..h {
        let src_scanline =  y * width * 4;
        let dest_scanline = (y + start_y) * dest_width * 4;
        for x in 0..w {
            let src_offset = src_scanline + x * 4;
            let dest_offset = dest_scanline + (x + start_x) * 4;
            buffer[dest_offset] = data[src_offset];
            buffer[dest_offset + 1] = data[src_offset + 1];
            buffer[dest_offset + 2] = data[src_offset + 2];
            buffer[dest_offset + 3] = 0xff;
        }
    }
    if start_x == 0 && start_y == 0 {
      log(&format!("{:>02X} {:>02X} {:>02X}",buffer[0],buffer[1],buffer[3]));
    }
    Ok(None)
  }
  
  fn terminate(_: &mut Dynamic) -> Result<Option<isize>,ImgError> {
    log("callback terminate");
    Ok(None)
  }
  
  fn next(_: &mut Dynamic, _: Vec<u8>) -> Result<Option<isize>,ImgError> {
    Ok(None) 
  }
}

impl Drop for Drawer<'_> {
  fn drop(&mut self) { //

  }
}


impl Drawer<'_> {
  pub fn new (canvas: &'static mut Canvas) -> Self {
    Self {
      canvas: canvas,
      image_width: 0,
      image_height: 0,
    }
  }
}

pub fn draw_image (canvas:&mut Canvas,data: &[u8]) {
  let mut drawer = ImageBuffer::new();
  let callback = Callback::new();
  let mut option = DecodeOptions{
    debug_flag: 0,
    drawer: &mut drawer,
    callback: callback,
  };

  let r = crate::img::jpeg::decoder::decode(data, &mut option);
  match r {
    Err(error) => {
      log(&error.fmt());
      return
    },
    Ok(worning) => {
      match worning  {
        Some(worning) => {
          log(&worning.fmt());
        },
        _ => {}
      }
    },
  }

  let buf = drawer.buffer.unwrap();

  log(&format!("{} {}",drawer.width,drawer.height));

  for y in 0..drawer.height {
    if y >= canvas.height() as usize { break;}
    let scanline_src = y * drawer.width * 4;
    let scanline_dest = y * canvas.width() as usize * 4;
    for x in 0..drawer.width {
      if x >= canvas.width() as usize { break;}
      let offset_src = scanline_src + x * 4;
      let offset_dest= scanline_dest + x * 4;
          canvas.buffer[offset_dest] = buf[offset_src];
          canvas.buffer[offset_dest + 1] = buf[offset_src + 1];
          canvas.buffer[offset_dest + 2] = buf[offset_src + 2];
          canvas.buffer[offset_dest + 3] = 0xff;
    }
  }


}

