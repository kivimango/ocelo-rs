use ratatui::style::{Style, Stylize};

mod cpu_details;
mod menu;
mod overview;
pub use self::cpu_details::*;
pub use self::menu::*;
pub use self::overview::*;

pub fn get_color_for(percentage: f64) -> Style {
    match percentage {
        0.0..24.99 => Style::default().light_green(),
        25.0..49.99 => Style::default().green(),
        50.0..74.99 => Style::default().yellow(),
        75.0..100.0 => Style::default().light_red(),
        _ => Style::reset(),
    }
}
