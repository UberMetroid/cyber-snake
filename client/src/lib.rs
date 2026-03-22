//! Cyber-Snake WASM client library.

mod input;
mod network;
mod render;
mod state;

use input::setup_input;
use network::setup_websocket;
use render::render;
use state::AppState;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

type AnimationFrameClosure = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

#[wasm_bindgen]
pub struct CyberSnakeClient {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    state: Rc<RefCell<AppState>>,
}

#[wasm_bindgen]
impl CyberSnakeClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<CyberSnakeClient, JsValue> {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id("glcanvas").unwrap();
        let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;
        let context: CanvasRenderingContext2d = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;

        let state = Rc::new(RefCell::new(AppState::default()));

        Ok(CyberSnakeClient {
            canvas,
            context,
            state,
        })
    }

    /// Connects to the game server and starts the game loop.
    pub fn start(&self) -> Result<(), JsValue> {
        let window = web_sys::window().expect("no global `window` exists");
        let location = window.location();
        let host = location.host().unwrap_or("localhost:8300".to_string());
        let protocol = if location.protocol().unwrap_or("http:".to_string()) == "https:" {
            "wss:"
        } else {
            "ws:"
        };

        let url = format!("{}//{}/ws", protocol, host);
        let ws = web_sys::WebSocket::new(&url)?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        setup_websocket(self.state.clone(), &ws)?;
        setup_input(self.state.clone(), ws)?;

        let state = self.state.clone();
        let canvas = self.canvas.clone();
        let context = self.context.clone();
        let window_clone = window.clone();

        let closure: AnimationFrameClosure = Rc::new(RefCell::new(None));
        let closure_clone = closure.clone();

        *closure.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let canvas_w = window.inner_width().unwrap().as_f64().unwrap();
            let canvas_h = window.inner_height().unwrap().as_f64().unwrap();
            canvas.set_width(canvas_w as u32);
            canvas.set_height(canvas_h as u32);

            let _ = render(&canvas, &context, &state);

            let _ = window.request_animation_frame(
                closure_clone
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            );
        }) as Box<dyn FnMut()>));

        let _ = window_clone
            .request_animation_frame(closure.borrow().as_ref().unwrap().as_ref().unchecked_ref());

        Ok(())
    }
}
