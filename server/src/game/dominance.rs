//! Dominance determination when two snakes collide.

use shared::Snake;

/// Context for determining collision dominance between two snakes.
#[allow(dead_code)]
pub struct DominanceContext {
    pub attacker_speed: u32,
    pub defender_speed: u32,
    pub attacker_shield: bool,
    pub defender_shield: bool,
    pub attacker_super: bool,
    pub defender_super: bool,
}

impl DominanceContext {
    pub fn new(
        attacker_speed: u32,
        defender_speed: u32,
        attacker_shield: bool,
        defender_shield: bool,
        attacker_super: bool,
        defender_super: bool,
    ) -> Self {
        Self {
            attacker_speed,
            defender_speed,
            attacker_shield,
            defender_shield,
            attacker_super,
            defender_super,
        }
    }
}

/// Determines dominance outcome when two snakes collide at the same position.
/// Returns (attacker_kills, attacker_died).
pub fn determine_dominance(
    attacker: &Snake,
    defender: &Snake,
    ctx: DominanceContext,
) -> (bool, bool) {
    if ctx.attacker_shield || ctx.attacker_super {
        tracing::info!(
            "[SHIELD/STAR] {} killed by {} ram",
            defender.color,
            attacker.color
        );
        return (true, false);
    }

    if attacker.segments.len() > defender.segments.len() || ctx.attacker_speed > ctx.defender_speed
    {
        tracing::info!(
            "[DOMINANCE] {} (Size: {}, Spd: {}) destroyed {} (Size: {}, Spd: {})",
            attacker.name,
            attacker.segments.len(),
            ctx.attacker_speed,
            defender.name,
            defender.segments.len(),
            ctx.defender_speed
        );
        return (true, false);
    }

    (false, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::{Direction, Point, Snake};

    fn create_test_snake(id: &str, segments: Vec<Point>) -> Snake {
        Snake {
            id: id.to_string(),
            segments,
            dir: Direction::Up,
            next_dir: Direction::Up,
            color: "#ff0000".to_string(),
            name: id.to_string(),
            is_bot: false,
            score: 0,
            alive: true,
            spawned: true,
            speed: 2,
            frame_count: 0,
            death_reason: None,
            held_powerup: None,
            active_effects: shared::ActiveEffects::default(),
            super_meter: 0,
            super_mode_start: None,
            own_food_count: 0,
        }
    }

    #[test]
    fn test_determine_dominance_attacker_shield_wins() {
        let attacker = create_test_snake("a", vec![Point::new(0, 0)]);
        let defender = create_test_snake("d", vec![Point::new(1, 0)]);
        let (kill_other, killed_by) = determine_dominance(
            &attacker,
            &defender,
            DominanceContext::new(2, 2, true, false, false, false),
        );
        assert!(kill_other);
        assert!(!killed_by);
    }

    #[test]
    fn test_determine_dominance_larger_snake_wins() {
        let attacker = create_test_snake(
            "a",
            vec![
                Point::new(0, 0),
                Point::new(0, 1),
                Point::new(0, 2),
                Point::new(0, 3),
                Point::new(0, 4),
            ],
        );
        let defender = create_test_snake(
            "d",
            vec![Point::new(1, 0), Point::new(1, 1), Point::new(1, 2)],
        );
        let (kill_other, killed_by) = determine_dominance(
            &attacker,
            &defender,
            DominanceContext::new(2, 2, false, false, false, false),
        );
        assert!(kill_other);
        assert!(!killed_by);
    }

    #[test]
    fn test_determine_dominance_smaller_snake_loses() {
        let attacker = create_test_snake("a", vec![Point::new(0, 0)]);
        let defender = create_test_snake("d", vec![Point::new(1, 0)]);
        let (kill_other, killed_by) = determine_dominance(
            &attacker,
            &defender,
            DominanceContext::new(2, 2, false, false, false, false),
        );
        assert!(!kill_other);
        assert!(killed_by);
    }

    #[test]
    fn test_determine_dominance_faster_snake_wins() {
        let attacker = create_test_snake("a", vec![Point::new(0, 0)]);
        let defender = create_test_snake("d", vec![Point::new(1, 0)]);
        let (kill_other, killed_by) = determine_dominance(
            &attacker,
            &defender,
            DominanceContext::new(4, 2, false, false, false, false),
        );
        assert!(kill_other);
        assert!(!killed_by);
    }
}
