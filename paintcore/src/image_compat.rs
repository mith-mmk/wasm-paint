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
use std::collections::HashMap;
use std::io::Write;
#[cfg(not(target_family = "wasm"))]
use wml2::draw::DecodeOptions;
use wml2::draw::{image_to, ImageBuffer};
use wml2::error::{ImgError, ImgErrorKind};
use wml2::metadata::DataMap;
use wml2::warning::ImgWarnings;

pub use wml2::util::ImageFormat;

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

/// A read-only rectangle borrowing an existing [`RgbaImage`] without copying.
///
/// Coordinates are local to the view. Each row is packed RGBA8, but neighboring
/// rows may have gaps in the parent buffer. Use [`Self::rows`] rather than
/// assuming the whole rectangle is contiguous. Layer position/opacity are not
/// applied. A view cannot outlive its image or coexist with mutable pixel access.
#[derive(Clone, Copy)]
pub struct RgbaView<'a> {
    image: &'a RgbaImage,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl RgbaView<'_> {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Borrows one pixel, or returns `None` outside this view's dimensions.
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<&[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.image.get_pixel(self.x + x, self.y + y)
    }

    /// Borrows a row of exactly `width * 4` bytes, excluding parent-row padding.
    /// An in-bounds row of a zero-width view is empty; other invalid rows are `None`.
    pub fn row(&self, y: u32) -> Option<&[u8]> {
        if y >= self.height {
            return None;
        }
        if self.width == 0 {
            return Some(&self.image.as_raw()[..0]);
        }
        let start = self.image.pixel_offset(self.x, self.y + y)?;
        Some(&self.image.as_raw()[start..start + self.width as usize * 4])
    }

    /// Iterates exactly `height` rows, from top to bottom, without allocating.
    pub fn rows(&self) -> impl ExactSizeIterator<Item = &[u8]> + DoubleEndedIterator + '_ {
        (0..self.height).map(|y| self.row(y).expect("validated view row"))
    }

    /// Copies this rectangle into an independent packed image.
    /// Raw samples, including RGB at zero alpha, are unchanged. Source metadata
    /// is not copied because its dimensions/orientation may no longer apply.
    pub fn to_image(&self) -> ImageResult<RgbaImage> {
        let mut result = RgbaImage::new(self.width, self.height)?;
        if self.width != 0 && self.height != 0 {
            for (source, destination) in self.rows().zip(
                result
                    .as_raw_mut()
                    .chunks_exact_mut(self.width as usize * 4),
            ) {
                destination.copy_from_slice(source);
            }
        }
        Ok(result)
    }
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

    // The validated full buffer bounds this multiplication, including on wasm32.
    // Check coordinates first so huge zero-area shapes cannot overflow offsets.
    fn pixel_offset(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width() || y >= self.height() {
            return None;
        }
        Some((y as usize * self.width() as usize + x as usize) * 4)
    }

    /// Borrows a straight RGBA8 pixel, or returns `None` for invalid coordinates.
    pub fn get_pixel(&self, x: u32, y: u32) -> Option<&[u8; 4]> {
        let offset = self.pixel_offset(x, y)?;
        self.as_raw()[offset..offset + 4].try_into().ok()
    }

    /// Mutably borrows a pixel without blending or changing the buffer's shape.
    /// Invalid coordinates return `None` rather than clipping or panicking.
    pub fn get_pixel_mut(&mut self, x: u32, y: u32) -> Option<&mut [u8; 4]> {
        let offset = self.pixel_offset(x, y)?;
        (&mut self.as_raw_mut()[offset..offset + 4]).try_into().ok()
    }

    /// Iterates borrowed pixels in row-major order without allocating.
    pub fn pixels(&self) -> std::slice::Iter<'_, [u8; 4]> {
        self.as_raw().as_chunks::<4>().0.iter()
    }

    /// Iterates mutable pixels in row-major order without resizing the buffer.
    pub fn pixels_mut(&mut self) -> std::slice::IterMut<'_, [u8; 4]> {
        self.as_raw_mut().as_chunks_mut::<4>().0.iter_mut()
    }

    /// Borrows a rectangle without copying. Out-of-bounds rectangles are errors,
    /// not silently clamped. Empty rectangles may start on the right/bottom edge
    /// when both coordinate ranges still fit within the image.
    ///
    /// ```
    /// use paintcore::image_compat::RgbaImage;
    /// let mut image = RgbaImage::new(3, 2)?;
    /// *image.get_pixel_mut(2, 1).unwrap() = [10, 20, 30, 0];
    /// let view = image.view(1, 0, 2, 2)?;
    /// assert_eq!(view.get_pixel(1, 1), Some(&[10, 20, 30, 0]));
    /// let owned = view.to_image()?;
    /// assert_eq!(owned.dimensions(), (2, 2));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn view(&self, x: u32, y: u32, width: u32, height: u32) -> ImageResult<RgbaView<'_>> {
        if x > self.width()
            || y > self.height()
            || width > self.width() - x
            || height > self.height() - y
        {
            return Err(invalid_parameter(
                "view rectangle is outside image dimensions",
            ));
        }
        Ok(RgbaView {
            image: self,
            x,
            y,
            width,
            height,
        })
    }

    /// Returns an independent, checked rectangular copy with no source metadata.
    /// Uses the same bounds and empty-rectangle policy as [`Self::view`].
    pub fn crop(&self, x: u32, y: u32, width: u32, height: u32) -> ImageResult<Self> {
        self.view(x, y, width, height)?.to_image()
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
    let mut layer = Layer::try_new("_decoded_image_".into(), 0, 0)?;
    let warnings = crate::image::draw_image(&mut layer, bytes, 0)?;
    Ok(DecodedImage {
        image: RgbaImage::from_layer(layer)?,
        warnings,
    })
}

/// Decodes from a buffered seekable reader without reading the whole file first.
#[cfg(not(target_family = "wasm"))]
pub fn read<R: std::io::BufRead + std::io::Seek>(reader: R) -> ImageResult<DecodedImage> {
    let mut layer = Layer::try_new("_decoded_image_".into(), 0, 0)?;
    let warnings = wml2::draw::image_reader(
        reader,
        &mut DecodeOptions {
            debug_flag: 0,
            drawer: &mut layer,
        },
    )?;
    Ok(DecodedImage {
        image: RgbaImage::from_layer(layer)?,
        warnings,
    })
}

/// Opens a native path. Format detection is based on contents, not the suffix.
#[cfg(not(target_family = "wasm"))]
pub fn open<P: AsRef<std::path::Path>>(path: P) -> ImageResult<DecodedImage> {
    read(std::io::BufReader::new(std::fs::File::open(path)?))
}
