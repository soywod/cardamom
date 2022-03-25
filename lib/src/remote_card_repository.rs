use std::path::PathBuf;

use chrono::{DateTime, Local};
use quick_xml::de as xml;
use reqwest::{blocking::Client, header::CONTENT_TYPE};

use crate::{
    card::{Card, Cards},
    card_repository::CardRepository,
    carddav::{addressbook_path, report, AddressDataProp, Multistatus},
    error::*,
};

#[derive(Debug)]
pub struct RemoteCardRepository<'a> {
    pub addressbook_path: String,
    pub client: &'a Client,
}

impl<'a> RemoteCardRepository<'a> {
    pub fn new(host: &str, client: &'a Client) -> Result<Self> {
        Ok(Self {
            addressbook_path: format!("{}{}", host, addressbook_path(host, client)?),
            client,
        })
    }
}

impl<'a> CardRepository for RemoteCardRepository<'a> {
    fn insert(&self, card: &mut Card) -> Result<()> {
        let res = self
            .client
            .put(format!("{}{}.vcf", self.addressbook_path, card.id))
            .basic_auth("user", Some(""))
            .header(CONTENT_TYPE, "text/vcard; charset=utf-8")
            .body(card.content.clone())
            .send()
            .map_err(|_| CardamomError::UnknownError)?;
        let res_status = res.status();

        if !res_status.is_success() {
            let reason = res.text().unwrap_or(res_status.to_string());
            panic!("{}", reason);
            // return Err(anyhow!(reason).context("cannot create card"));
        }

        println!("res: {:?}", res);
        card.etag = res
            .headers()
            .get("etag")
            .and_then(|h| h.to_str().ok())
            .unwrap_or(&card.etag)
            .to_string();

        Ok(())
    }

    fn select(&self, id: &str) -> Result<Card> {
        let res = self
            .client
            .get(format!("{}{}.vcf", self.addressbook_path, id))
            .basic_auth("user", Some(""))
            .header("Depth", "1")
            .send()
            .map_err(|_| CardamomError::UnknownError)?;
        let res_status = res.status();

        if !res_status.is_success() {
            let reason = res.text().unwrap_or(res_status.to_string());
            return Err(CardamomError::ReadCardError(id.to_owned(), reason));
        }

        let date = res
            .headers()
            .get("last-modified")
            .ok_or_else(|| CardamomError::UnknownError)?;
        let date = date.to_str().map_err(|_| CardamomError::UnknownError)?;
        let date = DateTime::parse_from_rfc2822(date)
            .map_err(|_| CardamomError::UnknownError)?
            .with_timezone(&Local);
        let etag = res
            .headers()
            .get("etag")
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default()
            .to_string();
        let content = res.text().map_err(|_| CardamomError::UnknownError)?;

        Ok(Card {
            id: id.to_owned(),
            etag,
            date,
            content,
        })
    }

    fn select_all(&self) -> Result<Cards> {
        println!("self.addressbook_path: {:?}", self.addressbook_path);
        let res = self
            .client
            .request(report()?, self.addressbook_path.clone())
            .basic_auth("user", Some(""))
            .header("Content-Type", "application/xml; charset=utf-8")
            .header("Depth", "0")
            .body(
                r#"
                <C:addressbook-query xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:carddav">
                    <D:prop>
                        <D:getetag />
                        <D:getlastmodified />
                        <C:address-data />
                    </D:prop>
                </C:addressbook-query>
            "#,
            )
            .send()
            .map_err(CardamomError::FetchRemoteCardsError)?;
        let res = res.text().map_err(CardamomError::FetchRemoteCardsError)?;
        println!("select all: {:?}", res);
        let res: Multistatus<AddressDataProp> =
            xml::from_str(&res).map_err(|_| CardamomError::UnknownError)?;
        println!("select all: {:?}", res);

        let cards = res
            .responses
            .iter()
            .fold(Cards::default(), |mut cards, res| {
                let card = Card {
                    id: PathBuf::from(&res.href.value)
                        .file_stem()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                    etag: res.propstat.prop.getetag.value.to_owned(),
                    date: res.propstat.prop.getlastmodified.value,
                    content: String::default(),
                };
                cards.insert(card.id.to_owned(), card);
                cards
            });

        Ok(cards)
    }

    fn update(&self, card: &mut Card) -> Result<()> {
        let mut req = self
            .client
            .put(format!("{}{}.vcf", self.addressbook_path, card.id))
            .basic_auth("user", Some(""))
            .header("Content-Type", "text/vcard; charset=utf-8")
            .body(card.content.clone());

        if !card.etag.is_empty() {
            req = req.header("If-Match", &card.etag);
        }

        let res = req.send().map_err(|_| CardamomError::UnknownError)?;
        let res_status = res.status();

        if !res_status.is_success() {
            let reason = res.text().unwrap_or(res_status.to_string());
            panic!("{}", reason);
            // return Err(anyhow!(reason).context(format!(r#"cannot update card "{}""#, card.id)));
        }

        card.etag = res
            .headers()
            .get("etag")
            .and_then(|h| h.to_str().ok())
            .unwrap_or(&card.etag)
            .to_string();

        Ok(())
    }

    fn delete(&self, card: &Card) -> Result<()> {
        let mut req = self
            .client
            .delete(format!("{}{}.vcf", self.addressbook_path, card.id))
            .basic_auth("user", Some(""));

        if !card.etag.is_empty() {
            req = req.header("If-Match", &card.etag);
        }

        let res = req.send().map_err(|_| CardamomError::UnknownError)?;
        let res_status = res.status();

        if !res_status.is_success() {
            let reason = res.text().unwrap_or(res_status.to_string());
            return Err(CardamomError::DeleteCardError(card.id.to_owned(), reason));
        }

        Ok(())
    }
}
