//! Food entity definitions.
//! Contains Food and BonusFood structs with constructors.

use rand::Rng;

#[allow(unused_imports)]
pub use shared::{BonusFood, Food};

/// Color palette for bonus food appearance.
pub const BONUS_COLORS: [&str; 6] = [
    "#ffd700", "#ffffff", "#ff69b4", "#00ff99", "#ff6600", "#99ff00",
];

/// Selects a bonus color not currently in use by any snake.
pub fn get_bonus_color(used_colors: &[String]) -> String {
    let available: Vec<&str> = BONUS_COLORS
        .iter()
        .copied()
        .filter(|c| !used_colors.contains(&c.to_string()))
        .collect();

    if available.is_empty() {
        BONUS_COLORS[rand::thread_rng().gen_range(0..BONUS_COLORS.len())].to_string()
    } else {
        available[rand::thread_rng().gen_range(0..available.len())].to_string()
    }
}
