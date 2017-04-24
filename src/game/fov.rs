use ::engine::*;
use super::{Game, Location, Position, Tile};

pub struct IsVisible(pub i8);
impl Component for IsVisible {}

pub struct WasVisible(pub Tile);
impl Component for WasVisible {}

pub fn update_fov(game: &mut Game) {
    game.world.component_mut::<IsVisible>().clear();

    if let Some(player) = game.find_player() {
        if let Ok(&Location::Position(pos)) = game.world.entity(player).get() {
            let view_distance = game.player_fov_range();

            insert(game, pos, 0, view_distance);
            for quadrant in 0..4 {
                let pt = |x, y| match quadrant {
                    0 => add_offset(pos, x, y),
                    1 => add_offset(pos, -y, x),
                    2 => add_offset(pos, -x, -y),
                    3 => add_offset(pos, y, -x),
                    _ => unreachable!(),
                };

                for dist in 1..6 {
                    insert(game, pt(dist, 0), dist as i8, view_distance);
                    if is_obstructed(game, pt(dist, 0)) { break; }
                }
                for &(dist, offset) in &[(1, 1), (3, 2), (4, 3)] {
                    insert(game, pt(offset, offset), dist, view_distance);
                    if is_obstructed(game, pt(offset, offset)) { break; }
                }

                for &reflected in &[true, false] {
                    let pt = |x, y| if reflected { pt(x, y) } else { pt(y, x) };
                    if not_obstructed(game, pt(1, 0)) || not_obstructed(game, pt(1, 1)) {
                        insert(game, pt(2, 1), 2, view_distance);
                    }

                    if none_obstructed(game, &[pt(1, 0), pt(2, 1)]) {
                        insert(game, pt(3, 1), 3, view_distance);
                        if not_obstructed(game, pt(3, 1))
                            && (not_obstructed(game, pt(1, 1)) || not_obstructed(game, pt(2, 0)))
                        {
                            insert(game, pt(4, 1), 4, view_distance);
                        }
                    } else if none_obstructed(game, &[pt(1, 0), pt(2, 0), pt(3, 0), pt(3, 1)]) {
                        insert(game, pt(4, 1), 4, view_distance);
                    }

                    if none_obstructed(game, &[pt(1, 0), pt(2, 0), pt(3, 1), pt(4, 1)])
                        && (not_obstructed(game, pt(2, 1)) || not_obstructed(game, pt(3, 0)))
                    {
                        insert(game, pt(5, 1), 5, view_distance);
                    }

                    if none_obstructed(game, &[pt(1, 1), pt(2, 1)]) {
                        insert(game, pt(3, 2), 4, view_distance);
                        if none_obstructed(game, &[pt(2, 2), pt(3, 2)]) {
                            insert(game, pt(4, 3), 5, view_distance);
                        }
                    }

                    if not_obstructed(game, pt(2, 1))
                        && (none_obstructed(game, &[pt(1, 0), pt(3, 1)])
                            || none_obstructed(game, &[pt(1, 1), pt(3, 2)]))
                    {
                        insert(game, pt(4, 2), 5, view_distance);
                    }
                }
            }
        }
    }
}

fn is_obstructed(game: &Game, pos: Position) -> bool {
    game.get_tile(pos).is_obstructed()
}

fn not_obstructed(game: &Game, pos: Position) -> bool {
    !is_obstructed(game, pos)
}

fn none_obstructed(game: &Game, positions: &[Position]) -> bool {
    positions.iter().all(|&pos| not_obstructed(game, pos))
}

fn add_offset(pos: Position, x: i32, y: i32) -> Position {
    Position { x: pos.x + x, y: pos.y + y }
}

fn insert(game: &mut Game, pos: Position, distance: i8, view_distance: i8) {
    if view_distance >= distance {
        let tile = game.get_tile(pos);
        game.world.entity_mut(pos).insert(WasVisible(tile));
    }
    game.world.entity_mut(pos).insert(IsVisible(distance));
}
