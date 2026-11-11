use paintcore::canvas::Screen;
use paintcore::image_compat::RgbaImage;

fn source() -> RgbaImage {
    // Distinct samples make row/coordinate errors visible.
    RgbaImage::from_raw(3, 2, (0..24).collect()).unwrap()
}

#[test]
fn checked_pixels_use_coordinates_and_borrow_storage() {
    let mut image = source();
    assert_eq!(image.get_pixel(2, 1), Some(&[20, 21, 22, 23]));
    assert_eq!(
        image.get_pixel(0, 1).unwrap().as_ptr(),
        image.as_raw()[12..].as_ptr()
    );
    for (x, y) in [(3, 0), (0, 2), (u32::MAX, 1), (1, u32::MAX)] {
        assert!(image.get_pixel(x, y).is_none());
        assert!(image.get_pixel_mut(x, y).is_none());
    }
    *image.get_pixel_mut(1, 1).unwrap() = [91, 92, 93, 0];
    assert_eq!(&image.as_raw()[16..20], &[91, 92, 93, 0]);
    assert_eq!(&image.as_raw()[..16], &(0..16).collect::<Vec<_>>());
    assert_eq!(&image.as_raw()[20..], &[20, 21, 22, 23]);
}

#[test]
fn pixel_iterators_are_row_major_exact_and_mutable() {
    let mut image = source();
    assert_eq!(image.pixels().len(), 6);
    assert_eq!(image.pixels().next_back(), Some(&[20, 21, 22, 23]));
    assert_eq!(
        image.pixels().copied().collect::<Vec<_>>(),
        vec![
            [0, 1, 2, 3],
            [4, 5, 6, 7],
            [8, 9, 10, 11],
            [12, 13, 14, 15],
            [16, 17, 18, 19],
            [20, 21, 22, 23],
        ]
    );
    for (index, pixel) in image.pixels_mut().enumerate() {
        pixel[3] = index as u8;
    }
    assert_eq!(
        image.pixels().map(|p| p[3]).collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4, 5]
    );
    assert_eq!(image.pixels_mut().len(), 6);
    assert_eq!(image.dimensions(), (3, 2));
}

#[test]
fn view_keeps_parent_stride_and_uses_local_coordinates() {
    let image = source();
    let view = image.view(1, 0, 2, 2).unwrap();
    assert_eq!(view.dimensions(), (2, 2));
    assert_eq!((view.width(), view.height()), (2, 2));
    assert_eq!(view.row(0), Some(&[4, 5, 6, 7, 8, 9, 10, 11][..]));
    assert_eq!(view.row(1), Some(&[16, 17, 18, 19, 20, 21, 22, 23][..]));
    assert_eq!(view.row(0).unwrap().as_ptr(), image.as_raw()[4..].as_ptr());
    assert_eq!(view.row(1).unwrap().as_ptr(), image.as_raw()[16..].as_ptr());
    assert_eq!(view.get_pixel(0, 1), Some(&[16, 17, 18, 19]));
    assert!(view.get_pixel(2, 0).is_none());
    assert!(view.get_pixel(0, 2).is_none());
    assert!(view.get_pixel(u32::MAX, u32::MAX).is_none());
    assert!(view.row(2).is_none());
    assert_eq!(view.rows().len(), 2);
    assert_eq!(view.rows().next_back(), view.row(1));
    assert_eq!(
        view.rows().flatten().copied().collect::<Vec<_>>(),
        vec![4, 5, 6, 7, 8, 9, 10, 11, 16, 17, 18, 19, 20, 21, 22, 23,]
    );
}

#[test]
fn crop_is_independent_and_preserves_raw_samples_not_layer_opacity() {
    let mut layer = source().into_layer();
    layer.buffer_mut()[7] = 0; // Hidden RGB must survive without compositing.
    layer.set_alpha(0);
    layer.set_disable();
    let image = RgbaImage::from_layer(layer).unwrap();
    let mut crop = image.crop(1, 0, 1, 2).unwrap();
    assert_eq!(crop.dimensions(), (1, 2));
    assert_eq!(crop.as_raw(), &[4, 5, 6, 0, 16, 17, 18, 19]);
    *crop.get_pixel_mut(0, 0).unwrap() = [255, 255, 255, 0];
    assert_eq!(image.get_pixel(1, 0), Some(&[4, 5, 6, 0]));
    assert_eq!(image.crop(0, 0, 3, 2).unwrap().as_raw(), image.as_raw());
    assert_eq!(
        image.view(2, 1, 1, 1).unwrap().to_image().unwrap().as_raw(),
        &[20, 21, 22, 23]
    );
}

#[test]
fn view_and_crop_reject_out_of_bounds_without_clamping_or_wrapping() {
    let image = source();
    for (x, y, width, height) in [
        (4, 0, 0, 0),
        (0, 3, 0, 0),
        (2, 0, 2, 1),
        (0, 1, 1, 2),
        (u32::MAX, 0, 1, 1),
        (0, u32::MAX, 1, 1),
        (1, 0, u32::MAX, 1),
        (0, 1, 1, u32::MAX),
        (3, 0, 1, 0),
        (0, 2, 0, 1),
    ] {
        assert!(image.view(x, y, width, height).is_err());
        assert!(image.crop(x, y, width, height).is_err());
    }
}

#[test]
fn empty_views_allow_edges_but_have_no_pixels() {
    let image = source();
    for (x, y, width, height) in [(3, 0, 0, 2), (1, 2, 2, 0), (3, 2, 0, 0)] {
        let view = image.view(x, y, width, height).unwrap();
        assert_eq!(view.dimensions(), (width, height));
        assert!(view.get_pixel(0, 0).is_none());
        assert_eq!(view.rows().len(), height as usize);
        assert!(view.rows().all(|row| row.is_empty()));
        let crop = view.to_image().unwrap();
        assert_eq!(crop.dimensions(), (width, height));
        assert!(crop.as_raw().is_empty());
    }
}

#[test]
fn huge_empty_shapes_never_calculate_an_invalid_pixel_offset() {
    for (width, height) in [(0, 0), (0, u32::MAX), (u32::MAX, 0)] {
        let mut image = RgbaImage::new(width, height).unwrap();
        assert!(image.get_pixel(0, 0).is_none());
        assert!(image.get_pixel_mut(0, 0).is_none());
        assert_eq!(image.pixels().len(), 0);
        assert_eq!(image.pixels_mut().len(), 0);
        let view = image.view(0, 0, width, height).unwrap();
        assert_eq!(view.rows().len(), height as usize);
        if height != 0 {
            assert_eq!(view.row(height - 1), Some(&[][..]));
        }
        // This must finish without iterating billions of empty rows.
        assert_eq!(view.to_image().unwrap().dimensions(), (width, height));
    }
}

#[test]
fn crop_drops_source_metadata() {
    use paintcore::image_compat::{load_from_memory, ImageFormat};
    let encoded = source().encode_to_vec(ImageFormat::Png).unwrap();
    let decoded = load_from_memory(&encoded).unwrap();
    assert!(decoded.image.metadata().is_some());
    assert!(decoded.image.crop(0, 0, 1, 1).unwrap().metadata().is_none());
}

#[test]
fn all_small_rectangles_match_independent_coordinate_expectations() {
    for width in 0..=4 {
        for height in 0..=4 {
            let bytes = (0..height)
                .flat_map(|y| (0..width).flat_map(move |x| [x as u8, y as u8, 23, 0]))
                .collect();
            let image = RgbaImage::from_raw(width, height, bytes).unwrap();
            for y in 0..=height {
                for x in 0..=width {
                    for crop_height in 0..=height - y {
                        for crop_width in 0..=width - x {
                            let expected: Vec<u8> = (y..y + crop_height)
                                .flat_map(|py| {
                                    (x..x + crop_width)
                                        .flat_map(move |px| [px as u8, py as u8, 23, 0])
                                })
                                .collect();
                            let view = image.view(x, y, crop_width, crop_height).unwrap();
                            assert_eq!(view.rows().len(), crop_height as usize);
                            assert_eq!(
                                view.rows().flatten().copied().collect::<Vec<_>>(),
                                expected
                            );
                            let crop = image.crop(x, y, crop_width, crop_height).unwrap();
                            assert_eq!(crop.dimensions(), (crop_width, crop_height));
                            assert_eq!(crop.as_raw(), expected);
                        }
                    }
                }
            }
        }
    }
}
