use super::*;

pub enum EntityClass {
    Item {
        display_priority: i8,
    },
    Actor {
        max_health: i8,
        max_stamina: i8,
        max_satiation: i16,
        fov_range: i8,
        damage: i8,
        smelling: i32,
        ai: Option<Ai>,
    },
}

pub struct Ai {
    attack: bool,
    flee: bool,
    wanders: bool
}

pub struct EntityData {
    pub name: &'static str,
    pub ch: char,
    pub color: Option<Color>,
    pub class: EntityClass,
}

impl EntityData {
    pub fn is_item(&self) -> bool {
        match self.class {
            EntityClass::Item { .. } => true,
            _ => false,
        }
    }

    pub fn is_actor(&self) -> bool {
        match self.class {
            EntityClass::Actor { .. } => true,
            _ => false,
        }
    }
}

macro_rules! entity_data {
    ($($name:ident: { $($tt:tt)* })*) => {
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        pub enum EntityType { $($name,)* }

        impl EntityType {
            pub fn data(&self) -> &'static EntityData {
                match self { $(
                    &EntityType::$name => {
                        static DATA: &'static EntityData = &EntityData { $($tt)* };
                        &DATA
                    }
                )* }
            }
        }
    };
}

impl Component for EntityType {}

entity_data! {
    Skeleton: {
        name: "skeleton",
        ch: '%',
        color: Some(Color::White),
        class: EntityClass::Item {
            display_priority: 10,
        },
    }
    Rock: { // (t)hrow
        name: "rock",
        ch: '*',
        color: Some(Color::White),
        class: EntityClass::Item {
            display_priority: 9,
        },
    }
    Herb: { // (e)at to heal
        name: "healing herbs",
        ch: '+',
        color: Some(Color::Green),
        class: EntityClass::Item {
            display_priority: 7,
        },
    }
    Arrow: { // fired by bow
        name: "arrow",
        ch: '/',
        color: Some(Color::Yellow),
        class: EntityClass::Item {
            display_priority: 6,
        },
    }
    Diamond: { // score item
        name: "diamond",
        ch: '*',
        color: Some(Color::Cyan),
        class: EntityClass::Item {
            display_priority: 5,
        },
    }
    Sword: { // melee weapon
        name: "sword",
        ch: '|',
        color: Some(Color::White),
        class: EntityClass::Item {
            display_priority: 3,
        },
    }
    Bow: { // (f)ire arrows
        name: "bow",
        ch: '}',
        color: Some(Color::Yellow),
        class: EntityClass::Item {
            display_priority: 2,
        },
    }
    Corpse: { // (d)rop to attract monsters
        name: "corpse",
        ch: '%',
        color: Some(Color::Red),
        class: EntityClass::Item {
            display_priority: 1,
        },
    }
    Scroll: { // not a real item... for display purposes
        name: "scroll",
        ch: '?',
        color: Some(Color::White),
        class: EntityClass::Item {
            display_priority: 0,
        },
    }

    Rat: {
        name: "rat",
        ch: 'r',
        color: Some(Color::White),
        class: EntityClass::Actor {
            max_health: 2,
            max_stamina: 2,
            max_satiation: 10,
            fov_range: 3,
            damage: 1,
            smelling: 40,
            ai: Some(Ai {
                attack: true,
                flee: true,
                wanders: true,
            }),
        },
    }
    Deer: {
        name: "deer",
        ch: 'd',
        color: Some(Color::Yellow),
        class: EntityClass::Actor {
            max_health: 5,
            max_stamina: 5,
            max_satiation: 40,
            fov_range: 4,
            damage: 0,
            smelling: 12,
            ai: Some(Ai {
                attack: false,
                flee: true,
                wanders: true,
            }),
        },
    }
    Wolf: {
        name: "wolf",
        ch: 'w',
        color: Some(Color::White),
        class: EntityClass::Actor {
            max_health: 5,
            max_stamina: 5,
            max_satiation: 40,
            fov_range: 4,
            damage: 2,
            smelling: 24,
            ai: Some(Ai {
                attack: true,
                flee: false,
                wanders: true,
            }),
        },
    }
    Dragon: {
        name: "dragon",
        ch: 'D',
        color: Some(Color::Green),
        class: EntityClass::Actor {
            max_health: 15,
            max_stamina: 10,
            max_satiation: 200,
            fov_range: 5,
            damage: 3,
            smelling: 16,
            ai: Some(Ai {
                attack: true,
                flee: true,
                wanders: false,
            }),
        },
    }

    Player: {
        name: "player",
        ch: '@',
        color: None,
        class: EntityClass::Actor {
            max_health: 10,
            max_stamina: 10,
            max_satiation: 100,
            fov_range: 4,
            damage: 1,
            smelling: 8,
            ai: None,
        },
    }
}

pub struct Corpse {
    pub turn_created: i32,
    pub original_type: EntityType,
}

impl Component for Corpse {}

#[derive(Copy, Clone)]
pub enum AiState {
    Waiting,
    Wandering(Position),
    Fleeing(Entity, Position),
    Hunting(Entity, Position),
}

impl Component for AiState {}

fn step_towards(g: &mut Game, actor: Entity, pos: Position) -> Position {
    let actor_pos: Option<Location> = g.world.entity(actor).get().ok().cloned();
    if let Some(Location::Position(actor_pos)) = actor_pos {
        if actor_pos != pos {
            let x_offset = pos.x - actor_pos.x;
            let y_offset = pos.y - actor_pos.y;
            let abs_x_offset = x_offset.abs();
            let abs_y_offset = y_offset.abs();
            if abs_y_offset >= abs_x_offset {
                if g.rand.gen_range(0, abs_y_offset) >= abs_x_offset {
                    return Position { x: actor_pos.x, y: actor_pos.y + y_offset.signum() };
                }
            } else {
                if g.rand.gen_range(0, abs_x_offset) >= abs_y_offset {
                    return Position { x: actor_pos.x + x_offset.signum(), y: actor_pos.y };
                }
            }
            return Position { x: actor_pos.x + x_offset.signum(), y: actor_pos.y + y_offset.signum() };
        }
    }
    pos
}

// TODO: dedup with Game::move_entity
fn move_towards(g: &mut Game, actor: Entity, pos: Position) -> bool {
    // TODO: allow running if sufficient stamina
    let actor_pos: Option<Location> = g.world.entity(actor).get().ok().cloned();
    let new_pos = step_towards(g, actor, pos);
    if actor_pos == Some(Location::Position(new_pos)) { return false; }
    if g.get_tile(new_pos).is_walkable() {
        let target = g.get_actor_by_position(new_pos);
        if target.is_err() {
            g.world.set_location(actor, Location::Position(new_pos));
            return true;
        }
    }
    false
}

fn move_randomly(g: &mut Game, actor: Entity) {
    let actor_pos: Option<Location> = g.world.entity(actor).get().ok().cloned();
    if let Some(Location::Position(actor_pos)) = actor_pos {
        for _ in 0..8 {
            let &dir = g.rand.choose(&ALL_DIRECTIONS).unwrap();
            let moved = move_towards(g, actor, actor_pos.step(dir));
            if moved {
                return;
            }
        }
    }
}

impl AiState {
    pub fn take_turn(mut self, g: &mut Game, actor: Entity) -> AiState {
        let actor_type = g.world.entity(actor).get::<EntityType>().ok().cloned();
        let actor_pos = g.world.entity(actor).get::<Location>().ok().cloned();
        if let (Some(actor_type), Some(Location::Position(actor_pos))) = (actor_type, actor_pos) {
            if let EntityClass::Actor { fov_range, smelling, ai: Some(ref ai), .. } = actor_type.data().class {
                let player = (|| {
                    let distance = g.world.entity(actor_pos).get::<IsVisible>().map(|v| v.0);
                    if distance.map(|d| d <= fov_range).unwrap_or(false) {
                        if let Some(player_id) = g.find_player() {
                            if let Some(Location::Position(player_pos)) =
                                g.world.entity(player_id).get().ok().cloned()
                            {
                                return Some((player_id, player_pos));
                            }
                        }
                    }
                    None
                }) ();

                let fov_range = fov_range as i32;
                match self {
                    AiState::Waiting => {}
                    AiState::Wandering(pos) => {
                        let moved = move_towards(g, actor, pos);
                        if !moved && ai.wanders {
                            self = AiState::Waiting;
                        }
                    }
                    AiState::Fleeing(id, pos) => {
                        let moved = move_towards(g, actor, Position {
                            x: actor_pos.x*2 - pos.x,
                            y: actor_pos.y*2 - pos.y,
                        });
                        if !moved {
                            move_randomly(g, actor);
                        } else {
                            if let Some((player_id, player_pos)) = player {
                                if player_id == id {
                                    return AiState::Fleeing(id, player_pos);
                                }
                            }
                            if actor_pos.distance_sq(pos) > 16*fov_range*fov_range {
                                return AiState::Waiting;
                            }
                        }
                        return self;
                    }
                    AiState::Hunting(id, pos) => {
                        if let Some(Location::Position(target_pos)) =
                            g.world.entity(id).get().ok().cloned()
                        {
                            if actor_pos.distance_sq(target_pos) < 4 {
                                // TODO: if stamina is at 0, do something else
                                // TODO: really ignore Result?
                                let _ = g.bump_attack(actor, id);
                                return self;
                            }
                        }
                        let moved = move_towards(g, actor, pos);
                        if !moved {
                            if pos == actor_pos {
                                self = AiState::Waiting;
                            } else {
                                move_randomly(g, actor);
                            }
                        }
                    }
                }

                if let Some((player_id, player_pos)) = player {
                    if ai.attack {
                        return AiState::Hunting(player_id, player_pos);
                    } else if ai.flee {
                        return AiState::Fleeing(player_id, player_pos);
                    }
                }

                if let Some(&smell) = g.smell_strength.get(&actor_pos) {
                    if smell <= smelling {
                        if let Some(dir) = (0..2).filter_map(|_| {
                            let &dir = g.rand.choose(&ALL_DIRECTIONS).unwrap();
                            if let Some(&dir_smell) = g.smell_strength.get(&actor_pos.step(dir)) {
                                if dir_smell < smell {
                                    return Some(dir);
                                }
                            }
                            None
                        }).next() {
                            if ai.attack {
                                return AiState::Wandering(actor_pos.step(dir).step(dir).step(dir));
                            } else if ai.flee {
                                let dir = dir.reverse();
                                return AiState::Wandering(actor_pos.step(dir).step(dir).step(dir));
                            }
                        }
                    }
                }

                // if ai.wanders {
                    if let AiState::Waiting = self {
                        return AiState::Wandering(Position {
                            x: actor_pos.x + g.rand.gen_range(0, 2*fov_range)
                                - g.rand.gen_range(0, 2*fov_range),
                            y: actor_pos.y + g.rand.gen_range(0, 2*fov_range)
                                - g.rand.gen_range(0, 2*fov_range),
                        });
                    }
                // }

                // TODO: herding / pack behavior?
            }
        }
        self
    }
}
