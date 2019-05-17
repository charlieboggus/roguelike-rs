use crate::object::Object;
use crate::game::PLAYER_ID;

use tcod::colors::{ self, Color };
use tcod::console::{ Console, BackgroundFlag };
use tcod::map::{ Map as FovMap, FovAlgorithm };
use rand::Rng;
use std::cmp;

pub const MAP_WIDTH: i32 = 80;
pub const MAP_HEIGHT: i32 = 40;

const ROOM_MIN_SIZE: i32 = 6;
const ROOM_MAX_SIZE: i32 = 10;
const MAX_ROOM_COUNT: i32 = 30;

#[derive(Serialize, Deserialize)]
pub struct Map
{
    pub tiles: Vec< Vec< Tile > >,
    pub width: i32,
    pub height: i32,

    #[serde(skip)]
    fov_wrapper: FovWrapper
}

impl Map
{
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

    pub fn draw(&mut self, con: &mut Console)
    {
        for y in 0..MAP_HEIGHT
        {
            for x in 0..MAP_WIDTH
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

    pub fn generate_fov_map(&mut self)
    {
        for y in 0..MAP_HEIGHT
        {
            for x in 0..MAP_WIDTH
            {
                self.fov_wrapper.fov.set(x, y, !self.tiles[x as usize][y as usize].blocks_sight, !self.tiles[x as usize][y as usize].blocked);
            }
        }
    }

    pub fn recompute_fov(&mut self, pos: (i32, i32))
    {
        self.fov_wrapper.fov.compute_fov(pos.0, pos.1, 10, true, FovAlgorithm::Basic);
    }

    pub fn is_in_fov(&self, pos: (i32, i32)) -> bool
    {
        self.fov_wrapper.fov.is_in_fov(pos.0, pos.1)
    }

    pub fn is_blocked(&self, x: i32, y: i32, objects: &[Object]) -> bool
    {
        if self.tiles[x as usize][y as usize].blocked
        {
            return true;
        }
        objects.iter().any(|o| { o.solid && o.pos.0 == x && o.pos.1 == y })
    }

    pub fn is_explored(&self, pos: (i32, i32)) -> bool
    {
        self.tiles[pos.0 as usize][pos.1 as usize].explored
    }

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

    fn generate_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32)
    {
        for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1)
        {
            self.tiles[x as usize][y as usize] = Tile::empty();
        }
    }

    fn generate_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32)
    {
        for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1)
        {
            self.tiles[x as usize][y as usize] = Tile::empty();
        }
    }

    fn populate_room(&mut self, room: &Rect, objects: &mut Vec< Object >, dungeon_level: i32)
    {
        // TODO: this
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tile
{
    pub blocked: bool,
    pub blocks_sight: bool,
    pub explored: bool,
}

impl Tile
{
    fn empty() -> Self
    {
        Tile
        {
            blocked: false,
            blocks_sight: false,
            explored: false,
        }
    }

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

    fn get_center(&self) -> (i32, i32)
    {
        let center_x = (self.x1 + self.x2) / 2;
        let center_y = (self.y1 + self.y2) / 2;

        (center_x, center_y)
    }

    fn intersects_with(&self, other: &Rect) -> bool
    {
        (self.x1 <= other.x2) && (self.x2 >= other.x1) && (self.y1 <= other.y2) && (self.y2 >= other.y1)
    }
}

/// Wrapper for tcod::map::Map so that I can implement Default trait for it...
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