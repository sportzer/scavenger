#![allow(dead_code)]  // TODO: remove me

extern crate pancurses;
extern crate rand;

use pancurses::{Input, Window};
use rand::Rng;

#[macro_use]
mod engine;
use engine::*;

mod game;
use game::*;

#[derive(Eq, PartialEq)]
enum InputMode {
    Normal,
    Fire,
    Throw(Position),
    None,
}

fn main() {
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

    'application: loop {
        window.clear();
        let mut g = Game::new(rand::thread_rng().gen());
        let mut display_center = g.player_position()
            .unwrap_or(Position { x: 0, y: 0 });
        let mut mode = InputMode::Normal;

        'game: loop {
            window.erase();
            let (max_y, max_x) = window.get_max_yx();

            let padding = 7;
            let old_center = display_center;
            if let Ok(fov_rect) = g.fov_bounding_rect() {
                if fov_rect.max_x - fov_rect.min_x + 1 + padding*2 > max_x {
                    display_center.x = (fov_rect.max_x + fov_rect.min_x) / 2;
                } else {
                    display_center.x = ::std::cmp::max(
                        display_center.x + (max_x - max_x/2),
                        fov_rect.max_x + 1 + padding,
                    ) - (max_x - max_x/2);
                    display_center.x = ::std::cmp::min(
                        display_center.x - max_x/2,
                        fov_rect.min_x - padding,
                    ) + max_x/2;
                }
                if fov_rect.max_y - fov_rect.min_y + 1 + padding*2 > max_y {
                    display_center.y = (fov_rect.max_y + fov_rect.min_y) / 2 + 1;
                } else {
                    display_center.y = ::std::cmp::max(
                        display_center.y + (max_y - max_y/2),
                        fov_rect.max_y + 2 + padding,
                    ) - (max_y - max_y/2);
                    display_center.y = ::std::cmp::min(
                        display_center.y - max_y/2,
                        fov_rect.min_y - padding,
                    ) + max_y/2;
                }
            }
            if old_center != display_center { window.clear(); }

            window.attrset(pancurses::A_BOLD);
            if let Some(status) = g.player_status() {
                if status.recall_turns == Some(0) {
                    window.mvaddstr(0, 0, &format!(
                        " You escaped with {} diamonds! Press 'N' to restart.",
                        status.diamonds,
                    ));
                    mode = InputMode::None;
                } else if status.health > 0 {
                    window.mvaddstr(0, 1, &format!(
                        "Health: {:2}/{:2}",
                        status.health,
                        status.max_health,
                    ));
                    let render_item = |t: EntityType, x, bold| put_cell(
                        &window, 0, x,
                        Cell {
                            ch: t.data().ch,
                            fg: t.data().color.unwrap_or(Color::White),
                            bg: Color::Black,
                            bold: bold,
                        }
                    );

                    if status.recall_turns.is_none() {
                        render_item(EntityType::Scroll, 18, true);
                    }
                    if status.has_sword {
                        render_item(EntityType::Sword, 18+5, true);
                    }
                    if status.has_bow {
                        render_item(EntityType::Bow, 20+5, true);
                    }

                    let render_count = |t: EntityType, x, count| {
                        window.attrset(if count > 0 { pancurses::A_BOLD } else { pancurses::A_NORMAL });
                        window.mvaddstr(0, x, &format!("$: {:2}", count));
                        render_item(t, x, count > 0);
                    };
                    render_count(EntityType::Arrow, 25+5, status.arrows);
                    render_count(EntityType::Rock, 34+5, status.rocks);
                    render_count(EntityType::Corpse, 43+5, status.corpses);
                    render_count(EntityType::Herb, 52+5, status.herbs);
                    render_count(EntityType::Diamond, 61+5, status.diamonds);
                } else {
                    window.mvaddstr(0, 0, &format!(
                        " You died carrying {} diamonds. Press 'N' to restart.",
                        status.diamonds,
                    ));
                    mode = InputMode::None;
                }
            }

            let player_position = g.player_position();
            let (x_offset, y_offset) =
                (display_center.x - max_x/2, display_center.y - max_y/2);
            for y in 0..max_y-1 {
                for x in 0..max_x {
                    let pos = Position { x: x + x_offset, y: y + y_offset };
                    let mut cell = g.render(pos);
                    if mode == InputMode::Fire && Ok(pos) == player_position {
                        cell.bg = cell.fg;
                        cell.fg = Color::Black;
                        cell.bold = false;
                    }
                    if mode == InputMode::Throw(pos)  {
                        cell.bg = Color::Red;
                    }
                    put_cell(&window, y+1, x, cell);
                }
            }

            window.refresh();

            let key = window.getch();

            let dir = key.and_then(|key| match key {
                // arrow keys
                Input::KeyDown => Some(Direction::South),
                Input::KeyUp => Some(Direction::North),
                Input::KeyLeft => Some(Direction::West),
                Input::KeyRight => Some(Direction::East),
                // allow insert / delete / page up / page down for diagonals
                Input::KeyIC => Some(Direction::NorthWest),
                Input::KeyDC => Some(Direction::SouthWest),
                Input::KeyPPage => Some(Direction::NorthEast),
                Input::KeyNPage => Some(Direction::SouthEast),
                // number keys
                Input::Character('1') => Some(Direction::SouthWest),
                Input::Character('2') => Some(Direction::South),
                Input::Character('3') => Some(Direction::SouthEast),
                Input::Character('4') => Some(Direction::West),
                Input::Character('6') => Some(Direction::East),
                Input::Character('7') => Some(Direction::NorthWest),
                Input::Character('8') => Some(Direction::North),
                Input::Character('9') => Some(Direction::NorthEast),
                // vi keys
                Input::Character('h') => Some(Direction::West),
                Input::Character('j') => Some(Direction::South),
                Input::Character('k') => Some(Direction::North),
                Input::Character('l') => Some(Direction::East),
                Input::Character('y') => Some(Direction::NorthWest),
                Input::Character('u') => Some(Direction::NorthEast),
                Input::Character('b') => Some(Direction::SouthWest),
                Input::Character('n') => Some(Direction::SouthEast),
                // handle home and end to allow numpad to work with numlock off
                Input::Character('\x1b') => {
                    window.nodelay(true);
                    let mut keys = vec![];
                    while let Some(key) = window.getch() { keys.push(key) }
                    window.nodelay(false);
                    if keys == [
                        Input::Character('['),
                        Input::Character('1'),
                        Input::Character('~'),
                    ] {
                        Some(Direction::NorthWest)
                    } else if keys == [
                        Input::Character('['),
                        Input::Character('4'),
                        Input::Character('~')
                    ] {
                        Some(Direction::SouthWest)
                    } else {
                        while let Some(key) = keys.pop() {
                            window.ungetch(&key);
                        }
                        None
                    }
                }
                _ => None,
            });

            #[allow(unused_must_use)]  // TODO: handle errors?
            match mode {
                InputMode::None => {}
                InputMode::Throw(pos) => {
                    if let Some(dir) = dir {
                        mode = InputMode::Throw(pos.step(dir));
                        continue 'game;
                    }
                    match key {
                        Some(Input::Character(' '))
                            | Some(Input::Character('5'))
                            | Some(Input::KeyB2) =>
                        {
                            mode = InputMode::Normal;
                            continue 'game;
                        }
                        Some(Input::Character('f')) => {
                            g.player_status().map(|status| {
                                if status.has_bow && status.arrows > 0 {
                                    mode = InputMode::Fire;
                                }
                            });
                            continue 'game;
                        }
                        Some(Input::Character('t')) => {
                            g.take_turn(Action::ThrowRock(pos));
                            mode = InputMode::Normal;
                            continue 'game;
                        }
                        _ => {}
                    }
                }
                InputMode::Fire => {
                    if let Some(dir) = dir {
                        g.take_turn(Action::FireBow(dir));
                        mode = InputMode::Normal;
                        continue 'game;
                    }
                    match key {
                        Some(Input::Character(' '))
                            | Some(Input::Character('5'))
                            | Some(Input::KeyB2) =>
                        {
                            mode = InputMode::Normal;
                            continue 'game;
                        }
                        Some(Input::Character('f')) => {
                            mode = InputMode::Normal;
                            continue 'game;
                        }
                        Some(Input::Character('t')) => {
                            if let Ok(player_position) = g.player_position() {
                                g.player_status().map(|status| {
                                    if status.rocks > 0 {
                                        mode = InputMode::Throw(player_position);
                                    }
                                });
                            }
                            continue 'game;
                        }
                        _ => {}
                    }
                }
                InputMode::Normal => {
                    if let Some(dir) = dir {
                        g.take_turn(Action::Move(dir));
                        continue 'game;
                    }
                    if let Some(action) = match key {
                        Some(Input::Character(' '))
                            | Some(Input::Character('5'))
                            | Some(Input::KeyB2) => Some(Action::Wait),
                        Some(Input::Character('e')) => {
                            g.player_status().and_then(|status| {
                                if status.health < status.max_health {
                                    Some(Action::EatHerb)
                                } else {
                                    None
                                }
                            })
                        },
                        Some(Input::Character('R')) => Some(Action::ReadScroll),
                        Some(Input::Character('g')) => Some(Action::GetCorpse),
                        Some(Input::Character('d')) => Some(Action::DropCorpse),
                        Some(Input::Character('t')) => {
                            if let Ok(player_position) = g.player_position() {
                                g.player_status().map(|status| {
                                    if status.rocks > 0 {
                                        mode = InputMode::Throw(player_position);
                                    }
                                });
                            }
                            continue 'game;
                        },
                        Some(Input::Character('f')) => {
                            g.player_status().map(|status| {
                                if status.has_bow && status.arrows > 0 {
                                    mode = InputMode::Fire;
                                }
                            });
                            continue 'game;
                        },
                        _ => None,
                    } {
                        g.take_turn(action);
                        continue 'game;
                    }
                }
            }

            match key {
                Some(Input::Character(c)) => { match c {
                    'Q' => { break 'application; }
                    '\x1b' => {
                        // handle ESC, but ignore things like Alt+key
                        window.nodelay(true);
                        // TODO: should probably have a menu here
                        if window.getch().is_none() { break 'application; }
                        while window.getch().is_some() {}
                        window.nodelay(false);
                    }
                    'N' => { break 'game; }
                    _ => {}
                }}
                _ => {}
            }
        }
    }

    pancurses::endwin();
}
