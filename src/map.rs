use super::Rect;
use rltk::{Algorithm2D, BaseMap, Point, RandomNumberGenerator, Rltk, RGB};
use specs::prelude::*;
use std::cmp::{max, min};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

type Rooms = Vec<Rect>;

#[derive(Default)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Rooms,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
}

impl Map {
    pub fn new_map_rooms_and_corridors() -> Map {
        const MAP_LENGTH: usize = crate::WIDTH as usize * crate::HEIGHT as usize;
        let mut map = Map {
            tiles: vec![TileType::Wall; MAP_LENGTH],
            rooms: Vec::new(),
            revealed_tiles: vec![false; MAP_LENGTH],
            visible_tiles: vec![false; MAP_LENGTH],
        };

        map.add_rooms_and_corridors();
        map
    }

    fn add_rooms_and_corridors(&mut self) {
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
                self.apply_room_to_map(&new_room);
                if !rooms.is_empty() {
                    self.connect_room_to_previous_room(&new_room, &rooms[rooms.len() - 1]);
                }
                rooms.push(new_room);
            }
        }

        self.rooms = rooms;
    }

    fn connect_room_to_previous_room(&mut self, new_room: &Rect, prev_room: &Rect) {
        let mut rng = RandomNumberGenerator::new();
        let (new_x, new_y) = new_room.center();
        let (prev_x, prev_y) = prev_room.center();
        if rng.range(0, 2) == 1 {
            self.apply_horizontal_tunnel(prev_x, new_x, prev_y);
            self.apply_vertical_tunnel(prev_y, new_y, new_x);
        } else {
            self.apply_horizontal_tunnel(prev_x, new_x, new_y);
            self.apply_vertical_tunnel(prev_y, new_y, prev_x);
        }
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < crate::WIDTH as usize * crate::HEIGHT as usize {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < crate::WIDTH as usize * crate::HEIGHT as usize {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * crate::WIDTH as usize) + x as usize
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(crate::WIDTH, crate::HEIGHT)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut y = 0;
    let mut x = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.revealed_tiles[idx] {
            let (glyph, mut fg) = get_tile_render(tile);
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale()
            }
            ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
        }

        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}

fn get_tile_render(tile: &TileType) -> (u16, RGB) {
    let glyph;
    let fg;

    match tile {
        TileType::Floor => {
            glyph = rltk::to_cp437('.');
            fg = RGB::from_f32(0.5, 0.5, 0.5);
        }
        TileType::Wall => {
            glyph = rltk::to_cp437('#');
            fg = RGB::from_f32(0.0, 1.0, 0.0);
        }
    }

    (glyph, fg)
}
