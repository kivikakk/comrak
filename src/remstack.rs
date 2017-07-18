use std::cell::Cell;
use std::ops::{Index, IndexMut};

enum Slot<T> {
    Occupied { val: T },
    Vacant { next: Cell<usize> },
}

pub struct RemStack<T> {
    vec: Vec<Slot<T>>,
}

impl<T> RemStack<T> {
    pub fn new() -> RemStack<T> {
        RemStack { vec: Vec::new() }
    }

    fn prune_top(&mut self) {
        while let Some(&Slot::Vacant { .. }) = self.vec.last() {
            self.vec.pop();
        }
    }

    pub fn push(&mut self, val: T) {
        self.prune_top();
        self.vec.push(Slot::Occupied { val: val })
    }

    pub fn pop(&mut self) -> Option<T> {
        self.prune_top();
        self.vec.pop().map(|val| match val {
            Slot::Occupied { val } => val,
            _ => panic!("expected slot to be occupied"),
        })
    }

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

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.index_probe(idx).map(|idx| match self.vec[idx] {
            Slot::Occupied { ref val } => val,
            _ => panic!("expected slot to be occupied"),
        })
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.index_probe(idx).map(move |idx| match self.vec[idx] {
            Slot::Occupied { ref mut val } => val,
            _ => panic!("expected slot to be occupied"),
        })
    }

    pub fn remove(&mut self, idx: usize) {
        if let Some(idx) = self.index_probe(idx) {
            self.vec[idx] = Slot::Vacant { next: Cell::new(idx + 1) }
        }

        self.prune_top();
    }

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
