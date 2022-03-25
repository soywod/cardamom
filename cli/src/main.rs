use anyhow::Result;
use clap::{App, AppSettings};
use std::env;

pub mod config;
pub mod contact;
pub mod output;

use crate::{
    config::{account_args, config_args, AccountConfig, DeserializedConfig},
    contact::{contact_args, contact_handlers},
    output::{output_args, StdoutPrinter},
};

fn create_app<'a>() -> App<'a, 'a> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .global_setting(AppSettings::GlobalVersion)
        .arg(&config_args::path_arg())
        .arg(&account_args::name_arg())
        .args(&output_args::args())
        .subcommands(contact_args::subcmds())
}

#[allow(clippy::single_match)]
fn main() -> Result<()> {
    // init env logger
    let default_env_filter = env_logger::DEFAULT_FILTER_ENV;
    env_logger::init_from_env(env_logger::Env::default().filter_or(default_env_filter, "off"));

    // init app
    let app = create_app();
    let m = app.get_matches();

    // init entities and services
    let config = DeserializedConfig::from_opt_path(m.value_of("config"))?;
    let account_config =
        AccountConfig::from_config_and_opt_account_name(&config, m.value_of("account"))?;
    let mut printer = StdoutPrinter::try_from(m.value_of("output"))?;

    // check contact commands
    match contact_args::matches(&m)? {
        Some(contact_args::Cmd::Sync) => {
            return contact_handlers::sync(&account_config, &mut printer);
        }
        _ => (),
    }

    Ok(())
}
