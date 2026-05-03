use paintcore::canvas::Screen;
use paintcore::layer::Layer;
use paintcore::math::{parse_tex_like, tex_like_plain_text, MathScript, MathToken};
use paintcore::path::{Glyph, GlyphBounds, GlyphMetrics, GlyphRun, PositionedGlyph};
use paintcore::text::{
    draw_text_decorations, glyph_run_bounds, TextDecoration, TextDecorationOptions,
};

fn glyph_with_bounds(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Glyph {
    let mut glyph = Glyph::new(Vec::new());
    glyph.metrics = GlyphMetrics {
        advance_x: max_x - min_x,
        advance_y: 0.0,
        bearing_x: 0.0,
        bearing_y: 0.0,
        bounds: Some(GlyphBounds {
            min_x,
            min_y,
            max_x,
            max_y,
        }),
    };
    glyph
}

#[test]
fn parse_tex_like_splits_super_and_subscripts() {
    assert_eq!(
        parse_tex_like("x^{2}_i"),
        vec![
            MathToken {
                text: "x".to_string(),
                script: MathScript::Normal,
            },
            MathToken {
                text: "2".to_string(),
                script: MathScript::Superscript,
            },
            MathToken {
                text: "i".to_string(),
                script: MathScript::Subscript,
            },
        ]
    );
    assert_eq!(tex_like_plain_text("x^{2}_i"), "x2i");
}

#[test]
fn glyph_run_bounds_combines_positioned_glyph_metrics() {
    let run = GlyphRun::new(vec![
        PositionedGlyph::new(glyph_with_bounds(0.0, -8.0, 5.0, 2.0), 10.0, 20.0),
        PositionedGlyph::new(glyph_with_bounds(0.0, -6.0, 4.0, 3.0), 16.0, 20.0),
    ]);

    let bounds = glyph_run_bounds(&run, 1.0, 2.0).unwrap();

    assert_eq!(bounds.min_x, 11.0);
    assert_eq!(bounds.min_y, 14.0);
    assert_eq!(bounds.max_x, 21.0);
    assert_eq!(bounds.max_y, 25.0);
}

#[test]
fn draw_text_decorations_draws_underlines_from_glyph_bounds() {
    let run = GlyphRun::new(vec![PositionedGlyph::new(
        glyph_with_bounds(2.0, 2.0, 8.0, 6.0),
        0.0,
        0.0,
    )]);
    let mut canvas = Layer::tmp(16, 16);

    draw_text_decorations(
        &mut canvas,
        &run,
        0.0,
        0.0,
        0x00ff_0000,
        &[TextDecoration::Underline],
        TextDecorationOptions {
            underline_offset: 1.0,
            ..TextDecorationOptions::default()
        },
    );

    let has_red = canvas
        .buffer()
        .chunks_exact(4)
        .any(|pixel| pixel[0] > 0 && pixel[1] == 0 && pixel[2] == 0);
    assert!(has_red);
}
