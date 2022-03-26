use chrono::DateTime;
use std::{collections::HashMap, path::PathBuf};

use crate::{
    cache::CachedCards,
    card::{Card, Cards, CardsMap},
    carddav::CardDavClient,
    error::*,
};

#[derive(Debug)]
pub struct RemoteCards {
    sync_dir: PathBuf,
    client: CardDavClient,
    pub prev: CardsMap,
    next: CardsMap,
}

impl Cards for RemoteCards {
    fn prev(&self) -> &CardsMap {
        &self.prev
    }

    fn next(&self) -> &CardsMap {
        &self.next
    }
}

impl RemoteCards {
    pub fn new(
        sync_dir: PathBuf,
        host: String,
        port: u16,
        login: String,
        passwd: String,
    ) -> Result<Self> {
        let prev = CachedCards::new(sync_dir.join(".remote"))?.cards;
        let mut next = HashMap::default();
        let client = CardDavClient::new(host, port, login, passwd)?;
        let address_data = client.fetch_address_data()?;

        for res in address_data.responses {
            let card = Card {
                id: PathBuf::from(&res.href)
                    .file_stem()
                    .ok_or_else(|| CardamomError::ParseAddressDataHrefError(res.href.clone()))?
                    .to_string_lossy()
                    .to_string(),
                date: res
                    .propstat
                    .first()
                    .and_then(|propstat| propstat.prop.getlastmodified.as_ref())
                    .and_then(|getlastmodified| {
                        DateTime::parse_from_rfc2822(&getlastmodified)
                            .map(|d| d.into())
                            .ok()
                    })
                    .ok_or_else(|| CardamomError::ParseAddressDataLastModifiedError)?,
                content: String::default(),
            };
            next.insert(card.id.to_owned(), card);
        }

        Ok(Self {
            sync_dir,
            client,
            prev,
            next,
        })
    }
}
