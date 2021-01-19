use nalgebra::Complex;

#[derive(Clone)]
pub struct CountGrid {
    boxes: Vec<u64>,
    origin: Complex<f64>,
    max: Complex<f64>,
    pixel_width: f64,
    width: usize,
    height: usize,
}

impl CountGrid {
    pub fn new(origin: Complex<f64>, pixel_width: f64, width: usize, height: usize) -> CountGrid {
        CountGrid {
            boxes: vec![0; width * height],
            origin,
            max: Complex::new(
                origin.re + width as f64 * pixel_width,
                origin.im + width as f64 * pixel_width,
            ),
            pixel_width,
            width,
            height,
        }
    }

    fn set_value(&mut self, value: u64, x: usize, y: usize) {
        self.boxes[y * self.width + x] = value;
    }

    pub fn value(&self, x: usize, y: usize) -> u64 {
        self.boxes[y * self.width + x]
    }

    pub fn increment(&mut self, point: &Complex<f64>) {
        let delta = point - self.origin;
        let x = (delta.re / self.pixel_width) as usize;
        let y = (delta.im / self.pixel_width) as usize;
        self.boxes[y * self.width + x] += 1;
        //println!("Increment {}, ({}, {}), ({}, {}) {}", point, (delta.re / self.pixel_width), (delta.im / self.pixel_width), x, y, self.value(x, y));
    }

    pub fn increment_samples(&mut self, samples: &[Complex<f64>]) {
        for c in samples {
            self.increment(c);
        }
    }

    pub fn to_normalized_grid(&self) -> NormalizedGrid {
        let mut max = 0;
        for x in 0..self.width {
            for y in 0..self.height {
                if self.value(x, y) > max {
                    max = self.value(x, y);
                }
            }
        }

        let max_fp = max as f64;
        let mut result = NormalizedGrid::new(self.width, self.height);
        for x in 0..self.width {
            for y in 0..self.height {
                result.set_value(self.value(x, y) as f64 / max_fp, x, y);
            }
        }

        result
    }
}

pub struct NormalizedGrid {
    boxes: Vec<f64>,
    width: usize,
    height: usize,
}

impl NormalizedGrid {
    pub fn new(width: usize, height: usize) -> NormalizedGrid {
        NormalizedGrid {
            boxes: vec![0.0; width * height],
            width,
            height,
        }
    }

    fn set_value(&mut self, value: f64, x: usize, y: usize) {
        // TODO(check that 0.0 <= value <= 1.0)
        self.boxes[y * self.width + x] = value;
    }

    pub fn value(&self, x: usize, y: usize) -> f64 {
        self.boxes[y * self.width + x]
    }
}
