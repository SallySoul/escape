use crate::cli_options::StlOptions;
use crate::config::{SampleConfig, StlConfig};
use crate::grid::Grid;
use crate::histogram_result::HistogramResult;
use crate::types::{EscapeResult, NormalizedGrid};

use std::io::{BufReader, BufWriter};
use tracing::info;

pub fn run_stl(stl_options: &StlOptions) -> EscapeResult {
    let logger_builder = tracing_subscriber::fmt()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_thread_ids(true)
        .with_max_level(&stl_options.verbosity);
    if stl_options.pretty_logging {
        logger_builder.pretty().init()
    } else {
        logger_builder.init();
    }
    info!("Starting stl operation");

    let mut config_reader = BufReader::new(std::fs::File::open(&stl_options.config)?);
    let stl_config: StlConfig = serde_json::from_reader(&mut config_reader)?;
    info!("Loaded stl config {}", &stl_options.config.display());

    let (sample_config, count_grids) = HistogramResult::from_file(&stl_options.histogram)?;
    stl_config.compatible(&sample_config)?;
    info!(
        "Loaded histogram result {}",
        &stl_options.histogram.display()
    );

    let normalized_grids: Vec<NormalizedGrid> = count_grids
        .iter()
        .map(|grid| grid.to_normalized_grid())
        .collect();
    info!("Grids have been normalized");

    let mesh = generate_height_mesh(&stl_config, &sample_config, &normalized_grids);
    info!("Mesh generated");

    let mut writer = BufWriter::new(std::fs::File::create(&stl_options.output)?);
    stl_io::write_stl(&mut writer, mesh.iter())?;
    info!("Result saved to {}", &stl_options.output.display());

    Ok(())
}

type Vec3 = nalgebra::Vector3<f32>;
fn to_vec(v: &stl_io::Vector<f32>) -> Vec3 {
    Vec3::new(v[0], v[1], v[2])
}

fn populate_normal(triangle: &mut stl_io::Triangle) {
    let o = to_vec(&triangle.vertices[0]);
    let a = to_vec(&triangle.vertices[1]);
    let b = to_vec(&triangle.vertices[2]);

    let normal = (a - o).cross(&(b - o));
    triangle.normal = stl_io::Vector::new([normal[0], normal[1], normal[2]]);
}

fn generate_height_mesh(
    stl_config: &StlConfig,
    sample_config: &SampleConfig,
    grids: &[NormalizedGrid],
) -> Vec<stl_io::Triangle> {
    let width = sample_config.view.width;
    let height = sample_config.view.height;
    let width_step = stl_config.width / width as f32;
    let height_step = stl_config.height / height as f32;
    let min_depth = stl_config.min_depth;
    let max_depth = stl_config.min_depth + stl_config.relief_height;

    // Generate height map vertices
    let mut height_vertices = Grid::from(width, height, [0.0; 3]);
    for x in 0..width {
        for y in 0..height {
            // Caclulate floating point  color
            let mut vertex = [x as f32 * width_step, y as f32 * height_step, min_depth];
            for (cutoff_index, contribution) in stl_config.contributions.iter().enumerate() {
                let power = stl_config.powers[cutoff_index];
                let value = grids[cutoff_index].value(x, y).powf(power);
                vertex[2] += contribution * (value as f32);
            }
            vertex[2] = vertex[2].clamp(min_depth, max_depth);
            height_vertices.set_value(vertex, x, y);
        }
    }

    // Generate height map triangles
    let mut result = Vec::new();
    for x in 1..width {
        for y in 1..height {
            let mut triangle_1 = stl_io::Triangle {
                normal: stl_io::Vector::new([0.0; 3]),
                vertices: [
                    stl_io::Vector::new(height_vertices.value(x - 1, y - 1)),
                    stl_io::Vector::new(height_vertices.value(x, y - 1)),
                    stl_io::Vector::new(height_vertices.value(x, y)),
                ],
            };
            populate_normal(&mut triangle_1);
            result.push(triangle_1);

            let mut triangle_2 = stl_io::Triangle {
                normal: stl_io::Vector::new([0.0; 3]),
                vertices: [
                    stl_io::Vector::new(height_vertices.value(x, y)),
                    stl_io::Vector::new(height_vertices.value(x - 1, y)),
                    stl_io::Vector::new(height_vertices.value(x - 1, y - 1)),
                ],
            };
            populate_normal(&mut triangle_2);
            result.push(triangle_2);
        }
    }

    result
}
