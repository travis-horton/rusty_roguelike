use super::Rect;
use rltk::{RandomNumberGenerator, Rltk, RGB};
use std::cmp::{max, min};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

type Map = Vec<TileType>;
type Rooms = Vec<Rect>;

pub fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        render_tile(x, y, tile, ctx);

        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}

fn render_tile(x: i32, y: i32, tile: &TileType, ctx: &mut Rltk) {
    match tile {
        TileType::Floor => ctx.set(
            x,
            y,
            RGB::from_f32(0.5, 0.5, 0.5),
            RGB::from_f32(0., 0., 0.),
            rltk::to_cp437('.'),
        ),
        TileType::Wall => ctx.set(
            x,
            y,
            RGB::from_f32(0.0, 1.0, 0.0),
            RGB::from_f32(0., 0., 0.),
            rltk::to_cp437('#'),
        ),
    }
}

pub fn new_map_rooms_and_corridors() -> (Rooms, Map) {
    let mut map = vec![TileType::Wall; crate::WIDTH as usize * crate::HEIGHT as usize];

    let rooms = add_rooms_and_corridors(&mut map);

    (rooms, map)
}

fn add_rooms_and_corridors(map: &mut Map) -> Rooms {
    let mut rng = RandomNumberGenerator::new();
    let mut rooms: Rooms = Vec::new();
    const MAX_ROOMS: i32 = 30;
    const MIN_SIZE: i32 = 6;
    const MAX_SIZE: i32 = 10;

    for _ in 0..MAX_ROOMS {
        let w = rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        let x = rng.roll_dice(1, crate::WIDTH - w - 1) - 1;
        let y = rng.roll_dice(1, crate::HEIGHT - h - 1) - 1;
        let new_room = Rect::new(x, y, w, h);
        let mut ok = true;

        for other_room in rooms.iter() {
            if new_room.intersect(other_room) {
                ok = false;
            }
        }

        if ok {
            apply_room_to_map(&new_room, map);
            if !rooms.is_empty() {
                connect_room_to_previous_room(&new_room, &rooms[rooms.len() - 1], map);
            }
            rooms.push(new_room);
        }
    }

    rooms
}

fn connect_room_to_previous_room(new_room: &Rect, prev_room: &Rect, map: &mut Map) {
    let mut rng = RandomNumberGenerator::new();
    let (new_x, new_y) = new_room.center();
    let (prev_x, prev_y) = prev_room.center();
    if rng.range(0, 2) == 1 {
        apply_horizontal_tunnel(map, prev_x, new_x, prev_y);
        apply_vertical_tunnel(map, prev_y, new_y, new_x);
    } else {
        apply_horizontal_tunnel(map, prev_x, new_x, new_y);
        apply_vertical_tunnel(map, prev_y, new_y, prev_x);
    }
}

fn apply_room_to_map(room: &Rect, map: &mut Map) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            map[xy_idx(x, y)] = TileType::Floor;
        }
    }
}

fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < crate::WIDTH as usize * crate::HEIGHT as usize {
            map[idx as usize] = TileType::Floor;
        }
    }
}

fn apply_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < crate::WIDTH as usize * crate::HEIGHT as usize {
            map[idx as usize] = TileType::Floor;
        }
    }
}

/// Makes a map with solid boundaries and 400 randomly placed walls. No guarantees that it won't
/// look awful.
pub fn new_map_test() -> Map {
    let mut map = vec![TileType::Floor; crate::WIDTH as usize * crate::HEIGHT as usize];
    add_boundary_walls(&mut map);
    add_random_walls(&mut map);

    map
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * crate::WIDTH as usize) + x as usize
}

fn get_random_location() -> usize {
    let mut rng = RandomNumberGenerator::new();

    let x = rng.roll_dice(1, crate::WIDTH - 1);
    let y = rng.roll_dice(1, crate::HEIGHT - 1);
    xy_idx(x, y)
}

fn add_boundary_walls(map: &mut Map) -> &mut Map {
    for x in 0..crate::WIDTH {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, crate::HEIGHT - 1)] = TileType::Wall;
    }
    for y in 0..crate::HEIGHT {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(crate::WIDTH - 1, y)] = TileType::Wall;
    }

    map
}

fn add_random_walls(map: &mut Map) -> &mut Map {
    for _i in 0..400 {
        let location = get_random_location();
        if location != xy_idx(crate::WIDTH / 2, crate::HEIGHT / 2) {
            map[location] = TileType::Wall;
        }
    }

    map
}
