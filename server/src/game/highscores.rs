//! High score persistence for the game.
//! Handles loading, saving, and updating player high scores.

use shared::HighScore;

/// Loads high scores from the JSON file in the data directory.
pub fn load_highscores() -> Vec<HighScore> {
    if let Ok(data) = std::fs::read_to_string("data/highscores.json") {
        if let Ok(scores) = serde_json::from_str(&data) {
            return scores;
        }
    }
    Vec::new()
}

/// Asynchronously saves high scores to the JSON file.
pub fn save_highscores(scores: &[HighScore]) {
    if let Ok(data) = serde_json::to_string(scores) {
        tokio::spawn(async move {
            let _ = tokio::fs::write("data/highscores.json", data).await;
        });
    }
}

/// Updates the high scores list with current player scores and persists to disk.
pub fn update_high_scores(
    snakes: &std::collections::HashMap<String, shared::Snake>,
) -> Vec<HighScore> {
    let mut all_scores: Vec<(&str, u32)> = snakes
        .values()
        .filter(|s| s.spawned)
        .map(|s| (s.name.as_str(), s.score))
        .collect();

    all_scores.sort_by(|a, b| b.1.cmp(&a.1));

    let top_10: Vec<HighScore> = all_scores
        .into_iter()
        .take(10)
        .map(|(name, score)| HighScore::new(name.to_string(), score))
        .collect();

    save_highscores(&top_10);
    top_10
}
