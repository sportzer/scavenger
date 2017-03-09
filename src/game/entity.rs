use ::engine::*;
use super::Color;

#[derive(Debug)]
pub enum EntityClass {
    Item,
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
            EntityClass::Item => true,
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
        ch: ',',
        color: Some(Color::White),
        class: EntityClass::Item,
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
