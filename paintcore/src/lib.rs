pub fn test() {

}

pub mod canvas;
pub mod clear;
pub mod draw;
pub mod layer;
pub mod utils;
pub mod pen;
pub mod point;
pub mod error;
pub mod affine;
pub mod circle;
pub mod fill;
pub mod filter;
pub mod grayscale;
pub mod image;
pub mod line;
pub mod polygram;
pub mod rect;
pub mod spline;

pub mod prelude;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
