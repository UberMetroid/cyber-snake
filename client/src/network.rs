//! WebSocket networking for the WASM client.

use js_sys::Uint8Array;
use shared::{GameBroadcast, WelcomeMessage};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};

use crate::state::AppState;

/// Sets up WebSocket connection and message handling.
pub fn setup_websocket(
    state: Rc<RefCell<AppState>>,
    ws: &WebSocket,
) -> Result<(), wasm_bindgen::JsValue> {
    let state_clone = state.clone();
    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        if let Ok(ab) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            let array = Uint8Array::new(&ab);
            let mut data = vec![0; array.length() as usize];
            array.copy_to(&mut data[..]);

            if let Ok(welcome) = rmp_serde::from_slice::<WelcomeMessage>(&data) {
                let mut state = state_clone.borrow_mut();
                state.my_id = welcome.id;
                state.grid_cols = welcome.cols;
                state.grid_rows = welcome.rows;
                state.connected = true;
            } else if let Ok(broadcast) = rmp_serde::from_slice::<GameBroadcast>(&data) {
                let mut state = state_clone.borrow_mut();
                if let Some(my_snake) = broadcast.snakes.get(&state.my_id) {
                    state.spawned = my_snake.spawned;
                }
                state.game_state = Some(broadcast);
            }
        }
    });
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();
    Ok(())
}
