use ::engine::*;
use super::{Color, Cell};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Tile {
    Wall,
    Ground,
    ShallowWater,
    DeepWater,
    ShortGrass,
    LongGrass,
}

impl Component for Tile {}

impl Tile {
    pub fn render(&self) -> Cell {
        match self {
            &Tile::Wall => Cell {
                ch: '#',
                fg: Color::Black,
                bg: Color::Yellow,
                bold: false,
            },
            &Tile::Ground => Cell {
                ch: '.',
                fg: Color::Yellow,
                bg: Color::Black,
                bold: false,
            },
            &Tile::ShallowWater => Cell {
                ch: '=',
                fg: Color::Cyan,
                bg: Color::Black,
                bold: false,
            },
            &Tile::DeepWater => Cell {
                ch: '=',
                fg: Color::Cyan,
                bg: Color::Blue,
                bold: false,
            },
            &Tile::ShortGrass => Cell {
                ch: '.',
                fg: Color::Green,
                bg: Color::Black,
                bold: false,
            },
            &Tile::LongGrass => Cell {
                ch: '"',
                fg: Color::Green,
                bg: Color::Black,
                bold: false,
            },
        }
    }

    pub fn is_walkable(&self) -> bool {
        self != &Tile::Wall && self != &Tile::DeepWater
    }
}
