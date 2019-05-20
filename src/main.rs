extern crate tcod;
extern crate rand;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod game;
mod map;
mod object;
mod fighter;
mod ai;
mod item;
mod menu;
mod gui;

use crate::map::{ MAP_WIDTH, MAP_HEIGHT };
use crate::gui::PANEL_HEIGHT;

use tcod::console::*;
use tcod::input::Mouse;

pub const SCREEN_WIDTH: i32 = 80;
pub const SCREEN_HEIGHT: i32 = 50;
const FPS_LIMIT: i32 = 20;

pub struct TCOD
{
    pub root: Root,
    pub con: Offscreen,
    pub gui: Offscreen,
    pub mouse: Mouse
}

fn main() 
{
    // Initialize the root console
    let root = Root::initializer()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("roguelike-rs")
        .init();
    tcod::system::set_fps(FPS_LIMIT);

    // Initialize core tcod stuff
    let mut tcod = TCOD
    {
        root: root,
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
        gui: Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT),
        mouse: Default::default()
    };

    // Show main menu
    menu::main_menu(&mut tcod);
}
