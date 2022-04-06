//! image is decoding enconded image and draw.
/* image.rs
 * (C)2022 Mith@mmk 
 */
type Error = Box<dyn std::error::Error>;

use wml2::warning::ImgWarnings;
use wml2::draw::{image_loader, DecodeOptions, DrawCallback};

pub fn draw_image (canvas:&mut (dyn DrawCallback + Sync + Send),data: &[u8],verbose:usize) -> Result<Option<ImgWarnings>,Error> {

//  canvas.set_verbose(write_log);

  let mut option = DecodeOptions{
    debug_flag: verbose,
    drawer: canvas,
  };

  image_loader(data, &mut option)
}

