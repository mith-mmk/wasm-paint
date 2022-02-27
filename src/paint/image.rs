use super::canvas::Canvas;

pub fn load_image (url: &str,x: i32,y: i32) -> Canvas {
  let canvas = Canvas::new(x as u32, y as u32);

  canvas
}