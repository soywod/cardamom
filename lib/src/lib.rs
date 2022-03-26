pub mod card;
pub mod card_parsers;
pub mod card_repository;
pub mod carddav;
pub mod error;
pub mod remote_card_repository;

pub mod cache;

use std::{collections::HashSet, fs, path::PathBuf};

use crate::{card::*, error::*};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hunk {
    Local(HunkKind),
    Cache(HunkKind),
    Remote(HunkKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HunkKind {
    Add(Card),
    Set(Card),
    Del(String),
}

pub type Patch = Vec<Hunk>;

pub fn sync(path: PathBuf) -> Result<()> {
    let cache_reader = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
        .map_err(|_| CardamomError::UnknownError)?;
    let cache = serde_json::from_reader(cache_reader).map_err(|_| CardamomError::UnknownError)?;
    let local = fs::read_dir(&path)
        .map_err(|_| CardamomError::UnknownError)?
        .filter_map(|entry| match entry {
            Err(_) => None,
            Ok(entry) => {
                let is_entry_vcf = entry
                    .path()
                    .extension()
                    .map(|ext| ext == "vcf")
                    .unwrap_or_default();
                if is_entry_vcf {
                    Some(entry)
                } else {
                    None
                }
            }
        })
        .fold(Cards::default(), |mut cards, entry| {
            match PathBuf::from(entry.path()).file_stem() {
                None => cards,
                Some(name) => {
                    let card = Card {
                        id: name.to_string_lossy().to_string(),
                        etag: String::default(),
                        date: entry.metadata().unwrap().modified().unwrap().into(),
                        content: String::default(),
                    };
                    cards.insert(card.id.clone(), card);
                    cards
                }
            }
        });
    Ok(())
}

pub fn build_local_cards(dir: PathBuf) -> Result<Cards> {
    Ok(fs::read_dir(dir)
        .map_err(|_| CardamomError::UnknownError)?
        .filter_map(|entry| match entry {
            Err(_) => None,
            Ok(entry) => {
                let is_entry_vcf = entry
                    .path()
                    .extension()
                    .map(|ext| ext == "vcf")
                    .unwrap_or_default();
                if is_entry_vcf {
                    Some(entry)
                } else {
                    None
                }
            }
        })
        .fold(Cards::default(), |mut cards, entry| {
            match PathBuf::from(entry.path()).file_stem() {
                None => cards,
                Some(name) => {
                    let card = Card {
                        id: name.to_string_lossy().to_string(),
                        etag: String::default(),
                        date: entry.metadata().unwrap().modified().unwrap().into(),
                        content: String::default(),
                    };
                    cards.insert(card.id.clone(), card);
                    cards
                }
            }
        }))
}

pub fn build_patch(local: Cards, cache: Cards, remote: Cards) -> Patch {
    let mut ids = HashSet::new();
    ids.extend(local.iter().map(|(id, _)| id.as_str()));
    ids.extend(cache.iter().map(|(id, _)| id.as_str()));
    ids.extend(remote.iter().map(|(id, _)| id.as_str()));

    let mut patch = Vec::new();

    for id in ids {
        // id present only in local cards
        if local.contains_key(id) && !cache.contains_key(id) && !remote.contains_key(id) {
            // add card to remote and cached cards
            let card = local.get(id).unwrap();
            patch.push(Hunk::Remote(HunkKind::Add(card.clone())));
            patch.push(Hunk::Cache(HunkKind::Add(card.clone())));
        }

        // id present only in cached cards
        if !local.contains_key(id) && cache.contains_key(id) && !remote.contains_key(id) {
            // nothing to do, it means both local and remote card have
            // been removed
            patch.push(Hunk::Cache(HunkKind::Del(id.to_owned())));
        }

        // id present only in remote cards
        if !local.contains_key(id) && !cache.contains_key(id) && remote.contains_key(id) {
            // add card to local and cached cards
            let card = remote.get(id).unwrap();
            patch.push(Hunk::Local(HunkKind::Add(card.clone())));
            patch.push(Hunk::Cache(HunkKind::Add(card.clone())));
        }

        // id present in local and cached cards
        if local.contains_key(id) && cache.contains_key(id) && !remote.contains_key(id) {
            // remove card from local and cached cards
            patch.push(Hunk::Local(HunkKind::Del(id.to_owned())));
            patch.push(Hunk::Cache(HunkKind::Del(id.to_owned())));
        }

        // id present in remote and cached cards
        if !local.contains_key(id) && cache.contains_key(id) && remote.contains_key(id) {
            // remove card from remote and cached cards
            patch.push(Hunk::Remote(HunkKind::Del(id.to_owned())));
            patch.push(Hunk::Cache(HunkKind::Del(id.to_owned())));
        }

        // id present in remote and local cards
        if local.contains_key(id) && !cache.contains_key(id) && remote.contains_key(id) {
            // should never happen, this means that the same card has
            // been added simultaneously locally and remotely
        }

        // id present everywhere
        if local.contains_key(id) && cache.contains_key(id) && remote.contains_key(id) {
            let lcard = local.get(id).unwrap();
            let ccard = cache.get(id).unwrap();
            let rcard = remote.get(id).unwrap();

            // etags are all the same
            if lcard.etag == ccard.etag && ccard.etag == rcard.etag {
                // nothing to do, it means all is up to date
            }

            // local etag is different
            if lcard.etag != ccard.etag && ccard.etag == rcard.etag {
                // update remote and cached card
                patch.push(Hunk::Remote(HunkKind::Set(lcard.clone())));
                patch.push(Hunk::Cache(HunkKind::Set(lcard.clone())));
            }

            // remote etag is different
            if lcard.etag == ccard.etag && ccard.etag != rcard.etag {
                // update local and cached card
                patch.push(Hunk::Local(HunkKind::Set(rcard.clone())));
                patch.push(Hunk::Cache(HunkKind::Set(rcard.clone())));
            }

            // local and remote etags are different
            if lcard.etag != ccard.etag && ccard.etag != rcard.etag {
                // update the most recent
                if lcard.date > rcard.date {
                    patch.push(Hunk::Remote(HunkKind::Set(lcard.clone())));
                    patch.push(Hunk::Cache(HunkKind::Set(lcard.clone())));
                } else {
                    patch.push(Hunk::Local(HunkKind::Set(rcard.clone())));
                    patch.push(Hunk::Cache(HunkKind::Set(rcard.clone())));
                }
            }
        }
    }

    patch
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Local};
    use std::{collections::HashMap, iter::FromIterator};

    use super::*;

    macro_rules! date {
        ($date: literal) => {
            DateTime::parse_from_rfc3339(&format!("{}T00:00:00+00:00", $date))
                .unwrap()
                .with_timezone(&Local)
        };
    }

    macro_rules! card {
        ($id: literal) => {
            Card {
                id: format!("{}", $id),
                etag: format!("{}", $id),
                date: date!("2022-01-02"),
                content: String::new(),
            }
        };
    }

    macro_rules! card_entry {
        ($id: literal) => {
            (
                format!("{}", $id),
                Card {
                    id: format!("{}", $id),
                    etag: format!("{}", $id),
                    date: date!("2022-01-02"),
                    content: String::new(),
                },
            )
        };
    }

    #[test]
    fn test_build_patch() {
        let local = Cards(HashMap::from_iter([
            card_entry!("everywhere-same"),
            card_entry!("local-only"),
            card_entry!("local-and-cache"),
        ]));
        let cache = Cards(HashMap::from_iter([
            card_entry!("everywhere-same"),
            card_entry!("cache-only"),
            card_entry!("local-and-cache"),
            card_entry!("remote-and-cache"),
        ]));
        let remote = Cards(HashMap::from_iter([
            card_entry!("everywhere-same"),
            card_entry!("remote-only"),
            card_entry!("remote-and-cache"),
        ]));

        let patch = build_patch(local, cache, remote);

        assert!(patch.contains(&Hunk::Remote(HunkKind::Add(card!("local-only")))));
        assert!(patch.contains(&Hunk::Cache(HunkKind::Add(card!("local-only")))));
        assert!(patch.contains(&Hunk::Cache(HunkKind::Del("cache-only".into()))));
        assert!(patch.contains(&Hunk::Local(HunkKind::Add(card!("remote-only")))));
        assert!(patch.contains(&Hunk::Cache(HunkKind::Add(card!("remote-only")))));
        assert!(patch.contains(&Hunk::Local(HunkKind::Del("local-and-cache".into()))));
        assert!(patch.contains(&Hunk::Cache(HunkKind::Del("local-and-cache".into()))));
        assert!(patch.contains(&Hunk::Remote(HunkKind::Del("remote-and-cache".into()))));
        assert!(patch.contains(&Hunk::Cache(HunkKind::Del("remote-and-cache".into()))));
    }
}
