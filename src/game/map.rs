use rand::Rng;

use ::engine::*;
use super::*;

// TODO: this is temporary
pub fn init_game(g: &mut Game) {
    g.put_entity(EntityType::Player, Position { x: 3, y: 3 });
    g.put_entity(EntityType::Rock, Position { x: 3, y: 3 });

    g.put_entity(EntityType::Rat, Position { x: 5, y: 3 });

    g.put_entity(EntityType::Rock, Position { x: 4, y: 4 });
    g.put_entity(EntityType::Rat, Position { x: 4, y: 4 });

    for x in 0..14 {
        for y in 0..9 {
            let tile = if x%8 == 0 || y%8 == 0 { Tile::Wall } else { Tile::Ground };
            g.world.insert(Position{ x, y }, tile);
        }
    }
    g.world.insert(Position{ x: 8, y: 4 }, Tile::Ground);

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

