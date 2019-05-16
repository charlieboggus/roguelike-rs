use crate::{ SCREEN_WIDTH, SCREEN_HEIGHT };
use crate::game::{ self, TCOD, Game };
use crate::map;
use tcod::console::*;
use tcod::colors;

pub fn main_menu(tcod: &mut TCOD)
{
    // load main menu background image
    let img = tcod::image::Image::from_file("menu_background.png").ok().expect("Failed to load asset: menu_background.png");

    while !tcod.root.window_closed()
    {
        // Draw main menu background image
        tcod::image::blit_2x(&img, (0, 0), (-1, -1), &mut tcod.root, (0, 0));

        // Draw main menu stuff
        tcod.root.set_default_foreground(colors::WHITE);
        tcod.root.print_ex(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2 - 4, BackgroundFlag::None, TextAlignment::Center, "roguelike-rs");
        tcod.root.print_ex(SCREEN_WIDTH / 2, SCREEN_HEIGHT - 2, BackgroundFlag::None, TextAlignment::Center, "by Charlie Boggus");

        // Menu options & wait for player selection
        let opts = &["New Game", "Continue Game", "Quit"];
        let selection = menu("", opts, 24, tcod);
        match selection
        {
            // New Game
            Some(0) => 
            {
                let mut game = game::new_game(tcod);
                game::play_game(tcod, &mut game);
            },

            // Continue game
            Some(1) => 
            {
                match game::load_game()
                {
                    Ok(mut game) => 
                    {
                        map::create_fov_map(&game.map, &mut tcod.fov);
                        game::play_game(tcod, &mut game);
                    },

                    Err(_e) =>
                    {
                        msg_box("\nNo saved game to load!\n", 24, tcod);
                        continue;
                    }
                }
            },

            // Quit
            Some(2) => 
            {
                break;
            },

            _ => {}
        }
    }
}

pub fn inventory_menu(tcod: &mut TCOD, game: &mut Game, header: &str)
{
}

pub fn character_info_menu(tcod: &mut TCOD, game: &mut Game)
{
}

pub fn level_up_menu()
{
}

/// Function for displaying a generic game menu
fn menu< T: AsRef< str > >(header: &str, opts: &[T], width: i32, tcod: &mut TCOD) -> Option< usize >
{
    assert!(opts.len() <= 26, "Cannot have a menu with more than 26 options.");

    let header_height = if header.is_empty() { 0 } else { tcod.root.get_height_rect(0, 0, width, SCREEN_HEIGHT, header) };
    let height = opts.len() as i32 + header_height;

    // Create an offscreen console to draw the menu to
    let mut menu_con = Offscreen::new(width, height);
    menu_con.set_default_background(colors::DARKER_BLUE);
    menu_con.set_default_foreground(colors::WHITE);
    menu_con.print_rect_ex(0, 0, width, height, BackgroundFlag::None, TextAlignment::Left, header);
    for (index, text) in opts.iter().enumerate()
    {
        let ch = (b'a' + index as u8) as char;
        let tx = format!("{}). {}", ch, text.as_ref());
        menu_con.print_ex(0, header_height + index as i32, BackgroundFlag::None, TextAlignment::Left, tx);
    }

    // Blit the menu to the root console
    let x = SCREEN_WIDTH / 2 - width / 2;
    let y = SCREEN_HEIGHT / 2 - height / 2;
    tcod::console::blit(&mut menu_con, (0, 0), (width, height), &mut tcod.root, (x, y), 1.0, 0.7);
    tcod.root.flush();

    // Get and return the player's menu item selection
    let key = tcod.root.wait_for_keypress(true);
    if key.printable.is_alphabetic()
    {
        let selection = key.printable.to_ascii_lowercase() as usize - 'a' as usize;
        if selection < opts.len()
        {
            Some(selection)
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

fn msg_box(text: &str, width: i32, tcod: &mut TCOD)
{
    let opts: &[&str] = &[];
    menu(text, opts, width, tcod);
}