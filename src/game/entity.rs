use super::Color;
use ::engine::*;

#[derive(Debug, Eq, PartialEq)]
pub enum EntityClass {
    Item,
    Creature,
    Player,
}

pub struct EntityData {
    pub class: EntityClass,
    pub name: &'static str,
    pub ch: char,
    pub color: Color,
}

macro_rules! entity_data {
    ($($name:ident: { $($tt:tt)* })*) => {
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
        class: EntityClass::Item,
        name: "rock",
        ch: ',',
        color: Color::White,
    }
    Rat: {
        class: EntityClass::Creature,
        name: "rat",
        ch: 'r',
        color: Color::White,
    }
    Player: {
        class: EntityClass::Player,
        name: "player",
        ch: '@',
        color: Color::Yellow,
    }
}
