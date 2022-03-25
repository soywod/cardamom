use anyhow::{anyhow, Result};
use log::{debug, info, trace};
use std::{env, path::PathBuf};

use crate::config::*;

/// Represents the user account.
#[derive(Debug, Default, Clone)]
pub struct AccountConfig {
    /// Represents the name of the user account.
    pub name: String,
    /// Makes this account the default one.
    pub default: bool,
    /// Represents the directory used to synchronize contacts.
    pub sync_dir: PathBuf,
    /// Represents the CardDAV server host.
    pub host: String,
    /// Represents the CardDAV server port.
    pub port: u16,
    /// Represents the CardDAV login.
    pub login: String,
    /// Represents the CardDAV password command.
    pub passwd_cmd: String,
}

impl<'a> AccountConfig {
    /// Tries to create an account from a config and an optional account name.
    pub fn from_config_and_opt_account_name(
        config: &'a DeserializedConfig,
        account_name: Option<&str>,
    ) -> Result<AccountConfig> {
        info!(">> build account from config and account name");
        debug!("account name: {:?}", account_name.unwrap_or("default"));

        let (name, account) = match account_name.map(|name| name.trim()) {
            Some("default") | Some("") | None => config
                .accounts
                .iter()
                .find(|(_, account)| account.default.unwrap_or_default())
                .map(|(name, account)| (name.to_owned(), account))
                .ok_or_else(|| anyhow!("cannot find default account")),
            Some(name) => config
                .accounts
                .get(name)
                .map(|account| (name.to_owned(), account))
                .ok_or_else(|| anyhow!("cannot find account {:?}", name)),
        }?;
        debug!("selected account name: {:?}", name);
        trace!("account: {:?}", account);

        let account_config = AccountConfig {
            name,
            default: account.default.unwrap_or_default(),
            sync_dir: account
                .sync_dir
                .as_ref()
                .and_then(|dir| shellexpand::full(dir).ok())
                .map(|dir| PathBuf::from(dir.to_string()))
                .unwrap_or_else(env::temp_dir),
            host: account.host.to_owned(),
            port: account.port.unwrap_or(8843),
            login: account.login.to_owned(),
            passwd_cmd: account.passwd_cmd.to_owned(),
        };
        trace!("account config: {:?}", account_config);

        info!("<< build account from config and account name");
        Ok(account_config)
    }
}