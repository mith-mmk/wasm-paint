#![cfg(feature = "svg-font")]

use paintcore::prelude::*;

const FIXTURE: &[u8] = include_bytes!("fixtures/svg-text.ttf");

#[test]
fn loaded_font_text_draw_carries_svg_placement_without_guessing() {
    let font = load_font_from_buffer(FIXTURE).unwrap();
    let options = FontOptions::new(&font).with_font_size(64.);
    let mut screen = Layer::tmp(76, 36);
    let run = draw_text_with_options(
        &mut screen,
        "A\u{fe0f}A\u{fe0f}",
        options,
        5.,
        7.,
        0xff000000,
    )
    .unwrap();
    assert_eq!(run.glyphs.len(), 2);
    for (i, pixel) in screen.buffer().chunks_exact(4).enumerate() {
        let (x, y) = (i % 76, i / 76);
        let in_art = ((9..25).contains(&x) || (41..57).contains(&x)) && (15..31).contains(&y);
        assert_eq!(pixel, if in_art { [34, 119, 204, 128] } else { [0; 4] });
    }
    // A detached GlyphRun has no SVG placement. Its legacy draw path stays explicit.
    assert!(draw_glyphs(&mut screen, &run, 0., 0., 0xff000000).is_err());
}

#[test]
fn unsupported_svg_in_loaded_text_does_not_paint_an_earlier_glyph() {
    let font = load_font_from_buffer(FIXTURE).unwrap();
    let mut screen = Layer::tmp(76, 36);
    screen.buffer_mut().fill(42);
    let error = draw_text_with_options(
        &mut screen,
        "A\u{fe0f}B\u{fe0f}",
        FontOptions::new(&font).with_font_size(64.),
        5.,
        7.,
        0xff000000,
    )
    .err()
    .expect("the unsupported filter must fail");
    let error = error.downcast_ref::<std::io::Error>().unwrap();
    assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
    assert!(screen.buffer().iter().all(|&byte| byte == 42));
}
