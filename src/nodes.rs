//! The CommonMark AST.

use arena_tree::Node;
use std::cell::RefCell;

/// The core AST node enum.
#[derive(Debug, Clone)]
pub enum NodeValue {
    /// The root of every CommonMark document.  Contains **blocks**.
    Document,

    /// **Block**. A [block quote](https://github.github.com/gfm/#block-quotes).  Contains other
    /// **blocks**.
    ///
    /// ``` md
    /// > A block quote.
    /// ```
    BlockQuote,

    /// **Block**.  A [list](https://github.github.com/gfm/#lists).  Contains
    /// [list items](https://github.github.com/gfm/#list-items).
    ///
    /// ``` md
    /// * An unordered list
    /// * Another item
    ///
    /// 1. An ordered list
    /// 2. Another item
    /// ```
    List(NodeList),

    /// **Block**.  A [list item](https://github.github.com/gfm/#list-items).  Contains other
    /// **blocks**.
    Item(NodeList),

    /// **Block**. A code block; may be [fenced](https://github.github.com/gfm/#fenced-code-blocks)
    /// or [indented](https://github.github.com/gfm/#indented-code-blocks).  Contains raw text
    /// which is not parsed as Markdown, although is HTML escaped.
    CodeBlock(NodeCodeBlock),

    /// **Block**. A [HTML block](https://github.github.com/gfm/#html-blocks).  Contains raw text
    /// which is neither parsed as Markdown nor HTML escaped.
    HtmlBlock(NodeHtmlBlock),

    /// **Block**. A [paragraph](https://github.github.com/gfm/#paragraphs).  Contains **inlines**.
    Paragraph,

    /// **Block**. A heading; may be an [ATX heading](https://github.github.com/gfm/#atx-headings)
    /// or a [setext heading](https://github.github.com/gfm/#setext-headings). Contains
    /// **inlines**.
    Heading(NodeHeading),

    /// **Block**. A [thematic break](https://github.github.com/gfm/#thematic-breaks).  Has no
    /// children.
    ThematicBreak,

    /// **Block**. A footnote definition.  The `Vec<u8>` is the footnote's name.
    /// Contains other **blocks**.
    FootnoteDefinition(Vec<u8>),

    /// **Block**. A [table](https://github.github.com/gfm/#tables-extension-) per the GFM spec.
    /// Contains table rows.
    Table(Vec<TableAlignment>),

    /// **Block**. A table row.  The `bool` represents whether the row is the header row or not.
    /// Contains table cells.
    TableRow(bool),

    /// **Block**.  A table cell.  Contains **inlines**.
    TableCell,

    /// **Inline**.  [Textual content](https://github.github.com/gfm/#textual-content).  All text
    /// in a document will be contained in a `Text` node.
    Text(Vec<u8>),

    /// **Inline**.  A [soft line break](https://github.github.com/gfm/#soft-line-breaks).  If
    /// the `hardbreaks` option is set in `ComrakOptions` during formatting, it will be formatted
    /// as a `LineBreak`.
    SoftBreak,

    /// **Inline**.  A [hard line break](https://github.github.com/gfm/#hard-line-breaks).
    LineBreak,

    /// **Inline**.  A [code span](https://github.github.com/gfm/#code-spans).
    Code(Vec<u8>),

    /// **Inline**.  [Raw HTML](https://github.github.com/gfm/#raw-html) contained inline.
    HtmlInline(Vec<u8>),

    /// **Inline**.  [Emphasised](https://github.github.com/gfm/#emphasis-and-strong-emphasis)
    /// text.
    Emph,

    /// **Inline**.  [Strong](https://github.github.com/gfm/#emphasis-and-strong-emphasis) text.
    Strong,

    /// **Inline**.  [Strikethrough](https://github.github.com/gfm/#strikethrough-extension-) text
    /// per the GFM spec.
    Strikethrough,

    /// **Inline**.  Superscript.  Enabled with `ext_superscript` option.
    Superscript,

    /// **Inline**.  A [link](https://github.github.com/gfm/#links) to some URL, with possible
    /// title.
    Link(NodeLink),

    /// **Inline**.  An [image](https://github.github.com/gfm/#images).
    Image(NodeLink),

    /// **Inline**.  A footnote reference; the `Vec<u8>` is the referent footnote's name.
    FootnoteReference(Vec<u8>),
}

/// Alignment of a single table cell.
#[derive(Debug, Copy, Clone)]
pub enum TableAlignment {
    /// Cell content is unaligned.
    None,

    /// Cell content is aligned left.
    Left,

    /// Cell content is centered.
    Center,

    /// Cell content is aligned right.
    Right,
}

/// The details of a link's destination, or an image's source.
#[derive(Debug, Clone)]
pub struct NodeLink {
    /// The URL for the link destination or image source.
    pub url: Vec<u8>,

    /// The title for the link or image.
    ///
    /// Note this field is used for the `title` attribute by the HTML formatter even for images;
    /// `alt` text is supplied in the image inline text.
    pub title: Vec<u8>,
}

/// The metadata of a list; the kind of list, the delimiter used and so on.
#[derive(Debug, Default, Clone, Copy)]
pub struct NodeList {
    /// The kind of list (bullet (unordered) or ordered).
    pub list_type: ListType,

    #[doc(hidden)]
    pub marker_offset: usize,

    #[doc(hidden)]
    pub padding: usize,

    /// For ordered lists, the ordinal the list starts at.
    pub start: usize,

    /// For ordered lists, the delimiter after each number.
    pub delimiter: ListDelimType,

    /// For bullet lists, the character used for each bullet.
    pub bullet_char: u8,

    /// Whether the list is [tight](https://github.github.com/gfm/#tight), i.e. whether the
    /// paragraphs are wrapped in `<p>` tags when formatted as HTML.
    pub tight: bool,
}

/// The type of list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListType {
    /// A bullet list, i.e. an unordered list.
    Bullet,

    /// An ordered list.
    Ordered,
}

impl Default for ListType {
    fn default() -> ListType {
        ListType::Bullet
    }
}

/// The delimiter for ordered lists, i.e. the character which appears after each number.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListDelimType {
    /// A period character `.`.
    Period,

    /// A paren character `)`.
    Paren,
}

impl Default for ListDelimType {
    fn default() -> ListDelimType {
        ListDelimType::Period
    }
}

/// The metadata and data of a code block (fenced or indented).
#[derive(Default, Debug, Clone)]
pub struct NodeCodeBlock {
    /// Whether the code block is fenced.
    pub fenced: bool,

    /// For fenced code blocks, the fence character itself (`` ` `` or `~`).
    pub fence_char: u8,

    /// For fenced code blocks, the length of the fence.
    pub fence_length: usize,

    #[doc(hidden)]
    pub fence_offset: usize,

    /// For fenced code blocks, the [info string](https://github.github.com/gfm/#info-string) after
    /// the opening fence, if any.
    pub info: Vec<u8>,

    /// The literal contents of the code block.  As the contents are not interpreted as Markdown at
    /// all, they are contained within this structure, rather than inserted into a child inline of
    /// any kind.
    pub literal: Vec<u8>,
}

/// The metadata of a heading.
#[derive(Default, Debug, Clone, Copy)]
pub struct NodeHeading {
    /// The level of the header; from 1 to 6 for ATX headings, 1 or 2 for setext headings.
    pub level: u32,

    /// Whether the heading is setext (if not, ATX).
    pub setext: bool,
}

/// The metadata of an included HTML block.
#[derive(Debug, Clone)]
pub struct NodeHtmlBlock {
    #[doc(hidden)]
    pub block_type: u8,

    /// The literal contents of the HTML block.  Per NodeCodeBlock, the content is included here
    /// rather than in any inline.
    pub literal: Vec<u8>,
}

impl NodeValue {
    /// Indicates whether this node is a block node or inline node.
    pub fn block(&self) -> bool {
        match *self {
            NodeValue::Document
            | NodeValue::BlockQuote
            | NodeValue::FootnoteDefinition(_)
            | NodeValue::List(..)
            | NodeValue::Item(..)
            | NodeValue::CodeBlock(..)
            | NodeValue::HtmlBlock(..)
            | NodeValue::Paragraph
            | NodeValue::Heading(..)
            | NodeValue::ThematicBreak
            | NodeValue::Table(..)
            | NodeValue::TableRow(..)
            | NodeValue::TableCell => true,
            _ => false,
        }
    }

    #[doc(hidden)]
    pub fn accepts_lines(&self) -> bool {
        match *self {
            NodeValue::Paragraph | NodeValue::Heading(..) | NodeValue::CodeBlock(..) => true,
            _ => false,
        }
    }

    /// Indicates whether this node may contain inlines.
    pub fn contains_inlines(&self) -> bool {
        match *self {
            NodeValue::Paragraph | NodeValue::Heading(..) | NodeValue::TableCell => true,
            _ => false,
        }
    }

    /// Return a reference to the text of a `Text` inline, if this node is one.
    ///
    /// Convenience method.
    pub fn text(&self) -> Option<&Vec<u8>> {
        match *self {
            NodeValue::Text(ref t) => Some(t),
            _ => None,
        }
    }

    /// Return a mutable reference to the text of a `Text` inline, if this node is one.
    ///
    /// Convenience method.
    pub fn text_mut(&mut self) -> Option<&mut Vec<u8>> {
        match *self {
            NodeValue::Text(ref mut t) => Some(t),
            _ => None,
        }
    }
}

/// A single node in the CommonMark AST.
///
/// The struct contains metadata about the node's position in the original document, and the core
/// enum, `NodeValue`.
#[derive(Debug, Clone)]
pub struct Ast {
    /// The node value itself.
    pub value: NodeValue,

    /// The line in the input document the node starts at.
    pub start_line: u32,

    #[doc(hidden)]
    pub content: Vec<u8>,
    #[doc(hidden)]
    pub open: bool,
    #[doc(hidden)]
    pub last_line_blank: bool,
}

#[doc(hidden)]
pub fn make_block(value: NodeValue, start_line: u32) -> Ast {
    Ast {
        value: value,
        content: vec![],
        start_line: start_line,
        open: true,
        last_line_blank: false,
    }
}

/// The type of a node within the document.
///
/// It is bound by the lifetime `'a`, which corresponds to the `Arena` nodes are allocated in.
/// `AstNode`s are almost handled as a reference itself bound by `'a`.  Child `Ast`s are wrapped in
/// `RefCell` for interior mutability.
pub type AstNode<'a> = Node<'a, RefCell<Ast>>;

#[doc(hidden)]
pub fn last_child_is_open<'a>(node: &'a AstNode<'a>) -> bool {
    node.last_child().map_or(false, |n| n.data.borrow().open)
}

#[doc(hidden)]
pub fn can_contain_type<'a>(node: &'a AstNode<'a>, child: &NodeValue) -> bool {
    if let NodeValue::Document = *child {
        return false;
    }

    match node.data.borrow().value {
        NodeValue::Document
        | NodeValue::BlockQuote
        | NodeValue::FootnoteDefinition(_)
        | NodeValue::Item(..) => {
            child.block() && match *child {
                NodeValue::Item(..) => false,
                _ => true,
            }
        }

        NodeValue::List(..) => match *child {
            NodeValue::Item(..) => true,
            _ => false,
        },

        NodeValue::Paragraph
        | NodeValue::Heading(..)
        | NodeValue::Emph
        | NodeValue::Strong
        | NodeValue::Link(..)
        | NodeValue::Image(..) => !child.block(),

        NodeValue::Table(..) => match *child {
            NodeValue::TableRow(..) => true,
            _ => false,
        },

        NodeValue::TableRow(..) => match *child {
            NodeValue::TableCell => true,
            _ => false,
        },

        NodeValue::TableCell => match *child {
            NodeValue::Text(..)
            | NodeValue::Code(..)
            | NodeValue::Emph
            | NodeValue::Strong
            | NodeValue::Link(..)
            | NodeValue::Image(..)
            | NodeValue::Strikethrough
            | NodeValue::HtmlInline(..) => true,
            _ => false,
        },

        _ => false,
    }
}

#[doc(hidden)]
pub fn ends_with_blank_line<'a>(node: &'a AstNode<'a>) -> bool {
    let mut it = Some(node);
    while let Some(cur) = it {
        if cur.data.borrow().last_line_blank {
            return true;
        }
        match cur.data.borrow().value {
            NodeValue::List(..) | NodeValue::Item(..) => it = cur.last_child(),
            _ => it = None,
        };
    }
    false
}

#[doc(hidden)]
pub fn containing_block<'a>(node: &'a AstNode<'a>) -> Option<&'a AstNode<'a>> {
    let mut ch = Some(node);
    while let Some(n) = ch {
        if n.data.borrow().value.block() {
            return Some(n);
        }
        ch = n.parent();
    }
    None
}
