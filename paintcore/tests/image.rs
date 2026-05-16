use paintcore::canvas::Screen;
use paintcore::image::decode_image_layer;
use wml2::draw::{image_to, ImageBuffer};
use wml2::util::ImageFormat;

#[test]
fn decode_image_layer_decodes_png_as_rgba_layer() {
    let rgba = vec![
        0xff, 0x00, 0x00, 0xff, 0x00, 0xff, 0x00, 0x80, 0x00, 0x00, 0xff, 0x40, 0xff, 0xff, 0xff,
        0x00,
    ];
    let mut source = ImageBuffer::from_buffer(2, 2, rgba.clone());
    let encoded = image_to(&mut source, ImageFormat::Png, None).expect("encode test png");

    let layer =
        decode_image_layer("_test_image_".to_string(), &encoded, 0).expect("decode image layer");

    assert_eq!(layer.width(), 2);
    assert_eq!(layer.height(), 2);
    assert_eq!(layer.buffer(), rgba.as_slice());
}
