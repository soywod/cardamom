use std::{collections::HashMap, fs, ops::Deref, path::PathBuf};

use crate::{
    cache::CachedCards,
    card::{Card, Cards, CardsMap},
    error::*,
};

#[derive(Debug, Default)]
pub struct LocalCards {
    sync_dir: PathBuf,
    prev: CardsMap,
    next: CardsMap,
}

impl Cards for LocalCards {
    fn prev(&self) -> &CardsMap {
        &self.prev
    }

    fn next(&self) -> &CardsMap {
        &self.next
    }
}

impl LocalCards {
    pub fn new(sync_dir: PathBuf) -> Result<Self> {
        let prev = CachedCards::new(sync_dir.join(".local"))?.cards;
        let mut next = HashMap::default();

        let vcf_entries = fs::read_dir(&sync_dir)
            .map_err(|e| CardamomError::ReadLocalCardsDirError(sync_dir.clone(), e))?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().ends_with(".vcf"));
        for vcf in vcf_entries {
            if let Some(vcf_file_name) = PathBuf::from(vcf.path()).file_stem() {
                let card = Card {
                    id: vcf_file_name.to_string_lossy().to_string(),
                    date: vcf
                        .metadata()
                        .map_err(|e| CardamomError::GetVcfMetadataError(vcf.path().to_owned(), e))?
                        .modified()
                        .map_err(|e| CardamomError::GetVcfModifiedError(vcf.path().to_owned(), e))?
                        .into(),
                    content: String::default(),
                };
                next.insert(card.id.clone(), card);
            }
        }

        Ok(Self {
            sync_dir,
            prev,
            next,
        })
    }
}
