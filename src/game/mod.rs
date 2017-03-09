use std::collections::BTreeSet;

use ::engine::*;

// TODO: how much of this stuff really need to be public?
mod entity;
use self::entity::*;

mod position;
use self::position::*;

mod stats;
use self::stats::*;

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

// TODO: allow bold attribute?
#[derive(Debug)]
pub struct Cell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
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

struct IsPlayer;

impl Component for IsPlayer {}

world! {
    GameWorld {
        Entity: {
            EntityType,
            Location,
            Contents,
            IsPlayer,
            Damage,
            Exhaustion,
            Hunger,
            CorpseType,
        }
        Position: {
            Contents,
            Tile,
        }
    }
}

pub enum Action {
    Wait,
    Move(Direction),
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

    pub fn take_turn(&mut self, action: Action) {
        // TODO: make sure there is at most one player?
        let player = self.world.component_ids::<IsPlayer>().next();
        if let Some(player) = player {
            match action {
                Action::Wait => { /* TODO */ }
                Action::Move(dir) => {
                    let moved = self.move_entity(player, dir);
                    if !moved { return; }
                }
            }
        }

        // TODO: creature actions

        // TODO: per turn processing
    }

    fn move_entity(&mut self, id: Entity, dir: Direction) -> bool {
        let location: Option<Location> = self.world.get(id).map(|&l| l);
        if let Some(Location::Position(pos)) = location {
            let new_pos = pos.step(dir);
            if self.world.entity_ref(new_pos).get::<Tile>()
                .map(Tile::is_walkable).unwrap_or(false)
            {
                // TODO: make sure there is only one valid target in location?
                let target = self.world.entity_ref(new_pos).get::<Contents>()
                    .and_then(|contents|
                         contents.0.iter().find(|&&id| {
                             self.world.entity_ref(id).get::<EntityType>().map(
                                 |t| t.data().is_actor()
                             ).unwrap_or(false)
                         })
                    ).map(|&id| id);
                if let Some(target) = target {
                    self.attack(id, target);
                } else {
                    self.world.set_location(id, Location::Position(new_pos));
                }
                return true;
            }
        }
        false
    }

    fn attack(&mut self, attacker: Entity, target: Entity) {
        // TODO: make this less bad
        if let Some(&EntityClass::Actor { max_health, .. }) =
            self.world.entity_ref(target).get::<EntityType>().map(|t| &t.data().class)
        {
            let total_damage = {
                let damage_mut: &mut Damage = self.world.get_or_default(target);
                damage_mut.0 += 1;
                damage_mut.0
            };
            if total_damage >= max_health {
                self.kill_entity(target);
            }
        }
    }

    fn kill_entity(&mut self, id: Entity) {
        let old_type = self.world.insert(id, EntityType::Corpse);
        if let Some(corpse_type) = old_type {
            self.world.insert(id, CorpseType(corpse_type));
        }
    }

    // TODO: this is a temporary (for testing)
    fn put_entity(&mut self, t: EntityType, p: Position) {
        let id = Entity(self.next_id);
        self.next_id += 1;
        if t == EntityType::Player {
            self.world.insert(id, IsPlayer);
        }
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
                    data = match (data.map(|d| &d.class), &entity_data.class) {
                        (Some(&EntityClass::Actor { .. }), _) => data,
                        (
                            Some(&EntityClass::Item { display_priority: old_priority, .. }),
                            &EntityClass::Item { display_priority: new_priority, .. },
                        ) if old_priority <= new_priority => data,
                        _ => Some(entity_data),
                    };
                }
            }
        }

        let cell = self.world.entity_ref(p).get::<Tile>().map(Tile::render)
            .unwrap_or(
                Cell {
                    ch: ' ',
                    fg: Color::Black,
                    bg: Color::Black,
                    bold: false,
                }
            );

        if let Some(d) = data {
            Cell {
                ch: d.ch,
                fg: d.color.unwrap_or(cell.fg),
                bg: cell.bg,
                bold: d.is_actor(),
            }
        } else {
            cell
        }
    }
}

// TODO: this is temporary
pub fn init_game(g: &mut Game) {
    g.put_entity(EntityType::Player, Position { x: 3, y: 3 });
    g.put_entity(EntityType::Rock, Position { x: 3, y: 3 });

    g.put_entity(EntityType::Rat, Position { x: 5, y: 3 });
    g.put_entity(EntityType::Rock, Position { x: 5, y: 3 });

    g.put_entity(EntityType::Rock, Position { x: 4, y: 4 });
    g.put_entity(EntityType::Rat, Position { x: 4, y: 4 });

    g.put_entity(EntityType::Rock, Position { x: 6, y: 4 });

    for x in 0..9 {
        for y in 0..9 {
            let tile = if x%8 == 0 || y%8 == 0 { Tile::Wall } else { Tile::Ground };
            g.world.insert(Position{ x, y }, tile);
        }
    }

    g.world.insert(Position{ x: 6, y: 5 }, Tile::ShallowWater);
    g.world.insert(Position{ x: 7, y: 5 }, Tile::ShallowWater);
    g.world.insert(Position{ x: 6, y: 6 }, Tile::ShallowWater);
    g.world.insert(Position{ x: 7, y: 6 }, Tile::DeepWater);
    g.world.insert(Position{ x: 6, y: 7 }, Tile::DeepWater);
    g.world.insert(Position{ x: 7, y: 7 }, Tile::DeepWater);

    g.put_entity(EntityType::Rock, Position { x: 7, y: 5 });
    g.put_entity(EntityType::Rock, Position { x: 7, y: 6 });

    g.world.insert(Position{ x: 1, y: 4 }, Tile::ShortGrass);
    g.world.insert(Position{ x: 2, y: 4 }, Tile::ShortGrass);
    g.world.insert(Position{ x: 3, y: 4 }, Tile::ShortGrass);
    g.world.insert(Position{ x: 1, y: 5 }, Tile::LongGrass);
    g.world.insert(Position{ x: 2, y: 5 }, Tile::LongGrass);
    g.world.insert(Position{ x: 3, y: 5 }, Tile::LongGrass);
    g.world.insert(Position{ x: 1, y: 6 }, Tile::LongGrass);
    g.world.insert(Position{ x: 2, y: 6 }, Tile::LongGrass);
    g.world.insert(Position{ x: 3, y: 6 }, Tile::LongGrass);
}
