//! MCP (Model Context Protocol) implementation for Essence Wars.
//!
//! This module implements the JSON-RPC based MCP protocol for communication
//! with Claude Code.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::sync::Arc;

use crate::session::SessionManager;
use crate::tools::{
    ai::ai_hint,
    discovery::{list_bots, list_decks},
    game::{end_game, legal_actions, play_action, show_hand, show_state, start_game},
    screenshot::take_screenshot,
};

/// JSON-RPC request structure
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

/// JSON-RPC response structure
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

/// JSON-RPC error structure
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// Tool definition for MCP
#[derive(Debug, Serialize)]
struct ToolDefinition {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

/// MCP Server for Essence Wars
pub struct McpServer {
    session_manager: Arc<RwLock<SessionManager>>,
}

impl McpServer {
    /// Create a new MCP server instance
    pub fn new() -> anyhow::Result<Self> {
        let session_manager = SessionManager::new()?;
        Ok(Self {
            session_manager: Arc::new(RwLock::new(session_manager)),
        })
    }

    /// Run the MCP server on stdio
    pub fn run(&self) -> anyhow::Result<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        let reader = BufReader::new(stdin.lock());

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };

            if line.trim().is_empty() {
                continue;
            }

            let response = self.handle_request(&line);

            if let Some(resp) = response {
                let resp_str = serde_json::to_string(&resp)?;
                writeln!(stdout, "{}", resp_str)?;
                stdout.flush()?;
            }
        }

        Ok(())
    }

    /// Handle a single JSON-RPC request
    fn handle_request(&self, line: &str) -> Option<JsonRpcResponse> {
        let request: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(r) => r,
            Err(e) => {
                return Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Value::Null,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                });
            }
        };

        let id = request.id.as_ref()?;
        let id = id.clone();

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(&request.params),
            "initialized" => return None, // Notification
            "tools/list" => self.handle_list_tools(),
            "tools/call" => self.handle_call_tool(&request.params),
            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        };

        Some(match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(value),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(error),
            },
        })
    }

    /// Handle initialize request
    fn handle_initialize(&self, _params: &Option<Value>) -> Result<Value, JsonRpcError> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "essence-wars-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        }))
    }

    /// Handle tools/list request
    fn handle_list_tools(&self) -> Result<Value, JsonRpcError> {
        let tools = vec![
            ToolDefinition {
                name: "list_decks".to_string(),
                description: "List all available decks for playing Essence Wars, grouped by faction".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            ToolDefinition {
                name: "list_bots".to_string(),
                description: "List all available AI opponent types with their descriptions and difficulty levels".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            ToolDefinition {
                name: "start_game".to_string(),
                description: "Start a new game of Essence Wars against an AI opponent".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "player_deck": {
                            "type": "string",
                            "description": "ID of the deck you want to play (use list_decks to see options)"
                        },
                        "opponent_deck": {
                            "type": "string",
                            "description": "ID of the opponent's deck"
                        },
                        "bot_type": {
                            "type": "string",
                            "description": "Bot type: random, greedy, mcts, or alphabeta (default: greedy)"
                        },
                        "seed": {
                            "type": "integer",
                            "description": "Optional random seed for reproducible games"
                        }
                    },
                    "required": ["player_deck", "opponent_deck"]
                }),
            },
            ToolDefinition {
                name: "show_state".to_string(),
                description: "Display the current game board state including creatures, supports, and player stats".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            ToolDefinition {
                name: "show_hand".to_string(),
                description: "Display all cards in your hand with their costs, stats, and abilities".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            ToolDefinition {
                name: "legal_actions".to_string(),
                description: "List all legal moves you can make this turn with their action indices".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            ToolDefinition {
                name: "play_action".to_string(),
                description: "Execute a game action by its index number (get indices from legal_actions)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action_index": {
                            "type": "integer",
                            "description": "The action index to play (from legal_actions list)"
                        }
                    },
                    "required": ["action_index"]
                }),
            },
            ToolDefinition {
                name: "ai_hint".to_string(),
                description: "Get AI analysis with recommended move and win rate estimates using MCTS".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "simulations": {
                            "type": "integer",
                            "description": "Number of MCTS simulations for analysis (default 500, higher = more accurate but slower)"
                        }
                    },
                    "required": []
                }),
            },
            ToolDefinition {
                name: "end_game".to_string(),
                description: "End the current game session and show final results".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            ToolDefinition {
                name: "take_screenshot".to_string(),
                description: "Capture a screenshot of the Essence Wars UI. Returns base64-encoded PNG for vision analysis. Requires the Tauri desktop app to be running.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
        ];

        Ok(json!({ "tools": tools }))
    }

    /// Handle tools/call request
    fn handle_call_tool(&self, params: &Option<Value>) -> Result<Value, JsonRpcError> {
        let params = params.as_ref().ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing params".to_string(),
            data: None,
        })?;

        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing tool name".to_string(),
                data: None,
            })?;

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        let result = match name {
            "list_decks" => {
                let manager = self.session_manager.read();
                list_decks(&manager)
            }
            "list_bots" => list_bots(),
            "start_game" => {
                let player_deck = arguments
                    .get("player_deck")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "Missing player_deck argument".to_string(),
                        data: None,
                    })?;
                let opponent_deck = arguments
                    .get("opponent_deck")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "Missing opponent_deck argument".to_string(),
                        data: None,
                    })?;
                let bot_type = arguments
                    .get("bot_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("greedy");
                let seed = arguments.get("seed").and_then(|v| v.as_u64());

                let mut manager = self.session_manager.write();
                start_game(&mut manager, player_deck, opponent_deck, bot_type, seed)
            }
            "show_state" => {
                let manager = self.session_manager.read();
                show_state(&manager)
            }
            "show_hand" => {
                let manager = self.session_manager.read();
                show_hand(&manager)
            }
            "legal_actions" => {
                let manager = self.session_manager.read();
                legal_actions(&manager)
            }
            "play_action" => {
                let action_index = arguments
                    .get("action_index")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| JsonRpcError {
                        code: -32602,
                        message: "Missing action_index argument".to_string(),
                        data: None,
                    })? as u8;

                let mut manager = self.session_manager.write();
                play_action(&mut manager, action_index)
            }
            "ai_hint" => {
                let simulations = arguments
                    .get("simulations")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(500) as u32;

                let manager = self.session_manager.read();
                ai_hint(&manager, simulations)
            }
            "end_game" => {
                let mut manager = self.session_manager.write();
                end_game(&mut manager)
            }
            "take_screenshot" => take_screenshot(),
            _ => {
                return Err(JsonRpcError {
                    code: -32602,
                    message: format!("Unknown tool: {}", name),
                    data: None,
                });
            }
        };

        Ok(json!({
            "content": [{
                "type": "text",
                "text": result
            }]
        }))
    }
}
