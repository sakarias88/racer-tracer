use thiserror::Error;

#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum TracerError {
    #[error("Failed to create window: {0}")]
    FailedToCreateWindow(String),

    #[error("Failed to update window: {0}")]
    FailedToUpdateWindow(String),

    #[error("Config Error ({0}): {1}")]
    Configuration(String, String),

    #[error("Argument parsing Error: {0}")]
    ArgumentParsingError(String),

    #[error("Unknown Material {0}.")]
    UnknownMaterial(String),

    #[error("Failed to acquire lock \"{0}\"")]
    FailedToAcquireLock(String),

    #[error("Exit event")]
    ExitEvent,

    #[error("Cancel event")]
    CancelEvent,

    #[error("Image save error: {0}")]
    ImageSave(String),

    #[error("Scene failed to load: {0}")]
    SceneLoad(String),

    #[error("Key callback failed: {0}")]
    KeyError(String),

    #[error("Failed to create log: {0}")]
    CreateLogError(String),
}

impl From<TracerError> for i32 {
    fn from(tracer_error: TracerError) -> Self {
        match tracer_error {
            TracerError::FailedToCreateWindow(_) => 1,
            TracerError::FailedToUpdateWindow(_) => 2,
            TracerError::Configuration(_, _) => 3,
            TracerError::UnknownMaterial(_) => 4,
            TracerError::FailedToAcquireLock(_) => 5,
            TracerError::ExitEvent => 6,
            TracerError::CancelEvent => 7,
            TracerError::ImageSave(_) => 8,
            TracerError::SceneLoad(_) => 9,
            TracerError::ArgumentParsingError(_) => 10,
            TracerError::KeyError(_) => 11,
            TracerError::CreateLogError(_) => 12,
        }
    }
}
