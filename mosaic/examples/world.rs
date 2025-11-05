use mosaic::{Entities, Handle, Index, IntoQuery, MaskStore, Mosaic, Store, VecStore};

#[derive(Debug)]
struct Player;

#[derive(Debug)]
struct Monster;

type Positions = MaskStore<VecStore<(f64, f64)>>;
type Velocities = MaskStore<VecStore<(f64, f64)>>;
type Healths = MaskStore<VecStore<u8>>;
type Players = MaskStore<VecStore<Player>>;
type Monsters = MaskStore<VecStore<Monster>>;

#[derive(Default)]
struct World {
    entities: Entities,
    healths: Healths,
    positions: Positions,
    velocities: Velocities,
    players: Players,
    monsters: Monsters,
}

impl World {
    fn free(&mut self, handle: Handle) {
        if let Some(index) = self.entities.free(handle) {
            self.healths.remove(index);
            self.positions.remove(index);
            self.velocities.remove(index);
            self.players.remove(index);
            self.monsters.remove(index);
        }
    }
}

struct Physics<'w> {
    positions: &'w mut Positions,
    velocities: &'w mut Velocities,
}

impl<'w> Physics<'w> {
    fn update(&mut self, dt: f64) {
        (&mut *self.positions, &mut *self.velocities)
            .query()
            .for_each(|(pos, vel)| {
                pos.0 += vel.0 * dt;
                pos.1 += vel.0 * dt;
            });
    }
}

#[derive(Mosaic)]
struct NewPlayer<'w> {
    entities: &'w mut Entities,
    positions: &'w mut Positions,
    velocities: &'w mut Velocities,
    healths: &'w mut Healths,
    players: &'w mut Players,
}

impl<'w> NewPlayer<'w> {
    fn create(&mut self, pos: (f64, f64)) -> Handle {
        let (handle, index) = self.entities.allocate();

        self.positions.insert(index, pos);
        self.velocities.insert(index, (0.0, 0.0));
        self.healths.insert(index, 100);
        self.players.insert(index, Player);

        handle
    }
}

#[derive(Mosaic)]
struct NewMonster<'w> {
    entities: &'w mut Entities,
    positions: &'w mut Positions,
    velocities: &'w mut Velocities,
    healths: &'w mut Healths,
    monsters: &'w mut Monsters,
}

impl<'w> NewMonster<'w> {
    fn create(&mut self, pos: (f64, f64)) -> Handle {
        let (handle, index) = self.entities.allocate();

        self.positions.insert(index, pos);
        self.velocities.insert(index, (0.0, 0.0));
        self.healths.insert(index, 100);
        self.monsters.insert(index, Monster);

        handle
    }
}

#[derive(Mosaic)]
struct Damage<'w> {
    positions: &'w Positions,
    players: &'w Players,
    monsters: &'w Monsters,
    healths: &'w mut Healths,
}

impl<'w> Damage<'w> {
    fn apply(&mut self) {
        (
            &mut *self.healths,
            self.positions,
            self.players.maybe(),
            self.monsters.maybe(),
        )
            .query()
            .for_each(|(pos, vel)| {
                pos.0 += vel.0 * dt;
                pos.1 += vel.0 * dt;
            });
    }
}

fn main() {
    let mut w = World::default();

    let p1 = NewPlayer!(w).create((0.0, 0.0));
    let j1 = NewMonster!(w).create((0.0, 0.0));

    Damage!(w).apply()

    w.free(p1);
    w.free(j1);
}
