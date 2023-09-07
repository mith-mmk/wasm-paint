pub fn test() {}

pub mod affine;
pub mod canvas;
pub mod circle;
pub mod clear;
pub mod draw;
pub mod error;
pub mod fill;
pub mod filter;
pub mod grayscale;
pub mod image;
pub mod layer;
pub mod line;
pub mod path;
pub mod pen;
pub mod point;
pub mod polygram;
pub mod rect;
pub mod spline;
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
