use std::{
    cell::UnsafeCell,
    mem::{self, MaybeUninit},
};

use crate::{BitSet, Index};

pub trait Store {
    type Item;

    fn mask(&self) -> &BitSet;
    fn get(&self, index: Index) -> Option<&Self::Item>;
    fn get_mut(&mut self, index: Index) -> Option<&mut Self::Item>;
    fn insert(&mut self, index: Index, value: Self::Item) -> Option<Self::Item>;
    fn remove(&mut self, index: Index) -> Option<Self::Item>;
}

#[derive(Default)]
pub struct MaskStore<S> {
    mask: BitSet,
    store: S,
}

impl<S: RawStore> MaskStore<S> {
    pub fn inner(&self) -> &S {
        &self.store
    }
}

impl<S: RawStore> Store for MaskStore<S> {
    type Item = S::Item;

    fn mask(&self) -> &BitSet {
        &self.mask
    }

    fn get(&self, index: Index) -> Option<&Self::Item> {
        if self.mask.contains(index) {
            Some(unsafe { self.store.get(index) })
        } else {
            None
        }
    }

    fn get_mut(&mut self, index: Index) -> Option<&mut Self::Item> {
        if self.mask.contains(index) {
            Some(unsafe { self.store.get_mut(index) })
        } else {
            None
        }
    }

    fn insert(&mut self, index: Index, mut value: Self::Item) -> Option<Self::Item> {
        if self.mask.contains(index) {
            mem::swap(&mut value, unsafe { self.store.get_mut(index) });
            Some(value)
        } else {
            self.mask.insert(index);
            unsafe { self.store.insert(index, value) };
            None
        }
    }

    fn remove(&mut self, index: Index) -> Option<Self::Item> {
        if self.mask.remove(index) {
            Some(unsafe { self.store.remove(index) })
        } else {
            None
        }
    }
}

pub trait RawStore {
    type Item;

    unsafe fn get(&self, index: Index) -> &Self::Item;
    #[allow(clippy::mut_from_ref)]
    unsafe fn get_mut(&self, index: Index) -> &mut Self::Item;
    unsafe fn insert(&mut self, index: Index, value: Self::Item);
    unsafe fn remove(&mut self, index: Index) -> Self::Item;
}

pub struct VecStore<T> {
    vec: Vec<UnsafeCell<MaybeUninit<T>>>,
}

unsafe impl<T: Send> Send for VecStore<T> {}
unsafe impl<T: Sync> Sync for VecStore<T> {}

impl<T> Default for VecStore<T> {
    fn default() -> Self {
        Self {
            vec: Default::default(),
        }
    }
}

impl<T> RawStore for VecStore<T> {
    type Item = T;

    unsafe fn get(&self, index: Index) -> &T {
        unsafe { &*(*self.vec.get_unchecked(index).get()).as_ptr() }
    }

    unsafe fn get_mut(&self, index: Index) -> &mut T {
        unsafe { &mut *(*self.vec.get_unchecked(index).get()).as_mut_ptr() }
    }

    unsafe fn insert(&mut self, index: Index, c: T) {
        unsafe {
            if self.vec.len() <= index {
                let delta = index + 1 - self.vec.len();
                self.vec.reserve(delta);
                self.vec.set_len(index + 1);
            }
            *self.vec.get_unchecked_mut(index) = UnsafeCell::new(MaybeUninit::new(c));
        }
    }

    unsafe fn remove(&mut self, index: Index) -> T {
        unsafe { (*self.vec.get_unchecked(index).get()).as_mut_ptr().read() }
    }
}
