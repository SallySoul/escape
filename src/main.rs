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

    fn to_image_buffer(&self) -> RgbImage {
        let mut result = RgbImage::new(self.width as u32, self.height as u32);

        let max = 0;
        for x in 0..self.width {
            for y in 0..self.height {
                if self.value(x, y) > max {
                    max = self.value(x, y);
                }
            }
        }

        for x in 0..self.width {
            for y in 0..self.height {
                let value = ((self.value(x, y) as f64 / max as f64) * 255.0 ) as u8;
                //println!("({}, {}) -> {}, value: {}", x, y, self.value(x, y), value);
                result.put_pixel(x as u32, y as u32, Rgb([value, 0, 0]));
            }
        }

        result
    }
}

fn main() -> Result<(), EscapeError> {
    let min = Complex::new(-4.0, -4.0);

    let mut grid = Grid::new(min, -0.01, 800, 800);
    //let mut grid2 = Grid::new(min, -0.01, 800, 800);
    //let mut grid3 = Grid::new(min, -0.01, 800, 800);

    let mut iterations = Vec::with_capacity(51);
    let total = 100000000;
    for _ in 0..total {
        let between = rand::distributions::Uniform::from(-4.0..4.0);
        let mut rng = rand::thread_rng();

        // Generate random c in range
        let c = Complex::new(between.sample(&mut rng), between.sample(&mut rng));
        let mut z = c;

        // iterate mandlebrot eq, keep iterations
        let mut i = 0;
        while z.norm_sqr() <= 4.0 && i < 51 {
            //println!("i: {}, z: {}, z norm {}", i, z, z.norm_sqr());
            iterations.push(z);
            z = z * z + c;
            i += 1;
        }

        if i < 50 {
            for c in &iterations {
                grid.increment(c);
            }
        }

        iterations.clear();
    }

    grid.to_image_buffer().save("Result.png")?;

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
