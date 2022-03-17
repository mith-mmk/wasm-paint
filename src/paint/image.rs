/* image.rs
 * (C)2022 Mith@mmk 
 */


use wml2::error::ImgError;
use wml2::DecodeOptions;
use wml2::jpeg::worning::JPEGWorning;
use wml2::jpeg::decoder::decode as jpeg_decoder;
use super::canvas::*;

pub fn draw_image (canvas:&mut Canvas,data: &[u8],verbose:usize) -> Result<Option<JPEGWorning>,ImgError> {

//  canvas.set_verbose(write_log);

  let mut option = DecodeOptions{
    debug_flag: verbose,
    drawer: canvas,
  };

  let r = jpeg_decoder(data, &mut option);
  match r {
    Err(error) => {
      return Err(error)
    },
    _ => {},
  }

  r
}

