//! Render a local font in horizontal, standard vertical, mixed and sideways modes.
//! cargo run -p paintcore --features font --example vertical_text -- FONT OUTPUT.png

#[cfg(feature = "font")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use paintcore::image_compat::{ImageFormat, RgbaImage};
    use paintcore::prelude::*;

    let mut args = std::env::args().skip(1);
    let font_path = args
        .next()
        .ok_or("usage: vertical_text FONT OUTPUT.png [TEXT]")?;
    let output = args
        .next()
        .ok_or("usage: vertical_text FONT OUTPUT.png [TEXT]")?;
    let text = args
        .next()
        .unwrap_or_else(|| "日本語ABC 123「縦書き」。".into());
    let face = load_font_from_buffer(&std::fs::read(font_path)?)?;
    let options = FontOptions::new(&face).with_font_size(36.0);
    let engine = face.engine().with_options(options.clone());
    let mut canvas = Layer::tmp(800, 850);
    fillrect(&mut canvas, 0x00ff_ffff);
    draw_text_with_options(&mut canvas, &text, options.clone(), 40.0, 65.0, 0xff20_2020)?;
    draw_text_with_options(
        &mut canvas,
        &text,
        options.with_vertical_flow(),
        150.0,
        120.0,
        0xff20_4080,
    )?;
    draw_glyphs(
        &mut canvas,
        &layout_text_vertical_mixed_ltr(&text, &engine)?,
        400.0,
        120.0,
        0xff20_8040,
    )?;
    draw_glyphs(
        &mut canvas,
        &layout_text_sideways_ltr(&text, &engine)?,
        650.0,
        120.0,
        0xff80_4020,
    )?;
    let image = RgbaImage::from_raw(canvas.width(), canvas.height(), canvas.buffer().to_vec())?;
    std::fs::write(output, image.encode_to_vec(ImageFormat::Png)?)?;
    Ok(())
}

#[cfg(not(feature = "font"))]
fn main() {
    eprintln!("Enable --features font to render this example.");
}
