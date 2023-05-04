//! The CommonMark AST.

use crate::arena_tree::Node;
use std::cell::RefCell;
use std::convert::TryFrom;

#[cfg(feature = "shortcodes")]
use crate::parser::shortcodes::NodeShortCode;

/// The core AST node enum.
#[derive(Debug, Clone)]
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
    FootnoteDefinition(String),

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
    Text(String),

    /// **Inline**. [Task list item](https://github.github.com/gfm/#task-list-items-extension-).
    /// The value is the symbol that was used in the brackets to mark a task item as checked, or
    /// None if the item is unchecked.
    TaskItem(Option<char>),

    /// **Inline**.  A [soft line break](https://github.github.com/gfm/#soft-line-breaks).  If
    /// the `hardbreaks` option is set in `ComrakOptions` during formatting, it will be formatted
    /// as a `LineBreak`.
    SoftBreak,

    /// **Inline**.  A [hard line break](https://github.github.com/gfm/#hard-line-breaks).
    LineBreak,

    /// **Inline**.  A [code span](https://github.github.com/gfm/#code-spans).
    Code(NodeCode),

    /// **Inline**.  [Raw HTML](https://github.github.com/gfm/#raw-html) contained inline.
    HtmlInline(String),

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

    /// **Inline**.  A footnote reference.
    FootnoteReference(NodeFootnoteReference),

    #[cfg(feature = "shortcodes")]
    /// **Inline**. An Emoji character generated from a shortcode. Enable with feature "shortcodes".
    ShortCode(NodeShortCode),
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

/// An inline [code span](https://github.github.com/gfm/#code-spans).
#[derive(Debug, Clone)]
pub struct NodeCode {
    /// The URL for the link destination or image source.
    pub num_backticks: usize,

    /// The content of the inline code span.
    /// As the contents are not interpreted as Markdown at all,
    /// they are contained within this structure,
    /// rather than inserted into a child inline of any kind.
    pub literal: String,
}

/// The details of a link's destination, or an image's source.
#[derive(Debug, Clone)]
pub struct NodeLink {
    /// The URL for the link destination or image source.
    pub url: String,

    /// The title for the link or image.
    ///
    /// Note this field is used for the `title` attribute by the HTML formatter even for images;
    /// `alt` text is supplied in the image inline text.
    pub title: String,
}

/// The metadata of a list; the kind of list, the delimiter used and so on.
#[derive(Debug, Default, Clone, Copy)]
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
}

/// The metadata of a description list
#[derive(Debug, Default, Clone, Copy)]
pub struct NodeDescriptionItem {
    /// Number of spaces before the list marker.
    pub marker_offset: usize,

    /// Number of characters between the start of the list marker and the item text (including the list marker(s)).
    pub padding: usize,
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

impl ListDelimType {
    pub(crate) fn xml_name(&self) -> &'static str {
        match *self {
            ListDelimType::Period => "period",
            ListDelimType::Paren => "paren",
        }
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
#[derive(Default, Debug, Clone, Copy)]
pub struct NodeHeading {
    /// The level of the header; from 1 to 6 for ATX headings, 1 or 2 for setext headings.
    pub level: u8,

    /// Whether the heading is setext (if not, ATX).
    pub setext: bool,
}

/// The metadata of an included HTML block.
#[derive(Debug, Default, Clone)]
pub struct NodeHtmlBlock {
    /// The HTML block's type
    pub block_type: u8,

    /// The literal contents of the HTML block.  Per NodeCodeBlock, the content is included here
    /// rather than in any inline.
    pub literal: String,
}

/// The metadata of a footnote reference.
#[derive(Debug, Default, Clone)]
pub struct NodeFootnoteReference {
    /// The name of the footnote.
    pub name: String,

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

    /// The positions in the source document this node comes from.
    pub sourcepos: Sourcepos,
    pub(crate) internal_offset: usize,

    pub(crate) content: String,
    pub(crate) open: bool,
    pub(crate) last_line_blank: bool,
    pub(crate) table_visited: bool,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
        }
    }
}

/// The type of a node within the document.
///
/// It is bound by the lifetime `'a`, which corresponds to the `Arena` nodes are allocated in.
/// Child `Ast`s are wrapped in `RefCell` for interior mutability.
pub type AstNode<'a> = Node<'a, RefCell<Ast>>;

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
        | NodeValue::Image(..) => !child.block(),

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
                | NodeValue::ShortCode(..)
                | NodeValue::Strikethrough
                | NodeValue::HtmlInline(..)
        ),

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
