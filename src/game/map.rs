use std::collections::{BinaryHeap, HashSet, HashMap};
use rand::Rng;

use super::*;

trait Feature {
    fn overwrite(&mut self, g: &mut Game, pos: Position, dist: i32) -> Option<i32>;
}

trait TerrainFeature {
    fn pick_tile<R: Rng>(&mut self, distance: i32, rand: &mut R) -> (Tile, i32);

    fn size(&self) -> i32;

    fn can_overwrite(&self, t: Tile) -> bool {
        t == Tile::BoringGround
    }
}

impl<F: TerrainFeature> Feature for F {
    fn overwrite(&mut self, g: &mut Game, pos: Position, dist: i32) -> Option<i32> {
        if dist > self.size() || !self.can_overwrite(g.get_tile(pos)) {
            return None;
        }

        let (tile, dist_increase) = self.pick_tile(dist, &mut g.rand);
        g.world.entity_mut(pos).insert(tile);
        Some(dist_increase)
    }
}

struct Meadow(i32, i32);

impl TerrainFeature for Meadow {
    fn pick_tile<R: Rng>(&mut self, distance: i32, rand: &mut R) -> (Tile, i32) {
        if rand.gen_range(0, self.size()) + self.size()/2 < distance {
            (Tile::Ground, rand.gen_range(3, 5))
        } else if rand.gen_range(0, self.1) != 0
            || rand.gen_range(0, self.size()) < distance
        {
            (Tile::ShortGrass, rand.gen_range(1, 3))
        } else {
            (Tile::LongGrass, rand.gen_range(1, 3))
        }
    }

    fn size(&self) -> i32 { self.0 }

    fn can_overwrite(&self, t: Tile) -> bool {
        t == Tile::BoringGround || t == Tile::Ground
    }
}

struct Lake(i32, i32);

impl TerrainFeature for Lake {
    fn pick_tile<R: Rng>(&mut self, distance: i32, rand: &mut R) -> (Tile, i32) {
        if self.size()/self.1 < self.size() - distance {
            (Tile::DeepWater, rand.gen_range(1, 3))
        } else {
            (Tile::ShallowWater, rand.gen_range(1, 8))
        }
    }

    fn size(&self) -> i32 { self.0 }
}

struct Tree;

impl TerrainFeature for Tree {
    fn pick_tile<R: Rng>(&mut self, distance: i32, rand: &mut R) -> (Tile, i32) {
        if distance == 0 {
            (Tile::Tree, 1)
        } else {
            (Tile::Ground, rand.gen_range(1, 3))
        }
    }

    fn size(&self) -> i32 { 2 }
}

struct Trees(i32, i32, i32);

impl Feature for Trees {
    fn overwrite(&mut self, g: &mut Game, pos: Position, dist: i32) -> Option<i32> {
        if dist > self.0 || g.get_tile(pos) != Tile::BoringGround {
            return None;
        }

        if dist > self.1 && g.rand.gen_range(0, self.2) == 0 {
            add_feature_at(g, Tree, Some(pos));
        } else {
            g.world.entity_mut(pos).insert(Tile::Ground);
        }
        Some(g.rand.gen_range(0, self.1))
    }
}

// TODO: This is painfully inefficient. Can I do better?
fn rand_position(g: &mut Game) -> Position {
    let count = g.world.component::<Tile>().count();
    let index = g.rand.gen_range(0, count);
    g.world.component::<Tile>().ids().nth(index).unwrap_or(Position { x: 0, y: 0 })
}

fn select_position<P: FnMut(Tile) -> bool>(g: &mut Game, mut predicate: P) -> Option<Position> {
    for _ in 0..10 {
        let pos = rand_position(g);
        if predicate(g.get_tile(pos)) {
            return Some(pos);
        }
    }
    None
}

fn add_feature_at<F: Feature>(g: &mut Game, mut f: F, pos: Option<Position>) {
    let mut queue = BinaryHeap::new();
    let mut visited = HashSet::new();

    if let Some(pos) = pos {
        if let Some(dist_increase) = f.overwrite(g, pos, 0) {
            queue.push((-dist_increase, pos));
            visited.insert(pos);
        }
    } else {
        for _ in 0..10 {
            let pos = rand_position(g);
            if let Some(dist_increase) = f.overwrite(g, pos, 0) {
                queue.push((-dist_increase, pos));
                visited.insert(pos);
                break;
            }
        }
    }

    while let Some((priority, pos)) = queue.pop() {
        for &dir in &ALL_DIRECTIONS {
            let next_pos = pos.step(dir);
            if !visited.contains(&next_pos)
                && (dir.is_orthogonal() || g.rand.gen_range(0, 5) < 2)
            {
                if let Some(dist_increase) = f.overwrite(g, next_pos, -priority) {
                    queue.push((priority-dist_increase, next_pos));
                }
                visited.insert(next_pos);
            }
        }
    }
}

fn add_feature<F: Feature>(g: &mut Game, f: F) {
    add_feature_at(g, f, None);
}

fn find_map_edge(g: &mut Game) -> Option<Position> {
    let mut count = 0;
    let mut map_edge = None;
    for pos in g.world.component::<Tile>().ids() {
        for &dir in &ORTHOGONAL_DIRECTIONS {
            if g.get_tile(pos.step(dir)) == Tile::Wall {
                count += 1;
                if g.rand.gen_range(0, count) == 0 {
                    map_edge = Some(pos);
                }
            }
        }
    }
    map_edge
}

fn create_ground(g: &mut Game) {
    let mut queue = BinaryHeap::new();
    let mut visited = HashSet::new();
    let initial_pos = Position { x: 0, y: 0 };
    g.world.entity_mut(initial_pos).insert(Tile::BoringGround);
    queue.push((0, initial_pos));
    visited.insert(initial_pos);

    while visited.len() < 4000 {
        let (priority, pos) = queue.pop().unwrap();
        for &dir in &ALL_DIRECTIONS {
            let next_pos = pos.step(dir);
            if !visited.contains(&next_pos) && (
                dir.is_orthogonal() || g.rand.gen_range(0, 5) < 2
            ) {
                g.world.entity_mut(next_pos).insert(Tile::BoringGround);
                let next_priority = priority - g.rand.gen_range(0, 3);
                queue.push((next_priority, next_pos));
                visited.insert(next_pos);
            }
        }
    }
}

struct RiverSegment;

impl TerrainFeature for RiverSegment {
    fn pick_tile<R: Rng>(&mut self, _: i32, rand: &mut R) -> (Tile, i32) {
        (Tile::ShallowWater, rand.gen_range(2, 3))
    }

    fn size(&self) -> i32 { 2 }

    fn can_overwrite(&self, t: Tile) -> bool {
        t == Tile::BoringGround || t == Tile::ShallowWater || t == Tile::Wall
    }
}

struct River {
    end: Position,
    order: HashMap<Position, i32>,
    max_distance: i32,
    done: bool,
}

impl Feature for River {
    fn overwrite(&mut self, g: &mut Game, pos: Position, dist: i32) -> Option<i32> {
        if self.done { return None; }
        let tile = g.get_tile(pos);
        if tile == Tile::Wall { return None; }
        self.max_distance = ::std::cmp::max(self.max_distance, dist);
        self.max_distance += 1;

        let count = self.order.len() as i32;
        if pos == self.end {
            self.order.insert(pos, count);
            let mut cur_pos = self.end;
            while self.order.get(&cur_pos).map(|&c| c > 0).unwrap_or(false) {
                let new_pos = ALL_DIRECTIONS.iter().map(|&d| cur_pos.step(d))
                    .min_by_key(|pos| *self.order.get(pos).unwrap_or(&(count + 1)))
                    .unwrap();
                add_feature_at(g, RiverSegment, Some(new_pos));
                cur_pos = new_pos;
            }
            self.done = true;
            None
        } else if tile == Tile::ShallowWater || tile == Tile::DeepWater {
            self.order.insert(pos, count);
            Some(- 1)
        } else if tile == Tile::BoringGround {
            self.order.insert(pos, count);
            Some(g.rand.gen_range(1, 3))
        } else {
            None
        }
    }
}

fn add_river(g: &mut Game, start: Position, end: Position) {
    add_feature_at(
        g,
        River {
            end: end,
            order: HashMap::new(),
            max_distance: 0,
            done: false,
        },
        Some(start),
    );
}

fn place_player(g: &mut Game) {
    let pt = rand_position(g);
    let player_pos = g.world.component::<Tile>().ids().filter(|&pos| {
        if ![Tile::Ground, Tile::BoringGround, Tile::ShortGrass, Tile::LongGrass]
            .contains(&g.get_tile(pos))
        {
            return false;
        }
        for &dir in &ALL_DIRECTIONS {
            if g.get_tile(pos.step(dir)) == Tile::Wall {
                return false;
            }
        }
        true
    }).max_by_key(|pos| {
        (pt.x - pos.x)*(pt.x - pos.x) + (pt.y - pos.y)*(pt.y - pos.y)
    }).unwrap();
    g.put_entity(EntityType::Player, player_pos);
}

pub fn init_game(g: &mut Game) {
    create_ground(g);

    let lake_pos = rand_position(g);
    add_feature_at(g, Lake(16, 3), Some(lake_pos));

    add_feature(g, Lake(12, 1));
    add_feature(g, Lake(8, 1));
    add_feature(g, Lake(8, 2));
    add_feature(g, Lake(8, 3));

    if let Some(map_edge) = find_map_edge(g) {
        add_river(g, map_edge, lake_pos);
    }

    add_feature(g, Trees(64, 7, 3));
    add_feature(g, Trees(32, 5, 5));
    add_feature(g, Trees(16, 3, 7));
    for _ in 0..16 {
        add_feature(g, Tree);
    }

    add_feature(g, Meadow(16, 1));
    add_feature(g, Meadow(16, 1));
    add_feature(g, Meadow(12, 1));
    add_feature(g, Meadow(8, 1));
    add_feature(g, Meadow(6, 1));
    add_feature(g, Meadow(4, 1));

    add_feature(g, Meadow(24, 7));
    add_feature(g, Meadow(12, 5));
    add_feature(g, Meadow(6, 3));

    place_player(g);

    let is_land = |tile| [
        Tile::Ground, Tile::BoringGround, Tile::ShortGrass, Tile::LongGrass,
    ].contains(&tile);

    {
        let mut randomly_place = |entity_type, count| {
            for _ in 0..count {
                if let Some(pos) = select_position(g, &is_land) {
                    g.put_entity(entity_type, pos);
                }
            }
        };

        randomly_place(EntityType::Rock, 32);
        randomly_place(EntityType::Herb, 8);
        randomly_place(EntityType::Rat, 8);
        randomly_place(EntityType::Deer, 8);
        randomly_place(EntityType::Wolf, 4);
    }

    let mut diamond_count = 0;

    for _ in 0..2 { // 0 + 2 * 3 = 6
        if let Some(pos) = select_position(g, &is_land) {
            g.put_entity(EntityType::Skeleton, pos);
            for _ in 0..g.rand.gen_range(0, 1) {
                g.put_entity(EntityType::Herb, pos);
            }
            for _ in 0..g.rand.gen_range(1, 3) {
                g.put_entity(EntityType::Diamond, pos);
                diamond_count += 1;
            }
            for _ in 0..g.rand.gen_range(3, 5) {
                g.put_entity(EntityType::Arrow, pos);
            }
            g.put_entity(EntityType::Bow, pos);
        }
    }

    for _ in 0..2 { // 6 + 2 * 3 = 12
        if let Some(pos) = select_position(g, &is_land) {
            g.put_entity(EntityType::Skeleton, pos);
            for _ in 0..g.rand.gen_range(0, 2) {
                g.put_entity(EntityType::Herb, pos);
            }
            for _ in 0..g.rand.gen_range(1, 3) {
                g.put_entity(EntityType::Diamond, pos);
                diamond_count += 1;
            }
            g.put_entity(EntityType::Sword, pos);
        }
    }

    for _ in 0..2 { // 12 + 2 * 5 = 22
        if let Some(pos) = select_position(g, &is_land) {
            g.put_entity(EntityType::Skeleton, pos);
            for _ in 0..g.rand.gen_range(0, 1) {
                g.put_entity(EntityType::Herb, pos);
            }
            for _ in 0..g.rand.gen_range(2, 5) {
                g.put_entity(EntityType::Diamond, pos);
                diamond_count += 1;
            }
            for _ in 0..g.rand.gen_range(0, 3) {
                g.put_entity(EntityType::Arrow, pos);
            }
            for _ in 0..g.rand.gen_range(0, 1) {
                g.put_entity(EntityType::Sword, pos);
            }
            for _ in 0..g.rand.gen_range(0, 1) {
                g.put_entity(EntityType::Bow, pos);
            }
            g.put_entity(EntityType::Dragon, pos);
        }
    }

    for _ in 0..3 { // 22 + 3 * 2 = 28
        if let Some(pos) = select_position(g, &is_land) {
            g.put_entity(EntityType::Skeleton, pos);
            for _ in 0..g.rand.gen_range(0, 2) {
                g.put_entity(EntityType::Diamond, pos);
                diamond_count += 1;
            }
        }
    }

    while diamond_count < 30 {
        if let Some(pos) = select_position(g, &is_land) {
            g.put_entity(EntityType::Diamond, pos);
            diamond_count += 1;
        }
    }
}
