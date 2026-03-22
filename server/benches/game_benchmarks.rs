//! Game performance benchmarks.
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use server::game::collision::{check_head_collision, check_self_collision};
use server::game::state::GameState;
use shared::{Point, Snake};

fn create_test_snake(id: &str, x: i32, y: i32) -> Snake {
    Snake::new(
        id.to_string(),
        "#ff0000".to_string(),
        format!("Test_{}", id),
        x,
        y,
    )
}

fn create_large_snake(length: usize, start_x: i32, start_y: i32) -> Snake {
    let mut segments = Vec::with_capacity(length);
    for i in 0..length {
        segments.push(Point::new(start_x, start_y + i as i32));
    }
    Snake {
        id: "benchmark".to_string(),
        segments,
        dir: shared::Direction::Up,
        next_dir: shared::Direction::Up,
        color: "#ff0000".to_string(),
        name: "Benchmark".to_string(),
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

pub fn criterion_benchmark(c: &mut Criterion) {
    // Self collision benchmark
    c.bench_function("self_collision_short", |b| {
        let snake = create_large_snake(10, 0, 0);
        let head = Point::new(0, 5);
        b.iter(|| check_self_collision(black_box(head), black_box(&snake.segments)));
    });

    c.bench_function("self_collision_long", |b| {
        let snake = create_large_snake(100, 0, 0);
        let head = Point::new(0, 50);
        b.iter(|| check_self_collision(black_box(head), black_box(&snake.segments)));
    });

    // Head collision benchmark
    c.bench_function("head_collision_no_hit", |b| {
        let mut snakes = std::collections::HashMap::new();
        snakes.insert("snake1".to_string(), create_test_snake("snake1", 0, 0));
        snakes.insert("snake2".to_string(), create_test_snake("snake2", 100, 100));

        let head = Point::new(50, 50);
        b.iter(|| check_head_collision(black_box(head), black_box("snake2"), black_box(&snakes)));
    });

    // Game tick benchmark
    c.bench_function("game_tick_empty", |b| {
        let mut state = GameState::new();
        b.iter(|| state.tick());
    });

    c.bench_function("game_tick_with_players", |b| {
        let mut state = GameState::new();
        for i in 0..10 {
            let snake = state.create_snake(format!("player{}", i));
            state.snakes.insert(format!("player{}", i), snake);
            state.spawn_snake(&format!("player{}", i));
        }
        b.iter(|| state.tick());
    });

    // Serialization benchmark
    c.bench_function("broadcast_state", |b| {
        let mut state = GameState::new();
        for i in 0..5 {
            let snake = state.create_snake(format!("player{}", i));
            state.snakes.insert(format!("player{}", i), snake);
            state.spawn_snake(&format!("player{}", i));
        }

        b.iter(|| {
            let broadcast = state.broadcast_state();
            rmp_serde::to_vec_named(&broadcast)
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = criterion_benchmark
}
criterion_main!(benches);
