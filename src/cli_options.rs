use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "escape")]
pub enum CliOptions {
    Sample(SampleOptions),
    Draw(DrawOptions),
}

#[derive(StructOpt, Debug)]
pub struct SampleOptions {
    /// Path to the sample config file
    #[structopt(short, long, parse(from_os_str))]
    pub config: PathBuf,

    /// The number of worker threads to spawn (more threads may be used)
    #[structopt(short, long)]
    pub workers: usize,

    /// Set a timeout for running workers
    #[structopt(short, long)]
    pub duration: Option<u64>,

    /// Path to store partial image
    #[structopt(short, long, parse(from_os_str))]
    pub output: PathBuf,
}

#[derive(StructOpt, Debug)]
pub struct DrawOptions {
    /// Path to the draw config file
    #[structopt(short, long, parse(from_os_str))]
    pub config: Option<PathBuf>,

    /// Path to store partial image
    #[structopt(short, long, parse(from_os_str))]
    pub output: PathBuf,
}
