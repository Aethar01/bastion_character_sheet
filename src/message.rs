use crate::model::{AbilityType, Origin};
use iced::widget::text_editor;
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

    WoundsChanged(i32),

    SpellsInputChanged(String),
    MiraclesInputChanged(String),
    AdjustSpells(i32),
    AdjustMiracles(i32),

    SaveCharacter,
    LoadCharacter,
    SaveFileSelected(Option<PathBuf>),
    LoadFileSelected(Option<PathBuf>),
    InventoryAction(usize, text_editor::Action),
    AddAbility,
    RequestDeleteAbility(usize),
    ConfirmDeleteAbility,
    CancelDeleteAbility,
    AbilityNameChanged(usize, String),
    AbilityTypeChanged(usize, AbilityType),
    AbilityTagsChanged(usize, String),
    AbilityDescChanged(usize, text_editor::Action),
    ToggleAbilityPrepared(usize, bool),
    ToggleEditor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeField {
    Strength,
    Dexterity,
    Endurance,
    Faith,
    Will,
    Intelligence,
    Luck,
}
