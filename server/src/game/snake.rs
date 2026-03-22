//! Snake entity definitions and name generation.
//! Contains Snake struct with constructors and helper functions.

use rand::Rng;

#[allow(unused_imports)]
pub use shared::{ActiveEffects, Direction, Point, Snake};

/// Neon color palette for snake appearance.
pub const NEON_COLORS: [&str; 10] = [
    "#00ff88", "#ff00ff", "#00ffff", "#ffff00", "#ff8800", "#88ff00", "#ff0088", "#00ffaa",
    "#ff4444", "#44ff44",
];

/// Prefixes for random snake name generation.
pub const PREFIXES: [&str; 10] = [
    "GHOST", "CIPHER", "NEON", "VECTOR", "PIXEL", "GLITCH", "BYTE", "NEXUS", "PROXY", "SYNC",
];

/// Suffixes for random snake name generation.
pub const SUFFIXES: [&str; 10] = ["42", "7X", "99", "3K", "77", "AA", "ZZ", "Q", "XX", "01"];

/// Generates a random cyberpunk-style snake name.
pub fn random_name() -> String {
    let mut rng = rand::thread_rng();
    let p = PREFIXES[rng.gen_range(0..PREFIXES.len())];
    let s = SUFFIXES[rng.gen_range(0..SUFFIXES.len())];
    format!("{}_{}", p, s)
}

/// Selects the next available neon color not currently in use.
pub fn get_next_color(used_colors: &[String]) -> String {
    for c in &NEON_COLORS {
        if !used_colors.contains(&c.to_string()) {
            return c.to_string();
        }
    }
    NEON_COLORS[rand::thread_rng().gen_range(0..NEON_COLORS.len())].to_string()
}
