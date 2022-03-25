use anyhow::{anyhow, Context, Result};
use log::{debug, info, trace};
use serde::Deserialize;
use std::{collections::HashMap, env, fs, path::PathBuf};
use toml;

use crate::config::DeserializedAccountConfig;

/// Represents the user config file.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeserializedConfig {
    /// Represents all the user accounts.
    #[serde(flatten)]
    pub accounts: HashMap<String, DeserializedAccountConfig>,
}

impl DeserializedConfig {
    /// Tries to create a config from an optional path.
    pub fn from_opt_path(path: Option<&str>) -> Result<Self> {
        info!("begin: try to parse config from path");
        debug!("path: {:?}", path);
        let path = path.map(|s| s.into()).unwrap_or(Self::path()?);
        let content = fs::read_to_string(path).context("cannot read config file")?;
        let config = toml::from_str(&content).context("cannot parse config file")?;
        info!("end: try to parse config from path");
        trace!("config: {:?}", config);
        Ok(config)
    }

    /// Tries to get the XDG config file path from XDG_CONFIG_HOME environment variable.
    fn path_from_xdg() -> Result<PathBuf> {
        let path =
            env::var("XDG_CONFIG_HOME").context("cannot find \"XDG_CONFIG_HOME\" env var")?;
        let path = PathBuf::from(path).join("cardamom").join("config.toml");
        if path.is_file() {
            Ok(path)
        } else {
            Err(anyhow!("cannot find config file {:?}", path))
        }
    }

    /// Tries to get the XDG config file path from HOME environment variable.
    fn path_from_xdg_alt() -> Result<PathBuf> {
        let home_var = if cfg!(target_family = "windows") {
            "USERPROFILE"
        } else {
            "HOME"
        };
        let path = env::var(home_var).context(format!("cannot find {:?} env var", home_var))?;
        let path = PathBuf::from(path)
            .join(".config")
            .join("cardamom")
            .join("config.toml");
        if path.is_file() {
            Ok(path)
        } else {
            Err(anyhow!("cannot find config file {:?}", path))
        }
    }

    /// Tries to get the .cardamomrc config file path from HOME environment variable.
    fn path_from_home() -> Result<PathBuf> {
        let home_var = if cfg!(target_family = "windows") {
            "USERPROFILE"
        } else {
            "HOME"
        };
        let path = env::var(home_var).context(format!("cannot find {:?} env var", home_var))?;
        let path = PathBuf::from(path).join(".cardamomrc");
        if path.is_file() {
            Ok(path)
        } else {
            Err(anyhow!("cannot find config file {:?}", path))
        }
    }

    /// Tries to get the config file path.
    pub fn path() -> Result<PathBuf> {
        Self::path_from_xdg()
            .or_else(|_| Self::path_from_xdg_alt())
            .or_else(|_| Self::path_from_home())
            .context("cannot find config path")
    }
}
