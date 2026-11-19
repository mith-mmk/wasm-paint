// Independent, test-only reference adapter; no paintcore code is linked.
// Uses the official resvg crate without optional text/image/system-font features.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let data = std::fs::read(&args[1])?;
    let tree = resvg::usvg::Tree::from_data(&data, &resvg::usvg::Options::default())?;
    let mut image = resvg::tiny_skia::Pixmap::new(256,256).ok_or("reference allocation")?;
    let size = tree.size();
    resvg::render(&tree,resvg::tiny_skia::Transform::from_scale(256./size.width(),256./size.height()),&mut image.as_mut());
    image.save_png(&args[2])?;
    Ok(())
}
