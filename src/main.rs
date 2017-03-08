#![feature(specialization)]
#![allow(dead_code)]  // TODO: remove me

extern crate pancurses;

use pancurses::{Input, Window};

#[macro_use]
mod engine;
use engine::*;

mod game;
use game::*;


fn main() {
    let mut g = Game::new();
    init_game(&mut g);

    let window = pancurses::initscr();
    window.keypad(true);
    pancurses::noecho();
    pancurses::cbreak();
    pancurses::curs_set(0);

    if pancurses::has_colors() {
        pancurses::start_color();
    }

    for i in 0..64 {
        let (fg, bg) = (i%8, i/8);
        pancurses::init_pair(i+1, fg, bg);
    }

    fn put_cell(window: &Window, y: i32, x: i32, c: Cell) {
        window.mvaddch(y, x, c.ch);
        let attr = if c.bold { pancurses::A_BOLD } else { pancurses::A_NORMAL };
        window.mvchgat(y, x, 1, attr, (c.fg as i16) + (c.bg as i16)*8 + 1);
    }

    loop {
        window.clear();
        let (max_y, max_x) = window.get_max_yx();
        for y in 0..max_y {
            for x in 0..max_x {
                put_cell(&window, y, x, g.render(Position { x, y }));
            }
        }
        window.refresh();

        let key = window.getch();
        if let Some(action) = key.and_then(|key| match key {
            // space bar
            Input::Character(' ') => Some(Action::Wait),
            // arrow keys
            Input::KeyDown => Some(Action::Move(Direction::South)),
            Input::KeyUp => Some(Action::Move(Direction::North)),
            Input::KeyLeft => Some(Action::Move(Direction::West)),
            Input::KeyRight => Some(Action::Move(Direction::East)),
            // allow insert / delete / page up / page down for diagonals
            Input::KeyIC => Some(Action::Move(Direction::NorthWest)),
            Input::KeyDC => Some(Action::Move(Direction::SouthWest)),
            Input::KeyPPage => Some(Action::Move(Direction::NorthEast)),
            Input::KeyNPage => Some(Action::Move(Direction::SouthEast)),
            // number keys
            Input::Character('1') => Some(Action::Move(Direction::SouthWest)),
            Input::Character('2') => Some(Action::Move(Direction::South)),
            Input::Character('3') => Some(Action::Move(Direction::SouthEast)),
            Input::Character('4') => Some(Action::Move(Direction::West)),
            Input::Character('5') => Some(Action::Wait),
            Input::Character('6') => Some(Action::Move(Direction::East)),
            Input::Character('7') => Some(Action::Move(Direction::NorthWest)),
            Input::Character('8') => Some(Action::Move(Direction::North)),
            Input::Character('9') => Some(Action::Move(Direction::NorthEast)),
            // vi keys
            Input::Character('h') => Some(Action::Move(Direction::West)),
            Input::Character('j') => Some(Action::Move(Direction::South)),
            Input::Character('k') => Some(Action::Move(Direction::North)),
            Input::Character('l') => Some(Action::Move(Direction::East)),
            Input::Character('y') => Some(Action::Move(Direction::NorthWest)),
            Input::Character('u') => Some(Action::Move(Direction::NorthEast)),
            Input::Character('b') => Some(Action::Move(Direction::SouthWest)),
            Input::Character('n') => Some(Action::Move(Direction::SouthEast)),
            // handle home and end to allow the numpad to work with numlock off
            Input::Character('\x1b') => {
                window.nodelay(true);
                let mut keys = vec![];
                while let Some(key) = window.getch() { keys.push(key) }
                window.nodelay(false);
                if keys == [Input::Character('['), Input::Character('1'), Input::Character('~')] {
                    Some(Action::Move(Direction::NorthWest))
                } else if keys == [Input::Character('['), Input::Character('4'), Input::Character('~')] {
                    Some(Action::Move(Direction::SouthWest))
                } else {
                    while let Some(key) = keys.pop() {
                        window.ungetch(&key);
                    }
                    None
                }
            }
            _ => None,
        }) {
            g.take_turn(action);
        } else {
            match key {
                None => {}
                Some(Input::Character(c)) => { match c {
                    'q' | 'Q' => { break; }
                    '\x1b' => {
                        // handle ESC, but ignore things like Alt+key
                        window.nodelay(true);
                        if window.getch().is_none() { break; }
                        while window.getch().is_some() {}
                        window.nodelay(false);
                    }
                    _ => {}
                }}
                _ => {}
            }
        }
    }

    pancurses::endwin();
}
