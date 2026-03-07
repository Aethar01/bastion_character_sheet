use iced::{Theme, theme::Palette};
pub fn t() {
    let p = Theme::Dark.palette();
    let t = Theme::custom("custom".to_string(), p.clone());
}
