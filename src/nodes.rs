use std::cell::RefCell;
use arena_tree::Node;

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

    /// **Block**. TODO.
    CustomBlock,

    /// **Block**. A [paragraph](https://github.github.com/gfm/#paragraphs).  Contains **inlines**.
    Paragraph,

    /// **Block**. A heading; may be an [ATX heading](https://github.github.com/gfm/#atx-headings)
    /// or a [setext heading](https://github.github.com/gfm/#setext-headings). Contains
    /// **inlines**.
    Heading(NodeHeading),

    /// **Block**. A [thematic break](https://github.github.com/gfm/#thematic-breaks).  Has no
    /// children.
    ThematicBreak,

    /// **Block**. A [table](https://github.github.com/gfm/#tables-extension-) per the GFM spec.
    /// Contains table rows.
    Table(Vec<TableAlignment>),

    /// **Block**. A table row.  The `bool` represents whether the row is the header row or not.
    /// Contains table cells.
    TableRow(bool),

    /// **Block**.  A table cell.  Contains **inlines**.
    TableCell,

    /// **Inline**
    Text(String),
    /// **Inline**
    SoftBreak,
    /// **Inline**
    LineBreak,
    /// **Inline**
    Code(String),
    /// **Inline**
    HtmlInline(String),
    /// **Inline**
    CustomInline,
    /// **Inline**
    Emph,
    /// **Inline**
    Strong,
    /// **Inline**
    Strikethrough,
    /// **Inline**
    Link(NodeLink),
    /// **Inline**
    Image(NodeLink),
}

#[derive(Debug, Clone)]
pub enum TableAlignment {
    None,
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone)]
pub struct NodeLink {
    pub url: String,
    pub title: String,
}

/// Hi
#[derive(Debug, Default, Clone, Copy)]
pub struct NodeList {
    pub list_type: ListType,
    pub marker_offset: usize,
    pub padding: usize,
    pub start: usize,
    pub delimiter: ListDelimType,
    pub bullet_char: u8,
    pub tight: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListType {
    None,
    Bullet,
    Ordered,
}

impl Default for ListType {
    fn default() -> ListType {
        ListType::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListDelimType {
    None,
    Period,
    Paren,
}

impl Default for ListDelimType {
    fn default() -> ListDelimType {
        ListDelimType::None
    }
}

#[derive(Default, Debug, Clone)]
pub struct NodeCodeBlock {
    pub fenced: bool,
    pub fence_char: u8,
    pub fence_length: usize,
    pub fence_offset: usize,
    pub info: String,
    pub literal: String,
}

#[derive(Default, Debug, Clone)]
pub struct NodeHeading {
    pub level: u32,
    pub setext: bool,
}

#[derive(Debug, Clone)]
pub struct NodeHtmlBlock {
    pub block_type: u8,
    pub literal: String,
}


impl NodeValue {
    pub fn block(&self) -> bool {
        match self {
            &NodeValue::Document |
            &NodeValue::BlockQuote |
            &NodeValue::List(..) |
            &NodeValue::Item(..) |
            &NodeValue::CodeBlock(..) |
            &NodeValue::HtmlBlock(..) |
            &NodeValue::CustomBlock |
            &NodeValue::Paragraph |
            &NodeValue::Heading(..) |
            &NodeValue::ThematicBreak |
            &NodeValue::Table(..) |
            &NodeValue::TableRow(..) |
            &NodeValue::TableCell => true,
            _ => false,
        }
    }

    pub fn accepts_lines(&self) -> bool {
        match self {
            &NodeValue::Paragraph |
            &NodeValue::Heading(..) |
            &NodeValue::CodeBlock(..) => true,
            _ => false,
        }
    }

    pub fn contains_inlines(&self) -> bool {
        match self {
            &NodeValue::Paragraph |
            &NodeValue::Heading(..) |
            &NodeValue::TableCell => true,
            _ => false,
        }
    }

    pub fn text(&mut self) -> Option<&mut String> {
        match self {
            &mut NodeValue::Text(ref mut t) => Some(t),
            _ => None,
        }
    }
}

/// A single node in the CommonMark AST.  The struct contains metadata about the node's position in
/// the original document, and the core enum, `NodeValue`.
#[derive(Debug, Clone)]
pub struct Ast {
    /// The node value itself.
    pub value: NodeValue,

    /// The line in the input document the node starts at.
    pub start_line: u32,

    /// The column in the input document the node starts at.
    pub start_column: usize,

    /// The line in the input document the node ends at.
    pub end_line: u32,

    /// The column in the input document the node ends at.
    pub end_column: usize,

    #[doc(hidden)]
    pub content: String,
    #[doc(hidden)]
    pub open: bool,
    #[doc(hidden)]
    pub last_line_blank: bool,
}

pub fn make_block(value: NodeValue, start_line: u32, start_column: usize) -> Ast {
    Ast {
        value: value,
        content: String::new(),
        start_line: start_line,
        start_column: start_column,
        end_line: start_line,
        end_column: 0,
        open: true,
        last_line_blank: false,
    }
}

pub type AstNode<'a> = Node<'a, RefCell<Ast>>;

pub fn last_child_is_open<'a>(node: &'a AstNode<'a>) -> bool {
    node.last_child().map_or(false, |n| n.data.borrow().open)
}

pub fn can_contain_type<'a>(node: &'a AstNode<'a>, child: &NodeValue) -> bool {
    if let &NodeValue::Document = child {
        return false;
    }

    match node.data.borrow().value {
        NodeValue::Document |
        NodeValue::BlockQuote |
        NodeValue::Item(..) => {
            child.block() &&
            match child {
                &NodeValue::Item(..) => false,
                _ => true,
            }
        }

        NodeValue::List(..) => {
            match child {
                &NodeValue::Item(..) => true,
                _ => false,
            }
        }

        NodeValue::CustomBlock => true,

        NodeValue::Paragraph |
        NodeValue::Heading(..) |
        NodeValue::Emph |
        NodeValue::Strong |
        NodeValue::Link(..) |
        NodeValue::Image(..) |
        NodeValue::CustomInline => !child.block(),

        NodeValue::Table(..) => {
            match child {
                &NodeValue::TableRow(..) => true,
                _ => false,
            }
        }

        NodeValue::TableRow(..) => {
            match child {
                &NodeValue::TableCell => true,
                _ => false,
            }
        }

        NodeValue::TableCell => {
            match child {
                &NodeValue::Text(..) |
                &NodeValue::Code(..) |
                &NodeValue::Emph |
                &NodeValue::Strong |
                &NodeValue::Link(..) |
                &NodeValue::Image(..) |
                &NodeValue::Strikethrough |
                &NodeValue::HtmlInline(..) => true,
                _ => false,
            }
        }

        _ => false,
    }
}

pub fn ends_with_blank_line<'a>(node: &'a AstNode<'a>) -> bool {
    let mut it = Some(node);
    while let Some(cur) = it {
        if cur.data.borrow().last_line_blank {
            return true;
        }
        match &cur.data.borrow().value {
            &NodeValue::List(..) |
            &NodeValue::Item(..) => it = cur.last_child(),
            _ => it = None,
        };
    }
    false
}

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
