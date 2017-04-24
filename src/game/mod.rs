use std::collections::{BTreeSet, BTreeMap};
use std::collections::btree_map::Entry;
use rand::{Rng, SeedableRng, StdRng};

use ::engine::*;

// TODO: how much of this stuff really need to be public?
mod entity;
use self::entity::{Corpse, AiState, EntityClass, EntityData};
pub use self::entity::EntityType;

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
            Corpse,
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
    EatHerb,
    ReadScroll,
    GetCorpse,
    DropCorpse,
    ThrowRock(Position),
    FireBow(Direction),
}

impl GameWorld {
    fn remove_from_contents<I: Id>(&mut self, entity: Entity, location: I)
        where GameWorld: EntityComponent<I, Contents>
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
        let old_location = self.entity_mut(id).remove();
        match old_location {
            Some(Location::Entity(e)) => { self.remove_from_contents(id, e); }
            Some(Location::Position(p)) => { self.remove_from_contents(id, p); }
            None => {}
        }
        old_location
    }

    fn set_location(&mut self, id: Entity, l: Location) -> Option<Location> {
        let old_location = self.entity_mut(id).insert(l);
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

pub struct Rect {
    pub min_x: i32,
    pub max_x: i32,
    pub min_y: i32,
    pub max_y: i32,
}

impl Rect {
    fn extend(&mut self, pos: Position) {
        self.min_x = ::std::cmp::min(self.min_x, pos.x);
        self.max_x = ::std::cmp::max(self.max_x, pos.x);
        self.min_y = ::std::cmp::min(self.min_y, pos.y);
        self.max_y = ::std::cmp::max(self.max_y, pos.y);
    }
}

impl From<Position> for Rect {
    fn from(pos: Position) -> Rect {
        Rect {
            min_x: pos.x,
            max_x: pos.x,
            min_y: pos.y,
            max_y: pos.y,
        }
    }
}

pub struct Game {
    world: GameWorld,
    next_id: u64,
    // TODO: this is total unserializable, I'll probably have to roll my own RNG
    rand: StdRng,
    recall_turns: Option<i32>,
    // TODO: I'd like this to be a component, but then I'd need a way to swap
    // out the entire map (or I could just be super inefficient). It's a hack
    // either way.
    smell_strength: BTreeMap<Position, i32>,
    current_turn: i32,
}

pub struct PlayerStatus {
    pub health: i8,
    pub max_health: i8,
    pub has_bow: bool,
    pub has_sword: bool,
    pub arrows: i32,
    pub herbs: i32,
    pub rocks: i32,
    pub corpses: i32,
    pub diamonds: i32,
    pub recall_turns: Option<i32>,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        let mut g = Game {
            world: GameWorld::new(),
            next_id: 1,
            rand: StdRng::from_seed(&[seed as usize]),
            recall_turns: None,
            smell_strength: BTreeMap::new(),
            current_turn: 0,
        };
        map::init_game(&mut g);
        update_fov(&mut g);
        g
    }

    pub fn take_turn(&mut self, action: Action) {
        if let (Some(player), Some(player_pos)) = (self.find_player(), self.player_position()) {
            match action {
                Action::Wait => {}
                Action::Move(dir) => {
                    let moved = self.move_entity(player, dir);
                    self.auto_pickup();
                    if !moved { return; }
                }
                Action::EatHerb => {
                    if self.consume_item(EntityType::Herb) {
                        self.add_damage(player, -1);
                    } else { return; }
                }
                Action::ReadScroll => { self.recall_turns = Some(self.rand.gen_range(20, 30)); }
                Action::GetCorpse => {
                    if let Some(corpse) = self.find_corpse() {
                        self.world.set_location(corpse, Location::Entity(player));
                    } else { return; }
                }
                Action::DropCorpse => {
                    if let Some(corpse) = self.find_item(EntityType::Corpse) {
                        self.world.set_location(corpse, Location::Position(player_pos));
                    } else { return; }
                }
                Action::ThrowRock(pos) => {
                    if let Ok(&IsVisible(dist)) = self.world.entity(pos).get() {
                        if dist <= self.player_fov_range()
                            && self.find_item(EntityType::Rock).is_some()
                        {
                            if self.attack_position(player, pos, 1) {
                                self.consume_item(EntityType::Rock);
                            } else { return; }
                        } else { return; }
                    } else { return; }
                }
                Action::FireBow(dir) => {
                    if self.find_item(EntityType::Bow).is_some()
                        && self.consume_item(EntityType::Arrow)
                    {
                        let mut pos = player_pos;
                        for _ in 0..8 {
                            pos = pos.step(dir);
                            if self.get_tile(pos).is_obstructed() {
                                break;
                            }
                            if self.attack_position(player, pos, 2) {
                                break;
                            }
                        }
                    } else { return; }
                }
            }
            update_fov(self);

            let creatures: Vec<Entity> = self.world.component::<AiState>().ids().collect();
            for id in creatures {
                if let Some(state) = self.world.entity(id).get::<AiState>().ok().cloned() {
                    let new_state = state.take_turn(self, id);
                    if let Some(state_mut) = self.world.entity_mut(id).get_mut::<AiState>() {
                        *state_mut = new_state;
                    }
                }
            }

            if let Some(turns) = self.recall_turns {
                self.recall_turns = Some(turns - 1);
                if self.recall_turns == Some(0) {
                    self.world.remove_location(player);
                }
            }

            update_fov(self);

            self.update_smells();
            self.current_turn += 1;
        }
    }

    pub fn render(&self, pos: Position) -> Cell {
        let mut cell = Cell {
            ch: ' ',
            fg: Color::Black,
            bg: Color::Black,
            bold: false,
        };

        if let Ok(&WasVisible(tile)) = self.world.entity(pos).get() {
            cell = tile.render_memory();
        }

        if let Ok(&IsVisible(dist)) = self.world.entity(pos).get() {
            if dist <= self.player_fov_range() {
                cell = self.get_tile(pos).render();

                let mut data: Option<&'static EntityData> = None;

                if let Ok(&Contents(ref entities)) = self.world.entity(pos).get() {
                    for &e in entities {
                        if let Ok(entity_data) = self.world.entity(e)
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
                        bold: true, // d.is_actor(),
                    }
                }
            }
        }

        // // Visualize smell propagation
        // if "\".,=".contains(cell.ch) {
        //     if let Some(&s) = self.smell_strength.get(&pos) {
        //         if s > 0 && s <= 26 {
        //             cell.ch = (64 + s) as u8 as char;
        //         }
        //     }
        // }

        cell
    }

    pub fn player_status(&self) -> Option<PlayerStatus> {
        if let Some(player) = self.find_player() {
            let player_ref = self.world.entity(player);
            // TODO: don't hardcode player type (handle death better)
            if let EntityClass::Actor { max_health, .. } = EntityType::Player.data().class {
                let damage = player_ref.get::<Damage>().map(|d| d.0).unwrap_or(0);
                return Some(PlayerStatus {
                    max_health: max_health,
                    health: max_health - damage,
                    has_bow: self.inventory_count(EntityType::Bow) > 0,
                    has_sword: self.inventory_count(EntityType::Sword) > 0,
                    arrows: self.inventory_count(EntityType::Arrow),
                    herbs: self.inventory_count(EntityType::Herb),
                    rocks: self.inventory_count(EntityType::Rock),
                    corpses: self.inventory_count(EntityType::Corpse),
                    diamonds: self.inventory_count(EntityType::Diamond),
                    recall_turns: self.recall_turns,
                });
            }
        }
        None
    }

    pub fn player_position(&self) -> Option<Position> {
        if let Some(&Location::Position(pos)) =
            self.find_player().and_then(|id| self.world.entity(id).get().ok())
        {
            Some(pos)
        } else {
            None
        }
    }

    pub fn fov_bounding_rect(&self) -> Option<Rect> {
        if let Some(pos) = self.player_position() {
            let fov_range = self.player_fov_range();
            let mut rect = Rect::from(pos);
            for (pos, &IsVisible(dist)) in self.world.component::<IsVisible>().iter() {
                if dist <= fov_range {
                    rect.extend(pos);
                }
            }
            Some(rect)
        } else {
            None
        }
    }
}

impl Game {
    fn find_player(&self) -> Option<Entity> {
        // TODO: make sure there is at most one player?
        self.world.component::<IsPlayer>().ids().next()
    }

    fn auto_pickup(&mut self) {
        if let (Some(pos), Some(player)) = (self.player_position(), self.find_player()) {
            let new_items: Vec<_> =
                if let Ok(&Contents(ref entities)) = self.world.entity(pos).get()
            {
                entities.iter()
                    .cloned()
                    .filter(|&id| {
                        if let Some(entity_type) = self.world.entity(id)
                            .get::<EntityType>().ok().cloned()
                        {
                            [
                                EntityType::Sword, EntityType::Bow,
                                EntityType::Arrow, EntityType::Rock,
                                EntityType::Herb, EntityType::Diamond,
                            ].contains(&entity_type)
                        } else {
                            false
                        }
                    })
                    .collect()
            } else { return; };
            for item in new_items {
                self.world.set_location(item, Location::Entity(player));
            }
        }
    }

    fn player_fov_range(&self) -> i8 {
        // TODO: dynamic view distance
        if let Some(&EntityClass::Actor { fov_range, .. }) =
            self.find_player()
            .and_then(|id| self.world.entity(id).get::<EntityType>().ok())
            .map(|t| &t.data().class)
        {
            fov_range
        } else {
            0
        }
    }

    fn get_tile(&self, pos: Position) -> Tile {
        *self.world.entity(pos).get::<Tile>().unwrap_or(&Tile::Wall)
    }

    fn get_actor_by_position(&self, pos: Position) -> Option<Entity> {
        // TODO: make sure there is only one valid target in location?
        self.world.entity(pos).get::<Contents>().ok()
            .and_then(
                |contents| contents.0.iter().find(|&&id| {
                    self.world.entity(id).get::<EntityType>().map(
                        |t| t.data().is_actor()
                    ).unwrap_or(false)
                }).cloned()
            )
    }

    // TODO: dedup with attack_position
    fn move_entity(&mut self, id: Entity, dir: Direction) -> bool {
        let location: Option<Location> = self.world.entity(id).get().ok().cloned();
        if let Some(Location::Position(pos)) = location {
            let new_pos = pos.step(dir);
            if self.get_tile(new_pos).is_walkable() {
                let target = self.get_actor_by_position(new_pos);
                if let Some(target) = target {
                    return self.bump_attack(id, target);
                } else {
                    self.world.set_location(id, Location::Position(new_pos));
                }
                return true;
            }
        }
        false
    }

    // TODO: dedup with attack_position
    fn bump_attack(&mut self, attacker: Entity, target: Entity) -> bool {
        if self.is_player(attacker) == self.is_player(target) { return false; }

        let actor_type = self.world.entity(attacker).get::<EntityType>().ok().cloned();
        if let Some(actor_type) = actor_type {
            if let EntityClass::Actor { damage, .. } = actor_type.data().class {
                if self.is_player(attacker) && self.find_item(EntityType::Sword).is_some() {
                    self.add_damage(target, 3);
                } else {
                    self.add_damage(target, damage);
                }
                return true;
            }
        }
        false
    }

    fn attack_position(&mut self, attacker: Entity, pos: Position, damage: i8) -> bool {
        let target = self.get_actor_by_position(pos);
        if let Some(target) = target {
            if self.is_player(attacker) == self.is_player(target) { return false; }
            self.add_damage(target, damage);
            true
        } else {
            false
        }
    }

    fn is_player(&self, id: Entity) -> bool {
        self.world.entity(id).get::<IsPlayer>().is_ok()
    }

    fn is_actor(&self, id: Entity) -> bool {
        self.world.entity(id).get::<EntityType>()
            .map(|t| t.data().is_actor()).unwrap_or(false)
    }

    fn add_damage(&mut self, target: Entity, damage: i8) {
        if let Ok(&EntityClass::Actor { max_health, .. }) =
            self.world.entity(target).get::<EntityType>().map(|t| &t.data().class)
        {
            let total_damage = {
                let mut target_entity = self.world.entity_mut(target);
                let target_damage: &mut Damage = target_entity.get_or_default();
                target_damage.0 += damage;
                if target_damage.0 < 0 { target_damage.0 = 0; }
                target_damage.0
            };
            if total_damage >= max_health {
                self.kill_entity(target);
            }
        }
    }

    fn kill_entity(&mut self, id: Entity) {
        let old_type = self.world.entity_mut(id).insert(EntityType::Corpse);
        if let Some(corpse_type) = old_type {
            self.world.entity_mut(id).insert(Corpse {
                turn_created: self.current_turn,
                original_type: corpse_type,
            });
        }
        self.world.entity_mut(id).remove::<AiState>();
    }

    fn put_entity(&mut self, t: EntityType, p: Position) {
        let id = Entity(self.next_id);
        self.next_id += 1;
        if t == EntityType::Player {
            self.world.entity_mut(id).insert(IsPlayer);
        } else if t.data().is_actor() {
            self.world.entity_mut(id).insert(AiState::Waiting);
        }
        self.world.entity_mut(id).insert(t);
        self.world.set_location(id, Location::Position(p));
    }

    fn inventory_count(&self, t: EntityType) -> i32 {
        let mut count = 0;
        if let Some(player) = self.find_player() {
            if let Ok(&Contents(ref inventory)) = self.world.entity(player).get() {
                for &item_id in inventory {
                    if self.world.entity(item_id).get() == Ok(&t) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    fn find_item(&self, t: EntityType) -> Option<Entity> {
        if let Some(player) = self.find_player() {
            if let Ok(&Contents(ref inventory)) = self.world.entity(player).get() {
                for &item_id in inventory {
                    if self.world.entity(item_id).get() == Ok(&t) {
                        return Some(item_id);
                    }
                }
            }
        }
        None
    }

    fn find_corpse(&self) -> Option<Entity> {
        if let Some(pos) = self.player_position() {
            if let Ok(&Contents(ref contents)) = self.world.entity(pos).get() {
                for &id in contents {
                    if self.world.entity(id).get() == Ok(&EntityType::Corpse) {
                        return Some(id);
                    }
                }
            }
        }
        None
    }

    fn consume_item(&mut self, t: EntityType) -> bool {
        if let Some(item) = self.find_item(t) {
            self.world.remove_location(item);
            true
        } else {
            false
        }
    }

    fn update_smells(&mut self) {
        let mut updated_smells: BTreeMap<_, _> =
            self.world.component::<Tile>().iter().filter_map(|(pos, &tile)| {
                match tile {
                    Tile::Wall => None,
                    Tile::ShallowWater | Tile::DeepWater => Some((pos, 1)),
                    _ => Some((pos, 2)),
                }
            }).map(|(pos, n)| {
                let mut rand = self.rand;
                let new_strength = (0..n).filter_map(|_| {
                    let &dir = rand.choose(&ALL_DIRECTIONS).unwrap();
                    self.smell_strength.get(&pos.step(dir))
                }).map(Clone::clone)
                    .min().unwrap_or(::std::i32::MAX)
                    .saturating_add(1);
                (pos, new_strength)
            }).collect();

        // TODO: Find some way to make stacks of corpses smell more
        for (id, &Corpse { turn_created, .. }) in self.world.component::<Corpse>().iter() {
            self.locate_entity(id).map(|pos| {
                let strength = (self.current_turn - turn_created) / 20;
                match updated_smells.entry(pos) {
                    Entry::Vacant(entry) => {
                        entry.insert(strength);
                    }
                    Entry::Occupied(mut entry) => {
                        let strength = ::std::cmp::min(*entry.get(), strength);
                        entry.insert(strength);
                    }
                }
            });
        }

        self.smell_strength = updated_smells;
    }

    fn locate_entity(&self, mut id: Entity) -> Option<Position> {
        for _ in 0..32 { // TODO: actual cycle detection?
            match self.world.entity(id).get() {
                Err(_) => { return None; }
                Ok(&Location::Entity(e)) => { id = e; }
                Ok(&Location::Position(p)) => { return Some(p); }
            }
        }
        None
    }
}
