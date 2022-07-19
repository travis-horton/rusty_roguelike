use rltk::{GameState, Rltk, RGB, VirtualKeyCode};
use specs::prelude::*; use std::cmp::{max, min};
use specs_derive::Component;

const WIDTH: i32 = 80;
const HEIGHT: i32 = 50;

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(Component, Debug)]
struct Player {}

#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall, Floor
}

type Map = Vec<TileType>;

struct State {
    ecs: World
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y as usize * WIDTH as usize) + x as usize
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map[destination_idx] != TileType::Wall {
            pos.x = min(WIDTH - 1, max(0, pos.x + delta_x));
            pos.y = min(HEIGHT - 1, max(0, pos.y + delta_y));
        }
    }
}

fn get_random_location() -> usize {
    let mut rng = rltk::RandomNumberGenerator::new();

    let x = rng.roll_dice(1, WIDTH - 1);
    let y = rng.roll_dice(1, HEIGHT - 1);
    xy_idx(x, y)
}

fn add_boundary_walls(
    map: &mut Map,
) -> &mut Map {
    for x in 0..WIDTH {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, HEIGHT - 1)] = TileType::Wall;
    };
    for y in 0..HEIGHT {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(WIDTH - 1, y)] = TileType::Wall;
    };

    map
}

fn add_random_walls(map: &mut Map) -> &mut Map {
    for _i in 0..400 {
        let location = get_random_location();
        if location != xy_idx(WIDTH / 2, HEIGHT / 2) {
            map[location] = TileType::Wall;
        }
    }

    map
}

fn new_map() -> Map {
    let mut map = vec![
        TileType::Floor;
        WIDTH as usize * HEIGHT as usize
    ];

    add_boundary_walls(&mut map);
    add_random_walls(&mut map);

    map
}

fn render_tile(x: i32, y: i32, tile: &TileType, ctx: &mut Rltk) {
    match tile {
        TileType::Floor => {
            ctx.set(
                x,
                y,
                RGB::from_f32(0.5, 0.5, 0.5),
                RGB::from_f32(0., 0., 0.),
                rltk::to_cp437('.'),
            )
        }
        TileType::Wall => {
            ctx.set(
                x,
                y,
                RGB::from_f32(0.0, 1.0, 0.0),
                RGB::from_f32(0., 0., 0.),
                rltk::to_cp437('#'),
            )
        }
    }
}

fn draw_map(map: &[TileType], ctx: &mut Rltk) {
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

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    // Player movement
    match ctx.key {
        None => {} // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::W => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::A => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::S => try_move_player(0, 1, &mut gs.ecs),
            VirtualKeyCode::D => try_move_player(1, 0, &mut gs.ecs),
            _ => {}
        },
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        player_input(self, ctx);
        self.run_systems();

        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(&map, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

impl State {
    fn run_systems(&mut self) {
        self.ecs.maintain();
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    let mut gs = State {
        ecs: World::new()
    };
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.insert(new_map());

    gs.ecs
        .create_entity()
        .with(Position { x: WIDTH / 2, y: HEIGHT / 2 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .build();

    rltk::main_loop(context, gs)
}
