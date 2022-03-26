use std::{collections::HashMap, fs, io::Read, path::PathBuf};

use crate::{card::CardsMap, error::*};

#[derive(Debug, Default)]
pub struct CachedCards {
    path: PathBuf,
    pub cards: CardsMap,
}

impl CachedCards {
    pub fn new(path: PathBuf) -> Result<Self> {
        let mut cache_buff = vec![];
        let mut cache_reader = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .map_err(|e| CardamomError::ReadCachedCardsError(path.clone(), e))?;
        cache_reader
            .read_to_end(&mut cache_buff)
            .map_err(|e| CardamomError::ReadCachedCardsError(path.clone(), e))?;

        let cards = if cache_buff.is_empty() {
            HashMap::default()
        } else {
            serde_json::from_slice(&cache_buff)
                .map_err(|e| CardamomError::ParseCachedCardsError(path.clone(), e))?
        };

        Ok(Self { path, cards })
    }

    pub fn save(&self) -> Result<()> {
        let cache_writer = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.path)
            .map_err(|e| CardamomError::ReadCachedCardsError(self.path.clone(), e))?;
        serde_json::to_writer(cache_writer, &self.cards)
            .map_err(|e| CardamomError::ParseCachedCardsError(self.path.clone(), e))
    }
}
