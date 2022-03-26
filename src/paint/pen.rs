use crate::paint::point::point_with_weight;
use crate::Canvas;

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

    pub fn square_pen (size:i32) -> Option<Self> {
        if size <= 0 { return None}
        let width = size as u32;
        let height = size as u32;
        let buffer = (0..size*size).map(|_| 0xff).collect();

        Some(Self {
            buffer,
            width,
            height,
        })
    }

    pub fn default_pen () -> Self {
        let size = 3;
        let width = size;
        let height = size;
        let buffer = [0x59,0x7f,0x59
                     ,0x7f,0xff,0x7f
                     ,0x59,0x7f,0x59].to_vec();

        Self {
            buffer,
            width,
            height,
        }
    }

    pub fn pen(&self) -> &[u8] {
        &self.buffer
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

pub fn point_pen(canvas:&mut Canvas,x :i32,y :i32,color :u32) {
    
    let width = canvas.pen().width();
    let height = canvas.pen().height();

    let mut py = - (height  as i32) / 2;

    for _y in 0..height {
        let mut px = - (width as i32) / 2;
        for _x in 0..width {
            let weight = 255.0 / canvas.pen().buffer[(_y * width + _x) as usize] as f32;
            point_with_weight(canvas,x + px ,y + py,color,weight);
            px += 1;
        }
        py += 1;
    }
}
