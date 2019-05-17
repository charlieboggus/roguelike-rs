use crate::object::Object;
use crate::game::Game;

use tcod::colors;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Fighter
{
    pub base_vit: i32,  // Vitality
    pub base_atk: i32,  // Attack
    pub base_str: i32,  // Strength
    pub base_def: i32,  // Defense
    pub base_int: i32,  // Intelligence
    pub base_lck: i32,  // Luck

    pub max_hp: i32,
    pub hp: i32,
    pub xp: i32,

    pub on_death: DeathCallback
}

impl Fighter
{
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DeathCallback
{
    PlayerDeath,
    MonsterDeath
}

impl DeathCallback
{
    pub fn callback(self, object: &mut Object, game: &mut Game)
    {
        let callback: fn(&mut Object, &mut Game) = match self
        {
            DeathCallback::PlayerDeath => player_death_callback,
            DeathCallback::MonsterDeath => monster_death_callback
        };

        callback(object, game);
    }
}

fn player_death_callback(player: &mut Object, game: &mut Game)
{
    // Log
    player.c = '%';
    player.color = colors::DARK_RED;
}

fn monster_death_callback(monster: &mut Object, game: &mut Game)
{
    // log
    monster.c = '%';
    monster.color = colors::DARK_RED;
    monster.name = format!("Remains of {}", monster.name);
    monster.solid = false;
    monster.fighter = None;
    // monster.ai = None;
}