//! Paint colors and sources shared by raster drawing operations.

/// An explicit, non-premultiplied RGBA color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);

    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::rgba(red, green, blue, 0xff)
    }

    /// Creates a color from the legacy paintcore `0xAARRGGBB` representation.
    pub const fn from_argb_u32(color: u32) -> Self {
        Self::rgba(
            ((color >> 16) & 0xff) as u8,
            ((color >> 8) & 0xff) as u8,
            (color & 0xff) as u8,
            ((color >> 24) & 0xff) as u8,
        )
    }

    /// Creates an opaque color from the legacy paintcore `0xRRGGBB` representation.
    pub const fn from_rgb_u32(color: u32) -> Self {
        Self::rgb(
            ((color >> 16) & 0xff) as u8,
            ((color >> 8) & 0xff) as u8,
            (color & 0xff) as u8,
        )
    }

    /// Creates a color from `0xRRGGBBAA`.
    pub const fn from_rgba_u32(color: u32) -> Self {
        Self::rgba(
            ((color >> 24) & 0xff) as u8,
            ((color >> 16) & 0xff) as u8,
            ((color >> 8) & 0xff) as u8,
            (color & 0xff) as u8,
        )
    }

    pub const fn to_argb_u32(self) -> u32 {
        ((self.alpha as u32) << 24)
            | ((self.red as u32) << 16)
            | ((self.green as u32) << 8)
            | self.blue as u32
    }
}

/// A reusable source of color for drawing operations.
#[derive(Debug, Clone)]
pub enum Paint {
    Solid(Color),
}

impl Paint {
    pub const fn solid(color: Color) -> Self {
        Self::Solid(color)
    }
}
