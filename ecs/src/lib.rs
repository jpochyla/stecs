mod entity;
mod query;
mod store;

use mosaic_derive::Mosaic;

use self::entity::*;
use self::query::*;
use self::store::*;

pub type BitSet = hi_sparse_bitset::BitSet<hi_sparse_bitset::config::_128bit>;
pub type Index = usize;
pub type Handle = u64;

#[derive(Debug)]
struct FooComp(u8);

#[derive(Debug)]
struct BarComp(u8);

type Foos = MaskStore<VecStore<FooComp>>;
type Bars = MaskStore<VecStore<BarComp>>;

#[derive(Default)]
struct World {
    entities: Entities,
    foos: Foos,
    bars: Bars,
}

impl World {
    fn free(&mut self, handle: Handle) {
        if let Some(index) = self.entities.free(handle) {
            self.foos.remove(index);
            self.bars.remove(index);
        }
    }
}

#[derive(Mosaic)]
struct MutFoos<'w> {
    foos: &'w mut Foos,
}

impl<'w> MutFoos<'w> {
    fn set_foo(&mut self, index: Index, val: u8) {
        self.foos.insert(index, FooComp(val));
    }
}

#[derive(Mosaic)]
struct MutBars<'w> {
    bars: &'w mut Bars,
}

impl<'w> MutBars<'w> {
    fn set_bar(&mut self, index: Index, val: u8) {
        self.bars.insert(index, BarComp(val));
    }
}

#[derive(Mosaic)]
struct GetBars<'w> {
    bars: &'w Bars,
}

impl<'w> GetBars<'w> {
    fn bar(&self, index: Index) -> Option<&'w BarComp> {
        self.bars.get(index)
    }
}

#[derive(Mosaic)]
struct MutFoosBars<'w> {
    first: MutFoos<'w>,
    second: MutBars<'w>,
}

impl<'w> MutFoosBars<'w> {
    fn foo_bar(&self, index: Index) -> Option<(&FooComp, &BarComp)> {
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

        let h = w.entities.allocate();
        let i = w.entities.get(h).unwrap();

        let mut foos = MutFoos!(w);
        let mut bars = MutBars!(w);

        foos.set_foo(i, 1);
        bars.set_bar(i, 1);

        let bars_again = GetBars!(bars);
        let bar = bars_again.bar(i).unwrap();

        let compound = MutFoosBars!(w);
        let (foo, bar) = compound.foo_bar(i).unwrap();

        (compound.first.foos, compound.second.bars)
            .query()
            .for_each(|(foo, bar)| {
                dbg!(foo, bar);
            });
    }
}
