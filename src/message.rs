use crate::model::{AbilityType, Origin};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    NameChanged(String),
    LevelChanged(String), 
    OriginSelected(Origin),
    AttributeChanged(AttributeField, i32),
    TenderChanged(String),
    ArmorBonusChanged(String),
    
    HpInputChanged(String),
    HpModifierChanged(String),
    ApplyHpModifier(i32),

    SpellsInputChanged(String),
    MiraclesInputChanged(String),
    AdjustSpells(i32),
    AdjustMiracles(i32),

    SaveCharacter,
    LoadCharacter,
    SaveFileSelected(Option<PathBuf>),
    LoadFileSelected(Option<PathBuf>),
    InventorySlotChanged(usize, String),
    AddAbility,
    RemoveAbility(usize),
    AbilityNameChanged(usize, String),
    AbilityTypeChanged(usize, AbilityType),
    AbilityTagsChanged(usize, String),
    AbilityDescChanged(usize, String),
    ToggleEditor,
}

#[derive(Debug, Clone, Copy)]
pub enum AttributeField {
    Strength,
    Dexterity,
    Endurance,
    Faith,
    Will,
    Intelligence,
    Luck,
}
