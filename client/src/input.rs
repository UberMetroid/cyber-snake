//! Keyboard input handling for the WASM client.

use shared::ClientMessage;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use web_sys::{KeyboardEvent, WebSocket};

use crate::state::AppState;

/// Sets up keyboard event handling.
pub fn setup_input(state: Rc<RefCell<AppState>>, ws: WebSocket) -> Result<(), JsValue> {
    let ws_clone = ws.clone();
    let state_clone = state.clone();

    let onkeydown_callback = Closure::wrap(Box::new(move |e: KeyboardEvent| {
        let state = state_clone.borrow();
        if !state.connected {
            return;
        }
        drop(state);

        let send = |msg: &ClientMessage| {
            if ws_clone.ready_state() == WebSocket::OPEN {
                if let Ok(data) = rmp_serde::to_vec_named(msg) {
                    let _ = ws_clone.send_with_u8_array(&data);
                }
            }
        };

        match e.key().as_str() {
            " " => {
                let spawned = state_clone.borrow().spawned;
                if !spawned {
                    send(&ClientMessage::Spawn);
                } else {
                    send(&ClientMessage::Respawn);
                }
            }
            "ArrowUp" => send(&ClientMessage::Input { dir: "up".into() }),
            "ArrowDown" => send(&ClientMessage::Input { dir: "down".into() }),
            "ArrowLeft" => send(&ClientMessage::Input { dir: "left".into() }),
            "ArrowRight" => send(&ClientMessage::Input {
                dir: "right".into(),
            }),
            _ => {}
        }
    }) as Box<dyn FnMut(_)>);

    web_sys::window()
        .unwrap()
        .add_event_listener_with_callback("keydown", onkeydown_callback.as_ref().unchecked_ref())?;
    onkeydown_callback.forget();

    Ok(())
}
