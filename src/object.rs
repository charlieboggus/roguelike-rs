use crate::game::Game;
use crate::fighter::Fighter;
use crate::ai::Ai;
use crate::item::{ Item, Equipment };
use crate::gui::MessageLog;

use tcod::colors::{ self, Color };
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
    pub fighter: Option< Fighter >,
    pub ai: Option< Ai >,
    pub item: Option< Item >,
    pub equipment: Option< Equipment >
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
            fighter: None,
            ai: None,
            item: None,
            equipment: None
        }
    }

    /// Draws this object to the given console
    pub fn draw(&self, con: &mut Console)
    {
        con.set_default_foreground(self.color);
        con.put_char(self.pos.0, self.pos.1, self.c, BackgroundFlag::None);
    }

    /// Sets the position of this object at the given (x, y) position
    pub fn set_pos(&mut self, x: i32, y: i32)
    {
        self.pos = (x, y)
    }

    /// Returns the distance that this object is to another given object
    pub fn distance_to(&self, other: &Object) -> f32 
    {
        let dx = other.pos.0 - self.pos.0;
        let dy = other.pos.1 - self.pos.1;
        ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
    }

    /// Returns the distance that this object is from a given (x, y) coordinate
    pub fn distance(&self, x: i32, y: i32) -> f32 
    {
        (((x - self.pos.0).pow(2) + (y - self.pos.1).pow(2)) as f32).sqrt()
    }

    /// Heal this object by some given amount
    pub fn heal(&mut self, amount: i32)
    {
        if let Some(fighter) = self.fighter.as_mut()
        {
            fighter.hp += amount;
            if fighter.hp > fighter.max_hp
            {
                fighter.hp = fighter.max_hp;
            }
        }
    }

    /// Function to make this object attack a different target object
    pub fn attack(&mut self, target: &mut Object, game: &mut Game)
    {
        // TODO: figure out accuracy formulas and shit

        // TODO: figure out damage formula
        let damage = 0;

        // If the rolled damage value is > 0 then the target takes damage
        if damage > 0
        {
            game.log.add(format!("{} attacks {} for {} damage", self.name, target.name, damage), colors::WHITE);
            if let Some(xp) = target.take_damage(damage, game)
            {
                self.fighter.as_mut().unwrap().xp += xp;
            }
        }
        else
        {
            game.log.add(format!("{} attacks {} but it has no effect!", self.name, target.name), colors::WHITE);
        }
    }

    /// Function to make this object take the given amount of damage
    pub fn take_damage(&mut self, amount: i32, game: &mut Game) -> Option< i32 >
    {
        if let Some(fighter) = self.fighter.as_mut()
        {
            if amount > 0
            {
                fighter.hp -= amount;
            }
        }

        if let Some(fighter) = self.fighter
        {
            if fighter.hp <= 0
            {
                self.alive = false;
                fighter.on_death.callback(self, game);
                return Some(fighter.xp);
            }
        }

        None
    }

    /// Function to equip this object
    pub fn equip(&mut self, log: &mut Vec< (String, Color) >)
    {
        if self.item.is_none()
        {
            log.add(format!("Cannot equip {:?} because it is not an item!", self), colors::RED);
            return
        }

        if let Some(ref mut equipment) = self.equipment
        {
            if !equipment.equipped
            {
                equipment.equipped = true;
                log.add(format!("Equipped {} on {:?}.", self.name, equipment.slot), colors::YELLOW);
            }
        }
        else
        {
            log.add(format!("Cannot equip {:?} because it is not equipment.", self), colors::RED);
        }
    }

    /// Function to unequip this object
    pub fn unequip(&mut self, log: &mut Vec< (String, Color) >)
    {
        if self.item.is_none()
        {
            log.add(format!("Cannot unequip {:?} because it is not an item!", self), colors::RED);
            return
        }

        if let Some(ref mut equipment) = self.equipment
        {
            if equipment.equipped
            {
                equipment.equipped = false;
                log.add(format!("Unequipped {} from {:?}.", self.name, equipment.slot), colors::YELLOW);
            }
        }
        else
        {
            log.add(format!("Cannot unequip {:?} because it is not equipment.", self), colors::RED);
        }
    }

    /// Function to get all items equipped to this object
    pub fn get_all_equipped(&self, game: &Game) -> Vec< Equipment >
    {
        game.inventory
            .iter()
            .filter(|item| { 
                item.equipment.map_or(false, |e| e.equipped) 
            })
            .map(|item| item.equipment.unwrap())
            .collect()
    }

    pub fn vitality_value(&self, game: &Game) -> i32
    {
        let base = self.fighter.map_or(0, |f| f.base_vit);
        let bonus = self.get_all_equipped(game).iter().fold(0, |sum, e| sum + e.vit_bonus);

        base + bonus
    }

    pub fn attack_value(&self, game: &Game) -> i32
    {
        let base = self.fighter.map_or(0, |f| f.base_atk);
        let bonus = self.get_all_equipped(game).iter().fold(0, |sum, e| sum + e.atk_bonus);

        base + bonus
    }

    pub fn strength_value(&self, game: &Game) -> i32
    {
        let base = self.fighter.map_or(0, |f| f.base_str);
        let bonus = self.get_all_equipped(game).iter().fold(0, |sum, e| sum + e.str_bonus);

        base + bonus
    }

    pub fn defense_value(&self, game: &Game) -> i32
    {
        let base = self.fighter.map_or(0, |f| f.base_def);
        let bonus = self.get_all_equipped(game).iter().fold(0, |sum, e| sum + e.def_bonus);

        base + bonus
    }

    pub fn intelligence_value(&self, game: &Game) -> i32
    {
        let base = self.fighter.map_or(0, |f| f.base_int);
        let bonus = self.get_all_equipped(game).iter().fold(0, |sum, e| sum + e.int_bonus);

        base + bonus
    }

    pub fn luck_value(&self, game: &Game) -> i32
    {
        let base = self.fighter.map_or(0, |f| f.base_lck);
        let bonus = self.get_all_equipped(game).iter().fold(0, |sum, e| sum + e.lck_bonus);

        base + bonus
    }
}

/// Function to move an object by the given delta X and delta Y
pub fn move_by(id: usize, dx: i32, dy: i32, game: &mut Game)
{
    let (x, y) = game.objects[id].pos;
    if !game.map.is_blocked(x + dx, y + dy, &game.objects)
    {
        game.objects[id].set_pos(x + dx, y + dy);
    }
}

/// Function to move an object towards the target (x, y) position
pub fn move_towards(id: usize, target_x: i32, target_y: i32, game: &mut Game)
{
    let dx = target_x - game.objects[id].pos.0;
    let dy = target_y - game.objects[id].pos.1;
    let dist = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();

    let dx = (dx as f32 / dist).round() as i32;
    let dy = (dy as f32 / dist).round() as i32;

    move_by(id, dx, dy, game);
}

/// Mutably borrow two *separate* elements from the given slice.
/// Panics when the indexes are equal or out of bounds.
pub fn mut_two< T >(first_index: usize, second_index: usize, items: &mut [T]) -> (&mut T, &mut T) 
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