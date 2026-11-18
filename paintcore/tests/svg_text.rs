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

const PRIMARY: &[u8] = include_bytes!("fixtures/svg-layout-primary.ttf");
const FALLBACK: &[u8] = include_bytes!("fixtures/svg-layout-fallback.ttf");
const SAMPLE: &str = "A\u{fe0f}😀A\u{fe0f}";
const MIXED: &str = "A\u{fe0f}一\u{fe0f}A\u{fe0f}";

fn assert_art(screen: &Layer, area: impl Fn(usize, usize) -> bool) {
    for (i, pixel) in screen.buffer().chunks_exact(4).enumerate() {
        let w = screen.width() as usize;
        let (x, y) = (i % w, i / w);
        assert_eq!(
            pixel,
            if area(x, y) {
                [34, 119, 204, 128]
            } else {
                [0; 4]
            },
            "pixel ({x}, {y})"
        );
    }
}

#[test]
fn family_and_mapped_family_preserve_distinct_face_scales() {
    use paintcore::mapped_text::MappedTextLayout;
    let mut family = FontFamily::new("SvgLayoutPrimary");
    family.add_font_face(load_font_from_buffer(PRIMARY).unwrap());
    family.add_font_face(load_font_from_buffer(FALLBACK).unwrap());
    let options = family.options().with_font_size(64.0);
    let mut ordinary = Layer::tmp(84, 36);
    draw_text_with_family(
        &mut ordinary,
        &family,
        SAMPLE,
        options.clone(),
        5.,
        7.,
        0xff000000,
    )
    .ok()
    .unwrap();
    assert_art(&ordinary, |x, y| {
        (((9..25).contains(&x) || (57..73).contains(&x)) && (15..31).contains(&y))
            || ((39..47).contains(&x) && (11..19).contains(&y))
    });
    let mapped = MappedTextLayout::shape_family(&family, SAMPLE, options).unwrap();
    let mapped = mapped.placed(5., 7.).unwrap();
    let mut via_mapped = Layer::tmp(84, 36);
    mapped.draw(&mut via_mapped, 0xff000000).unwrap();
    assert_eq!(via_mapped.buffer(), ordinary.buffer());
}

#[test]
fn sideways_svg_rotates_exactly_once() {
    let face = load_font_from_buffer(PRIMARY).unwrap();
    let engine = face.engine().with_font_size(64.0);
    let layout = layout_text_sideways_ltr_with_svg_context("A\u{fe0f}", &engine).unwrap();
    let mut screen = Layer::tmp(56, 36);
    layout.draw(&mut screen, 40., 7., 0xff000000).unwrap();
    assert_art(&screen, |x, y| {
        (16..32).contains(&x) && (11..27).contains(&y)
    });
}

#[test]
fn mixed_svg_matches_mapped_drawing_and_preserves_carets() {
    use paintcore::mapped_text::{CaretAffinity, MappedTextLayout, TextPosition};
    let face = load_font_from_buffer(PRIMARY).unwrap();
    let engine = face.engine().with_font_size(64.0);
    let ordinary = layout_text_vertical_mixed_ltr_with_svg_context(MIXED, &engine).unwrap();
    let mut a = Layer::tmp(160, 240);
    ordinary.draw(&mut a, 94., 72., 0xff000000).unwrap();
    assert_eq!(
        a.buffer().chunks_exact(4).filter(|px| px[3] != 0).count(),
        3 * 16 * 16
    );

    let mapped = MappedTextLayout::shape_vertical_mixed_ltr(&engine, MIXED).unwrap();
    let placed = mapped.placed(94., 72.).unwrap();
    let mut b = Layer::tmp(160, 240);
    placed.draw(&mut b, 0xff000000).unwrap();
    assert_eq!(a.buffer(), b.buffer());
    let carets = placed
        .caret_positions(TextPosition::new(4, CaretAffinity::Downstream))
        .unwrap();
    assert_eq!(carets[0].group_index(), Some(1));
}

#[test]
fn mapped_hard_lines_and_bidi_apply_origin_once_and_fail_atomically() {
    use paintcore::mapped_text::{BidiBaseDirection, MappedTextLayout};
    let face = load_font_from_buffer(PRIMARY).unwrap();
    let engine = face.engine().with_font_size(64.0);
    let mut screen = Layer::tmp(84, 110);

    let layout = MappedTextLayout::shape_hard_lines(&engine, "A\u{fe0f}\nA\u{fe0f}").unwrap();
    layout
        .placed(5., 7.)
        .unwrap()
        .draw(&mut screen, 0xff000000)
        .unwrap();
    assert_art(&screen, |x, y| {
        (9..25).contains(&x) && ((15..31).contains(&y) || (79..95).contains(&y))
    });

    let before = screen.buffer().to_vec();
    let unsupported = MappedTextLayout::shape_hard_lines(&engine, "A\u{fe0f}\nB\u{fe0f}").unwrap();
    assert!(unsupported
        .placed(0., 0.)
        .unwrap()
        .draw(&mut screen, 0xff000000)
        .is_err());
    assert_eq!(screen.buffer(), before);

    let bidi = MappedTextLayout::shape_bidi(
        &engine,
        "A\u{fe0f}א\u{fe0f}",
        BidiBaseDirection::LeftToRight,
    )
    .unwrap();
    let mut bidi_screen = Layer::tmp(84, 36);
    bidi.placed(5., 7.)
        .unwrap()
        .draw(&mut bidi_screen, 0xff000000)
        .unwrap();
    assert_art(&bidi_screen, |x, y| {
        ((9..25).contains(&x) || (41..57).contains(&x)) && (15..31).contains(&y)
    });
}
