use thiserror::Error;

#[derive(Error, Debug)]
pub enum AhabError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Aha API error: {0}")]
    AhaApi(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Partial failure: {completed} of {total} epics created successfully")]
    PartialFailure { completed: usize, total: usize },

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Interactive prompt error: {0}")]
    DialoguerError(String),
}

impl From<toml::de::Error> for AhabError {
    fn from(err: toml::de::Error) -> Self {
        AhabError::Serialization(err.to_string())
    }
}

impl From<toml::ser::Error> for AhabError {
    fn from(err: toml::ser::Error) -> Self {
        AhabError::Serialization(err.to_string())
    }
}

impl From<serde_json::Error> for AhabError {
    fn from(err: serde_json::Error) -> Self {
        AhabError::Serialization(err.to_string())
    }
}

impl From<dialoguer::Error> for AhabError {
    fn from(err: dialoguer::Error) -> Self {
        AhabError::DialoguerError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AhabError>;
