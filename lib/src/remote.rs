use chrono::DateTime;
use std::path::PathBuf;

use crate::{
    card::{Card, Cards},
    carddav::CardDavClient,
    error::*,
};

#[derive(Debug)]
pub struct RemoteCards {
    client: CardDavClient,
    cards: Cards,
}

impl RemoteCards {
    pub fn new(host: String, port: u16, login: String, passwd: String) -> Result<Self> {
        let mut cards = Cards::default();
        let client = CardDavClient::new(host, port, login, passwd)?;
        let address_data = client.fetch_address_data()?;

        for res in address_data.responses {
            let card = Card {
                id: PathBuf::from(&res.href)
                    .file_stem()
                    .ok_or_else(|| CardamomError::ParseAddressDataHrefError(res.href.clone()))?
                    .to_string_lossy()
                    .to_string(),
                etag: res
                    .propstat
                    .first()
                    .and_then(|propstat| propstat.prop.getetag.as_ref())
                    .unwrap_or(&String::default())
                    .to_owned(),
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
            cards.insert(card.id.to_owned(), card);
        }

        Ok(Self { client, cards })
    }
}
