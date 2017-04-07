use std::cell::RefCell;
use arena_tree::Node;

#[derive(Debug, Clone)]
pub enum NodeValue {
    Document,
    BlockQuote,
    List(NodeList),
    Item(NodeList),
    CodeBlock(NodeCodeBlock),
    HtmlBlock(NodeHtmlBlock),
    CustomBlock,
    Paragraph,
    Heading(NodeHeading),
    ThematicBreak,
    Table(Vec<TableAlignment>),
    TableRow(bool),
    TableCell,

    Text(String),
    SoftBreak,
    LineBreak,
    Code(String),
    HtmlInline(String),
    CustomInline,
    Emph,
    Strong,
    Strikethrough,
    Link(NodeLink),
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

#[derive(Debug, Clone)]
pub struct Ast {
    pub value: NodeValue,
    pub content: String,
    pub start_line: u32,
    pub start_column: usize,
    pub end_line: u32,
    pub end_column: usize,
    pub open: bool,
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
