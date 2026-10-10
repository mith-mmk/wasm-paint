use paintcore::canvas::Screen;
use paintcore::composite::{
    composite_pixel, composite_screen, fill_rect_with_options, BlendMode, ClipMask, CompositeOp,
    DrawOptions,
};
use paintcore::layer::Layer;
use paintcore::mask::Mask;
use paintcore::paint::Color;

#[test]
fn explicit_color_conversions_do_not_guess_channel_order() {
    assert_eq!(
        Color::from_argb_u32(0x8040_2010),
        Color::rgba(0x40, 0x20, 0x10, 0x80)
    );
    assert_eq!(
        Color::from_rgba_u32(0x4020_1080),
        Color::rgba(0x40, 0x20, 0x10, 0x80)
    );
    assert_eq!(
        Color::rgba(0x40, 0x20, 0x10, 0x80).to_argb_u32(),
        0x8040_2010
    );
}

#[test]
fn source_over_preserves_transparent_destination_alpha() {
    let result = composite_pixel(
        Color::rgba(0, 0, 255, 64),
        Color::rgba(255, 0, 0, 128),
        1.0,
        DrawOptions::default(),
    );
    assert_eq!(result.alpha, 160);
    assert!((result.red as i16 - 204).abs() <= 1);
    assert!((result.blue as i16 - 51).abs() <= 1);
}

#[test]
fn porter_duff_source_in_uses_destination_alpha() {
    let result = composite_pixel(
        Color::rgba(0, 0, 255, 64),
        Color::rgba(255, 0, 0, 128),
        1.0,
        DrawOptions {
            composite_op: CompositeOp::SourceIn,
            ..DrawOptions::default()
        },
    );
    assert_eq!(result, Color::rgba(255, 0, 0, 32));
}

#[test]
fn multiply_blends_opaque_channels() {
    let result = composite_pixel(
        Color::rgb(128, 200, 64),
        Color::rgb(200, 128, 128),
        1.0,
        DrawOptions {
            blend_mode: BlendMode::Multiply,
            ..DrawOptions::default()
        },
    );
    assert_eq!(result, Color::rgb(100, 100, 32));
}

#[test]
fn composite_screen_respects_layer_alpha_and_offsets() {
    let mut source = Layer::new("src".to_string(), 1, 1);
    source.buffer_mut().copy_from_slice(&[255, 0, 0, 128]);
    source.set_alpha(128);
    let mut dest = Layer::new("dst".to_string(), 2, 1);

    composite_screen(&source, &mut dest, 1, 0, DrawOptions::default());

    assert_eq!(&dest.buffer()[0..4], &[0, 0, 0, 0]);
    assert_eq!(&dest.buffer()[4..7], &[255, 0, 0]);
    assert_eq!(dest.buffer()[7], 64);
}

#[test]
fn fill_rect_clips_without_panicking() {
    let mut layer = Layer::new("rect".to_string(), 2, 2);
    fill_rect_with_options(
        &mut layer,
        -1,
        -1,
        2,
        2,
        Color::rgb(10, 20, 30),
        DrawOptions::default(),
    );
    assert_eq!(&layer.buffer()[0..4], &[10, 20, 30, 255]);
    assert_eq!(&layer.buffer()[4..8], &[0, 0, 0, 0]);
}

#[test]
fn clear_preserves_the_uncovered_destination() {
    let dest = Color::rgb(20, 40, 60);
    let options = DrawOptions {
        composite_op: CompositeOp::Clear,
        ..DrawOptions::default()
    };
    assert_eq!(
        composite_pixel(dest, Color::TRANSPARENT, 0.0, options),
        dest
    );
    assert_eq!(
        composite_pixel(dest, Color::TRANSPARENT, 64.0 / 255.0, options),
        Color::rgba(20, 40, 60, 191)
    );
    assert_eq!(
        composite_pixel(dest, Color::TRANSPARENT, 1.0, options),
        Color::TRANSPARENT
    );
}

#[test]
fn source_coverage_blends_with_the_uncovered_destination() {
    let out = composite_pixel(
        Color::rgb(0, 0, 255),
        Color::rgb(255, 0, 0),
        0.5,
        DrawOptions {
            composite_op: CompositeOp::Source,
            ..DrawOptions::default()
        },
    );
    assert_eq!(out, Color::rgba(128, 0, 128, 255));
}

#[test]
fn clear_respects_a_partial_clip_mask() {
    let mut dest = Layer::new("dest".to_string(), 1, 1);
    dest.buffer_mut().copy_from_slice(&[20, 40, 60, 255]);
    let mask = Mask::from_coverage(1, 1, vec![64]).unwrap();
    fill_rect_with_options(
        &mut dest,
        0,
        0,
        1,
        1,
        Color::TRANSPARENT,
        DrawOptions {
            composite_op: CompositeOp::Clear,
            clip_mask: Some(ClipMask::new(&mask, 0, 0)),
            ..DrawOptions::default()
        },
    );
    assert_eq!(dest.buffer(), &[20, 40, 60, 191]);
}
