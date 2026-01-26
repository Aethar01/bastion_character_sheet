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

fn theme(_: &CharacterSheet) -> Theme {
    Theme::Dark
}