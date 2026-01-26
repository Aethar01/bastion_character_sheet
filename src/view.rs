use crate::app::CharacterSheet;
use crate::logic;
use crate::message::{AttributeField, Message};
use crate::model::{AbilityType, Origin};
use iced::widget::{
    Space, button, checkbox, column, container, pick_list, row, scrollable, stack, text,
    text_editor, text_input,
};
use iced::{Alignment, Color, Element, Length, alignment};

pub fn view(state: &CharacterSheet) -> Element<'_, Message> {
    let content = scrollable(
        column![
            view_header(state),
            row![
                container(view_attributes(state)).width(Length::FillPortion(1)),
                container(view_vitals(state)).width(Length::FillPortion(1)),
                container(view_traits(state)).width(Length::FillPortion(1)),
            ]
            .spacing(20),
            row![view_inventory(state), view_abilities(state),].spacing(20),
        ]
        .spacing(20)
        .padding(20),
    );

    let mut layers = stack![container(content).width(Length::Fill).height(Length::Fill)];

    if state.is_editing {
        layers = layers.push(view_editor(state));
    }

    layers.into()
}

fn view_header(state: &CharacterSheet) -> Element<'_, Message> {
    row![
        text(&state.character.name).size(30).width(Length::Fill),
        text(format!("Lvl {}", state.character.level)).size(24),
        text(format!("{}", state.character.origin.to_string())).size(24),
        button("Edit Character").on_press(Message::ToggleEditor),
        button("Save").on_press(Message::SaveCharacter),
        button("Load").on_press(Message::LoadCharacter),
    ]
    .spacing(20)
    .padding(10)
    .align_y(Alignment::Center)
    .into()
}

fn view_attributes(state: &CharacterSheet) -> Element<'_, Message> {
    let attr_row = |label: &'static str, val: i32| {
        row![
            text(label).width(100),
            text(val.to_string())
                .size(20)
                .width(30)
                .align_x(alignment::Horizontal::Center),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
    };

    column![
        text("Attributes").size(24),
        attr_row("Strength", state.character.attributes.strength),
        attr_row("Dexterity", state.character.attributes.dexterity),
        attr_row("Endurance", state.character.attributes.endurance),
        attr_row("Faith", state.character.attributes.faith),
        attr_row("Will", state.character.attributes.will),
        attr_row("Intelligence", state.character.attributes.intelligence),
        attr_row("Luck", state.character.attributes.luck),
    ]
    .spacing(10)
    .padding(10)
    .into()
}

fn view_vitals(state: &CharacterSheet) -> Element<'_, Message> {
    let max_hp = logic::calculate_max_hp(&state.character);
    let speed = logic::calculate_movement_speed(&state.character);
    let ac = logic::calculate_armor_class(&state.character);
    let crit_start = logic::calculate_crit_range(&state.character);
    let spell_slots_max = logic::calculate_spell_slots(&state.character);
    let miracle_slots_max = logic::calculate_miracle_slots(&state.character);

    // Wounds Row
    let wounds_row = row![
        text("Wounds:"),
        row((1..=4).map(|i| {
            checkbox(state.character.wounds >= i)
                .on_toggle(move |checked| {
                    if checked {
                        Message::WoundsChanged(i)
                    } else {
                        Message::WoundsChanged(i - 1)
                    }
                })
                .into()
        }))
        .spacing(10)
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let hp_row = row![
        text("HP:").width(20),
        text_input("HP", &state.hp_input)
            .on_input(Message::HpInputChanged)
            .width(50)
            .align_x(alignment::Horizontal::Center),
        text(format!("/ {}", max_hp)).size(20).width(50),
        Space::new().width(10),
        button("-").on_press(Message::ApplyHpModifier(-1)),
        text_input("Mod", &state.hp_modifier)
            .on_input(Message::HpModifierChanged)
            .width(50)
            .align_x(alignment::Horizontal::Center),
        button("+").on_press(Message::ApplyHpModifier(1)),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let resource_ticker = |label: &'static str,
                           input_val: &String,
                           max: i32,
                           on_change: fn(String) -> Message,
                           on_adjust: fn(i32) -> Message| {
        row![
            text(label).width(80),
            button("-").on_press(on_adjust(-1)),
            text_input("0", input_val)
                .on_input(on_change)
                .width(40)
                .align_x(alignment::Horizontal::Center),
            button("+").on_press(on_adjust(1)),
            text(format!("/ {}", max)).size(20),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
    };

    column![
        text("Vitals").size(24),
        wounds_row,
        hp_row,
        row![text("Speed:"), text(speed.to_string()).size(20)]
            .spacing(10)
            .align_y(Alignment::Center),
        row![
            text("AC:"),
            text(ac.to_string()).size(20),
            text("(+"),
            text_input("0", &state.armor_bonus_input)
                .on_input(Message::ArmorBonusChanged)
                .width(40),
            text("Armor)")
        ]
        .spacing(5)
        .align_y(Alignment::Center),
        row![
            text("Crit Range:"),
            text(if crit_start == 12 {
                "12".to_string()
            } else {
                format!("{}-12", crit_start)
            })
            .size(20)
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        row![
            text("Tender:"),
            text_input("200", &state.tender_input)
                .on_input(Message::TenderChanged)
                .width(80)
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        Space::new().height(20),
        text("Resources").size(24),
        resource_ticker(
            "Spells:",
            &state.spells_input,
            spell_slots_max,
            Message::SpellsInputChanged,
            Message::AdjustSpells
        ),
        resource_ticker(
            "Miracles:",
            &state.miracles_input,
            miracle_slots_max,
            Message::MiraclesInputChanged,
            Message::AdjustMiracles
        ),
    ]
    .spacing(10)
    .padding(10)
    .into()
}

fn view_traits(state: &CharacterSheet) -> Element<'_, Message> {
    let origin_traits = logic::get_origin_traits(state.character.origin);
    let traits_col = column(
        origin_traits
            .iter()
            .map(|t| text(format!("• {}", t)).into())
            .collect::<Vec<_>>(),
    )
    .spacing(5);

    column![
        text(format!("{} Traits", state.character.origin.to_string())).size(24),
        traits_col
    ]
    .padding(10)
    .into()
}

fn view_editor(state: &CharacterSheet) -> Element<'_, Message> {
    let attr_row = |label: &'static str, val: i32, field: AttributeField| {
        row![
            text(label).width(100),
            button("-").on_press(Message::AttributeChanged(field, val - 1)),
            text(val.to_string())
                .size(20)
                .width(30)
                .align_x(alignment::Horizontal::Center),
            button("+").on_press(Message::AttributeChanged(field, val + 1)),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
    };

    let content = column![
        text("Edit Character").size(30),
        row![
            text("Name:"),
            text_input("Name", &state.character.name).on_input(Message::NameChanged)
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        row![
            text("Level:"),
            text_input("1", &state.level_input)
                .on_input(Message::LevelChanged)
                .width(50)
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        row![
            text("Origin:"),
            pick_list(
                Origin::all(),
                Some(state.character.origin),
                Message::OriginSelected
            )
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        column![
            text("Attributes").size(20),
            attr_row(
                "Strength",
                state.character.attributes.strength,
                AttributeField::Strength
            ),
            attr_row(
                "Dexterity",
                state.character.attributes.dexterity,
                AttributeField::Dexterity
            ),
            attr_row(
                "Endurance",
                state.character.attributes.endurance,
                AttributeField::Endurance
            ),
            attr_row(
                "Faith",
                state.character.attributes.faith,
                AttributeField::Faith
            ),
            attr_row(
                "Will",
                state.character.attributes.will,
                AttributeField::Will
            ),
            attr_row(
                "Intelligence",
                state.character.attributes.intelligence,
                AttributeField::Intelligence
            ),
            attr_row(
                "Luck",
                state.character.attributes.luck,
                AttributeField::Luck
            ),
        ]
        .spacing(10),
        button("Done").on_press(Message::ToggleEditor)
    ]
    .spacing(20)
    .padding(20);

    container(
        container(content)
            .style(container::bordered_box)
            .padding(20),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center)
    .style(|_| container::Style {
        background: Some(
            Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.8,
            }
            .into(),
        ),
        ..Default::default()
    })
    .into()
}

fn view_inventory(state: &CharacterSheet) -> Element<'_, Message> {
    let total_slots = logic::calculate_carrying_slots(&state.character);

    let display_count = total_slots.max(state.character.inventory.len() as i32) as usize;

    let mut slots_col = column![].spacing(5);

    for i in 0..display_count {
        let editor_content = state
            .inventory_editors
            .get(i)
            .expect("Inventory editor should exist");
        let label = if i < total_slots as usize {
            format!("{}.", i + 1)
        } else {
            format!("Over limit {}.", i + 1)
        };

        slots_col = slots_col.push(
            row![
                text(label).width(80),
                text_editor(editor_content)
                    .placeholder("Empty slot...")
                    .height(Length::Shrink)
                    .on_action(move |a| Message::InventoryAction(i, a))
            ]
            .align_y(Alignment::Center),
        );
    }

    column![
        text(format!(
            "Inventory ({}/{})",
            state
                .character
                .inventory
                .iter()
                .filter(|s| !s.is_empty())
                .count(),
            total_slots
        ))
        .size(24),
        slots_col
    ]
    .padding(10)
    .spacing(10)
    .width(Length::FillPortion(1))
    .into()
}

fn view_abilities(state: &CharacterSheet) -> Element<'_, Message> {
    let mut list = column![].spacing(20);

    let prepared_count = state
        .character
        .abilities
        .iter()
        .filter(|a| a.prepared)
        .count();
    let max_prepared = logic::calculate_prepared_slots(&state.character);

    for (i, ability) in state.character.abilities.iter().enumerate() {
        let delete_btn = if state.deleting_ability_index == Some(i) {
            row![
                text("Sure?"),
                button("Yes").on_press(Message::ConfirmDeleteAbility),
                button("No").on_press(Message::CancelDeleteAbility),
            ]
            .spacing(5)
            .align_y(Alignment::Center)
        } else {
            row![button("🗑").on_press(Message::RequestDeleteAbility(i))].align_y(Alignment::Center)
        };

        let header_row = row![
            checkbox(ability.prepared).on_toggle(move |b| Message::ToggleAbilityPrepared(i, b)),
            text_input("Ability Name", &ability.name)
                .on_input(move |s| Message::AbilityNameChanged(i, s))
                .width(Length::FillPortion(2)),
            pick_list(
                AbilityType::all(),
                Some(ability.ability_type.clone()),
                move |t| Message::AbilityTypeChanged(i, t)
            )
            .width(Length::FillPortion(1)),
            delete_btn
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let editor_content = state
            .ability_editors
            .get(i)
            .expect("Editor should exist for ability");

        let details = column![
            text_input("Tags (e.g. 1 Action, Punish)", &ability.tags)
                .on_input(move |s| Message::AbilityTagsChanged(i, s)),
            text_editor(editor_content)
                .placeholder("Description")
                .height(Length::Shrink)
                .on_action(move |a| Message::AbilityDescChanged(i, a)),
        ]
        .spacing(5);

        list = list.push(
            container(column![header_row, details].spacing(10))
                .style(container::bordered_box)
                .padding(10),
        );
    }

    column![
        row![
            text("Abilities").size(24),
            text(format!("(Prepared: {}/{})", prepared_count, max_prepared)).size(20),
            button("Add Ability").on_press(Message::AddAbility)
        ]
        .spacing(20)
        .align_y(Alignment::Center),
        list
    ]
    .padding(10)
    .spacing(20)
    .width(Length::FillPortion(1))
    .into()
}
