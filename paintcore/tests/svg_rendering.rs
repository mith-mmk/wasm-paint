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
fn coincident_linear_gradient_uses_last_stop_rgba() {
    for (opacity, expected) in [(".5", [34, 119, 204, 128]), ("0", [0, 0, 0, 0])] {
        let doc = format!(
            "<svg viewBox='0 0 16 16'>
                <linearGradient id='base' gradientUnits='userSpaceOnUse'
                    x1='3' y1='4' x2='3.0' y2='4' spreadMethod='repeat'>
                    <stop offset='0' stop-color='#ff0000'/>
                    <stop offset='.8' stop-color='#2277cc' stop-opacity='{opacity}'/>
                </linearGradient>
                <linearGradient id='g' href='#base' gradientTransform='translate(2 3)'/>
                <rect width='16' height='16' fill='url(#g)'/>
            </svg>"
        );
        let pixels = render(vec![svg(&doc)]);
        for (i, pixel) in pixels.chunks_exact(4).enumerate() {
            let (x, y) = (i % 20, i / 20);
            let rgba = if x < 16 && y < 16 { expected } else { [0; 4] };
            assert_eq!(pixel, rgba, "pixel ({x}, {y}), stop-opacity {opacity}");
        }
    }
}

#[test]
fn nonzero_short_linear_gradient_fails_before_drawing() {
    // The second endpoint pair rounds to a single point in f32.
    for x2 in ["8.0001", "8.00000001"] {
        let doc = format!(
            "<svg viewBox='0 0 16 16'>
                <linearGradient id='g' gradientUnits='userSpaceOnUse'
                    x1='8' y1='4' x2='{x2}' y2='4'>
                    <stop offset='0' stop-color='red'/>
                    <stop offset='1' stop-color='blue'/>
                </linearGradient>
                <rect width='16' height='16' fill='green'/>
                <rect width='16' height='16' fill='url(#g)'/>
            </svg>"
        );
        let mut canvas = Layer::tmp(20, 20);
        canvas.buffer_mut().fill(42);
        let error = draw_svg_document(
            &mut canvas,
            &doc,
            SvgRenderTransform {
                viewport_units_to_pixels: [1., 0., 0., 1., 0., 0.],
            },
            0.,
            0.,
            0xff000000,
        )
        .unwrap_err()
        .downcast::<std::io::Error>()
        .expect("SVG preparation returns an I/O error");
        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
        assert_eq!(error.to_string(), "SVG: linear gradient is too short");
        assert!(canvas.buffer().iter().all(|&byte| byte == 42));
    }
}

#[test]
fn svg_gradient_zero_alpha_is_not_legacy_rgb_shorthand() {
    let doc="<svg viewBox='0 0 16 16'><linearGradient id='g' gradientUnits='userSpaceOnUse' x1='0' y1='0' x2='16' y2='0'><stop offset='0' stop-color='#ffff00' stop-opacity='0'/><stop offset='1' stop-color='#ffff00' stop-opacity='0'/></linearGradient><rect width='16' height='16' fill='url(#g)'/></svg>";
    assert!(render(vec![svg(doc)]).chunks_exact(4).all(|p| p[3] == 0));
}

#[test]
fn local_use_clip_and_screen_offset_mask_the_shape() {
    let doc="<svg viewBox='0 0 16 16'><defs><path id='p' d='M0 0H8V16H0Z'/></defs><clipPath id='c'><use href='#p'/></clipPath><rect width='16' height='16' fill='red' clip-path='url(#c)'/></svg>";
    let mut canvas = Layer::tmp(30, 30);
    draw_svg_document(
        &mut canvas,
        doc,
        SvgRenderTransform {
            viewport_units_to_pixels: [1., 0., 0., 1., 0., 0.],
        },
        4.,
        3.,
        0xff000000,
    )
    .unwrap();
    assert_eq!(canvas.buffer()[(5 * 30 + 6) * 4 + 3], 255);
    assert_eq!(canvas.buffer()[(5 * 30 + 16) * 4 + 3], 0);
    assert_eq!(canvas.buffer()[(30 + 6) * 4 + 3], 0);
}

#[test]
fn leaf_and_group_opacity_are_applied_once() {
    let doc =
        "<svg viewBox='0 0 16 16'><rect width='16' height='16' fill='red' opacity='.5'/></svg>";
    let pixels = render(vec![svg(doc)]);
    assert!((127..=128).contains(&pixels[(8 * 20 + 8) * 4 + 3]));
    let group="<svg viewBox='0 0 16 16'><g opacity='.5'><rect width='16' height='16'/><rect width='16' height='16'/></g></svg>";
    let mut canvas = Layer::tmp(20, 20);
    draw_svg_document(
        &mut canvas,
        group,
        SvgRenderTransform {
            viewport_units_to_pixels: [1., 0., 0., 1., 0., 0.],
        },
        0.,
        0.,
        0xff000000,
    )
    .unwrap();
    assert!((127..=128).contains(&canvas.buffer()[(8 * 20 + 8) * 4 + 3]));
}

#[test]
fn inherited_clip_is_not_transformed_again_by_child() {
    let doc="<svg viewBox='0 0 32 16'><defs><clipPath id='c'><rect width='8' height='16'/></clipPath></defs><g transform='translate(4)' clip-path='url(#c)'><rect transform='translate(4)' width='16' height='16' fill='red'/></g></svg>";
    let mut canvas = Layer::tmp(32, 16);
    draw_svg_document(
        &mut canvas,
        doc,
        SvgRenderTransform {
            viewport_units_to_pixels: [1., 0., 0., 1., 0., 0.],
        },
        0.,
        0.,
        0xff000000,
    )
    .unwrap();
    assert_eq!(canvas.buffer()[(8 * 32 + 10) * 4 + 3], 255);
    assert_eq!(canvas.buffer()[(8 * 32 + 14) * 4 + 3], 0);
}

#[test]
fn overlapping_group_children_composite_once_over_backdrop() {
    let doc="<svg viewBox='0 0 16 16'><rect width='16' height='16' fill='blue'/><g opacity='.5'><rect width='12' height='16' fill='red'/><rect x='4' width='12' height='16' fill='red'/></g></svg>";
    let pixels = render(vec![svg(doc)]);
    let left = &pixels[(8 * 20 + 2) * 4..][..4];
    let overlap = &pixels[(8 * 20 + 8) * 4..][..4];
    assert_eq!(left, overlap);
    assert!((127..=128).contains(&overlap[0]));
    assert!((127..=128).contains(&overlap[2]));
    assert_eq!(overlap[3], 255);
}

#[test]
fn group_budget_failure_does_not_draw_an_earlier_glyph() {
    let document = format!(
        "<svg viewBox='0 0 16 16'>{}<rect width='16' height='16'/>{}</svg>",
        "<g opacity='.5'>".repeat(17),
        "</g>".repeat(17)
    );
    let first = PositionedGlyph::new(
        Glyph::new(vec![GlyphLayer::Path(PathGlyphLayer::new(
            vec![
                Command::MoveTo(0., 0.),
                Command::Line(8., 0.),
                Command::Line(8., 8.),
                Command::Close,
            ],
            GlyphPaint::CurrentColor,
        ))]),
        0.,
        0.,
    );
    let second = PositionedGlyph::new(Glyph::new(vec![svg(&document)]), 0., 0.);
    let mut canvas = Layer::tmp(20, 20);
    assert!(draw_glyphs_with_svg_transforms(
        &mut canvas,
        &GlyphRun::new(vec![first, second]),
        &[
            None,
            Some(SvgRenderTransform {
                viewport_units_to_pixels: [1., 0., 0., 1., 0., 0.]
            })
        ],
        0.,
        0.,
        0xffff0000
    )
    .is_err());
    assert!(canvas.buffer().iter().all(|v| *v == 0));
}

#[test]
fn local_use_style_and_translation_match_expanded_svg() {
    let reference="<svg viewBox='0 0 16 16'><g fill='red' transform='translate(2 3)'><rect width='4' height='4'/><rect x='4' width='4' height='4' fill='blue'/></g></svg>";
    let used="<svg viewBox='0 0 16 16'><defs><g id='pair'><rect width='4' height='4'/><rect x='4' width='4' height='4' fill='blue'/></g></defs><use href='#pair' x='2' y='3' fill='red'/></svg>";
    assert_eq!(render(vec![svg(reference)]), render(vec![svg(used)]));
}

#[test]
fn isolated_surface_limit_is_checked_before_screen_access() {
    struct DimensionsOnly;
    impl Screen for DimensionsOnly {
        fn width(&self) -> u32 {
            4097
        }
        fn height(&self) -> u32 {
            4097
        }
        fn reinit(&mut self, _: u32, _: u32) {
            panic!("unexpected screen mutation")
        }
        fn buffer(&self) -> &[u8] {
            panic!("unexpected screen access")
        }
        fn buffer_mut(&mut self) -> &mut [u8] {
            panic!("unexpected screen mutation")
        }
        fn clear(&mut self) {
            panic!("unexpected screen mutation")
        }
        fn clear_with_color(&mut self, _: u32) {
            panic!("unexpected screen mutation")
        }
        fn alpha(&self) -> Option<u8> {
            None
        }
        fn set_alpha(&mut self, _: u8) {
            panic!("unexpected screen mutation")
        }
    }
    let doc="<svg viewBox='0 0 16 16'><g opacity='.5'><rect width='8' height='16'/><rect x='4' width='8' height='16'/></g></svg>";
    let error = draw_svg_document(
        &mut DimensionsOnly,
        doc,
        SvgRenderTransform {
            viewport_units_to_pixels: [1., 0., 0., 1., 0., 0.],
        },
        0.,
        0.,
        0xff000000,
    )
    .unwrap_err();
    assert!(error.to_string().contains("budget"));
}

#[test]
fn cumulative_group_pixel_work_is_bounded_before_drawing() {
    let doc = format!(
        "<svg viewBox='0 0 16 16'>{}</svg>",
        "<g opacity='.5'><rect width='8' height='16'/><rect x='4' width='8' height='16'/></g>"
            .repeat(257)
    );
    let mut canvas = Layer::tmp(512, 512);
    let error = draw_svg_document(
        &mut canvas,
        &doc,
        SvgRenderTransform {
            viewport_units_to_pixels: [1., 0., 0., 1., 0., 0.],
        },
        0.,
        0.,
        0xff000000,
    )
    .unwrap_err();
    assert!(error.to_string().contains("budget"));
    assert!(canvas.buffer().iter().all(|p| *p == 0));
}
