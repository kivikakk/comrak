use std::ptr;

use crate::nodes::Node;

#[derive(Default)]
pub(crate) struct TableCellIndexer {
    last: Option<(*const (), usize)>,
}

impl TableCellIndexer {
    pub(crate) fn index(&mut self, node: Node<'_>) -> usize {
        let mut ix = 0;
        let mut cur = node;
        while let Some(prev) = cur.previous_sibling() {
            ix += 1;
            if let Some((last, last_ix)) = self.last
                && Self::key(prev) == last
            {
                ix += last_ix;
                break;
            }
            cur = prev;
        }
        self.last = Some((Self::key(node), ix));
        ix
    }

    fn key(node: Node<'_>) -> *const () {
        ptr::from_ref(node).cast()
    }
}
