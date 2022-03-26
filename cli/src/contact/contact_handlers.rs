//! Contact handlers module.
//!
//! This module contains all handlers related to the contact.

use anyhow::{Context, Result};
use cardamom_lib::cache::CachedCards;
use log::info;

use crate::{config::AccountConfig, output::PrinterService};

/// Synchronizes contacts.
pub fn sync<'a, P: PrinterService>(config: &AccountConfig, printer: &mut P) -> Result<()> {
    info!(">> sync contacts handler");

    let cache = CachedCards::new(config.cache_cards_file_path())?;
    println!("cache: {:?}", cache);
    printer.print_str("TODO")?;

    info!("<< sync contacts handler");
    Ok(())
}
