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

    #[error("cannot parse carddav url {0}: {1}")]
    ParseCardDavUrlError(String, url::ParseError),

    #[error("cannot read cached cards at {0:?}: {1}")]
    ReadCachedCardsError(PathBuf, io::Error),
    #[error("cannot parse cached cards at {0:?}: {1}")]
    ParseCachedCardsError(PathBuf, serde_json::Error),

    #[error("cannot read local cards directory at {0:?}: {1}")]
    ReadLocalCardsDirError(PathBuf, io::Error),
    #[error("cannot get local card metadata at {0:?}: {1}")]
    GetVcfMetadataError(PathBuf, io::Error),
    #[error("cannot get local card modified time at {0:?}: {1}")]
    GetVcfModifiedError(PathBuf, io::Error),

    #[error("cannot parse missing last modified date from address data")]
    ParseAddressDataLastModifiedError,
    #[error("cannot parse address data href {0:?}")]
    ParseAddressDataHrefError(String),
    #[error("cannot fetch current user principal url: {0}")]
    FetchCurrentUserPrincipalUrlError(reqwest::Error),
    #[error("cannot parse current user principal url: {0}")]
    ParseCurrentUserPrincipalUrlError(quick_xml::de::DeError),
    #[error("cannot fetch addressbook home set url: {0}")]
    FetchAddressbookHomeSetUrlError(reqwest::Error),
    #[error("cannot parse addressbook home set url: {0}")]
    ParseAddressbookHomeSetUrlError(quick_xml::de::DeError),
    #[error("cannot fetch addressbook url: {0}")]
    FetchAddressbookUrlError(reqwest::Error),
    #[error("cannot parse addressbook url: {0}")]
    ParseAddressbookUrlError(quick_xml::de::DeError),
    #[error("cannot fetch remote cards: {0}")]
    FetchAddressDataError(reqwest::Error),
    #[error("cannot parse remote cards: {0}")]
    ParseAddressDataError(quick_xml::de::DeError),
}

pub type Result<T> = result::Result<T, CardamomError>;
