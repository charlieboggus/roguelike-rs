use tcod::console::{ Console, BackgroundFlag };
use tcod::colors::{ self, Color };

// Entity modules


/// An entity is essentially any game object. Player, monsters, items, etc.
/// Every entity has a position, color, glyph, name, and other generic fields.
#[derive(Debug, Serialize, Deserialize)]
pub struct Entity
{
    pub pos: (i32, i32),
    pub ch: char,
    pub color: Color,
    pub name: String,
    pub level: i32,
    pub solid: bool,
    pub always_visible: bool,
    pub alive: bool,

}

impl Entity
{
    /// Creates a new Entity with given position, glyph, color, and name
    pub fn new(x: i32, y: i32, ch: char, color: Color, name: &str, solid: bool) -> Self
    {
        Entity
        {
            pos: (x, y),
            ch: ch,
            color: color,
            name: name.into(),
            level: 1,
            solid: solid,
            always_visible: false,
            alive: false,
        }
    }

    /// Draws the entity to the given console
    pub fn draw(&self, con: &mut Console)
    {
        con.set_default_foreground(self.color);
        con.put_char(self.pos.0, self.pos.1, self.ch, BackgroundFlag::None);
    }

    /// Set the position of the entity to the given x, y position
    pub fn set_pos(&mut self, x: i32, y: i32)
    {
        self.pos = (x, y);
    }

    /// Returns the distance this entity is to some position
    pub fn distance(&self, x: i32, y: i32) -> f32 
    {
        (((x - self.pos.0).pow(2) + (y - self.pos.1).pow(2)) as f32).sqrt()
    }

    /// Returns the distance this entity is to another entity
    pub fn distance_to(&self, other: &Entity) -> f32 
    {
        let dx = other.pos.0 - self.pos.0;
        let dy = other.pos.1 - self.pos.1;
        ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
    }
}

pub fn move_by(id: usize, dx: i32, dy: i32)
{
}

pub fn move_towards(id: usize, target_x: i32, target_y: i32)
{
}