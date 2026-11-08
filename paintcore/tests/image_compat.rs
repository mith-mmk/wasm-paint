use paintcore::affine::{Affine, InterpolationAlgorithm};
use paintcore::canvas::Screen;
use paintcore::image_compat::{load_from_memory, ImageFormat, RgbaImage};
use paintcore::layer::Layer;

fn source() -> RgbaImage {
    RgbaImage::from_raw(
        2,
        2,
        vec![255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 0, 20, 30, 40, 17],
    )
    .unwrap()
}

#[test]
fn raw_construction_checks_shape_and_moves_storage() {
    assert!(RgbaImage::from_raw(1, 1, vec![0; 3]).is_err());
    assert!(RgbaImage::from_raw(1, 1, vec![0; 5]).is_err());
    assert!(RgbaImage::from_raw(u32::MAX, u32::MAX, vec![]).is_err());
    let bytes = vec![1, 2, 3, 4];
    let pointer = bytes.as_ptr();
    let image = RgbaImage::from_raw(1, 1, bytes).unwrap();
    assert_eq!(image.dimensions(), (1, 1));
    assert_eq!(image.as_raw().as_ptr(), pointer);
    let recovered = image.into_raw();
    assert_eq!(recovered.as_ptr(), pointer);
    assert!(RgbaImage::new(u32::MAX, u32::MAX).is_err());
}

#[test]
fn layer_handoff_checks_shape_and_reuses_existing_pixels() {
    let invalid = Layer::new_in("invalid".into(), vec![0; 3], 1, 1);
    assert!(RgbaImage::from_layer(invalid).is_err());
    let mut image = source();
    image.as_raw_mut()[0] = 42;
    let pointer = image.as_raw().as_ptr();
    let layer = image.into_layer();
    assert_eq!(layer.buffer().as_ptr(), pointer);
    assert_eq!(layer.buffer()[0], 42);
    let image = RgbaImage::from_layer(layer).unwrap();
    assert_eq!(image.as_layer().buffer().as_ptr(), pointer);
}

#[test]
fn memory_png_roundtrip_preserves_rgba_including_hidden_rgb() {
    let image = source();
    let encoded = image.encode_to_vec(ImageFormat::Png).unwrap();
    let decoded = load_from_memory(&encoded).unwrap();
    assert_eq!(decoded.image.dimensions(), image.dimensions());
    assert_eq!(decoded.image.as_raw(), image.as_raw());
    assert!(decoded.warnings.is_none());
    assert!(decoded.image.metadata().is_some());
}

#[test]
fn malformed_optional_exif_preserves_pixels_and_warning() {
    let jpeg = source().encode_to_vec(ImageFormat::Jpeg).unwrap();
    let expected = load_from_memory(&jpeg).unwrap();
    let payload = b"Exif\0\0bad-tiff";
    let mut with_exif = jpeg[..2].to_vec();
    with_exif.extend_from_slice(&[0xff, 0xe1]);
    with_exif.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
    with_exif.extend_from_slice(payload);
    with_exif.extend_from_slice(&jpeg[2..]);
    let decoded = load_from_memory(&with_exif).unwrap();
    assert_eq!(decoded.image.as_raw(), expected.image.as_raw());
    assert!(decoded.warnings.is_some());
}
#[test]
fn grayscale_and_brighten_delegate_to_existing_algorithms() {
    let image = source();
    let mut expected = Layer::try_new("expected".into(), 2, 2).unwrap();
    paintcore::filter::grayscale(image.as_layer(), &mut expected).unwrap();
    let gray = image.grayscale().unwrap();
    assert_eq!(gray.as_raw(), expected.buffer());
    paintcore::color::brightness_control(image.as_layer(), &mut expected, -30.0).unwrap();
    let darker = image.brighten(-30).unwrap();
    assert_eq!(darker.as_raw(), expected.buffer());
    for (original, adjusted) in image
        .as_raw()
        .as_chunks::<4>()
        .0
        .iter()
        .zip(darker.as_raw().as_chunks::<4>().0)
    {
        assert_eq!(original[3], adjusted[3]);
    }
    assert_eq!(
        image.as_raw(),
        source().as_raw(),
        "processing must not mutate input"
    );
}

#[test]
fn resize_exact_one_pixel_fills_the_entire_destination() {
    let image = RgbaImage::from_raw(1, 1, vec![255, 0, 0, 255]).unwrap();
    let resized = image
        .resize_exact(2, 2, InterpolationAlgorithm::NearestNeighbor)
        .unwrap();
    assert_eq!(resized.as_raw(), [255, 0, 0, 255].repeat(4));
}

#[test]
fn resize_exact_nearest_preserves_rectangular_pixel_blocks_and_identity() {
    let red = [255, 0, 0, 255];
    let green = [0, 255, 0, 128];
    let blue = [0, 0, 255, 0];
    let yellow = [255, 255, 0, 255];
    let original = [red, green, blue, yellow].concat();
    let image = RgbaImage::from_raw(2, 2, original.clone()).unwrap();
    let enlarged = image
        .resize_exact(4, 6, InterpolationAlgorithm::NearestNeighbor)
        .unwrap();
    let top = [red, red, green, green].concat().repeat(3);
    let bottom = [blue, blue, yellow, yellow].concat().repeat(3);
    assert_eq!(enlarged.as_raw(), [top, bottom].concat());
    let reduced = enlarged
        .resize_exact(2, 2, InterpolationAlgorithm::NearestNeighbor)
        .unwrap();
    assert_eq!(reduced.as_raw(), original);
    let identity = image
        .resize_exact(2, 2, InterpolationAlgorithm::NearestNeighbor)
        .unwrap();
    assert_eq!(identity.as_raw(), original);
    assert_eq!(image.as_raw(), original, "resize must not mutate input");
}

#[test]
fn resize_exact_matches_existing_resize_for_each_sampler_and_uniform_scale() {
    let image = source();
    for algorithm in [
        InterpolationAlgorithm::NearestNeighbor,
        InterpolationAlgorithm::Bilinear,
        InterpolationAlgorithm::Bicubic,
        InterpolationAlgorithm::Lanczos3,
    ] {
        for (width, height) in [(1, 1), (2, 2), (4, 4)] {
            let mut expected = Layer::try_new("expected".into(), width, height).unwrap();
            Affine::resize(
                image.as_layer(),
                &mut expected,
                width as f32 / 2.0,
                algorithm.clone(),
                paintcore::image::ImageAlign::LeftUp,
            );
            let resized = image
                .resize_exact(width, height, algorithm.clone())
                .unwrap();
            assert_eq!(resized.dimensions(), (width, height));
            assert_eq!(resized.as_raw(), expected.buffer());
        }
    }
}

#[test]
fn resize_rejects_empty_and_signed_coordinate_overflow_before_allocation() {
    let image = source();
    assert!(image
        .resize_exact(0, 2, InterpolationAlgorithm::Bilinear)
        .is_err());
    assert!(image
        .resize_exact(u32::MAX, 1, InterpolationAlgorithm::Bilinear)
        .is_err());
    let empty = RgbaImage::new(0, 0).unwrap();
    assert!(empty
        .resize_exact(2, 2, InterpolationAlgorithm::Bilinear)
        .is_err());
}

#[test]
#[allow(deprecated)]
fn resize_rejects_invalid_custom_sampling_parameters() {
    let image = source();
    for algorithm in [
        InterpolationAlgorithm::Lanczos(Some(0)),
        InterpolationAlgorithm::Lanczos(Some(65)),
        InterpolationAlgorithm::Lanczos(Some(i32::MAX as usize)),
        InterpolationAlgorithm::Lanzcos(Some(usize::MAX)),
        InterpolationAlgorithm::BicubicAlpha(Some(f32::NAN)),
        InterpolationAlgorithm::BicubicAlpha(Some(f32::INFINITY)),
    ] {
        let error = image.resize_exact(1, 1, algorithm).err().unwrap();
        assert!(error.downcast_ref::<wml2::error::ImgError>().is_some());
    }
    assert!(image
        .resize_exact(1, 1, InterpolationAlgorithm::Lanczos(Some(3)))
        .is_ok());
}

#[test]
fn animated_gif_decode_returns_first_frame_only() {
    // A complete 1x1 GIF with a red frame followed by a blue frame.
    let mut gif = b"GIF89a\x01\0\x01\0\x80\0\0\xff\0\0\0\0\xff".to_vec();
    for pixel_codes in [[0x44, 0x01], [0x4c, 0x01]] {
        gif.extend_from_slice(b"\x21\xf9\x04\0\x01\0\0\0");
        gif.extend_from_slice(b"\x2c\0\0\0\0\x01\0\x01\0\0\x02\x02");
        gif.extend_from_slice(&pixel_codes);
        gif.push(0);
    }
    gif.push(0x3b);
    let decoded = load_from_memory(&gif).unwrap();
    assert_eq!(decoded.image.dimensions(), (1, 1));
    assert_eq!(decoded.image.as_raw(), [255, 0, 0, 255]);
}

#[test]
fn processing_does_not_copy_stale_source_metadata() {
    let png = source().encode_to_vec(ImageFormat::Png).unwrap();
    let decoded = load_from_memory(&png).unwrap();
    assert!(decoded.image.metadata().is_some());
    assert!(decoded.image.grayscale().unwrap().metadata().is_none());
    assert!(decoded.image.brighten(1).unwrap().metadata().is_none());
    assert!(decoded
        .image
        .resize_exact(1, 1, InterpolationAlgorithm::Bilinear)
        .unwrap()
        .metadata()
        .is_none());
}

#[test]
fn writer_encodes_before_writing_and_propagates_io_errors() {
    let image = source();
    let mut output = vec![1, 2, 3];
    assert!(image.write_to(&mut output, ImageFormat::Unknown).is_err());
    assert_eq!(output, [1, 2, 3]);
    output.clear();
    image.write_to(&mut output, ImageFormat::Png).unwrap();
    assert_eq!(
        load_from_memory(&output).unwrap().image.as_raw(),
        image.as_raw()
    );

    struct BrokenWriter;
    impl std::io::Write for BrokenWriter {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "test writer",
            ))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            panic!("must not flush caller's writer")
        }
    }
    let error = image
        .write_to(&mut BrokenWriter, ImageFormat::Png)
        .err()
        .unwrap();
    assert_eq!(
        error.downcast_ref::<std::io::Error>().unwrap().kind(),
        std::io::ErrorKind::BrokenPipe
    );
}

#[test]
fn invalid_input_returns_original_error_type() {
    let error = load_from_memory(b"not an image").err().unwrap();
    assert!(error.downcast_ref::<wml2::error::ImgError>().is_some());
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn reader_and_path_detect_contents_instead_of_extension() {
    let image = source();
    let encoded = image.encode_to_vec(ImageFormat::Png).unwrap();
    let decoded = paintcore::image_compat::read(std::io::Cursor::new(&encoded)).unwrap();
    assert_eq!(decoded.image.as_raw(), image.as_raw());
    let path = std::env::temp_dir().join(format!(
        ".test-paintcore-image-api-{}.unknown",
        std::process::id()
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    std::io::Write::write_all(&mut file, &encoded).unwrap();
    drop(file);
    let result = paintcore::image_compat::open(&path);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(result.unwrap().image.as_raw(), image.as_raw());
}
