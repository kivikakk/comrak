//! Plugin definitions.

#[cfg(feature = "syntect")]
#[cfg_attr(docsrs, doc(cfg(feature = "syntect")))]
pub mod syntect;

#[cfg(feature = "mathml")]
#[cfg_attr(docsrs, doc(cfg(feature = "mathml")))]
pub mod pulldown_latex;
