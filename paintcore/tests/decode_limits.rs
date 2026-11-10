use paintcore::image_compat::{
    load_from_memory, load_from_memory_with_limits, DecodeLimitExceeded, DecodeLimits, ImageFormat,
    ImgError, ImgErrorKind, RgbaImage,
};
use paintcore::layer::Layer;

fn source() -> RgbaImage {
    RgbaImage::from_raw(2, 2, vec![255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 0, 20, 30, 40, 17]).unwrap()
}

fn assert_limit(error: &(dyn std::error::Error + 'static), requested: u64, limit: u64) {
    let image_error = error.downcast_ref::<ImgError>().expect("original WML2 error");
    assert_eq!(image_error.kind(), ImgErrorKind::OutOfMemory);
    let detail = error.source().unwrap().downcast_ref::<DecodeLimitExceeded>().unwrap();
    assert_eq!(detail.requested(), requested);
    assert_eq!(detail.limit(), limit);
    assert!(!detail.resource().is_empty()); // A human label, not a stable code.
}

#[test]
fn memory_default_and_explicit_limits_match_existing_layer_callback() {
    for format in [ImageFormat::Png, ImageFormat::Bmp, ImageFormat::Gif, ImageFormat::Jpeg, ImageFormat::Webp] {
        let encoded = source().encode_to_vec(format).unwrap();
        let mut legacy = Layer::try_new("legacy".into(), 0, 0).unwrap();
        let legacy_warnings = paintcore::image::draw_image(&mut legacy, &encoded, 0).unwrap();
        let legacy = RgbaImage::from_layer(legacy).unwrap();
        let expected_warnings: Vec<_> = legacy_warnings.as_ref().into_iter().flat_map(|w| w.iter()).map(ToString::to_string).collect();
        for decoded in [load_from_memory(&encoded).unwrap(), load_from_memory_with_limits(&encoded, DecodeLimits::default()).unwrap()] {
            assert_eq!(decoded.image.as_raw(), legacy.as_raw());
            assert_eq!(decoded.image.dimensions(), (legacy.width(), legacy.height()));
            assert_eq!(decoded.image.metadata().map(|m| m.len()), legacy.metadata().map(|m| m.len()));
            let warnings: Vec<_> = decoded.warnings.as_ref().into_iter().flat_map(|w| w.iter()).map(ToString::to_string).collect();
            assert_eq!(warnings, expected_warnings);
        }
    }
}

#[test]
fn input_budget_rejects_before_decode_and_accepts_exact_boundary() {
    let encoded = source().encode_to_vec(ImageFormat::Png).unwrap();
    let limits = DecodeLimits { input_bytes: encoded.len(), ..DecodeLimits::default() };
    assert_eq!(load_from_memory_with_limits(&encoded, limits).unwrap().image.as_raw(), source().as_raw());
    let limits = DecodeLimits { input_bytes: encoded.len() - 1, ..limits };
    let error = load_from_memory_with_limits(&encoded, limits).err().unwrap();
    assert_limit(error.as_ref(), encoded.len() as u64, encoded.len() as u64 - 1);
    let error = load_from_memory_with_limits(b"invalid", DecodeLimits { input_bytes: 0, ..limits }).err().unwrap();
    assert_limit(error.as_ref(), 7, 0);
}

#[test]
fn pixel_budget_is_independent_and_accepts_exact_boundary() {
    let encoded = source().encode_to_vec(ImageFormat::Png).unwrap();
    let limits = DecodeLimits { pixels: 4, ..DecodeLimits::default() };
    assert!(load_from_memory_with_limits(&encoded, limits).is_ok());
    let error = load_from_memory_with_limits(&encoded, DecodeLimits { pixels: 3, ..limits }).err().unwrap();
    assert_limit(error.as_ref(), 4, 3);
}

#[test]
fn expanded_budget_is_independent_and_accepts_exact_boundary() {
    let encoded = source().encode_to_vec(ImageFormat::Bmp).unwrap();
    let limits = DecodeLimits { expanded_bytes: 16, ..DecodeLimits::default() };
    assert!(load_from_memory_with_limits(&encoded, limits).is_ok());
    let error = load_from_memory_with_limits(&encoded, DecodeLimits { expanded_bytes: 15, ..limits }).err().unwrap();
    assert_limit(error.as_ref(), 16, 15);
}

#[test]
fn failed_limits_do_not_leak_into_later_calls() {
    let encoded = source().encode_to_vec(ImageFormat::Png).unwrap();
    assert!(load_from_memory_with_limits(&encoded, DecodeLimits { pixels: 0, ..DecodeLimits::default() }).is_err());
    assert_eq!(load_from_memory(&encoded).unwrap().image.as_raw(), source().as_raw());
    assert_eq!(load_from_memory_with_limits(&encoded, DecodeLimits::unlimited()).unwrap().image.as_raw(), source().as_raw());
}

#[test]
fn warnings_remain_individually_iterable_with_custom_limits() {
    let jpeg = source().encode_to_vec(ImageFormat::Jpeg).unwrap();
    let payload = b"Exif\0\0bad-tiff";
    let mut damaged = jpeg[..2].to_vec();
    damaged.extend_from_slice(&[0xff, 0xe1]);
    damaged.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
    damaged.extend_from_slice(payload);
    damaged.extend_from_slice(&jpeg[2..]);
    let decoded = load_from_memory_with_limits(&damaged, DecodeLimits::default()).unwrap();
    let warnings = decoded.warnings.unwrap();
    assert!(!warnings.is_empty());
    assert_eq!(warnings.iter().len(), warnings.len());
    assert!(warnings.iter().all(|w| !w.to_string().is_empty()));
    assert_eq!(decoded.image.as_raw(), load_from_memory(&jpeg).unwrap().image.as_raw());
}

#[test]
fn unknown_format_retains_public_error_kind() {
    let error = load_from_memory(b"not an image").err().unwrap();
    assert_eq!(error.downcast_ref::<ImgError>().unwrap().kind(), ImgErrorKind::UnknownFormat);
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn native_reader_and_path_share_limits_and_preserve_io_errors() {
    use paintcore::image_compat::{open_with_limits, read_with_limits};
    use std::io::{Cursor, ErrorKind, Write};
    let encoded = source().encode_to_vec(ImageFormat::Png).unwrap();
    let limits = DecodeLimits { input_bytes: encoded.len() - 1, ..DecodeLimits::default() };
    let error = read_with_limits(Cursor::new(&encoded), limits).err().unwrap();
    assert_limit(error.as_ref(), encoded.len() as u64, encoded.len() as u64 - 1);
    let mut reader = Cursor::new(&encoded);
    assert_eq!(read_with_limits(&mut reader, DecodeLimits::default()).unwrap().image.as_raw(), source().as_raw());
    let path = std::env::temp_dir().join(format!(".test-paintcore-decode-limits-{}.unknown", std::process::id()));
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap();
    file.write_all(&encoded).unwrap();
    drop(file);
    let rejected = open_with_limits(&path, limits);
    let accepted = open_with_limits(&path, DecodeLimits::default());
    std::fs::remove_file(&path).unwrap();
    assert_limit(rejected.err().unwrap().as_ref(), encoded.len() as u64, encoded.len() as u64 - 1);
    assert_eq!(accepted.unwrap().image.as_raw(), source().as_raw());
    let error = open_with_limits(&path, limits).err().unwrap();
    assert_eq!(error.downcast_ref::<std::io::Error>().unwrap().kind(), ErrorKind::NotFound);
}

#[cfg(not(target_family = "wasm"))]
#[test]
fn failing_reader_preserves_original_io_error() {
    use std::io::{BufRead, Error, ErrorKind, Read, Result, Seek, SeekFrom};
    struct BrokenReader;
    impl Read for BrokenReader {
        fn read(&mut self, _: &mut [u8]) -> Result<usize> { Err(Error::new(ErrorKind::PermissionDenied, "test reader")) }
    }
    impl BufRead for BrokenReader {
        fn fill_buf(&mut self) -> Result<&[u8]> { Err(Error::new(ErrorKind::PermissionDenied, "test reader")) }
        fn consume(&mut self, _: usize) {}
    }
    impl Seek for BrokenReader {
        fn seek(&mut self, _: SeekFrom) -> Result<u64> { Err(Error::new(ErrorKind::PermissionDenied, "test reader")) }
    }
    let error = paintcore::image_compat::read_with_limits(BrokenReader, DecodeLimits::default()).err().unwrap();
    assert_eq!(error.downcast_ref::<Error>().unwrap().kind(), ErrorKind::PermissionDenied);
}
