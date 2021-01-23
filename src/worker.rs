type Complex = nalgebra::Complex<f64>;

struct ViewConfig {
    center: Complex,
    zoom: f64,
    width: i32,
    height: i32,
}

impl ViewConfig {
    fn project(&self, c: &Complex) -> Option<(usize, usize)> {
        let x_fp = ((c.re - self.center.re) * self.zoom) + 0.5;
        let y_fp = ((c.im - self.center.im) * self.zoom) + 0.5;

        let x_signed = (x_fp * self.width as f64) as i32;
        let y_signed = (y_fp * self.height as f64) as i32;

        if x_signed >= 0 && y_signed >= 0 && x_signed < self.width && y_signed < self.height {
            Some((x_signed as usize, y_signed as usize))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_1() {
        let config = ViewConfig {
            center: Complex::new(0.0, 0.0),
            zoom: 1.0,
            width: 500,
            height: 400,
        };

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
        let config = ViewConfig {
            center: Complex::new(-1.0, 2.0),
            zoom: 2.0,
            width: 500,
            height: 400,
        };

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

/*
struct WorkerState<'a> {
    config: &'a RenderConfig,
    grids: Vec<Arc<CountGrid>>,
}



async fn run_worker(state: Arc<WorkerState>) -> EscapeResult {
    Ok(())
}
*/
