use super::Id;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

pub static ALL_DIRECTIONS: [Direction; 8] = [
    Direction::North,
    Direction::NorthEast,
    Direction::East,
    Direction::SouthEast,
    Direction::South,
    Direction::SouthWest,
    Direction::West,
    Direction::NorthWest,
];

pub static ORTHOGONAL_DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

pub static DIAGONAL_DIRECTIONS: [Direction; 4] = [
    Direction::NorthEast,
    Direction::SouthEast,
    Direction::SouthWest,
    Direction::NorthWest,
];

impl Direction {
    pub fn reverse(self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::NorthEast => Direction::SouthWest,
            Direction::East => Direction::West,
            Direction::SouthEast => Direction::NorthWest,
            Direction::South => Direction::North,
            Direction::SouthWest => Direction::NorthEast,
            Direction::West => Direction::East,
            Direction::NorthWest => Direction::SouthEast,
        }
    }

    pub fn is_orthogonal(&self) -> bool {
        self == &Direction::North
        || self == &Direction::East
        || self == &Direction::South
        || self == &Direction::West
    }
}

// deriving Ord here is probably the wrong thing to do, but whatever,
// I just want to use these as BTreeMap keys and get on with my life
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Id for Position {}

impl Position {
    pub fn step(self, d: Direction) -> Position {
        match d {
            Direction::North => Position { x: self.x, y: self.y - 1 },
            Direction::NorthEast => Position { x: self.x + 1, y: self.y - 1 },
            Direction::East => Position { x: self.x + 1, y: self.y },
            Direction::SouthEast => Position { x: self.x + 1, y: self.y + 1 },
            Direction::South => Position { x: self.x, y: self.y + 1 },
            Direction::SouthWest => Position { x: self.x - 1, y: self.y + 1 },
            Direction::West => Position { x: self.x - 1, y: self.y },
            Direction::NorthWest => Position { x: self.x - 1, y: self.y - 1 },
        }
    }
}
