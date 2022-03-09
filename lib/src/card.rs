use chrono::{DateTime, Local};
use serde::Deserialize;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    path::PathBuf,
};
use url::Url;

use crate::card_parsers::{date_parser, url_parser};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct Card {
    pub id: String,
    pub path: PathBuf,
    #[serde(with = "url_parser")]
    pub url: Url,
    pub etag: String,
    #[serde(with = "date_parser")]
    pub date: DateTime<Local>,
    pub content: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Cards(pub HashMap<String, Card>);

impl Deref for Cards {
    type Target = HashMap<String, Card>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Cards {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
