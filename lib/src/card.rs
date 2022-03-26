use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::card_parsers::date_parser;

pub type CardsMap = HashMap<String, Card>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    #[serde(with = "date_parser")]
    pub date: DateTime<Local>,
    pub content: String,
}

pub trait Cards {
    fn prev(&self) -> &CardsMap;
    fn next(&self) -> &CardsMap;
}
