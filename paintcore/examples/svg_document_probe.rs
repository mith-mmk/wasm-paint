//! Render a standalone SVG through the private paintcore adapter without font loading.
#[cfg(feature = "svg-font")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use paintcore::image_compat::{ImageFormat, RgbaImage};
    use paintcore::prelude::*;
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        return Err(
            "usage: svg_document_probe SVG OUTPUT_PNG SCALE Y_ORIGIN (256x256 canvas)".into(),
        );
    }
    let scale = args[3].parse()?;
    let mut canvas = Layer::tmp(256, 256);
    draw_svg_document(
        &mut canvas,
        &std::fs::read_to_string(&args[1])?,
        SvgRenderTransform {
            viewport_units_to_pixels: [scale, 0., 0., scale, 0., 0.],
        },
        0.,
        args[4].parse()?,
        0xff000000,
    )?;
    let image = RgbaImage::from_raw(canvas.width(), canvas.height(), canvas.buffer().to_vec())?;
    std::fs::write(&args[2], image.encode_to_vec(ImageFormat::Png)?)?;
    Ok(())
}
#[cfg(not(feature = "svg-font"))]
fn main() {
    eprintln!("Enable --features svg-font");
}
