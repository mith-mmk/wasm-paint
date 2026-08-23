//! Checked 8-bit coverage masks and paint-based mask filling.

use crate::{
    canvas::Screen,
    composite::{blend_pixel, DrawOptions},
    error::Error,
    paint::{Paint, PaintBounds},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mask {
    width: u32,
    height: u32,
    coverage: Vec<u8>,
}

impl Mask {
    pub fn try_new(width: u32, height: u32) -> Result<Self, Error> {
        let len = usize::try_from(width)
            .ok()
            .and_then(|width| {
                usize::try_from(height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .ok_or_else(|| Error {
                message: format!("mask buffer size overflow: {width}x{height}"),
            })?;
        let mut coverage = Vec::new();
        coverage.try_reserve_exact(len).map_err(|err| Error {
            message: format!("mask buffer allocation failed for {width}x{height}: {err}"),
        })?;
        coverage.resize(len, 0);
        Ok(Self {
            width,
            height,
            coverage,
        })
    }

    pub fn new(width: u32, height: u32) -> Self {
        Self::try_new(width, height).unwrap_or_else(|_| Self {
            width: 0,
            height: 0,
            coverage: Vec::new(),
        })
    }

    pub fn from_coverage(width: u32, height: u32, coverage: Vec<u8>) -> Result<Self, Error> {
        let expected = usize::try_from(width)
            .ok()
            .and_then(|width| {
                usize::try_from(height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .ok_or_else(|| Error {
                message: format!("mask buffer size overflow: {width}x{height}"),
            })?;
        if coverage.len() != expected {
            return Err(Error {
                message: format!(
                    "invalid mask coverage length: expected {expected}, got {}",
                    coverage.len()
                ),
            });
        }
        Ok(Self {
            width,
            height,
            coverage,
        })
    }

    pub fn from_rect(width: u32, height: u32, x: i32, y: i32, w: u32, h: u32) -> Self {
        let mut mask = Self::new(width, height);
        if mask.is_empty() {
            return mask;
        }
        let right = x.saturating_add(w.min(i32::MAX as u32) as i32);
        let bottom = y.saturating_add(h.min(i32::MAX as u32) as i32);
        let mask_width = width.min(i32::MAX as u32) as i32;
        let mask_height = height.min(i32::MAX as u32) as i32;
        let left = x.max(0).min(mask_width);
        let top = y.max(0).min(mask_height);
        let right = right.max(0).min(mask_width);
        let bottom = bottom.max(0).min(mask_height);
        for py in top..bottom {
            let row = py as usize * width as usize;
            for px in left..right {
                mask.coverage[row + px as usize] = 255;
            }
        }
        mask
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0 || self.coverage.is_empty()
    }

    pub fn coverage(&self) -> &[u8] {
        &self.coverage
    }

    pub fn coverage_mut(&mut self) -> &mut [u8] {
        &mut self.coverage
    }

    pub fn get(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return 0;
        }
        self.coverage[y as usize * self.width as usize + x as usize]
    }

    pub fn set(&mut self, x: i32, y: i32, value: u8) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        self.coverage[y as usize * self.width as usize + x as usize] = value;
    }

    pub fn invert(&mut self) {
        for value in &mut self.coverage {
            *value = 255 - *value;
        }
    }

    fn ensure_same_size(&self, other: &Self) -> Result<(), Error> {
        if self.width == other.width && self.height == other.height {
            Ok(())
        } else {
            Err(Error {
                message: format!(
                    "mask size mismatch: {}x{} and {}x{}",
                    self.width, self.height, other.width, other.height
                ),
            })
        }
    }

    pub fn union(&self, other: &Self) -> Result<Self, Error> {
        self.ensure_same_size(other)?;
        let coverage = self
            .coverage
            .iter()
            .zip(&other.coverage)
            .map(|(&a, &b)| a.max(b))
            .collect();
        Self::from_coverage(self.width, self.height, coverage)
    }

    pub fn intersect(&self, other: &Self) -> Result<Self, Error> {
        self.ensure_same_size(other)?;
        let coverage = self
            .coverage
            .iter()
            .zip(&other.coverage)
            .map(|(&a, &b)| a.min(b))
            .collect();
        Self::from_coverage(self.width, self.height, coverage)
    }

    pub fn difference(&self, other: &Self) -> Result<Self, Error> {
        self.ensure_same_size(other)?;
        let coverage = self
            .coverage
            .iter()
            .zip(&other.coverage)
            .map(|(&a, &b)| ((a as u16 * (255 - b) as u16 + 127) / 255) as u8)
            .collect();
        Self::from_coverage(self.width, self.height, coverage)
    }

    pub fn grow(&self, radius: u32) -> Self {
        self.morphology(radius, true)
    }

    pub fn shrink(&self, radius: u32) -> Self {
        self.morphology(radius, false)
    }

    fn morphology(&self, radius: u32, grow: bool) -> Self {
        if radius == 0 || self.is_empty() {
            return self.clone();
        }
        let mut output = Self::new(self.width, self.height);
        let radius = radius.min(self.width.max(self.height)).min(i32::MAX as u32) as i32;
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let mut result = if grow { 0 } else { 255 };
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let value = self.get(x.saturating_add(dx), y.saturating_add(dy));
                        result = if grow {
                            result.max(value)
                        } else {
                            result.min(value)
                        };
                    }
                }
                output.set(x, y, result);
            }
        }
        output
    }

    pub fn feather(&self, radius: u32) -> Self {
        if radius == 0 || self.is_empty() {
            return self.clone();
        }
        let radius = radius.min(self.width.max(self.height)).min(i32::MAX as u32) as i32;
        let mut horizontal = vec![0u8; self.coverage.len()];
        let mut output = Self::new(self.width, self.height);
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let mut total = 0u64;
                let mut weight = 0u64;
                for dx in -radius..=radius {
                    let sample_weight = (radius + 1 - dx.abs()) as u64;
                    total += self.get(x.saturating_add(dx), y) as u64 * sample_weight;
                    weight += sample_weight;
                }
                horizontal[y as usize * self.width as usize + x as usize] =
                    ((total + weight / 2) / weight) as u8;
            }
        }
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let mut total = 0u64;
                let mut weight = 0u64;
                for dy in -radius..=radius {
                    let sample_weight = (radius + 1 - dy.abs()) as u64;
                    let sy = y.saturating_add(dy);
                    let value = if sy < 0 || sy >= self.height as i32 {
                        0
                    } else {
                        horizontal[sy as usize * self.width as usize + x as usize]
                    };
                    total += value as u64 * sample_weight;
                    weight += sample_weight;
                }
                output.set(x, y, ((total + weight / 2) / weight) as u8);
            }
        }
        output
    }
}

pub fn fill_mask(
    screen: &mut dyn Screen,
    mask: &Mask,
    mask_x: i32,
    mask_y: i32,
    paint: &Paint,
    options: DrawOptions<'_>,
) {
    if mask.is_empty() {
        return;
    }
    let prepared = paint.prepare(PaintBounds::new(
        mask_x as f32,
        mask_y as f32,
        mask.width as f32,
        mask.height as f32,
    ));
    for y in 0..mask.height as i32 {
        for x in 0..mask.width as i32 {
            let coverage = mask.get(x, y);
            if coverage == 0 {
                continue;
            }
            let screen_x = mask_x.saturating_add(x);
            let screen_y = mask_y.saturating_add(y);
            blend_pixel(
                screen,
                screen_x,
                screen_y,
                prepared.sample(screen_x as f32 + 0.5, screen_y as f32 + 0.5),
                coverage as f32 / 255.0,
                options,
            );
        }
    }
}
