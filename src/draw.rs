use crate::cli_options::DrawOptions;
use crate::config::{DrawConfig, SampleConfig};
use crate::histogram_result::HistogramResult;
use crate::types::{EscapeResult, NormalizedGrid};

use std::io::BufReader;
use tracing::info;

pub fn run_draw(draw_options: &DrawOptions) -> EscapeResult {
    let logger_builder = tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_thread_ids(true)
        .with_max_level(&draw_options.verbosity);
    if draw_options.pretty_logging {
        logger_builder.pretty().init()
    } else {
        logger_builder.init();
    }
    info!("Starting draw operation");

    let mut config_reader = BufReader::new(std::fs::File::open(&draw_options.config)?);
    let draw_config: DrawConfig = serde_json::from_reader(&mut config_reader)?;
    info!("Loaded draw config {}", &draw_options.config.display());

    let (sample_config, count_grids) = HistogramResult::from_file(&draw_options.histogram)?;
    draw_config.compatible(&sample_config)?;
    info!(
        "Loaded histogram result {}",
        &draw_options.histogram.display()
    );

    let normalized_grids: Vec<NormalizedGrid> = count_grids
        .iter()
        .map(|grid| grid.to_normalized_grid())
        .collect();
    info!("Grids have been normalized");

    let image = color_grids(&draw_config, &sample_config, &normalized_grids);
    info!("Image generated");

    image.save(&draw_options.output)?;
    info!("Result saved to {}", &draw_options.output.display());

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
            let mut rgb_fp: [f64; 3] = draw_config.background_color;
            for (cutoff_index, rgb_color) in draw_config.colors.iter().enumerate() {
                let power = draw_config.powers[cutoff_index];
                let value = grids[cutoff_index].value(x, y).powf(power);
                for i in 0..3 {
                    rgb_fp[i] += value * (rgb_color[i] as f64 / 255.0);
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
