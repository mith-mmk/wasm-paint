use wasm_bindgen::prelude::*;
use paintcore::prelude::*;
use wml2::draw::{image_loader, DecodeOptions};

const MAIN_LAYER: &str = "main";

#[wasm_bindgen]
pub struct UniverseFast {
    canvas: Canvas,
}

#[wasm_bindgen]
impl UniverseFast {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> UniverseFast {
        UniverseFast {
            canvas: canvas_with_main_layer(width, height),
        }
    }

    #[wasm_bindgen(js_name = resize)]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.canvas = canvas_with_main_layer(width, height);
    }

    #[wasm_bindgen(js_name = clear)]
    pub fn clear(&mut self) {
        clear_main_layer(&mut self.canvas);
        self.canvas.combine();
    }

    #[wasm_bindgen(js_name = decode)]
    pub fn decode(&mut self, buffer: &[u8]) -> bool {
        clear_main_layer(&mut self.canvas);

        let mut options = DecodeOptions {
           debug_flag: 0,
            drawer: &mut self.canvas,
        };

        let ok = image_loader(buffer, &mut options).is_ok();
        self.canvas.combine();
        ok
    }

    #[wasm_bindgen(js_name = ptr)]
    pub fn ptr(&self) -> *const u8 {
        self.canvas.buffer().as_ptr()
    }

    #[wasm_bindgen(js_name = len)]
    pub fn len(&self) -> usize {
        self.canvas.buffer().len()
    }

    #[wasm_bindgen(js_name = width)]
    pub fn width(&self) -> u32 {
        self.canvas.width()
    }

    #[wasm_bindgen(js_name = height)]
    pub fn height(&self) -> u32 {
        self.canvas.height()
    }
}

fn canvas_with_main_layer(width: u32, height: u32) -> Canvas {
    let mut canvas = Canvas::new(width, height);
    canvas
        .add_layer(MAIN_LAYER.to_string(), width, height, 0, 0)
        .expect("main layer should be initialized");
    canvas.set_current(MAIN_LAYER.to_string());
    canvas
}

fn clear_main_layer(canvas: &mut Canvas) {
    canvas
        .clear_layer(MAIN_LAYER.to_string())
        .expect("main layer should exist");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn universe_fast_initializes_main_layer() {
        let universe = UniverseFast::new(4, 4);

        assert_eq!(universe.canvas.layers_len(), 1);
        assert_eq!(universe.canvas.current(), MAIN_LAYER);
    }

    #[test]
    fn universe_fast_resize_recreates_main_layer() {
        let mut universe = UniverseFast::new(4, 4);

        universe.resize(8, 6);

        assert_eq!(universe.canvas.layers_len(), 1);
        assert_eq!(universe.canvas.current(), MAIN_LAYER);
        assert_eq!(universe.width(), 8);
        assert_eq!(universe.height(), 6);
    }
}
