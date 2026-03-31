use crate::app::CharacterSheet;
use crate::logic;
use crate::message::{AttributeField, Message, OffsetField};
use crate::model::Origin;
use iced::widget::{
    button, checkbox, column, container, opaque, pick_list, row, scrollable, stack, text,
    text_editor, text_input, Space,
};
use iced::{alignment, Alignment, Color, Element, Length};

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

    if let Some(error) = &state.error_message {
        layers = layers.push(view_error_modal(error));
    } else if let Some(notification) = &state.notification {
        layers = layers.push(view_notification_modal(notification));
    } else if state.is_editing {
        layers = layers.push(view_editor(state));
    } else if state.show_ability_browser {
        layers = layers.push(view_ability_browser(state));
    }

    if state.show_save_menu {
        let save_as_menu = container(
            container(
                button(text("Save As").align_x(alignment::Horizontal::Center))
                    .on_press(Message::SaveAsCharacter)
                    .width(Length::Fill),
            )
            .style(container::bordered_box)
            .padding(5)
            .width(100.0),
        )
        .padding(iced::Padding {
            top: 75.0,
            right: 110.0,
            bottom: 0.0,
            left: 0.0,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Right)
        .align_y(alignment::Vertical::Top);

        layers = layers.push(save_as_menu);
    }

    layers.into()
}

fn view_error_modal(error: &str) -> Element<'_, Message> {
    let content = column![
        text("Unable to load character:").size(30),
        text(error).size(20),
        button("OK").on_press(Message::DismissError)
    ]
    .spacing(20)
    .padding(20)
    .align_x(alignment::Horizontal::Center);

    opaque(
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
        }),
    )
    .into()
}

fn view_notification_modal(message: &str) -> Element<'_, Message> {
    let content = column![
        text("Success").size(30),
        text(message).size(20),
        button("OK").on_press(Message::DismissNotification)
    ]
    .spacing(20)
    .padding(20)
    .align_x(alignment::Horizontal::Center);

    opaque(
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
        }),
    )
    .into()
}

fn view_header(state: &CharacterSheet) -> Element<'_, Message> {
    let save_group = row![
        button("Save").on_press(Message::SaveCharacter),
        button("▼").on_press(Message::ToggleSaveMenu),
    ]
    .spacing(2)
    .align_y(Alignment::Center);

    let left_group = row![
        text(&state.character.name).size(30).width(Length::Fill),
        text(format!("Lvl {}", state.character.level)).size(24),
        text(format!("{}", state.character.origin.to_string())).size(24),
        button("Edit Character").on_press(Message::ToggleEditor),
    ]
    .spacing(20)
    .align_y(Alignment::Center);

    row![
        left_group.width(Length::Fill),
        save_group,
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
            text("Armor)"),
            Space::new().width(10),
            text("DR:"),
            text_input("e.g. 5/holy", &state.dr_input)
                .on_input(Message::DrChanged)
                .width(100)
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

    let offset_row = |label: &'static str, val: &String, field: OffsetField| {
        row![
            text(label).width(150),
            text_input("+0", val)
                .on_input(move |s| Message::OffsetChanged(field, s))
                .width(60)
                .align_x(alignment::Horizontal::Center)
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
        row![
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
            column![
                text("Derived Stat Overrides").size(20),
                offset_row(
                    "Speed Offset",
                    &state.speed_offset_input,
                    OffsetField::Speed
                ),
                offset_row(
                    "Crit Range Offset",
                    &state.crit_range_offset_input,
                    OffsetField::CritRange
                ),
                offset_row(
                    "Max Spells Offset",
                    &state.max_spells_offset_input,
                    OffsetField::MaxSpells
                ),
                offset_row(
                    "Max Miracles Offset",
                    &state.max_miracles_offset_input,
                    OffsetField::MaxMiracles
                ),
                offset_row(
                    "Max Abilities Offset",
                    &state.max_abilities_offset_input,
                    OffsetField::MaxAbilities
                ),
                offset_row(
                    "Max Inventory Slots Offset",
                    &state.max_inventory_slots_offset_input,
                    OffsetField::MaxInventorySlots
                ),
                offset_row(
                    "Max HP Offset",
                    &state.max_hp_offset_input,
                    OffsetField::MaxHp
                ),
            ]
            .spacing(10),
            column![
                text("Theme Colors").size(20),
                row![
                    text("Background").width(100),
                    text_input("#hexcode", &state.bg_color_input)
                        .on_input(Message::BgColorChanged)
                        .width(100)
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                row![
                    text("Foreground").width(100),
                    text_input("#hexcode", &state.fg_color_input)
                        .on_input(Message::FgColorChanged)
                        .width(100)
                ]
                .spacing(10)
                .align_y(Alignment::Center),
                row![
                    text("Accent").width(100),
                    text_input("#hexcode", &state.accent_color_input)
                        .on_input(Message::AccentColorChanged)
                        .width(100)
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            ]
            .spacing(10)
        ]
        .spacing(40),
        button("Done").on_press(Message::ToggleEditor)
    ]
    .spacing(20)
    .padding(20);

    opaque(
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
        }),
    )
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

fn view_ability_browser(state: &CharacterSheet) -> Element<'_, Message> {
    let search_bar = text_input("Search abilities...", &state.ability_search_query)
        .on_input(Message::AbilityBrowserSearchChanged)
        .padding(10)
        .width(Length::Fill);

    let mut all_tags: std::collections::HashSet<String> = std::collections::HashSet::new();
    for a in &state.available_abilities {
        for s in a.tags.split(',') {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                all_tags.insert(trimmed.to_string());
            }
        }
    }

    let mut tags_row = row![].spacing(10);
    let mut sorted_tags: Vec<String> = all_tags.into_iter().collect();

    // Sort logic
    let tag_priority = |tag: &str| -> i32 {
        match tag {
            "Passive" => 1,
            "Maneuver" => 2,
            "Spell" => 3,
            "Miracle" => 4,
            "1 Action" => 5,
            "2 Actions" => 6,
            "3 Actions" => 7,
            "Instantaneous" => 8,
            "Short" => 9,
            "Long" => 10,
            "Punish" => 11,
            _ => 100,
        }
    };

    sorted_tags.sort_by(|a, b| {
        let p_a = tag_priority(a);
        let p_b = tag_priority(b);
        if p_a != p_b {
            p_a.cmp(&p_b)
        } else {
            a.cmp(b)
        }
    });

    for tag in sorted_tags {
        let state = state.ability_selected_tags.get(&tag);
        let btn = button(text(match state {
            Some(crate::model::TagFilterState::Include) => format!("✓ {}", tag),
            Some(crate::model::TagFilterState::Exclude) => format!("✗ {}", tag),
            None => tag.clone(),
        }));

        let styled_btn = match state {
            Some(crate::model::TagFilterState::Include) => {
                btn.style(|t, s| iced::widget::button::primary(t, s))
            }
            Some(crate::model::TagFilterState::Exclude) => {
                btn.style(|t, s| iced::widget::button::danger(t, s))
            }
            None => btn.style(|t, s| iced::widget::button::secondary(t, s)),
        };

        tags_row = tags_row.push(styled_btn.on_press(Message::AbilityBrowserTagToggled(tag)));
    }

    let mut list = column![].spacing(10);

    let query_lower = state.ability_search_query.to_lowercase();
    for ability in &state.available_abilities {
        if !query_lower.is_empty() {
            if !ability.name.to_lowercase().contains(&query_lower)
                && !ability.body.to_lowercase().contains(&query_lower)
                && !ability.desc.to_lowercase().contains(&query_lower)
                && !ability.tags.to_lowercase().contains(&query_lower)
            {
                continue;
            }
        }

        let ability_tags: Vec<String> = ability
            .tags
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        if !state.ability_selected_tags.is_empty() {
            let mut matches_all = true;
            for (req_tag, req_state) in &state.ability_selected_tags {
                let has_tag = ability_tags.contains(req_tag);
                match req_state {
                    crate::model::TagFilterState::Include => {
                        if !has_tag {
                            matches_all = false;
                            break;
                        }
                    }
                    crate::model::TagFilterState::Exclude => {
                        if has_tag {
                            matches_all = false;
                            break;
                        }
                    }
                }
            }
            if !matches_all {
                continue;
            }
        }

        let header = row![
            text(&ability.name).size(20).width(Length::Fill),
            text(&ability.tags).size(16),
            button("Import").on_press(Message::ImportAbility(ability.clone()))
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let body = text(&ability.body).size(16);
        let desc = text(&ability.desc)
            .size(14)
            .color(Color::from_rgb(0.7, 0.7, 0.7));

        list = list.push(
            container(column![header, body, desc].spacing(5))
                .style(container::bordered_box)
                .padding(10)
                .width(Length::Fill),
        );
    }

    let list_container = container(list).padding(iced::Padding {
        top: 0.0,
        right: 15.0,
        bottom: 0.0,
        left: 0.0,
    });
    let tags_container = container(tags_row).padding(iced::Padding {
        top: 0.0,
        right: 15.0,
        bottom: 15.0,
        left: 0.0,
    });

    let scrollable_list = scrollable(list_container).height(Length::Fill);
    let scrollable_tags = scrollable(tags_container).direction(
        iced::widget::scrollable::Direction::Horizontal(iced::widget::scrollable::Scrollbar::new()),
    );

    let content = column![
        row![
            text("Ability Browser").size(30).width(Length::Fill),
            button("Close").on_press(Message::ToggleAbilityBrowser)
        ]
        .align_y(Alignment::Center),
        search_bar,
        scrollable_tags,
        scrollable_list
    ]
    .spacing(20)
    .padding(20)
    .width(Length::Fill)
    .height(Length::Fill);

    opaque(
        container(
            container(content)
                .style(container::bordered_box)
                .width(800.0)
                .height(600.0)
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
        }),
    )
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
        if state.is_editing_abilities {
            let delete_btn = if state.deleting_ability_index == Some(i) {
                row![
                    text("Sure?"),
                    button("Yes").on_press(Message::ConfirmDeleteAbility),
                    button("No").on_press(Message::CancelDeleteAbility),
                ]
                .spacing(5)
                .align_y(Alignment::Center)
            } else {
                let up_btn = button("↑");
                let up_btn = if i > 0 {
                    up_btn.on_press(Message::MoveAbilityUp(i))
                } else {
                    up_btn
                };

                let down_btn = button("↓");
                let down_btn = if i < state.character.abilities.len() - 1 {
                    down_btn.on_press(Message::MoveAbilityDown(i))
                } else {
                    down_btn
                };

                row![
                    up_btn,
                    down_btn,
                    button("🗑").on_press(Message::RequestDeleteAbility(i))
                ]
                .spacing(5)
                .align_y(Alignment::Center)
            };

            let header_row = row![
                checkbox(ability.prepared).on_toggle(move |b| Message::ToggleAbilityPrepared(i, b)),
                text_input("Ability Name", &ability.name)
                    .on_input(move |s| Message::AbilityNameChanged(i, s))
                    .width(Length::FillPortion(2)),
                delete_btn
            ]
            .spacing(10)
            .align_y(Alignment::Center);

            let body_editor = state
                .ability_body_editors
                .get(i)
                .expect("Editor should exist for ability body");

            let desc_editor = state
                .ability_desc_editors
                .get(i)
                .expect("Editor should exist for ability desc");

            let details = column![
                text_input("Tags (e.g. 1 Action, Punish)", &ability.tags)
                    .on_input(move |s| Message::AbilityTagsChanged(i, s)),
                text_editor(body_editor)
                    .placeholder("Rules Text (Body)")
                    .height(Length::Shrink)
                    .on_action(move |a| Message::AbilityBodyChanged(i, a)),
                text_editor(desc_editor)
                    .placeholder("Flavour Text (Description)")
                    .height(Length::Shrink)
                    .on_action(move |a| Message::AbilityDescChanged(i, a)),
            ]
            .spacing(5);

            list = list.push(
                container(column![header_row, details].spacing(10))
                    .style(container::bordered_box)
                    .padding(10),
            );
        } else {
            let header_row = row![
                checkbox(ability.prepared).on_toggle(move |b| Message::ToggleAbilityPrepared(i, b)),
                text(&ability.name).size(20).width(Length::Fill),
                text(&ability.tags).size(16),
            ]
            .spacing(10)
            .align_y(Alignment::Center);

            let body = text(&ability.body).size(16);
            let desc = text(&ability.desc)
                .size(14)
                .color(Color::from_rgb(0.7, 0.7, 0.7));

            list = list.push(
                container(column![header_row, body, desc].spacing(5))
                    .style(container::bordered_box)
                    .padding(10)
                    .width(Length::Fill),
            );
        }
    }

    let mut header_controls = row![
        text("Abilities").size(24),
        text(format!("(Prepared: {}/{})", prepared_count, max_prepared)).size(20),
        Space::new().width(Length::Fill),
    ]
    .spacing(20)
    .align_y(Alignment::Center);

    if state.is_editing_abilities {
        header_controls = header_controls
            .push(button("Add Ability").on_press(Message::AddAbility))
            .push(button("Browse Abilities").on_press(Message::ToggleAbilityBrowser));
    }

    header_controls = header_controls.push(
        button(if state.is_editing_abilities {
            "Done Editing"
        } else {
            "Edit"
        })
        .on_press(Message::ToggleEditAbilities),
    );

    column![header_controls, list]
        .padding(10)
        .spacing(20)
        .width(Length::FillPortion(1))
        .into()
}
