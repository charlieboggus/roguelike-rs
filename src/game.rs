use crate::TCOD;
use crate::map::{ Map, MAP_WIDTH, MAP_HEIGHT };
use crate::object::{ self, Object };
use crate::fighter::{ Fighter, DeathCallback };
use crate::item;
use crate::ai::{ self, Ai };
use crate::menu;
use crate::gui::{ self, * };

use tcod::colors;
use tcod::console::*;
use tcod::input::{ Key, KeyCode, Event };

use std::error::Error;
use std::fs::File;
use std::io::{ Read, Write };

pub const PLAYER_ID: usize = 0;
pub const LEVEL_UP_BASE: i32 = 200;
pub const LEVEL_UP_FACTOR: i32 = 150;

/// Deserializes a game save file and loads a game from the data
pub fn load_game() -> Result< Game, Box< Error > >
{
    let mut json_save = String::new();
    let mut file = File::open("savegame")?;
    file.read_to_string(&mut json_save)?;
    let result = serde_json::from_str::< Game >(&json_save)?;

    Ok(result)
}

/// Represents an instance of the game
#[derive(Serialize, Deserialize)]
pub struct Game
{
    pub map: Map,
    pub objects: Vec< Object >,
    pub inventory: Vec< Object >,
    pub log: Messages,
    pub dungeon_level: i32,
}

impl Game
{
    /// Creates a new game
    pub fn new() -> Self
    {
        // Create player
        // TODO: character creation to determine player name and stats?
        let mut player = Object::new(0, 0, '@', colors::WHITE, "Player", true);
        player.alive = true;
        player.fighter = Some(Fighter::new(1, 1, 1, 1, 1, 1, 0, DeathCallback::PlayerDeath));

        // Create objects vec
        let mut objects = vec![ player ];

        // Create initial map & set player starting position
        let mut map = Map::new();
        map.generate(&mut objects, 1);

        // Create inventory w/ starting gear
        let mut inventory: Vec< Object > = vec![];
        // TODO: starting gear

        // Return newly created game
        Game
        {
            map: map,
            objects: objects,
            inventory: inventory,
            log: vec![],
            dungeon_level: 1,
        }
    }

    /// Starts the game itself
    pub fn start(&mut self, tcod: &mut TCOD)
    {
        let mut prev_player_pos = (-1, -1);
        let mut key = Default::default();
        self.log.add("You awaken in a dark dungeon...", colors::RED);

        while !tcod.root.window_closed()
        {
            // Check for tcod input events
            match tcod::input::check_for_event(tcod::input::MOUSE | tcod::input::KEY_PRESS)
            {
                Some((_, Event::Mouse(m))) => tcod.mouse = m,
                Some((_, Event::Key(k))) => key = k,
                _ => key = Default::default()
            }

            // Recompute the FOV map if the player position has updated
            let fov_recompute = prev_player_pos != self.objects[PLAYER_ID].pos;

            // Render
            self.render(tcod, fov_recompute);

            // Update player
            player_level_up(tcod, self);
            prev_player_pos = self.objects[PLAYER_ID].pos;
            let player_action = self.handle_key_input(tcod, key);
            if player_action == PlayerAction::Exit
            {
                self.save_game().unwrap();
                break;
            }

            // Update AI
            if self.objects[PLAYER_ID].alive && player_action != PlayerAction::NoAction
            {
                for id in 0..self.objects.len()
                {
                    if self.objects[id].ai.is_some()
                    {
                        ai_take_turn(id, self);
                    }
                }
            }
        }
    }

    /// Renders the game
    fn render(&mut self, tcod: &mut TCOD, fov_recompute: bool)
    {
        tcod.root.clear();
        tcod.con.clear();

        // Recompute FOV if necessary
        if fov_recompute
        {
            self.map.recompute_fov(self.objects[PLAYER_ID].pos);
        }

        // Render map
        self.map.draw(&mut tcod.con);

        // Render objects
        let mut to_draw: Vec< _ > = self.objects.iter().filter(|o| { self.map.is_in_fov(o.pos) || (o.always_visible && self.map.is_explored(o.pos)) }).collect();
        to_draw.sort_by(|o1, o2| o1.solid.cmp(&o2.solid));
        for obj in &to_draw
        {
            obj.draw(&mut tcod.con);
        }

        // Blit map and objects to root console
        blit(&mut tcod.con, (0, 0), (MAP_WIDTH, MAP_HEIGHT), &mut tcod.root, (0, 0), 1.0, 1.0);

        // Render gui
        gui::render_gui(tcod, self);

        tcod.root.flush();
    }

    /// Handles player keyboard input and game controls
    fn handle_key_input(&mut self, tcod: &mut TCOD, key: Key) -> PlayerAction
    {
        let player_alive = self.objects[PLAYER_ID].alive;
        match (key, player_alive)
        {
            // Escape to exit game
            (Key { code: KeyCode::Escape, .. }, _) => PlayerAction::Exit,

            // Up to move player upwards
            (Key { code: KeyCode::Up, .. }, true) => { object::move_by(PLAYER_ID, 0, -1, self); PlayerAction::Action },

            // Down to move player downwards
            (Key { code: KeyCode::Down, .. }, true) => { object::move_by(PLAYER_ID, 0, 1, self); PlayerAction::Action },

            // Left to move player left
            (Key { code: KeyCode::Left, .. }, true) => { object::move_by(PLAYER_ID, -1, 0, self); PlayerAction::Action },

            // Right to move player right
            (Key { code: KeyCode::Right, .. }, true) => { object::move_by(PLAYER_ID, 1, 0, self); PlayerAction::Action },

            // R to skip player turn
            (Key { printable: 'r', .. }, true) => PlayerAction::Action,

            // F to interact with item or non-monster object with
            // TODO: this
            (Key { printable: 'f', .. }, true) => 
            {
                // first check if the object is an item
                let item_id = self.objects.iter().position(|o| o.pos == self.objects[PLAYER_ID].pos && o.item.is_some());
                if let Some(item_id) = item_id
                {
                    item::pick_item_up(item_id, self);
                    return PlayerAction::NoAction;
                }

                // Next check if it's stairs
                let player_on_stairs = self.objects.iter().any(|o| { o.pos == self.objects[PLAYER_ID].pos && o.name == "Stairs" });
                if player_on_stairs
                {
                    advance_dungeon_level(self);
                    return PlayerAction::NoAction;
                }

                PlayerAction::NoAction 
            },

            // E to interact with monster (melee attack)
            // TODO: this
            (Key { printable: 'e', .. }, true) => 
            {
                PlayerAction::Action 
            },

            // I to open inventory
            (Key { printable: 'i', .. }, true) => 
            {
                let inv_index = menu::inventory_menu(&self.inventory, "Press the key next to an item to use it, or any other to cancel.\n", &mut tcod.root);
                if let Some(inv_index) = inv_index
                {
                    item::use_item(inv_index, self, tcod);
                }

                PlayerAction::NoAction 
            },

            // O to open inventory in drop mode
            (Key { printable: 'o', .. }, true) =>
            {
                let inv_index = menu::inventory_menu(&self.inventory, "Press the key next to an item to drop it, or any other key to cancel.\n", &mut tcod.root);
                if let Some(inv_index) = inv_index
                {
                    item::drop_item(inv_index, self);
                }
                PlayerAction::NoAction
            },

            // C to open character info 
            (Key { printable: 'c', .. }, true) => 
            { 
                menu::character_menu(self, &mut tcod.root);
                PlayerAction::NoAction 
            },

            // No other controls at this time...
            _ => PlayerAction::NoAction
        }
    }

    /// Serialize the current game state into a save file using Serde Json
    fn save_game(&mut self) -> Result< (), Box< Error > >
    {
        let save_data = serde_json::to_string(&(self))?;
        let mut file = File::create("savegame")?;
        file.write_all(save_data.as_bytes())?;

        Ok(())
    }
}

/// Represents an "action" the player can take on their turn
#[derive(Debug, Clone, Copy, PartialEq)]
enum PlayerAction
{
    Exit,
    Action,
    NoAction
}

/// Increases the player's level if necessary
fn player_level_up(tcod: &mut TCOD, game: &mut Game)
{
    let player = &mut game.objects[PLAYER_ID];
    let level_xp = LEVEL_UP_BASE + player.level * LEVEL_UP_FACTOR;

    if player.fighter.as_ref().map_or(0, |f| f.xp) >= level_xp
    {
        player.level += 1;
        game.log.add(format!("Your battle skills grow stronger! You've reached level {}!", player.level), colors::YELLOW);

        let fighter = player.fighter.as_mut().unwrap();
        let stat = menu::level_up_menu(fighter, "Choose a stat to increase:", &mut tcod.root);
        match stat.unwrap()
        {
            // Vitality
            0 => {
                fighter.base_vit += 1;
                fighter.max_hp = 10 + (5 * fighter.base_vit);
            },

            // Attack
            1 => fighter.base_atk += 1,

            // Strength
            2 => fighter.base_str += 1,

            // Defense
            3 => fighter.base_def += 1,

            // Intelligence
            4 => fighter.base_int += 1,

            // Luck
            5 => fighter.base_lck += 1,

            _ => unreachable!()
        }

        fighter.hp = fighter.max_hp;
        fighter.xp -= level_xp;
    }
}

/// Called whenever it is the ai's "turn" (after the player took an action)
fn ai_take_turn(id: usize, game: &mut Game)
{
    if let Some(ai) = game.objects[id].ai.take()
    {
        let new_ai = match ai
        {
            Ai::BasicMonster => ai::ai_basic_monster(id, game),
        };

        game.objects[id].ai = Some(new_ai);
    }
}

/// Advances the dungeon level
fn advance_dungeon_level(game: &mut Game)
{
    // Heal player up by half their max hp
    game.log.add("You take a moment to rest and recover your strength.", colors::VIOLET);
    let heal_amt = game.objects[PLAYER_ID].fighter.map_or(0, |f| f.max_hp / 2);
    game.objects[PLAYER_ID].heal(heal_amt);

    // Create the new dungeon level
    game.log.add("You descend deeper into the heart of the dungeon...", colors::RED);
    game.dungeon_level += 1;
    game.map.generate(&mut game.objects, game.dungeon_level);
}