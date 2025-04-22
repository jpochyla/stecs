use std::{
    cell::UnsafeCell,
    mem::{self, MaybeUninit},
};

use hibitset::BitSet;
use rayon_ecs_derive::Lend;

type Index = u32;
type Gen = u32;

#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
struct Entity {
    gen: Gen,
    index: Index,
}

trait Store {
    type Item;

    fn mask(&self) -> &BitSet;
    fn get(&self, index: Index) -> Option<&Self::Item>;
    fn get_mut(&mut self, index: Index) -> Option<&mut Self::Item>;
    fn insert(&mut self, index: Index, value: Self::Item) -> Option<Self::Item>;
    fn remove(&mut self, index: Index) -> Option<Self::Item>;
}

#[derive(Default)]
struct MaskStore<S> {
    mask: BitSet,
    store: S,
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
            self.mask.add(index);
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

trait RawStore {
    type Item;

    unsafe fn get(&self, index: Index) -> &Self::Item;
    unsafe fn get_mut(&self, index: Index) -> &mut Self::Item;
    unsafe fn insert(&mut self, index: Index, value: Self::Item);
    unsafe fn remove(&mut self, index: Index) -> Self::Item;
}

#[derive(Default)]
struct VecStore<T> {
    vec: Vec<UnsafeCell<MaybeUninit<T>>>,
}

unsafe impl<T: Send> Send for VecStore<T> {}
unsafe impl<T: Sync> Sync for VecStore<T> {}

impl<T> RawStore for VecStore<T> {
    type Item = T;

    unsafe fn get(&self, index: Index) -> &T {
        let index = index as usize;
        unsafe { &*(*self.vec.get_unchecked(index).get()).as_ptr() }
    }

    unsafe fn get_mut(&self, index: Index) -> &mut T {
        let index = index as usize;
        unsafe { &mut *(*self.vec.get_unchecked(index).get()).as_mut_ptr() }
    }

    unsafe fn insert(&mut self, index: Index, c: T) {
        let index = index as usize;
        if self.vec.len() <= index {
            let delta = index + 1 - self.vec.len();
            self.vec.reserve(delta);
            unsafe { self.vec.set_len(index + 1) };
        }
        unsafe { *self.vec.get_unchecked_mut(index) = UnsafeCell::new(MaybeUninit::new(c)) };
    }

    unsafe fn remove(&mut self, index: Index) -> T {
        let index = index as usize;
        unsafe { (*self.vec.get_unchecked(index).get()).as_mut_ptr().read() }
    }
}

#[derive(Default)]
struct Foo(u8);

#[derive(Default)]
struct Bar(u8);

type Entities = MaskStore<VecStore<Gen>>;
type Foos = MaskStore<VecStore<Foo>>;
type Bars = MaskStore<VecStore<Bar>>;

fn new_entity(entities: &mut Entities) -> Entity {
    let gen = 0; // TODO: actual entity allocation.
    let idx = entities.store.vec.len() as u32;
    entities.insert(idx, gen);
    Entity { index: idx, gen }
}

#[derive(Default)]
struct World {
    entities: Entities,
    foos: Foos,
    bars: Bars,
}

#[derive(Lend)]
struct MutFoos<'w> {
    foos: &'w mut Foos,
}

impl<'w> MutFoos<'w> {
    fn set_foo(&mut self, index: Index, val: u8) {
        self.foos.insert(index, Foo(val));
    }
}

#[derive(Lend)]
struct MutBars<'w> {
    bars: &'w mut Bars,
}

impl<'w> MutBars<'w> {
    fn set_bar(&mut self, index: Index, val: u8) {
        self.bars.insert(index, Bar(val));
    }
}

#[derive(Lend)]
struct GetBars<'w> {
    bars: &'w Bars,
}

impl<'w> GetBars<'w> {
    fn bar(&self, index: Index) -> Option<&'w Bar> {
        self.bars.get(index)
    }
}

#[derive(Lend)]
struct MutFoosBars<'w> {
    first: MutFoos<'w>,
    second: MutBars<'w>,
}

impl<'w> MutFoosBars<'w> {
    fn foo_bar(&self, index: Index) -> Option<(&Foo, &Bar)> {
        let foo = self.first.foos.get(index)?;
        let bar = self.second.bars.get(index)?;
        Some((foo, bar))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut w = World::default();

        let e = new_entity(&mut w.entities);
        let mut foos = MutFoos!(w);
        let mut bars = MutBars!(w);

        foos.set_foo(e.index, 1);
        bars.set_bar(e.index, 1);

        let bars_again = GetBars!(bars);
        let bar = bars_again.bar(e.index).unwrap();

        let compound = MutFoosBars!(w);
        let (foo, bar) = compound.foo_bar(e.index).unwrap();
    }
}
