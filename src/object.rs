use crate::map::Map;
use tcod::colors::Color;
use tcod::console::{ Console, BackgroundFlag };
use std::cmp;

#[derive(Debug, Serialize, Deserialize)]
pub struct Object
{
    pub pos: (i32, i32),
    pub c: char,
    pub color: Color,
    pub name: String,
    pub alive: bool,
    pub solid: bool,
    pub always_visible: bool,
    pub level: i32,
}

impl Object
{
    pub fn new(x: i32, y: i32, c: char, color: Color, name: &str, solid: bool) -> Self
    {
        Object
        {
            pos: (x, y),
            c: c,
            color: color,
            name: name.into(),
            alive: false,
            solid: solid,
            always_visible: false,
            level: 1,
        }
    }

    pub fn draw(&self, con: &mut Console)
    {
        con.set_default_foreground(self.color);
        con.put_char(self.pos.0, self.pos.1, self.c, BackgroundFlag::None);
    }

    pub fn set_pos(&mut self, x: i32, y: i32)
    {
        self.pos = (x, y)
    }

    pub fn move_by(&mut self, dx: i32, dy: i32, map: &Map, objects: &[Object])
    {
        let (x, y) = self.pos;
        if !map.is_blocked(x + dx, y + dy, objects)
        {
            self.pos = (x + dx, y + dy);
        }
    }

    pub fn move_towards(&mut self, target_x: i32, target_y: i32, map: &Map, objects: &[Object])
    {
        let dx = target_x - self.pos.0;
        let dy = target_y - self.pos.1;
        let dist = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

        let dx = (dx as f32 / dist).round() as i32;
        let dy = (dy as f32 / dist).round() as i32;

        self.move_by(dx, dy, map, objects);
    }

    pub fn distance_to(&self, other: &Object) -> f32 
    {
        let dx = other.pos.0 - self.pos.0;
        let dy = other.pos.1 - self.pos.1;
        ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
    }

    pub fn distance(&self, x: i32, y: i32) -> f32 
    {
        (((x - self.pos.0).pow(2) + (y - self.pos.1).pow(2)) as f32).sqrt()
    }
}

/// Mutably borrow two *separate* elements from the given slice.
/// Panics when the indexes are equal or out of bounds.
fn mut_two<T>(first_index: usize, second_index: usize, items: &mut [T]) -> (&mut T, &mut T) 
{
    assert!(first_index != second_index);
    let split_at_index = cmp::max(first_index, second_index);
    let (first_slice, second_slice) = items.split_at_mut(split_at_index);
    if first_index < second_index 
    {
        (&mut first_slice[first_index], &mut second_slice[0])
    } 
    else 
    {
        (&mut second_slice[0], &mut first_slice[second_index])
    }
}