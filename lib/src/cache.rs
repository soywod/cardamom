use std::{fs, io::Read, path::PathBuf};

use crate::{card::Cards, error::*};

#[derive(Debug, Default)]
pub struct CachedCards {
    file_path: PathBuf,
    cards: Cards,
}

impl CachedCards {
    pub fn new(file_path: PathBuf) -> Result<Self> {
        let mut cache_buff = vec![];
        let mut cache_reader = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)
            .map_err(|e| CardamomError::ReadCachedCardsError(file_path.clone(), e))?;
        cache_reader
            .read_to_end(&mut cache_buff)
            .map_err(|e| CardamomError::ReadCachedCardsError(file_path.clone(), e))?;

        let cards = if cache_buff.is_empty() {
            Cards::default()
        } else {
            serde_json::from_slice(&cache_buff)
                .map_err(|e| CardamomError::ParseCachedCardsError(file_path.clone(), e))?
        };

        Ok(Self { file_path, cards })
    }

    pub fn save(&self) -> Result<()> {
        let cache_writer = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.file_path)
            .map_err(|e| CardamomError::ReadCachedCardsError(self.file_path.clone(), e))?;
        serde_json::to_writer(cache_writer, &self.cards)
            .map_err(|e| CardamomError::ParseCachedCardsError(self.file_path.clone(), e))
    }
}
