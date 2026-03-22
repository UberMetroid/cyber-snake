//! Canvas rendering for the WASM client.

#![allow(deprecated)]

use std::cell::RefCell;
use std::rc::Rc;
use web_sys::CanvasRenderingContext2d;

use crate::state::AppState;

/// Grid cell size in pixels.
const GRID_SIZE: f64 = 20.0;

/// Renders the game state to the canvas.
pub fn render(
    canvas: &web_sys::HtmlCanvasElement,
    context: &CanvasRenderingContext2d,
    state: &Rc<RefCell<AppState>>,
) -> Result<(), wasm_bindgen::JsValue> {
    let canvas_w = canvas.width() as f64;
    let canvas_h = canvas.height() as f64;

    // Clear canvas
    context.set_fill_style(&wasm_bindgen::JsValue::from_str("#050508"));
    context.fill_rect(0.0, 0.0, canvas_w, canvas_h);

    // Get state
    let (cols, rows, my_id, spawned, current_state) = {
        let state = state.borrow();
        let gs = state.game_state.clone();
        (
            state.grid_cols as f64,
            state.grid_rows as f64,
            state.my_id.clone(),
            state.spawned,
            gs,
        )
    };

    // Calculate camera position
    let (mut target_x, mut target_y, mut cam_x, mut cam_y) = {
        let state = state.borrow();
        (
            state.camera_x,
            state.camera_y,
            state.camera_x,
            state.camera_y,
        )
    };

    if let Some(gs) = &current_state {
        // Find snake to follow
        let mut focus_snake = gs.snakes.get(&my_id);
        if focus_snake.is_none() || !focus_snake.unwrap().alive || !focus_snake.unwrap().spawned {
            let mut best_score = -1;
            for s in gs.snakes.values() {
                if s.alive && s.spawned && (s.score as i32) > best_score {
                    best_score = s.score as i32;
                    focus_snake = Some(s);
                }
            }
        }

        if let Some(s) = focus_snake {
            if !s.segments.is_empty() {
                let head = s.segments[0];
                let head_px = head.x as f64 * GRID_SIZE + GRID_SIZE / 2.0;
                let head_py = head.y as f64 * GRID_SIZE + GRID_SIZE / 2.0;

                let margin_x = canvas_w * 0.25;
                let margin_y = canvas_h * 0.25;

                let rel_x = head_px - cam_x + canvas_w / 2.0;
                let rel_y = head_py - cam_y + canvas_h / 2.0;

                if rel_x < margin_x {
                    cam_x -= margin_x - rel_x;
                } else if rel_x > canvas_w - margin_x {
                    cam_x += rel_x - (canvas_w - margin_x);
                }

                if rel_y < margin_y {
                    cam_y -= margin_y - rel_y;
                } else if rel_y > canvas_h - margin_y {
                    cam_y += rel_y - (canvas_h - margin_y);
                }

                let world_w = cols * GRID_SIZE;
                let world_h = rows * GRID_SIZE;
                cam_x = cam_x.clamp(canvas_w / 2.0, world_w - canvas_w / 2.0);
                cam_y = cam_y.clamp(canvas_h / 2.0, world_h - canvas_h / 2.0);

                target_x = cam_x;
                target_y = cam_y;
            }
        }
    }

    state.borrow_mut().camera_x = target_x;
    state.borrow_mut().camera_y = target_y;

    // Apply camera transform
    context.save();
    context.translate(canvas_w / 2.0 - target_x, canvas_h / 2.0 - target_y)?;

    // Draw grid
    let world_w = cols * GRID_SIZE;
    let world_h = rows * GRID_SIZE;
    context.set_stroke_style(&wasm_bindgen::JsValue::from_str("rgba(0, 255, 255, 0.05)"));
    context.set_line_width(1.0);
    context.begin_path();
    for x in 0..=(cols as i32) {
        context.move_to(x as f64 * GRID_SIZE, 0.0);
        context.line_to(x as f64 * GRID_SIZE, world_h);
    }
    for y in 0..=(rows as i32) {
        context.move_to(0.0, y as f64 * GRID_SIZE);
        context.line_to(world_w, y as f64 * GRID_SIZE);
    }
    context.stroke();

    // Draw world boundary
    context.set_stroke_style(&wasm_bindgen::JsValue::from_str("#00ffff"));
    context.set_line_width(4.0);
    context.stroke_rect(0.0, 0.0, world_w, world_h);

    // Draw food
    if let Some(gs) = &current_state {
        for food in gs.foods.values() {
            context.set_fill_style(&wasm_bindgen::JsValue::from_str(&food.color));
            context.set_stroke_style(&wasm_bindgen::JsValue::from_str(&food.color));
            context.set_line_width(2.0);
            context.begin_path();
            let cx = food.x as f64 * GRID_SIZE + GRID_SIZE / 2.0;
            let cy = food.y as f64 * GRID_SIZE + GRID_SIZE / 2.0;

            if food.is_ring {
                context.arc(cx, cy, GRID_SIZE / 2.0, 0.0, std::f64::consts::PI * 2.0)?;
                context.stroke();
            } else if food.is_super {
                context.arc(
                    cx,
                    cy,
                    GRID_SIZE / 2.0 - 2.0,
                    0.0,
                    std::f64::consts::PI * 2.0,
                )?;
                context.fill();
                context.begin_path();
                context.arc(
                    cx,
                    cy,
                    GRID_SIZE / 2.0 + 2.0,
                    0.0,
                    std::f64::consts::PI * 2.0,
                )?;
                context.stroke();
            } else {
                context.arc(
                    cx,
                    cy,
                    GRID_SIZE / 2.0 - 2.0,
                    0.0,
                    std::f64::consts::PI * 2.0,
                )?;
                context.fill();
            }
        }

        // Draw snakes
        for snake in gs.snakes.values() {
            if !snake.spawned {
                continue;
            }
            context.set_fill_style(&wasm_bindgen::JsValue::from_str(&snake.color));
            if !snake.alive {
                context.set_global_alpha(0.3);
            } else {
                context.set_global_alpha(1.0);
            }
            for seg in &snake.segments {
                context.fill_rect(
                    seg.x as f64 * GRID_SIZE + 1.0,
                    seg.y as f64 * GRID_SIZE + 1.0,
                    GRID_SIZE - 2.0,
                    GRID_SIZE - 2.0,
                );
            }
        }
    }

    context.restore();

    // Draw UI
    context.set_global_alpha(1.0);
    context.set_fill_style(&wasm_bindgen::JsValue::from_str("white"));
    context.set_font("30px sans-serif");
    let score = current_state
        .as_ref()
        .and_then(|gs| gs.snakes.get(&my_id))
        .map(|s| s.score)
        .unwrap_or(0);
    context.fill_text(&format!("SCORE: {}", score), 10.0, 40.0)?;

    if !spawned {
        context.fill_text(
            "PRESS SPACE TO SPAWN",
            canvas_w / 2.0 - 180.0,
            canvas_h / 2.0,
        )?;
    }

    Ok(())
}
