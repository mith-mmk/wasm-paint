//! Stateful, distance-spaced brush dabs.

use crate::{
    canvas::Screen,
    composite::{blend_pixel, DrawOptions},
    error::Error,
    mask::Mask,
    paint::{Paint, PaintBounds, PreparedPaint},
    pen::Pen,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrushTip {
    mask: Mask,
}

impl BrushTip {
    pub fn circle(diameter: u32) -> Self {
        let mut mask = Mask::new(diameter, diameter);
        if diameter == 0 {
            return Self { mask };
        }
        let center = diameter as f32 * 0.5;
        let radius = center;
        for y in 0..diameter as i32 {
            for x in 0..diameter as i32 {
                let mut covered = 0u32;
                for sy in 0..4 {
                    for sx in 0..4 {
                        let px = x as f32 + (sx as f32 + 0.5) / 4.0 - center;
                        let py = y as f32 + (sy as f32 + 0.5) / 4.0 - center;
                        if px * px + py * py <= radius * radius {
                            covered += 1;
                        }
                    }
                }
                mask.set(x, y, ((covered * 255 + 8) / 16) as u8);
            }
        }
        Self { mask }
    }

    pub fn rectangle(width: u32, height: u32) -> Self {
        Self {
            mask: Mask::from_rect(width, height, 0, 0, width, height),
        }
    }

    pub fn from_mask(mask: Mask) -> Self {
        Self { mask }
    }

    pub fn from_pen(pen: &Pen) -> Result<Self, Error> {
        Ok(Self {
            mask: Mask::from_coverage(pen.width(), pen.height(), pen.pen().to_vec())?,
        })
    }

    pub fn mask(&self) -> &Mask {
        &self.mask
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeSample {
    pub x: f32,
    pub y: f32,
    pub pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub rotation: f32,
    pub timestamp: f64,
}

impl StrokeSample {
    pub fn new(x: f32, y: f32, pressure: f32, timestamp: f64) -> Self {
        Self {
            x,
            y,
            pressure,
            tilt_x: 0.0,
            tilt_y: 0.0,
            rotation: 0.0,
            timestamp,
        }
    }

    fn interpolate(self, other: Self, t: f32) -> Self {
        let lerp = |a: f32, b: f32| a + (b - a) * t;
        Self {
            x: lerp(self.x, other.x),
            y: lerp(self.y, other.y),
            pressure: lerp(self.pressure, other.pressure),
            tilt_x: lerp(self.tilt_x, other.tilt_x),
            tilt_y: lerp(self.tilt_y, other.tilt_y),
            rotation: lerp(self.rotation, other.rotation),
            timestamp: self.timestamp + (other.timestamp - self.timestamp) * t as f64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PressureMapping {
    pub size: f32,
    pub opacity: f32,
    pub flow: f32,
}

impl Default for PressureMapping {
    fn default() -> Self {
        Self {
            size: 1.0,
            opacity: 0.0,
            flow: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrushSettings {
    pub size: f32,
    pub opacity: f32,
    pub flow: f32,
    /// Dab distance as a fraction of the unpressured brush size.
    pub spacing: f32,
    pub rotation: f32,
    /// Maximum deterministic displacement as a fraction of dab size.
    pub scatter: f32,
    pub pressure: PressureMapping,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            size: 16.0,
            opacity: 1.0,
            flow: 1.0,
            spacing: 0.2,
            rotation: 0.0,
            scatter: 0.0,
            pressure: PressureMapping::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrushDab {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub opacity: f32,
    pub flow: f32,
    pub rotation: f32,
    pub timestamp: f64,
}

#[derive(Debug, Clone)]
pub struct BrushStroke {
    tip: BrushTip,
    settings: BrushSettings,
    previous: Option<StrokeSample>,
    distance_since_dab: f32,
    random_state: u64,
}

impl BrushStroke {
    pub fn new(tip: BrushTip, settings: BrushSettings, seed: u64) -> Self {
        Self {
            tip,
            settings,
            previous: None,
            distance_since_dab: 0.0,
            random_state: if seed == 0 {
                0x9e37_79b9_7f4a_7c15
            } else {
                seed
            },
        }
    }

    pub fn tip(&self) -> &BrushTip {
        &self.tip
    }

    pub fn settings(&self) -> &BrushSettings {
        &self.settings
    }

    fn random_unit(&mut self) -> f32 {
        let mut value = self.random_state;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.random_state = value;
        ((value >> 40) as u32) as f32 / 16_777_215.0
    }

    fn dab_from_sample(&mut self, sample: StrokeSample) -> Option<BrushDab> {
        if !sample.x.is_finite() || !sample.y.is_finite() {
            return None;
        }
        let pressure = if sample.pressure.is_finite() {
            sample.pressure.clamp(0.0, 1.0)
        } else {
            1.0
        };
        let pressure_value = |amount: f32| 1.0 + (pressure - 1.0) * amount.clamp(0.0, 1.0);
        let size =
            (self.settings.size.max(0.0) * pressure_value(self.settings.pressure.size)).max(0.0);
        let mut x = sample.x;
        let mut y = sample.y;
        let scatter = self.settings.scatter.max(0.0) * size;
        if scatter > 0.0 {
            let angle = self.random_unit() * std::f32::consts::TAU;
            let radius = self.random_unit().sqrt() * scatter;
            x += angle.cos() * radius;
            y += angle.sin() * radius;
        }
        Some(BrushDab {
            x,
            y,
            size,
            opacity: (self.settings.opacity * pressure_value(self.settings.pressure.opacity))
                .clamp(0.0, 1.0),
            flow: (self.settings.flow * pressure_value(self.settings.pressure.flow))
                .clamp(0.0, 1.0),
            rotation: self.settings.rotation + sample.rotation,
            timestamp: sample.timestamp,
        })
    }

    pub fn push_sample(&mut self, sample: StrokeSample) -> Vec<BrushDab> {
        let Some(previous) = self.previous else {
            self.previous = Some(sample);
            self.distance_since_dab = 0.0;
            return self.dab_from_sample(sample).into_iter().collect();
        };
        self.previous = Some(sample);
        if !previous.x.is_finite()
            || !previous.y.is_finite()
            || !sample.x.is_finite()
            || !sample.y.is_finite()
        {
            return Vec::new();
        }
        let dx = sample.x - previous.x;
        let dy = sample.y - previous.y;
        let distance = dx.hypot(dy);
        if !distance.is_finite() || distance <= f32::EPSILON {
            return Vec::new();
        }
        let spacing = (self.settings.size.max(0.0) * self.settings.spacing.max(0.0)).max(0.1);
        let mut next_distance = spacing - self.distance_since_dab;
        let mut dabs = Vec::new();
        while next_distance <= distance + 1.0e-5 {
            let interpolated =
                previous.interpolate(sample, (next_distance / distance).clamp(0.0, 1.0));
            if let Some(dab) = self.dab_from_sample(interpolated) {
                dabs.push(dab);
            }
            next_distance += spacing;
        }
        self.distance_since_dab = (self.distance_since_dab + distance).rem_euclid(spacing);
        dabs
    }

    pub fn draw_sample(
        &mut self,
        screen: &mut dyn Screen,
        sample: StrokeSample,
        paint: &Paint,
        options: DrawOptions<'_>,
    ) -> usize {
        let dabs = self.push_sample(sample);
        draw_brush_dabs(screen, &self.tip, &dabs, paint, options);
        dabs.len()
    }
}

fn tip_coverage(tip: &BrushTip, x: f32, y: f32) -> f32 {
    if x < -0.5
        || y < -0.5
        || x > tip.mask.width() as f32 - 0.5
        || y > tip.mask.height() as f32 - 0.5
    {
        return 0.0;
    }
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let tx = x - x0 as f32;
    let ty = y - y0 as f32;
    let sample = |sx, sy| tip.mask.get(sx, sy) as f32 / 255.0;
    let top = sample(x0, y0) * (1.0 - tx) + sample(x0 + 1, y0) * tx;
    let bottom = sample(x0, y0 + 1) * (1.0 - tx) + sample(x0 + 1, y0 + 1) * tx;
    top * (1.0 - ty) + bottom * ty
}

fn draw_prepared_dab(
    screen: &mut dyn Screen,
    tip: &BrushTip,
    dab: BrushDab,
    paint: &PreparedPaint,
    options: DrawOptions<'_>,
) {
    if dab.size <= 0.0 || tip.mask.is_empty() {
        return;
    }
    let base_size = tip.mask.width().max(tip.mask.height()) as f32;
    if base_size <= 0.0 {
        return;
    }
    let scale = dab.size / base_size;
    let extent = (tip.mask.width() as f32 * scale).hypot(tip.mask.height() as f32 * scale) * 0.5;
    let left = (dab.x - extent).floor() as i32;
    let right = (dab.x + extent).ceil() as i32;
    let top = (dab.y - extent).floor() as i32;
    let bottom = (dab.y + extent).ceil() as i32;
    let (sin, cos) = dab.rotation.sin_cos();
    for y in top..=bottom {
        for x in left..=right {
            let dx = x as f32 + 0.5 - dab.x;
            let dy = y as f32 + 0.5 - dab.y;
            let local_x = (cos * dx + sin * dy) / scale + tip.mask.width() as f32 * 0.5 - 0.5;
            let local_y = (-sin * dx + cos * dy) / scale + tip.mask.height() as f32 * 0.5 - 0.5;
            let coverage = tip_coverage(tip, local_x, local_y) * dab.opacity * dab.flow;
            if coverage > 0.0 {
                blend_pixel(
                    screen,
                    x,
                    y,
                    paint.sample(x as f32 + 0.5, y as f32 + 0.5),
                    coverage,
                    options,
                );
            }
        }
    }
}

pub fn draw_brush_dabs(
    screen: &mut dyn Screen,
    tip: &BrushTip,
    dabs: &[BrushDab],
    paint: &Paint,
    options: DrawOptions<'_>,
) {
    let prepared = paint.prepare(PaintBounds::new(
        0.0,
        0.0,
        screen.width() as f32,
        screen.height() as f32,
    ));
    for &dab in dabs {
        draw_prepared_dab(screen, tip, dab, &prepared, options);
    }
}
