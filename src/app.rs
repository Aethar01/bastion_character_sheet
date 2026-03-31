use crate::logic;
use crate::message::{AttributeField, Message};
use crate::model::{Ability, Character, Origin};
use iced::Task;
use iced::widget::text_editor;
use rfd::AsyncFileDialog;
use std::fs;
use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct AppConfig {
    last_file_path: Option<PathBuf>,
}

fn get_config_path() -> PathBuf {
    if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "BastionCharacterSheet") {
        let dir = proj_dirs.state_dir().unwrap_or_else(|| proj_dirs.data_local_dir());
        std::fs::create_dir_all(dir).ok();
        let mut path = dir.to_path_buf();
        path.push("config.json");
        path
    } else {
        PathBuf::from("bastion_sheet_config.json")
    }
}

fn load_config() -> AppConfig {
    if let Ok(content) = fs::read_to_string(get_config_path()) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

fn save_config(config: &AppConfig) {
    if let Ok(content) = serde_json::to_string_pretty(config) {
        let _ = fs::write(get_config_path(), content);
    }
}

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
    pub dr_input: String,
    pub bg_color_input: String,
    pub fg_color_input: String,
    pub accent_color_input: String,
    pub max_hp_offset_input: String,
    pub speed_offset_input: String,
    pub max_inventory_slots_offset_input: String,
    pub max_abilities_offset_input: String,
    pub max_spells_offset_input: String,
    pub max_miracles_offset_input: String,
    pub crit_range_offset_input: String,
    pub ability_body_editors: Vec<text_editor::Content>,
    pub ability_desc_editors: Vec<text_editor::Content>,
    pub inventory_editors: Vec<text_editor::Content>,
    pub deleting_ability_index: Option<usize>,
    pub error_message: Option<String>,
    pub notification: Option<String>,
    pub current_file_path: Option<std::path::PathBuf>,
    pub show_save_menu: bool,
    pub show_ability_browser: bool,
    pub is_editing_abilities: bool,
    pub available_abilities: Vec<Ability>,
    pub ability_search_query: String,
    pub ability_selected_tags: std::collections::HashMap<String, crate::model::TagFilterState>,
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
        let dr_input = character.dr.clone();
        let bg_color_input = character.background_color.clone();
        let fg_color_input = character.foreground_color.clone();
        let accent_color_input = character.accent_color.clone();
        let max_hp_offset = character.max_hp_offset.to_string();
        let speed_offset = character.speed_offset.to_string();
        let max_inventory_slots_offset = character.max_inventory_slots_offset.to_string();
        let max_abilities_offset = character.max_abilities_offset.to_string();
        let max_spells_offset = character.max_spells_offset.to_string();
        let max_miracles_offset = character.max_miracles_offset.to_string();
        let crit_range_offset = character.crit_range_offset.to_string();
        let ability_body_editors = character
            .abilities
            .iter()
            .map(|a| text_editor::Content::with_text(&a.body))
            .collect();
        let ability_desc_editors = character
            .abilities
            .iter()
            .map(|a| text_editor::Content::with_text(&a.desc))
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
            dr_input,
            bg_color_input,
            fg_color_input,
            accent_color_input,
            max_hp_offset_input: max_hp_offset,
            speed_offset_input: speed_offset,
            max_inventory_slots_offset_input: max_inventory_slots_offset,
            max_abilities_offset_input: max_abilities_offset,
            max_spells_offset_input: max_spells_offset,
            max_miracles_offset_input: max_miracles_offset,
            crit_range_offset_input: crit_range_offset,
            ability_body_editors,
            ability_desc_editors,
            inventory_editors,
            deleting_ability_index: None,
            error_message: None,
            notification: None,
            current_file_path: None,
            show_save_menu: false,
            show_ability_browser: false,
            is_editing_abilities: false,
            available_abilities: Vec::new(),
            ability_search_query: String::new(),
            ability_selected_tags: std::collections::HashMap::new(),
        }
    }
}

async fn load_abilities_task() -> Vec<Ability> {
    let mut abilities = Vec::new();
    let path = std::env::current_dir().unwrap_or_default().join("abilities");
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("bastion") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    abilities.extend(crate::parser::parse_bastion_abilities(&content));
                }
            }
        }
    }
    let config_dir_path = get_config_path().parent().unwrap_or(&std::path::PathBuf::from("")).join("abilities");
    if let Ok(entries) = std::fs::read_dir(config_dir_path) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("bastion") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    abilities.extend(crate::parser::parse_bastion_abilities(&content));
                }
            }
        }
    }
    abilities.sort_by(|a, b| a.name.cmp(&b.name));
    abilities
}

impl CharacterSheet {
    pub fn new() -> (Self, Task<Message>) {
        let mut sheet = Self::default();
        sheet.sync_inventory_editors();

        let config = load_config();
        
        let load_abs_task = Task::perform(load_abilities_task(), Message::AbilitiesLoaded);

        if let Some(path) = config.last_file_path {
            return (
                sheet,
                Task::batch(vec![
                    Task::perform(async { Some(path) }, Message::LoadFileSelected),
                    load_abs_task,
                ]),
            );
        }

        (sheet, load_abs_task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleEditor => self.is_editing = !self.is_editing,
            Message::ToggleSaveMenu => self.show_save_menu = !self.show_save_menu,
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
            Message::DrChanged(val) => {
                self.dr_input = val.clone();
                self.character.dr = val;
            }
            Message::BgColorChanged(val) => {
                self.bg_color_input = val.clone();
                self.character.background_color = val;
            }
            Message::FgColorChanged(val) => {
                self.fg_color_input = val.clone();
                self.character.foreground_color = val;
            }
            Message::AccentColorChanged(val) => {
                self.accent_color_input = val.clone();
                self.character.accent_color = val;
            }
            Message::OffsetChanged(field, val) => {
                let parsed = if val.trim().is_empty() || val.trim() == "+" {
                    Ok(0)
                } else if val.trim() == "-" {
                    // Allow typing negative numbers
                    Err(())
                } else {
                    val.parse::<i32>().map_err(|_| ())
                };

                match field {
                    crate::message::OffsetField::MaxHp => {
                        self.max_hp_offset_input = val;
                        if let Ok(num) = parsed {
                            self.character.max_hp_offset = num;
                        }
                    }
                    crate::message::OffsetField::Speed => {
                        self.speed_offset_input = val;
                        if let Ok(num) = parsed {
                            self.character.speed_offset = num;
                        }
                    }
                    crate::message::OffsetField::MaxInventorySlots => {
                        self.max_inventory_slots_offset_input = val;
                        if let Ok(num) = parsed {
                            self.character.max_inventory_slots_offset = num;
                            self.sync_inventory_editors();
                        }
                    }
                    crate::message::OffsetField::MaxAbilities => {
                        self.max_abilities_offset_input = val;
                        if let Ok(num) = parsed {
                            self.character.max_abilities_offset = num;
                        }
                    }
                    crate::message::OffsetField::MaxSpells => {
                        self.max_spells_offset_input = val;
                        if let Ok(num) = parsed {
                            self.character.max_spells_offset = num;
                        }
                    }
                    crate::message::OffsetField::MaxMiracles => {
                        self.max_miracles_offset_input = val;
                        if let Ok(num) = parsed {
                            self.character.max_miracles_offset = num;
                        }
                    }
                    crate::message::OffsetField::CritRange => {
                        self.crit_range_offset_input = val;
                        if let Ok(num) = parsed {
                            self.character.crit_range_offset = num;
                        }
                    }
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
                self.show_save_menu = false;
                if let Some(path) = &self.current_file_path {
                    if let Ok(json) = serde_json::to_string_pretty(&self.character) {
                        if let Err(e) = fs::write(path, json) {
                            self.error_message = Some(format!("Could not save file: {}", e));
                        } else {
                            let config = AppConfig {
                                last_file_path: Some(path.clone()),
                            };
                            save_config(&config);
                            self.notification = Some(format!(
                                "Character successfully saved to {:?}",
                                path.file_name().unwrap_or_default()
                            ));
                        }
                    }
                } else {
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
            }
            Message::SaveAsCharacter => {
                self.show_save_menu = false;
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
                        if let Err(e) = fs::write(&path, json) {
                            self.error_message = Some(format!("Could not save file: {}", e));
                        } else {
                            self.current_file_path = Some(path.clone());
                            let config = AppConfig {
                                last_file_path: Some(path.clone()),
                            };
                            save_config(&config);
                            self.notification = Some(format!(
                                "Character successfully saved to {:?}",
                                path.file_name().unwrap_or_default()
                            ));
                        }
                    }
                }
            }
            Message::LoadFileSelected(path_opt) => {
                if let Some(path) = path_opt {
                    match fs::read_to_string(&path) {
                        Ok(content) => match serde_json::from_str::<Character>(&content) {
                            Ok(char) => {
                                self.character = char;
                                self.current_file_path = Some(path.clone());
                                let config = AppConfig {
                                    last_file_path: Some(path.clone()),
                                };
                                save_config(&config);
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
                                self.dr_input = self.character.dr.clone();
                                self.bg_color_input = self.character.background_color.clone();
                                self.fg_color_input = self.character.foreground_color.clone();
                                self.accent_color_input = self.character.accent_color.clone();
                                self.max_hp_offset_input = self.character.max_hp_offset.to_string();
                                self.speed_offset_input = self.character.speed_offset.to_string();
                                self.max_inventory_slots_offset_input =
                                    self.character.max_inventory_slots_offset.to_string();
                                self.max_abilities_offset_input =
                                    self.character.max_abilities_offset.to_string();
                                self.max_spells_offset_input =
                                    self.character.max_spells_offset.to_string();
                                self.max_miracles_offset_input =
                                    self.character.max_miracles_offset.to_string();
                                self.crit_range_offset_input =
                                    self.character.crit_range_offset.to_string();
                                self.ability_body_editors = self
                                    .character
                                    .abilities
                                    .iter()
                                    .map(|a| text_editor::Content::with_text(&a.body))
                                    .collect();
                                self.ability_desc_editors = self
                                    .character
                                    .abilities
                                    .iter()
                                    .map(|a| text_editor::Content::with_text(&a.desc))
                                    .collect();
                                self.inventory_editors = self
                                    .character
                                    .inventory
                                    .iter()
                                    .map(|i| text_editor::Content::with_text(i))
                                    .collect();
                                self.sync_inventory_editors();
                            }
                            Err(e) => {
                                self.error_message =
                                    Some(format!("Invalid character format: {}", e));
                            }
                        },
                        Err(e) => {
                            self.error_message = Some(format!("Could not read file: {}", e));
                        }
                    }
                }
            }
            Message::DismissError => {
                self.error_message = None;
            }
            Message::DismissNotification => {
                self.notification = None;
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
                    tags: String::new(),
                    body: String::new(),
                    desc: String::new(),
                    prepared: false,
                });
                self.ability_body_editors.push(text_editor::Content::new());
                self.ability_desc_editors.push(text_editor::Content::new());
            }
            Message::RequestDeleteAbility(idx) => {
                self.deleting_ability_index = Some(idx);
            }
            Message::ConfirmDeleteAbility => {
                if let Some(idx) = self.deleting_ability_index {
                    if idx < self.character.abilities.len() {
                        self.character.abilities.remove(idx);
                    }
                    if idx < self.ability_body_editors.len() {
                        self.ability_body_editors.remove(idx);
                        self.ability_desc_editors.remove(idx);
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
            Message::AbilityTagsChanged(idx, val) => {
                if let Some(ab) = self.character.abilities.get_mut(idx) {
                    ab.tags = val;
                }
            }
            Message::AbilityBodyChanged(idx, action) => {
                if let Some(editor) = self.ability_body_editors.get_mut(idx) {
                    editor.perform(action);
                    if let Some(ab) = self.character.abilities.get_mut(idx) {
                        ab.body = editor.text();
                    }
                }
            }
            Message::AbilityDescChanged(idx, action) => {
                if let Some(editor) = self.ability_desc_editors.get_mut(idx) {
                    editor.perform(action);
                    if let Some(ab) = self.character.abilities.get_mut(idx) {
                        ab.desc = editor.text();
                    }
                }
            }
            Message::ToggleAbilityPrepared(idx, val) => {
                if let Some(ab) = self.character.abilities.get_mut(idx) {
                    ab.prepared = val;
                }
            }
            Message::MoveAbilityUp(idx) => {
                if idx > 0 && idx < self.character.abilities.len() {
                    self.character.abilities.swap(idx, idx - 1);
                    self.ability_body_editors.swap(idx, idx - 1);
                    self.ability_desc_editors.swap(idx, idx - 1);

                    if let Some(del_idx) = self.deleting_ability_index {
                        if del_idx == idx {
                            self.deleting_ability_index = Some(idx - 1);
                        } else if del_idx == idx - 1 {
                            self.deleting_ability_index = Some(idx);
                        }
                    }
                }
            }
            Message::MoveAbilityDown(idx) => {
                if idx < self.character.abilities.len() - 1 {
                    self.character.abilities.swap(idx, idx + 1);
                    self.ability_body_editors.swap(idx, idx + 1);
                    self.ability_desc_editors.swap(idx, idx + 1);

                    if let Some(del_idx) = self.deleting_ability_index {
                        if del_idx == idx {
                            self.deleting_ability_index = Some(idx + 1);
                        } else if del_idx == idx + 1 {
                            self.deleting_ability_index = Some(idx);
                        }
                    }
                }
            }
            Message::ToggleAbilityBrowser => {
                self.show_ability_browser = !self.show_ability_browser;
            }
            Message::ToggleEditAbilities => {
                self.is_editing_abilities = !self.is_editing_abilities;
            }
            Message::AbilityBrowserSearchChanged(query) => {
                self.ability_search_query = query;
            }
            Message::AbilityBrowserTagToggled(tag) => {
                match self.ability_selected_tags.get(&tag) {
                    None => {
                        self.ability_selected_tags.insert(tag, crate::model::TagFilterState::Include);
                    }
                    Some(crate::model::TagFilterState::Include) => {
                        self.ability_selected_tags.insert(tag, crate::model::TagFilterState::Exclude);
                    }
                    Some(crate::model::TagFilterState::Exclude) => {
                        self.ability_selected_tags.remove(&tag);
                    }
                }
            }
            Message::ImportAbility(ability) => {
                let body = ability.body.clone();
                let desc = ability.desc.clone();
                self.character.abilities.push(ability);
                self.ability_body_editors.push(text_editor::Content::with_text(&body));
                self.ability_desc_editors.push(text_editor::Content::with_text(&desc));
                self.show_ability_browser = false;
            }
            Message::AbilitiesLoaded(loaded) => {
                self.available_abilities = loaded;
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
