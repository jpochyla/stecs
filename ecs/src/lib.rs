use rayon_ecs_derive::Lend;

trait AsStore<T> {
    fn as_store(&self) -> &Store<T>;
}

trait AsStoreMut<T> {
    fn as_store_mut(&mut self) -> &mut Store<T>;
}

#[derive(Default)]
struct Store<T> {
    vec: Vec<T>,
}

impl<T> AsStore<T> for Store<T> {
    fn as_store(&self) -> &Store<T> {
        self
    }
}

impl<T> AsStoreMut<T> for Store<T> {
    fn as_store_mut(&mut self) -> &mut Store<T> {
        self
    }
}

impl<T> Store<T> {
    fn get(&self, entity: Entity) -> Option<&T> {
        self.vec.get(entity as usize)
    }

    fn insert(&mut self, entity: Entity, value: T) {
        self.vec[entity as usize] = value;
    }
}

struct Read<'w, T> {
    store: &'w Store<T>,
}

impl<'w, T> AsStore<T> for Read<'w, T> {
    fn as_store(&self) -> &'w Store<T> {
        self.store
    }
}

impl<'w, T> Read<'w, T> {
    fn get(&self, entity: Entity) -> Option<&'w T> {
        self.store.get(entity)
    }
}

macro_rules! Read {
    ($obj:ident, $field:ident) => {
        Read {
            store: $obj.$field.as_store(),
        }
    };
}

struct Write<'w, T> {
    store: &'w mut Store<T>,
}

impl<'w, T> AsStore<T> for Write<'w, T> {
    fn as_store(&self) -> &Store<T> {
        self.store
    }
}

impl<'w, T> AsStoreMut<T> for Write<'w, T> {
    fn as_store_mut(&mut self) -> &mut Store<T> {
        self.store
    }
}

impl<'w, T> Write<'w, T> {
    fn get(&self, entity: Entity) -> Option<&T> {
        self.store.get(entity)
    }

    fn insert(&mut self, entity: Entity, value: T) {
        self.store.insert(entity, value);
    }
}

macro_rules! Write {
    ($obj:ident, $field:ident) => {
        Write {
            store: $obj.$field.as_store_mut(),
        }
    };
}

type Entity = u32;

#[derive(Default)]
struct Foo;

#[derive(Default)]
struct Bar;

#[derive(Default)]
struct World {
    entities: Store<Entity>,
    foos: Store<Foo>,
    bars: Store<Bar>,
}

impl World {
    fn new_entity(&mut self) -> Entity {
        let entity = self.entities.vec.len() as u32;
        self.entities.vec.push(entity);
        let capacity = self.entities.vec.len();
        self.foos.vec.resize_with(capacity, Default::default);
        self.bars.vec.resize_with(capacity, Default::default);
        entity
    }
}

#[derive(Lend)]
struct WriteFoos<'w> {
    foos: Write<'w, Foo>,
}

impl<'w> WriteFoos<'w> {
    fn set_foo(&mut self, ent: Entity) {
        self.foos.insert(ent, Foo);
    }
}

#[derive(Lend)]
struct WriteBars<'w> {
    bars: Write<'w, Bar>,
}

impl<'w> WriteBars<'w> {
    fn set_bar(&mut self, ent: Entity) {
        self.bars.insert(ent, Bar);
    }
}

#[derive(Lend)]
struct ReadBars<'w> {
    bars: Read<'w, Bar>,
}

impl<'w> ReadBars<'w> {
    fn get_bar(&self, ent: Entity) -> Option<&'w Bar> {
        self.bars.get(ent)
    }
}

#[derive(Lend)]
struct Compound<'w> {
    first: WriteFoos<'w>,
    second: WriteBars<'w>,
}

impl<'w> Compound<'w> {
    fn get_foo_bar(&self, ent: Entity) -> Option<(&Foo, &Bar)> {
        let foo = self.first.foos.get(ent)?;
        let bar = self.second.bars.get(ent)?;
        Some((foo, bar))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut w = World::default();

        let ent = w.new_entity();
        let mut first = WriteFoos!(w);
        let mut second = WriteBars!(w);

        first.set_foo(ent);
        second.set_bar(ent);

        let second_read = ReadBars!(second);
        let bar = second_read.get_bar(ent).unwrap();

        let compound = Compound!(w);
        let (foo, bar) = compound.get_foo_bar(ent).unwrap();
    }
}
