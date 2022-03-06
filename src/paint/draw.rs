use super::canvas::Canvas;
use super::utils::calc_alphablend;

pub fn canvas_to_canvas(src:&Canvas,dest:&mut Canvas, dest_x: u32, dest_y: u32) {
    let width = if dest_x + src.width() < dest.width() { src.width() } else { dest.width() };
    let height = if dest_y + src.height() < dest.height() { src.height() } else { dest.height() };
    let dest_width = dest.width();
    let srcbuf = &src.buffer;
    let destbuf = &mut dest.buffer; // move ownership dest -> destbuf because use mut

    for y in 0..height {
        let src_offset = y * src.width() * 4;
        let dest_offset = (y + dest_y) * dest_width * 4;
        for x  in 0..width {
            let src_pos = (src_offset + x * 4) as usize;
            let dest_pos = (dest_offset + (x * dest_x )* 4) as usize;

            destbuf[dest_pos    ] = srcbuf[src_pos    ];  // Red
            destbuf[dest_pos + 1] = srcbuf[src_pos + 1]; // Green
            destbuf[dest_pos + 2] = srcbuf[src_pos + 2]; // Blue
            destbuf[dest_pos + 3] = 0xff; // alpha
        }
    }
}


pub fn canvas_to_canvas_with_alpha(src:&Canvas,dest:&mut Canvas, dest_x: u32, dest_y: u32) {
    let width = if dest_x + src.width() < dest.width() { src.width() } else { dest.width() };
    let height = if dest_y + src.height() < dest.height() { src.height() } else { dest.height() };
    let dest_width = dest.width();

    let srcbuf = &src.buffer;
    let destbuf = &mut dest.buffer; // move ownership dest -> destbuf because use mut

    for y in 0..height {
        let src_offset = y * src.width() * 4;
        let dest_offset = (y + dest_y) * dest_width * 4;
        for x  in 0..width {
            let src_pos = (src_offset + x * 4) as usize;
            let dest_pos = (dest_offset + (x * dest_x )* 4) as usize;
            let alpha = srcbuf[src_pos + 2] as f32 / 255.0;

            destbuf[dest_pos    ] = calc_alphablend(srcbuf[src_pos    ],destbuf[dest_pos    ],alpha);
            destbuf[dest_pos + 1] = calc_alphablend(srcbuf[src_pos    ],destbuf[dest_pos    ],alpha);
            destbuf[dest_pos + 2] = calc_alphablend(srcbuf[src_pos    ],destbuf[dest_pos    ],alpha);
            destbuf[dest_pos + 3] = 0xff; // alpha
        }
    }
}