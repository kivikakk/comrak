//! The CommonMark AST.

use crate::arena_tree::Node;
use std::cell::RefCell;
use std::convert::TryFrom;

#[cfg(feature = "shortcodes")]
pub use crate::parser::shortcodes::NodeShortCode;

pub use crate::parser::math::NodeMath;
pub use crate::parser::multiline_block_quote::NodeMultilineBlockQuote;

/// The core AST node enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeValue {
    /// The root of every CommonMark document.  Contains **blocks**.
    Document,

    /// Non-Markdown front matter.  Treated as an opaque blob.
    FrontMatter(String),

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

    /// **Block**. A description list, enabled with `ext_description_lists` option.  Contains
    /// description items.
    ///
    /// It is required to put a blank line between terms and details.
    ///
    /// ``` md
    /// Term 1
    ///
    /// : Details 1
    ///
    /// Term 2
    ///
    /// : Details 2
    /// ```
    DescriptionList,

    /// *Block**. An item of a description list.  Contains a term and one details block.
    DescriptionItem(NodeDescriptionItem),

    /// **Block**. Term of an item in a definition list.
    DescriptionTerm,

    /// **Block**. Details of an item in a definition list.
    DescriptionDetails,

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

    /// **Block**. A footnote definition.  The `String` is the footnote's name.
    /// Contains other **blocks**.
    FootnoteDefinition(NodeFootnoteDefinition),

    /// **Block**. A [table](https://github.github.com/gfm/#tables-extension-) per the GFM spec.
    /// Contains table rows.
    Table(NodeTable),

    /// **Block**. A table row.  The `bool` represents whether the row is the header row or not.
    /// Contains table cells.
    TableRow(bool),

    /// **Block**.  A table cell.  Contains **inlines**.
    TableCell,

    /// **Inline**.  [Textual content](https://github.github.com/gfm/#textual-content).  All text
    /// in a document will be contained in a `Text` node.
    Text(String),

    /// **Block**. [Task list item](https://github.github.com/gfm/#task-list-items-extension-).
    /// The value is the symbol that was used in the brackets to mark a task item as checked, or
    /// None if the item is unchecked.
    TaskItem(Option<char>),

    /// **Inline**.  A [soft line break](https://github.github.com/gfm/#soft-line-breaks).  If
    /// the `hardbreaks` option is set in `Options` during formatting, it will be formatted
    /// as a `LineBreak`.
    SoftBreak,

    /// **Inline**.  A [hard line break](https://github.github.com/gfm/#hard-line-breaks).
    LineBreak,

    /// **Inline**.  A [code span](https://github.github.com/gfm/#code-spans).
    Code(NodeCode),

    /// **Inline**.  [Raw HTML](https://github.github.com/gfm/#raw-html) contained inline.
    HtmlInline(String),

    /// **Inline**.  [Emphasized](https://github.github.com/gfm/#emphasis-and-strong-emphasis)
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

    /// **Inline**.  A footnote reference.
    FootnoteReference(NodeFootnoteReference),

    #[cfg(feature = "shortcodes")]
    /// **Inline**. An Emoji character generated from a shortcode. Enable with feature "shortcodes".
    ShortCode(NodeShortCode),

    /// **Inline**. A math span. Contains raw text which is not parsed as Markdown.
    /// Dollar math or code math
    ///
    /// Inline math $1 + 2$ and $`1 + 2`$
    ///
    /// Display math $$1 + 2$$ and
    /// $$
    /// 1 + 2
    /// $$
    ///
    Math(NodeMath),

    /// **Block**. A [multiline block quote](https://github.github.com/gfm/#block-quotes).  Spans multiple
    /// lines and contains other **blocks**.
    ///
    /// ``` md
    /// >>>
    /// A paragraph.
    ///
    /// - item one
    /// - item two
    /// >>>
    /// ```
    MultilineBlockQuote(NodeMultilineBlockQuote),

    /// **Inline**.  A character that has been [escaped](https://github.github.com/gfm/#backslash-escapes)
    ///
    /// Enabled with [`escaped_char_spans`](crate::RenderOptionsBuilder::escaped_char_spans).
    Escaped,

    /// **Inline**.  A wikilink to some URL.
    WikiLink(NodeWikiLink),

    /// **Inline**.  Underline. Enabled with `underline` option.
    Underline,

    /// **Inline**.  Subscript. Enabled with `subscript` options.
    Subscript,

    /// **Inline**.  Spoilered text.  Enabled with `spoiler` option.
    SpoileredText,

    /// **Inline**. Text surrounded by escaped markup. Enabled with `spoiler` option.
    /// The `String` is the tag to be escaped.
    EscapedTag(String),
}

/// Alignment of a single table cell.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

impl TableAlignment {
    pub(crate) fn xml_name(&self) -> Option<&'static str> {
        match *self {
            TableAlignment::None => None,
            TableAlignment::Left => Some("left"),
            TableAlignment::Center => Some("center"),
            TableAlignment::Right => Some("right"),
        }
    }
}

/// The metadata of a table
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeTable {
    /// The table alignments
    pub alignments: Vec<TableAlignment>,

    /// Number of columns of the table
    pub num_columns: usize,

    /// Number of rows of the table
    pub num_rows: usize,

    /// Number of non-empty, non-autocompleted cells
    pub num_nonempty_cells: usize,
}

/// An inline [code span](https://github.github.com/gfm/#code-spans).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeCode {
    /// The number of backticks
    pub num_backticks: usize,

    /// The content of the inline code span.
    /// As the contents are not interpreted as Markdown at all,
    /// they are contained within this structure,
    /// rather than inserted into a child inline of any kind.
    pub literal: String,
}

/// The details of a link's destination, or an image's source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeLink {
    /// The URL for the link destination or image source.
    pub url: String,

    /// The title for the link or image.
    ///
    /// Note this field is used for the `title` attribute by the HTML formatter even for images;
    /// `alt` text is supplied in the image inline text.
    pub title: String,
}

/// The details of a wikilink's destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeWikiLink {
    /// The URL for the link destination.
    pub url: String,
}

/// The metadata of a list; the kind of list, the delimiter used and so on.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NodeList {
    /// The kind of list (bullet (unordered) or ordered).
    pub list_type: ListType,

    /// Number of spaces before the list marker.
    pub marker_offset: usize,

    /// Number of characters between the start of the list marker and the item text (including the list marker(s)).
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

    /// Whether the list contains tasks (checkbox items)
    pub is_task_list: bool,
}

/// The metadata of a description list
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NodeDescriptionItem {
    /// Number of spaces before the list marker.
    pub marker_offset: usize,

    /// Number of characters between the start of the list marker and the item text (including the list marker(s)).
    pub padding: usize,

    /// Whether the list is [tight](https://github.github.com/gfm/#tight), i.e. whether the
    /// paragraphs are wrapped in `<p>` tags when formatted as HTML.
    pub tight: bool,
}

/// The type of list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListType {
    /// A bullet list, i.e. an unordered list.
    #[default]
    Bullet,

    /// An ordered list.
    Ordered,
}

/// The delimiter for ordered lists, i.e. the character which appears after each number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ListDelimType {
    /// A period character `.`.
    #[default]
    Period,

    /// A paren character `)`.
    Paren,
}

impl ListDelimType {
    pub(crate) fn xml_name(&self) -> &'static str {
        match *self {
            ListDelimType::Period => "period",
            ListDelimType::Paren => "paren",
        }
    }
}

/// The metadata and data of a code block (fenced or indented).
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct NodeCodeBlock {
    /// Whether the code block is fenced.
    pub fenced: bool,

    /// For fenced code blocks, the fence character itself (`` ` `` or `~`).
    pub fence_char: u8,

    /// For fenced code blocks, the length of the fence.
    pub fence_length: usize,

    /// For fenced code blocks, the indentation level of the code within the block.
    pub fence_offset: usize,

    /// For fenced code blocks, the [info string](https://github.github.com/gfm/#info-string) after
    /// the opening fence, if any.
    pub info: String,

    /// The literal contents of the code block.  As the contents are not interpreted as Markdown at
    /// all, they are contained within this structure, rather than inserted into a child inline of
    /// any kind.
    pub literal: String,
}

/// The metadata of a heading.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeHeading {
    /// The level of the header; from 1 to 6 for ATX headings, 1 or 2 for setext headings.
    pub level: u8,

    /// Whether the heading is setext (if not, ATX).
    pub setext: bool,
}

/// The metadata of an included HTML block.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeHtmlBlock {
    /// The HTML block's type
    pub block_type: u8,

    /// The literal contents of the HTML block.  Per NodeCodeBlock, the content is included here
    /// rather than in any inline.
    pub literal: String,
}

/// The metadata of a footnote definition.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeFootnoteDefinition {
    /// The name of the footnote.
    pub name: String,

    /// Total number of references to this footnote
    pub total_references: u32,
}

/// The metadata of a footnote reference.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NodeFootnoteReference {
    /// The name of the footnote.
    pub name: String,

    /// The index of reference to the same footnote
    pub ref_num: u32,

    /// The index of the footnote in the document.
    pub ix: u32,
}

impl NodeValue {
    /// Indicates whether this node is a block node or inline node.
    pub fn block(&self) -> bool {
        matches!(
            *self,
            NodeValue::Document
                | NodeValue::BlockQuote
                | NodeValue::FootnoteDefinition(_)
                | NodeValue::List(..)
                | NodeValue::DescriptionList
                | NodeValue::DescriptionItem(_)
                | NodeValue::DescriptionTerm
                | NodeValue::DescriptionDetails
                | NodeValue::Item(..)
                | NodeValue::CodeBlock(..)
                | NodeValue::HtmlBlock(..)
                | NodeValue::Paragraph
                | NodeValue::Heading(..)
                | NodeValue::ThematicBreak
                | NodeValue::Table(..)
                | NodeValue::TableRow(..)
                | NodeValue::TableCell
                | NodeValue::TaskItem(..)
                | NodeValue::MultilineBlockQuote(_)
        )
    }

    /// Whether the type the node is of can contain inline nodes.
    pub fn contains_inlines(&self) -> bool {
        matches!(
            *self,
            NodeValue::Paragraph | NodeValue::Heading(..) | NodeValue::TableCell
        )
    }

    /// Return a reference to the text of a `Text` inline, if this node is one.
    ///
    /// Convenience method.
    pub fn text(&self) -> Option<&String> {
        match *self {
            NodeValue::Text(ref t) => Some(t),
            _ => None,
        }
    }

    /// Return a mutable reference to the text of a `Text` inline, if this node is one.
    ///
    /// Convenience method.
    pub fn text_mut(&mut self) -> Option<&mut String> {
        match *self {
            NodeValue::Text(ref mut t) => Some(t),
            _ => None,
        }
    }

    pub(crate) fn accepts_lines(&self) -> bool {
        matches!(
            *self,
            NodeValue::Paragraph | NodeValue::Heading(..) | NodeValue::CodeBlock(..)
        )
    }

    pub(crate) fn xml_node_name(&self) -> &'static str {
        match *self {
            NodeValue::Document => "document",
            NodeValue::BlockQuote => "block_quote",
            NodeValue::FootnoteDefinition(_) => "footnote_definition",
            NodeValue::List(..) => "list",
            NodeValue::DescriptionList => "description_list",
            NodeValue::DescriptionItem(_) => "description_item",
            NodeValue::DescriptionTerm => "description_term",
            NodeValue::DescriptionDetails => "description_details",
            NodeValue::Item(..) => "item",
            NodeValue::CodeBlock(..) => "code_block",
            NodeValue::HtmlBlock(..) => "html_block",
            NodeValue::Paragraph => "paragraph",
            NodeValue::Heading(..) => "heading",
            NodeValue::ThematicBreak => "thematic_break",
            NodeValue::Table(..) => "table",
            NodeValue::TableRow(..) => "table_row",
            NodeValue::TableCell => "table_cell",
            NodeValue::Text(..) => "text",
            NodeValue::SoftBreak => "softbreak",
            NodeValue::LineBreak => "linebreak",
            NodeValue::Image(..) => "image",
            NodeValue::Link(..) => "link",
            NodeValue::Emph => "emph",
            NodeValue::Strong => "strong",
            NodeValue::Code(..) => "code",
            NodeValue::HtmlInline(..) => "html_inline",
            NodeValue::Strikethrough => "strikethrough",
            NodeValue::FrontMatter(_) => "frontmatter",
            NodeValue::TaskItem { .. } => "taskitem",
            NodeValue::Superscript => "superscript",
            NodeValue::FootnoteReference(..) => "footnote_reference",
            #[cfg(feature = "shortcodes")]
            NodeValue::ShortCode(_) => "shortcode",
            NodeValue::MultilineBlockQuote(_) => "multiline_block_quote",
            NodeValue::Escaped => "escaped",
            NodeValue::Math(..) => "math",
            NodeValue::WikiLink(..) => "wikilink",
            NodeValue::Underline => "underline",
            NodeValue::Subscript => "subscript",
            NodeValue::SpoileredText => "spoiler",
            NodeValue::EscapedTag(_) => "escaped_tag",
        }
    }
}

/// A single node in the CommonMark AST.
///
/// The struct contains metadata about the node's position in the original document, and the core
/// enum, `NodeValue`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ast {
    /// The node value itself.
    pub value: NodeValue,

    /// The positions in the source document this node comes from.
    pub sourcepos: Sourcepos,
    pub(crate) internal_offset: usize,

    pub(crate) content: String,
    pub(crate) open: bool,
    pub(crate) last_line_blank: bool,
    pub(crate) table_visited: bool,
    pub(crate) line_offsets: Vec<usize>,
}

/// Represents the position in the source Markdown this node was rendered from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Sourcepos {
    /// The line and column of the first character of this node.
    pub start: LineColumn,
    /// The line and column of the last character of this node.
    pub end: LineColumn,
}

impl std::fmt::Display for Sourcepos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}-{}:{}",
            self.start.line, self.start.column, self.end.line, self.end.column,
        )
    }
}

impl From<(usize, usize, usize, usize)> for Sourcepos {
    fn from(sp: (usize, usize, usize, usize)) -> Sourcepos {
        Sourcepos {
            start: LineColumn {
                line: sp.0,
                column: sp.1,
            },
            end: LineColumn {
                line: sp.2,
                column: sp.3,
            },
        }
    }
}

/// Represents the 1-based line and column positions of a given character.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LineColumn {
    /// The 1-based line number of the character.
    pub line: usize,
    /// The 1-based column number of the character.
    pub column: usize,
}

impl From<(usize, usize)> for LineColumn {
    fn from(lc: (usize, usize)) -> LineColumn {
        LineColumn {
            line: lc.0,
            column: lc.1,
        }
    }
}

impl LineColumn {
    /// Return a new LineColumn based on this one, with the column adjusted by offset.
    pub fn column_add(&self, offset: isize) -> LineColumn {
        LineColumn {
            line: self.line,
            column: usize::try_from((self.column as isize) + offset).unwrap(),
        }
    }
}

impl Ast {
    /// Create a new AST node with the given value.
    pub fn new(value: NodeValue, start: LineColumn) -> Self {
        Ast {
            value,
            content: String::new(),
            sourcepos: (start.line, start.column, start.line, 0).into(),
            internal_offset: 0,
            open: true,
            last_line_blank: false,
            table_visited: false,
            line_offsets: Vec::with_capacity(0),
        }
    }
}

/// The type of a node within the document.
///
/// It is bound by the lifetime `'a`, which corresponds to the `Arena` nodes are
/// allocated in. Child `Ast`s are wrapped in `RefCell` for interior mutability.
///
/// You can construct a new `AstNode` from a `NodeValue` using the `From` trait:
///
/// ```no_run
/// # use comrak::nodes::{AstNode, NodeValue};
/// let root = AstNode::from(NodeValue::Document);
/// ```
///
/// Note that no sourcepos information is given to the created node. If you wish
/// to assign sourcepos information, use the `From` trait to create an `AstNode`
/// from an `Ast`:
///
/// ```no_run
/// # use comrak::nodes::{Ast, AstNode, NodeValue};
/// let root = AstNode::from(Ast::new(
///     NodeValue::Paragraph,
///     (4, 1).into(), // start_line, start_col
/// ));
/// ```
///
/// Adjust the `end` position manually.
///
/// For practical use, you'll probably need it allocated in an `Arena`, in which
/// case you can use `.into()` to simplify creation:
///
/// ```no_run
/// # use comrak::{nodes::{AstNode, NodeValue}, Arena};
/// # let arena = Arena::<AstNode>::new();
/// let node_in_arena = arena.alloc(NodeValue::Document.into());
/// ```
pub type AstNode<'a> = Node<'a, RefCell<Ast>>;

impl<'a> From<NodeValue> for AstNode<'a> {
    /// Create a new AST node with the given value. The sourcepos is set to (0,0)-(0,0).
    fn from(value: NodeValue) -> Self {
        Node::new(RefCell::new(Ast::new(value, LineColumn::default())))
    }
}

impl<'a> From<Ast> for AstNode<'a> {
    /// Create a new AST node with the given Ast.
    fn from(ast: Ast) -> Self {
        Node::new(RefCell::new(ast))
    }
}

/// Validation errors produced by [Node::validate].
#[derive(Debug, Clone)]
pub enum ValidationError<'a> {
    /// The type of a child node is not allowed in the parent node. This can happen when an inline
    /// node is found in a block container, a block is found in an inline node, etc.
    InvalidChildType {
        /// The parent node.
        parent: &'a AstNode<'a>,
        /// The child node.
        child: &'a AstNode<'a>,
    },
}

impl<'a> Node<'a, RefCell<Ast>> {
    /// The comrak representation of a markdown node in Rust isn't strict enough to rule out
    /// invalid trees according to the CommonMark specification. One simple example is that block
    /// containers, such as lists, should only contain blocks, but it's possible to put naked
    /// inline text in a list item. Such invalid trees can lead comrak to generate incorrect output
    /// if rendered.
    ///
    /// This method performs additional structural checks to ensure that a markdown AST is valid
    /// according to the CommonMark specification.
    ///
    /// Note that those invalid trees can only be generated programmatically. Parsing markdown with
    /// comrak, on the other hand, should always produce a valid tree.
    pub fn validate(&'a self) -> Result<(), ValidationError<'a>> {
        let mut stack = vec![self];

        while let Some(node) = stack.pop() {
            // Check that this node type is valid wrt to the type of its parent.
            if let Some(parent) = node.parent() {
                if !can_contain_type(parent, &node.data.borrow().value) {
                    return Err(ValidationError::InvalidChildType {
                        parent,
                        child: node,
                    });
                }
            }

            stack.extend(node.children());
        }

        Ok(())
    }
}

pub(crate) fn last_child_is_open<'a>(node: &'a AstNode<'a>) -> bool {
    node.last_child().map_or(false, |n| n.data.borrow().open)
}

/// Returns true if the given node can contain a node with the given value.
pub fn can_contain_type<'a>(node: &'a AstNode<'a>, child: &NodeValue) -> bool {
    match *child {
        NodeValue::Document => {
            return false;
        }
        NodeValue::FrontMatter(_) => {
            return matches!(node.data.borrow().value, NodeValue::Document);
        }
        _ => {}
    }

    match node.data.borrow().value {
        NodeValue::Document
        | NodeValue::BlockQuote
        | NodeValue::FootnoteDefinition(_)
        | NodeValue::DescriptionTerm
        | NodeValue::DescriptionDetails
        | NodeValue::Item(..)
        | NodeValue::TaskItem(..) => {
            child.block() && !matches!(*child, NodeValue::Item(..) | NodeValue::TaskItem(..))
        }

        NodeValue::List(..) => matches!(*child, NodeValue::Item(..) | NodeValue::TaskItem(..)),

        NodeValue::DescriptionList => matches!(*child, NodeValue::DescriptionItem(_)),

        NodeValue::DescriptionItem(_) => matches!(
            *child,
            NodeValue::DescriptionTerm | NodeValue::DescriptionDetails
        ),

        #[cfg(feature = "shortcodes")]
        NodeValue::ShortCode(..) => !child.block(),

        NodeValue::Paragraph
        | NodeValue::Heading(..)
        | NodeValue::Emph
        | NodeValue::Strong
        | NodeValue::Link(..)
        | NodeValue::Image(..)
        | NodeValue::WikiLink(..)
        | NodeValue::Strikethrough
        | NodeValue::Superscript
        | NodeValue::SpoileredText
        | NodeValue::Underline
        | NodeValue::Subscript
        // XXX: this is quite a hack: the EscapedTag _contains_ whatever was
        // possibly going to fall into the spoiler. This should be fixed in
        // inlines.
        | NodeValue::EscapedTag(_)
        => !child.block(),

        NodeValue::Table(..) => matches!(*child, NodeValue::TableRow(..)),

        NodeValue::TableRow(..) => matches!(*child, NodeValue::TableCell),

        #[cfg(not(feature = "shortcodes"))]
        NodeValue::TableCell => matches!(
            *child,
            NodeValue::Text(..)
                | NodeValue::Code(..)
                | NodeValue::Emph
                | NodeValue::Strong
                | NodeValue::Link(..)
                | NodeValue::Image(..)
                | NodeValue::Strikethrough
                | NodeValue::HtmlInline(..)
                | NodeValue::Math(..)
                | NodeValue::WikiLink(..)
                | NodeValue::FootnoteReference(..)
                | NodeValue::Superscript
                | NodeValue::SpoileredText
                | NodeValue::Underline
                | NodeValue::Subscript
        ),

        #[cfg(feature = "shortcodes")]
        NodeValue::TableCell => matches!(
            *child,
            NodeValue::Text(..)
            | NodeValue::Code(..)
            | NodeValue::Emph
            | NodeValue::Strong
            | NodeValue::Link(..)
            | NodeValue::Image(..)
            | NodeValue::Strikethrough
            | NodeValue::HtmlInline(..)
            | NodeValue::Math(..)
            | NodeValue::WikiLink(..)
            | NodeValue::FootnoteReference(..)
            | NodeValue::Superscript
            | NodeValue::SpoileredText
            | NodeValue::Underline
            | NodeValue::Subscript
            | NodeValue::ShortCode(..)
        ),

        NodeValue::MultilineBlockQuote(_) => {
            child.block() && !matches!(*child, NodeValue::Item(..) | NodeValue::TaskItem(..))
        }

        _ => false,
    }
}

pub(crate) fn ends_with_blank_line<'a>(node: &'a AstNode<'a>) -> bool {
    let mut it = Some(node);
    while let Some(cur) = it {
        if cur.data.borrow().last_line_blank {
            return true;
        }
        match cur.data.borrow().value {
            NodeValue::List(..) | NodeValue::Item(..) | NodeValue::TaskItem(..) => {
                it = cur.last_child()
            }
            _ => it = None,
        };
    }
    false
}

pub(crate) fn containing_block<'a>(node: &'a AstNode<'a>) -> Option<&'a AstNode<'a>> {
    let mut ch = Some(node);
    while let Some(n) = ch {
        if n.data.borrow().value.block() {
            return Some(n);
        }
        ch = n.parent();
    }
    None
}
