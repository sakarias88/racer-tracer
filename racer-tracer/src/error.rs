use thiserror::Error;

#[derive(Clone, Error, Debug, PartialEq, Eq)]
pub enum TracerError {
    #[error("Unknown error: {message}")]
    Unknown { message: String, exit_code: i32 },

    #[error("Error: {0}")]
    Generic(String),

    #[error("Failed to create window: {0}")]
    FailedToCreateWindow(String),

    #[error("Failed to update window: {0}")]
    FailedToUpdateWindow(String),

    #[error("Resolution is not power of two.")]
    ResolutionIsNotPowerOfTwo(),

    #[error("Config Error ({0}): {1}")]
    Configuration(String, String),

    #[error("Argument parsing Error: {0}")]
    ArgumentParsingError(String),

    #[error("Unknown Material {0}.")]
    UnknownMaterial(String),

    #[error("No scene supplied.")]
    NoScene(),

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
}

impl From<TracerError> for i32 {
    fn from(tracer_error: TracerError) -> Self {
        match tracer_error {
            TracerError::Unknown {
                message: _,
                exit_code,
            } => exit_code,
            TracerError::FailedToCreateWindow(_) => 2,
            TracerError::FailedToUpdateWindow(_) => 3,
            TracerError::ResolutionIsNotPowerOfTwo() => 4,
            TracerError::Configuration(_, _) => 5,
            TracerError::UnknownMaterial(_) => 6,
            TracerError::NoScene() => 7,
            TracerError::FailedToAcquireLock(_) => 8,
            TracerError::ExitEvent => 9,
            TracerError::CancelEvent => 10,
            TracerError::Generic(_) => 11,
            TracerError::ImageSave(_) => 12,
            TracerError::SceneLoad(_) => 13,
            TracerError::ArgumentParsingError(_) => 14,
            TracerError::KeyError(_) => 15,
        }
    }
}
