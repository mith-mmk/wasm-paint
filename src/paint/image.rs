/* image.rs
 * (C)2022 Mith@mmk 
 */

use crate::img::jpeg::worning::JPEGWorning;
use super::canvas::*;
use crate::img::error::ImgError;
use crate::img::*;


pub fn draw_image (canvas:&mut Canvas,data: &[u8],verbose:usize) -> Result<Option<JPEGWorning>,ImgError> {

//  canvas.set_verbose(write_log);

  let mut option = DecodeOptions{
    debug_flag: verbose,
    drawer: canvas,
  };

  let r = crate::img::jpeg::decoder::decode(data, &mut option);
  match r {
    Err(error) => {
      return Err(error)
    },
    _ => {},
  }

  r
}

