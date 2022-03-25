use serde::Deserialize;

/// Represents the user account from the config file.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeserializedAccountConfig {
    /// Makes this account the default one. Defaults to false.
    pub default: Option<bool>,
    /// Represents the directory used to synchronize
    /// contacts. Defaults to $XDG_DATA_HOME/<account-name>.
    pub sync_dir: Option<String>,
    /// Represents the CardDAV server host.
    pub host: String,
    /// Represents the CardDAV server port. Defaults to 8843.
    pub port: Option<u16>,
    /// Represents the CardDAV login.
    pub login: String,
    /// Represents the CardDAV password command.
    pub passwd_cmd: String,
}
