use crate::types::Verbosity;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "escape")]
pub enum CliOptions {
    Sample(SampleOptions),
    Draw(DrawOptions),
    Merge(MergeOptions),
}

#[derive(StructOpt, Debug)]
pub struct SampleOptions {
    /// Path to the sample config file
    #[structopt(short, long, parse(from_os_str))]
    pub config: PathBuf,

    /// The number of worker threads to spawn (more threads may be used)
    #[structopt(short, long, default_value = "1")]
    pub workers: usize,

    /// Set a timeout for running workers
    #[structopt(short, long)]
    pub duration: Option<u64>,

    /// Path to store partial image
    #[structopt(short, long, parse(from_os_str))]
    pub output: PathBuf,

    /// Path to store partial image
    #[structopt(short, long, default_value = "info")]
    pub verbosity: Verbosity,

    /// Use pretty logging
    #[structopt(short, long)]
    pub pretty_logging: bool,
}

#[derive(StructOpt, Debug)]
pub struct DrawOptions {
    /// Path to the draw config file
    #[structopt(short, long, parse(from_os_str))]
    pub config: PathBuf,

    /// Path to sampling result
    #[structopt(short, long, parse(from_os_str))]
    pub histogram: PathBuf,

    /// Path to store image output
    /// TODO: Find what acceptable image types are
    #[structopt(short, long, parse(from_os_str))]
    pub output: PathBuf,
}

#[derive(StructOpt, Debug)]
pub struct MergeOptions {
    /// Paths to histogram results
    pub histograms: Vec<PathBuf>,

    /// Path to store merged histogram output
    /// TODO: Find what acceptable image types are
    #[structopt(short, long, parse(from_os_str))]
    pub output: PathBuf,

    /// The number of worker threads to spawn (more threads may be used)
    #[structopt(short, long, default_value = "1")]
    pub workers: usize,

    /// Path to store partial image
    #[structopt(short, long, default_value = "info")]
    pub verbosity: Verbosity,

    /// Use pretty logging
    #[structopt(short, long)]
    pub pretty_logging: bool,
}
