extern crate emojis;

use std::str;

/// The details of an inline emoji
#[derive(Debug, Clone)]
pub struct NodeShortCode {
    /// A short code that is translated into an emoji
    shortcode: Option<String>,
}

impl NodeShortCode {
    pub fn is_valid(value: Vec<u8>) -> bool {
        let code = Self::from(value);
        code.emoji().is_some()
    }

    pub fn shortcode(&self) -> Option<String> {
        self.shortcode.clone()
    }

    pub fn emoji(&self) -> Option<&'static str> {
        Some(emojis::get_by_shortcode(self.shortcode()?.as_str())?.as_str())
    }
}

impl<'a> From<Vec<u8>> for NodeShortCode {
    fn from(value: Vec<u8>) -> Self {
        let captured = unsafe { str::from_utf8_unchecked(&value) };
        Self {
            shortcode: Some(captured.to_string()),
        }
    }
}
