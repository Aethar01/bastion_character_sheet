#![windows_subsystem = "windows"]

mod app;
mod logic;
mod message;
mod model;
mod view;

use app::CharacterSheet;
use iced::Theme;

pub fn main() -> iced::Result {
    iced::application(CharacterSheet::new, CharacterSheet::update, view::view)
        .title("Bastion Character Sheet")
        .theme(theme)
        .run()
}

fn theme(state: &CharacterSheet) -> Theme {
    let mut palette = Theme::Dark.palette();
    let mut custom = false;

    let parse_hex = |hex: &str| -> Option<iced::Color> {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f32 / 255.0;
            Some(iced::Color::from_rgb(r, g, b))
        } else {
            None
        }
    };

    if let Some(c) = parse_hex(&state.character.background_color) {
        palette.background = c;
        custom = true;
    }
    if let Some(c) = parse_hex(&state.character.foreground_color) {
        palette.text = c;
        custom = true;
    }
    if let Some(c) = parse_hex(&state.character.accent_color) {
        palette.primary = c;
        custom = true;
    }

    if custom {
        Theme::custom("Custom Theme".to_string(), palette)
    } else {
        Theme::Dark
    }
}
