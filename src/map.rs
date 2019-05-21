use crate::object::Object;
use crate::game::PLAYER_ID;
use crate::fighter::{ Fighter, DeathCallback };
use crate::ai::Ai;
use crate::item::*;

use tcod::colors;
use tcod::chars;
use tcod::console::{ Console, BackgroundFlag };
use tcod::map::{ Map as FovMap, FovAlgorithm };
use rand::{ Rng, distributions::WeightedIndex, prelude::* };
use std::cmp;

pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 40;

const ROOM_MIN_SIZE: i32 = 6;
const ROOM_MAX_SIZE: i32 = 10;
const MAX_ROOM_COUNT: i32 = 30;

/// Represents the game map and all of its associated fields.
#[derive(Serialize, Deserialize)]
pub struct Map
{
    /// A map is simply comprised of a 2D array of Tiles
    pub tiles: Vec< Vec< Tile > >,

    /// The width of the map
    pub width: i32,

    /// The height of the map
    pub height: i32,

    /// The map's FOV map
    #[serde(skip)]
    fov_wrapper: FovWrapper
}

impl Map
{
    /// Creates a new instance of the map. Does not generate the map, however.
    /// Simply initializes all of the map fields.
    pub fn new() -> Self
    {
        let mut map = Map {
            tiles: vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize],
            width: MAP_WIDTH,
            height: MAP_HEIGHT,
            fov_wrapper: FovWrapper::new()
        };
        map.generate_fov_map();

        map
    }

    /// Function to generate the map
    pub fn generate(&mut self, objects: &mut Vec< Object >, dungeon_level: i32)
    {
        self.tiles = vec![vec![Tile::wall(); self.height as usize]; self.width as usize];
        let mut rooms: Vec< Rect > = vec![];
        
        // Remove everything except player from objects vec when generating a new map
        objects.truncate(1);

        for _ in 0..MAX_ROOM_COUNT
        {
            // Generate random size for room
            let room_w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
            let room_h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);

            // Generate random position for room
            let room_x = rand::thread_rng().gen_range(0, self.width - room_w);
            let room_y = rand::thread_rng().gen_range(0, self.height - room_h);

            // Create a new rect from the generated position and size
            let new_room = Rect::new(room_x, room_y, room_w, room_h);

            // Only create the room if the newly created one doesn't intersect with any other
            let failed = rooms.iter().any(|o| new_room.intersects_with(o));
            if !failed
            {
                self.generate_room(&new_room);
                let (new_x, new_y) = new_room.get_center();

                if rooms.is_empty()
                {
                    // Position player in center of first room
                    objects[PLAYER_ID].pos = (new_x, new_y);
                }
                else
                {
                    // Connect all the rooms after first with tunnels & put stuff in em
                    let (prev_x, prev_y) = rooms[rooms.len() - 1].get_center();
                    if rand::random()
                    {
                        self.generate_horizontal_tunnel(prev_x, new_x, prev_y);
                        self.generate_vertical_tunnel(prev_y, new_y, new_x);
                    }
                    else
                    {
                        self.generate_vertical_tunnel(prev_y, new_y, prev_x);
                        self.generate_horizontal_tunnel(prev_x, new_x, new_y);
                    }

                    self.populate_room(&new_room, objects, dungeon_level);
                }

                rooms.push(new_room);
            }
        }

        // Generate stairs at center of last room
        let (stair_x, stair_y) = rooms[rooms.len() - 1].get_center();
        let mut stairs = Object::new(stair_x, stair_y, 'H', colors::WHITE, "Stairs", false);
        stairs.always_visible = true;
        objects.push(stairs);

        self.generate_fov_map();
    }

    /// Draws the map to the given TCOD console
    pub fn draw(&mut self, con: &mut Console)
    {
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                let visible = self.is_in_fov((x, y));
                let wall = self.tiles[x as usize][y as usize].blocks_sight;
                let color = match (visible, wall) 
                {
                    (false, false)  => colors::DARKEST_SEPIA,
                    (false, true)   => colors::DARKEST_GREY,
                    (true, false)   => colors::DARK_SEPIA,
                    (true, true)    => colors::DARK_GREY
                };

                let explored = &mut self.tiles[x as usize][y as usize].explored;
                if visible
                {
                    *explored = true;
                }

                if *explored
                {
                    con.set_char_background(x, y, color, BackgroundFlag::Set);
                }
            }
        }
    }

    /// Creates the FOV map
    pub fn generate_fov_map(&mut self)
    {
        for y in 0..self.height
        {
            for x in 0..self.width
            {
                self.fov_wrapper.fov.set(x, y, !self.tiles[x as usize][y as usize].blocks_sight, !self.tiles[x as usize][y as usize].blocked);
            }
        }
    }

    /// Recomputes the player's FOV
    pub fn recompute_fov(&mut self, pos: (i32, i32))
    {
        self.fov_wrapper.fov.compute_fov(pos.0, pos.1, 10, true, FovAlgorithm::Basic);
    }

    /// Returns true if the given position is in the player's FOV
    pub fn is_in_fov(&self, pos: (i32, i32)) -> bool
    {
        self.fov_wrapper.fov.is_in_fov(pos.0, pos.1)
    }

    /// Returns true if the tile at the given position is blocked (either a wall or occupied)
    pub fn is_blocked(&self, x: i32, y: i32, objects: &[Object]) -> bool
    {
        if self.tiles[x as usize][y as usize].blocked
        {
            return true;
        }
        objects.iter().any(|o| { o.solid && o.pos.0 == x && o.pos.1 == y })
    }

    /// Returns true if the tile at the given position has been explored
    pub fn is_explored(&self, pos: (i32, i32)) -> bool
    {
        self.tiles[pos.0 as usize][pos.1 as usize].explored
    }

    /// Function to carve out a room on the map using the position and size 
    /// of the given rect
    fn generate_room(&mut self, room: &Rect)
    {
        for x in (room.x1 + 1)..room.x2
        {
            for y in (room.y1 + 1)..room.y2
            {
                self.tiles[x as usize][y as usize] = Tile::empty();
            }
        }
    }

    /// Function to carve out a horizontal tunnel on the map to connect two rooms
    fn generate_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32)
    {
        for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1)
        {
            self.tiles[x as usize][y as usize] = Tile::empty();
        }
    }

    /// Function to carve out a vertical tunnel on the map to connect two rooms
    fn generate_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32)
    {
        for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1)
        {
            self.tiles[x as usize][y as usize] = Tile::empty();
        }
    }

    /// Function to spawn monsters and items in the given room
    fn populate_room(&mut self, room: &Rect, objects: &mut Vec< Object >, dungeon_level: i32)
    {
        // Maximum number of monsters that can spawn in a room is determined by dungeon level
        let max_monsters = from_dungeon_level(&[
            Transition { level: 1, value: 2 },  // Levels 1-3: 2 monsters max per room 
            Transition { level: 4, value: 4 },  // Levels 4-7: 4 monsters max per room
            Transition { level: 8, value: 5 }], // Levels 8+:  5 monsters max per room
            dungeon_level
        );

        // Number of monsters in the room is a random number [0, max]
        let num_monsters = rand::thread_rng().gen_range(0, max_monsters + 1);

        // The different types of monsters that can spawn
        let monster_choices = [
            "Orc", 
            "Troll"
        ];

        // The weighted probabilities for each type of monster to spawn
        let monster_weights = [
            // Orc monster weight
            80, 

            // Troll Monster weight
            from_dungeon_level(&[
                Transition { level: 3, value: 15 }, 
                Transition { level: 5, value: 30 }, 
                Transition { level: 8, value: 50 }], 
                dungeon_level
            )
        ];

        // Distribution using weighted index sampling to determine the type of
        // monster that is spawned
        let monster_dist = WeightedIndex::new(&monster_weights).unwrap();

        for _ in 0..num_monsters
        {
            // Generate random position for monster
            let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
            let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

            if !self.is_blocked(x, y, objects)
            {
                // Generate a monster based off of the weighted sample from our monster distribution
                let mut monster = match monster_choices[monster_dist.sample(&mut rand::thread_rng())]
                {
                    "Orc" => 
                    {
                        let mut orc = Object::new(x, y, 'O', colors::DESATURATED_GREEN, "Orc", true);
                        orc.fighter = Some(Fighter::new(2, 3, 2, 3, 2, 0, 0, 50, DeathCallback::MonsterDeath));
                        orc.ai = Some(Ai::BasicMonster);
                        orc
                    },

                    "Troll" => 
                    {
                        let mut troll = Object::new(x, y, 'T', colors::DARKER_GREEN, "Troll", true);
                        troll.fighter = Some(Fighter::new(5, 5, 5, 3, 3, 0, 0, 100, DeathCallback::MonsterDeath));
                        troll.ai = Some(Ai::BasicMonster);
                        troll
                    },

                    _ => unreachable!()
                };

                monster.alive = true;
                objects.push(monster);
            }
        }

        // Maximum number of items per room is determined by the dungeon level
        let max_items = from_dungeon_level(&[
            Transition { level: 1, value: 1 }, // Levels 1-4: 1 item max per room
            Transition { level: 5, value: 2 }, // Levels 5+:  2 item max per room
            ], 
            dungeon_level
        );

        // Generate number of items in the room [0, max]
        let num_items = rand::thread_rng().gen_range(0, max_items + 1);

        // The possible items that can be spawned
        let item_choices = [ 
            Item::HealthPotion, 
            Item::Sword, 
            Item::Shield, 
            Item::PlateArmor 
        ];

        // The weights for each different type of item to spawn
        let item_weights = [
            // Health potion weight
            35,

            // Sword weight
            from_dungeon_level(&[Transition{ level: 4, value: 25 }], dungeon_level),

            // Shield weight
            from_dungeon_level(&[Transition{ level: 5, value: 20 }], dungeon_level),

            // Plate Armor weight
            from_dungeon_level(&[Transition{ level: 3, value: 15 }], dungeon_level),
        ];

        // Distribution using weighted index sampling to determine the type of
        // item that is spawned
        let item_dist = WeightedIndex::new(&item_weights).unwrap();

        for _ in 0..num_items
        {
            // Generate a random position in the room for the item
            let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
            let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

            if !self.is_blocked(x, y, objects)
            {
                let mut item = match item_choices[item_dist.sample(&mut rand::thread_rng())]
                {
                    Item::HealthPotion =>
                    {
                        let mut item = Object::new(x, y, '!', colors::LIGHT_VIOLET, "Health Potion", false);
                        item.item = Some(Item::HealthPotion);
                        item
                    },

                    Item::Sword =>
                    {
                        // TODO: figure out different tiers of sword
                        let mut item = Object::new(x, y, '/', colors::BRASS, "Sword", false);
                        item.item = Some(Item::Sword);
                        item.equipment = Some(Equipment {
                            slot: EquipmentSlot::RightHand,
                            equipped: false,
                            vit_bonus: 0,
                            atk_bonus: 2,
                            str_bonus: 2,
                            def_bonus: 0,
                            dex_bonus: 0,
                            int_bonus: 0,
                            lck_bonus: 0
                        });

                        item
                    },

                    Item::Shield =>
                    {
                        // TODO: figure out different tiers of shield
                        let mut item = Object::new(x, y, '0', colors::BRASS, "Shield", false);
                        item.item = Some(Item::Sword);
                        item.equipment = Some(Equipment {
                            slot: EquipmentSlot::RightHand,
                            equipped: false,
                            vit_bonus: 3,
                            atk_bonus: 0,
                            str_bonus: 0,
                            def_bonus: 6,
                            dex_bonus: 6,
                            int_bonus: 0,
                            lck_bonus: 0
                        });

                        item
                    },

                    Item::PlateArmor =>
                    {
                        // TODO: figure out different tiers of plate armor
                        let mut item = Object::new(x, y, '#', colors::BRASS, "Plate Armor", false);
                        item.item = Some(Item::Sword);
                        item.equipment = Some(Equipment {
                            slot: EquipmentSlot::RightHand,
                            equipped: false,
                            vit_bonus: 5,
                            atk_bonus: 2,
                            str_bonus: 2,
                            def_bonus: 2,
                            dex_bonus: 2,
                            int_bonus: 0,
                            lck_bonus: 5
                        });

                        item
                    }
                };

                item.always_visible = true;
                objects.push(item);
            }
        }
    }
}

/// Represents a single tile on the map and its associated properties.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tile
{
    /// True if the tile cannot be moved through
    pub blocked: bool,

    /// True if the tile blocks line of sight
    pub blocks_sight: bool,

    /// True if the tile has been explored
    pub explored: bool,
}

impl Tile
{
    /// Creates an empty tile
    fn empty() -> Self
    {
        Tile
        {
            blocked: false,
            blocks_sight: false,
            explored: false,
        }
    }

    /// Creates a wall tile
    fn wall() -> Self
    {
        Tile
        {
            blocked: true,
            blocks_sight: true,
            explored: false,
        }
    }
}

/// Represents a rectangle of tiles on the map. Rectangles are used for creating
/// rooms.
#[derive(Debug, Clone, Copy)]
struct Rect
{
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32
}

impl Rect
{
    /// Creates a new rectangle of the given size at the given position
    fn new(x: i32, y: i32, w: i32, h: i32) -> Self
    {
        Rect
        {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h
        }
    }

    /// Returns the coordinates of a rectangle's center
    fn get_center(&self) -> (i32, i32)
    {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;

        (center_x, center_y)
    }

    /// Returns true if one rectangle intersects with another given rectangle
    fn intersects_with(&self, other: &Rect) -> bool
    {
        (self.x1 <= other.x2) && (self.x2 >= other.x1) && (self.y1 <= other.y2) && (self.y2 >= other.y1)
    }
}

/// Structure that associates a value with a level. Used for different values
/// for things at different dungeon levels
struct Transition
{
    level: i32,
    value: i32
}

/// Returns a value that depends on the given level from the given table that
/// specifies what value occurs after each level
fn from_dungeon_level(table: &[Transition], level: i32) -> i32
{
    table.iter().rev().find(|t| level >= t.level).map_or(0, |t| t.value)
}

/// Wrapper for tcod::map::Map so that the Default trait can be implemented for it
struct FovWrapper
{
    fov: FovMap
}

impl FovWrapper
{
    fn new() -> Self
    {
        FovWrapper { fov: FovMap::new(MAP_WIDTH, MAP_HEIGHT) }
    }
}

impl Default for FovWrapper
{
    fn default() -> Self { FovWrapper::new() }
}