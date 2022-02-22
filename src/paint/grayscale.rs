use super::super::paint::canvas::Canvas;

pub enum Weights {
    Jpeg,
    Bt601,
    Bt709,
    Average,
    RedOnly,
    GreenOnly,
    BlueOnly,
}

pub fn get_weight(weight: Weights) -> (f64,f64,f64) {
    match weight {
        Weights::Jpeg=> return (0.299_f64, 0.587_f64, 0.114_f64),
        Weights::Bt601 => return (0.299_f64, 0.587_f64, 0.114_f64),
        Weights::Bt709 => return (0.2126_f64, 0.7152_f64, 0.0722_f64),
        Weights::Average => return  (0.3333333_f64,0.3333334_f64,0.3333333_f64), 
        Weights::RedOnly => return (1.0_f64,0.0_f64,0.0_f64),
        Weights::GreenOnly => return  (0.0_f64,1.0_f64,0.0_f64),
        Weights::BlueOnly => return (0.0_f64,0.0_f64,1.0_f64),
    }
}

pub fn weight(t: usize) -> (f64,f64,f64) {
    match t {
        0 => return get_weight(Weights::Bt601),
        1 => return get_weight(Weights::Bt709),
        2 => return get_weight(Weights::Average),
        3 => return get_weight(Weights::RedOnly),
        4 => return get_weight(Weights::GreenOnly),
        5 => return get_weight(Weights::BlueOnly),
        _ => return get_weight(Weights::Jpeg)
    }
}

pub fn to_grayscale(input: &Canvas, output: &mut Canvas, t: usize) {
    let height = output.height();
    let width = output.width();
    let ibuf = &input.buffer;
    let buf = &mut output.buffer;
    let (wred, wgreen, wblue)  = weight(t);
    for y in 0..height {
        let offset = y * width * 4;
        for x  in 0..width {
            let pos = (offset + (x * 4)) as usize;
            let blue = ibuf[pos + 2] as f64;
            let green  = ibuf[pos + 1] as f64;
            let red = ibuf[pos] as f64;

            let gray =  (wred * red + wgreen * green  + wblue * blue).round() as u8;
            buf[pos] = gray;     // Red
            buf[pos + 1] = gray; // Green
            buf[pos + 2] = gray; // Blue
            buf[pos + 3] = 0xff; // alpha
        }
    }
}