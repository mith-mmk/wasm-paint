use paintcore::canvas::Screen;
use paintcore::composite::{
    composite_pixel, composite_screen, fill_rect_with_options, BlendMode, CompositeOp, DrawOptions,
};
use paintcore::layer::Layer;
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
