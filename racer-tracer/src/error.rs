use thiserror::Error;

#[derive(Clone, Error, Debug)]
pub enum TracerError {
    #[error("Unknown error: {message}")]
    Unknown { message: String, exit_code: i32 },

    #[error("Failed to create window: {0}")]
    FailedToCreateWindow(String),

    #[error("Failed to update window: {0}")]
    FailedToUpdateWindow(String),

    #[error("Resolution is not power of two.")]
    ResolutionIsNotPowerOfTwo(),

    #[error("Config Error ({0}): {1}")]
    Configuration(String, String),
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
        }
    }
}
