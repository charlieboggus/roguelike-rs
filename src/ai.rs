use crate::game::{ Game, PLAYER_ID };
use crate::object;

/// Represents the different types of AI for monsters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Ai
{
    BasicMonster,
}

pub fn ai_basic_monster(id: usize, game: &mut Game) -> Ai
{
    let (mx, my) = game.objects[id].pos;
    if game.map.is_in_fov((mx, my))
    {
        if game.objects[id].distance_to(&game.objects[PLAYER_ID]) >= 2.0
        {
            object::move_towards(id, game.objects[PLAYER_ID].pos.0, game.objects[PLAYER_ID].pos.1, game);
        }
        else if game.objects[PLAYER_ID].fighter.map_or(false, |f| f.hp > 0)
        {
            // attack
        }
    }

    Ai::BasicMonster
}