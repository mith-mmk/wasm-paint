//! Diagnostic: render A from a supplied OpenType-SVG font, and each layer type separately.
//! This intentionally measures the legacy renderer; non-empty output is not conformance.
#[cfg(feature = "svg-font")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use paintcore::image_compat::{ImageFormat, RgbaImage};
    use paintcore::prelude::*;
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        return Err("usage: svg_font_probe FONT OUTPUT_PREFIX Y_ORIGIN".into());
    }
    let face = load_font_from_buffer(&std::fs::read(&args[1])?)?;
    let raw = face.engine().with_font_size(256.0).shape("A\u{fe0f}")?;
    let run: GlyphRun = raw.into();
    let y: f32 = args[3].parse()?;
    let mut counts = [0; 3];
    for glyph in &run.glyphs {
        for layer in &glyph.glyph.layers {
            counts[match layer {
                GlyphLayer::Path(_) => 0,
                GlyphLayer::Raster(_) => 1,
                GlyphLayer::Svg(_) => 2,
            }] += 1;
        }
    }
    println!(
        "{{\"paths\":{},\"rasters\":{},\"svg\":{},\"renders\":[",
        counts[0], counts[1], counts[2]
    );
    for (i, kind) in ["all", "paths", "svg"].iter().enumerate() {
        let mut selected = run.clone();
        for g in &mut selected.glyphs {
            g.glyph.layers.retain(|l| match *kind {
                "paths" => matches!(l, GlyphLayer::Path(_)),
                "svg" => matches!(l, GlyphLayer::Svg(_)),
                _ => true,
            });
        }
        let mut canvas = Layer::tmp(256, 256);
        let result = draw_glyphs(&mut canvas, &selected, 0.0, y, 0xff00_0000);
        if let Err(error) = &result {
            eprintln!("{kind}: {error}");
        }
        let painted = canvas
            .buffer()
            .chunks_exact(4)
            .filter(|p| p[3] != 0)
            .count();
        let image = RgbaImage::from_raw(canvas.width(), canvas.height(), canvas.buffer().to_vec())?;
        std::fs::write(
            format!("{}-{kind}.png", args[2]),
            image.encode_to_vec(ImageFormat::Png)?,
        )?;
        println!(
            "{}{{\"kind\":\"{kind}\",\"ok\":{},\"nontransparent_pixels\":{painted}}}",
            if i == 0 { "" } else { "," },
            result.is_ok()
        );
    }
    println!("]}}");
    Ok(())
}
#[cfg(not(feature = "svg-font"))]
fn main() {
    eprintln!("Enable --features svg-font");
}
