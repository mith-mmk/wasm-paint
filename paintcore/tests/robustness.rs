use std::collections::HashMap;

use paintcore::affine::{Affine, InterpolationAlgorithm};
use paintcore::canvas::{Canvas, Screen};
use paintcore::fill::fill;
use paintcore::filter::{
    combine, copy_to, filter_with_option, grayscale, median, rgb_filter, Kernel,
};
use paintcore::grayscale::to_grayscale;
use paintcore::image::ImageAlign;
use paintcore::layer::Layer;

fn layer(width: u32, height: u32, pixels: &[[u8; 4]]) -> Layer {
    let buffer = pixels
        .iter()
        .flat_map(|pixel| pixel.iter().copied())
        .collect();
    Layer::new_in("test".to_string(), buffer, width, height)
}

fn pixel(screen: &dyn Screen, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * screen.width() + x) * 4) as usize;
    [
        screen.buffer()[offset],
        screen.buffer()[offset + 1],
        screen.buffer()[offset + 2],
        screen.buffer()[offset + 3],
    ]
}

#[test]
fn fill_entire_region_does_not_read_past_scanline() {
    let mut target = Layer::new("fill".to_string(), 3, 3);

    fill(&mut target, 1, 1, 0x112233);

    assert!(target
        .buffer()
        .chunks_exact(4)
        .all(|pixel| pixel == [0x11, 0x22, 0x33, 0xff]));
}

#[test]
fn affine_right_alignment_uses_right_edge() {
    let source = layer(
        2,
        2,
        &[
            [255, 0, 0, 255],
            [255, 0, 0, 255],
            [255, 0, 0, 255],
            [255, 0, 0, 255],
        ],
    );
    let mut target = Layer::new("target".to_string(), 6, 4);

    Affine::resize(
        &source,
        &mut target,
        1.0,
        InterpolationAlgorithm::NearestNeighbor,
        ImageAlign::RightUp,
    );

    for y in 0..2 {
        for x in 4..6 {
            assert_eq!(pixel(&target, x, y), [255, 0, 0, 255], "x={x} y={y}");
        }
    }
    assert_eq!(
        target
            .buffer()
            .chunks_exact(4)
            .filter(|pixel| pixel[3] > 0)
            .count(),
        4
    );
    assert_eq!(pixel(&target, 0, 0), [0, 0, 0, 0]);
}

#[test]
fn affine_right_bottom_alignment_reaches_both_edges() {
    let source = layer(2, 2, &[[255, 0, 0, 255]; 4]);
    let mut target = Layer::new("target".to_string(), 6, 4);

    Affine::resize(
        &source,
        &mut target,
        1.0,
        InterpolationAlgorithm::NearestNeighbor,
        ImageAlign::RightBottom,
    );

    for y in 2..4 {
        for x in 4..6 {
            assert_eq!(pixel(&target, x, y), [255, 0, 0, 255], "x={x} y={y}");
        }
    }
    assert_eq!(pixel(&target, 5, 3), [255, 0, 0, 255]);
}

#[test]
fn affine_upscale_covers_the_full_projected_extent() {
    let source = layer(2, 2, &[[255, 0, 0, 255]; 4]);
    let mut target = Layer::new("target".to_string(), 4, 4);

    Affine::resize(
        &source,
        &mut target,
        2.0,
        InterpolationAlgorithm::NearestNeighbor,
        ImageAlign::LeftUp,
    );

    assert_eq!(
        target
            .buffer()
            .chunks_exact(4)
            .filter(|pixel| pixel[3] > 0)
            .count(),
        16
    );
    assert_eq!(pixel(&target, 3, 3), [255, 0, 0, 255]);
}

#[test]
fn affine_centered_downscale_with_negative_offset_is_clipped() {
    let source = Layer::new_in(
        "source".to_string(),
        (0..100 * 100).flat_map(|_| [255, 0, 0, 255]).collect(),
        100,
        100,
    );
    let mut target = Layer::new("target".to_string(), 20, 20);

    Affine::resize(
        &source,
        &mut target,
        0.4,
        InterpolationAlgorithm::Bilinear,
        ImageAlign::Center,
    );

    assert_eq!(
        target
            .buffer()
            .chunks_exact(4)
            .filter(|pixel| pixel[3] > 0)
            .count(),
        400
    );
}

#[test]
fn copy_to_uses_destination_stride_for_common_region() {
    let source = layer(
        2,
        2,
        &[
            [1, 2, 3, 4],
            [5, 6, 7, 8],
            [9, 10, 11, 12],
            [13, 14, 15, 16],
        ],
    );
    let mut target = Layer::new_in("target".to_string(), vec![99; 8], 1, 2);

    copy_to(&source, &mut target).unwrap();

    assert_eq!(pixel(&target, 0, 0), [1, 2, 3, 4]);
    assert_eq!(pixel(&target, 0, 1), [9, 10, 11, 12]);

    let mut empty_target = Layer::new("empty".to_string(), 0, 0);
    copy_to(&source, &mut empty_target).unwrap();
    assert_eq!((empty_target.width(), empty_target.height()), (2, 2));
    assert_eq!(pixel(&empty_target, 1, 1), [13, 14, 15, 16]);
}

#[test]
fn grayscale_and_ranking_preserve_destination_outside_common_region() {
    let source = layer(
        2,
        2,
        &[
            [200, 100, 50, 255],
            [10, 20, 30, 255],
            [30, 40, 50, 255],
            [60, 70, 80, 255],
        ],
    );
    let mut grayscale_target = Layer::new_in("gray".to_string(), vec![77; 24], 3, 2);
    let mut ranking_target = Layer::new_in("rank".to_string(), vec![88; 24], 3, 2);

    grayscale(&source, &mut grayscale_target).unwrap();
    median(&source, &mut ranking_target, 3).unwrap();

    assert_eq!(pixel(&grayscale_target, 2, 0), [77, 77, 77, 77]);
    assert_eq!(pixel(&grayscale_target, 2, 1), [77, 77, 77, 77]);
    assert_eq!(pixel(&ranking_target, 2, 0), [88, 88, 88, 88]);
    assert_eq!(pixel(&ranking_target, 2, 1), [88, 88, 88, 88]);
}

#[test]
fn combine_and_rgb_filter_handle_different_strides() {
    let source1 = layer(2, 2, &[[100, 0, 0, 255]; 4]);
    let source2 = layer(1, 2, &[[0, 100, 0, 255]; 2]);
    let mut combined = Layer::new_in("combined".to_string(), vec![66; 8], 1, 2);
    let mut filtered = Layer::new_in("filtered".to_string(), vec![55; 8], 1, 2);

    combine(&source1, &source2, &mut combined).unwrap();
    rgb_filter(&source1, &mut filtered, &Kernel::new([[1.0, 0.0, 0.0]; 3])).unwrap();

    assert_eq!(pixel(&combined, 0, 0)[3], 255);
    assert_eq!(pixel(&combined, 0, 1)[3], 255);
    assert_eq!(pixel(&filtered, 0, 0)[3], 255);
    assert_eq!(pixel(&filtered, 0, 1)[3], 255);
}

#[test]
fn invalid_kernel_sizes_and_options_return_errors() {
    assert!(Kernel::gaussian_kernel(0, 1.0).is_err());
    assert!(Kernel::gaussian_kernel(2, 1.0).is_err());
    assert!(Kernel::gaussian_kernel(33, 1.0).is_err());
    assert!(Kernel::gaussian_kernel(3, 0.0).is_err());
    assert!(Kernel::gaussian_kernel(3, f32::NAN).is_err());
    assert!(Kernel::gaussian_kernel(3, f32::INFINITY).is_err());

    let source = layer(1, 1, &[[1, 2, 3, 255]]);
    let mut target = Layer::new("target".to_string(), 1, 1);
    let mut options = HashMap::new();
    options.insert("size", -1.0);
    assert!(filter_with_option(&source, &mut target, "median", Some(options)).is_err());

    let mut options = HashMap::new();
    options.insert("size", 3.5);
    assert!(filter_with_option(&source, &mut target, "median", Some(options)).is_err());

    let mut options = HashMap::new();
    options.insert("size", f32::INFINITY);
    assert!(filter_with_option(&source, &mut target, "median", Some(options)).is_err());

    for size in [2.0, 33.0, f32::NAN] {
        let mut options = HashMap::new();
        options.insert("size", size);
        assert!(filter_with_option(&source, &mut target, "median", Some(options)).is_err());
    }

    let malformed = Kernel::from(3, 3, vec![vec![1.0; 2]; 3], false);
    assert!(rgb_filter(&source, &mut target, &malformed).is_err());

    let malformed = Kernel::from(3, 3, vec![vec![1.0; 3]; 2], false);
    assert!(rgb_filter(&source, &mut target, &malformed).is_err());

    let column_major = Kernel::from(3, 1, vec![vec![1.0], vec![0.0], vec![0.0]], false);
    let source = layer(3, 1, &[[10, 0, 0, 255], [20, 0, 0, 255], [30, 0, 0, 255]]);
    let mut target = Layer::new("target".to_string(), 3, 1);
    rgb_filter(&source, &mut target, &column_major).unwrap();
    assert_eq!(pixel(&target, 1, 0)[0], 10);

    let row_major = Kernel::from(1, 3, vec![vec![1.0, 0.0, 0.0]], false);
    let source = layer(1, 3, &[[10, 0, 0, 255], [20, 0, 0, 255], [30, 0, 0, 255]]);
    let mut target = Layer::new("target".to_string(), 1, 3);
    rgb_filter(&source, &mut target, &row_major).unwrap();
    assert_eq!(pixel(&target, 0, 1)[0], 10);
}

#[test]
fn canny_handles_empty_and_tiny_images() {
    for (width, height) in [(0, 0), (1, 1), (2, 2)] {
        let source = if width == 1 && height == 1 {
            layer(1, 1, &[[11, 22, 33, 44]])
        } else if width == 2 && height == 2 {
            layer(
                2,
                2,
                &[
                    [11, 22, 33, 44],
                    [55, 66, 77, 88],
                    [99, 110, 121, 132],
                    [143, 154, 165, 176],
                ],
            )
        } else {
            Layer::new("source".to_string(), width, height)
        };
        let mut target = Layer::new("target".to_string(), width, height);
        paintcore::filter::canny(&source, &mut target).unwrap();
        assert_eq!(target.width(), width);
        assert_eq!(target.height(), height);
        assert_eq!(target.buffer(), source.buffer());
    }
}

#[test]
fn checked_constructors_reject_overflow_and_legacy_constructors_fallback_empty() {
    assert!(Layer::try_new("layer".to_string(), 0, 0).is_ok());
    assert!(Canvas::try_new(0, 0).is_ok());
    assert!(Layer::try_new("layer".to_string(), u32::MAX, u32::MAX).is_err());
    assert!(Canvas::try_new(u32::MAX, 2).is_err());

    let layer = Layer::new("layer".to_string(), u32::MAX, u32::MAX);
    let canvas = Canvas::new(u32::MAX, 2);
    assert_eq!(
        (layer.width(), layer.height(), layer.buffer().len()),
        (0, 0, 0)
    );
    assert_eq!(
        (canvas.width(), canvas.height(), canvas.buffer().len()),
        (0, 0, 0)
    );
}

#[test]
fn grayscale_weighted_output_uses_destination_stride() {
    let source = layer(2, 2, &[[200, 100, 50, 255]; 4]);
    let mut target = Layer::new_in("target".to_string(), vec![42; 8], 1, 2);

    to_grayscale(&source, &mut target, 0);

    assert_eq!(pixel(&target, 0, 0)[0], pixel(&target, 0, 0)[1]);
    assert_eq!(pixel(&target, 0, 1)[0], pixel(&target, 0, 1)[2]);
}
