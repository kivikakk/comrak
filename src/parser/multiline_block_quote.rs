/// The metadata of a multiline blockquote.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeMultilineBlockQuote {
    /// The length of the fence.
    pub fence_length: usize,

    /// The indentation level of the fence marker.
    pub fence_offset: usize,
}
