extern crate tcod;
extern crate rand;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod map;
mod object;

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

    let mut map = map::Map::new();
    let pos = map.generate();

    root.clear();
    map.recompute_fov(pos);
    map.draw(&mut root);
    root.flush();
    root.wait_for_keypress(true);
}
