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
    // TODO: return a ActionResult of some sort?
    fn remove_from_contents<I: Id>(&mut self, entity: Entity, location: I)
        where GameWorld: EntityComponent<I, Contents>
    {
        let is_empty = self.entity_mut(location).get_mut::<Contents>().map(|c| {
            c.0.remove(&entity);
            c.0.is_empty()
        }).ok();
        if is_empty == Some(true) {
            // TODO: actually just ignore?
            let _ = self.entity_mut(location).remove::<Contents>();
        }
    }

    fn remove_location(&mut self, id: Entity) -> ActionResult<Location>
    {
        let old_location = self.entity_mut(id).remove();
        match old_location {
            Ok(Location::Entity(e)) => { self.remove_from_contents(id, e); }
            Ok(Location::Position(p)) => { self.remove_from_contents(id, p); }
            Err(_) => {}
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

    pub fn take_turn(&mut self, action: Action) -> ActionResult<()> {
        let player = self.player()?;
        let player_pos = self.player_position()?;

        // TODO: only abort rest of turn if uncommitted
        match action {
            Action::Wait => {}
            Action::Move(dir) => {
                self.move_entity(player, dir)?;
                // TODO: really ignore result?
                let _ = self.auto_pickup();
            }
            Action::EatHerb => {
                self.consume_item(EntityType::Herb)?;
                self.add_damage(player, -1);
            }
            Action::ReadScroll => {
                self.recall_turns = Some(self.rand.gen_range(20, 30));
            }
            Action::GetCorpse => {
                let corpse = self.find_corpse()?;
                self.world.set_location(corpse, Location::Entity(player));
            }
            Action::DropCorpse => {
                let corpse = self.find_item(EntityType::Corpse)?;
                self.world.set_location(corpse, Location::Position(player_pos));
            }
            Action::ThrowRock(pos) => {
                let &IsVisible(dist) = self.world.entity(pos).get()?;
                if dist > self.player_fov_range() {
                    return self.world.err();
                }
                let rock = self.find_item(EntityType::Rock)?;
                self.attack_position(player, pos, 1)?;
                // TODO: do something with ActionResult?
                let _ = self.world.remove_location(rock);
            }
            Action::FireBow(dir) => {
                self.find_item(EntityType::Bow)?;
                self.consume_item(EntityType::Arrow)?;
                let mut pos = player_pos;
                for _ in 0..8 {
                    pos = pos.step(dir);
                    if self.get_tile(pos).is_obstructed() {
                        break;
                    }
                    if self.attack_position(player, pos, 2).is_ok() {
                        break;
                    }
                }
            }
        }

        update_fov(self);

        let creatures: Vec<Entity> = self.world.component::<AiState>().ids().collect();
        for id in creatures {
            if let Ok(&state) = self.world.entity(id).get::<AiState>() {
                let new_state = state.take_turn(self, id);
                if let Ok(state_mut) = self.world.entity_mut(id).get_mut::<AiState>() {
                    *state_mut = new_state;
                }
            }
        }

        if let Some(turns) = self.recall_turns {
            self.recall_turns = Some(turns - 1);
            if self.recall_turns == Some(0) {
                // TODO: really ignore result?
                let _ = self.world.remove_location(player);
            }
        }

        update_fov(self);
        self.update_smells();
        self.current_turn += 1;

        Ok(())
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

    // TODO: make this return a Result of some sort
    pub fn player_status(&self) -> Option<PlayerStatus> {
        if let Ok(player) = self.player() {
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

    pub fn player_position(&self) -> QueryResult<Position> {
        self.entity_position(self.player()?)
    }

    pub fn fov_bounding_rect(&self) -> QueryResult<Rect> {
        self.player_position().map(|pos| {
            let fov_range = self.player_fov_range();
            let mut rect = Rect::from(pos);
            for (pos, &IsVisible(dist)) in self.world.component::<IsVisible>().iter() {
                if dist <= fov_range {
                    rect.extend(pos);
                }
            }
            rect
        })
    }
}

impl Game {
    fn player(&self) -> QueryResult<Entity> {
        // TODO: make sure there is at most one player?
        self.world.component::<IsPlayer>().ids().next().ok_or(())
    }

    fn auto_pickup(&mut self) -> QueryResult<()> {
        let player = self.player()?;
        let pos = self.player_position()?;
        let new_items: Vec<_> = self.world.entity(pos).get::<Contents>()
            .iter()
            .flat_map(|c| &c.0)
            .cloned()
            .filter(
                |&id| self.world.entity(id).get().map(
                    |entity_type| [
                        EntityType::Sword, EntityType::Bow,
                        EntityType::Arrow, EntityType::Rock,
                        EntityType::Herb, EntityType::Diamond,
                    ].contains(entity_type)
                ).unwrap_or(false)
            ).collect();
        for item in new_items {
            self.world.set_location(item, Location::Entity(player));
        }
        Ok(())
    }

    // TODO: make this return a Result of some sort?
    fn player_fov_range(&self) -> i8 {
        // TODO: dynamic view distance
        if let Ok(&EntityClass::Actor { fov_range, .. }) =
            self.player()
            .and_then(|id| self.world.entity(id).get::<EntityType>())
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

    fn get_actor_by_position(&self, pos: Position) -> QueryResult<Entity> {
        // TODO: make sure there is only one valid target in location?
        // TODO: clean up error handling
        self.world.entity(pos).get::<Contents>()
            .and_then(
                |contents| contents.0.iter().cloned().find(|&id| {
                    self.world.entity(id).get::<EntityType>().map(
                        |t| t.data().is_actor()
                    ).unwrap_or(false)
                }).ok_or(())
            )
    }

    fn move_entity(&mut self, id: Entity, dir: Direction) -> ActionResult<()> {
        let pos = self.entity_position(id)?;
        let new_pos = pos.step(dir);
        // TODO: allow attacking enemies on unwalkable tiles?
        if !self.get_tile(new_pos).is_walkable() {
            return self.world.err();
        }
        if let Ok(target) = self.get_actor_by_position(new_pos) {
            self.bump_attack(id, target)
        } else {
            self.world.set_location(id, Location::Position(new_pos));
            Ok(())
        }
    }

    fn bump_attack(&mut self, attacker: Entity, target: Entity) -> ActionResult<()> {
        let bump_damage = self.bump_damage(attacker)?;
        self.attack_entity(attacker, target, bump_damage)
    }

    fn bump_damage(&mut self, attacker: Entity) -> ActionResult<i8> {
        let actor_type = self.world.entity(attacker).get::<EntityType>()?;
        if let EntityClass::Actor { damage, .. } = actor_type.data().class {
            Ok(if self.is_player(attacker) && self.find_item(EntityType::Sword).is_ok() {
                3
            } else {
                damage
            })
        } else {
            self.world.err()
        }
    }

    fn attack_position(&mut self, attacker: Entity, pos: Position, damage: i8) -> ActionResult<()> {
        let target = self.get_actor_by_position(pos)?;
        self.attack_entity(attacker, target, damage)
    }

    fn attack_entity(&mut self, attacker: Entity, target: Entity, damage: i8) -> ActionResult<()> {
        if self.is_player(attacker) == self.is_player(target) {
            self.world.err()
        } else {
            self.add_damage(target, damage);
            Ok(())
        }
    }

    fn is_player(&self, id: Entity) -> bool {
        self.world.entity(id).get::<IsPlayer>().is_ok()
    }

    // TODO: make this return a ActionResult of some sort?
    fn add_damage(&mut self, target: Entity, damage: i8) {
        // TODO: add helpers for getting entities as actors
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

    // TODO: make this return a ActionResult of some sort?
    fn kill_entity(&mut self, id: Entity) {
        let old_type = self.world.entity_mut(id).insert(EntityType::Corpse);
        if let Some(corpse_type) = old_type {
            self.world.entity_mut(id).insert(Corpse {
                turn_created: self.current_turn,
                original_type: corpse_type,
            });
        }
        let _ = self.world.entity_mut(id).remove::<AiState>();
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
        if let Ok(player) = self.player() {
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

    fn find_item(&self, t: EntityType) -> QueryResult<Entity> {
        let player = self.player()?;
        let &Contents(ref inventory) = self.world.entity(player).get()?;
        for &item_id in inventory {
            if self.world.entity(item_id).get() == Ok(&t) {
                return Ok(item_id);
            }
        }
        Err(())
    }

    fn find_corpse(&self) -> QueryResult<Entity> {
        let pos = self.player_position()?;
        let &Contents(ref contents) = self.world.entity(pos).get()?;
        for &id in contents {
            if self.world.entity(id).get() == Ok(&EntityType::Corpse) {
                return Ok(id);
            }
        }
        Err(())
    }

    fn consume_item(&mut self, t: EntityType) -> ActionResult<()> {
        let item = self.find_item(t)?;
        // TODO: really ignore result?
        // TODO: destroy item too?
        let _ = self.world.remove_location(item);
        Ok(())
    }

    // TODO: make this return a ActionResult of some sort?
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
            // TODO: really ignore errors?
            let _ = self.locate_entity(id).map(|pos| {
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

    fn locate_entity(&self, mut id: Entity) -> QueryResult<Position> {
        for _ in 0..32 { // TODO: actual cycle detection?
            match self.world.entity(id).get()? {
                &Location::Entity(e) => { id = e; }
                &Location::Position(p) => { return Ok(p); }
            }
        }
        Err(())
    }

    fn entity_position(&self, id: Entity) -> QueryResult<Position> {
        match self.world.entity(id).get()? {
            &Location::Entity(_) => Err(()),
            &Location::Position(p) => Ok(p),
        }
    }
}
