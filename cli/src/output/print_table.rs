use anyhow::Result;
use std::io;
use termcolor::{self, StandardStream};

pub trait WriteColor: io::Write + termcolor::WriteColor {}

impl WriteColor for StandardStream {}

pub trait PrintTable {
    fn print_table(&self, writer: &mut dyn WriteColor, opts: PrintTableOpts) -> Result<()>;
}

pub struct PrintTableOpts {
    pub max_width: Option<usize>,
}
