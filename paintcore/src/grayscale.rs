//! grayscaling and output other screen
/*
 * galyscale.rs  Mith@mmk (C) 2022
 *
 */

use crate::canvas::Screen;

pub enum Weights {
    Jpeg,
    Bt601,
    Bt709,
    Average,
    RedOnly,
    GreenOnly,
    BlueOnly,
}

pub fn get_weight(weight: Weights) -> (f64, f64, f64) {
    match weight {
        Weights::Jpeg => (0.299_f64, 0.587_f64, 0.114_f64),
        Weights::Bt601 => (0.299_f64, 0.587_f64, 0.114_f64),
        Weights::Bt709 => (0.2126_f64, 0.7152_f64, 0.0722_f64),
        Weights::Average => (0.3333333_f64, 0.3333334_f64, 0.3333333_f64),
        Weights::RedOnly => (1.0_f64, 0.0_f64, 0.0_f64),
        Weights::GreenOnly => (0.0_f64, 1.0_f64, 0.0_f64),
        Weights::BlueOnly => (0.0_f64, 0.0_f64, 1.0_f64),
    }
}

pub fn weight(t: usize) -> (f64, f64, f64) {
    match t {
        0 => get_weight(Weights::Bt601),
        1 => get_weight(Weights::Bt709),
        2 => get_weight(Weights::Average),
        3 => get_weight(Weights::RedOnly),
        4 => get_weight(Weights::GreenOnly),
        5 => get_weight(Weights::BlueOnly),
        _ => get_weight(Weights::Jpeg),
    }
}

pub fn to_grayscale(src: &dyn Screen, dest: &mut dyn Screen, t: usize) {
    if dest.width() == 0 || dest.height() == 0 {
        dest.reinit(src.width(), src.height());
        if dest.width() == 0 || dest.height() == 0 {
            return;
        }
    }
    let width = (src.width() as usize).min(dest.width() as usize);
    let height = (src.height() as usize).min(dest.height() as usize);

    let src_buffer = src.buffer();
    let dest_width = dest.width() as usize;
    let dest_buffer = dest.buffer_mut();
    let (wred, wgreen, wblue) = weight(t);
    for y in 0..height {
        for x in 0..width {
            let src_offset = (y * src.width() as usize + x) * 4;
            let dest_offset = (y * dest_width + x) * 4;
            let r = src_buffer[src_offset] as f64;
            let g = src_buffer[src_offset + 1] as f64;
            let b = src_buffer[src_offset + 2] as f64;
            let a = src_buffer[src_offset + 3];
            let l = ((wred * r + wgreen * g + wblue * b).round() as i16).clamp(0, 255) as u8;

            dest_buffer[dest_offset] = l;
            dest_buffer[dest_offset + 1] = l;
            dest_buffer[dest_offset + 2] = l;
            dest_buffer[dest_offset + 3] = a;
        }
    }
}
