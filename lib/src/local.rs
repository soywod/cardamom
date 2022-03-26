use std::{fs, path::PathBuf};

use crate::{
    card::{Card, Cards},
    error::*,
};

#[derive(Debug, Default)]
pub struct LocalCards {
    dir: PathBuf,
    cards: Cards,
}

impl LocalCards {
    pub fn new(dir: PathBuf) -> Result<Self> {
        let mut cards = Cards::default();
        let vcf_entries = fs::read_dir(&dir)
            .map_err(|e| CardamomError::ReadLocalCardsDirError(dir.clone(), e))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().ends_with(".vcf"));

        for vcf in vcf_entries {
            if let Some(vcf_file_name) = PathBuf::from(vcf.path()).file_stem() {
                let card = Card {
                    id: vcf_file_name.to_string_lossy().to_string(),
                    etag: String::default(),
                    date: vcf
                        .metadata()
                        .map_err(|e| CardamomError::GetVcfMetadataError(vcf.path().to_owned(), e))?
                        .modified()
                        .map_err(|e| CardamomError::GetVcfModifiedError(vcf.path().to_owned(), e))?
                        .into(),
                    content: String::default(),
                };
                cards.insert(card.id.clone(), card);
            }
        }

        Ok(Self { dir, cards })
    }
}
