use crate::object::Object;
use crate::gui::{ Messages, MessageLog};

use tcod::colors;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Fighter
{
    pub base_vit: i32,  // Vitality - hitpoints
    pub base_atk: i32,  // Attack - stat for attack accuracy
    pub base_str: i32,  // Strength - stat for attack damage
    pub base_def: i32,  // Defense - damage reduction
    pub base_dex: i32,  // Dexterity - stat for attack dodge
    pub base_int: i32,  // Intelligence - magic damage 
    pub base_lck: i32,  // Luck - stat for luck; affects stuff like drops/items

    pub max_hp: i32,
    pub hp: i32,
    pub xp: i32,

    pub on_death: DeathCallback
}

impl Fighter
{
    pub fn new(vit: i32, atk: i32, strn: i32, def: i32, dex: i32, int: i32, lck: i32, xp: i32, on_death: DeathCallback) -> Self
    {
        // Formula to determine HP: Max HP = 10 + (5 * vitality)
        let max_hp = 10 + (5 * vit);
        
        Fighter
        {
            base_vit: vit,
            base_atk: atk,
            base_str: strn,
            base_def: def,
            base_dex: dex,
            base_int: int,
            base_lck: lck,

            max_hp: max_hp,
            hp: max_hp,
            xp: xp,

            on_death: on_death
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DeathCallback
{
    PlayerDeath,
    MonsterDeath
}

impl DeathCallback
{
    pub fn callback(self, object: &mut Object, log: &mut Messages)
    {
        let callback: fn(&mut Object, &mut Messages) = match self
        {
            DeathCallback::PlayerDeath => player_death_callback,
            DeathCallback::MonsterDeath => monster_death_callback
        };

        callback(object, log);
    }
}

fn player_death_callback(player: &mut Object, log: &mut Messages)
{
    log.add(format!("{} has died!", player.name), colors::RED);
    player.c = '%';
    player.color = colors::DARK_RED;
}

fn monster_death_callback(monster: &mut Object, log: &mut Messages)
{
    log.add(format!("The {} has died! You gain {} experience points.", monster.name, monster.fighter.unwrap().xp), colors::PINK);
    monster.c = '%';
    monster.color = colors::DARK_RED;
    monster.name = format!("Remains of {}", monster.name);
    monster.solid = false;
    monster.fighter = None;
    monster.ai = None;
}