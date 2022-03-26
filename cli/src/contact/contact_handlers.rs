//! Contact handlers module.
//!
//! This module contains all handlers related to the contact.

use anyhow::Result;
use log::{info, trace};

use cardamom_lib::{cache::CachedCards, local::LocalCards, remote::RemoteCards};

use crate::{config::AccountConfig, output::PrinterService};

/// Synchronizes contacts.
pub fn sync<'a, P: PrinterService>(config: &AccountConfig, printer: &mut P) -> Result<()> {
    info!(">> sync contacts handler");

    let cache = CachedCards::new(config.cache_cards_file_path())?;
    trace!("cache: {:?}", cache);

    let local = LocalCards::new(config.sync_dir.clone())?;
    trace!("local: {:?}", local);

    let remote = RemoteCards::new(
        config.host.clone(),
        config.port.clone(),
        config.login.clone(),
        config.passwd()?,
    )?;
    trace!("remote: {:?}", remote);

    printer.print_str("TODO")?;

    info!("<< sync contacts handler");
    Ok(())
}
