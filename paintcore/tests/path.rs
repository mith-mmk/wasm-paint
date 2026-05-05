#[cfg(feature = "font")]
use fontcore::load_font_from_buffer;
use paintcore::canvas::{Canvas, Screen};
use paintcore::clear::fillrect;
use paintcore::path::*;
#[cfg(feature = "font")]
use std::path::{Path, PathBuf};

fn rgba(screen: &dyn Screen, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * screen.width() + x) * 4) as usize;
    [
        screen.buffer()[offset],
        screen.buffer()[offset + 1],
        screen.buffer()[offset + 2],
        screen.buffer()[offset + 3],
    ]
}

fn has_gray_pixel(screen: &dyn Screen, x0: u32, y0: u32, x1: u32, y1: u32) -> bool {
    let x1 = x1.min(screen.width());
    let y1 = y1.min(screen.height());
    for y in y0.min(y1)..y1 {
        for x in x0.min(x1)..x1 {
            let pixel = rgba(screen, x, y);
            if pixel[3] == 0xff
                && pixel[0] > 0x00
                && pixel[0] < 0xff
                && pixel[0] == pixel[1]
                && pixel[1] == pixel[2]
            {
                return true;
            }
        }
    }
    false
}

#[test]
fn draw_glyphs_fills_nonzero_path_with_hole() {
    let commands = vec![
        Command::MoveTo(1.0, 1.0),
        Command::Line(8.0, 1.0),
        Command::Line(8.0, 8.0),
        Command::Line(1.0, 8.0),
        Command::Close,
        Command::MoveTo(3.0, 3.0),
        Command::Line(3.0, 6.0),
        Command::Line(6.0, 6.0),
        Command::Line(6.0, 3.0),
        Command::Close,
    ];

    let glyph = Glyph::new(vec![GlyphLayer::Path(PathGlyphLayer::new(
        commands,
        GlyphPaint::CurrentColor,
    ))]);
    let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 0.0, 0.0)]);
    let mut canvas = Canvas::new(10, 10);

    draw_glyphs(&mut canvas, &run, 0.0, 0.0, 0xff33_6699).unwrap();

    assert_eq!(rgba(&canvas, 2, 2), [0x33, 0x66, 0x99, 0xff]);
    assert_eq!(rgba(&canvas, 4, 4), [0x00, 0x00, 0x00, 0x00]);
    assert_eq!(rgba(&canvas, 0, 0), [0x00, 0x00, 0x00, 0x00]);
}

#[test]
fn draw_glyphs_draws_rgba_raster_layers() {
    let raster = RasterGlyphLayer {
        source: RasterGlyphSource::Rgba {
            width: 2,
            height: 2,
            data: vec![
                0xff, 0x00, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
                0x00, 0x80,
            ],
        },
        offset_x: 1.0,
        offset_y: 2.0,
        width: None,
        height: None,
    };

    let glyph = Glyph::new(vec![GlyphLayer::Raster(raster)]);
    let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 2.0, 1.0)]);
    let mut canvas = Canvas::new(8, 8);

    draw_glyphs(&mut canvas, &run, 0.0, 0.0, 0xffff_ffff).unwrap();

    assert_eq!(rgba(&canvas, 3, 3), [0xff, 0x00, 0x00, 0xff]);
    assert_eq!(rgba(&canvas, 4, 3), [0x00, 0xff, 0x00, 0xff]);
    assert_eq!(rgba(&canvas, 3, 4), [0x00, 0x00, 0xff, 0xff]);
    assert_eq!(rgba(&canvas, 4, 4), [0x80, 0x80, 0x00, 0xff]);
}

#[test]
fn draw_glyphs_antialiases_diagonal_edges() {
    let commands = vec![
        Command::MoveTo(1.0, 1.0),
        Command::Line(8.0, 1.0),
        Command::Line(8.0, 8.0),
        Command::Close,
    ];

    let glyph = Glyph::new(vec![GlyphLayer::Path(PathGlyphLayer::new(
        commands,
        GlyphPaint::CurrentColor,
    ))]);
    let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 0.0, 0.0)]);
    let mut canvas = Canvas::new(10, 10);
    fillrect(&mut canvas, 0x00ff_ffff);

    draw_glyphs(&mut canvas, &run, 0.0, 0.0, 0xff00_0000).unwrap();

    let edge = rgba(&canvas, 3, 3);
    assert_eq!(edge[3], 0xff);
    assert!(edge[0] > 0x00 && edge[0] < 0xff);
    assert_eq!(edge[0], edge[1]);
    assert_eq!(edge[1], edge[2]);
}

fn append_cubic_circle(
    commands: &mut Vec<Command>,
    cx: f32,
    cy: f32,
    radius: f32,
    clockwise: bool,
) {
    let k = radius * 0.552_284_8;
    if clockwise {
        commands.push(Command::MoveTo(cx + radius, cy));
        commands.push(Command::CubicBezier(
            (cx + radius, cy + k),
            (cx + k, cy + radius),
            (cx, cy + radius),
        ));
        commands.push(Command::CubicBezier(
            (cx - k, cy + radius),
            (cx - radius, cy + k),
            (cx - radius, cy),
        ));
        commands.push(Command::CubicBezier(
            (cx - radius, cy - k),
            (cx - k, cy - radius),
            (cx, cy - radius),
        ));
        commands.push(Command::CubicBezier(
            (cx + k, cy - radius),
            (cx + radius, cy - k),
            (cx + radius, cy),
        ));
    } else {
        commands.push(Command::MoveTo(cx + radius, cy));
        commands.push(Command::CubicBezier(
            (cx + radius, cy - k),
            (cx + k, cy - radius),
            (cx, cy - radius),
        ));
        commands.push(Command::CubicBezier(
            (cx - k, cy - radius),
            (cx - radius, cy - k),
            (cx - radius, cy),
        ));
        commands.push(Command::CubicBezier(
            (cx - radius, cy + k),
            (cx - k, cy + radius),
            (cx, cy + radius),
        ));
        commands.push(Command::CubicBezier(
            (cx + k, cy + radius),
            (cx + radius, cy + k),
            (cx + radius, cy),
        ));
    }
    commands.push(Command::Close);
}

#[test]
fn draw_glyphs_antialiases_round_cubic_edges() {
    let mut commands = Vec::new();
    append_cubic_circle(&mut commands, 20.0, 20.0, 12.0, true);
    append_cubic_circle(&mut commands, 20.0, 20.0, 6.0, false);

    let glyph = Glyph::new(vec![GlyphLayer::Path(PathGlyphLayer::new(
        commands,
        GlyphPaint::CurrentColor,
    ))]);
    let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 0.0, 0.0)]);
    let mut canvas = Canvas::new(40, 40);
    fillrect(&mut canvas, 0x00ff_ffff);

    draw_glyphs(&mut canvas, &run, 0.0, 0.0, 0xff00_0000).unwrap();

    assert_eq!(rgba(&canvas, 20, 20), [0xff, 0xff, 0xff, 0xff]);
    assert_eq!(rgba(&canvas, 20, 10), [0x00, 0x00, 0x00, 0xff]);

    assert!(has_gray_pixel(&canvas, 27, 11, 31, 15));
    assert!(has_gray_pixel(&canvas, 23, 15, 26, 18));
}

#[test]
fn flatten_commands_does_not_subdivide_straight_cubic_per_half_pixel() {
    let commands = vec![
        Command::MoveTo(0.0, 0.0),
        Command::CubicBezier((40.0, 0.0), (80.0, 0.0), (120.0, 0.0)),
    ];

    let glyph = Glyph::new(vec![GlyphLayer::Path(PathGlyphLayer::stroke(
        commands,
        GlyphPaint::Solid(0xff00_0000),
        1.0,
    ))]);
    let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 0.0, 0.0)]);
    let mut canvas = Canvas::new(128, 3);

    draw_glyphs(&mut canvas, &run, 0.0, 1.0, 0xff00_0000).unwrap();

    let ink = (0..canvas.width())
        .filter(|&x| rgba(&canvas, x, 1)[3] > 0)
        .count();
    assert!(
        ink >= 100,
        "expected straight cubic stroke to render as a continuous line"
    );
}

#[test]
fn draw_glyphs_keeps_argb_path_colors() {
    let commands = vec![
        Command::MoveTo(1.0, 1.0),
        Command::Line(5.0, 1.0),
        Command::Line(5.0, 5.0),
        Command::Line(1.0, 5.0),
        Command::Close,
    ];

    let glyph = Glyph::new(vec![GlyphLayer::Path(PathGlyphLayer::new(
        commands,
        GlyphPaint::Solid(0xff11_2233),
    ))]);
    let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 0.0, 0.0)]);
    let mut canvas = Canvas::new(8, 8);

    draw_glyphs(&mut canvas, &run, 0.0, 0.0, 0xffff_ffff).unwrap();

    assert_eq!(rgba(&canvas, 2, 2), [0x11, 0x22, 0x33, 0xff]);
}

#[test]
fn draw_glyphs_accepts_opaque_rgba_path_colors() {
    let commands = vec![
        Command::MoveTo(1.0, 1.0),
        Command::Line(5.0, 1.0),
        Command::Line(5.0, 5.0),
        Command::Line(1.0, 5.0),
        Command::Close,
    ];

    let glyph = Glyph::new(vec![GlyphLayer::Path(PathGlyphLayer::new(
        commands,
        GlyphPaint::Solid(0x11_22_33_ff),
    ))]);
    let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 0.0, 0.0)]);
    let mut canvas = Canvas::new(8, 8);

    draw_glyphs(&mut canvas, &run, 0.0, 0.0, 0xffff_ffff).unwrap();

    assert_eq!(rgba(&canvas, 2, 2), [0x11, 0x22, 0x33, 0xff]);
}

#[test]
fn draw_glyphs_strokes_open_paths() {
    let glyph = Glyph::new(vec![GlyphLayer::Path(PathGlyphLayer::stroke(
        vec![Command::MoveTo(2.0, 2.0), Command::Line(8.0, 8.0)],
        GlyphPaint::Solid(0xff00_0000),
        2.0,
    ))]);
    let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 0.0, 0.0)]);
    let mut canvas = Canvas::new(12, 12);
    fillrect(&mut canvas, 0x00ff_ffff);

    draw_glyphs(&mut canvas, &run, 0.0, 0.0, 0xff00_0000).unwrap();

    let pixel = rgba(&canvas, 5, 5);
    assert_eq!(pixel[3], 0xff);
    assert!(pixel[0] < 0xff);
}

#[test]
fn draw_glyphs_fill_linear_gradient_interpolates_between_stops() {
    let gradient = LinearGradientPaint {
        x1: 1.0,
        y1: 0.0,
        x2: 7.0,
        y2: 0.0,
        units: GradientUnits::UserSpaceOnUse,
        transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        spread: GradientSpread::Pad,
        stops: vec![
            GradientStop {
                offset: 0.0,
                color: 0xffff_0000,
            },
            GradientStop {
                offset: 1.0,
                color: 0xff00_00ff,
            },
        ],
    };
    let glyph = Glyph::new(vec![GlyphLayer::Path(PathGlyphLayer::new(
        vec![
            Command::MoveTo(1.0, 1.0),
            Command::Line(7.0, 1.0),
            Command::Line(7.0, 7.0),
            Command::Line(1.0, 7.0),
            Command::Close,
        ],
        GlyphPaint::LinearGradient(gradient),
    ))]);
    let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 0.0, 0.0)]);
    let mut canvas = Canvas::new(10, 10);

    draw_glyphs(&mut canvas, &run, 0.0, 0.0, 0xff00_0000).unwrap();

    let left = rgba(&canvas, 2, 4);
    let right = rgba(&canvas, 6, 4);
    assert!(left[0] > right[0], "left side should stay redder");
    assert!(right[2] > left[2], "right side should become bluer");
}

#[test]
fn draw_glyphs_applies_clip_commands_to_fill_layers() {
    let mut layer = PathGlyphLayer::new(
        vec![
            Command::MoveTo(1.0, 1.0),
            Command::Line(8.0, 1.0),
            Command::Line(8.0, 8.0),
            Command::Line(1.0, 8.0),
            Command::Close,
        ],
        GlyphPaint::Solid(0xff00_0000),
    );
    layer.clip_commands = vec![
        Command::MoveTo(1.0, 1.0),
        Command::Line(4.0, 1.0),
        Command::Line(4.0, 8.0),
        Command::Line(1.0, 8.0),
        Command::Close,
    ];

    let glyph = Glyph::new(vec![GlyphLayer::Path(layer)]);
    let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 0.0, 0.0)]);
    let mut canvas = Canvas::new(10, 10);

    draw_glyphs(&mut canvas, &run, 0.0, 0.0, 0xff00_0000).unwrap();

    assert_eq!(rgba(&canvas, 2, 4), [0x00, 0x00, 0x00, 0xff]);
    assert_eq!(rgba(&canvas, 6, 4), [0x00, 0x00, 0x00, 0x00]);
}

#[cfg(feature = "font")]
#[test]
fn from_fontcore_path_layer_keeps_clip_commands() {
    let source = fontcore::PathGlyphLayer {
        commands: vec![
            fontcore::Command::MoveTo(1.0, 1.0),
            fontcore::Command::Line(5.0, 1.0),
            fontcore::Command::Close,
        ],
        clip_commands: vec![
            fontcore::Command::MoveTo(2.0, 2.0),
            fontcore::Command::Line(4.0, 2.0),
            fontcore::Command::Close,
        ],
        paint: fontcore::GlyphPaint::CurrentColor,
        paint_mode: fontcore::PathPaintMode::Fill,
        fill_rule: fontcore::FillRule::NonZero,
        stroke_width: 1.0,
        offset_x: 0.0,
        offset_y: 0.0,
    };

    let layer: PathGlyphLayer = source.into();
    assert_eq!(layer.clip_commands.len(), 3);
    assert!(matches!(layer.clip_commands[0], Command::MoveTo(2.0, 2.0)));
}

#[cfg(feature = "font")]
#[test]
fn from_fontcore_gradient_paint_keeps_gradient_variants() {
    let gradient = fontcore::GlyphLinearGradient {
        x1: 1.0,
        y1: 2.0,
        x2: 9.0,
        y2: 2.0,
        units: fontcore::GlyphGradientUnits::UserSpaceOnUse,
        transform: [1.0, 0.0, 0.0, 1.0, 3.0, 4.0],
        spread: fontcore::GlyphGradientSpread::Reflect,
        stops: vec![
            fontcore::GlyphGradientStop {
                offset: 0.0,
                color: 0xffff_0000,
            },
            fontcore::GlyphGradientStop {
                offset: 1.0,
                color: 0xff00_00ff,
            },
        ],
    };

    let paint: GlyphPaint = fontcore::GlyphPaint::LinearGradient(gradient).into();
    match paint {
        GlyphPaint::LinearGradient(gradient) => {
            assert_eq!(gradient.x1, 1.0);
            assert_eq!(gradient.x2, 9.0);
            assert!(matches!(gradient.spread, GradientSpread::Reflect));
            assert_eq!(gradient.stops.len(), 2);
            assert_eq!(gradient.stops[0].color, 0xffff_0000);
        }
        other => panic!("expected linear gradient, got {other:?}"),
    }
}

#[cfg(feature = "font")]
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[cfg(feature = "font")]
fn find_test_font_path(name: &str) -> Option<PathBuf> {
    let root = workspace_root();
    let candidates = [
        root.join(".test-fonts").join(name),
        root.join(".tmp-fonts").join(name),
        root.join(".tmp-font").join(name),
        PathBuf::from(r"C:\Windows\Fonts").join(name),
    ];

    candidates.into_iter().find(|path| path.exists())
}

#[cfg(feature = "font")]
fn load_test_font(name: &str) -> Option<LoadedFont> {
    let path = find_test_font_path(name)?;
    let buffer = std::fs::read(&path).ok()?;
    load_font_from_buffer(&buffer).ok()
}

#[cfg(feature = "font")]
fn into_local_run(run: fontcore::GlyphRun) -> GlyphRun {
    run.into()
}

#[cfg(all(feature = "font", feature = "svg-font"))]
fn find_svg_test_font(name: &str) -> Option<PathBuf> {
    let root = workspace_root();
    let candidates = [
        root.join(".test-fonts").join(name),
        root.join(".test_fonts").join(name),
    ];
    candidates.into_iter().find(|path| path.exists())
}

#[cfg(feature = "font")]
fn count_non_white_pixels(screen: &dyn Screen, x0: u32, y0: u32, x1: u32, y1: u32) -> usize {
    let x1 = x1.min(screen.width());
    let y1 = y1.min(screen.height());
    let mut count = 0;

    for y in y0.min(y1)..y1 {
        for x in x0.min(x1)..x1 {
            let pixel = rgba(screen, x, y);
            if pixel[0] != 0xff || pixel[1] != 0xff || pixel[2] != 0xff {
                count += 1;
            }
        }
    }

    count
}

#[cfg(feature = "font")]
#[test]
#[ignore = "diagnostic: inspect Yu Gothic round glyph flattening density"]
fn inspect_yu_gothic_round_glyph_flattening() {
    let Some(font) = load_test_font("YuGothB.ttc").or_else(|| load_test_font("YuGothB.ttf")) else {
        return;
    };

    let run = into_local_run(
        font.text2glyph_run(
            "CGOQ",
            fontcore::FontOptions::new(&font).with_font_size(64.0),
        )
        .expect("glyph run"),
    );

    for (index, glyph) in run.glyphs.iter().enumerate() {
        for (layer_index, layer) in glyph.glyph.layers.iter().enumerate() {
            if let GlyphLayer::Path(path) = layer {
                eprintln!(
                    "glyph {} layer {} commands={} offset=({}, {}) bounds={:?}",
                    index,
                    layer_index,
                    path.commands.len(),
                    glyph.x + path.offset_x,
                    glyph.y + path.offset_y,
                    glyph.glyph.metrics.bounds
                );
            }
        }
    }
}

#[cfg(feature = "font")]
#[test]
#[ignore = "diagnostic: currently fails on fontcore outline extraction for FiraSans-Black"]
fn font_reader_fira_black_text2command_still_has_commands() {
    let Some(font) = load_test_font("FiraSans-Black.ttf") else {
        return;
    };

    for ch in ['i', 'j'] {
        let commands = into_local_run(
            font.text2glyph_run(
                &ch.to_string(),
                fontcore::FontOptions::new(&font).with_font_size(64.0),
            )
            .expect("text2glyph_run should succeed"),
        );
        assert_eq!(commands.glyphs.len(), 1, "expected one glyph for {}", ch);
        assert!(
            commands.glyphs[0]
                .glyph
                .layers
                .iter()
                .any(|layer| matches!(layer, GlyphLayer::Path(path) if !path.commands.is_empty())),
            "text2glyph_run returned no commands for {}",
            ch
        );
    }
}

#[cfg(feature = "font")]
#[test]
#[ignore = "diagnostic: currently fails on fontcore glyph_run output for FiraSans-Black"]
fn font_reader_fira_black_i_and_j_have_outline_layers() {
    let Some(font) = load_test_font("FiraSans-Black.ttf") else {
        return;
    };

    let mut options = fontcore::FontOptions::new(&font);
    options.font_size = 64.0;
    let run = into_local_run(font.text2glyph_run("ij", options).expect("glyph run"));

    assert_eq!(run.glyphs.len(), 2, "expected two glyphs for 'ij'");
    for (index, glyph) in run.glyphs.iter().enumerate() {
        let path_layers: Vec<&PathGlyphLayer> = glyph
            .glyph
            .layers
            .iter()
            .filter_map(|layer| match layer {
                GlyphLayer::Path(path) => Some(path),
                GlyphLayer::Raster(_) => None,
                #[cfg(feature = "svg-font")]
                GlyphLayer::Svg(_) => None,
            })
            .collect();
        assert!(
            !path_layers.is_empty(),
            "font reader returned no outline layers for glyph index {}",
            index
        );
        assert!(
            path_layers.iter().any(|path| !path.commands.is_empty()),
            "font reader returned only empty outline layers for glyph index {}",
            index
        );
        assert!(
            glyph.glyph.metrics.bounds.is_some(),
            "font reader returned no bounds for glyph index {}",
            index
        );
    }
}

#[cfg(feature = "font")]
#[test]
#[ignore = "diagnostic: currently fails on fontcore glyph_run output for seguiemj"]
fn font_reader_segoe_emoji_has_colr_path_layers() {
    let Some(font) = load_test_font("seguiemj.ttf") else {
        return;
    };

    let mut options = fontcore::FontOptions::new(&font);
    options.font_size = 64.0;
    let run = into_local_run(font.text2glyph_run("🥺", options).expect("glyph run"));

    assert_eq!(run.glyphs.len(), 1, "expected one glyph for emoji");
    let mut solid_layers = 0usize;
    for layer in &run.glyphs[0].glyph.layers {
        if let GlyphLayer::Path(path) = layer {
            if matches!(path.paint, GlyphPaint::Solid(_)) {
                solid_layers += 1;
                assert!(
                    !path.commands.is_empty(),
                    "COLR path layer should have commands"
                );
            }
        }
    }

    assert!(solid_layers > 0, "expected solid COLR path layers");
}

#[cfg(feature = "font")]
#[test]
fn segoe_emoji_colr_layers_resolve_to_opaque_argb() {
    let Some(font) = load_test_font("seguiemj.ttf") else {
        return;
    };

    let mut options = fontcore::FontOptions::new(&font);
    options.font_size = 64.0;
    let run = into_local_run(font.text2glyph_run("🥺", options).expect("glyph run"));

    let mut found_solid = false;
    for glyph in &run.glyphs {
        for layer in &glyph.glyph.layers {
            if let GlyphLayer::Path(path) = layer {
                if let GlyphPaint::Solid(color) = path.paint {
                    found_solid = true;
                    let glyph = Glyph::new(vec![GlyphLayer::Path(PathGlyphLayer::new(
                        vec![
                            Command::MoveTo(1.0, 1.0),
                            Command::Line(3.0, 1.0),
                            Command::Line(3.0, 3.0),
                            Command::Line(1.0, 3.0),
                            Command::Close,
                        ],
                        GlyphPaint::Solid(color),
                    ))]);
                    let run = GlyphRun::new(vec![PositionedGlyph::new(glyph, 0.0, 0.0)]);
                    let mut canvas = Canvas::new(5, 5);
                    draw_glyphs(&mut canvas, &run, 0.0, 0.0, 0xff00_0000)
                        .expect("draw COLR color sample");
                    assert_eq!(
                        rgba(&canvas, 2, 2)[3],
                        0xff,
                        "COLR layer colors should render as opaque ARGB"
                    );
                }
            }
        }
    }

    assert!(found_solid, "expected at least one solid COLR layer");
}

#[cfg(feature = "font")]
#[test]
fn draw_text_with_family_renders_cached_face() {
    let Some(font) = load_test_font("FiraSans-Black.ttf") else {
        return;
    };
    let mut family = FontFamily::new("Fira Sans");
    family.add_face(
        FontFaceDescriptor::new("Fira Sans")
            .with_font_name("Fira Sans Black")
            .with_font_weight(FontWeight::BLACK),
        font,
    );

    let mut canvas = Canvas::new(160, 120);
    fillrect(&mut canvas, 0x00ff_ffff);

    let glyphs = draw_text_with_family(
        &mut canvas,
        &family,
        "A",
        FontOptions::from_family(&family)
            .with_font_family("Fira Sans")
            .with_font_weight(FontWeight::BLACK)
            .with_font_size(48.0),
        16.0,
        72.0,
        0xff11_1111,
    )
    .expect("draw text from font family");

    assert_eq!(glyphs.glyphs.len(), 1);
    let ink = count_non_white_pixels(&canvas, 0, 0, canvas.width(), canvas.height());
    assert!(ink > 0, "expected rendered ink from cached family face");
}

#[cfg(feature = "font")]
#[test]
#[ignore = "diagnostic: compare direct loaded font against FontFamily face resolution"]
fn compare_direct_font_and_family_yugothb() {
    let Some(font) = load_test_font("YuGothB.ttc").or_else(|| load_test_font("YuGothB.ttf")) else {
        return;
    };

    let direct = into_local_run(
        font.text2glyph_run(
            "CGO",
            fontcore::FontOptions::new(&font).with_font_size(64.0),
        )
        .expect("direct glyph run"),
    );

    let mut family_auto = FontFamily::new("Yu Gothic");
    family_auto.add_font_face(font.clone());
    let auto = into_local_run(
        family_auto
            .text2glyph_run(
                "CGO",
                FontOptions::from_family(&family_auto)
                    .with_font_family("Yu Gothic")
                    .with_font_size(64.0)
                    .with_font_weight(FontWeight::BOLD),
            )
            .expect("auto family glyph run"),
    );

    let mut family_face = FontFamily::new("Yu Gothic");
    family_face.add_face(FontFaceDescriptor::from_face(&font), font);
    let face = into_local_run(
        family_face
            .text2glyph_run(
                "CGO",
                FontOptions::from_family(&family_face)
                    .with_font_family("Yu Gothic")
                    .with_font_size(64.0)
                    .with_font_weight(FontWeight::BOLD),
            )
            .expect("descriptor family glyph run"),
    );

    fn summarize(run: &GlyphRun) -> Vec<(usize, usize, usize, Option<(i32, i32, i32, i32)>)> {
        run.glyphs
            .iter()
            .map(|glyph| {
                let mut layers = 0usize;
                let mut commands = 0usize;
                for layer in &glyph.glyph.layers {
                    if let GlyphLayer::Path(path) = layer {
                        layers += 1;
                        commands += path.commands.len();
                    }
                }
                let bounds = glyph.glyph.metrics.bounds.map(|bounds| {
                    (
                        (bounds.min_x * 1024.0).round() as i32,
                        (bounds.min_y * 1024.0).round() as i32,
                        (bounds.max_x * 1024.0).round() as i32,
                        (bounds.max_y * 1024.0).round() as i32,
                    )
                });
                (layers, commands, glyph.glyph.layers.len(), bounds)
            })
            .collect()
    }

    eprintln!("direct={:?}", summarize(&direct));
    eprintln!("auto={:?}", summarize(&auto));
    eprintln!("face={:?}", summarize(&face));

    assert_eq!(summarize(&direct), summarize(&auto));
    assert_eq!(summarize(&direct), summarize(&face));
}

#[cfg(feature = "font")]
#[test]
fn twemoji_sbix_woff2_loads_from_buffer_and_emits_raster_layers() {
    let Some(path) = find_test_font_path("TwemojiMozilla-sbix.woff2") else {
        return;
    };
    let bytes = std::fs::read(&path).expect("read TwemojiMozilla-sbix.woff2");
    let font = load_font_from_buffer(&bytes).expect("load TwemojiMozilla-sbix.woff2");

    let run = into_local_run(
        font.text2glyph_run("😀", fontcore::FontOptions::new(&font).with_font_size(96.0))
            .expect("build glyph run for sbix font"),
    );

    assert!(
        !run.glyphs.is_empty(),
        "expected at least one glyph from TwemojiMozilla-sbix.woff2"
    );
    assert!(
        run.glyphs
            .iter()
            .flat_map(|glyph| glyph.glyph.layers.iter())
            .any(|layer| matches!(layer, GlyphLayer::Raster(_))),
        "expected sbix font to emit raster glyph layers"
    );
}

#[cfg(feature = "font")]
#[test]
fn twemoji_sbix_woff2_promotes_chunked_face_into_family_cache() {
    let Some(path) = find_test_font_path("TwemojiMozilla-sbix.woff2") else {
        return;
    };
    let bytes = std::fs::read(&path).expect("read TwemojiMozilla-sbix.woff2");

    let mut family = FontFamily::new("Twemoji Mozilla");
    family
        .begin_chunked_face(
            "twemoji-sbix",
            FontFaceDescriptor::new("Twemoji Mozilla")
                .with_font_name("Twemoji Mozilla sbix")
                .with_font_weight(FontWeight::NORMAL),
            bytes.len(),
        )
        .expect("begin chunked sbix face");

    let chunk = 256 * 1024;
    let mut offset = 0;
    while offset < bytes.len() {
        let end = (offset + chunk).min(bytes.len());
        family
            .append_chunk("twemoji-sbix", offset, &bytes[offset..end])
            .expect("append sbix chunk");
        offset = end;
    }

    family
        .finalize_chunked_face("twemoji-sbix")
        .expect("finalize chunked sbix face");
    assert_eq!(family.cached_faces_len(), 1);

    let run = into_local_run(
        family
            .text2glyph_run("😀", FontOptions::from_family(&family).with_font_size(96.0))
            .expect("layout sbix glyph from chunked family"),
    );
    assert!(
        run.glyphs
            .iter()
            .flat_map(|glyph| glyph.glyph.layers.iter())
            .any(|layer| matches!(layer, GlyphLayer::Raster(_))),
        "expected chunked sbix family load to keep raster glyph layers"
    );
}

#[cfg(feature = "font")]
#[test]
fn twemoji_sbix_woff2_raster_layers_draw_on_canvas() {
    let Some(path) = find_test_font_path("TwemojiMozilla-sbix.woff2") else {
        return;
    };
    let bytes = std::fs::read(&path).expect("read TwemojiMozilla-sbix.woff2");
    let font = load_font_from_buffer(&bytes).expect("load TwemojiMozilla-sbix.woff2");
    let run = into_local_run(
        font.text2glyph_run("😀", fontcore::FontOptions::new(&font).with_font_size(96.0))
            .expect("build glyph run for sbix font"),
    );

    let mut canvas = Canvas::new(256, 256);
    fillrect(&mut canvas, 0x00ff_ffff);
    draw_glyphs(&mut canvas, &run, 32.0, 128.0, 0xff11_1111).expect("draw sbix glyph");

    let ink = count_non_white_pixels(&canvas, 0, 0, canvas.width(), canvas.height());
    assert!(ink > 0, "expected rendered pixels from sbix raster glyph");
}

#[cfg(all(feature = "font", feature = "svg-font"))]
#[test]
fn svg_font_feature_exposes_svg_layer_for_emojione_color() {
    let Some(path) = find_svg_test_font("EmojiOneColor.otf") else {
        return;
    };
    let bytes = std::fs::read(&path).expect("read EmojiOneColor.otf");
    let font = load_font_from_buffer(&bytes).expect("load EmojiOneColor.otf");
    let run = into_local_run(
        font.text2glyph_run("😀", fontcore::FontOptions::new(&font).with_font_size(32.0))
            .expect("glyph run for EmojiOneColor.otf"),
    );

    assert_eq!(run.glyphs.len(), 1);
    assert!(
        run.glyphs[0].glyph.layers.iter().any(|layer| {
            matches!(layer, GlyphLayer::Path(path) if !path.commands.is_empty())
                || matches!(layer, GlyphLayer::Svg(svg) if !svg.document.is_empty())
        }),
        "expected usable svg-derived layer from EmojiOneColor.otf"
    );
}

#[cfg(all(feature = "font", feature = "svg-font"))]
#[test]
fn svg_font_feature_exposes_svg_layer_for_noto_color_emoji() {
    let Some(path) = find_svg_test_font("NotoColorEmoji-Regular.ttf") else {
        return;
    };
    let bytes = std::fs::read(&path).expect("read NotoColorEmoji-Regular.ttf");
    let font = load_font_from_buffer(&bytes).expect("load NotoColorEmoji-Regular.ttf");
    let run = into_local_run(
        font.text2glyph_run("😀", fontcore::FontOptions::new(&font).with_font_size(32.0))
            .expect("glyph run for NotoColorEmoji-Regular.ttf"),
    );

    assert_eq!(run.glyphs.len(), 1);
    assert!(
        run.glyphs[0].glyph.layers.iter().any(|layer| {
            matches!(layer, GlyphLayer::Path(path) if !path.commands.is_empty())
                || matches!(layer, GlyphLayer::Svg(svg) if !svg.document.is_empty())
        }),
        "expected usable svg-derived layer from NotoColorEmoji-Regular.ttf"
    );
}

#[cfg(feature = "font")]
#[test]
#[ignore = "diagnostic: currently fails because FiraSans-Black glyph_run has empty bounds/paths"]
fn composite_lowercase_glyphs_render_with_visible_ink_when_fira_is_available() {
    let Some(font) = load_test_font("FiraSans-Black.ttf") else {
        return;
    };

    let mut options = fontcore::FontOptions::new(&font);
    options.font_size = 64.0;
    let run = into_local_run(font.text2glyph_run("ij", options).expect("glyph run"));

    let mut canvas = Canvas::new(256, 160);
    fillrect(&mut canvas, 0x00ff_ffff);
    draw_glyphs(&mut canvas, &run, 24.0, 96.0, 0xff11_1111).unwrap();

    for (index, glyph) in run.glyphs.iter().enumerate() {
        let bounds = glyph.glyph.metrics.bounds.expect("glyph bounds");
        let min_x = (24.0 + glyph.x + bounds.min_x - 2.0).floor().max(0.0) as u32;
        let max_x = (24.0 + glyph.x + bounds.max_x + 2.0).ceil().max(0.0) as u32;
        let min_y = (96.0 + glyph.y + bounds.min_y - 2.0).floor().max(0.0) as u32;
        let max_y = (96.0 + glyph.y + bounds.max_y + 2.0).ceil().max(0.0) as u32;
        let ink = count_non_white_pixels(&canvas, min_x, min_y, max_x, max_y);
        assert!(
            ink > 0,
            "renderer left no visible ink for glyph index {} in its own bounds",
            index
        );
    }
}
