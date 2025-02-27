use wasm_bindgen::prelude::*;

use crate::gamemodes::base::Game;
use crate::GameView;
use crate::models::Move;

#[wasm_bindgen]
pub struct LocalGame {
    game: Game,
    on_send_view: js_sys::Function,
    on_send_error: js_sys::Function,
}

#[wasm_bindgen]
impl LocalGame {
    #[wasm_bindgen(constructor)]
    pub fn new(on_send_view: js_sys::Function, on_send_error: js_sys::Function) -> LocalGame {
        let game = Game::new();
        let game = LocalGame { game, on_send_view, on_send_error };
        game.send_current_view();
        return game;
    }
}

impl LocalGame {
    fn try_move(&mut self, game_move: Move) -> Result<(), String> {
        self.game.try_move(game_move)?;
        self.send_current_view();
        Ok(())
    }
    fn send_current_view(&self) {
        let view = GameView::from(&self.game.get_state());
        self.send_view(view);
    }
    fn send_view(&self, view: GameView) {
        let view = JsValue::from_serde(&view).unwrap();
        let this = JsValue::null();
        match self.on_send_view.call1(&this, &view) {
            Ok(_) => {},
            Err(err) => {
                log::error!("Failed to call on_send_view: {:?}", err);
            },
        };
    }
    fn send_error(&self, error: String) {
        let error = JsValue::from(error);
        let this = JsValue::null();
        match self.on_send_error.call1(&this, &error) {
            Ok(_) => {},
            Err(err) => {
                log::error!("Failed to call on_send_error: {:?}", err);
            },
        };
    }
}

#[wasm_bindgen]
impl LocalGame {
    #[wasm_bindgen(js_name = move)]
    pub fn play_move(&mut self, game_move: &JsValue) {
        let game_move: Move = match game_move.into_serde() {
            Ok(game_move) => game_move,
            Err(err) => {
                self.send_error(err.to_string());
                return;
            }
        };
        match self.try_move(game_move) {
            Ok(()) => {
                log::info!("Successfully played move");
            },
            Err(err) => {
                self.send_error(err);
            }
        };
    }
    pub fn reset(&mut self) {
        self.game.reset();
        self.send_current_view();
    }
}
