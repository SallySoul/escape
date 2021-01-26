use serde::{Deserialize, Serialize};

use crate::types::Complex;

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct ViewConfig {
    pub center: Complex,
    pub zoom: f64,
    pub width: usize,
    pub height: usize,
    width_i32: i32,
    height_i32: i32,
}

impl ViewConfig {
    pub fn new(center: Complex, zoom: f64, width: usize, height: usize) -> ViewConfig {
        ViewConfig {
            center,
            zoom,
            width,
            height,
            width_i32: width as i32,
            height_i32: height as i32,
        }
    }

    pub fn project(&self, c: &Complex) -> Option<(usize, usize)> {
        let x_fp = ((c.re - self.center.re) * self.zoom) + 0.5;
        let y_fp = ((c.im - self.center.im) * self.zoom) + 0.5;

        let x_signed = (x_fp * self.width as f64) as i32;
        let y_signed = (y_fp * self.height as f64) as i32;

        if x_signed >= 0 && y_signed >= 0 && x_signed < self.width_i32 && y_signed < self.height_i32
        {
            Some((x_signed as usize, y_signed as usize))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod view_config_tests {
    use super::*;

    #[test]
    fn test_project_1() {
        let config = ViewConfig::new(
            Complex::new(0.0, 0.0),
            1.0,
            500,
            400,
        );

        let pixel_eps = 0.00001;

        assert_eq!(config.project(&Complex::new(0.0, 0.0)), Some((250, 200)));
        assert_eq!(config.project(&Complex::new(-0.5, -0.5)), Some((0, 0)));
        assert_eq!(
            config.project(&Complex::new(-0.5, 0.5 - pixel_eps)),
            Some((0, 399))
        );
        assert_eq!(
            config.project(&Complex::new(0.5 - pixel_eps, -0.5)),
            Some((499, 0))
        );
        assert_eq!(
            config.project(&Complex::new(0.5 - pixel_eps, 0.5 - pixel_eps)),
            Some((499, 399))
        );

        assert_eq!(config.project(&Complex::new(100.0, 0.5)), None);
        assert_eq!(config.project(&Complex::new(-0.6, 0.8)), None);
        assert_eq!(config.project(&Complex::new(std::f64::NAN, 0.8)), None);
    }

    #[test]
    fn test_project_2() {
        let config = ViewConfig::new(
            Complex::new(-1.0, 2.0),
            2.0,
            500,
            400,
        );

        let pixel_eps = 0.00001;

        assert_eq!(config.project(&Complex::new(-1.0, 2.0)), Some((250, 200)));
        assert_eq!(config.project(&Complex::new(-1.25, 1.75)), Some((0, 0)));
        assert_eq!(
            config.project(&Complex::new(-1.25, 2.25 - pixel_eps)),
            Some((0, 399))
        );
        assert_eq!(
            config.project(&Complex::new(-0.75 - pixel_eps, 1.75)),
            Some((499, 0))
        );
        assert_eq!(
            config.project(&Complex::new(-0.75 - pixel_eps, 2.25 - pixel_eps)),
            Some((499, 399))
        );

        assert_eq!(config.project(&Complex::new(100.0, 0.5)), None);
        assert_eq!(config.project(&Complex::new(-0.6, 0.8)), None);
        assert_eq!(config.project(&Complex::new(std::f64::NAN, 0.8)), None);
    }
}
