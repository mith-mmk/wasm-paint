use paintcore::canvas::Screen;
use paintcore::color::{
    auto_brightness, brightness_control, gamma_control, invert_crcb, saturation_control,
};
use paintcore::filter::{filter, harris, object_extract};
use paintcore::layer::Layer;

fn layer(width: u32, height: u32, pixels: &[[u8; 4]]) -> Layer {
    let buffer = pixels
        .iter()
        .flat_map(|pixel| pixel.iter().copied())
        .collect();
    Layer::new_in("test".to_string(), buffer, width, height)
}

#[test]
fn gamma_control_preserves_alpha_and_changes_channels() {
    let src = layer(1, 1, &[[64, 128, 192, 77]]);
    let mut dest = Layer::tmp(1, 1);

    gamma_control(&src, &mut dest, 2.0).unwrap();

    assert!(dest.buffer()[0] > src.buffer()[0]);
    assert!(dest.buffer()[1] > src.buffer()[1]);
    assert!(dest.buffer()[2] > src.buffer()[2]);
    assert_eq!(dest.buffer()[3], 77);
}

#[test]
fn saturation_zero_outputs_luminance_gray() {
    let src = layer(1, 1, &[[200, 100, 50, 255]]);
    let mut dest = Layer::tmp(1, 1);

    saturation_control(&src, &mut dest, 0.0).unwrap();

    assert_eq!(dest.buffer()[0], dest.buffer()[1]);
    assert_eq!(dest.buffer()[1], dest.buffer()[2]);
    assert_eq!(dest.buffer()[3], 255);
}

#[test]
fn brightness_and_auto_brightness_clamp_output() {
    let src = layer(2, 1, &[[10, 20, 30, 255], [200, 210, 220, 128]]);
    let mut bright = Layer::tmp(2, 1);
    let mut leveled = Layer::tmp(2, 1);

    brightness_control(&src, &mut bright, 80.0).unwrap();
    auto_brightness(&src, &mut leveled).unwrap();

    assert_eq!(&bright.buffer()[0..4], &[90, 100, 110, 255]);
    assert_eq!(bright.buffer()[7], 128);
    assert_eq!(leveled.buffer()[0], 0);
    assert_eq!(leveled.buffer()[6], 255);
}

#[test]
fn invert_crcb_changes_chroma_without_touching_alpha() {
    let src = layer(1, 1, &[[220, 40, 40, 99]]);
    let mut dest = Layer::tmp(1, 1);

    invert_crcb(&src, &mut dest).unwrap();

    assert_ne!(&dest.buffer()[0..3], &src.buffer()[0..3]);
    assert_eq!(dest.buffer()[3], 99);
}

#[test]
fn object_extract_makes_corner_colored_background_transparent() {
    let src = layer(
        3,
        3,
        &[
            [10, 10, 10, 255],
            [10, 10, 10, 255],
            [10, 10, 10, 255],
            [10, 10, 10, 255],
            [240, 240, 240, 255],
            [10, 10, 10, 255],
            [10, 10, 10, 255],
            [10, 10, 10, 255],
            [10, 10, 10, 255],
        ],
    );
    let mut dest = Layer::tmp(3, 3);

    object_extract(&src, &mut dest, 40.0).unwrap();

    assert_eq!(dest.buffer()[3], 0);
    assert_eq!(dest.buffer()[(4 * 4) + 3], 255);
}

#[test]
fn harris_corner_filter_outputs_corner_response() {
    let src = layer(
        5,
        5,
        &[
            [0, 0, 0, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],
            [255, 255, 255, 255],
            [255, 255, 255, 255],
            [255, 255, 255, 255],
            [255, 255, 255, 255],
            [0, 0, 0, 255],
            [255, 255, 255, 255],
            [255, 255, 255, 255],
            [255, 255, 255, 255],
            [255, 255, 255, 255],
            [0, 0, 0, 255],
            [255, 255, 255, 255],
            [255, 255, 255, 255],
            [255, 255, 255, 255],
            [255, 255, 255, 255],
            [0, 0, 0, 255],
            [255, 255, 255, 255],
            [255, 255, 255, 255],
            [255, 255, 255, 255],
            [255, 255, 255, 255],
        ],
    );
    let mut dest = Layer::tmp(5, 5);

    harris(&src, &mut dest, 0.05, 0.04).unwrap();

    assert!(dest.buffer().chunks_exact(4).any(|pixel| pixel[0] == 255));
}

#[test]
fn filter_dispatch_exposes_notion_todo_filter_names() {
    let src = layer(1, 1, &[[64, 128, 192, 255]]);
    let mut dest = Layer::tmp(1, 1);

    for name in [
        "gamma",
        "colorCurve",
        "invertCrCb",
        "saturation",
        "brightness",
        "autoBrightness",
        "objectExtract",
        "cornerDetect",
        "harris",
    ] {
        filter(&src, &mut dest, name).unwrap();
    }
}
