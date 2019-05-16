use crate::{ SCREEN_WIDTH, SCREEN_HEIGHT };
use crate::game::{ TCOD, Game };

use tcod::colors::{ self, Color };
use tcod::console::*;
use tcod::input::Mouse;
use tcod::map::Map as FovMap;

pub const PANEL_HEIGHT: i32 = 7;
const BAR_WIDTH: i32 = 20;
const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;
const MSG_X: i32 = BAR_WIDTH + 2;
const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;
const INVENTORY_WIDTH: i32 = 50;

pub type Messages = Vec< (String, Color) >;

pub trait MessageLog
{
    fn add< T: Into< String > >(&mut self, message: T, color: Color);
}

impl MessageLog for Vec< (String, Color) >
{
    fn add< T: Into< String > >(&mut self, message: T, color: Color)
    {
        self.push((message.into(), color));
    }
}

pub fn render_gui(tcod: &mut TCOD, game: &mut Game)
{
    // Prepare gui panel for rendering
    tcod.panel.set_default_background(colors::BLACK);
    tcod.panel.clear();

    // Render message log
    let mut y = MSG_HEIGHT as i32;
    for &(ref msg, color) in game.log.iter().rev()
    {
        let msg_h = tcod.panel.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
        y -= msg_h;
        if y < 0
        {
            break;
        }

        tcod.panel.set_default_foreground(color);
        tcod.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
    }

    // Render player stats and info section
    // TODO: this

    // Display name of entities under mouse
    tcod.panel.set_default_foreground(colors::LIGHTER_GREY);
    tcod.panel.print_ex(1, 0, BackgroundFlag::None, TextAlignment::Left, get_names_under_mouse(tcod.mouse, game, &tcod.fov));

    // blit the gui panel to root console
    tcod::console::blit(&tcod.panel, (0, 0), (SCREEN_WIDTH, PANEL_HEIGHT), &mut tcod.root, (0, PANEL_Y), 1.0, 1.0);
}

fn render_bar(panel: &mut Offscreen, x: i32, y: i32, total_width: i32, name: &str, value: i32, max_value: i32, bar_color: Color, background_color: Color)
{
}

fn get_names_under_mouse(mouse: Mouse, game: &Game, fov: &FovMap) -> String
{
    let (x, y) = (mouse.cx as i32, mouse.cy as i32);
    let names = game.entities
        .iter()
        .filter(|e| e.pos == (x, y) && fov.is_in_fov(e.pos.0, e.pos.1))
        .map(|e| e.name.clone())
        .collect::< Vec< _ > >();
    
    names.join(", ")
}