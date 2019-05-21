use crate::TCOD;
use crate::game::{ Game, PLAYER_ID };
use crate::object::Object;
use crate::gui::MessageLog;

use tcod::colors;

pub const HEALTH_POTION_HEAL_AMT: i32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Item
{
    HealthPotion,
    Sword,
    Shield,
    PlateArmor
}

enum ItemUseResult
{
    Used,
    UsedAndKept,
    Cancelled
}

pub fn use_item(inv_id: usize, game: &mut Game, tcod: &mut TCOD)
{
    if let Some(item) = game.inventory[inv_id].item
    {
        let on_use = match item
        {
            Item::HealthPotion => use_health_potion,
            Item::Sword => toggle_equipment,
            Item::Shield => toggle_equipment,
            Item::PlateArmor => toggle_equipment
        };

        match on_use(inv_id, game, tcod)
        {
            ItemUseResult::Used         => { game.inventory.remove(inv_id); },
            ItemUseResult::UsedAndKept  => {  },
            ItemUseResult::Cancelled    => { game.log.add("Cancelled.", colors::WHITE); }
        }
    }
    else
    {
        game.log.add(format!("The {} cannot be used.", game.inventory[inv_id].name), colors::WHITE);
    }
}

pub fn pick_item_up(id: usize, game: &mut Game)
{
    if game.inventory.len() >= 26
    {
        game.log.add(format!("Your inventory is full! Cannot pick up {}!", game.objects[id].name), colors::RED);
    }
    else
    {
        let item = game.objects.swap_remove(id);
        game.log.add(format!("You picked up {}!", item.name), colors::GREEN);
        let index = game.inventory.len();
        let slot = item.equipment.map(|e| e.slot);
        game.inventory.push(item);

        if let Some(slot) = slot
        {
            if get_equipped_in_slot(slot, &game.inventory).is_none()
            {
                game.inventory[index].equip(&mut game.log);
            }
        }
    }
}

pub fn drop_item(inv_id: usize, game: &mut Game)
{
    let mut item = game.inventory.remove(inv_id);
    if item.equipment.is_some()
    {
        item.unequip(&mut game.log);
    }

    item.set_pos(game.objects[PLAYER_ID].pos.0, game.objects[PLAYER_ID].pos.1);
    game.log.add(format!("You dropped {}!", item.name), colors::YELLOW);
    game.objects.push(item);
}

fn use_health_potion(_inv_id: usize, game: &mut Game, _tcod: &mut TCOD) -> ItemUseResult
{
    if let Some(fighter) = game.objects[PLAYER_ID].fighter
    {
        if fighter.hp == fighter.max_hp
        {
            game.log.add("You are already at full health.", colors::WHITE);
            return ItemUseResult::Cancelled;
        }

        game.log.add("Your wounds start to feel better.", colors::LIGHT_VIOLET);
        game.objects[PLAYER_ID].heal(HEALTH_POTION_HEAL_AMT);
        return ItemUseResult::Used;
    }

    ItemUseResult::Cancelled
}

fn toggle_equipment(inv_id: usize, game: &mut Game, _tcod: &mut TCOD) -> ItemUseResult
{
    let equipment = match game.inventory[inv_id].equipment
    {
        Some(equipment) => equipment,
        None => return ItemUseResult::Cancelled
    };

    if let Some(old) = get_equipped_in_slot(equipment.slot, &game.inventory)
    {
        game.inventory[old].unequip(&mut game.log);
    }

    if equipment.equipped
    {
        game.inventory[inv_id].unequip(&mut game.log);
    }
    else
    {
        game.inventory[inv_id].equip(&mut game.log);
    }

    ItemUseResult::UsedAndKept
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Equipment
{
    pub slot: EquipmentSlot,
    pub equipped: bool,
    pub vit_bonus: i32,
    pub atk_bonus: i32,
    pub str_bonus: i32,
    pub def_bonus: i32,
    pub dex_bonus: i32,
    pub int_bonus: i32,
    pub lck_bonus: i32
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EquipmentSlot
{
    Head,
    Torso,
    Legs,
    Feet,
    LeftHand,
    RightHand
}

impl std::fmt::Display for EquipmentSlot
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        match *self
        {
            EquipmentSlot::Head         => write!(f, "{}", "Head"),
            EquipmentSlot::Torso        => write!(f, "{}", "Torso"),
            EquipmentSlot::Legs         => write!(f, "{}", "Legs"),
            EquipmentSlot::Feet         => write!(f, "{}", "Feet"),
            EquipmentSlot::LeftHand     => write!(f, "{}", "Left Hand"),
            EquipmentSlot::RightHand    => write!(f, "{}", "Right Hand")
        }
    }
}

fn get_equipped_in_slot(slot: EquipmentSlot, inventory: &[Object]) -> Option< usize >
{
    for (inv_id, item) in inventory.iter().enumerate()
    {
        if item.equipment.as_ref().map_or(false, |e| e.equipped && e.slot == slot)
        {
            return Some(inv_id);
        }
    }

    None
}