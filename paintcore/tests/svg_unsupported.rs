#![cfg(feature = "svg-font")]
use paintcore::{canvas::Screen, layer::Layer, path::*};
fn raw() -> GlyphLayer {
    GlyphLayer::Svg(SvgGlyphLayer {
        document: "<svg viewBox='0 0 8 8'><filter id='blur'/><path filter='url(#blur)' d='M0 0H8V8Z'/></svg>".into(),
        view_box_min_x: 0.0,
        view_box_min_y: 0.0,
        view_box_width: 8.0,
        view_box_height: 8.0,
        width: 8.0,
        height: 8.0,
        offset_x: 0.0,
        offset_y: 0.0,
    })
}
fn path() -> GlyphLayer {
    GlyphLayer::Path(PathGlyphLayer::new(
        vec![
            Command::MoveTo(0.0, 0.0),
            Command::Line(8.0, 0.0),
            Command::Line(8.0, 8.0),
            Command::Close,
        ],
        GlyphPaint::CurrentColor,
    ))
}
#[test]
fn unsupported_and_mixed_svg_glyphs_fail_before_mutating_pixels() {
    for layers in [vec![raw()], vec![path(), raw()], vec![raw(), path()]] {
        let mut screen = Layer::tmp(16, 16);
        let before = screen.buffer().to_vec();
        let glyph = PositionedGlyph::new(Glyph::new(layers), 0.0, 0.0);
        let e = draw_glyph(&mut screen, &glyph, 0.0, 0.0, 0xffff_0000).unwrap_err();
        assert_eq!(
            e.downcast_ref::<std::io::Error>().unwrap().kind(),
            std::io::ErrorKind::Unsupported
        );
        assert_eq!(before, screen.buffer());
    }
}
#[test]
fn unsupported_svg_later_in_run_does_not_leave_earlier_glyphs_drawn() {
    let mut screen = Layer::tmp(16, 16);
    let before = screen.buffer().to_vec();
    let run = GlyphRun::new(vec![
        PositionedGlyph::new(Glyph::new(vec![path()]), 0.0, 0.0),
        PositionedGlyph::new(Glyph::new(vec![raw()]), 0.0, 0.0),
    ]);
    assert!(draw_glyphs(&mut screen, &run, 0.0, 0.0, 0xffff_0000).is_err());
    assert_eq!(before, screen.buffer());
    let transforms = vec![
        Some(SvgRenderTransform {
            viewport_units_to_pixels: [1., 0., 0., 1., 0., 0.]
        });
        run.glyphs.len()
    ];
    assert!(
        draw_glyphs_with_svg_transforms(&mut screen, &run, &transforms, 0., 0., 0xffff0000)
            .is_err()
    );
    assert_eq!(before, screen.buffer());
}
#[test]
fn supported_path_run_still_renders() {
    let mut screen = Layer::tmp(16, 16);
    let run = GlyphRun::new(vec![PositionedGlyph::new(
        Glyph::new(vec![path()]),
        0.0,
        0.0,
    )]);
    draw_glyphs(&mut screen, &run, 0.0, 0.0, 0xffff_0000).unwrap();
    assert!(screen.buffer().chunks_exact(4).any(|p| p[3] != 0));
}
