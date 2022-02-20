#[derive(Debug,Clone)]
pub struct Pen {
    buffer: Vec<u8>,
    width : u32,
    height: u32,
}

impl Pen {
    pub fn new (width: u32, height: u32,buffer: Vec<u8>) -> Self {
        Self {
            buffer,
            width,
            height,
        }
    }

    pub fn pen(&self) -> *const u8 {
        self.buffer.as_ptr()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

}
