#![allow(
    dead_code,
    clippy::doc_overindented_list_items,
    clippy::enum_variant_names,
    clippy::explicit_counter_loop,
    clippy::needless_range_loop,
    clippy::too_many_arguments
)]

pub fn test() {}

pub mod affine;
pub mod canvas;
pub mod circle;
pub mod clear;
pub mod color;
pub mod draw;
pub mod error;
pub mod fill;
pub mod filter;
pub mod grayscale;
pub mod image;
pub mod layer;
pub mod line;
pub mod math;
pub mod path;
pub mod pen;
pub mod point;
pub mod polygram;
pub mod rect;
pub mod spline;
pub mod text;
pub mod utils;

pub mod prelude;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
