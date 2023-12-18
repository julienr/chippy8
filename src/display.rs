use crate::array2d::Array2D;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

pub struct Display {
    pub pixels: Array2D<bool>,
}

impl Default for Display {
    fn default() -> Self {
        Display {
            pixels: Array2D::new(DISPLAY_HEIGHT, DISPLAY_WIDTH, || false),
        }
    }
}
