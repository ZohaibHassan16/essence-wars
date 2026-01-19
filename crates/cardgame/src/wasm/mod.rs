//! WASM bridge for browser-based game clients.
//!
//! This module provides WebAssembly bindings for the card game engine,
//! enabling deployment in web browsers via wasm-pack.
//!
//! # Building
//!
//! ```bash
//! wasm-pack build --target web --features wasm
//! ```
//!
//! # Usage from JavaScript
//!
//! ```javascript
//! import init, { WasmGameClient } from './pkg/cardgame.js';
//!
//! async function main() {
//!     await init();
//!
//!     const client = WasmGameClient.new();
//!     const deck1 = JSON.stringify([1000, 1001, 1002, ...]);
//!     const deck2 = JSON.stringify([2000, 2001, 2002, ...]);
//!
//!     client.start_game(deck1, deck2, BigInt(12345));
//!
//!     const actions = JSON.parse(client.get_legal_actions());
//!     const events = client.apply_action(JSON.stringify(actions[0]));
//! }
//! ```

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use crate::cards::CardDatabase;
#[cfg(feature = "wasm")]
use crate::client_api::{GameClient, GameClientBuilder};
#[cfg(feature = "wasm")]
use crate::actions::Action;
#[cfg(feature = "wasm")]
use crate::types::CardId;
#[cfg(feature = "wasm")]
use std::sync::Arc;

/// Initialize panic hook for better error messages in browser console.
#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// WASM-compatible game client wrapper.
///
/// All methods accept and return JSON strings for easy JavaScript interop.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmGameClient {
    client: GameClient,
    card_db: Arc<CardDatabase>,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmGameClient {
    /// Create a new game client with an empty card database.
    ///
    /// Note: In WASM, the card database must be loaded via `new_with_cards()`.
    /// This constructor is mainly for testing.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmGameClient, JsValue> {
        // Create an empty database - cards must be loaded separately
        let card_db = Arc::new(CardDatabase::empty());
        let client = GameClientBuilder::new()
            .with_card_db(card_db.clone())
            .build();

        Ok(WasmGameClient { client, card_db })
    }

    /// Create a game client with a card database loaded from JSON.
    ///
    /// # Arguments
    /// * `cards_json` - JSON array of card definitions
    ///
    /// # Example
    /// ```javascript
    /// const cards = [
    ///   { "id": 1000, "name": "Brass Sentinel", "cost": 2, ... },
    ///   { "id": 1001, "name": "Steel Guardian", "cost": 3, ... }
    /// ];
    /// const client = WasmGameClient.new_with_cards(JSON.stringify(cards));
    /// ```
    #[wasm_bindgen]
    pub fn new_with_cards(cards_json: &str) -> Result<WasmGameClient, JsValue> {
        // Parse as a list of CardDefinition
        let cards: Vec<crate::cards::CardDefinition> = serde_json::from_str(cards_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse cards JSON: {}", e)))?;

        let card_db = Arc::new(CardDatabase::new(cards));
        let client = GameClientBuilder::new()
            .with_card_db(card_db.clone())
            .build();

        Ok(WasmGameClient { client, card_db })
    }

    /// Start a new game with the given decks and seed.
    ///
    /// # Arguments
    /// * `deck1_json` - JSON array of card IDs for player 1
    /// * `deck2_json` - JSON array of card IDs for player 2
    /// * `seed` - Random seed for deterministic gameplay
    ///
    /// # Returns
    /// JSON string containing initial game events
    #[wasm_bindgen]
    pub fn start_game(&mut self, deck1_json: &str, deck2_json: &str, seed: u64) -> Result<String, JsValue> {
        let deck1_ids: Vec<u16> = serde_json::from_str(deck1_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse deck1: {}", e)))?;
        let deck2_ids: Vec<u16> = serde_json::from_str(deck2_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse deck2: {}", e)))?;

        // Convert u16 to CardId
        let deck1: Vec<CardId> = deck1_ids.into_iter().map(CardId).collect();
        let deck2: Vec<CardId> = deck2_ids.into_iter().map(CardId).collect();

        self.client.start_game(deck1, deck2, seed);

        let events = self.client.drain_events();
        serde_json::to_string(&events)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize events: {}", e)))
    }

    /// Apply an action and return the resulting events.
    ///
    /// # Arguments
    /// * `action_json` - JSON string representing the action
    ///
    /// # Returns
    /// JSON string containing game events triggered by the action
    #[wasm_bindgen]
    pub fn apply_action(&mut self, action_json: &str) -> Result<String, JsValue> {
        let action: Action = serde_json::from_str(action_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse action: {}", e)))?;

        let events = self.client.apply_action(action)
            .map_err(|e| JsValue::from_str(&e))?;

        serde_json::to_string(&events)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize events: {}", e)))
    }

    /// Apply an action by its index in the action space (0-255).
    ///
    /// # Arguments
    /// * `action_index` - Index of the action (0-255)
    ///
    /// # Returns
    /// JSON string containing game events triggered by the action
    #[wasm_bindgen]
    pub fn apply_action_by_index(&mut self, action_index: u8) -> Result<String, JsValue> {
        let events = self.client.apply_action_by_index(action_index)
            .map_err(|e| JsValue::from_str(&e))?;

        serde_json::to_string(&events)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize events: {}", e)))
    }

    /// Get the list of legal actions.
    ///
    /// # Returns
    /// JSON array of legal actions
    #[wasm_bindgen]
    pub fn get_legal_actions(&self) -> Result<String, JsValue> {
        let actions = self.client.get_legal_actions();
        serde_json::to_string(&actions)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize actions: {}", e)))
    }

    /// Get the legal action mask (256 floats, 1.0 = legal, 0.0 = illegal).
    ///
    /// # Returns
    /// JSON array of 256 floats
    #[wasm_bindgen]
    pub fn get_legal_action_mask(&self) -> Result<String, JsValue> {
        let mask = self.client.get_legal_action_mask();
        // Convert array to Vec for serialization (arrays > 32 don't impl Serialize)
        let mask_vec: Vec<f32> = mask.to_vec();
        serde_json::to_string(&mask_vec)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize mask: {}", e)))
    }

    /// Get the current game state as JSON.
    ///
    /// # Returns
    /// JSON string of the game state, or null if no game is active
    #[wasm_bindgen]
    pub fn get_state_json(&self) -> Result<String, JsValue> {
        match self.client.get_state() {
            Some(state) => serde_json::to_string(state)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize state: {}", e))),
            None => Ok("null".to_string()),
        }
    }

    /// Get the state tensor (326 floats) for neural network input.
    ///
    /// # Returns
    /// JSON array of 326 floats
    #[wasm_bindgen]
    pub fn get_state_tensor(&self) -> Result<String, JsValue> {
        let tensor = self.client.get_state_tensor();
        // Convert array to Vec for serialization (arrays > 32 don't impl Serialize)
        let tensor_vec: Vec<f32> = tensor.to_vec();
        serde_json::to_string(&tensor_vec)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize tensor: {}", e)))
    }

    /// Check if the game is over.
    #[wasm_bindgen]
    pub fn is_game_over(&self) -> bool {
        self.client.is_game_over()
    }

    /// Get the game result.
    ///
    /// # Returns
    /// JSON string of the game result, or null if game is not over
    #[wasm_bindgen]
    pub fn get_result(&self) -> Result<String, JsValue> {
        match self.client.get_result() {
            Some(result) => serde_json::to_string(&result)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e))),
            None => Ok("null".to_string()),
        }
    }

    /// Get the active player ID (0 or 1).
    #[wasm_bindgen]
    pub fn get_active_player(&self) -> u8 {
        self.client.get_active_player().0
    }

    /// Get the current turn number.
    #[wasm_bindgen]
    pub fn get_turn(&self) -> u16 {
        self.client.get_turn()
    }

    /// Drain all pending events.
    ///
    /// # Returns
    /// JSON array of events
    #[wasm_bindgen]
    pub fn drain_events(&mut self) -> Result<String, JsValue> {
        let events = self.client.drain_events();
        serde_json::to_string(&events)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize events: {}", e)))
    }

    /// Get card information by ID.
    ///
    /// # Returns
    /// JSON string of card definition, or null if not found
    #[wasm_bindgen]
    pub fn get_card(&self, card_id: u16) -> Result<String, JsValue> {
        match self.card_db.get(CardId(card_id)) {
            Some(card) => serde_json::to_string(card)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize card: {}", e))),
            None => Ok("null".to_string()),
        }
    }
}

/// Utility functions for WASM environment.
#[cfg(feature = "wasm")]
pub mod utils {
    use wasm_bindgen::prelude::*;

    /// Log a message to the browser console.
    #[wasm_bindgen]
    pub fn log(message: &str) {
        web_sys::console::log_1(&JsValue::from_str(message));
    }

    /// Get the engine version string.
    #[wasm_bindgen]
    pub fn version() -> String {
        crate::version::version_string()
    }
}

// Re-export for non-WASM builds (no-op stubs)
#[cfg(not(feature = "wasm"))]
pub struct WasmGameClient;

#[cfg(not(feature = "wasm"))]
impl WasmGameClient {
    pub fn new() -> Result<Self, String> {
        Err("WASM feature not enabled. Build with --features wasm".to_string())
    }
}
