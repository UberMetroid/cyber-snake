//! Food, bonus food, and powerup pickup handlers.
//! Re-exports from magnet.rs, pickup.rs, and speed.rs for backwards compatibility.

pub use crate::game::magnet::apply_magnet_effect;
pub use crate::game::pickup::{
    handle_bonus_food_pickup, handle_food_pickup, handle_powerup_pickup,
};
pub use crate::game::speed::calculate_effective_speed;
