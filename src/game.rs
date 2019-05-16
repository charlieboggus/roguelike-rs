use crate::entity::Entity;
use crate::map::{ self, Map, MAP_WIDTH, MAP_HEIGHT };
use crate::menus::*;
use crate::gui::{ self, * };

use tcod::colors;
use tcod::console::*;
use tcod::map::{ Map as FovMap, FovAlgorithm };
use tcod::input::{ Event, Key, Mouse };

use std::cmp;
use std::error::Error;
use std::fs::File;
use std::io::{ Read, Write };

pub const PLAYER_ID: usize = 0;

const FOV_TORCH_RADIUS: i32 = 10;
const FOV_LIGHT_WALLS: bool = true;
const FOV_ALGORITHM: FovAlgorithm = FovAlgorithm::Basic;

/// Holds all of the core tcod fields
pub struct TCOD
{
    pub root: Root,
    pub con: Offscreen,
    pub panel: Offscreen,
    pub mouse: Mouse,
    pub fov: FovMap
}

/// Represents an instance of the game and all of the core game fields
#[derive(Serialize, Deserialize)]
pub struct Game
{
    pub map: Map,
    pub entities: Vec< Entity >,
    pub inventory: Vec< Entity >,
    pub log: Messages,
    pub dungeon_level: i32
}

/// Represents a game action that the player can take
#[derive(Debug, Clone, Copy, PartialEq)]
enum PlayerAction
{
    ExitGame,
    Action,
    NoAction
}

/// Creates a new instance of the game
pub fn new_game(tcod: &mut TCOD) -> Game
{
    let mut player = Entity::new(0, 0, '@', colors::WHITE, "Player", true);
    player.alive = true;
    // todo: player fighter setup
    
    let mut entities = vec![player];
    let mut game = Game {
        map: map::generate_map(&mut entities, 1),
        entities: entities,
        inventory: vec![],
        log: vec![],
        dungeon_level: 1
    };

    // Give player starting gear?

    map::create_fov_map(&game.map, &mut tcod.fov);
    game.log.add("Welcome to roguelike-rs! Best of luck on your adventure!", colors::WHITE);

    game
}

/// Creates an instance of the game using data loaded from save file
pub fn load_game() -> Result< Game, Box< Error > >
{
    let mut save_data = String::new();
    let mut file = File::open("savegame").unwrap();
    file.read_to_string(&mut save_data).unwrap();
    let result = serde_json::from_str::< Game >(&save_data).unwrap();

    Ok(result)
}

/// Serializes the game data to a save file
pub fn save_game(game: &mut Game) -> Result< (), Box< Error > >
{
    let save_data = serde_json::to_string(&game).unwrap();
    let mut file = File::create("savegame").unwrap();
    file.write_all(save_data.as_bytes()).unwrap();

    Ok(())
}

/// Play the given game
pub fn play_game(tcod: &mut TCOD, game: &mut Game)
{
    let mut prev_player_pos = (-1, -1);
    let mut key = Default::default();

    while !tcod.root.window_closed()
    {
        // Check for keyboard and mouse events
        match tcod::input::check_for_event(tcod::input::MOUSE | tcod::input::KEY_PRESS)
        {
            Some((_, Event::Mouse(m))) => tcod.mouse = m,
            Some((_, Event::Key(k))) => key = k,
            _ => key = Default::default()
        }

        // Do we need to recompute the fov?
        let fov_recompute = prev_player_pos != game.entities[PLAYER_ID].pos;

        // Render
        tcod.con.clear();
        render_all(tcod, game, fov_recompute);
        tcod.root.flush();

        // Update player
        // TODO: level up!
        prev_player_pos = game.entities[PLAYER_ID].pos;
        let player_action = handle_key_input(key, tcod, game);
        if player_action == PlayerAction::ExitGame
        {
            save_game(game).unwrap();
            break;
        }

        // Update ai
        if game.entities[PLAYER_ID].alive && player_action != PlayerAction::NoAction
        {
            for id in 0..game.entities.len()
            {
                // ai_take_turn
            }
        }
    }
}

/// Render all of the game stuff
fn render_all(tcod: &mut TCOD, game: &mut Game, fov_recompute: bool)
{
    // Recompute FOV if necessary
    if fov_recompute
    {
        let player = &game.entities[PLAYER_ID];
        tcod.fov.compute_fov(player.pos.0, player.pos.1, FOV_TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGORITHM);
    }

    // Render map
    map::render_map(game);

    // Sort entities .. and then render them
    let mut entities: Vec< _ > = game.entities.iter().filter(|e| tcod.fov.is_in_fov(e.pos.0, e.pos.1) || (e.always_visible && game.map[e.pos.0 as usize][e.pos.1 as usize].explored)).collect();
    entities.sort_by(|e1, e2| e1.solid.cmp(&e2.solid));
    for e in &entities
    {
        e.draw(&mut tcod.con);
    }

    // blit the map and entities to the root console
    tcod::console::blit(&mut tcod.con, (0, 0), (MAP_WIDTH, MAP_HEIGHT), &mut tcod.root, (0, 0), 1.0, 1.0);

    // Render GUI stuff
    gui::render_gui(tcod, game);
}

/// Handle player keyboard input
fn handle_key_input(key: Key, tcod: &mut TCOD, game: &mut Game) -> PlayerAction
{
    use tcod::input::KeyCode::*;

    let player_alive = game.entities[PLAYER_ID].alive;
    match (key, player_alive)
    {
        // Escape to exit game
        (Key { code: Escape, .. }, _) => PlayerAction::ExitGame,

        // Move up
        (Key { printable: 'w', .. }, true) => {
            player_take_turn(0, -1, game);
            PlayerAction::Action
        },

        // Move down
        (Key { printable: 's', .. }, true) => {
            player_take_turn(0, 1, game);
            PlayerAction::Action
        },

        // Move left
        (Key { printable: 'a', .. }, true) => {
            player_take_turn(-1, 0, game);
            PlayerAction::Action
        },

        // Move right
        (Key { printable: 'd', .. }, true) => {
            player_take_turn(1, 0, game);
            PlayerAction::Action
        },

        // Skip turn
        (Key { printable: 'r', .. }, true) => PlayerAction::Action,

        _ => PlayerAction::NoAction
    }
}

/// Stuff done on the player's turn
fn player_take_turn(dx: i32, dy: i32, game: &mut Game)
{
    let x = game.entities[PLAYER_ID].pos.0 + dx;
    let y = game.entities[PLAYER_ID].pos.1 + dy;
}

/// Stuff done on the ai's turn
fn ai_take_turn(id: usize, game: &mut Game, fov: &FovMap)
{
}