//! Game tick loop and bot decision updates.
//! Extracted from state.rs to reduce file size.

use crate::config::{BONUS_FOOD_SPAWN_INTERVAL, MAX_BONUS_FOOD_COUNT, POWERUP_SPAWN_INTERVAL};
use crate::game::{bot_ai, effects, snake_mgr, spawner};
use chrono::Utc;

use crate::game::state::GameState;

const MAX_POWERUPS: usize = 3;

/// Main game tick - updates all entities each game loop iteration.
pub fn tick(state: &mut GameState) {
    let now = Utc::now().timestamp_millis();
    state.tick += 1;

    state.foods.retain(|_, food| !food.is_expired());
    state.bonus_foods.retain(|bf| !bf.is_expired());
    state.powerups.retain(|pu| !pu.is_expired());
    state.explosions.retain(|e| e.expires_at > now);

    snake_mgr::remove_dead_bots(
        &mut state.snakes,
        &mut state.foods,
        &mut state.bonus_foods,
        &mut state.powerups,
    );
    snake_mgr::spawn_bots_if_needed(&mut state.snakes, &mut state.foods);

    update_bot_decisions(state);

    if state.bonus_foods.len() < MAX_BONUS_FOOD_COUNT
        && state.tick.is_multiple_of(BONUS_FOOD_SPAWN_INTERVAL)
    {
        spawner::spawn_bonus_food(
            &state.snakes,
            &state.foods,
            &state.powerups,
            &mut state.bonus_foods,
        );
    }

    if state.powerups.len() < MAX_POWERUPS && state.tick.is_multiple_of(POWERUP_SPAWN_INTERVAL) {
        spawner::spawn_powerup(
            &state.snakes,
            &state.foods,
            &state.bonus_foods,
            &mut state.powerups,
        );
    }

    let spawned_player_ids: Vec<String> = state
        .snakes
        .values()
        .filter(|p| p.alive && p.spawned)
        .map(|p| p.id.clone())
        .collect();

    for id in spawned_player_ids {
        state.update_snake(&id, now);
    }
}

/// Updates AI decisions for all active bots.
fn update_bot_decisions(state: &mut GameState) {
    let mut bots_to_activate = Vec::new();
    let mut bot_decisions: Vec<(String, bot_ai::BotDecision)> = Vec::new();

    for (id, bot) in state.snakes.iter() {
        if !bot.is_bot || !bot.alive || !bot.spawned {
            continue;
        }

        let decision = bot_ai::compute_bot_direction(
            bot,
            &state.snakes,
            &state.foods,
            &state.bonus_foods,
            &state.powerups,
        );

        if decision.should_activate_powerup {
            bots_to_activate.push(id.clone());
        }

        bot_decisions.push((id.clone(), decision));
    }

    for (id, decision) in bot_decisions {
        if let Some(bot) = state.snakes.get_mut(&id) {
            bot.next_dir = decision.next_direction;
        }
    }

    for id in bots_to_activate {
        effects::activate_powerup(
            &mut state.snakes,
            &mut state.powerups,
            &mut state.explosions,
            &id,
        );
    }
}
