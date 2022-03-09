use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum CardamomError {
    #[error("unknown error")]
    UnknownError,
    #[error("cannot read card {0}: {1}")]
    ReadCardError(String, String),
    #[error("cannot delete card {0}: {1}")]
    DeleteCardError(String, String),
}

pub type Result<T> = std::result::Result<T, CardamomError>;
