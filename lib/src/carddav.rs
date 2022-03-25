use chrono::{DateTime, Local};
use quick_xml::de as xml;
use reqwest::{blocking::Client, Method};
use serde::Deserialize;

use crate::{card_parsers::date_parser, error::*};

// Multistatus structs

#[derive(Debug, Deserialize)]
pub struct Status {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Propstat<T> {
    pub prop: T,
    pub status: Option<Status>,
}

#[derive(Debug, Deserialize)]
pub struct Href {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub href: Href,
    pub propstat: Propstat<T>,
}

#[derive(Debug, Deserialize)]
pub struct Multistatus<T> {
    #[serde(rename = "response")]
    pub responses: Vec<Response<T>>,
}

// Other structs

#[derive(Debug, Deserialize)]
pub struct GetCtag {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct GetEtag {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct GetLastModified {
    #[serde(with = "date_parser", rename = "$value")]
    pub value: DateTime<Local>,
}

// Current user principal structs

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct CurrentUserPrincipalProp {
    pub current_user_principal: CurrentUserPrincipal,
}

#[derive(Debug, Deserialize)]
struct CurrentUserPrincipal {
    pub href: Href,
}

// Addressbook home set structs

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct AddressbookHomeSetProp {
    pub addressbook_home_set: AddressbookHomeSet,
}

#[derive(Debug, Deserialize)]
struct AddressbookHomeSet {
    pub href: Href,
}

// Addressbook structs

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct AddressbookProp {
    pub resourcetype: AddressbookResourceType,
}

#[derive(Debug, Deserialize)]
struct AddressbookResourceType {
    pub addressbook: Option<Addressbook>,
}

#[derive(Debug, Deserialize)]
struct Addressbook {}

// Address data structs

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AddressDataProp {
    pub address_data: AddressData,
    pub getetag: GetEtag,
    pub getlastmodified: GetLastModified,
}

#[derive(Debug, Deserialize)]
pub struct AddressData {
    #[serde(rename = "$value")]
    pub value: String,
}

// Ctag structs

#[derive(Debug, Deserialize)]
pub struct CtagProp {
    pub getctag: GetCtag,
}

// Methods

fn propfind() -> Result<Method> {
    Method::from_bytes(b"PROPFIND").map_err(|_| CardamomError::UnknownError)
}

pub fn report() -> Result<Method> {
    Method::from_bytes(b"REPORT").map_err(|_| CardamomError::UnknownError)
}

fn fetch_current_user_principal_url(host: &str, path: String, client: &Client) -> Result<String> {
    let res = client
        .request(propfind()?, format!("{}{}", host, path))
        .basic_auth("user", Some(""))
        .header("Content-Type", "application/xml; charset=utf-8")
        .header("Depth", "0")
        .body(
            r#"
            <D:propfind xmlns:D="DAV:">
                <D:prop>
                    <D:current-user-principal />
                </D:prop>
            </D:propfind>
            "#,
        )
        .send()
        .map_err(|_| CardamomError::UnknownError)?;
    let res = res.text().map_err(|_| CardamomError::UnknownError)?;
    println!("current user principal: {:?}", res);
    let res: Multistatus<CurrentUserPrincipalProp> =
        xml::from_str(&res).map_err(|_| CardamomError::UnknownError)?;
    let path = res
        .responses
        .first()
        .map(|res| {
            res.propstat
                .prop
                .current_user_principal
                .href
                .value
                .to_owned()
        })
        .unwrap_or(path);
    Ok(path)
}

fn fetch_addressbook_home_set_url(host: &str, path: String, client: &Client) -> Result<String> {
    let res = client
        .request(propfind()?, format!("{}{}", host, path))
        .basic_auth("user", Some(""))
        .header("Content-Type", "application/xml; charset=utf-8")
        .header("Depth", "0")
        .body(
            r#"
            <D:propfind xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:carddav">
                <D:prop>
                    <C:addressbook-home-set />
                </D:prop>
            </D:propfind>
            "#,
        )
        .send()
        .map_err(|_| CardamomError::UnknownError)?;
    let res = res.text().map_err(|_| CardamomError::UnknownError)?;
    println!("addressbook home set: {:?}", res);
    let res: Multistatus<AddressbookHomeSetProp> =
        xml::from_str(&res).map_err(|_| CardamomError::UnknownError)?;
    let path = res
        .responses
        .first()
        .map(|res| res.propstat.prop.addressbook_home_set.href.value.to_owned())
        .unwrap_or(path);
    Ok(path)
}

fn fetch_addressbook_url(host: &str, path: String, client: &Client) -> Result<String> {
    let res = client
        .request(propfind()?, format!("{}{}", host, path))
        .basic_auth("user", Some(""))
        .header("Content-Type", "application/xml; charset=utf-8")
        .header("Depth", "1")
        .body(
            r#"
            <D:propfind xmlns:D="DAV:">
              <D:prop>
                <D:resourcetype />
              </D:prop>
            </D:propfind>
            "#,
        )
        .send()
        .map_err(|_| CardamomError::UnknownError)?;
    let res = res.text().map_err(|_| CardamomError::UnknownError)?;
    let res: Multistatus<AddressbookProp> = xml::from_str(&res).unwrap();
    let path = res
        .responses
        .iter()
        .find(|res| {
            let valid_status = res
                .propstat
                .status
                .as_ref()
                .map(|s| s.value.ends_with("200 OK"))
                .unwrap_or(false);
            let has_addressbook = res
                .propstat
                .prop
                .resourcetype
                .addressbook
                .as_ref()
                .is_some();
            valid_status && has_addressbook
        })
        .map(|res| res.href.value.to_owned())
        .unwrap_or(path);
    Ok(path)
}

pub fn addressbook_path(host: &str, client: &Client) -> Result<String> {
    let path = String::from("/");
    let path = fetch_current_user_principal_url(host, path, client)?;
    let path = fetch_addressbook_home_set_url(host, path, client)?;
    let path = fetch_addressbook_url(host, path, client)?;
    Ok(path)
}
