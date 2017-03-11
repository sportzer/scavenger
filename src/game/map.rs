use std::collections::{BinaryHeap, HashSet};
use rand::Rng;

use ::engine::*;
use super::*;

fn create_ground(g: &mut Game) {
    let mut queue = BinaryHeap::new();
    let mut visited = HashSet::new();
    let mut player_pos = Position { x: 0, y: 0 };
    g.world.insert(player_pos, Tile::Ground);
    queue.push((0, player_pos));
    visited.insert(player_pos);

    while visited.len() < 4000 {
        let (priority, pos) = queue.pop().unwrap();
        player_pos = pos;
        for &dir in &ALL_DIRECTIONS {
            let next_pos = pos.step(dir);
            if !visited.contains(&next_pos) && (
                dir.is_orthogonal() || g.rand.gen_range(0, 3) < 2
            ) {
                g.world.insert(next_pos, Tile::Ground);
                let next_priority = priority - g.rand.gen_range(0, 3);
                queue.push((next_priority, next_pos));
                visited.insert(next_pos);
            }
        }
    }

    g.put_entity(EntityType::Player, player_pos);
}

// TODO: this is temporary
pub fn init_game(g: &mut Game) {
    create_ground(g);
    g.put_entity(EntityType::Rock, Position { x: 3, y: 3 });

    g.put_entity(EntityType::Rat, Position { x: 5, y: 3 });

    g.put_entity(EntityType::Rock, Position { x: 4, y: 4 });
    g.put_entity(EntityType::Rat, Position { x: 4, y: 4 });

    g.world.insert(Position{ x: 6, y: 5 }, Tile::ShallowWater);
    g.world.insert(Position{ x: 7, y: 5 }, Tile::ShallowWater);
    g.world.insert(Position{ x: 6, y: 6 }, Tile::ShallowWater);
    g.world.insert(Position{ x: 7, y: 6 }, Tile::DeepWater);
    g.world.insert(Position{ x: 6, y: 7 }, Tile::DeepWater);
    g.world.insert(Position{ x: 7, y: 7 }, Tile::DeepWater);

    g.put_entity(EntityType::Rock, Position { x: 7, y: 5 });
    g.put_entity(EntityType::Rock, Position { x: 7, y: 6 });

    g.world.insert(Position{ x: 1, y: 4 }, Tile::ShortGrass);
    g.world.insert(Position{ x: 2, y: 4 }, Tile::ShortGrass);
    g.world.insert(Position{ x: 3, y: 4 }, Tile::ShortGrass);
    g.world.insert(Position{ x: 1, y: 5 }, Tile::LongGrass);
    g.world.insert(Position{ x: 2, y: 5 }, Tile::LongGrass);
    g.world.insert(Position{ x: 3, y: 5 }, Tile::LongGrass);
    g.world.insert(Position{ x: 1, y: 6 }, Tile::LongGrass);
    g.world.insert(Position{ x: 2, y: 6 }, Tile::LongGrass);
    g.world.insert(Position{ x: 3, y: 6 }, Tile::LongGrass);

    g.world.insert(Position{ x: g.rand.gen_range(1,8), y: 1 }, Tile::Tree);
}

