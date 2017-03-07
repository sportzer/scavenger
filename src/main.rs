#![feature(specialization)]
#![allow(dead_code)]  // TODO: remove me

extern crate pancurses;

use pancurses::Window;

#[macro_use]
mod engine;
use engine::*;

mod game;
use game::*;


fn main() {
    let mut g = Game::new();

    g.put_entity(EntityType::Rat, Position { x: 3, y: 3 });
    g.put_entity(EntityType::Player, Position { x: 3, y: 3 });
    g.put_entity(EntityType::Rock, Position { x: 3, y: 3 });

    g.put_entity(EntityType::Rat, Position { x: 5, y: 3 });
    g.put_entity(EntityType::Rock, Position { x: 5, y: 3 });

    g.put_entity(EntityType::Rock, Position { x: 4, y: 4 });
    g.put_entity(EntityType::Rat, Position { x: 4, y: 4 });

    g.put_entity(EntityType::Rock, Position { x: 6, y: 4 });

    let window = pancurses::initscr();
    pancurses::noecho();
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
        window.mvchgat(y, x, 1, pancurses::A_NORMAL,
                       (c.fg as i16) + (c.bg as i16)*8 + 1);
    }

    window.clear();
    let (max_y, max_x) = window.get_max_yx();
    for y in 0..max_y {
        for x in 0..max_x {
            put_cell(&window, y, x, g.render(Position { x, y }));
        }
    }
    window.refresh();

    window.getch();
    pancurses::endwin();
}
