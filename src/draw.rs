use crate::cli_options::DrawOptions;
use crate::config::{DrawConfig, SampleConfig};
use crate::histogram_result::HistogramResult;
use crate::types::{EscapeError, EscapeResult, NormalizedGrid};

use std::io::BufReader;

pub fn run_draw(draw_options: &DrawOptions) -> EscapeResult {
    let mut config_reader = BufReader::new(std::fs::File::open(&draw_options.config)?);
    let draw_config: DrawConfig = serde_json::from_reader(&mut config_reader)?;

    let (sample_config, count_grids) = HistogramResult::from_file(&draw_options.histogram)?;

    let color_count = draw_config.colors.len();
    let cutoff_count = sample_config.cutoffs.len();
    if color_count != cutoff_count {
        let error_message = format!(
            "Mismatched configs, sample config had {} cutoffs, but draw config had {} colors",
            cutoff_count, color_count
        );
        return Err(EscapeError::BadDrawConfig(error_message));
    }

    let normalized_grids: Vec<NormalizedGrid> = count_grids
        .iter()
        .map(|grid| grid.to_normalized_grid())
        .collect();

    let image = color_grids(&draw_config, &sample_config, &normalized_grids);
    image.save(&draw_options.output)?;

    Ok(())
}

fn color_grids(
    draw_config: &DrawConfig,
    sample_config: &SampleConfig,
    grids: &[NormalizedGrid],
) -> image::RgbImage {
    let width = sample_config.view.width;
    let height = sample_config.view.height;
    let mut result = image::RgbImage::new(width as u32, height as u32);
    for x in 0..width {
        for y in 0..height {
            // Caclulate floating point  color
            let mut rgb_fp: [f64; 3] = [0.0, 0.0, 0.0];
            for (cutoff_index, rgb_color) in draw_config.colors.iter().enumerate() {
                let value = grids[cutoff_index].value(x, y);
                for i in 0..3 {
                    rgb_fp[i] += value * rgb_color[i];
                }
            }

            // Convert floating color to 8 bit
            let rgb = {
                let mut rgb = [0, 0, 0];
                for color in 0..3 {
                    rgb[color] = (rgb_fp[color].clamp(0.0, 1.0) * 255.0) as u8;
                }
                rgb
            };

            result.put_pixel(x as u32, y as u32, image::Rgb(rgb));
        }
    }

    result
}
