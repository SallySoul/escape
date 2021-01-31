#[derive(Debug, thiserror::Error)]
pub enum EscapeError {
    #[error("Image conversion")]
    Image(#[from] image::ImageError),

    #[error("IO Issue")]
    Io(#[from] std::io::Error),

    #[error("JSON Error")]
    Json(#[from] serde_json::error::Error),

    #[error("Bad draw config")]
    BadDrawConfig(String),

    #[error("Couldn't part verbosity")]
    VerbosityParse(String),
}

pub type EscapeResult = Result<(), EscapeError>;
