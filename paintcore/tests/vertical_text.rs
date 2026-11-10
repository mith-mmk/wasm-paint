#![cfg(feature = "font")]

use paintcore::canvas::Screen;
use paintcore::layer::Layer;
use paintcore::path::*;
use paintcore::text::glyph_run_bounds;
use std::io::ErrorKind;

const BASE: &[u8] = include_bytes!("fixtures/vertical-metrics.ttf");
const MIXED: &[u8] = include_bytes!("fixtures/mixed-vertical-ltr.ttc");

fn assert_run(actual: &GlyphRun, expected: &fontcore::GlyphRun) {
    assert_eq!(actual.glyphs.len(), expected.glyphs.len());
    for (actual, expected) in actual.glyphs.iter().zip(&expected.glyphs) {
        assert_eq!((actual.x, actual.y), (expected.x, expected.y));
        assert_eq!(
            format!("{:?}", actual.glyph.font),
            format!("{:?}", expected.glyph.font)
        );
        assert_eq!(
            format!("{:?}", actual.glyph.metrics),
            format!("{:?}", expected.glyph.metrics)
        );
        assert_eq!(actual.glyph.layers.len(), expected.glyph.layers.len());
        for (actual, expected) in actual.glyph.layers.iter().zip(&expected.glyph.layers) {
            let (GlyphLayer::Path(actual), fontcore::GlyphLayer::Path(expected)) =
                (actual, expected)
            else {
                panic!("expected outlines");
            };
            assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
        }
    }
}

fn rendered(run: &GlyphRun) -> Layer {
    let mut layer = Layer::tmp(320, 600);
    draw_glyphs(&mut layer, run, 80.0, 100.0, 0xff12_3456).unwrap();
    layer
}

fn alpha(layer: &Layer, x: usize, y: usize) -> u8 {
    layer.buffer()[(y * layer.width() as usize + x) * 4 + 3]
}

#[test]
fn horizontal_and_upright_vertical_preserve_metrics_and_render_at_the_origin() {
    let face = load_font_from_buffer(BASE).unwrap();
    let options = FontOptions::new(&face).with_font_size(100.0);
    let horizontal = layout_text("AB", options.clone()).unwrap();
    assert_run(
        &horizontal,
        &face
            .engine()
            .with_options(options.clone())
            .shape("AB")
            .unwrap(),
    );
    assert_eq!(
        (horizontal.glyphs[1].x, horizontal.glyphs[1].y),
        (60.0, 0.0)
    );
    assert_eq!(alpha(&rendered(&horizontal), 90, 40), 255);
    let vertical = layout_text(
        "AB",
        options.with_text_direction(TextDirection::TopToBottom),
    )
    .unwrap();
    assert_eq!((vertical.glyphs[1].x, vertical.glyphs[1].y), (0.0, 120.0));
    assert_eq!(vertical.glyphs[0].glyph.metrics.advance_y, 120.0);
    assert_eq!(
        vertical.glyphs[0].glyph.font.unwrap().flow,
        GlyphFlow::Vertical
    );
    let bounds = vertical.glyphs[0].glyph.metrics.bounds.unwrap();
    assert_eq!(
        (bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y),
        (-25.0, 10.0, 15.0, 80.0)
    );
    let image = rendered(&vertical);
    assert_eq!(alpha(&image, 60, 115), 255);
    assert_eq!(alpha(&image, 110, 115), 0);
}

#[test]
fn sideways_rotates_once_and_keeps_the_engine_unchanged() {
    let face = load_font_from_buffer(BASE).unwrap();
    let engine = face.engine().with_font_size(100.0).with_vertical_flow();
    let run = layout_text_sideways_ltr("AB A", &engine).unwrap();
    assert_run(&run, &engine.shape_sideways_ltr("AB A").unwrap());
    assert_eq!((run.glyphs[1].x, run.glyphs[1].y), (0.0, 60.0));
    assert_eq!(run.glyphs[0].glyph.metrics.advance_y, 60.0);
    let bounds = run.glyphs[0].glyph.metrics.bounds.unwrap();
    assert_eq!(
        (bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y),
        (0.0, 5.0, 70.0, 45.0)
    );
    let image = rendered(&run);
    assert_eq!(alpha(&image, 90, 110), 255);
    assert_eq!(alpha(&image, 60, 115), 0);
    assert_eq!(alpha(&image, 90, 40), 0);
    assert_eq!(engine.options().text_direction, TextDirection::TopToBottom);
    assert_eq!(engine.shape("AB").unwrap().glyphs[1].y, 120.0);
}

#[test]
fn mixed_vertical_preserves_centering_kerning_marks_and_pixels() {
    let file = FontFile::from_buffer(MIXED).unwrap();
    for index in [0, 1, 2, 4] {
        let face = file.face(index).unwrap();
        let engine = face.engine().with_font_size(100.0);
        for text in [
            "漢AV漢",
            "漢A\u{301}漢",
            "「AV、",
            "1\u{fe0f}\u{20e3}A",
            " A ",
            "",
            "  ",
        ] {
            let actual = layout_text_vertical_mixed_ltr(text, &engine).unwrap();
            let expected = engine.shape_vertical_mixed_ltr(text).unwrap();
            assert_run(&actual, &expected);
            assert_eq!(
                rendered(&actual).buffer(),
                rendered(&expected.into()).buffer()
            );
        }
    }
    let face = file.face(0).unwrap();
    let engine = face.engine().with_font_size(100.0);
    let run = layout_text_vertical_mixed_ltr("漢AV漢", &engine).unwrap();
    assert_eq!((run.glyphs[1].x, run.glyphs[1].y), (-30.0, 100.0));
    assert_eq!((run.glyphs[3].x, run.glyphs[3].y), (0.0, 253.0));
    assert!(rendered(&run)
        .buffer()
        .as_chunks::<4>()
        .0
        .iter()
        .any(|p| p[3] != 0));
}

#[test]
fn vertical_columns_use_line_height_without_changing_horizontal_defaults() {
    let face = load_font_from_buffer(BASE).unwrap();
    let options = FontOptions::new(&face)
        .with_font_size(100.0)
        .with_line_height(150.0);
    let run = layout_text("A\nB", options.clone().with_vertical_flow()).unwrap();
    assert_eq!((run.glyphs[1].x, run.glyphs[1].y), (-150.0, 0.0));
    let run = layout_text("A\nB", options).unwrap();
    assert_eq!((run.glyphs[1].x, run.glyphs[1].y), (0.0, 150.0));
}

#[test]
fn restricted_layouts_preserve_errors_and_reject_unsupported_color_layers() {
    let face = load_font_from_buffer(BASE).unwrap();
    let engine = face.engine().with_font_size(100.0);
    for control in [
        '\t', '\r', '\n', '\u{b}', '\u{c}', '\u{85}', '\u{2028}', '\u{2029}',
    ] {
        let text = format!("A{control}B");
        for result in [
            layout_text_sideways_ltr(&text, &engine),
            layout_text_vertical_mixed_ltr(&text, &engine),
        ] {
            let error = result.err().expect("single-line restriction");
            assert_eq!(
                error.downcast_ref::<std::io::Error>().unwrap().kind(),
                ErrorKind::InvalidInput
            );
        }
    }
    let file = FontFile::from_buffer(include_bytes!("fixtures/colr-vertical-origin.ttc")).unwrap();
    let face = file.face(0).unwrap();
    let engine = face.engine().with_font_size(100.0);
    for result in [
        layout_text_sideways_ltr("A、A", &engine),
        layout_text_vertical_mixed_ltr("A、A", &engine),
    ] {
        let error = result.err().expect("unsupported color layer");
        assert_eq!(
            error.downcast_ref::<std::io::Error>().unwrap().kind(),
            ErrorKind::Unsupported
        );
    }
    assert!(layout_text_vertical_mixed_ltr("A", &engine).is_ok());
}

#[test]
fn no_ink_glyphs_do_not_discard_vertical_ink_bounds() {
    let face = load_font_from_buffer(BASE).unwrap();
    let engine = face.engine().with_font_size(100.0);
    for text in ["A ", " A", " A A "] {
        let run = layout_text_sideways_ltr(text, &engine).unwrap();
        let actual = glyph_run_bounds(&run, 7.0, 11.0).unwrap();
        let expected = engine
            .measure_layout_sideways_ltr(text)
            .unwrap()
            .ink_bounds
            .unwrap();
        assert_eq!(
            (actual.min_x, actual.min_y, actual.max_x, actual.max_y),
            (
                expected.min_x + 7.0,
                expected.min_y + 11.0,
                expected.max_x + 7.0,
                expected.max_y + 11.0
            )
        );
    }
    for text in ["", "   "] {
        assert!(
            glyph_run_bounds(&layout_text_sideways_ltr(text, &engine).unwrap(), 0.0, 0.0).is_none()
        );
    }
}
