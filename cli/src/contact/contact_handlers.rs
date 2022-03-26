//! Contact handlers module.
//!
//! This module contains all handlers related to the contact.

use anyhow::{Context, Result};
use log::{info, trace};

use cardamom_lib::{cache::CachedCards, local::LocalCards, remote::RemoteCards, sync::Patch};

use crate::{config::AccountConfig, output::PrinterService};

/// Synchronizes contacts.
pub fn sync<'a, P: PrinterService>(config: &AccountConfig, printer: &mut P) -> Result<()> {
    info!(">> sync contacts handler");

    let cache = CachedCards::new(config.cache_cards_file_path())?;
    let local = LocalCards::new(config.sync_dir.clone())?;
    let remote = RemoteCards::new(
        config.host.clone(),
        config.port.clone(),
        config.login.clone(),
        config.passwd()?,
    )?;

    let patch = Patch::new(cache.cards, local.cards, remote.cards);
    trace!("patch: {:?}", patch);

    printer.print_str("TODO")?;

    info!("<< sync contacts handler");
    Ok(())
}
