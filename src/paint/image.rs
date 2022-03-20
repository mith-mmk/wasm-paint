/* image.rs
 * (C)2022 Mith@mmk 
 */

use wml2::error::ImgError;
use wml2::warning::ImgWarning;
use wml2::draw::*;
use super::canvas::*;

pub fn draw_image (canvas:&mut Canvas,data: &[u8],verbose:usize) -> Result<Option<ImgWarning>,ImgError> {

//  canvas.set_verbose(write_log);

  let mut option = DecodeOptions{
    debug_flag: verbose,
    drawer: canvas,
  };

  let r = image_decoder(data, &mut option);
  match r {
    Err(error) => {
      return Err(error)
    },
    Ok(..) => {
    },
  }

  r
}

