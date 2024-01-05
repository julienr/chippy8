use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub struct Array2D<T> {
    data: Vec<T>,
    rows: usize,
    cols: usize,
}

impl<T> Array2D<T>
where
    T: PartialEq,
{
    pub fn new<F>(rows: usize, cols: usize, f: F) -> Self
    where
        F: FnMut() -> T,
    {
        let mut data = Vec::new();
        data.resize_with(rows * cols, f);
        Array2D { data, rows, cols }
    }

    pub fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn count_value(&self, value: T) -> usize {
        self.data.iter().filter(|&v| *v == value).count()
    }
}

impl<T> Index<(usize, usize)> for Array2D<T> {
    type Output = T;

    fn index(&self, (row, column): (usize, usize)) -> &Self::Output {
        &self.data[row * self.cols + column]
    }
}

impl<T> IndexMut<(usize, usize)> for Array2D<T> {
    fn index_mut(&mut self, (row, column): (usize, usize)) -> &mut Self::Output {
        &mut self.data[row * self.cols + column]
    }
}
