use crate::{ TCOD, SCREEN_WIDTH, SCREEN_HEIGHT };
use crate::game::{ Game, PLAYER_ID, LEVEL_UP_BASE, LEVEL_UP_FACTOR };

use tcod::colors::{ self, Color };
use tcod::console::*;
use tcod::input::Mouse;

pub const PANEL_HEIGHT: i32 = 10;
const BAR_WIDTH: i32 = 16;
const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;
const MSG_X: i32 = BAR_WIDTH + 6;
const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;

pub type Messages = Vec< (String, Color) >;

pub trait MessageLog
{
    fn add< T: Into< String > >(&mut self, msg: T, color: Color);
}

impl MessageLog for Vec< (String, Color) >
{
    fn add< T: Into< String > >(&mut self, msg: T, color: Color)
    {
        self.push((msg.into(), color));
    }
}

/// Renders the gui to the root tcod console
pub fn render_gui(tcod: &mut TCOD, game: &mut Game)
{
    // Render message log
    tcod.gui.set_default_background(colors::DARKER_GREY);
    tcod.gui.clear();
    let mut y = MSG_HEIGHT as i32;
    for &(ref msg, color) in game.log.iter().rev()
    {
        let msg_h = tcod.gui.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
        y -= msg_h;
        if y < 0
        {
            break;
        }

        tcod.gui.set_default_foreground(color);
        tcod.gui.print_rect(MSG_X, y + 1, MSG_WIDTH, 0, format!("> {}", msg));
    }

    // Get player stats
    let hp = game.objects[PLAYER_ID].fighter.map_or(0, |f| f.hp);
    let max_hp = game.objects[PLAYER_ID].fighter.map_or(0, |f| f.max_hp);
    let xp = game.objects[PLAYER_ID].fighter.map_or(0, |f| f.xp);
    let xp_target = LEVEL_UP_BASE + game.objects[PLAYER_ID].level * LEVEL_UP_FACTOR;

    // Reset foreground color to white
    tcod.gui.set_default_foreground(colors::WHITE);

    // Render player stats
    tcod.gui.print_ex(1, 3, BackgroundFlag::None, TextAlignment::Left, "HP:");
    render_progress_bar(&mut tcod.gui, 4, 3, BAR_WIDTH, hp, max_hp, colors::LIGHT_RED, colors::BLACK);

    tcod.gui.print_ex(1, 4, BackgroundFlag::None, TextAlignment::Left, "XP:");
    render_progress_bar(&mut tcod.gui, 4, 4, BAR_WIDTH, xp, xp_target, colors::LIGHT_BLUE, colors::BLACK);

    tcod.gui.print_ex(1, 9, BackgroundFlag::None, TextAlignment::Left, format!("Dungeon Level: {}", game.dungeon_level));

    // Display names of objects under mouse
    tcod.gui.set_default_foreground(colors::LIGHT_GREEN);
    tcod.gui.print_ex(1, 1, BackgroundFlag::None, TextAlignment::Left, get_names_under_mouse(tcod.mouse, game));

    // Blit gui to root console
    blit(&tcod.gui, (0, 0), (SCREEN_WIDTH, PANEL_HEIGHT), &mut tcod.root, (0, PANEL_Y), 1.0, 1.0);
}

/// Renders a progress bar to the gui panel
fn render_progress_bar(panel: &mut Offscreen, x: i32, y: i32, total_width: i32, value: i32, max: i32, bar_color: Color, bg_color: Color)
{
    let bar_width = (value as f32 / max as f32 * total_width as f32) as i32;

    // Bar background
    panel.set_default_background(bg_color);
    panel.rect(x, y, total_width, 1, false, BackgroundFlag::Set);

    // Bar progress
    panel.set_default_background(bar_color);
    if bar_width > 0
    {
        panel.rect(x, y, bar_width, 1, false, BackgroundFlag::Set);
    }

    // Bar text
    panel.set_default_foreground(colors::WHITE);
    panel.print_ex(x + total_width / 2, y, BackgroundFlag::None, TextAlignment::Center, format!("{}/{}", value, max));
}

/// Returns the names of all the objects under the mouse
fn get_names_under_mouse(mouse: Mouse, game: &Game) -> String
{
    let (x, y) = (mouse.cx as i32, mouse.cy as i32);
    let names = game.objects
        .iter()
        .filter(|o| o.pos == (x, y) && game.map.is_in_fov(o.pos))
        .map(|o| o.name.clone())
        .collect::< Vec< _ > >();

    names.join(", ")
}