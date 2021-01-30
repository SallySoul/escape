use nalgebra::Complex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Grid<N: num_traits::Num + Copy + Clone> {
    boxes: Vec<N>,
    width: usize,
    height: usize,
}

impl<N: num_traits::Num + Copy + Clone> Grid<N> {
    pub fn new(width: usize, height: usize) -> Grid<N> {
        Grid {
            boxes: vec![N::zero(); width * height],
            width,
            height,
        }
    }

    pub fn set_value(&mut self, value: N, x: usize, y: usize) {
        self.boxes[y * self.width + x] = value;
    }

    pub fn value(&self, x: usize, y: usize) -> N {
        self.boxes[y * self.width + x]
    }

    pub fn increment(&mut self, x: usize, y: usize) {
        let temp = self.boxes[y * self.width + x] + N::one();
        self.boxes[y * self.width + x] = temp;
    }
}

impl Grid<u64> {
    pub fn to_normalized_grid(&self) -> Grid<f64> {
        let mut max = 0;
        for x in 0..self.width {
            for y in 0..self.height {
                if self.value(x, y) > max {
                    max = self.value(x, y);
                }
            }
        }

        let max_fp = max as f64;
        let mut result = Grid::new(self.width, self.height);
        for x in 0..self.width {
            for y in 0..self.height {
                result.set_value(self.value(x, y) as f64 / max_fp, x, y);
            }
        }

        result
    }
}
