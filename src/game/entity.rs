use ::engine::*;

enum EntityClass {
    Item,
    Creature,
    Player,
}

enum Color {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
    DarkGray = 8,
    DarkRed = 9,
    DarkGreen = 10,
    DarkYellow = 11,
    DarkBlue = 12,
    DarkMagenta = 13,
    DarkCyan = 14,
    Gray = 15,
}

struct EntityData {
    class: EntityClass,
    name: &'static str,
    ch: char,
    color: Color,
}

macro_rules! entity_data {
    ($($name:ident: { $($tt:tt)* })*) => {
        pub enum EntityType { $($name,)* }

        impl EntityType {
            fn data(&self) -> &'static EntityData {
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
        color: Color::Gray,
    }
    Muskrat: {
        class: EntityClass::Creature,
        name: "muskrat",
        ch: 'm',
        color: Color::DarkYellow,
    }
    Player: {
        class: EntityClass::Player,
        name: "player",
        ch: '@',
        color: Color::Yellow,
    }
}
