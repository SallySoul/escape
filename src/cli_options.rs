use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "escape")]
pub struct CliOptions {
    /// Path to the config file
    #[structopt(short, long, parse(from_os_str))]
    pub config: Option<PathBuf>,

    /// Path to the partial result file
    #[structopt(short, long, parse(from_os_str))]
    pub partial_result: Option<PathBuf>,

    /// Path to store partial result
    #[structopt(short, long, parse(from_os_str))]
    pub store_result: Option<PathBuf>,

    /// Path to store partial image
    #[structopt(short, long, parse(from_os_str))]
    pub output: PathBuf,

    /// The number of worker threads to spawn (more threads may be used)
    #[structopt(short, long)]
    pub workers: usize,

    /// Set a timeout for running workers
    #[structopt(short, long)]
    pub duration: Option<u64>,
}
