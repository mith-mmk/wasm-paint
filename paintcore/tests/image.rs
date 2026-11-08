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

#[test]
fn legacy_layer_callback_still_decodes_default_codec_outputs() {
    // Use opaque, palette-friendly colors for the lossless formats. JPEG is a
    // lossy smoke check only; the other formats must retain exact RGB pixels.
    let rgba = vec![
        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
    ];
    for format in [
        ImageFormat::Bmp,
        ImageFormat::Gif,
        ImageFormat::Png,
        ImageFormat::Tiff,
        ImageFormat::Webp,
    ] {
        let mut source = ImageBuffer::from_buffer(2, 2, rgba.clone());
        let encoded = image_to(&mut source, format, None).expect("encode default codec");
        let layer =
            decode_image_layer("codec".into(), &encoded, 0).expect("decode through Layer callback");
        assert_eq!((layer.width(), layer.height()), (2, 2));
        assert_eq!(layer.buffer(), rgba);
    }
    let mut source = ImageBuffer::from_buffer(2, 2, rgba);
    let encoded = image_to(&mut source, ImageFormat::Jpeg, None).unwrap();
    let layer = decode_image_layer("jpeg".into(), &encoded, 0).unwrap();
    assert_eq!((layer.width(), layer.height()), (2, 2));
    assert_eq!(layer.buffer().len(), 16);
    assert!(layer
        .buffer()
        .as_chunks::<4>()
        .0
        .iter()
        .all(|pixel| pixel[3] == 255));
}
