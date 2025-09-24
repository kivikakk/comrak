use std::borrow::Cow;
use std::collections::HashSet;
use unicode_categories::UnicodeCategories;

/// Converts header strings to canonical, unique, but still human-readable,
/// anchors.
///
/// To guarantee uniqueness, an anchorizer keeps track of the anchors it has
/// returned; use one per output file.
///
/// ## Example
///
/// ```
/// # use comrak::Anchorizer;
/// let mut anchorizer = Anchorizer::new();
/// // First "stuff" is unsuffixed.
/// assert_eq!("stuff", anchorizer.anchorize("Stuff"));
/// // Second "stuff" has "-1" appended to make it unique.
/// assert_eq!("stuff-1", anchorizer.anchorize("Stuff"));
/// ```
#[derive(Debug, Default)]
#[doc(hidden)]
pub struct Anchorizer(HashSet<String>);

impl Anchorizer {
    /// Construct a new anchorizer.
    pub fn new() -> Self {
        Anchorizer(HashSet::new())
    }

    /// Returns a String that has been converted into an anchor using the
    /// GFM algorithm, which involves changing spaces to dashes, removing
    /// problem characters and, if needed, adding a suffix to make the
    /// resultant anchor unique.
    ///
    /// ```
    /// # use comrak::Anchorizer;
    /// let mut anchorizer = Anchorizer::new();
    /// let source = "Ticks aren't in";
    /// assert_eq!("ticks-arent-in", anchorizer.anchorize(source));
    /// ```
    pub fn anchorize(&mut self, header: &str) -> String {
        fn is_permitted_char(&c: &char) -> bool {
            c == ' '
                || c == '-'
                || c.is_letter()
                || c.is_mark()
                || c.is_number()
                || c.is_punctuation_connector()
        }

        let mut id = header.to_lowercase();
        id = id
            .chars()
            .filter(is_permitted_char)
            .map(|c| if c == ' ' { '-' } else { c })
            .collect();

        let mut uniq = 0;
        id = loop {
            let anchor = if uniq == 0 {
                Cow::from(&id)
            } else {
                Cow::from(format!("{}-{}", id, uniq))
            };

            if !self.0.contains(&*anchor) {
                break anchor.into_owned();
            }

            uniq += 1;
        };
        self.0.insert(id.clone());
        id
    }
}
