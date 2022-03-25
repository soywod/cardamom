//! Contact arguments module.
//!
//! This module provides subcommands, arguments and a command matcher
//! related to the contact.

use anyhow::Result;
use clap::{self, App, ArgMatches, SubCommand};
use log::{debug, info};

/// Represents the contact commands.
#[derive(Debug, PartialEq, Eq)]
pub enum Cmd {
    /// Represents the sync contact command.
    Sync,
}

/// Represents the contact command matcher.
pub fn matches(m: &ArgMatches) -> Result<Option<Cmd>> {
    info!(">> carddav command matcher");

    let cmd = if let Some(m) = m.subcommand_matches("sync") {
        debug!("sync command matched");
        Some(Cmd::Sync)
    } else {
        None
    };

    info!("<< carddav command matcher");
    Ok(cmd)
}

/// Represents the contact subcommands.
pub fn subcmds<'a>() -> Vec<App<'a, 'a>> {
    vec![SubCommand::with_name("sync")
        .aliases(&["synchronize", "synchro", "syn", "s"])
        .about("Synchronizes contacts")]
}

#[cfg(test)]
mod tests {
    use clap::App;

    use super::*;

    #[test]
    fn it_should_match_cmds() {
        let arg = App::new("cardamom")
            .subcommands(subcmds())
            .get_matches_from(&["cardamom", "sync"]);

        assert_eq!(Some(Cmd::Sync), matches(&arg).unwrap());
    }

    #[test]
    fn it_should_match_aliases() {
        macro_rules! get_matches_from {
            ($alias:expr) => {
                App::new("cardamom")
                    .subcommands(subcmds())
                    .get_matches_from(&["cardamom", $alias])
                    .subcommand_name()
            };
        }

        assert_eq!(Some("sync"), get_matches_from!["synchronize"]);
        assert_eq!(Some("sync"), get_matches_from!["synchro"]);
        assert_eq!(Some("sync"), get_matches_from!["sync"]);
        assert_eq!(Some("sync"), get_matches_from!["syn"]);
        assert_eq!(Some("sync"), get_matches_from!["s"]);
    }
}
