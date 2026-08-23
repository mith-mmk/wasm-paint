use paintcore::prelude::*;

fn layer(width: u32, height: u32, pixels: &[[u8; 4]]) -> Layer {
    let mut layer = Layer::new("test".to_string(), width, height);
    for (target, pixel) in layer.buffer_mut().chunks_exact_mut(4).zip(pixels) {
        target.copy_from_slice(pixel);
    }
    layer
}

#[test]
fn mask_boolean_operations_preserve_coverage() {
    let a = Mask::from_coverage(3, 1, vec![255, 128, 0]).unwrap();
    let b = Mask::from_coverage(3, 1, vec![0, 128, 255]).unwrap();
    assert_eq!(a.union(&b).unwrap().coverage(), &[255, 128, 255]);
    assert_eq!(a.intersect(&b).unwrap().coverage(), &[0, 128, 0]);
    assert_eq!(a.difference(&b).unwrap().coverage(), &[255, 64, 0]);
    assert!(a.union(&Mask::new(1, 1)).is_err());
}

#[test]
fn rect_mask_clips_and_inverts() {
    let mut mask = Mask::from_rect(3, 2, -1, 0, 3, 1);
    assert_eq!(mask.coverage(), &[255, 255, 0, 0, 0, 0]);
    mask.invert();
    assert_eq!(mask.coverage(), &[0, 0, 255, 255, 255, 255]);
}

#[test]
fn grow_shrink_and_feather_have_defined_edges() {
    let mut point = Mask::new(5, 5);
    point.set(2, 2, 255);
    let grown = point.grow(1);
    assert_eq!(grown.coverage().iter().filter(|&&v| v == 255).count(), 9);
    assert_eq!(grown.shrink(1).get(2, 2), 255);
    assert_eq!(grown.shrink(1).get(1, 1), 0);

    let feathered = point.feather(1);
    assert!(feathered.get(2, 2) > feathered.get(1, 2));
    assert!(feathered.get(1, 2) > 0);
    assert_eq!(feathered.get(0, 0), 0);
}

#[test]
fn fill_mask_uses_gradient_and_coverage() {
    let mut target = Layer::new("target".to_string(), 3, 1);
    let mask = Mask::from_coverage(3, 1, vec![255, 128, 0]).unwrap();
    let paint = Paint::LinearGradient(LinearGradient {
        start: (0.0, 0.0),
        end: (3.0, 0.0),
        stops: vec![
            ColorStop::new(0.0, Color::rgb(255, 0, 0)),
            ColorStop::new(1.0, Color::rgb(0, 0, 255)),
        ],
        spread: SpreadMode::Pad,
        units: PaintUnits::UserSpaceOnUse,
        transform: PaintTransform::IDENTITY,
        interpolation: GradientInterpolation::Srgb,
    });
    fill_mask(&mut target, &mask, 0, 0, &paint, DrawOptions::default());
    assert_eq!(target.buffer()[3], 255);
    assert!((126..=129).contains(&target.buffer()[7]));
    assert_eq!(&target.buffer()[8..12], &[0, 0, 0, 0]);
}

#[test]
fn draw_options_clip_mask_multiplies_source_coverage() {
    let mut target = Layer::new("target".to_string(), 3, 1);
    let clip = Mask::from_coverage(2, 1, vec![255, 128]).unwrap();
    fill_rect_with_options(
        &mut target,
        0,
        0,
        3,
        1,
        Color::rgb(255, 0, 0),
        DrawOptions {
            clip_mask: Some(ClipMask::new(&clip, 1, 0)),
            ..DrawOptions::default()
        },
    );
    assert_eq!(&target.buffer()[0..4], &[0, 0, 0, 0]);
    assert_eq!(&target.buffer()[4..8], &[255, 0, 0, 255]);
    assert_eq!(&target.buffer()[8..11], &[255, 0, 0]);
    assert!((127..=128).contains(&target.buffer()[11]));
}

#[test]
fn flood_tolerance_and_reference_surface_are_independent() {
    let reference = layer(
        3,
        1,
        &[[10, 10, 10, 255], [14, 10, 10, 255], [20, 10, 10, 255]],
    );
    let exact = flood_mask(&reference, 0, 0, FloodOptions::default());
    assert_eq!(exact.coverage(), &[255, 0, 0]);
    let tolerant = flood_mask(
        &reference,
        0,
        0,
        FloodOptions {
            tolerance: 4,
            ..FloodOptions::default()
        },
    );
    assert_eq!(tolerant.coverage(), &[255, 255, 0]);

    let mut target = Layer::new("target".to_string(), 3, 1);
    fill_mask(
        &mut target,
        &tolerant,
        0,
        0,
        &Paint::Solid(Color::rgb(0, 255, 0)),
        DrawOptions::default(),
    );
    assert_eq!(&target.buffer()[0..4], &[0, 255, 0, 255]);
    assert_eq!(reference.buffer()[0], 10);
}

#[test]
fn flood_rgb_and_rgba_comparison_differ_on_alpha() {
    let reference = layer(2, 1, &[[40, 50, 60, 10], [40, 50, 60, 200]]);
    let rgb = flood_mask(&reference, 0, 0, FloodOptions::default());
    assert_eq!(rgb.coverage(), &[255, 255]);
    let rgba = flood_mask(
        &reference,
        0,
        0,
        FloodOptions {
            color_mode: FloodColorMode::Rgba,
            ..FloodOptions::default()
        },
    );
    assert_eq!(rgba.coverage(), &[255, 0]);
}

#[test]
fn flood_four_and_eight_connectivity_differ_for_diagonals() {
    let reference = layer(
        2,
        2,
        &[
            [1, 1, 1, 255],
            [2, 2, 2, 255],
            [2, 2, 2, 255],
            [1, 1, 1, 255],
        ],
    );
    let four = flood_mask(&reference, 0, 0, FloodOptions::default());
    assert_eq!(four.coverage().iter().filter(|&&v| v != 0).count(), 1);
    let eight = flood_mask(
        &reference,
        0,
        0,
        FloodOptions {
            connectivity: FloodConnectivity::Eight,
            ..FloodOptions::default()
        },
    );
    assert_eq!(eight.coverage().iter().filter(|&&v| v != 0).count(), 2);
}

#[test]
fn flood_boundaries_and_huge_masks_fail_safely() {
    let reference = layer(1, 1, &[[1, 2, 3, 4]]);
    assert_eq!(
        flood_mask(&reference, -1, 0, FloodOptions::default()).coverage(),
        &[0]
    );
    assert_eq!(
        flood_mask(&reference, 1, 0, FloodOptions::default()).coverage(),
        &[0]
    );
    assert!(Mask::try_new(u32::MAX, u32::MAX).is_err());
    assert!(Mask::from_coverage(2, 2, vec![0; 3]).is_err());
}
