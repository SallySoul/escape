use crate::cli_options::DrawOptions;
use crate::error::EscapeResult;
use crate::histogram_result::HistogramResult;

pub fn run_draw(draw_options: &DrawOptions) -> EscapeResult {
    let _ = HistogramResult::from_file(&draw_options.config);
    Ok(())
}

/*
fn color_grids(config: &RenderConfig, grids: &[NormalizedGrid]) -> RgbImage {
let mut result = RgbImage::new(config.view.width as u32, config.view.height as u32);
for x in 0..config.view.width {
for y in 0..config.view.height {
let mut rgb_fp = [0.0, 0.0, 0.0];
for (cutoff_index, config) in config.cutoffs.iter().enumerate() {
for color in 0..3 {
rgb_fp[color] += grids[cutoff_index].value(x, y) * config.color[color];
                }
            }

            let rgb = {
                let mut rgb = [0, 0, 0];
                for color in 0..3 {
                    rgb[color] = (rgb_fp[color].clamp(0.0, 1.0) * 255.0) as u8;
                }
                rgb
            };

            result.put_pixel(x as u32, y as u32, Rgb(rgb));
        }
    }

    result
}
 */
