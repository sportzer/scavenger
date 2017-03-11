use ::engine::*;
use super::{Color, Entity};

#[derive(Debug)]
pub enum EntityClass {
    Item {
        display_priority: i8,
    },
    Actor {
        max_health: i8,
        max_stamina: i8,
        max_satiation: i16,
    },
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
    Rock: {
        name: "rock",
        ch: '*',
        color: Some(Color::White),
        class: EntityClass::Item {
            display_priority: 10,
        },
    }
    Corpse: {
        name: "corpse",
        ch: '%',
        color: Some(Color::Red),
        class: EntityClass::Item {
            display_priority: 5,
        },
    }
    Rat: {
        name: "rat",
        ch: 'r',
        color: Some(Color::White),
        class: EntityClass::Actor {
            max_health: 2,
            max_stamina: 2,
            max_satiation: 20,
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
        },
    }
}

pub struct CorpseType(pub EntityType);

impl Component for CorpseType {}

pub enum AiState {
    Waiting,
    Wandering(Position),
    Fleeing(Position),
    Searching(Position),
    Hunting(Entity, Position),
}

impl Component for AiState {}
