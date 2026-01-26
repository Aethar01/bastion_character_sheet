mod logic;
mod model;

use iced::widget::{button, column, container, pick_list, row, text, text_input, scrollable, Space, stack};
use iced::{Element, Length, Task, Theme, Color};
use model::{Character, Origin, Ability, AbilityType};
use std::fs;
use std::path::PathBuf;
use rfd::AsyncFileDialog;

pub fn main() -> iced::Result {
    iced::application(CharacterSheet::new, CharacterSheet::update, CharacterSheet::view)
        .title("Bastion Character Sheet")
        .theme(theme)
        .run()
}

fn theme(_: &CharacterSheet) -> Theme {
    Theme::Dark
}

#[derive(Debug, Clone)]
enum Message {
    NameChanged(String),
    LevelChanged(String), 
    OriginSelected(Origin),
    AttributeChanged(AttributeField, i32),
    TenderChanged(String),
    ArmorBonusChanged(String),
    
    // HP Logic
    HpInputChanged(String),
    HpModifierChanged(String),
    ApplyHpModifier(i32), // +1 or -1 multiplier

    // Resource Logic
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
enum AttributeField {
    Strength,
    Dexterity,
    Endurance,
    Faith,
    Will,
    Intelligence,
    Luck,
}

struct CharacterSheet {
    character: Character,
    is_editing: bool,
    
    // UI State for inputs
    hp_input: String,
    hp_modifier: String,
    spells_input: String,
    miracles_input: String,
    level_input: String,
    tender_input: String,
    armor_bonus_input: String,
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
    fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
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
            
            // HP Logic
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

            // Resource Logic
            Message::SpellsInputChanged(val) => {
                self.spells_input = val;
                if let Ok(avail) = self.spells_input.parse::<i32>() {
                    let max = logic::calculate_spell_slots(&self.character);
                    // Expended = Max - Available
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
                            // Sync UI State
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

    fn view(&self) -> Element<'_, Message> {
        let content = scrollable(
            column![
                self.view_header(),
                row![
                    container(self.view_attributes()).width(Length::FillPortion(1)),
                    container(self.view_vitals()).width(Length::FillPortion(1)),
                    container(self.view_traits()).width(Length::FillPortion(1)),
                ]
                .spacing(20),
                row![
                    self.view_inventory(),
                    self.view_abilities(),
                ]
                .spacing(20),
            ]
            .spacing(20)
            .padding(20),
        );

        let mut layers = stack![container(content).width(Length::Fill).height(Length::Fill)];

        if self.is_editing {
            layers = layers.push(self.view_editor());
        }

        layers.into()
    }

    fn view_header(&self) -> Element<'_, Message> {
        row![
            text(&self.character.name).size(30).width(Length::Fill),
            text(format!("Lvl {}", self.character.level)).size(24),
            text(format!("{}", self.character.origin.to_string())).size(24),
            button("Edit Character").on_press(Message::ToggleEditor),
            button("Save").on_press(Message::SaveCharacter),
            button("Load").on_press(Message::LoadCharacter),
        ]
        .spacing(20)
        .padding(10)
        .align_y(iced::Alignment::Center)
        .into()
    }

    fn view_attributes(&self) -> Element<'_, Message> {
        let attr_row = |label: &'static str, val: i32| {
            row![
                text(label).width(100),
                text(val.to_string()).size(20).width(30).align_x(iced::alignment::Horizontal::Center),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center)
        };

        column![
            text("Attributes").size(24),
            attr_row("Strength", self.character.attributes.strength),
            attr_row("Dexterity", self.character.attributes.dexterity),
            attr_row("Endurance", self.character.attributes.endurance),
            attr_row("Faith", self.character.attributes.faith),
            attr_row("Will", self.character.attributes.will),
            attr_row("Intelligence", self.character.attributes.intelligence),
            attr_row("Luck", self.character.attributes.luck),
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn view_vitals(&self) -> Element<'_, Message> {
        let max_hp = logic::calculate_max_hp(&self.character);
        let speed = logic::calculate_movement_speed(&self.character);
        let ac = logic::calculate_armor_class(&self.character);
        let spell_slots_max = logic::calculate_spell_slots(&self.character);
        let miracle_slots_max = logic::calculate_miracle_slots(&self.character);

        // HP Row
        let hp_row = row![
            text("HP:").width(20),
            text_input("HP", &self.hp_input)
                .on_input(Message::HpInputChanged)
                .width(50)
                .align_x(iced::alignment::Horizontal::Center),
            text(format!("/ {}", max_hp)).size(20).width(50),
            
            // Modifier Controls
            Space::new().width(10),
            button("-").on_press(Message::ApplyHpModifier(-1)),
            text_input("Mod", &self.hp_modifier)
                .on_input(Message::HpModifierChanged)
                .width(50)
                .align_x(iced::alignment::Horizontal::Center),
            button("+").on_press(Message::ApplyHpModifier(1)),
        ].spacing(10).align_y(iced::Alignment::Center);

        // Resource Tickers
        let resource_ticker = |label: &'static str, input_val: &String, max: i32, on_change: fn(String) -> Message, on_adjust: fn(i32) -> Message| {
             row![
                text(label).width(80),
                button("-").on_press(on_adjust(-1)),
                text_input("0", input_val)
                    .on_input(on_change)
                    .width(40)
                    .align_x(iced::alignment::Horizontal::Center),
                button("+").on_press(on_adjust(1)),
                text(format!("/ {}", max)).size(20),
            ].spacing(10).align_y(iced::Alignment::Center)
        };

        column![
            text("Vitals").size(24),
            hp_row,
            row![text("Speed:"), text(speed.to_string()).size(20)].spacing(10).align_y(iced::Alignment::Center),
            row![text("AC:"), text(ac.to_string()).size(20), text("(+"), text_input("0", &self.armor_bonus_input).on_input(Message::ArmorBonusChanged).width(40), text("Armor)")].spacing(5).align_y(iced::Alignment::Center),
            row![text("Tender:"), text_input("200", &self.tender_input).on_input(Message::TenderChanged).width(80)].spacing(10).align_y(iced::Alignment::Center),
            Space::new().height(20),
            text("Resources").size(24),
            resource_ticker("Spells:", &self.spells_input, spell_slots_max, Message::SpellsInputChanged, Message::AdjustSpells),
            resource_ticker("Miracles:", &self.miracles_input, miracle_slots_max, Message::MiraclesInputChanged, Message::AdjustMiracles),
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn view_traits(&self) -> Element<'_, Message> {
        let origin_traits = logic::get_origin_traits(self.character.origin);
        let traits_col = column(
            origin_traits.iter().map(|t| text(format!("• {}", t)).into()).collect::<Vec<_>>()
        )
        .spacing(5);

        column![
            text(format!("{} Traits", self.character.origin.to_string())).size(24),
            traits_col
        ]
        .padding(10)
        .into()
    }

    fn view_editor(&self) -> Element<'_, Message> {
        let attr_row = |label: &'static str, val: i32, field: AttributeField| {
            row![
                text(label).width(100),
                button("-").on_press(Message::AttributeChanged(field, val - 1)),
                text(val.to_string()).size(20).width(30).align_x(iced::alignment::Horizontal::Center),
                button("+").on_press(Message::AttributeChanged(field, val + 1)),
            ]
            .spacing(10)
            .align_y(iced::Alignment::Center)
        };

        let content = column![
            text("Edit Character").size(30),
            row![
                text("Name:"),
                text_input("Name", &self.character.name).on_input(Message::NameChanged)
            ].spacing(10).align_y(iced::Alignment::Center),
            row![
                text("Level:"),
                text_input("1", &self.level_input).on_input(Message::LevelChanged).width(50)
            ].spacing(10).align_y(iced::Alignment::Center),
            row![
                text("Origin:"),
                pick_list(Origin::all(), Some(self.character.origin), Message::OriginSelected)
            ].spacing(10).align_y(iced::Alignment::Center),
            column![
                text("Attributes").size(20),
                attr_row("Strength", self.character.attributes.strength, AttributeField::Strength),
                attr_row("Dexterity", self.character.attributes.dexterity, AttributeField::Dexterity),
                attr_row("Endurance", self.character.attributes.endurance, AttributeField::Endurance),
                attr_row("Faith", self.character.attributes.faith, AttributeField::Faith),
                attr_row("Will", self.character.attributes.will, AttributeField::Will),
                attr_row("Intelligence", self.character.attributes.intelligence, AttributeField::Intelligence),
                attr_row("Luck", self.character.attributes.luck, AttributeField::Luck),
            ].spacing(10),
            button("Done").on_press(Message::ToggleEditor)
        ]
        .spacing(20)
        .padding(20);

        container(
            container(content)
                .style(container::bordered_box)
                .padding(20)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .style(|_| container::Style {
            background: Some(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.8 }.into()),
            ..Default::default()
        })
        .into()
    }

    fn view_inventory(&self) -> Element<'_, Message> {
        let total_slots = logic::calculate_carrying_slots(&self.character);
        
        let display_count = total_slots.max(self.character.inventory.len() as i32) as usize;

        let mut slots_col = column![].spacing(5);
        
        for i in 0..display_count {
            let val = self.character.inventory.get(i).cloned().unwrap_or_default();
            let label = if i < total_slots as usize {
                format!("{}.", i + 1)
            } else {
                format!("Over limit {}.", i + 1)
            };
            
            slots_col = slots_col.push(
                row![
                    text(label).width(80),
                    text_input("Empty slot...", &val)
                        .on_input(move |s| Message::InventorySlotChanged(i, s))
                ].align_y(iced::Alignment::Center)
            );
        }

        column![
            text(format!("Inventory ({}/{})", self.character.inventory.iter().filter(|s| !s.is_empty()).count(), total_slots)).size(24),
            slots_col
        ].padding(10).spacing(10).width(Length::FillPortion(1)).into()
    }

    fn view_abilities(&self) -> Element<'_, Message> {
        let mut list = column![].spacing(20);

        for (i, ability) in self.character.abilities.iter().enumerate() {
            let header_row = row![
                text_input("Ability Name", &ability.name)
                    .on_input(move |s| Message::AbilityNameChanged(i, s))
                    .width(Length::FillPortion(2)),
                pick_list(
                    AbilityType::all(),
                    Some(ability.ability_type.clone()),
                    move |t| Message::AbilityTypeChanged(i, t)
                ).width(Length::FillPortion(1)),
                button("Remove").on_press(Message::RemoveAbility(i))
            ].spacing(10);

            let details = column![
                text_input("Tags (e.g. 1 Action, Punish)", &ability.tags)
                    .on_input(move |s| Message::AbilityTagsChanged(i, s)),
                text_input("Description", &ability.description)
                    .on_input(move |s| Message::AbilityDescChanged(i, s)),
            ].spacing(5);

            list = list.push(
                container(
                    column![header_row, details].spacing(10)
                ).style(container::bordered_box).padding(10)
            );
        }

        column![
            row![
                text("Abilities").size(24),
                button("Add Ability").on_press(Message::AddAbility)
            ].spacing(20),
            list
        ].padding(10).spacing(20).width(Length::FillPortion(1)).into()
    }
}
