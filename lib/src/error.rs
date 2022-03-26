use std::{io, path::PathBuf, result};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CardamomError {
    #[error("unknown error")]
    UnknownError,
    #[error("cannot read card {0}: {1}")]
    ReadCardError(String, String),
    #[error("cannot delete card {0}: {1}")]
    DeleteCardError(String, String),
    #[error("cannot fetch remote cards: {0}")]
    FetchRemoteCardsError(reqwest::Error),

    #[error("cannot read cached cards at {0:?}: {1}")]
    ReadCachedCardsError(PathBuf, io::Error),
    #[error("cannot parse cached cards at {0:?}: {1}")]
    ParseCachedCardsError(PathBuf, serde_json::Error),
}

pub type Result<T> = result::Result<T, CardamomError>;
