use std::collections::BTreeSet;
use ::engine::*;

mod entity;
use self::entity::*;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct Entity(u64);

impl Id for Entity {}

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
    World {
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

impl World {
    fn remove_from_contents<I: Id>(&mut self, entity: Entity, location: I)
        where World: ComponentStorage<I, Contents>
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
