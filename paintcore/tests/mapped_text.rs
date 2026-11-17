#![cfg(feature = "font")]
use paintcore::{canvas::Screen, layer::Layer, mapped_text::*, path::*};
const EURO: &[u8] = include_bytes!("fixtures/european-fallback.ttc");
fn at(n: usize) -> TextPosition {
    TextPosition::new(n, CaretAffinity::Downstream)
}
fn render(run: &GlyphRun) -> Vec<u8> {
    let mut layer = Layer::tmp(400, 500);
    draw_glyphs(&mut layer, run, 120.0, 150.0, 0xff12_3456).unwrap();
    layer.buffer().to_vec()
}
fn assert_render(layout: &MappedTextLayout, expected: &GlyphRun) {
    let mut layer = Layer::tmp(400, 500);
    layout
        .placed(120.0, 150.0)
        .unwrap()
        .draw(&mut layer, 0xff12_3456)
        .unwrap();
    assert_eq!(layer.buffer(), render(expected));
    if !layout.text().is_empty() {
        assert!(layer.buffer().as_chunks::<4>().0.iter().any(|p| p[3] > 0));
    }
}
#[test]
fn single_preserves_source_and_rendering_and_translates_carets() {
    let file = FontFile::from_buffer(EURO).unwrap();
    let face = file.face(1).unwrap();
    let engine = face.engine().with_font_size(100.0);
    for text in ["", "fißa\u{308}B", "B\u{fe0f}B"] {
        let layout = MappedTextLayout::shape(&engine, text).unwrap();
        let TextLayoutSource::Single(raw) = layout.source() else {
            panic!()
        };
        assert_eq!(layout.text(), text);
        assert_eq!(raw.glyph_sources().len(), layout.glyph_run().glyphs.len());
        assert_render(&layout, &engine.shape(text).unwrap().into());
        for range in raw.grapheme_ranges() {
            let raw_c = fontcore::CaretLayout::new(raw)
                .unwrap()
                .caret_positions(at(range.start))
                .unwrap();
            let c = layout
                .placed(120.0, 150.0)
                .unwrap()
                .caret_positions(at(range.start))
                .unwrap();
            for (a, b) in raw_c.iter().zip(c) {
                assert_eq!((a.x() + 120.0, a.y() + 150.0), (b.x(), b.y()));
                assert_eq!(a.quality(), b.quality());
            }
        }
    }
    let layout = MappedTextLayout::shape(&engine, "fi").unwrap();
    assert_eq!(
        layout
            .placed(0.0, 0.0)
            .unwrap()
            .caret_positions(at(1))
            .unwrap()[0]
            .quality(),
        CaretQuality::Approximate
    );
    let l = MappedTextLayout::shape(&engine, "a\u{308}").unwrap();
    assert!(l.placed(0.0, 0.0).unwrap().caret_positions(at(1)).is_err());
}
#[test]
fn family_provenance_survives_conversion() {
    let file = FontFile::from_buffer(EURO).unwrap();
    let mut family = FontFamily::new("Primary");
    family.add_face(
        FontFaceDescriptor::new("Primary"),
        file.face(0).unwrap().clone(),
    );
    family.add_face(
        FontFaceDescriptor::new("Fallback"),
        file.face(1).unwrap().clone(),
    );
    let layout = MappedTextLayout::shape_family(&family, "BäB", family.options()).unwrap();
    let TextLayoutSource::Single(raw) = layout.source() else {
        panic!()
    };
    assert!(raw
        .glyph_sources()
        .iter()
        .any(|s| s.byte_range() == (1..3) && s.face_index() == 1));
    assert_render(
        &layout,
        &family
            .text2glyph_run("BäB", family.options())
            .unwrap()
            .into(),
    );
}
#[test]
fn hard_lines_render_with_origins_and_hit_global_offsets() {
    let face = load_font_from_buffer(include_bytes!("fixtures/vertical-metrics.ttf")).unwrap();
    for direction in [
        TextDirection::LeftToRight,
        TextDirection::RightToLeft,
        TextDirection::TopToBottom,
    ] {
        let engine = face.engine().with_options(
            FontOptions::new(&face)
                .with_font_size(30.0)
                .with_text_direction(direction),
        );
        let layout = MappedTextLayout::shape_hard_lines(&engine, "AB\r\n\nA\n").unwrap();
        let TextLayoutSource::HardLines(raw) = layout.source() else {
            panic!()
        };
        assert_eq!(raw.lines().len(), 4);
        let mut layer = Layer::tmp(400, 500);
        for l in raw.lines() {
            draw_glyphs(
                &mut layer,
                &l.shaped().glyph_run().clone().into(),
                120.0 + l.origin_x(),
                150.0 + l.origin_y(),
                0xff12_3456,
            )
            .unwrap();
        }
        assert_eq!(render(layout.glyph_run()), layer.buffer());
        let placed = layout.placed(120.0, 150.0).unwrap();
        for byte in [0, 2, 4, 5, 6, 7] {
            let c = placed.caret_positions(at(byte)).unwrap()[0];
            assert_eq!(
                placed
                    .hit_test(c.x(), c.y())
                    .unwrap()
                    .text_position()
                    .byte_offset(),
                byte
            );
        }
        assert!(placed.caret_positions(at(3)).is_err());
    }
}
#[test]
fn bidi_render_and_queries_match_upstream() {
    let face = load_font_from_buffer(include_bytes!("fixtures/bidi-layout.ttf")).unwrap();
    let engine = face.engine().with_font_size(30.0);
    let layout =
        MappedTextLayout::shape_bidi(&engine, "AB אב\nAB", BidiBaseDirection::Auto).unwrap();
    let TextLayoutSource::Bidi(raw) = layout.source() else {
        panic!()
    };
    let mut layer = Layer::tmp(400, 500);
    for l in raw.lines() {
        for r in l.visual_runs() {
            draw_glyphs(
                &mut layer,
                &r.shaped().glyph_run().clone().into(),
                120.0 + r.origin_x(),
                150.0 + r.origin_y(),
                0xff12_3456,
            )
            .unwrap();
        }
    }
    assert_eq!(render(layout.glyph_run()), layer.buffer());
    for (x, y) in [(0.0, 0.0), (60.0, 0.0), (0.0, 30.0)] {
        let expected = raw.hit_test(x, y).unwrap();
        let actual = layout
            .placed(120.0, 150.0)
            .unwrap()
            .hit_test(x + 120.0, y + 150.0)
            .unwrap();
        assert_eq!(expected.text_position(), actual.text_position());
        assert_eq!(Some(expected.line_index()), actual.line_index());
    }
}
#[test]
fn mixed_vertical_is_not_rotated_twice() {
    let file = FontFile::from_buffer(include_bytes!("fixtures/mixed-vertical-ltr.ttc")).unwrap();
    let face = file.face(0).unwrap();
    let engine = face.engine().with_font_size(30.0);
    let layout = MappedTextLayout::shape_vertical_mixed_ltr(&engine, "AあB").unwrap();
    assert_render(
        &layout,
        &engine.shape_vertical_mixed_ltr("AあB").unwrap().into(),
    );
    let TextLayoutSource::VerticalMixed(raw) = layout.source() else {
        panic!()
    };
    for byte in [0, 1, 4, 5] {
        let a = raw.caret_positions(at(byte)).unwrap();
        let b = layout
            .placed(120.0, 150.0)
            .unwrap()
            .caret_positions(at(byte))
            .unwrap();
        for (a, b) in a.iter().zip(b) {
            assert_eq!((a.x() + 120.0, a.y() + 150.0), (b.x(), b.y()));
            assert_eq!(Some(a.group_index()), b.group_index());
        }
    }
}
#[test]
fn invalid_coordinates_and_upstream_errors_are_preserved() {
    let face = load_font_from_buffer(include_bytes!("fixtures/vertical-metrics.ttf")).unwrap();
    let engine = face.engine();
    let layout = MappedTextLayout::shape(&engine, "AB").unwrap();
    assert!(layout.placed(f32::NAN, 0.0).is_err());
    let placed = layout.placed(-f32::MAX, 0.0).unwrap();
    assert!(placed.hit_test(f32::MAX, 0.0).is_err());
    assert!(placed.hit_test(0.0, f32::INFINITY).is_err());
    assert!(MappedTextLayout::shape_hard_lines(&engine, "A\tB").is_err());
}

#[test]
fn bidi_direction_boundary_preserves_both_affinities() {
    let face = load_font_from_buffer(include_bytes!("fixtures/bidi-layout.ttf")).unwrap();
    let engine = face.engine().with_font_size(100.0);
    let layout =
        MappedTextLayout::shape_bidi(&engine, "aאבb", BidiBaseDirection::LeftToRight).unwrap();
    let placed = layout.placed(120.0, 150.0).unwrap();
    let up = placed
        .caret_positions(TextPosition::new(1, CaretAffinity::Upstream))
        .unwrap()[0];
    let down = placed
        .caret_positions(TextPosition::new(1, CaretAffinity::Downstream))
        .unwrap()[0];
    assert_eq!((up.x(), down.x()), (180.0, 300.0));
    assert_eq!(up.text_position().affinity(), CaretAffinity::Upstream);
    assert_eq!(down.text_position().affinity(), CaretAffinity::Downstream);
}
