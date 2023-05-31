//! Plugins for enhancing the default implementation of comrak can be defined in this module.

#[cfg(feature = "syntect")]
#[cfg_attr(docsrs, doc(cfg(feature = "syntect")))]
pub mod syntect;
