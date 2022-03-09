use std::path::PathBuf;

use chrono::{DateTime, Local};
use reqwest::blocking::Client;

use crate::{card::Card, card_repository::CardRepository, carddav::addressbook_path, error::*};

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
    fn create(&self, card: &mut Card) -> Result<()> {
        let res = self
            .client
            .put(format!("{}{}.vcf", self.addressbook_path, card.id))
            .basic_auth("user", Some(""))
            .header("Content-Type", "text/vcard; charset=utf-8")
            .body(card.content.clone())
            .send()
            .map_err(|_| CardamomError::UnknownError)?;
        let res_status = res.status();

        if !res_status.is_success() {
            let reason = res.text().unwrap_or(res_status.to_string());
            panic!("{}", reason);
            // return Err(anyhow!(reason).context("cannot create card"));
        }

        card.etag = res
            .headers()
            .get("etag")
            .and_then(|h| h.to_str().ok())
            .unwrap_or(&card.etag)
            .to_string();

        Ok(())
    }

    fn read(&self, id: &str) -> Result<Card> {
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
            path: PathBuf::new(),
            url: "http://localhost/".parse().unwrap(),
            content,
        })
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
