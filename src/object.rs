use tcod::colors::Color;

pub struct Object
{
    pub pos: (i32, i32),
    pub c: char,
    pub color: Color,
    pub name: String,
}

impl Object
{
    pub fn new(x: i32, y: i32, c: char, color: Color, name: &str) -> Self
    {
        Object
        {
            pos: (x, y),
            c: c,
            color: color,
            name: name.into(),
        }
    }
}