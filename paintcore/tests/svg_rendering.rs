#![cfg(feature = "svg-font")]
use paintcore::{canvas::Screen, layer::Layer, path::*};
fn svg(document: &str) -> GlyphLayer {
    GlyphLayer::Svg(SvgGlyphLayer {
        document: document.into(),
        view_box_min_x: 0.,
        view_box_min_y: 0.,
        view_box_width: 16.,
        view_box_height: 16.,
        width: 16.,
        height: 16.,
        offset_x: 0.,
        offset_y: 0.,
    })
}
fn render(layers: Vec<GlyphLayer>) -> Vec<u8> {
    let mut canvas = Layer::tmp(20, 20);
    let run = GlyphRun::new(vec![PositionedGlyph::new(Glyph::new(layers), 0., 0.)]);
    draw_glyphs_with_svg_transforms(
        &mut canvas,
        &run,
        &[Some(SvgRenderTransform {
            viewport_units_to_pixels: [1., 0., 0., 1., 0., 0.],
        })],
        0.,
        0.,
        0xff000000,
    )
    .unwrap();
    canvas.buffer().to_vec()
}
#[test]
fn raw_svg_replaces_approximation_in_both_layer_orders() {
    let doc = "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'><path fill='#00ff00' d='M2 2H14V14H2Z'/></svg>";
    let path = GlyphLayer::Path(PathGlyphLayer::new(
        vec![
            Command::MoveTo(0., 0.),
            Command::Line(20., 0.),
            Command::Line(20., 20.),
            Command::Close,
        ],
        GlyphPaint::CurrentColor,
    ));
    let expected = render(vec![svg(doc)]);
    assert!(expected.chunks_exact(4).any(|p| p[3] > 0));
    assert_eq!(expected, render(vec![path.clone(), svg(doc)]));
    assert_eq!(expected, render(vec![svg(doc), path]));
}
#[test]
fn smooth_relative_cubics_match_expanded_absolute_commands() {
    let a="<svg viewBox='0 0 16 16'><path fill='#ff0000' d='M2 8c0-4 4-4 6 0s6 4 6 0L14 14H2Z'/></svg>";
    let b="<svg viewBox='0 0 16 16'><path fill='#ff0000' d='M2 8C2 4 6 4 8 8C10 12 14 12 14 8L14 14L2 14Z'/></svg>";
    assert_eq!(render(vec![svg(a)]), render(vec![svg(b)]));
}
#[test]
fn malformed_and_external_svg_leave_destination_unchanged() {
    for doc in [
        "<svg><path",
        "<svg viewBox='0 0 16 16'><image href='https://example.com/p.png'/></svg>",
        "<!DOCTYPE svg [<!ENTITY x SYSTEM 'file:///etc/passwd'>]><svg>&x;</svg>",
    ] {
        let mut canvas = Layer::tmp(20, 20);
        let before = canvas.buffer().to_vec();
        assert!(draw_glyph(
            &mut canvas,
            &PositionedGlyph::new(Glyph::new(vec![svg(doc)]), 0., 0.),
            0.,
            0.,
            0xff000000
        )
        .is_err());
        assert_eq!(before, canvas.buffer());
        assert!(draw_svg_document(
            &mut canvas,
            doc,
            SvgRenderTransform {
                viewport_units_to_pixels: [1., 0., 0., 1., 0., 0.]
            },
            0.,
            0.,
            0xff000000
        )
        .is_err());
        assert_eq!(before, canvas.buffer());
    }
}

#[test]
fn radial_gradient_transform_produces_center_and_edge_colors() {
    let doc = "<svg viewBox='0 0 16 16'><defs><radialGradient id='g' gradientUnits='userSpaceOnUse' cx='8' cy='8' r='8' gradientTransform='translate(0 16) scale(1 -1)'><stop offset='0' stop-color='#ffff00'/><stop offset='1' stop-color='#ff0000'/></radialGradient></defs><path fill='url(#g)' d='M0 0H16V16H0Z'/></svg>";
    let pixels = render(vec![svg(doc)]);
    let center = &pixels[(8 * 20 + 8) * 4..][..4];
    let edge = &pixels[(8 * 20) * 4..][..4];
    assert_eq!(center[0], 255);
    assert!(center[1] > 200);
    assert!(edge[1] < 40);
    assert_eq!(center[2], 0);
    assert_eq!(center[3], 255);
}

#[test]
fn explicit_transform_includes_origin_exactly_once() {
    let doc = "<svg viewBox='0 0 16 16'><path fill='#00ff00' d='M0 0H4V4H0Z'/></svg>";
    let mut layer = svg(doc);
    if let GlyphLayer::Svg(s) = &mut layer {
        s.offset_x = 3.;
        s.offset_y = 2.;
    }
    let run = GlyphRun::new(vec![PositionedGlyph::new(Glyph::new(vec![layer]), 5., 6.)]);
    let mut canvas = Layer::tmp(20, 20);
    draw_glyphs_with_svg_transforms(
        &mut canvas,
        &run,
        &[Some(SvgRenderTransform {
            viewport_units_to_pixels: [1., 0., 0., 1., 8., 8.],
        })],
        0.,
        0.,
        0xff000000,
    )
    .unwrap();
    assert_eq!(canvas.buffer()[(9 * 20 + 9) * 4 + 3], 255);
    assert_eq!(canvas.buffer()[(17 * 20 + 17) * 4 + 3], 0);
}

#[test]
fn root_viewport_is_resolved_before_explicit_transform() {
    let a="<svg width='16' height='16' viewBox='4 4 8 8'><path fill='#00ff00' d='M4 4H8V8H4Z'/></svg>";
    let b="<svg width='16' height='16' viewBox='0 0 16 16'><path fill='#00ff00' d='M0 0H8V8H0Z'/></svg>";
    assert_eq!(render(vec![svg(a)]), render(vec![svg(b)]));
}

#[test]
fn screen_offset_moves_gradient_and_geometry_together() {
    let doc = "<svg width='16' height='16' viewBox='0 0 16 16'><defs><linearGradient id='g' gradientUnits='userSpaceOnUse' x1='0' y1='0' x2='16' y2='0'><stop offset='0' stop-color='red'/><stop offset='1' stop-color='blue'/></linearGradient></defs><path fill='url(#g)' d='M0 0H16V16H0Z'/></svg>";
    let t = SvgRenderTransform {
        viewport_units_to_pixels: [1., 0., 0., 1., 0., 0.],
    };
    let mut a = Layer::tmp(40, 40);
    let mut b = Layer::tmp(40, 40);
    draw_svg_document(&mut a, doc, t, 0., 0., 0xff000000).unwrap();
    draw_svg_document(&mut b, doc, t, 10., 7., 0xff000000).unwrap();
    for y in 0..16 {
        for x in 0..16 {
            assert_eq!(
                &a.buffer()[(y * 40 + x) * 4..][..4],
                &b.buffer()[((y + 7) * 40 + x + 10) * 4..][..4]
            );
        }
    }
}

#[test]
fn stroke_cap_is_not_lost_in_private_svg_adapter() {
    let round="<svg viewBox='0 0 16 16'><line x1='4' y1='8' x2='12' y2='8' stroke='red' stroke-width='4' stroke-linecap='round'/></svg>";
    let butt = round.replace("round", "butt");
    let a = render(vec![svg(round)]);
    let b = render(vec![svg(&butt)]);
    assert!(a[(8 * 20 + 2) * 4 + 3] > 0);
    assert_eq!(b[(8 * 20 + 2) * 4 + 3], 0);
}

#[test]
fn svg_gradient_zero_alpha_is_not_legacy_rgb_shorthand() {
    let doc="<svg viewBox='0 0 16 16'><linearGradient id='g' gradientUnits='userSpaceOnUse' x1='0' y1='0' x2='16' y2='0'><stop offset='0' stop-color='#ffff00' stop-opacity='0'/><stop offset='1' stop-color='#ffff00' stop-opacity='0'/></linearGradient><rect width='16' height='16' fill='url(#g)'/></svg>";
    assert!(render(vec![svg(doc)]).chunks_exact(4).all(|p| p[3] == 0));
}
