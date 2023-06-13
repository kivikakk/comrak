/// The metadata of a multiline blockquote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeMultilineBlockQuote {
    /// The length of the fence.
    pub fence_length: usize,

    /// ??? The indentation level of the code within the block.
    pub fence_offset: usize,
}
