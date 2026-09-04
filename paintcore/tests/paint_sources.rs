use std::f32::consts::PI;

use paintcore::canvas::Screen;
use paintcore::composite::DrawOptions;
use paintcore::layer::Layer;
use paintcore::paint::{
    fill_paint_rect, Color, ColorStop, GradientInterpolation, LinearGradient, Paint, PaintBounds,
    PaintTransform, PaintUnits, Pattern, PatternImage, RadialGradient, SamplingMode, SpreadMode,
    SweepGradient, TileMode,
};

fn stops() -> Vec<ColorStop> {
    vec![
        ColorStop::new(0.0, Color::rgb(255, 0, 0)),
        ColorStop::new(1.0, Color::rgb(0, 0, 255)),
    ]
}

fn linear(units: PaintUnits, spread: SpreadMode) -> Paint {
    Paint::LinearGradient(LinearGradient {
        start: (0.0, 0.0),
        end: (1.0, 0.0),
        stops: stops(),
        spread,
        units,
        transform: PaintTransform::IDENTITY,
        interpolation: GradientInterpolation::Srgb,
    })
}

#[test]
fn linear_gradient_honors_object_bounding_box() {
    let paint = linear(PaintUnits::ObjectBoundingBox, SpreadMode::Pad);
    let prepared = paint.prepare(PaintBounds::new(10.0, 20.0, 100.0, 50.0));
    let color = prepared.sample(35.0, 45.0);
    assert!((color.red as i16 - 191).abs() <= 1);
    assert!((color.blue as i16 - 64).abs() <= 1);
}

#[test]
fn repeat_and_reflect_spreads_map_outside_values() {
    let repeat = linear(PaintUnits::UserSpaceOnUse, SpreadMode::Repeat)
        .prepare(PaintBounds::new(0.0, 0.0, 1.0, 1.0));
    let reflect = linear(PaintUnits::UserSpaceOnUse, SpreadMode::Reflect)
        .prepare(PaintBounds::new(0.0, 0.0, 1.0, 1.0));
    assert_eq!(repeat.sample(1.25, 0.0), Color::rgba(191, 0, 64, 255));
    assert_eq!(reflect.sample(1.25, 0.0), Color::rgba(64, 0, 191, 255));
}

#[test]
fn radial_and_sweep_gradients_sample_expected_geometry() {
    let radial = Paint::RadialGradient(RadialGradient {
        center: (0.5, 0.5),
        radius: 0.5,
        focal: (0.5, 0.5),
        focal_radius: 0.0,
        stops: stops(),
        spread: SpreadMode::Pad,
        units: PaintUnits::ObjectBoundingBox,
        transform: PaintTransform::IDENTITY,
        interpolation: GradientInterpolation::Srgb,
    })
    .prepare(PaintBounds::new(0.0, 0.0, 100.0, 50.0));
    assert_eq!(radial.sample(100.0, 25.0), Color::rgb(0, 0, 255));

    let sweep = Paint::SweepGradient(SweepGradient {
        center: (0.0, 0.0),
        start_angle: -PI,
        end_angle: PI,
        stops: stops(),
        spread: SpreadMode::Pad,
        units: PaintUnits::UserSpaceOnUse,
        transform: PaintTransform::IDENTITY,
        interpolation: GradientInterpolation::Srgb,
    })
    .prepare(PaintBounds::new(-1.0, -1.0, 2.0, 2.0));
    assert_eq!(sweep.sample(1.0, 0.0), Color::rgba(128, 0, 128, 255));

    let full_turn = Paint::SweepGradient(SweepGradient {
        center: (0.0, 0.0),
        start_angle: 0.0,
        end_angle: std::f32::consts::TAU,
        stops: stops(),
        spread: SpreadMode::Pad,
        units: PaintUnits::UserSpaceOnUse,
        transform: PaintTransform::IDENTITY,
        interpolation: GradientInterpolation::Srgb,
    })
    .prepare(PaintBounds::new(-1.0, -1.0, 2.0, 2.0));
    let lower_half = full_turn.sample(0.0, -1.0);
    assert!(lower_half.blue > lower_half.red);
}

#[test]
fn transparent_stops_interpolate_without_hidden_rgb_bleed() {
    let paint = Paint::LinearGradient(LinearGradient {
        start: (0.0, 0.0),
        end: (1.0, 0.0),
        stops: vec![
            ColorStop::new(0.0, Color::rgba(255, 0, 0, 0)),
            ColorStop::new(1.0, Color::rgba(0, 0, 255, 255)),
        ],
        spread: SpreadMode::Pad,
        units: PaintUnits::UserSpaceOnUse,
        transform: PaintTransform::IDENTITY,
        interpolation: GradientInterpolation::LinearSrgb,
    });
    let color = paint
        .prepare(PaintBounds::new(0.0, 0.0, 1.0, 1.0))
        .sample(0.5, 0.0);
    assert_eq!(color.red, 0);
    assert_eq!(color.blue, 255);
    assert_eq!(color.alpha, 128);
}

fn pattern_image() -> PatternImage {
    PatternImage::new(
        2,
        2,
        vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ],
    )
    .unwrap()
}

#[test]
fn pattern_supports_negative_repeat_and_mirror_coordinates() {
    let repeat = Paint::Pattern(Pattern {
        image: pattern_image(),
        tile_x: TileMode::Repeat,
        tile_y: TileMode::Repeat,
        sampling: SamplingMode::Nearest,
        transform: PaintTransform::IDENTITY,
    })
    .prepare(PaintBounds::new(0.0, 0.0, 4.0, 4.0));
    assert_eq!(repeat.sample(-0.5, 0.5), Color::rgb(0, 255, 0));

    let mirror = Paint::Pattern(Pattern {
        image: pattern_image(),
        tile_x: TileMode::Mirror,
        tile_y: TileMode::Mirror,
        sampling: SamplingMode::Nearest,
        transform: PaintTransform::IDENTITY,
    })
    .prepare(PaintBounds::new(0.0, 0.0, 4.0, 4.0));
    assert_eq!(mirror.sample(2.5, 0.5), Color::rgb(0, 255, 0));
}

#[test]
fn bilinear_pattern_sampling_is_alpha_aware() {
    let image = PatternImage::new(
        2,
        2,
        vec![255, 0, 0, 0, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255],
    )
    .unwrap();
    let pattern = Paint::Pattern(Pattern {
        image,
        tile_x: TileMode::Clamp,
        tile_y: TileMode::Clamp,
        sampling: SamplingMode::Bilinear,
        transform: PaintTransform::IDENTITY,
    })
    .prepare(PaintBounds::new(0.0, 0.0, 2.0, 2.0));
    let color = pattern.sample(1.0, 1.0);
    assert_eq!(color.red, 0);
    assert_eq!(color.blue, 255);
    assert_eq!(color.alpha, 191);
}

#[test]
fn pattern_constructor_rejects_invalid_buffers() {
    assert!(PatternImage::new(2, 2, vec![0; 15]).is_err());
    assert!(PatternImage::new(0, 0, Vec::new()).is_err());
}

#[test]
fn fill_paint_rect_uses_one_prepared_paint_for_the_region() {
    let mut layer = Layer::new("gradient".to_string(), 4, 1);
    let paint = Paint::LinearGradient(LinearGradient {
        start: (0.0, 0.0),
        end: (4.0, 0.0),
        stops: stops(),
        spread: SpreadMode::Pad,
        units: PaintUnits::UserSpaceOnUse,
        transform: PaintTransform::IDENTITY,
        interpolation: GradientInterpolation::Srgb,
    });
    fill_paint_rect(&mut layer, 0, 0, 4, 1, &paint, DrawOptions::default());
    assert!(layer.buffer()[0] > layer.buffer()[12]);
    assert!(layer.buffer()[2] < layer.buffer()[14]);
    assert_eq!(layer.buffer()[3], 255);
}
