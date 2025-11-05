use std::collections::HashMap;

use crate::{
    BitSet, Handle, Index,
    store::{MaskStore, Store, VecStore},
};

pub type Handles = MaskStore<VecStore<Handle>>;

#[derive(Default)]
pub struct Entities {
    pub index_alloc: IndexAlloc,
    pub handle_alloc: HandleAlloc,
    pub index_map: IndexMap,
    pub handles: Handles,
}

impl Entities {
    pub fn allocate(&mut self) -> (Handle, Index) {
        let index = self.index_alloc.allocate();
        let handle = self.handle_alloc.allocate();
        self.index_map.insert(handle, index);
        self.handles.insert(index, handle);
        (handle, index)
    }

    pub fn free(&mut self, handle: Handle) -> Option<Index> {
        let Some(index) = self.index_map.remove(handle) else {
            return None;
        };
        self.handles.remove(index);
        self.index_alloc.free(index);
        Some(index)
    }

    pub fn get(&self, handle: Handle) -> Option<Index> {
        self.index_map.get(handle)
    }
}

#[derive(Default)]
pub struct IndexAlloc {
    freed: BitSet,
    next: Index,
}

impl IndexAlloc {
    pub fn allocate(&mut self) -> Index {
        let freed = self.freed.iter().next();
        if let Some(index) = freed {
            self.freed.remove(index);
            index
        } else {
            let index = self.next;
            self.next = self
                .next
                .checked_add(1)
                .expect("no entity left to allocate");
            index
        }
    }

    pub fn free(&mut self, index: Index) {
        self.freed.insert(index);
    }
}

#[derive(Default)]
pub struct HandleAlloc {
    next: Handle,
}

impl HandleAlloc {
    pub fn allocate(&mut self) -> Handle {
        let index = self.next;
        self.next = self
            .next
            .checked_add(1)
            .expect("no handle left to allocate");
        index
    }
}

#[derive(Default)]
pub struct IndexMap {
    indices: HashMap<Handle, Index>,
}

impl IndexMap {
    pub fn insert(&mut self, handle: Handle, index: Index) {
        self.indices.insert(handle, index);
    }

    pub fn remove(&mut self, handle: Handle) -> Option<Index> {
        self.indices.remove(&handle)
    }

    pub fn get(&self, handle: Handle) -> Option<Index> {
        self.indices.get(&handle).copied()
    }
}
