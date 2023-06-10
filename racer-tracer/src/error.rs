use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
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

    #[error("Failed to recieve data: {0}")]
    RecieveError(String),

    #[error("Failed to recieve data: {0}")]
    SendError(String),
    #[error("Action protocol error! Unsupported action for: {0}")]
    ActionProtocolError(String),

    #[error("Failed to write data to bus \"{0}\": {1}")]
    BusWriteError(String, String),

    #[error("Failed to read data from bus \"{0}\": {1}")]
    BusReadError(String, String),

    #[error("Failed to update databus \"{0}\"")]
    BusUpdateError(String),

    #[error("Bus timeout error.")]
    BusTimeoutError(),

    #[error("No object with id: {0}.")]
    NoObjectWithId(usize),

    #[error("Failed to open image {0}: {1}")]
    FailedToOpenImage(String, String),

    #[error("Failed to parse \"{0}\" into a vector: {1}")]
    FailedToParse(String, String),
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
            TracerError::RecieveError(_) => 13,
            TracerError::SendError(_) => 14,
            TracerError::ActionProtocolError(_) => 15,
            TracerError::BusWriteError(_, _) => 16,
            TracerError::BusReadError(_, _) => 17,
            TracerError::BusUpdateError(_) => 18,
            TracerError::BusTimeoutError() => 19,
            TracerError::NoObjectWithId(_) => 20,
            TracerError::FailedToOpenImage(_, _) => 21,
            TracerError::FailedToParse(_, _) => 22,
        }
    }
}
