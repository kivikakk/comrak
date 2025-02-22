use crate::html;
use crate::html::Anchorizer;
use crate::Options;
use crate::Plugins;

use std::cell::Cell;
use std::io;
use std::io::Write;

/// TODO
pub struct Context<'o, 'c> {
    output: &'o mut dyn Write,
    last_was_lf: Cell<bool>,

    /// TODO
    pub options: &'o Options<'c>,
    /// TODO
    pub plugins: &'o Plugins<'o>,

    pub(super) anchorizer: Anchorizer,
    pub(super) footnote_ix: u32,
    pub(super) written_footnote_ix: u32,
}

impl<'o, 'c> Context<'o, 'c> {
    /// TODO
    pub fn new(
        output: &'o mut dyn Write,
        options: &'o Options<'c>,
        plugins: &'o Plugins<'o>,
    ) -> Self {
        Context {
            output,
            last_was_lf: Cell::new(true),
            options,
            plugins,
            anchorizer: Anchorizer::new(),
            footnote_ix: 0,
            written_footnote_ix: 0,
        }
    }

    /// TODO
    pub fn finish(&mut self) -> io::Result<()> {
        if self.footnote_ix > 0 {
            self.write_all(b"</ol>\n</section>\n")?;
        }
        Ok(())
    }

    /// TODO
    pub fn cr(&mut self) -> io::Result<()> {
        if !self.last_was_lf.get() {
            self.write_all(b"\n")?;
        }
        Ok(())
    }

    /// TODO
    pub fn escape(&mut self, buffer: &[u8]) -> io::Result<()> {
        html::escape(self, buffer)
    }

    /// TODO
    pub fn escape_href(&mut self, buffer: &[u8]) -> io::Result<()> {
        html::escape_href(self, buffer)
    }
}

impl<'o, 'c> Write for Context<'o, 'c> {
    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let l = buf.len();
        if l > 0 {
            self.last_was_lf.set(buf[l - 1] == 10);
        }
        self.output.write(buf)
    }
}

impl<'o, 'c> std::fmt::Debug for Context<'o, 'c> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        formatter.write_str("<comrak::html::Context>")
    }
}
