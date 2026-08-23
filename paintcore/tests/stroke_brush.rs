use paintcore::prelude::*;

fn line_commands() -> Vec<Command> {
    vec![Command::MoveTo(10.0, 10.0), Command::Line(30.0, 10.0)]
}

#[test]
fn stroke_caps_have_distinct_endpoint_coverage() {
    let mut style = StrokeStyle {
        width: 8.0,
        ..StrokeStyle::default()
    };
    style.cap = StrokeCap::Butt;
    let butt = rasterize_stroke_mask(&line_commands(), 48, 24, 0.0, 0.0, &style);
    style.cap = StrokeCap::Round;
    let round = rasterize_stroke_mask(&line_commands(), 48, 24, 0.0, 0.0, &style);
    style.cap = StrokeCap::Square;
    let square = rasterize_stroke_mask(&line_commands(), 48, 24, 0.0, 0.0, &style);
    assert_eq!(butt.get(7, 10), 0);
    assert!(round.get(7, 10) > 0);
    assert!(square.get(7, 10) > 0);
}

#[test]
fn miter_limit_falls_back_to_bevel() {
    let commands = vec![
        Command::MoveTo(10.0, 30.0),
        Command::Line(20.0, 20.0),
        Command::Line(30.0, 30.0),
    ];
    let sharp = rasterize_stroke_mask(
        &commands,
        40,
        40,
        0.0,
        0.0,
        &StrokeStyle {
            width: 10.0,
            join: StrokeJoin::Miter,
            miter_limit: 10.0,
            ..StrokeStyle::default()
        },
    );
    let limited = rasterize_stroke_mask(
        &commands,
        40,
        40,
        0.0,
        0.0,
        &StrokeStyle {
            width: 10.0,
            join: StrokeJoin::Miter,
            miter_limit: 1.0,
            ..StrokeStyle::default()
        },
    );
    assert!(sharp.get(20, 13) > limited.get(20, 13));
}

#[test]
fn dash_and_offset_define_on_off_intervals() {
    let commands = vec![Command::MoveTo(5.0, 10.0), Command::Line(45.0, 10.0)];
    let style = StrokeStyle {
        width: 4.0,
        cap: StrokeCap::Butt,
        dash: vec![5.0, 5.0],
        ..StrokeStyle::default()
    };
    let mask = rasterize_stroke_mask(&commands, 52, 20, 0.0, 0.0, &style);
    assert!(mask.get(7, 10) > 0);
    assert_eq!(mask.get(12, 10), 0);

    let offset = rasterize_stroke_mask(
        &commands,
        52,
        20,
        0.0,
        0.0,
        &StrokeStyle {
            dash_offset: 5.0,
            ..style
        },
    );
    assert_eq!(offset.get(7, 10), 0);
    assert!(offset.get(12, 10) > 0);
}

#[test]
fn public_fill_path_honors_fill_rule_and_paint() {
    let commands = vec![
        Command::MoveTo(2.0, 2.0),
        Command::Line(18.0, 2.0),
        Command::Line(18.0, 18.0),
        Command::Line(2.0, 18.0),
        Command::Close,
        Command::MoveTo(6.0, 6.0),
        Command::Line(14.0, 6.0),
        Command::Line(14.0, 14.0),
        Command::Line(6.0, 14.0),
        Command::Close,
    ];
    let even_odd = rasterize_path_mask(&commands, 20, 20, 0.0, 0.0, FillRule::EvenOdd);
    let non_zero = rasterize_path_mask(&commands, 20, 20, 0.0, 0.0, FillRule::NonZero);
    assert_eq!(even_odd.get(10, 10), 0);
    assert!(non_zero.get(10, 10) > 0);

    let mut target = Layer::new("path".to_string(), 20, 20);
    fill_path(
        &mut target,
        &commands,
        &Paint::Solid(Color::rgb(20, 40, 60)),
        FillRule::EvenOdd,
        0.0,
        0.0,
        DrawOptions::default(),
    );
    assert_eq!(&target.buffer()[0..4], &[0, 0, 0, 0]);
    assert_eq!(target.buffer()[(3 * 20 + 3) * 4], 20);
}

#[test]
fn brush_spacing_is_distance_based_and_input_frequency_independent() {
    let settings = BrushSettings {
        size: 10.0,
        spacing: 0.5,
        scatter: 0.0,
        ..BrushSettings::default()
    };
    let mut sparse = BrushStroke::new(BrushTip::circle(8), settings.clone(), 7);
    let mut sparse_dabs = sparse.push_sample(StrokeSample::new(0.0, 0.0, 1.0, 0.0));
    sparse_dabs.extend(sparse.push_sample(StrokeSample::new(20.0, 0.0, 1.0, 20.0)));

    let mut dense = BrushStroke::new(BrushTip::circle(8), settings, 7);
    let mut dense_dabs = dense.push_sample(StrokeSample::new(0.0, 0.0, 1.0, 0.0));
    for value in [7.0, 13.0, 20.0] {
        dense_dabs.extend(dense.push_sample(StrokeSample::new(value, 0.0, 1.0, value as f64)));
    }
    assert_eq!(sparse_dabs.len(), dense_dabs.len());
    for (sparse, dense) in sparse_dabs.iter().zip(&dense_dabs) {
        assert!((sparse.x - dense.x).abs() < 1.0e-5);
        assert!((sparse.y - dense.y).abs() < 1.0e-5);
        assert!((sparse.timestamp - dense.timestamp).abs() < 1.0e-5);
        assert_eq!(sparse.size, dense.size);
    }
    assert_eq!(sparse_dabs.len(), 5);
    assert_eq!(sparse_dabs.last().unwrap().x, 20.0);
}

#[test]
fn pressure_maps_size_opacity_and_flow() {
    let mut stroke = BrushStroke::new(
        BrushTip::rectangle(2, 2),
        BrushSettings {
            size: 20.0,
            opacity: 0.8,
            flow: 0.5,
            pressure: PressureMapping {
                size: 1.0,
                opacity: 1.0,
                flow: 1.0,
            },
            ..BrushSettings::default()
        },
        1,
    );
    let dab = stroke.push_sample(StrokeSample::new(5.0, 5.0, 0.5, 0.0))[0];
    assert_eq!(dab.size, 10.0);
    assert!((dab.opacity - 0.4).abs() < 1.0e-6);
    assert!((dab.flow - 0.25).abs() < 1.0e-6);
}

#[test]
fn brush_flow_and_opacity_use_premultiplied_compositing() {
    let mut target = Layer::new("flow".to_string(), 16, 16);
    draw_brush_dabs(
        &mut target,
        &BrushTip::rectangle(4, 4),
        &[BrushDab {
            x: 8.0,
            y: 8.0,
            size: 4.0,
            opacity: 0.5,
            flow: 0.5,
            rotation: 0.0,
            timestamp: 0.0,
        }],
        &Paint::Solid(Color::rgb(255, 0, 0)),
        DrawOptions::default(),
    );
    let alpha = target.buffer()[(7 * 16 + 7) * 4 + 3];
    assert!((63..=64).contains(&alpha));
}

#[test]
fn brush_dabs_accept_gradient_and_pattern_paints() {
    let tip = BrushTip::rectangle(8, 8);
    let dabs = [BrushDab {
        x: 8.0,
        y: 8.0,
        size: 8.0,
        opacity: 1.0,
        flow: 1.0,
        rotation: 0.0,
        timestamp: 0.0,
    }];
    let mut gradient_target = Layer::new("gradient".to_string(), 16, 16);
    draw_brush_dabs(
        &mut gradient_target,
        &tip,
        &dabs,
        &Paint::LinearGradient(LinearGradient {
            start: (0.0, 0.0),
            end: (16.0, 0.0),
            stops: vec![
                ColorStop::new(0.0, Color::rgb(255, 0, 0)),
                ColorStop::new(1.0, Color::rgb(0, 0, 255)),
            ],
            spread: SpreadMode::Pad,
            units: PaintUnits::UserSpaceOnUse,
            transform: PaintTransform::IDENTITY,
            interpolation: GradientInterpolation::Srgb,
        }),
        DrawOptions::default(),
    );
    assert!(
        gradient_target.buffer()[(8 * 16 + 5) * 4] > gradient_target.buffer()[(8 * 16 + 10) * 4]
    );

    let mut pattern_target = Layer::new("pattern".to_string(), 16, 16);
    let image = PatternImage::new(2, 1, vec![255, 255, 255, 255, 0, 0, 0, 255]).unwrap();
    draw_brush_dabs(
        &mut pattern_target,
        &tip,
        &dabs,
        &Paint::Pattern(Pattern {
            image,
            tile_x: TileMode::Repeat,
            tile_y: TileMode::Repeat,
            sampling: SamplingMode::Nearest,
            transform: PaintTransform::IDENTITY,
        }),
        DrawOptions::default(),
    );
    assert_ne!(
        pattern_target.buffer()[(8 * 16 + 6) * 4],
        pattern_target.buffer()[(8 * 16 + 7) * 4]
    );
}

#[test]
fn scatter_is_seeded_and_rotation_changes_rectangular_tip() {
    let settings = BrushSettings {
        size: 12.0,
        spacing: 0.25,
        scatter: 0.5,
        ..BrushSettings::default()
    };
    let samples = [
        StrokeSample::new(16.0, 16.0, 1.0, 0.0),
        StrokeSample::new(32.0, 16.0, 1.0, 1.0),
    ];
    let make_dabs = |seed| {
        let mut stroke = BrushStroke::new(BrushTip::circle(8), settings.clone(), seed);
        samples
            .into_iter()
            .flat_map(|sample| stroke.push_sample(sample))
            .collect::<Vec<_>>()
    };
    assert_eq!(make_dabs(42), make_dabs(42));
    assert_ne!(make_dabs(42), make_dabs(43));

    let tip = BrushTip::rectangle(8, 2);
    let mut horizontal = Layer::new("horizontal".to_string(), 32, 32);
    let mut vertical = Layer::new("vertical".to_string(), 32, 32);
    let dab = BrushDab {
        x: 16.0,
        y: 16.0,
        size: 16.0,
        opacity: 1.0,
        flow: 1.0,
        rotation: 0.0,
        timestamp: 0.0,
    };
    draw_brush_dabs(
        &mut horizontal,
        &tip,
        &[dab],
        &Paint::Solid(Color::rgb(255, 255, 255)),
        DrawOptions::default(),
    );
    draw_brush_dabs(
        &mut vertical,
        &tip,
        &[BrushDab {
            rotation: std::f32::consts::FRAC_PI_2,
            ..dab
        }],
        &Paint::Solid(Color::rgb(255, 255, 255)),
        DrawOptions::default(),
    );
    assert!(horizontal.buffer()[(16 * 32 + 22) * 4 + 3] > 0);
    assert_eq!(vertical.buffer()[(16 * 32 + 22) * 4 + 3], 0);
    assert!(vertical.buffer()[(22 * 32 + 16) * 4 + 3] > 0);
}

#[test]
fn brush_tip_constructors_and_nonfinite_samples_are_safe() {
    let pen = Pen::square_pen(3).unwrap();
    assert_eq!(BrushTip::from_pen(&pen).unwrap().mask().coverage().len(), 9);
    assert!(BrushTip::circle(0).mask().is_empty());
    let mut stroke = BrushStroke::new(
        BrushTip::from_mask(Mask::from_rect(1, 1, 0, 0, 1, 1)),
        BrushSettings::default(),
        1,
    );
    assert!(stroke
        .push_sample(StrokeSample::new(f32::NAN, 0.0, 1.0, 0.0))
        .is_empty());
}
