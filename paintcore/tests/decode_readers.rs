//! Regression coverage for the bin-rs 0.0.11 buffered-reader boundary.
#![cfg(not(target_family = "wasm"))]
use paintcore::image_compat::{
    load_from_memory, read, read_with_limits, DecodeLimits, ImageFormat, RgbaImage,
};
use std::io::{self, BufReader, Cursor, Read, Seek, SeekFrom};

fn encoded(format: ImageFormat) -> Vec<u8> {
    RgbaImage::from_raw(
        2,
        2,
        vec![255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 0, 20, 30, 40, 17],
    )
    .unwrap()
    .encode_to_vec(format)
    .unwrap()
}

fn formats() -> [ImageFormat; 5] {
    [
        ImageFormat::Png,
        ImageFormat::Bmp,
        ImageFormat::Gif,
        ImageFormat::Jpeg,
        ImageFormat::Webp,
    ]
}

#[test]
fn small_buffers_match_memory_for_five_codecs_and_both_apis() {
    for format in formats() {
        let bytes = encoded(format);
        let expected = load_from_memory(&bytes).unwrap();
        for capacity in [1, 8, 16, 255, 256, 8192] {
            for explicit in [false, true] {
                let mut reader = BufReader::with_capacity(capacity, Cursor::new(&bytes));
                let decoded = if explicit {
                    read_with_limits(&mut reader, DecodeLimits::default())
                } else {
                    read(&mut reader)
                }
                .unwrap();
                assert_eq!(
                    decoded.image.as_raw(),
                    expected.image.as_raw(),
                    "capacity={capacity}, explicit={explicit}"
                );
                assert_eq!(decoded.image.dimensions(), expected.image.dimensions());
                assert_eq!(decoded.image.metadata(), expected.image.metadata());
                let warnings = |w: &Option<paintcore::image_compat::ImgWarnings>| {
                    w.iter()
                        .flat_map(|w| w.iter())
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                };
                assert_eq!(warnings(&decoded.warnings), warnings(&expected.warnings));
            }
        }
    }
}

/// Returns at most three bytes, and optionally fails one underlying read.
struct ShortReader {
    bytes: Cursor<Vec<u8>>,
    fail_once: bool,
}
impl Read for ShortReader {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if self.fail_once {
            self.fail_once = false;
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "injected read failure",
            ));
        }
        let count = out.len().min(3);
        self.bytes.read(&mut out[..count])
    }
}
impl Seek for ShortReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.bytes.seek(pos)
    }
}

#[test]
fn short_reads_and_io_failure_recovery_preserve_the_caller_reader() {
    for format in formats() {
        let bytes = encoded(format);
        let expected = load_from_memory(&bytes).unwrap();
        for capacity in [1, 8, 16, 255, 256] {
            for explicit in [false, true] {
                let inner = ShortReader {
                    bytes: Cursor::new(bytes.clone()),
                    fail_once: true,
                };
                let mut reader = BufReader::with_capacity(capacity, inner);
                let result = if explicit {
                    read_with_limits(&mut reader, DecodeLimits::default())
                } else {
                    read(&mut reader)
                };
                let error = result.err().unwrap();
                assert_eq!(
                    error.downcast_ref::<io::Error>().unwrap().kind(),
                    io::ErrorKind::PermissionDenied
                );
                assert_eq!(reader.stream_position().unwrap(), 0);
                // Ownership remains with the caller; retry after the transient failure.
                let decoded = if explicit {
                    read_with_limits(&mut reader, DecodeLimits::default())
                } else {
                    read(&mut reader)
                }
                .unwrap();
                assert_eq!(decoded.image.as_raw(), expected.image.as_raw());
            }
        }
    }
}

#[test]
fn nonzero_cursor_matches_the_existing_reader_contract() {
    // Some codecs use absolute offsets. This checks the actual contract against
    // the same reader starting position, not an implicit substream guarantee.
    for format in formats() {
        let bytes = encoded(format);
        let mut prefixed = vec![0xaa; 11];
        prefixed.extend_from_slice(&bytes);
        for capacity in [1, 8, 16, 255, 256] {
            let mut expected = Cursor::new(&prefixed);
            expected.set_position(11);
            let expected = read(expected);
            for explicit in [false, true] {
                let mut reader = BufReader::with_capacity(capacity, Cursor::new(&prefixed));
                reader.seek(SeekFrom::Start(11)).unwrap();
                let actual = if explicit {
                    read_with_limits(&mut reader, DecodeLimits::default())
                } else {
                    read(&mut reader)
                };
                match (&expected, actual) {
                    (Ok(expected), Ok(actual)) => {
                        assert_eq!(actual.image.as_raw(), expected.image.as_raw())
                    }
                    (Err(expected), Err(actual)) => {
                        assert_eq!(actual.to_string(), expected.to_string())
                    }
                    _ => panic!("buffer capacity changed nonzero-position decoding"),
                }
            }
        }
    }
}
