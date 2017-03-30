use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};
use arena_tree::Node;

#[derive(Debug, Clone)]
pub enum NodeValue {
    Document,
    BlockQuote,
    List(NodeList),
    Item(NodeList),
    CodeBlock(NodeCodeBlock),
    HtmlBlock(u8),
    CustomBlock,
    Paragraph,
    Heading(NodeHeading),
    ThematicBreak,

    Text(Vec<u8>),
    SoftBreak,
    LineBreak,
    Code,
    HtmlInline,
    CustomInline,
    Emph,
    Strong,
    Link,
    Image,
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
    pub literal: Vec<u8>,
}

#[derive(Default, Debug, Clone)]
pub struct NodeHeading {
    pub level: u32,
    pub setext: bool,
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
            &NodeValue::ThematicBreak => true,
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
            &NodeValue::Heading(..) => true,
            _ => false,
        }
    }

    pub fn text(&mut self) -> Option<&mut Vec<u8>> {
        match self {
            &mut NodeValue::Text(ref mut t) => Some(t),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Ast {
    pub value: NodeValue,
    pub content: Vec<u8>,
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
        content: vec![],
        start_line: start_line,
        start_column: start_column,
        end_line: start_line,
        end_column: 0,
        open: true,
        last_line_blank: false,
    }
}

pub type AstCell = RefCell<Ast>;

impl<'a> Node<'a, AstCell> {
    pub fn last_child_is_open(&self) -> bool {
        self.last_child().map_or(false, |n| n.data.borrow().open)
    }

    pub fn can_contain_type(&self, child: &NodeValue) -> bool {
        if let &NodeValue::Document = child {
            return false;
        }

        match self.data.borrow().value {
            NodeValue::Document | NodeValue::BlockQuote | NodeValue::Item(..) => {
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
            NodeValue::Link |
            NodeValue::Image |
            NodeValue::CustomInline => !child.block(),

            _ => false,
        }
    }

    pub fn ends_with_blank_line(&self) -> bool {
        let mut it = Some(self);
        while let Some(cur) = it {
            if cur.data.borrow().last_line_blank {
                return true;
            }
            match &cur.data.borrow().value {
                &NodeValue::List(..) | &NodeValue::Item(..) =>
                    it = cur.last_child(),
                _ => it = None,
            };
        }
        false
    }
}

impl<'a, T: Debug> Debug for Node<'a, RefCell<T>> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut ch = vec![];
        let mut c = self.first_child();
        while let Some(e) = c {
            ch.push(e);
            c = e.next_sibling();
        }
        write!(f, "[({:?}) {} children: {{", self.data.borrow(), ch.len())?;
        let mut first = true;
        for e in &ch {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", e)?;
        }
        write!(f, "}}]")?;
        Ok(())
    }
}
