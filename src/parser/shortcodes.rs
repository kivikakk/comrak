use std::{convert::TryFrom, str};

/// The details of an inline emoji.
#[derive(Debug, Clone)]
pub struct NodeShortCode(
    /// A short code that is translated into an emoji
    String,
);

impl NodeShortCode {
    pub fn is_valid(value: &str) -> bool {
        emojis::get_by_shortcode(value).is_some()
    }

    pub fn shortcode(&self) -> &str {
        &self.0
    }

    pub fn emoji(&self) -> &'static str {
        emojis::get_by_shortcode(&self.0).unwrap().as_str()
    }
}

impl TryFrom<&str> for NodeShortCode {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, ()> {
        if Self::is_valid(value) {
            Ok(Self(value.into()))
        } else {
            Err(())
        }
    }
}
