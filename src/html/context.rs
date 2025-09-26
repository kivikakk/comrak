use crate::html::{self, Anchorizer};
use crate::{Options, Plugins};

use std::cell::Cell;
use std::fmt::{self, Write};

/// Context struct given to formatter functions as taken by
/// [`html::format_document_with_formatter`].  Output can be appended to through
/// this struct's [`Write`] interface.
pub struct Context<'o, 'c, T = ()> {
    output: &'o mut dyn Write,
    last_was_lf: Cell<bool>,

    /// [`Options`] in use in this render.
    pub options: &'o Options<'c>,
    /// [`Plugins`] in use in this render.
    pub plugins: &'o Plugins<'o>,
    /// [`Anchorizer`] instance used in this render.
    pub anchorizer: Anchorizer,
    /// Any user data used by the [`Context`].
    pub user: T,

    pub(super) footnote_ix: u32,
    pub(super) written_footnote_ix: u32,
}

impl<'o, 'c, T> Context<'o, 'c, T> {
    pub(super) fn new(
        output: &'o mut dyn Write,
        options: &'o Options<'c>,
        plugins: &'o Plugins<'o>,
        user: T,
    ) -> Self {
        Context {
            output,
            last_was_lf: Cell::new(true),
            options,
            plugins,
            anchorizer: Anchorizer::new(),
            user,
            footnote_ix: 0,
            written_footnote_ix: 0,
        }
    }

    pub(super) fn finish(mut self) -> Result<T, fmt::Error> {
        if self.footnote_ix > 0 {
            self.write_str("</ol>\n</section>\n")?;
        }
        Ok(self.user)
    }

    /// If the last byte written to ts [`Write`] interface was **not** a U+000A
    /// LINE FEED, writes one. Otherwise, does nothing.
    ///
    /// (In other words, ensures the output is at a new line.)
    pub fn cr(&mut self) -> fmt::Result {
        if !self.last_was_lf.get() {
            self.write_str("\n")?;
        }
        Ok(())
    }

    /// Convenience wrapper for [`html::escape`].
    pub fn escape(&mut self, buffer: &str) -> fmt::Result {
        html::escape(self, buffer)
    }

    /// Convenience wrapper for [`html::escape_href`].
    pub fn escape_href(&mut self, buffer: &str) -> fmt::Result {
        let relaxed_autolinks = self.options.parse.relaxed_autolinks;
        html::escape_href(self, buffer, relaxed_autolinks)
    }
}

impl<'o, 'c, T> Write for Context<'o, 'c, T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let l = s.len();
        if l > 0 {
            self.last_was_lf.set(s.as_bytes()[l - 1] == 10);
        }
        self.output.write_str(s)
    }
}

impl<'o, 'c, T> fmt::Debug for Context<'o, 'c, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        formatter.write_str("<comrak::html::Context>")
    }
}
