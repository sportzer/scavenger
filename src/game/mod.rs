use std::collections::BTreeSet;

use ::engine::*;

// TODO: how much of this stuff really need to be public?
mod entity;
pub use self::entity::*;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct Entity(u64);

impl Id for Entity {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Color {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
}

#[derive(Debug)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
}

// TODO: impl From<Entity>, From<Position>, etc
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Location {
    Entity(Entity),
    Position(Position),
}

impl Component for Location {}

#[derive(Default)]
struct Contents(BTreeSet<Entity>);

impl Component for Contents {}

world! {
    GameWorld {
        Entity: {
            EntityType,
            Location,
            Contents,
        }
        Position: {
            Contents,
        }
    }
}

impl GameWorld {
    fn remove_from_contents<I: Id>(&mut self, entity: Entity, location: I)
        where GameWorld: ComponentStorage<I, Contents>
    {
        let is_empty = self.entity_mut(location).get_mut::<Contents>().map(|c| {
            c.0.remove(&entity);
            c.0.is_empty()
        });
        if is_empty == Some(true) {
            self.entity_mut(location).remove::<Contents>();
        }
    }

    fn remove_location(&mut self, id: Entity) -> Option<Location> {
        let old_location = self.remove(id);
        match old_location {
            Some(Location::Entity(e)) => { self.remove_from_contents(id, e); }
            Some(Location::Position(p)) => { self.remove_from_contents(id, p); }
            None => {}
        }
        old_location
    }

    fn set_location(&mut self, id: Entity, l: Location) -> Option<Location> {
        let old_location = self.insert(id, l);
        match old_location {
            Some(Location::Entity(e)) => { self.remove_from_contents(id, e); }
            Some(Location::Position(p)) => { self.remove_from_contents(id, p); }
            None => {}
        }
        match l {
            Location::Entity(e) => {
                self.entity_mut(e).get_or_else::<Contents, _>(Default::default).0.insert(id);
            }
            Location::Position(p) => {
                self.entity_mut(p).get_or_else::<Contents, _>(Default::default).0.insert(id);
            }
        }
        old_location
    }
}

pub struct Game {
    world: GameWorld,
    next_id: u64,
}

impl Game {
    pub fn new() -> Game {
        Game {
            world: GameWorld::new(),
            next_id: 1,
        }
    }

    // TODO: this is a temporary (for testing)
    pub fn put_entity(&mut self, t: EntityType, p: Position) {
        let id = Entity(self.next_id);
        self.next_id += 1;
        self.world.insert(id, t);
        self.world.set_location(id, Location::Position(p));
    }

    pub fn render(&self, p: Position) -> Cell {
        let mut data: Option<&'static EntityData> = None;

        if let Some(&Contents(ref entities)) = self.world.get(p) {
            for &e in entities {
                if let Some(entity_data) = self.world.entity_ref(e)
                    .get::<EntityType>().map(|t| t.data())
                {
                    if entity_data.class == EntityClass::Player
                        || data.map(|d| {
                            d.class != EntityClass::Player && entity_data.class == EntityClass::Creature
                            || d.class == EntityClass::Item && entity_data.class == EntityClass::Item
                        }).unwrap_or(true)
                    {
                        data = Some(entity_data);
                    }
                }
            }
        }

        data.map(|d| {
            Cell {
                ch: d.ch,
                fg: d.color,
                bg: Color::Black,
            }
        }).unwrap_or(
            Cell {
                ch: ' ',
                fg: Color::Black,
                bg: Color::Black,
            }
        )
    }
}
