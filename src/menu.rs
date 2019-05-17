use crate::{ TCOD, SCREEN_WIDTH, SCREEN_HEIGHT };
use crate::game::{ self, Game };

use tcod::colors::{ self, Color };
use tcod::console::*;

const MAIN_MENU_WIDTH: i32 = 24;
const INVENTORY_MENU_WIDTH: i32 = 50;
const CHARACTER_MENU_WIDTH: i32 = 50;
const LEVEL_UP_MENU_WIDTH: i32 = 40;

pub fn main_menu(tcod: &mut TCOD)
{
    let img = tcod::image::Image::from_file("./res/menu_background.png").ok().expect("Failed to load background image!");
    while !tcod.root.window_closed()
    {
        // Draw background image & title/version text
        tcod::image::blit_2x(&img, (0, 0), (-1, -1), &mut tcod.root, (0, 0));
        tcod.root.set_default_foreground(colors::WHITE);
        tcod.root.print_ex(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2 - 4, BackgroundFlag::None, TextAlignment::Center, "roguelike-rs");
        tcod.root.print_ex(SCREEN_WIDTH / 2, SCREEN_HEIGHT - 2, BackgroundFlag::None, TextAlignment::Center, "version 0.1.0");

        // Menu functionality
        let opts = &["New Game", "Continue Game", "Exit"];
        let choice = menu("", opts, MAIN_MENU_WIDTH, colors::BLACK, 0.7, &mut tcod.root);
        match choice
        {
            // New Game
            Some(0) => 
            {
                // Create a new game and start it
                let mut game = Game::new();
                game.start(tcod);
            }

            // Continue Game
            Some(1) =>
            {
                match game::load_game()
                {
                    // If the game loaded successfully we can just start it
                    Ok(mut game) =>
                    {
                        game.start(tcod);
                    }

                    // If there was an error simply display a message
                    Err(_e) =>
                    {
                        msg_box("No saved game to load!", MAIN_MENU_WIDTH, &mut tcod.root);
                        continue;
                    }
                }
            }

            // Exit
            Some(2) =>
            {
                break;
            }

            _ => {}
        }
    }
}

fn menu< T: AsRef< str > >(header: &str, opts: &[T], width: i32, background_color: Color, background_alpha: f32, root: &mut Root) -> Option< usize >
{
    assert!(opts.len() <= 26, "Cannot have a menu with more than 26 options.");
    
    let header_height = if header.is_empty() { 0 } else { root.get_height_rect(0, 0, width, SCREEN_HEIGHT, header) };
    let height = opts.len() as i32 + header_height;

    let mut menu_con = Offscreen::new(width, height);
    menu_con.set_default_background(background_color);
    menu_con.set_default_foreground(colors::WHITE);
    menu_con.clear();
    menu_con.print_rect_ex(0, 0, width, height, BackgroundFlag::None, TextAlignment::Left, header);
    for (i, opt_text) in opts.iter().enumerate()
    {
        let letter = (b'a' + i as u8) as char;
        let text = format!("{}) {}", letter, opt_text.as_ref());
        menu_con.print_ex(0, header_height + i as i32, BackgroundFlag::None, TextAlignment::Left, text);
    }

    let x = SCREEN_WIDTH / 2 - width / 2;
    let y = SCREEN_HEIGHT / 2 - height / 2;
    blit(&mut menu_con, (0, 0), (width, height), root, (x, y), 1.0, background_alpha);
    root.flush();

    let key = root.wait_for_keypress(true);
    if key.printable.is_alphabetic()
    {
        let index = key.printable.to_ascii_lowercase() as usize - 'a' as usize;
        if index < opts.len()
        {
            Some(index)
        }
        else
        {
            None
        }
    }
    else
    {
        None
    }
}

fn msg_box(text: &str, width: i32, root: &mut Root)
{
    let opts: &[&str] = &[];
    menu(text, opts, width, colors::BLACK, 1.0, root);
}