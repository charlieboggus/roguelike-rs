extern crate tcod;
extern crate rand;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod game;
mod map;
mod object;
mod menu;

use crate::map::Map;
use crate::object::Object;
use tcod::console::*;

pub const SCREEN_WIDTH: i32 = 80;
pub const SCREEN_HEIGHT: i32 = 50;
const FPS_LIMIT: i32 = 60;

fn main() 
{
    // Initialize the root console
    let mut root = Root::initializer()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("roguelike-rs")
        .init();
    tcod::system::set_fps(FPS_LIMIT);

    let mut objects: Vec< Object > = vec![];

    let mut map = Map::new();
    let pos = map.generate(&mut objects, 1);

    root.clear();
    map.recompute_fov(pos);
    map.draw(&mut root);
    root.flush();
    root.wait_for_keypress(true);
}
