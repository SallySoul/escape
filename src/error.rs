#[derive(Debug, thiserror::Error)]
pub enum EscapeError {
    #[error("default error")]
    Default,

    #[error("Image conversion")]
    Image(#[from] image::ImageError),

    #[error("IO Issue")]
    Io(#[from] std::io::Error),

    #[error("JSON Error")]
    Json(#[from] serde_json::error::Error),
}

pub type EscapeResult = Result<(), EscapeError>;
