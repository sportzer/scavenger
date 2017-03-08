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

    window.clear();
    let (max_y, max_x) = window.get_max_yx();
    for y in 0..max_y {
        for x in 0..max_x {
            put_cell(&window, y, x, g.render(Position { x, y }));
        }
    }
    window.refresh();

    let key = window.getch();
    pancurses::endwin();
    println!("{:?}", key);
}
