use paintcore::prelude::*;
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

struct CountingAllocator;
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn measure(label: &str, operation: impl FnOnce()) {
    ALLOCATIONS.store(0, Ordering::Relaxed);
    let start = Instant::now();
    operation();
    println!(
        "{label}: {:.3}s, heap allocations={}",
        start.elapsed().as_secs_f64(),
        ALLOCATIONS.load(Ordering::Relaxed)
    );
}

fn main() {
    const WIDTH: u32 = 3840;
    const HEIGHT: u32 = 2160;
    let mut surface = Layer::new("uhd".to_string(), WIDTH, HEIGHT);
    let gradient = Paint::LinearGradient(LinearGradient {
        start: (0.0, 0.0),
        end: (WIDTH as f32, HEIGHT as f32),
        stops: vec![
            ColorStop::new(0.0, Color::rgba(255, 48, 32, 220)),
            ColorStop::new(0.5, Color::rgba(32, 255, 128, 160)),
            ColorStop::new(1.0, Color::rgba(48, 64, 255, 240)),
        ],
        spread: SpreadMode::Pad,
        units: PaintUnits::UserSpaceOnUse,
        transform: PaintTransform::IDENTITY,
        interpolation: GradientInterpolation::LinearSrgb,
    });
    measure("UHD linear gradient", || {
        fill_paint_rect(
            &mut surface,
            0,
            0,
            WIDTH,
            HEIGHT,
            &gradient,
            DrawOptions::default(),
        );
    });

    let pattern = Paint::Pattern(Pattern {
        image: PatternImage::new(
            2,
            2,
            vec![
                255, 255, 255, 180, 24, 32, 48, 220, 24, 32, 48, 220, 255, 255, 255, 180,
            ],
        )
        .expect("valid benchmark pattern"),
        tile_x: TileMode::Mirror,
        tile_y: TileMode::Repeat,
        sampling: SamplingMode::Bilinear,
        transform: PaintTransform::new([8.0, 2.0, -2.0, 8.0, 0.0, 0.0]),
    });
    measure("UHD transformed pattern", || {
        fill_paint_rect(
            &mut surface,
            0,
            0,
            WIDTH,
            HEIGHT,
            &pattern,
            DrawOptions::default(),
        );
    });

    let mut brush = BrushStroke::new(
        BrushTip::circle(24),
        BrushSettings {
            size: 24.0,
            spacing: 0.2,
            scatter: 0.1,
            pressure: PressureMapping {
                size: 1.0,
                opacity: 0.5,
                flow: 0.0,
            },
            ..BrushSettings::default()
        },
        2026,
    );
    measure("long pressure brush stroke", || {
        for x in (32..WIDTH - 32).step_by(48) {
            let phase = x as f32 / WIDTH as f32;
            let sample = StrokeSample {
                x: x as f32,
                y: HEIGHT as f32 * 0.5 + (phase * std::f32::consts::TAU * 8.0).sin() * 320.0,
                pressure: 0.2 + phase * 0.8,
                tilt_x: 0.0,
                tilt_y: 0.0,
                rotation: phase * std::f32::consts::TAU,
                timestamp: phase as f64 * 1_000.0,
            };
            brush.draw_sample(
                &mut surface,
                sample,
                &Paint::Solid(Color::rgba(255, 128, 32, 200)),
                DrawOptions::default(),
            );
        }
    });
}
