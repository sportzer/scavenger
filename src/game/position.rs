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
    Tree,
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
            &Tile::Tree => Cell {
                ch: '#',
                fg: Color::Green,
                bg: Color::Black,
                bold: false,
            },
        }
    }

    pub fn render_memory(&self) -> Cell {
        let cell = self.render();
        if cell.bg == Color::Black {
            Cell {
                ch: cell.ch,
                fg: Color::White,
                bg: Color::Black,
                bold: false,
            }
        } else {
            Cell {
                ch: cell.ch,
                fg: Color::Black,
                bg: Color::White,
                bold: false,
            }
        }
    }

    pub fn is_walkable(&self) -> bool {
        // TODO: account for swimming and flying critters
        self != &Tile::Wall && self != &Tile::DeepWater && self != &Tile::Tree
    }

    pub fn is_obstructed(&self) -> bool {
        self == &Tile::Wall || self == &Tile::Tree
    }
}
