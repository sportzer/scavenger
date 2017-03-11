use std::collections::BTreeSet;
use rand::{Rng, SeedableRng, StdRng};

use ::engine::*;

// TODO: how much of this stuff really need to be public?
mod entity;
use self::entity::*;

mod position;
use self::position::*;

mod stats;
use self::stats::*;

mod fov;
use self::fov::*;

mod map;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Entity(u64);

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
            AiState,
        }
        Position: {
            Contents,
            Tile,
            IsVisible,
            WasVisible,
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
    // TODO: this is total unserializable, I'll probably have to roll my own RNG
    rand: StdRng,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        let mut g = Game {
            world: GameWorld::new(),
            next_id: 1,
            rand: StdRng::from_seed(&[seed as usize]),
        };
        map::init_game(&mut g);
        update_fov(&mut g);
        g
    }

    pub fn take_turn(&mut self, action: Action) {
        if let Some(player) = self.find_player() {
            match action {
                Action::Wait => { /* TODO */ }
                Action::Move(dir) => {
                    let moved = self.move_entity(player, dir);
                    if !moved { return; }
                }
            }
        }
        update_fov(self);

        // TODO: creature actions
        let creatures: Vec<Entity> = self.world.component_ids::<AiState>().collect();
        for id in creatures {
            // TODO: add real AI
            self.move_entity(id, Direction::West);
        }

        // TODO: per turn processing

        update_fov(self);
    }

    pub fn render(&self, pos: Position) -> Cell {
        let mut cell = Cell {
            ch: ' ',
            fg: Color::Black,
            bg: Color::Black,
            bold: false,
        };

        if let Some(&WasVisible(tile)) = self.world.get(pos) {
            cell = tile.render_memory();
        }

        if let Some(&IsVisible(dist)) = self.world.get(pos) {
            if dist <= self.player_fov_range() {
                cell = self.get_tile(pos).render();

                let mut data: Option<&'static EntityData> = None;

                if let Some(&Contents(ref entities)) = self.world.get(pos) {
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

                if let Some(d) = data {
                    cell = Cell {
                        ch: d.ch,
                        fg: d.color.unwrap_or(cell.fg),
                        bg: cell.bg,
                        bold: d.is_actor(),
                    }
                }
            }
        }

        cell
    }

    pub fn player_status(&self) -> String {
        if let Some(player) = self.find_player() {
            let player_ref = self.world.entity_ref(player);
            if let Some(&EntityClass::Actor {
                max_health, max_stamina, max_satiation, ..
            }) = player_ref.get::<EntityType>().map(|d| &d.data().class)
            {
                let damage = player_ref.get::<Damage>().map(|d| d.0).unwrap_or(0);
                let exhaustion = player_ref.get::<Exhaustion>().map(|e| e.0).unwrap_or(0);
                let hunger = player_ref.get::<Hunger>().map(|h| h.0).unwrap_or(0);
                return format!(
                    " Health: {:2}/{} -- Stamina: {:2}/{} -- Satiation: {:3}/{} ",
                    max_health - damage, max_health,
                    max_stamina - exhaustion, max_stamina,
                    max_satiation - hunger, max_satiation,
                );
            }
        }
        // TODO: Add final score or something?
        format!(" Game over. Press 'N' to restart. ")
    }
}

impl Game {
    fn find_player(&self) -> Option<Entity> {
        // TODO: make sure there is at most one player?
        self.world.component_ids::<IsPlayer>().next()
    }

    fn player_fov_range(&self) -> i8 {
        // TODO: dynamic view distance
        if let Some(&EntityClass::Actor { fov_range, .. }) =
            self.find_player()
            .and_then(|id| self.world.entity_ref(id).get::<EntityType>())
            .map(|t| &t.data().class)
        {
            fov_range
        } else {
            0
        }
    }

    fn get_tile(&self, pos: Position) -> Tile {
        *self.world.entity_ref(pos).get::<Tile>().unwrap_or(&Tile::Wall)
    }

    fn move_entity(&mut self, id: Entity, dir: Direction) -> bool {
        let location: Option<Location> = self.world.get(id).map(|&l| l);
        if let Some(Location::Position(pos)) = location {
            let new_pos = pos.step(dir);
            if self.get_tile(new_pos).is_walkable() {
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
        // TODO: figure out whether I want these checks or not
        if !self.is_actor(attacker) || !self.is_actor(target) { return; }
        if self.is_player(attacker) == self.is_player(target) { return; }

        // TODO: make this less bad
        self.add_damage(target, 1);
    }

    fn is_player(&self, id: Entity) -> bool {
        self.world.entity_ref(id).get::<IsPlayer>().is_some()
    }

    fn is_actor(&self, id: Entity) -> bool {
        self.world.entity_ref(id).get::<EntityType>()
            .map(|t| t.data().is_actor()).unwrap_or(false)
    }

    fn add_damage(&mut self, target: Entity, damage: i8) {
        if let Some(&EntityClass::Actor { max_health, .. }) =
            self.world.entity_ref(target).get::<EntityType>().map(|t| &t.data().class)
        {
            let total_damage = {
                let target_damage: &mut Damage = self.world.get_or_default(target);
                target_damage.0 += damage;
                if target_damage.0 < 0 { target_damage.0 = 0; }
                target_damage.0
            };
            if total_damage >= max_health {
                self.kill_entity(target);
            }
        }
    }

    fn add_exhaustion(&mut self, target: Entity, exhaustion: i8) {
        if let Some(&EntityClass::Actor { max_stamina, .. }) =
            self.world.entity_ref(target).get::<EntityType>().map(|t| &t.data().class)
        {
            let excess_exhaustion = {
                let target_exhaustion: &mut Exhaustion = self.world.get_or_default(target);
                target_exhaustion.0 += exhaustion;
                if target_exhaustion.0 < 0 { target_exhaustion.0 = 0; }
                if target_exhaustion.0 >= max_stamina {
                    let excess_exhaustion = target_exhaustion.0 - max_stamina;
                    target_exhaustion.0 = max_stamina;
                    Some(excess_exhaustion)
                } else {
                    None
                }
            };
            if let Some(excess_exhaustion) = excess_exhaustion {
                self.add_damage(target, excess_exhaustion);
            }
        }
    }

    fn add_hunger(&mut self, target: Entity, hunger: i16) {
        if let Some(&EntityClass::Actor { max_satiation, .. }) =
            self.world.entity_ref(target).get::<EntityType>().map(|t| &t.data().class)
        {
            let excess_hunger = {
                let target_hunger: &mut Hunger = self.world.get_or_default(target);
                target_hunger.0 += hunger;
                if target_hunger.0 < 0 { target_hunger.0 = 0; }
                if target_hunger.0 >= max_satiation {
                    let excess_hunger = target_hunger.0 - max_satiation;
                    target_hunger.0 = max_satiation;
                    Some(excess_hunger)
                } else {
                    None
                }
            };
            if let Some(excess_hunger) = excess_hunger {
                self.add_exhaustion(target, ::std::cmp::min(excess_hunger, 127) as i8);
            }
        }
    }

    fn kill_entity(&mut self, id: Entity) {
        let old_type = self.world.insert(id, EntityType::Corpse);
        if let Some(corpse_type) = old_type {
            self.world.insert(id, CorpseType(corpse_type));
        }
        self.world.entity_mut(id).remove::<AiState>();
    }

    // TODO: this is a temporary (for testing)
    fn put_entity(&mut self, t: EntityType, p: Position) {
        let id = Entity(self.next_id);
        self.next_id += 1;
        if t == EntityType::Player {
            self.world.insert(id, IsPlayer);
        } else if t.data().is_actor() {
            self.world.insert(id, AiState::Waiting);
        }
        self.world.insert(id, t);
        self.world.set_location(id, Location::Position(p));
    }
}
