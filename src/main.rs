extern crate tcod;
extern crate rand;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod game;
mod map;
mod entity;
mod menus;
mod gui;

use crate::game::TCOD;
use crate::map::{ MAP_WIDTH, MAP_HEIGHT };
use crate::gui::PANEL_HEIGHT;

use tcod::console::{ Root, FontType, Offscreen };
use tcod::map::Map as FovMap;

pub const SCREEN_WIDTH: i32 = 80;
pub const SCREEN_HEIGHT: i32 = 50;

fn main() 
{
    // Create the root console
    let root = Root::initializer()
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("roguelike-rs")
        .init();
    tcod::system::set_fps(60);
    
    // Create the TCOD struct
    let mut tcod = TCOD {
        root: root,
        con: Offscreen::new(MAP_WIDTH, MAP_HEIGHT),
        panel: Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT),  // todo: change these values
        fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT),
        mouse: Default::default()
    };

    // show main menu
    menus::main_menu(&mut tcod);
}
