use paintcore::affine::{Affine, InterpolationAlgorithm};
use paintcore::canvas::Screen;
use paintcore::image::ImageAlign;
use paintcore::layer::Layer;

fn rgba(screen: &dyn Screen, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * screen.width() + x) * 4) as usize;
    [
        screen.buffer()[offset],
        screen.buffer()[offset + 1],
        screen.buffer()[offset + 2],
        screen.buffer()[offset + 3],
    ]
}

fn alpha_test_source() -> Layer {
    Layer::new_in(
        "_alpha_test_".to_string(),
        vec![
            0xff, 0x00, 0x00, 0xff, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0x00, 0xff, 0xff,
            0xff, 0x00,
        ],
        2,
        2,
    )
}

fn assert_no_transparent_rgb_bleed(algorithm: InterpolationAlgorithm) {
    let source = alpha_test_source();
    let mut target = Layer::tmp(4, 4);

    Affine::resize(&source, &mut target, 2.0, algorithm, ImageAlign::LeftUp);

    let edge = rgba(&target, 1, 0);
    assert!(edge[0] > 0, "expected red coverage from the opaque pixel");
    assert_eq!(edge[1], 0, "transparent green pixels must not bleed");
    assert_eq!(edge[2], 0, "transparent blue pixels must not bleed");
    assert!(
        edge[3] < 0xff,
        "interpolated transparent edge should keep partial alpha"
    );
}

#[test]
fn bilinear_resize_is_alpha_aware() {
    assert_no_transparent_rgb_bleed(InterpolationAlgorithm::Bilinear);
}

#[test]
fn bicubic_resize_is_alpha_aware() {
    assert_no_transparent_rgb_bleed(InterpolationAlgorithm::Bicubic);
}

#[test]
fn lanczos_resize_is_alpha_aware() {
    assert_no_transparent_rgb_bleed(InterpolationAlgorithm::Lanczos3);
}

#[test]
fn pixel_mixing_downscale_is_alpha_aware() {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&[0xff, 0x00, 0x00, 0xff]);
    for _ in 1..16 {
        buffer.extend_from_slice(&[0x00, 0xff, 0x00, 0x00]);
    }
    let source = Layer::new_in("_alpha_downscale_".to_string(), buffer, 4, 4);
    let mut target = Layer::tmp(1, 1);

    Affine::resize(
        &source,
        &mut target,
        0.25,
        InterpolationAlgorithm::Bilinear,
        ImageAlign::LeftUp,
    );

    let pixel = rgba(&target, 0, 0);
    assert!(pixel[0] > 0, "expected red coverage from the opaque pixel");
    assert_eq!(pixel[1], 0, "transparent green pixels must not bleed");
    assert_eq!(pixel[2], 0, "transparent blue pixels must not bleed");
    assert!(
        pixel[3] > 0 && pixel[3] < 0xff,
        "downscaled alpha should remain partial"
    );
}
