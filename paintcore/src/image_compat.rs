//! A small image-rs-style workflow backed by paintcore and WML2.
//!
//! This is functional compatibility, not a source-compatible replacement for
//! the `image` crate. Storage is straight RGBA8. Decoding preserves warnings;
//! processing uses paintcore's existing algorithms and their pixel semantics.
//! No `image` dependency or second raster-processing implementation is used.
//!
//! ```
//! use paintcore::image_compat::{load_from_memory, ImageFormat, RgbaImage};
//!
//! let image = RgbaImage::from_raw(1, 1, vec![255, 0, 0, 128])?;
//! let png = image.encode_to_vec(ImageFormat::Png)?;
//! let decoded = load_from_memory(&png)?;
//! assert_eq!(decoded.image.as_raw(), image.as_raw());
//! let lighter = decoded.image.brighten(16)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::affine::{Affine, InterpolationAlgorithm};
use crate::canvas::Screen;
use crate::layer::{checked_buffer_len, Layer};
use bin_rs::reader::StreamReader;
use std::collections::HashMap;
use std::io::Write;
use wml2::draw::DecodeOptions;
use wml2::draw::{image_to, ImageBuffer};
use wml2::metadata::DataMap;

pub use wml2::error::{ImgError, ImgErrorKind};
pub use wml2::limits::{DecodeLimitExceeded, DecodeLimits};
pub use wml2::util::ImageFormat;
pub use wml2::warning::{ImgWarning, ImgWarnings};

/// Preserves original WML2, callback, allocation, and I/O error types.
pub type ImageResult<T> = Result<T, Box<dyn std::error::Error>>;

/// A decoded first image plus all warnings returned by the decoder.
///
/// Animation is deliberately limited to the first image in this API. Use the
/// existing `Canvas` and WML2 callback APIs when animation frames are needed.
pub struct DecodedImage {
    pub image: RgbaImage,
    pub warnings: Option<ImgWarnings>,
}

/// An owned, checked, straight-alpha RGBA8 raster backed by a paintcore `Layer`.
pub struct RgbaImage {
    layer: Layer,
}

fn invalid_parameter(message: &str) -> Box<dyn std::error::Error> {
    Box::new(ImgError::new_const(
        ImgErrorKind::InvalidParameter,
        message.to_owned(),
    ))
}

impl RgbaImage {
    /// Allocates a transparent raster. Allocation and size overflow are errors.
    pub fn new(width: u32, height: u32) -> ImageResult<Self> {
        Ok(Self {
            layer: Layer::try_new("_image_".into(), width, height)?,
        })
    }

    /// Takes ownership without copying. The length must be exactly width*height*4.
    pub fn from_raw(width: u32, height: u32, bytes: Vec<u8>) -> ImageResult<Self> {
        if checked_buffer_len(width, height)? != bytes.len() {
            return Err(invalid_parameter(
                "RGBA8 buffer length does not match dimensions",
            ));
        }
        Ok(Self {
            layer: Layer::new_in("_image_".into(), bytes, width, height),
        })
    }

    /// Takes an existing paintcore layer after validating its pixel buffer.
    /// Layer position, visibility and global opacity do not change raw pixels.
    pub fn from_layer(layer: Layer) -> ImageResult<Self> {
        if checked_buffer_len(layer.width(), layer.height())? != layer.buffer().len() {
            return Err(invalid_parameter(
                "layer buffer length does not match dimensions",
            ));
        }
        Ok(Self { layer })
    }

    pub fn width(&self) -> u32 {
        self.layer.width()
    }

    pub fn height(&self) -> u32 {
        self.layer.height()
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    pub fn as_raw(&self) -> &[u8] {
        self.layer.buffer()
    }

    /// Mutates samples without exposing a resizable buffer.
    pub fn as_raw_mut(&mut self) -> &mut [u8] {
        self.layer.buffer_mut()
    }

    pub fn into_raw(self) -> Vec<u8> {
        self.layer.buffer
    }

    pub fn as_layer(&self) -> &Layer {
        &self.layer
    }

    /// Returns the original layer for paintcore's drawing APIs, without copying.
    pub fn into_layer(self) -> Layer {
        self.layer
    }

    /// Source metadata is retained on decoded images for explicit inspection.
    /// Processing results are new rasters and do not copy source metadata such
    /// as EXIF dimensions, orientation, or profiles that may no longer apply.
    pub fn metadata(&self) -> Option<&HashMap<String, DataMap>> {
        self.layer.metadata.as_ref()
    }

    /// Resizes to exact dimensions using the existing affine sampler.
    ///
    /// Sampling is paintcore's, not a promise of image-rs pixel equivalence.
    /// Empty images and dimensions outside the affine engine's signed range
    /// are rejected instead of silently producing an unchanged/empty result.
    /// Custom Lanczos radii are limited to 1..=64 in this checked adapter;
    /// custom bicubic parameters must be finite. Legacy APIs are unchanged.
    #[allow(deprecated)]
    pub fn resize_exact(
        &self,
        width: u32,
        height: u32,
        algorithm: InterpolationAlgorithm,
    ) -> ImageResult<Self> {
        if [self.width(), self.height(), width, height]
            .iter()
            .any(|&dimension| dimension == 0 || dimension > i32::MAX as u32)
        {
            return Err(invalid_parameter(
                "resize dimensions must be nonzero and fit i32",
            ));
        }
        match &algorithm {
            InterpolationAlgorithm::Lanczos(Some(radius))
            | InterpolationAlgorithm::Lanzcos(Some(radius))
                if !(1..=64).contains(radius) =>
            {
                return Err(invalid_parameter("custom Lanczos radius must be in 1..=64"));
            }
            InterpolationAlgorithm::BicubicAlpha(Some(alpha)) if !alpha.is_finite() => {
                return Err(invalid_parameter("custom bicubic parameter must be finite"));
            }
            _ => {}
        }
        let mut result = Self::new(width, height)?;
        let mut affine = Affine::new();
        affine.scale(
            width as f32 / self.width() as f32,
            height as f32 / self.height() as f32,
        );
        // A resize maps the source origin to the destination origin. The
        // general conversion helper instead pivots around the output center.
        // Use the same zero-origin sampling entry point as Affine::resize,
        // while allowing independent horizontal and vertical scales.
        affine.conversion_with_area_center(
            &self.layer,
            &mut result.layer,
            0.0,
            0.0,
            self.width() as f32,
            self.height() as f32,
            0,
            0,
            width as i32,
            height as i32,
            0.0,
            0.0,
            algorithm,
        );
        Ok(result)
    }

    /// Returns neutral RGB with unchanged alpha using paintcore's grayscale.
    pub fn grayscale(&self) -> ImageResult<Self> {
        let mut result = Self::new(self.width(), self.height())?;
        crate::filter::grayscale(&self.layer, &mut result.layer)?;
        Ok(result)
    }

    /// Adds a signed channel offset with saturation; alpha is unchanged.
    pub fn brighten(&self, value: i32) -> ImageResult<Self> {
        let mut result = Self::new(self.width(), self.height())?;
        crate::color::brightness_control(&self.layer, &mut result.layer, value as f32)?;
        Ok(result)
    }

    /// Encodes pixels through WML2. Codec support follows enabled WML2 features.
    /// Source metadata is not implicitly embedded. This initial adapter copies
    /// the pixel buffer into WML2's existing encoder buffer.
    pub fn encode_to_vec(&self, format: ImageFormat) -> ImageResult<Vec<u8>> {
        let mut image = ImageBuffer::from_buffer(
            self.width() as usize,
            self.height() as usize,
            self.as_raw().to_vec(),
        );
        image_to(&mut image, format, None)
    }

    /// Encodes before writing so encoder failure does not modify the writer.
    /// Does not flush or close the caller-owned writer. Write failures may leave
    /// partial output, as with ordinary `Write::write_all`.
    pub fn write_to<W: Write>(&self, writer: &mut W, format: ImageFormat) -> ImageResult<()> {
        let encoded = self.encode_to_vec(format)?;
        writer.write_all(&encoded)?;
        Ok(())
    }
}

/// Decodes the first image using the existing paintcore Layer callback.
/// WML2's default resource budgets apply; warnings are never silently discarded.
pub fn load_from_memory(bytes: &[u8]) -> ImageResult<DecodedImage> {
    load_from_memory_with_limits(bytes, DecodeLimits::default())
}

/// Decodes the first image with explicit WML2 resource budgets.
///
/// Input is borrowed through a seekable reader, not copied into a second input
/// buffer. The existing Layer callback still owns the decoded RGBA8 pixels.
/// Limits are independent budgets, not a total peak-memory bound; using
/// [`DecodeLimits::unlimited`] explicitly disables them. Warnings and original
/// errors are preserved, including [`DecodeLimitExceeded`] in error sources.
///
/// ```
/// use paintcore::image_compat::{load_from_memory_with_limits, DecodeLimits};
/// let limits = DecodeLimits { input_bytes: 1024 * 1024, ..DecodeLimits::default() };
/// assert!(load_from_memory_with_limits(b"not an image", limits).is_err());
/// ```
pub fn load_from_memory_with_limits(
    bytes: &[u8],
    limits: DecodeLimits,
) -> ImageResult<DecodedImage> {
    decode_reader_with_limits(std::io::Cursor::new(bytes), limits)
}

fn decode_reader_with_limits<R: std::io::BufRead + std::io::Seek>(
    reader: R,
    limits: DecodeLimits,
) -> ImageResult<DecodedImage> {
    let mut layer = Layer::try_new("_decoded_image_".into(), 0, 0)?;
    let warnings = wml2::draw::image_decoder_with_limits(
        &mut StreamReader::new(reader),
        &mut DecodeOptions {
            debug_flag: 0,
            drawer: &mut layer,
        },
        limits,
    )?;
    Ok(DecodedImage {
        image: RgbaImage::from_layer(layer)?,
        warnings,
    })
}

/// Decodes from a buffered seekable reader without reading the whole file first.
#[cfg(not(target_family = "wasm"))]
pub fn read<R: std::io::BufRead + std::io::Seek>(reader: R) -> ImageResult<DecodedImage> {
    read_with_limits(reader, DecodeLimits::default())
}

/// Decodes a native buffered seekable reader with explicit WML2 budgets.
/// I/O errors remain downcastable to their original type. Takes the reader by
/// value; pass `&mut reader` to retain ownership. Position follows the decoder.
#[cfg(not(target_family = "wasm"))]
pub fn read_with_limits<R: std::io::BufRead + std::io::Seek>(
    reader: R,
    limits: DecodeLimits,
) -> ImageResult<DecodedImage> {
    decode_reader_with_limits(reader, limits)
}

/// Opens a native path. Format detection is based on contents, not the suffix.
#[cfg(not(target_family = "wasm"))]
pub fn open<P: AsRef<std::path::Path>>(path: P) -> ImageResult<DecodedImage> {
    open_with_limits(path, DecodeLimits::default())
}

/// Opens and decodes a native path with explicit WML2 budgets.
/// File-open failures preserve their original I/O error type.
#[cfg(not(target_family = "wasm"))]
pub fn open_with_limits<P: AsRef<std::path::Path>>(
    path: P,
    limits: DecodeLimits,
) -> ImageResult<DecodedImage> {
    read_with_limits(std::io::BufReader::new(std::fs::File::open(path)?), limits)
}
