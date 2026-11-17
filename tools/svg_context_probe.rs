// Optional integration harness for unreleased fontcore's additive SVG context API.
// Compiled by run_svg_context_probe.py, not against the published dependency.
use paintcore::prelude::*;
use paintcore::image_compat::{ImageFormat, RgbaImage};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let face = fontcore::load_font_from_buffer(&std::fs::read(&args[1])?)?;
    let context = face.engine().with_font_size(256.).shape_with_svg_context("A\u{fe0f}")?;
    let run: GlyphRun = context.run().clone().into();
    let mut transforms = Vec::new();
    let mut counts = [0;3];
    for (gi,g) in context.run().glyphs.iter().enumerate() {
        let mut transform = None;
        for (li,layer) in g.glyph.layers.iter().enumerate() {
            match layer {
                fontcore::GlyphLayer::Path(_) => counts[0] += 1,
                fontcore::GlyphLayer::Raster(_) => counts[1] += 1,
                fontcore::GlyphLayer::Svg(_) => { counts[2] += 1; transform = context.svg_layer_transform(gi,li).map(|m| SvgRenderTransform { viewport_units_to_pixels:m }); }
            }
        }
        transforms.push(transform);
    }
    let mut canvas = Layer::tmp(256,256);
    let result = draw_glyphs_with_svg_transforms(&mut canvas,&run,&transforms,0.,args[3].parse()?,0xff000000);
    if let Err(e)=&result { eprintln!("{e}"); }
    let painted=canvas.buffer().chunks_exact(4).filter(|p|p[3]>0).count();
    let image=RgbaImage::from_raw(256,256,canvas.buffer().to_vec())?;
    std::fs::write(format!("{}-all.png",args[2]),image.encode_to_vec(ImageFormat::Png)?)?;
    println!("{{\"paths\":{},\"rasters\":{},\"svg\":{},\"renders\":[{{\"kind\":\"all\",\"ok\":{},\"nontransparent_pixels\":{painted}}}]}}",counts[0],counts[1],counts[2],result.is_ok());
    result?;
    Ok(())
}
