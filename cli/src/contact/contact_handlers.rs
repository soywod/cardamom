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

    let local = LocalCards::new(config.sync_dir.clone())?;
    let remote = RemoteCards::new(
        config.sync_dir.clone(),
        config.host.clone(),
        config.port.clone(),
        config.login.clone(),
        config.passwd()?,
    )?;

    let patch = Patch::new(local, remote);
    trace!("patch: {:?}", patch);

    printer.print_str("TODO")?;

    info!("<< sync contacts handler");
    Ok(())
}
