use crate::logic;
use crate::message::{AttributeField, Message};
use crate::model::{Ability, AbilityType, Character, Origin};
use iced::Task;
use iced::widget::text_editor;
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
    pub ability_editors: Vec<text_editor::Content>,
    pub inventory_editors: Vec<text_editor::Content>,
    pub deleting_ability_index: Option<usize>,
}

impl Default for CharacterSheet {
    fn default() -> Self {
        let character = Character::default();
        let hp = character.current_hp.to_string();
        let spells = logic::calculate_spell_slots(&character)
            .saturating_sub(character.expended_spell_slots)
            .to_string();
        let miracles = logic::calculate_miracle_slots(&character)
            .saturating_sub(character.expended_miracle_slots)
            .to_string();
        let level = character.level.to_string();
        let tender = character.tender.to_string();
        let armor_bonus = character.armor_bonus.to_string();
        let ability_editors = character
            .abilities
            .iter()
            .map(|a| text_editor::Content::with_text(&a.description))
            .collect();
        let inventory_editors = character
            .inventory
            .iter()
            .map(|i| text_editor::Content::with_text(i))
            .collect();

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
            ability_editors,
            inventory_editors,
            deleting_ability_index: None,
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

            Message::WoundsChanged(val) => {
                self.character.wounds = val.clamp(0, 4);
                let max = logic::calculate_max_hp(&self.character);
                if self.character.current_hp > max {
                    self.character.current_hp = max;
                    self.hp_input = max.to_string();
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
                self.sync_inventory_editors();
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
                    AttributeField::Intelligence => {
                        self.character.attributes.intelligence = new_val
                    }
                    AttributeField::Luck => self.character.attributes.luck = new_val,
                }
                if field == AttributeField::Strength {
                    self.sync_inventory_editors();
                }
            }
            Message::SaveCharacter => {
                let default_name = format!("{}.json", self.character.name);
                return Task::perform(
                    async move {
                        let file = AsyncFileDialog::new()
                            .add_filter("json", &["json"])
                            .set_file_name(&default_name)
                            .save_file()
                            .await;
                        file.map(|f| f.path().to_owned())
                    },
                    Message::SaveFileSelected,
                );
            }
            Message::LoadCharacter => {
                return Task::perform(
                    async {
                        let file = AsyncFileDialog::new()
                            .add_filter("json", &["json"])
                            .pick_file()
                            .await;
                        file.map(|f| f.path().to_owned())
                    },
                    Message::LoadFileSelected,
                );
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
                            self.spells_input = s_max
                                .saturating_sub(self.character.expended_spell_slots)
                                .to_string();
                            let m_max = logic::calculate_miracle_slots(&self.character);
                            self.miracles_input = m_max
                                .saturating_sub(self.character.expended_miracle_slots)
                                .to_string();
                            self.level_input = self.character.level.to_string();
                            self.tender_input = self.character.tender.to_string();
                            self.armor_bonus_input = self.character.armor_bonus.to_string();
                            self.ability_editors = self
                                .character
                                .abilities
                                .iter()
                                .map(|a| text_editor::Content::with_text(&a.description))
                                .collect();
                            self.inventory_editors = self
                                .character
                                .inventory
                                .iter()
                                .map(|i| text_editor::Content::with_text(i))
                                .collect();
                        }
                    }
                }
            }
            Message::InventoryAction(idx, action) => {
                while self.inventory_editors.len() <= idx {
                    self.inventory_editors.push(text_editor::Content::new());
                }
                if let Some(editor) = self.inventory_editors.get_mut(idx) {
                    editor.perform(action);
                    let text = editor.text();
                    if idx < self.character.inventory.len() {
                        self.character.inventory[idx] = text;
                    } else {
                        while self.character.inventory.len() <= idx {
                            self.character.inventory.push(String::new());
                        }
                        self.character.inventory[idx] = text;
                    }
                }
            }
            Message::AddAbility => {
                self.character.abilities.push(Ability {
                    name: "New Ability".to_string(),
                    ability_type: AbilityType::Passive,
                    tags: String::new(),
                    description: String::new(),
                    prepared: false,
                });
                self.ability_editors.push(text_editor::Content::new());
            }
            Message::RequestDeleteAbility(idx) => {
                self.deleting_ability_index = Some(idx);
            }
            Message::ConfirmDeleteAbility => {
                if let Some(idx) = self.deleting_ability_index {
                    if idx < self.character.abilities.len() {
                        self.character.abilities.remove(idx);
                    }
                    if idx < self.ability_editors.len() {
                        self.ability_editors.remove(idx);
                    }
                }
                self.deleting_ability_index = None;
            }
            Message::CancelDeleteAbility => {
                self.deleting_ability_index = None;
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
            Message::AbilityDescChanged(idx, action) => {
                if let Some(editor) = self.ability_editors.get_mut(idx) {
                    editor.perform(action);
                    if let Some(ab) = self.character.abilities.get_mut(idx) {
                        ab.description = editor.text();
                    }
                }
            }
            Message::ToggleAbilityPrepared(idx, val) => {
                if let Some(ab) = self.character.abilities.get_mut(idx) {
                    ab.prepared = val;
                }
            }
        }
        Task::none()
    }

    fn sync_inventory_editors(&mut self) {
        let total_slots = logic::calculate_carrying_slots(&self.character);
        let display_count = total_slots.max(self.character.inventory.len() as i32) as usize;

        while self.inventory_editors.len() < display_count {
            let text = self
                .character
                .inventory
                .get(self.inventory_editors.len())
                .cloned()
                .unwrap_or_default();
            self.inventory_editors
                .push(text_editor::Content::with_text(&text));
        }
    }
}
