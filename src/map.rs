use crate::game::{ Game, PLAYER_ID };
use crate::entity::Entity;
use rand::Rng;
use std::cmp;
use tcod::colors;
use tcod::map::Map as FovMap;

/// The width of the map in tiles
pub const MAP_WIDTH: i32 = 80;

/// The height of the map in tiles
pub const MAP_HEIGHT: i32 = 43;

/// The maximum number of rooms that can appear on the map
const MAX_ROOMS: i32 = 30;

/// The minimum size for a room is 6x6
const ROOM_MIN_SIZE: i32 = 6;

/// The maximum size for a room is 10x10
const ROOM_MAX_SIZE: i32 = 10;

/// A map is a 2D array of Tiles
pub type Map = Vec< Vec< Tile > >;

/// Creates the TCOD FOV map from the given map
pub fn create_fov_map(map: &Map, fov: &mut FovMap)
{
    for y in 0..MAP_HEIGHT
    {
        for x in 0..MAP_WIDTH
        {
            fov.set(x, y, !map[x as usize][y as usize].blocks_sight, !map[x as usize][y as usize].blocked);
        }
    }
}

/// Generates a new map
pub fn generate_map(entities: &mut Vec< Entity >, dungeon_level: i32) -> Map
{
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS
    {
        let room_w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let room_h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let room_x = rand::thread_rng().gen_range(0, MAP_WIDTH - room_w);
        let room_y = rand::thread_rng().gen_range(0, MAP_HEIGHT - room_h);
        let new_room = Rect::new(room_x, room_y, room_w, room_h);

        // We should only create the room if the new room doesn't intersect
        // with any existing rooms
        let can_create = !rooms.iter().any(|o| new_room.intersects_with(o));
        if can_create
        {
            generate_room(new_room, &mut map);
            place_room_monsters(new_room, &mut map, entities, dungeon_level);
            place_room_items(new_room, &mut map, entities, dungeon_level);

            let (c_x, c_y) = new_room.get_center();
            if rooms.is_empty()
            {
                // set player position at center of first room
                entities[PLAYER_ID].set_pos(c_x, c_y);
            }
            else
            {
                // Connect all rooms after the first with tunnels
                let (prev_x, prev_y) = rooms[rooms.len() - 1].get_center();
                if rand::random()
                {
                    generate_horizontal_tunnel(prev_x, c_x, prev_x, &mut map);
                    generate_vertical_tunnel(prev_y, c_y, c_x, &mut map);
                }
                else
                {
                    generate_vertical_tunnel(prev_y, c_y, prev_x, &mut map);
                    generate_horizontal_tunnel(prev_x, c_x, c_y, &mut map);
                }
            }

            rooms.push(new_room);
        }
    }

    // Create stairs at center of last room
    let (stair_x, stair_y) = rooms[rooms.len() - 1].get_center();
    let mut stairs = Entity::new(stair_x, stair_y, 'H', colors::WHITE, "Stairs", false);
    stairs.always_visible = true;
    entities.push(stairs);

    map
}

pub fn render_map(game: &mut Game)
{
}

/// Used to generate a single room in the map by carving out a rectangle of empty tiles
fn generate_room(room: Rect, map: &mut Map)
{
    for x in (room.x1 + 1)..room.x2
    {
        for y in (room.y1 + 1)..room.y2
        {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

/// Used to generate a horizontal tunnel in the map
fn generate_horizontal_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map)
{
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1)
    {
        map[x as usize][y as usize] = Tile::empty();
    }
}

/// Used to generate a vertical tunnel in the map
fn generate_vertical_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map)
{
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1)
    {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn place_room_monsters(room: Rect, map: &mut Map, entities: &mut Vec< Entity >, dungeon_level: i32)
{
}

fn place_room_items(room: Rect, map: &mut Map, entities: &mut Vec< Entity >, dungeon_level: i32)
{
}

/// Represents a single tile on the map and its properties
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tile
{
    pub blocked: bool,
    pub blocks_sight: bool,
    pub explored: bool
}

impl Tile
{
    fn empty() -> Self
    {
        Tile
        {
            blocked: false,
            blocks_sight: false,
            explored: false
        }
    }

    fn wall() -> Self
    {
        Tile
        {
            blocked: true,
            blocks_sight: true,
            explored: false
        }
    }
}

/// A room on the map is simply represented by a rectangle
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
    /// Create a new rectangle with given width and height at given position
    fn new(x: i32, y: i32, w: i32, h: i32) -> Self
    {
        Rect { x1: x, y1: y, x2: x + w, y2: y + h }
    }

    /// Returns the center point (x, y) of the rect
    fn get_center(&self) -> (i32, i32)
    {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;

        (center_x, center_y)
    }

    /// Returns true if this rect intersects another rect
    fn intersects_with(&self, other: &Rect) -> bool
    {
        (self.x1 <= other.x2) && (self.x2 >= other.x1) && (self.y1 <= other.y2) && (self.y2 >= other.y1)
    }
}

/// Used for increasing difficulty with dungeon level
struct Transition
{
    dungeon_level: i32,
    value: i32
}