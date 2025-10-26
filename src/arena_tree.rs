//! A DOM-like tree data structure based on XXX.
//!
//! Based on <https://github.com/SimonSapin/rust-forest/blob/5783c8be8680b84c0438638bdee07d4e4aca40ac/arena-tree/lib.rs>.
//! MIT license (per Cargo.toml).

use std::cell::Cell;
use std::fmt;

#[derive(Hash, PartialOrd, Ord, Debug)]
/// XXX
pub struct Id<T>(pub id_arena::Id<Node<T>>);

impl<T> Copy for Id<T> {}

impl<T> Clone for Id<T> {
    #[inline]
    fn clone(&self) -> Id<T> {
        *self
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> From<id_arena::Id<Node<T>>> for Id<T> {
    fn from(value: id_arena::Id<Node<T>>) -> Self {
        Self(value)
    }
}

/// A node inside a DOM-like tree.
pub struct Node<T> {
    parent: Cell<Option<Id<T>>>,
    previous_sibling: Cell<Option<Id<T>>>,
    next_sibling: Cell<Option<Id<T>>>,
    first_child: Cell<Option<Id<T>>>,
    last_child: Cell<Option<Id<T>>>,

    /// The data held by the node.
    pub data: T,
}

/// XXX
pub type Arena<T> = id_arena::Arena<Node<T>>;

/// A simple Debug implementation that prints the children as a tree, without
/// looping through the various interior pointer cycles.
impl<T> fmt::Debug for Node<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        // struct Children<'a, T>(Option<(&'a Arena<T>, Id<Node<T>>)>);
        // impl<T: fmt::Debug> fmt::Debug for Children<'_, T> {
        //     fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        //         f.debug_list()
        //             .entries(std::iter::successors(self.0, |child| {
        //                 child.next_sibling.get()
        //             }))
        //             .finish()
        //     }
        // }

        let mut struct_fmt = f.debug_struct("Node");
        struct_fmt.field("data", &self.data);
        // struct_fmt.field("children", &Children(self.first_child.get()));
        struct_fmt.finish()?;

        Ok(())
    }
}

impl<T> Node<T> {
    /// Create a new node from its associated data.
    ///
    /// Typically, this node needs to be moved into an arena allocator
    /// before it can be used in a tree.
    pub fn new(data: T) -> Node<T> {
        Node {
            parent: Cell::new(None),
            first_child: Cell::new(None),
            last_child: Cell::new(None),
            previous_sibling: Cell::new(None),
            next_sibling: Cell::new(None),
            data,
        }
    }

    /// Return a reference to the parent node, unless this node is the root of the tree.
    pub fn parent(&self) -> Option<Id<T>> {
        self.parent.get()
    }

    /// Return a reference to the first child of this node, unless it has no child.
    pub fn first_child(&self) -> Option<Id<T>> {
        self.first_child.get()
    }

    /// Return a reference to the last child of this node, unless it has no child.
    pub fn last_child(&self) -> Option<Id<T>> {
        self.last_child.get()
    }

    /// Return a reference to the previous sibling of this node, unless it is a first child.
    pub fn previous_sibling(&self) -> Option<Id<T>> {
        self.previous_sibling.get()
    }

    /// Return a reference to the next sibling of this node, unless it is a last child.
    pub fn next_sibling(&self) -> Option<Id<T>> {
        self.next_sibling.get()
    }

    /// Detach a node from its parent and siblings. Children are not affected.
    pub fn detach(&self, arena: &Arena<T>) {
        let parent = self.parent.take().map(|i| &arena[i.0]);
        let previous_sibling = self.previous_sibling.take();
        let next_sibling = self.next_sibling.take();

        if let Some(next_sibling) = next_sibling {
            arena[next_sibling.0].previous_sibling.set(previous_sibling);
        } else if let Some(parent) = parent {
            parent.last_child.set(previous_sibling);
        }

        if let Some(previous_sibling) = previous_sibling {
            arena[previous_sibling.0].next_sibling.set(next_sibling);
        } else if let Some(parent) = parent {
            parent.first_child.set(next_sibling);
        }
    }
}

impl<T> Id<T> {
    /// Return a reference to the parent node, unless this node is the root of the tree.
    pub fn parent(self, arena: &Arena<T>) -> Option<Id<T>> {
        arena[self.0].parent.get()
    }

    /// Return a reference to the first child of this node, unless it has no child.
    pub fn first_child(self, arena: &Arena<T>) -> Option<Id<T>> {
        arena[self.0].first_child.get()
    }

    /// Return a reference to the last child of this node, unless it has no child.
    pub fn last_child(self, arena: &Arena<T>) -> Option<Id<T>> {
        arena[self.0].last_child.get()
    }

    /// Return a reference to the previous sibling of this node, unless it is a first child.
    pub fn previous_sibling(self, arena: &Arena<T>) -> Option<Id<T>> {
        arena[self.0].previous_sibling.get()
    }

    /// Return a reference to the next sibling of this node, unless it is a last child.
    pub fn next_sibling(self, arena: &Arena<T>) -> Option<Id<T>> {
        arena[self.0].next_sibling.get()
    }

    /// Return an iterator of references to this node and its ancestors.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn ancestors(self, arena: &Arena<T>) -> Ancestors<'_, T> {
        Ancestors(Some((arena, self)))
    }

    /// Return an iterator of references to this node and the siblings before it.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn preceding_siblings(self, arena: &Arena<T>) -> PrecedingSiblings<'_, T> {
        PrecedingSiblings(Some((arena, self)))
    }

    /// Return an iterator of references to this node and the siblings after it.
    ///
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    pub fn following_siblings(self, arena: &Arena<T>) -> FollowingSiblings<'_, T> {
        FollowingSiblings(Some((arena, self)))
    }

    /// Return an iterator of references to this node’s children.
    pub fn children(self, arena: &Arena<T>) -> Children<'_, T> {
        Children(arena[self.0].first_child.get().map(|r| (arena, r)))
    }

    /// XXX
    pub fn children_free(self, arena: &Arena<T>) -> ChildrenFree<T> {
        ChildrenFree(arena[self.0].first_child.get())
    }

    /// Return an iterator of references to this node’s children, in reverse order.
    pub fn reverse_children(self, arena: &Arena<T>) -> ReverseChildren<'_, T> {
        ReverseChildren(arena[self.0].last_child.get().map(|r| (arena, r)))
    }

    /// Return an iterator of references to this `Node` and its descendants, in tree order.
    ///
    /// Parent nodes appear before the descendants.
    /// Call `.next().unwrap()` once on the iterator to skip the node itself.
    ///
    /// *Similar Functions:* Use `traverse()` or `reverse_traverse` if you need
    /// references to the `NodeEdge` structs associated with each `Node`
    pub fn descendants(self, arena: &Arena<T>) -> Descendants<'_, T> {
        Descendants(self.traverse(arena))
    }

    /// XXX
    pub fn descendants_free(self) -> DescendantsFree<T> {
        DescendantsFree(self.traverse_free())
    }

    /// Return an iterator of references to `NodeEdge` enums for each `Node` and its descendants,
    /// in tree order.
    ///
    /// `NodeEdge` enums represent the `Start` or `End` of each node.
    ///
    /// *Similar Functions:* Use `descendants()` if you don't need `Start` and `End`.
    pub fn traverse(self, arena: &Arena<T>) -> Traverse<'_, T> {
        Traverse {
            arena,
            root: self,
            next: Some(NodeEdge::Start(self)),
        }
    }

    /// XXX
    pub fn traverse_free(self) -> TraverseFree<T> {
        TraverseFree {
            root: self,
            next: Some(NodeEdge::Start(self)),
        }
    }

    /// Return an iterator of references to `NodeEdge` enums for each `Node` and its descendants,
    /// in *reverse* order.
    ///
    /// `NodeEdge` enums represent the `Start` or `End` of each node.
    ///
    /// *Similar Functions:* Use `descendants()` if you don't need `Start` and `End`.
    pub fn reverse_traverse(self, arena: &Arena<T>) -> ReverseTraverse<'_, T> {
        ReverseTraverse {
            arena,
            root: self,
            next: Some(NodeEdge::End(self)),
        }
    }

    /// Detach a node from its parent and siblings. Children are not affected.
    #[inline]
    pub fn detach(self, arena: &Arena<T>) {
        arena[self.0].detach(arena);
    }

    /// Append a new child to this node, after existing children.
    pub fn append(self, arena: &Arena<T>, new_child: Id<T>) {
        let node = &arena[self.0];
        let new_child_node = &arena[new_child.0];
        new_child_node.detach(arena);

        new_child_node.parent.set(Some(self));
        if let Some(last_child) = node.last_child.take() {
            let last_child_node = &arena[last_child.0];
            debug_assert!(last_child_node.next_sibling.get().is_none());
            new_child_node.previous_sibling.set(Some(last_child));
            last_child_node.next_sibling.set(Some(new_child));
        } else {
            debug_assert!(node.first_child.get().is_none());
            node.first_child.set(Some(new_child));
        }
        node.last_child.set(Some(new_child));
    }

    /// Prepend a new child to this node, before existing children.
    pub fn prepend(self, arena: &Arena<T>, new_child: Id<T>) {
        let node = &arena[self.0];
        let new_child_node = &arena[new_child.0];
        new_child_node.detach(arena);

        new_child_node.parent.set(Some(self));
        if let Some(first_child) = node.first_child.take() {
            let first_child_node = &arena[first_child.0];
            debug_assert!(first_child_node.previous_sibling.get().is_none());
            first_child_node.previous_sibling.set(Some(new_child));
            new_child_node.next_sibling.set(Some(first_child));
        } else {
            debug_assert!(node.first_child.get().is_none());
            node.last_child.set(Some(new_child));
        }
        node.first_child.set(Some(new_child));
    }

    /// Insert a new sibling after this node.
    pub fn insert_after(self, arena: &Arena<T>, new_sibling: Id<T>) {
        let node = &arena[self.0];
        let new_sibling_node = &arena[new_sibling.0];
        new_sibling_node.detach(arena);

        new_sibling_node.parent.set(node.parent.get());
        new_sibling_node.previous_sibling.set(Some(self));
        if let Some(next_sibling) = node.next_sibling.take() {
            let next_sibling_node = &arena[next_sibling.0];
            debug_assert!(next_sibling_node.previous_sibling.get().unwrap() == self);
            next_sibling_node.previous_sibling.set(Some(new_sibling));
            new_sibling_node.next_sibling.set(Some(next_sibling));
        } else if let Some(parent) = node.parent.get() {
            let parent_node = &arena[parent.0];
            debug_assert!(parent_node.last_child.get().unwrap() == self);
            parent_node.last_child.set(Some(new_sibling));
        }
        node.next_sibling.set(Some(new_sibling));
    }

    /// Insert a new sibling before this node.
    pub fn insert_before(self, arena: &Arena<T>, new_sibling: Id<T>) {
        let node = &arena[self.0];
        let new_sibling_node = &arena[new_sibling.0];
        new_sibling_node.detach(arena);

        new_sibling_node.parent.set(node.parent.get());
        new_sibling_node.next_sibling.set(Some(self));
        if let Some(previous_sibling) = node.previous_sibling.take() {
            let previous_sibling_node = &arena[previous_sibling.0];
            debug_assert!(previous_sibling_node.next_sibling.get().unwrap() == self);
            new_sibling_node
                .previous_sibling
                .set(Some(previous_sibling));
            previous_sibling_node.next_sibling.set(Some(new_sibling));
        } else if let Some(parent) = node.parent.get() {
            let parent_node = &arena[parent.0];
            debug_assert!(parent_node.first_child.get().unwrap() == self);
            parent_node.first_child.set(Some(new_sibling));
        }
        node.previous_sibling.set(Some(new_sibling));
    }
}

macro_rules! axis_iterator {
    (#[$attr:meta] $name:ident, $free:ident : $next:ident) => {
        #[$attr]
        #[derive(Debug)]
        pub struct $name<'a, T>(Option<(&'a Arena<T>, Id<T>)>);

        impl<'a, T> Iterator for $name<'a, T> {
            type Item = Id<T>;

            fn next(&mut self) -> Option<Id<T>> {
                match self.0.take() {
                    Some((arena, id)) => {
                        let node = &arena[id.0];
                        self.0 = node.$next.get().map(|r| (arena, r));
                        Some(id)
                    }
                    None => None,
                }
            }
        }

        #[$attr]
        #[derive(Debug)]
        pub struct $free<T>(Option<Id<T>>);

        impl<T> $free<T> {
            /// XXX
            pub fn next(&mut self, arena: &Arena<T>) -> Option<Id<T>> {
                match self.0.take() {
                    Some(id) => {
                        let node = &arena[id.0];
                        self.0 = node.$next.get();
                        Some(id)
                    }
                    None => None,
                }
            }
        }
    };
}

axis_iterator! {
    #[doc = "An iterator of references to the ancestors a given node."]
    Ancestors, AncestorsFree: parent
}

axis_iterator! {
    #[doc = "An iterator of references to the siblings before a given node."]
    PrecedingSiblings, PreviousSiblingsFree: previous_sibling
}

axis_iterator! {
    #[doc = "An iterator of references to the siblings after a given node."]
    FollowingSiblings, FollowingSiblingsFree: next_sibling
}

axis_iterator! {
    #[doc = "An iterator of references to the children of a given node."]
    Children, ChildrenFree: next_sibling
}

axis_iterator! {
    #[doc = "An iterator of references to the children of a given node, in reverse order."]
    ReverseChildren, ReverseChildrenFree: previous_sibling
}

/// An iterator of references to a given node and its descendants, in tree order.
#[derive(Debug)]
pub struct Descendants<'a, T: 'a>(Traverse<'a, T>);

impl<'a, T> Iterator for Descendants<'a, T> {
    type Item = Id<T>;

    fn next(&mut self) -> Option<Id<T>> {
        loop {
            match self.0.next() {
                Some(NodeEdge::Start(node)) => return Some(node),
                Some(NodeEdge::End(_)) => {}
                None => return None,
            }
        }
    }
}

/// XXX
#[derive(Debug)]
pub struct DescendantsFree<T>(TraverseFree<T>);

impl<T> DescendantsFree<T> {
    /// XXX
    pub fn next(&mut self, arena: &Arena<T>) -> Option<Id<T>> {
        loop {
            match self.0.next(arena) {
                Some(NodeEdge::Start(node)) => return Some(node),
                Some(NodeEdge::End(_)) => {}
                None => return None,
            }
        }
    }
}

#[derive(Debug, Clone)]
/// Indicator if the node is at a start or endpoint of the tree
pub enum NodeEdge<T> {
    /// Indicates that start of a node that has children.
    /// Yielded by `Traverse::next` before the node’s descendants.
    /// In HTML or XML, this corresponds to an opening tag like `<div>`
    Start(T),

    /// Indicates that end of a node that has children.
    /// Yielded by `Traverse::next` after the node’s descendants.
    /// In HTML or XML, this corresponds to a closing tag like `</div>`
    End(T),
}

macro_rules! traverse_iterator {
    (#[$attr:meta] $name:ident, $free:ident : $first_child:ident, $next_sibling:ident) => {
        #[$attr]
        #[derive(Debug)]
        pub struct $name<'a, T: 'a> {
            arena: &'a Arena<T>,
            root: Id<T>,
            next: Option<NodeEdge<Id<T>>>,
        }

        impl<'a, T> Iterator for $name<'a, T> {
            type Item = NodeEdge<Id<T>>;

            fn next(&mut self) -> Option<NodeEdge<Id<T>>> {
                match self.next.take() {
                    Some(item) => {
                        self.next = match item {
                            NodeEdge::Start(id) => match self.arena[id.0].$first_child.get() {
                                Some(child) => Some(NodeEdge::Start(child)),
                                None => Some(NodeEdge::End(id)),
                            },
                            NodeEdge::End(id) => {
                                if id == self.root {
                                    None
                                } else {
                                    match self.arena[id.0].$next_sibling.get() {
                                        Some(sibling) => Some(NodeEdge::Start(sibling)),
                                        None => match self.arena[id.0].parent.get() {
                                            Some(parent) => Some(NodeEdge::End(parent)),
                                            None => panic!("tree modified during iteration"),
                                        },
                                    }
                                }
                            }
                        };
                        Some(item)
                    }
                    None => None,
                }
            }
        }

        #[$attr]
        #[derive(Debug)]
        #[allow(dead_code)]
        pub struct $free<T> {
            root: Id<T>,
            next: Option<NodeEdge<Id<T>>>,
        }

        #[allow(dead_code)]
        impl<T> $free<T> {
            fn next(&mut self, arena: &Arena<T>) -> Option<NodeEdge<Id<T>>> {
                match self.next.take() {
                    Some(item) => {
                        self.next = match item {
                            NodeEdge::Start(id) => match arena[id.0].$first_child.get() {
                                Some(child) => Some(NodeEdge::Start(child)),
                                None => Some(NodeEdge::End(id)),
                            },
                            NodeEdge::End(id) => {
                                if id == self.root {
                                    None
                                } else {
                                    match arena[id.0].$next_sibling.get() {
                                        Some(sibling) => Some(NodeEdge::Start(sibling)),
                                        None => match arena[id.0].parent.get() {
                                            Some(parent) => Some(NodeEdge::End(parent)),
                                            None => panic!("tree modified during iteration"),
                                        },
                                    }
                                }
                            }
                        };
                        Some(item)
                    }
                    None => None,
                }
            }
        }
    };
}

traverse_iterator! {
    #[doc = "An iterator of the start and end edges of a given
    node and its descendants, in tree order."]
    Traverse, TraverseFree: first_child, next_sibling
}

traverse_iterator! {
    #[doc = "An iterator of the start and end edges of a given
    node and its descendants, in reverse tree order."]
    ReverseTraverse, ReverseTraverseFree: last_child, previous_sibling
}

impl<T> Node<T> {
    /// XXX
    #[inline]
    pub fn data(&self) -> &T {
        &self.data
    }

    /// XXX
    #[inline]
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T> Id<T> {
    /// XXX
    #[inline]
    pub fn data(self, arena: &Arena<T>) -> &T {
        arena[self.0].data()
    }

    /// XXX
    #[inline]
    pub fn data_mut(self, arena: &mut Arena<T>) -> &mut T {
        arena[self.0].data_mut()
    }
}
