pub type Complex = nalgebra::Complex<f64>;
pub type CountGrid = crate::grid::Grid<u64>;
pub type NormalizedGrid = crate::grid::Grid<f64>;

#[derive(Debug, thiserror::Error)]
pub enum EscapeError {
    #[error("Image conversion")]
    Image(#[from] image::ImageError),

    #[error("IO Issue")]
    Io(#[from] std::io::Error),

    #[error("JSON Error")]
    Json(#[from] serde_json::error::Error),

    #[error("Incompatible draw config")]
    IncompatibleDrawConfig(String),

    #[error("Incompatible stl config")]
    IncompatibleStlConfig(String),

    #[error("Incompatible Histograms")]
    IncompatibleHistograms,

    #[error("Couldn't part verbosity")]
    VerbosityParse(String),

    #[error("Tokio join error")]
    JoinError(#[from] tokio::task::JoinError),
}

pub type EscapeResult = Result<(), EscapeError>;

#[derive(Debug)]
pub enum Verbosity {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl std::default::Default for Verbosity {
    fn default() -> Self {
        Verbosity::Info
    }
}

impl std::str::FromStr for Verbosity {
    type Err = EscapeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "off" => Ok(Verbosity::Off),
            "error" => Ok(Verbosity::Error),
            "warn" => Ok(Verbosity::Warn),
            "info" => Ok(Verbosity::Info),
            "debug" => Ok(Verbosity::Debug),
            "trace" => Ok(Verbosity::Trace),
            m => Err(EscapeError::VerbosityParse(m.to_string())),
        }
    }
}

use tracing_subscriber::filter::LevelFilter;
impl From<&Verbosity> for LevelFilter {
    fn from(v: &Verbosity) -> LevelFilter {
        match v {
            Verbosity::Off => LevelFilter::OFF,
            Verbosity::Error => LevelFilter::ERROR,
            Verbosity::Warn => LevelFilter::WARN,
            Verbosity::Info => LevelFilter::INFO,
            Verbosity::Debug => LevelFilter::DEBUG,
            Verbosity::Trace => LevelFilter::TRACE,
        }
    }
}
