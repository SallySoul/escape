#![allow(unused_imports)]
#![allow(dead_code)]

use image::ImageError;
use image::{Rgb, RgbImage};
use nalgebra::Complex;
use rand::distributions::Distribution;

#[derive(Debug, thiserror::Error)]
enum EscapeError {
    #[error("default error")]
    Default,

    #[error("Image conversion")]
    Image(#[from] ImageError),
}

fn error() -> Result<(), EscapeError> {
    Ok(())
}

struct Grid {
    boxes: Vec<u32>,
    origin: Complex<f64>,
    max: Complex<f64>,
    pixel_width: f64,
    width: usize,
    height: usize,
}

impl Grid {
    fn new(origin: Complex<f64>, pixel_width: f64, width: usize, height: usize) -> Grid {
        Grid {
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

    fn get_box(&mut self, x: usize, y: usize) -> &mut u32 {
        &mut self.boxes[y * self.width + x]
    }

    fn value(&self, x: usize, y: usize) -> u32 {
        self.boxes[y * self.width + x]
    }

    fn increment(&mut self, point: &Complex<f64>) {
        let delta = self.origin - point;
        let x = (delta.re / self.pixel_width) as usize;
        let y = (delta.im / self.pixel_width) as usize;
        self.boxes[y * self.width + x] += 1;
        //println!("Increment {}, ({}, {}), ({}, {}) {}", point, (delta.re / self.pixel_width), (delta.im / self.pixel_width), x, y, self.value(x, y));
    }

    fn to_image_buffer<P: Fn(f64) -> Rgb<u8>>(&self, pred: P) -> RgbImage {
        let mut result = RgbImage::new(self.width as u32, self.height as u32);

        let mut max = 0;
        for x in 0..self.width {
            for y in 0..self.height {
                if self.value(x, y) > max {
                    max = self.value(x, y);
                }
            }
        }

        for x in 0..self.width {
            for y in 0..self.height {
                let value = self.value(x, y) as f64 / max as f64;
                //println!("({}, {}) -> {}, value: {}", x, y, self.value(x, y), value);
                result.put_pixel(x as u32, y as u32, pred(value));
            }
        }
        result
    }
}

fn main() -> Result<(), EscapeError> {
    let min = Complex::new(-2.0, -2.0);

    let width = 1600;
    let height = 1600;

    let mut grid1 = Grid::new(min, -0.0025, width, height);
    let mut grid2 = Grid::new(min, -0.0025, width, height);
    let mut grid3 = Grid::new(min, -0.0025, width, height);

    let mut iterations = Vec::with_capacity(51);
    let total = 1000000000;
    for r in 0..total {
        if r % 10000  == 0 {
            println!("i: {}, {}%", r, (r as f64 / total as f64) * 100.0);
        }


        let between = rand::distributions::Uniform::from(-2.0..2.0);
        let mut rng = rand::thread_rng();

        // Generate random c in range
        let c = Complex::new(between.sample(&mut rng), between.sample(&mut rng));
        let mut z = c;

        // iterate mandlebrot eq, keep iterations
        let mut i = 0;
        while z.norm_sqr() <= 4.0 && i < 10001 {
            //println!("i: {}, z: {}, z norm {}", i, z, z.norm_sqr());
            iterations.push(z);
            z = z * z + c;
            i += 1;
        }

        if i < 200 {
            for c in &iterations {
                grid1.increment(c);
            }
        }

        if i < 5000 {
            for c in &iterations {
                grid2.increment(c);
            }
        }

        if i < 10000 {
            for c in &iterations {
                grid3.increment(c);
            }
        }

        iterations.clear();
    }

    let mut result = RgbImage::new(width as u32, height as u32);

    let mut max1 = 0;
    let mut max2 = 0;
    let mut max3 = 0;
    for x in 0..width {
        for y in 0..height {
            if grid1.value(x, y) > max1 {
                max1 = grid1.value(x, y);
            }
            if grid2.value(x, y) > max2 {
                max2 = grid2.value(x, y);
            }
            if grid3.value(x, y) > max3 {
                max3 = grid3.value(x, y);
            }
        }
    }

    for x in 0..width {
        for y in 0..height {
            let v1 = ((grid1.value(x, y) as f64 / max1 as f64) * 255.0) as u8;
            let v2 = ((grid2.value(x, y) as f64 / max2 as f64) * 255.0) as u8;
            let v3 = ((grid3.value(x, y) as f64 / max3 as f64) * 255.0) as u8;

            //println!("({}, {}) -> {}, value: {}", x, y, self.value(x, y), value);
            result.put_pixel(x as u32, y as u32, Rgb([v1, v2, v3]));
        }
    }

    result.save("Result.png")?;

    Ok(())
}

/*
#[tokio::main]
async fn main() -> Result<(), EscapeError> {
    println!("Hello World");
    error()?;
    Ok(())
}
*/
