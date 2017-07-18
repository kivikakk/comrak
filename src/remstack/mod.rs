//! Provides a stack with O(1) random access and amortized O(1) random removal.

#[cfg(test)]
mod tests;

use std::cell::Cell;
use std::fmt::{Debug, Formatter, Result};
use std::ops::{Index, IndexMut};

enum Slot<T> {
    Occupied { val: T },
    Vacant { next: Cell<usize> },
}

impl<T> Debug for Slot<T> where T : Debug {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        match self {
            &Slot::Occupied { ref val } => write!(formatter, "{:?}", val),
            &Slot::Vacant { .. } => write!(formatter, "_"),
        }
    }
}

/// A stack with O(1) random access and amortized O(1) random removal,
/// pronounced 'removable stack'.
pub struct RemStack<T> {
    vec: Vec<Slot<T>>,
}

impl<T> Debug for RemStack<T> where T : Debug {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        write!(formatter, "RemStack {:?}", self.vec)
    }
}

impl<T> RemStack<T> {
    /// Constructs a new, empty `RemStack<T>`.
    pub fn new() -> RemStack<T> {
        RemStack { vec: Vec::new() }
    }

    fn prune_top(&mut self) {
        while let Some(&Slot::Vacant { .. }) = self.vec.last() {
            self.vec.pop();
        }
    }

    /// Appends an element to the back of a collection.
    pub fn push(&mut self, val: T) {
        self.prune_top();
        self.vec.push(Slot::Occupied { val: val })
    }

    /// Removes the last element from a removable stack and returns it, or
    /// `None` if it is empty.
    pub fn pop(&mut self) -> Option<T> {
        self.prune_top();
        self.vec.pop().map(|val| match val {
            Slot::Occupied { val } => val,
            _ => panic!("expected slot to be occupied"),
        })
    }

    /// Shortens the removable stack, keeping the first `len` slots.
    ///
    /// Note that slots are retained rather than elements; if the first `len`
    /// slots are vacant, no elements will remain after truncation.
    pub fn truncate(&mut self, len: usize) {
        self.vec.truncate(len);
        self.prune_top();
    }

    fn index_probe(&self, idx: usize) -> Option<usize> {
        if idx >= self.vec.len() {
            return None;
        }

        match self.vec[idx] {
            Slot::Occupied { .. } => Some(idx),
            Slot::Vacant { ref next } =>
                self.index_probe(next.get()).map(|idx| {
                    next.set(idx);
                    idx
                })
        }
    }

    /// Return the first element found in the removable stack from slot position
    /// `idx` onward, or `None` if out of bounds.
    pub fn get(&self, idx: usize) -> Option<&T> {
        self.index_probe(idx).map(|idx| match self.vec[idx] {
            Slot::Occupied { ref val } => val,
            _ => panic!("expected slot to be occupied"),
        })
    }

    /// Return a mutable reference to the first element found in the removable
    /// stack from slot position `idx` onward, or `None` if out of bounds.
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.index_probe(idx).map(move |idx| match self.vec[idx] {
            Slot::Occupied { ref mut val } => val,
            _ => panic!("expected slot to be occupied"),
        })
    }

    /// Remove the element found at slot position `idx` if any.
    pub fn remove(&mut self, idx: usize) {
        if let Some(idx) = self.index_probe(idx) {
            self.vec[idx] = Slot::Vacant { next: Cell::new(idx + 1) }
        }

        self.prune_top();
    }

    /// Return the number of slots in the removable stack's underlying vector.
    pub fn len(&self) -> usize {
        self.vec.len()
    }
}

impl<T> Index<usize> for RemStack<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.get(index).expect("out of bounds index")
    }
}

impl<T> IndexMut<usize> for RemStack<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.get_mut(index).expect("out of bounds index")
    }
}

impl<T> From<Vec<T>> for RemStack<T> {
    fn from(vec: Vec<T>) -> RemStack<T> {
        let mut rs: RemStack<T> = RemStack::new();
        for i in vec.into_iter() {
            rs.push(i);
        }
        rs
    }
}
