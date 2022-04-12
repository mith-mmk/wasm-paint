//! image is decoding enconded image and draw.
/* image.rs
 * (C)2022 Mith@mmk 
 */
type Error = Box<dyn std::error::Error>;

use crate::Affine;
use crate::InterpolationAlgorithm;
use crate::Screen;
use crate::Layer;


pub enum ImageAlign {
  Default,
  Center,
  RightUp,
  RightBottom,
  LeftUp,
  LeftBottom,
  Right,
  Left,
  Up,
  Bottom,
}

use wml2::warning::ImgWarnings;
use wml2::draw::{image_loader, DecodeOptions, DrawCallback};

pub fn draw_image(screen:&mut (dyn DrawCallback + Sync + Send),data: &[u8],verbose:usize) -> Result<Option<ImgWarnings>,Error> {

//  canvas.set_verbose(write_log);

  let mut option = DecodeOptions{
    debug_flag: verbose,
    drawer: screen,
  };

  image_loader(data, &mut option)
}

pub fn draw_image_fit_screen (screen:&mut dyn Screen,data: &[u8],interop:Option<InterpolationAlgorithm>,align: ImageAlign) -> Result<Option<ImgWarnings>,Error> {

    let interop =  if let Some(interop) = interop { interop } else {InterpolationAlgorithm::Bilinear};

    let mut image_buffer = Layer::new("temp".to_string(), 0, 0);
  
    let mut option = DecodeOptions{
      debug_flag: 0,
      drawer: &mut image_buffer,
    };
  
    let warnings = image_loader(data, &mut option)?;

    let mut scale = 1.0;
    if screen.width() < image_buffer.width() {
      scale = screen.width() as f32 / image_buffer.width() as f32;
    } 
    if (screen.height() as f32) < image_buffer.height() as f32 * scale {
      scale =  screen.height() as f32 / image_buffer.height() as f32;
    }

    Affine::resize(&image_buffer,screen,scale,interop,align);
    Ok(warnings)
  }
  
  