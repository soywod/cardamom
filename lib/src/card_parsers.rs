pub mod date_parser {
    use chrono::{DateTime, Local};
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc2822(&s)
            .map(|d| d.into())
            .map_err(serde::de::Error::custom)
    }
}

pub mod opt_date_parser {
    use chrono::{DateTime, Local};
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Local>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        return Ok(None);
        // let s = String::deserialize(deserializer)?;
        // DateTime::parse_from_rfc2822(&s)
        //     .map(|d| Some(d.into()))
        //     .map_err(serde::de::Error::custom)
    }
}

pub mod url_parser {
    use serde::{self, Deserialize, Deserializer};
    use url::Url;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Url, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Url::parse(&s)
            .map(|d| d.into())
            .map_err(serde::de::Error::custom)
    }
}
