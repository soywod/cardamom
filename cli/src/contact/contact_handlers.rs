//! Contact handlers module.
//!
//! This module contains all handlers related to the contact.

use anyhow::Result;
use log::info;

use crate::{config::AccountConfig, output::PrinterService};

/// Synchronizes contacts.
pub fn sync<'a, P: PrinterService>(_config: &AccountConfig, printer: &mut P) -> Result<()> {
    info!(">> sync contacts handler");

    printer.print_str("TODO")?;

    info!("<< sync contacts handler");
    Ok(())
}
