use std::collections::HashMap;
use std::ptr;

use crate::arena_tree::NodeEdge;
use crate::node_matches;
use crate::nodes::{Node, NodeValue};
use crate::parser::options::Options;

#[derive(Default)]
pub(crate) struct BlockIds {
    ids: HashMap<*const (), usize>,
    names: Vec<String>,
}

impl BlockIds {
    pub(crate) fn new(options: &Options<'_>, root: Node<'_>) -> Self {
        let mut block_ids = BlockIds::default();
        if !options.extension.ofm_block_ids
            || !root
                .descendants()
                .any(|node| node_matches!(node, NodeValue::BlockId(_)))
        {
            return block_ids;
        }

        let headings_excluded = options.extension.header_id_prefix.is_some();
        let mut markers = TrailingMarkers::default();
        let mut owned: HashMap<usize, usize> = HashMap::new();

        for ancestor in root.ancestors().skip(1) {
            if is_labelled_container(ancestor)
                && let Some(name) = markers.of(ancestor)
            {
                *owned.entry(name).or_default() += 1;
            }
        }

        for edge in root.traverse() {
            match edge {
                NodeEdge::Start(node) => {
                    let Some(name) = markers.of(node) else {
                        continue;
                    };
                    let excluded = headings_excluded && node_matches!(node, NodeValue::Heading(_));
                    if !excluded && !owned.contains_key(&name) {
                        block_ids.ids.insert(key(node), name);
                    }
                    if is_labelled_container(node) {
                        *owned.entry(name).or_default() += 1;
                    }
                }
                NodeEdge::End(node) => {
                    if is_labelled_container(node)
                        && let Some(name) = markers.of(node)
                        && let Some(count) = owned.get_mut(&name)
                    {
                        *count -= 1;
                        if *count == 0 {
                            owned.remove(&name);
                        }
                    }
                }
            }
        }

        block_ids.names = markers.names;
        block_ids
    }

    pub(crate) fn get(&self, node: Node<'_>) -> Option<&str> {
        self.ids
            .get(&key(node))
            .map(|&name| self.names[name].as_str())
    }
}

#[derive(Default)]
struct TrailingMarkers {
    memo: HashMap<*const (), Option<usize>>,
    names: Vec<String>,
    name_ixs: HashMap<String, usize>,
}

impl TrailingMarkers {
    fn of(&mut self, node: Node<'_>) -> Option<usize> {
        let mut chain = vec![];
        let mut current = node;
        let marker = loop {
            if let Some(&marker) = self.memo.get(&key(current)) {
                break marker;
            }
            chain.push(current);
            let Some(last) = current.last_child() else {
                break None;
            };
            match last.data().value {
                NodeValue::BlockId(ref n) => break Some(self.intern(&n.name)),
                NodeValue::List(_)
                | NodeValue::Item(_)
                | NodeValue::TaskItem(_)
                | NodeValue::DescriptionList
                | NodeValue::DescriptionItem(_) => break None,
                _ => current = last,
            }
        };
        for node in chain {
            self.memo.insert(key(node), marker);
        }
        marker
    }

    fn intern(&mut self, name: &str) -> usize {
        if let Some(&ix) = self.name_ixs.get(name) {
            return ix;
        }
        let ix = self.names.len();
        self.names.push(name.to_string());
        self.name_ixs.insert(name.to_string(), ix);
        ix
    }
}

fn is_labelled_container(node: Node<'_>) -> bool {
    node_matches!(
        node,
        NodeValue::BlockQuote
            | NodeValue::MultilineBlockQuote(_)
            | NodeValue::Table(_)
            | NodeValue::Callout(_)
            | NodeValue::Alert(_)
            | NodeValue::DescriptionTerm
            | NodeValue::DescriptionDetails
            | NodeValue::Subtext
            | NodeValue::BlockDirective(_)
            | NodeValue::Item(_)
            | NodeValue::TaskItem(_)
    )
}

fn key(node: Node<'_>) -> *const () {
    ptr::from_ref(node).cast()
}
