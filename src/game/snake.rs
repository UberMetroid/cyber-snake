use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: &Point) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn to_point(&self) -> Point {
        match self {
            Direction::Up => Point::new(0, -1),
            Direction::Down => Point::new(0, 1),
            Direction::Left => Point::new(-1, 0),
            Direction::Right => Point::new(1, 0),
        }
    }

    pub fn opposite(&self) -> bool {
        matches!(
            self,
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snake {
    pub id: String,
    pub segments: Vec<Point>,
    pub dir: Direction,
    pub next_dir: Direction,
    pub color: String,
    pub name: String,
    pub score: u32,
    pub alive: bool,
    pub spawned: bool,
    pub speed: u32,
    pub frame_count: u32,
    pub death_reason: Option<String>,
    pub held_powerup: Option<String>,
    pub active_effects: ActiveEffects,
    pub super_meter: u32,
    pub super_mode_start: Option<i64>,
    pub own_food_count: u32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ActiveEffects {
    pub speed_boost: Option<i64>,
    pub shield: Option<i64>,
    pub ghost: Option<i64>,
    pub magnet: Option<i64>,
    pub super_mode: Option<i64>,
    pub slowed: Option<i64>,
}

impl Snake {
    pub fn new(id: String, color: String, name: String) -> Self {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(5..25) as i32;
        let y = rng.gen_range(5..25) as i32;
        let dirs = vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        let dir = dirs[rng.gen_range(0..4)].clone();

        Self {
            id,
            segments: vec![Point::new(x, y)],
            dir: dir.clone(),
            next_dir: dir,
            color,
            name,
            score: 0,
            alive: true,
            spawned: false,
            speed: 2,
            frame_count: 0,
            death_reason: None,
            held_powerup: None,
            active_effects: ActiveEffects::default(),
            super_meter: 0,
            super_mode_start: None,
            own_food_count: 0,
        }
    }

    pub fn head(&self) -> Point {
        self.segments[0]
    }
}

pub const NEON_COLORS: [&str; 10] = [
    "#00ff88", "#ff00ff", "#00ffff", "#ffff00", "#ff8800", "#88ff00", "#ff0088", "#00ffaa",
    "#ff4444", "#44ff44",
];

pub const PREFIXES: [&str; 10] = [
    "GHOST", "CIPHER", "NEON", "VECTOR", "PIXEL", "GLITCH", "BYTE", "NEXUS", "PROXY", "SYNC",
];

pub const SUFFIXES: [&str; 10] = ["42", "7X", "99", "3K", "77", "AA", "ZZ", "Q", "XX", "01"];

pub fn random_name() -> String {
    let mut rng = rand::thread_rng();
    let p = PREFIXES[rng.gen_range(0..PREFIXES.len())];
    let s = SUFFIXES[rng.gen_range(0..SUFFIXES.len())];
    format!("{}_{}", p, s)
}

pub fn get_next_color(used_colors: &[String]) -> String {
    for c in &NEON_COLORS {
        if !used_colors.contains(&c.to_string()) {
            return c.to_string();
        }
    }
    NEON_COLORS[rand::thread_rng().gen_range(0..NEON_COLORS.len())].to_string()
}
