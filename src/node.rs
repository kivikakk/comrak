use std::cell::RefCell;
use std::fmt::{Debug, Formatter, Result};
use arena_tree::Node;

#[derive(Debug, Clone)]
pub enum NodeVal {
    Document,
    BlockQuote,
    List,
    Item,
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

#[derive(Default, Debug, Clone)]
pub struct NodeCodeBlock {
    pub fenced: bool,
    pub fence_char: u8,
    pub fence_length: usize,
    pub fence_offset: usize,
    pub info: String,
}

#[derive(Default, Debug, Clone)]
pub struct NodeHeading {
    pub level: u32,
    pub setext: bool,
}

impl NodeVal {
    pub fn block(&self) -> bool {
        match self {
            &NodeVal::Document | &NodeVal::BlockQuote | &NodeVal::List | &NodeVal::Item |
            &NodeVal::CodeBlock(..) | &NodeVal::HtmlBlock(..) | &NodeVal::CustomBlock |
            &NodeVal::Paragraph | &NodeVal::Heading(..) | &NodeVal::ThematicBreak => true,
            _ => false,
        }
    }

    pub fn accepts_lines(&self) -> bool {
        match self {
            &NodeVal::Paragraph | &NodeVal::Heading(..) | &NodeVal::CodeBlock(..) =>
                true,
            _ => false,
        }
    }

    pub fn contains_inlines(&self) -> bool {
        match self {
            &NodeVal::Paragraph | &NodeVal::Heading(..) => true,
            _ => false,
        }
    }

    pub fn text(&mut self) -> Option<&mut Vec<u8>> {
        match self {
            &mut NodeVal::Text(ref mut t) => Some(t),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct NI {
    pub typ: NodeVal,
    pub content: Vec<u8>,
    pub start_line: u32,
    pub start_column: usize,
    pub end_line: u32,
    pub end_column: usize,
    pub open: bool,
    pub last_line_blank: bool,
}

pub fn make_block(typ: NodeVal, start_line: u32, start_column: usize) -> NI {
    NI {
        typ: typ,
        content: vec![],
        start_line: start_line,
        start_column: start_column,
        end_line: start_line,
        end_column: 0,
        open: true,
        last_line_blank: false,
    }
}

pub type N = RefCell<NI>;

impl<'a> Node<'a, N> {
    pub fn last_child_is_open(&self) -> bool {
        self.last_child().map_or(false, |n| n.data.borrow().open)
    }

    pub fn can_contain_type(&self, child: &NodeVal) -> bool {
        if let &NodeVal::Document = child {
            return false;
        }

        match self.data.borrow().typ {
            NodeVal::Document | NodeVal::BlockQuote | NodeVal::Item =>
                child.block() && match child {
                    &NodeVal::Item => false,
                    _ => true,
                },

            NodeVal::List =>
                match child {
                    &NodeVal::Item => true,
                    _ => false,
                },

            NodeVal::CustomBlock => true,

            NodeVal::Paragraph | NodeVal::Heading(..) | NodeVal::Emph | NodeVal::Strong |
            NodeVal::Link | NodeVal::Image | NodeVal::CustomInline =>
                !child.block(),

            _ => false,
        }
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

