use crate::paint::canvas::Screen;
use wasm_bindgen::JsCast;
use crate::img::jpeg::worning::JPEGWorning;
use crate::Canvas;
use crate::log;
use crate::img::error::ImgError;
use crate::img::*;
use web_sys::HtmlElement;

pub fn write_log(str: &str) -> Result<Option<isize>,ImgError> {
  match web_sys::window() {
    Some(window) => {
      match window.document() {
        Some(document) => {
          match document.get_element_by_id("wasm_verbose"){
            Some(elmid) => {
              match elmid.dyn_ref::<HtmlElement>(){
                Some(elm) =>{
                  elm.set_inner_text(str);
                  return Ok(None)
                },
                _  => {},
              }
            },
            _ =>{}
          }
        },
        _ =>{
        }
      }
    },
    _ => {
    }
  }
  log(str);
  Ok(None)
}

pub fn draw_image (canvas:&mut Canvas,data: &[u8],verbose:usize) -> Result<Option<JPEGWorning>,ImgError> {
  let mut drawer = ImageBuffer::new();
  drawer.set_verbose(write_log);

  let mut option = DecodeOptions{
    debug_flag: verbose,
    drawer: &mut drawer,

  };

  let r = crate::img::jpeg::decoder::decode(data, &mut option);
  match r {
    Err(error) => {
      return Err(error)
    },
    _ => {},
  }

  let buf = drawer.buffer.unwrap();

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

  r
}

