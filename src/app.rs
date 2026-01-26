use crate::logic;
use crate::message::{AttributeField, Message};
use crate::model::{Ability, AbilityType, Character, Origin};
use iced::Task;
use rfd::AsyncFileDialog;
use std::fs;

pub struct CharacterSheet {
    pub character: Character,
    pub is_editing: bool,
    
    pub hp_input: String,
    pub hp_modifier: String,
    pub spells_input: String,
    pub miracles_input: String,
    pub level_input: String,
    pub tender_input: String,
    pub armor_bonus_input: String,
}

impl Default for CharacterSheet {
    fn default() -> Self {
        let character = Character::default();
        let hp = character.current_hp.to_string();
        let spells = logic::calculate_spell_slots(&character).saturating_sub(character.expended_spell_slots).to_string();
        let miracles = logic::calculate_miracle_slots(&character).saturating_sub(character.expended_miracle_slots).to_string();
        let level = character.level.to_string();
        let tender = character.tender.to_string();
        let armor_bonus = character.armor_bonus.to_string();

        Self {
            character,
            is_editing: false,
            hp_input: hp,
            hp_modifier: "1".to_string(),
            spells_input: spells,
            miracles_input: miracles,
            level_input: level,
            tender_input: tender,
            armor_bonus_input: armor_bonus,
        }
    }
}

impl CharacterSheet {
    pub fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleEditor => self.is_editing = !self.is_editing,
            Message::NameChanged(name) => self.character.name = name,
            Message::LevelChanged(lvl) => {
                self.level_input = lvl;
                if let Ok(val) = self.level_input.parse::<i32>() {
                    self.character.level = val.clamp(1, 10);
                }
            }
            Message::TenderChanged(val) => {
                self.tender_input = val;
                if let Ok(num) = self.tender_input.parse::<i32>() {
                    self.character.tender = num;
                }
            }
             Message::ArmorBonusChanged(val) => {
                self.armor_bonus_input = val;
                if let Ok(num) = self.armor_bonus_input.parse::<i32>() {
                    self.character.armor_bonus = num;
                }
            }
            
            Message::HpInputChanged(val) => {
                self.hp_input = val;
                if let Ok(num) = self.hp_input.parse::<i32>() {
                    let max = logic::calculate_max_hp(&self.character);
                    self.character.current_hp = num.clamp(0, max);
                }
            }
            Message::HpModifierChanged(val) => {
                self.hp_modifier = val;
            }
            Message::ApplyHpModifier(sign) => {
                if let Ok(mod_val) = self.hp_modifier.parse::<i32>() {
                    let max = logic::calculate_max_hp(&self.character);
                    let current = self.character.current_hp;
                    let new_val = (current + (mod_val * sign)).clamp(0, max);
                    self.character.current_hp = new_val;
                    self.hp_input = new_val.to_string();
                }
            }

            Message::SpellsInputChanged(val) => {
                self.spells_input = val;
                if let Ok(avail) = self.spells_input.parse::<i32>() {
                    let max = logic::calculate_spell_slots(&self.character);
                    let expended = max.saturating_sub(avail).max(0); 
                    self.character.expended_spell_slots = expended;
                }
            }
            Message::MiraclesInputChanged(val) => {
                self.miracles_input = val;
                if let Ok(avail) = self.miracles_input.parse::<i32>() {
                    let max = logic::calculate_miracle_slots(&self.character);
                    let expended = max.saturating_sub(avail).max(0);
                    self.character.expended_miracle_slots = expended;
                }
            }
            Message::AdjustSpells(delta) => {
                let max = logic::calculate_spell_slots(&self.character);
                let current_avail = max.saturating_sub(self.character.expended_spell_slots);
                let new_avail = (current_avail + delta).clamp(0, max);
                self.character.expended_spell_slots = max - new_avail;
                self.spells_input = new_avail.to_string();
            }
            Message::AdjustMiracles(delta) => {
                let max = logic::calculate_miracle_slots(&self.character);
                let current_avail = max.saturating_sub(self.character.expended_miracle_slots);
                let new_avail = (current_avail + delta).clamp(0, max);
                self.character.expended_miracle_slots = max - new_avail;
                self.miracles_input = new_avail.to_string();
            }

            Message::OriginSelected(origin) => {
                self.character.origin = origin;
                if origin == Origin::Human && self.character.attributes.luck < 3 {
                     self.character.attributes.luck = 3;
                }
            }
            Message::AttributeChanged(field, val) => {
                let max = (self.character.level + 3).min(10);
                let new_val = val.clamp(1, max);
                match field {
                    AttributeField::Strength => self.character.attributes.strength = new_val,
                    AttributeField::Dexterity => self.character.attributes.dexterity = new_val,
                    AttributeField::Endurance => self.character.attributes.endurance = new_val,
                    AttributeField::Faith => self.character.attributes.faith = new_val,
                    AttributeField::Will => self.character.attributes.will = new_val,
                    AttributeField::Intelligence => self.character.attributes.intelligence = new_val,
                    AttributeField::Luck => self.character.attributes.luck = new_val,
                }
            }
            Message::SaveCharacter => {
                return Task::perform(async {
                    let file = AsyncFileDialog::new()
                        .add_filter("json", &["json"])
                        .set_file_name("character.json")
                        .save_file()
                        .await;
                    file.map(|f| f.path().to_owned())
                }, Message::SaveFileSelected);
            }
            Message::LoadCharacter => {
                return Task::perform(async {
                    let file = AsyncFileDialog::new()
                        .add_filter("json", &["json"])
                        .pick_file()
                        .await;
                    file.map(|f| f.path().to_owned())
                }, Message::LoadFileSelected);
            }
            Message::SaveFileSelected(path_opt) => {
                if let Some(path) = path_opt {
                    if let Ok(json) = serde_json::to_string_pretty(&self.character) {
                        let _ = fs::write(path, json);
                    }
                }
            }
            Message::LoadFileSelected(path_opt) => {
                if let Some(path) = path_opt {
                    if let Ok(content) = fs::read_to_string(path) {
                        if let Ok(char) = serde_json::from_str(&content) {
                            self.character = char;
                            self.hp_input = self.character.current_hp.to_string();
                            let s_max = logic::calculate_spell_slots(&self.character);
                            self.spells_input = s_max.saturating_sub(self.character.expended_spell_slots).to_string();
                            let m_max = logic::calculate_miracle_slots(&self.character);
                            self.miracles_input = m_max.saturating_sub(self.character.expended_miracle_slots).to_string();
                            self.level_input = self.character.level.to_string();
                            self.tender_input = self.character.tender.to_string();
                            self.armor_bonus_input = self.character.armor_bonus.to_string();
                        }
                    }
                }
            }
            Message::InventorySlotChanged(idx, text) => {
                if idx < self.character.inventory.len() {
                    self.character.inventory[idx] = text;
                } else {
                    while self.character.inventory.len() <= idx {
                        self.character.inventory.push(String::new());
                    }
                    self.character.inventory[idx] = text;
                }
            }
            Message::AddAbility => {
                self.character.abilities.push(Ability {
                    name: "New Ability".to_string(),
                    ability_type: AbilityType::Passive,
                    tags: String::new(),
                    description: String::new(),
                });
            }
            Message::RemoveAbility(idx) => {
                if idx < self.character.abilities.len() {
                    self.character.abilities.remove(idx);
                }
            }
            Message::AbilityNameChanged(idx, val) => {
                if let Some(ab) = self.character.abilities.get_mut(idx) {
                    ab.name = val;
                }
            }
            Message::AbilityTypeChanged(idx, val) => {
                if let Some(ab) = self.character.abilities.get_mut(idx) {
                    ab.ability_type = val;
                }
            }
            Message::AbilityTagsChanged(idx, val) => {
                if let Some(ab) = self.character.abilities.get_mut(idx) {
                    ab.tags = val;
                }
            }
            Message::AbilityDescChanged(idx, val) => {
                if let Some(ab) = self.character.abilities.get_mut(idx) {
                    ab.description = val;
                }
            }
        }
        Task::none()
    }
}
