//! CardDAV module
//!
//! This module contains everything to interact with CardDAV servers.

use quick_xml::de as xml;
use reqwest::{blocking::Client, Method};
use serde::Deserialize;

use crate::error::*;

/// Represents the CardDAV response wrapper. The CardDAV response
/// wraps multiple `response` in a single `multistatus`.
///
/// ```xml
/// <multistatus xmlns="DAV:">
///     <response>
///         ...
///     </response>
///     <response>
///         ...
///     </response>
///     ...
/// </multistatus>
/// ```
#[derive(Debug, Deserialize)]
pub struct Multistatus<T> {
    #[serde(rename = "response", default)]
    pub responses: Vec<Response<T>>,
}

/// Represents the CardDAV response. The CardDAV response contains a
/// `href` and many `propstat`.
///
/// ```xml
/// <response>
///     <href>/path</href>
///     <propstat>
///         ...
///     </propstat>
///     <propstat>
///         ...
///     </propstat>
///     ...
/// <response>
/// ```
#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub href: String,
    #[serde(default)]
    pub propstat: Vec<Propstat<T>>,
}

/// Represents the properties wrapper associated to the CardDAV
/// response. The propstat contains a property `prop` and sometimes a
/// `status` code.
///
/// ```xml
/// <propstat>
///     <prop>
///         ...
///     </prop>
///     <status>HTTP/1.1 200 OK</status>
/// </propstat>
/// ```
#[derive(Debug, Deserialize)]
pub struct Propstat<T> {
    pub prop: T,
    pub status: Option<String>,
}

// Current user principal structs

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct CurrentUserPrincipalProp {
    pub current_user_principal: CurrentUserPrincipal,
}

#[derive(Debug, Default, Deserialize)]
struct CurrentUserPrincipal {
    pub href: String,
}

// Addressbook home set structs

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct AddressbookHomeSetProp {
    pub addressbook_home_set: AddressbookHomeSet,
}

#[derive(Debug, Default, Deserialize)]
struct AddressbookHomeSet {
    pub href: String,
}

// Addressbook structs

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct AddressbookProp {
    pub resourcetype: AddressbookResourceType,
}

#[derive(Debug, Default, Deserialize)]
struct AddressbookResourceType {
    pub addressbook: Option<Addressbook>,
}

#[derive(Debug, Default, Deserialize)]
struct Addressbook {}

// Address data structs

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AddressDataProp {
    pub address_data: Option<String>,
    pub getetag: Option<String>,
    pub getlastmodified: Option<String>,
}

// Ctag structs

#[derive(Debug, Deserialize)]
pub struct CtagProp {
    pub getctag: String,
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
            <propfind xmlns="DAV:">
                <prop>
                    <current-user-principal />
                </prop>
            </propfind>
            "#,
        )
        .send()
        .map_err(|_| CardamomError::UnknownError)?;
    let res = res.text().map_err(|_| CardamomError::UnknownError)?;
    let res: Multistatus<CurrentUserPrincipalProp> =
        xml::from_str(&res).map_err(|_| CardamomError::UnknownError)?;
    let path = res
        .responses
        .first()
        .and_then(|res| res.propstat.first())
        .map(|propstat| propstat.prop.current_user_principal.href.to_owned())
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
            <propfind xmlns="DAV:" xmlns:c="urn:ietf:params:xml:ns:carddav">
                <prop>
                    <c:addressbook-home-set />
                </prop>
            </propfind>
            "#,
        )
        .send()
        .map_err(|_| CardamomError::UnknownError)?;
    let res = res.text().map_err(|_| CardamomError::UnknownError)?;
    let res: Multistatus<AddressbookHomeSetProp> =
        xml::from_str(&res).map_err(|_| CardamomError::UnknownError)?;
    let path = res
        .responses
        .first()
        .and_then(|res| res.propstat.first())
        .map(|propstat| propstat.prop.addressbook_home_set.href.to_owned())
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
            <propfind xmlns="DAV:">
                <prop>
                    <resourcetype />
                </prop>
            </propfind>
            "#,
        )
        .send()
        .map_err(|_| CardamomError::UnknownError)?;
    let res = res.text().map_err(|_| CardamomError::UnknownError)?;
    let res: Multistatus<AddressbookProp> = xml::from_str(&res).unwrap();
    let path = res
        .responses
        .iter()
        .find_map(|res| {
            res.propstat
                .iter()
                .find(|propstat| {
                    let valid_status = propstat
                        .status
                        .as_ref()
                        .map(|s| s.ends_with("200 OK"))
                        .unwrap_or(false);
                    let has_addressbook = propstat.prop.resourcetype.addressbook.as_ref().is_some();
                    valid_status && has_addressbook
                })
                .map(|_| res.href.to_owned())
        })
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

#[cfg(test)]
mod tests {
    use quick_xml::de as xml;

    use super::*;

    #[test]
    fn empty_response() {
        let res: Multistatus<String> = xml::from_str(r#"<multistatus xmlns="DAV:" />"#).unwrap();
        assert_eq!(0, res.responses.len());
    }

    #[test]
    fn single_propstat() {
        let res: Multistatus<String> = xml::from_str(
            r#"
            <multistatus xmlns="DAV:">
	        <response>
		    <href>/path</href>
                    <propstat>
			<prop>data</prop>
			<status>HTTP/1.1 200 OK</status>
		    </propstat>
	        </response>
            </multistatus>
            "#,
        )
        .unwrap();

        assert_eq!(1, res.responses.len());
        assert_eq!("/path", res.responses[0].href);
        assert_eq!(1, res.responses[0].propstat.len());
        assert_eq!("data", res.responses[0].propstat[0].prop);
        assert_eq!(
            Some("HTTP/1.1 200 OK"),
            res.responses[0].propstat[0]
                .status
                .as_ref()
                .map(|s| s.as_ref())
        );
    }

    #[test]
    fn multiple_propstats() {
        #[derive(Debug, Default, Deserialize)]
        struct Response {
            getetag: Option<String>,
            getlastmodified: Option<String>,
        }

        let res: Multistatus<Response> = xml::from_str(
            r#"
            <multistatus xmlns="DAV:">
	        <response>
		    <href>/path</href>
                    <propstat>
			<prop>
                            <getetag>etag</getetag>
                        </prop>
			<status>HTTP/1.1 200 OK</status>
		    </propstat>
                    <propstat>
			<prop>
                            <getlastmodified />
                        </prop>
			<status>HTTP/1.1 404 Not Found</status>
		    </propstat>
	        </response>
            </multistatus>
            "#,
        )
        .unwrap();

        assert_eq!(1, res.responses.len());
        assert_eq!("/path", res.responses[0].href);
        assert_eq!(2, res.responses[0].propstat.len());
        assert_eq!(
            Some("etag"),
            res.responses[0].propstat[0]
                .prop
                .getetag
                .as_ref()
                .map(|etag| etag.as_ref())
        );
        assert_eq!(
            Some("HTTP/1.1 200 OK"),
            res.responses[0].propstat[0]
                .status
                .as_ref()
                .map(|v| v.as_ref())
        );
        assert_eq!(
            Some(""),
            res.responses[0].propstat[1]
                .prop
                .getlastmodified
                .as_ref()
                .map(|v| v.as_ref())
        );
        assert_eq!(
            Some("HTTP/1.1 404 Not Found"),
            res.responses[0].propstat[1]
                .status
                .as_ref()
                .map(|s| s.as_ref())
        );
    }
}
