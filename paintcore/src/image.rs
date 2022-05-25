//! image is decoding enconded image and draw.
/* image.rs
 * (C)2022 Mith@mmk 
 */
type Error = Box<dyn std::error::Error>;

use crate::canvas::Canvas;
use crate::affine::Affine;
use crate::affine::InterpolationAlgorithm;
use crate::canvas::Screen;

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

pub fn draw_image_fit_screen (screen:&mut dyn Screen,data: &[u8],interop:Option<InterpolationAlgorithm>,align: ImageAlign) -> Result<Canvas,Error> {

    let interop =  if let Some(interop) = interop { interop } else {InterpolationAlgorithm::Bilinear};

    let mut image_buffer = Canvas::new(0, 0);
    image_buffer.add_layer("temp".to_string(), 0, 0, 0, 0)?;
    image_buffer.set_current("temp".to_string());
  
    let mut option = DecodeOptions{
      debug_flag: 0,
      drawer: &mut image_buffer,
    };
  
    let _ = image_loader(data, &mut option)?;

    let mut scale = 1.0;
    if screen.width() < image_buffer.width() {
      scale = screen.width() as f32 / image_buffer.width() as f32;
    } 
    if (screen.height() as f32) < image_buffer.height() as f32 * scale {
      scale =  screen.height() as f32 / image_buffer.height() as f32;
    }

    Affine::resize(image_buffer.layer("temp".to_string()).unwrap(),screen,scale,interop,align);
    Ok(image_buffer)
  }
  
  