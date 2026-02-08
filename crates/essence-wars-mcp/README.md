# Essence Wars - MCP Server

Model Context Protocol server enabling Claude Code to play Essence Wars interactively.

## Overview

This crate provides:

- **13 MCP tools** for complete game control
- **JSON-RPC 2.0** transport over stdio
- **AI analysis** with Alpha-Beta and MCTS recommendations
- **Educational tools** for learning rules and keywords
- **Optional UI sync** to the Tauri desktop application

## Quick Start

### Configuration

Add to `.mcp.json` at repository root:

```json
{
  "mcpServers": {
    "essence-wars": {
      "command": "/path/to/essence-wars-mcp"
    }
  }
}
```

### Build

```bash
cargo build --release -p essence-wars-mcp
```

### Usage in Claude Code

Once configured, Claude Code can play games using natural language:

```
"Start a game with the Iron Wall deck against Swarm Aggro"
"Show me my hand"
"What move does the AI recommend?"
"Play action 12"
```

## Available Tools

### Discovery

| Tool | Description |
|------|-------------|
| `list_decks` | List all decks grouped by faction |
| `list_bots` | List AI opponent types with descriptions |

### Game Control

| Tool | Parameters | Description |
|------|------------|-------------|
| `start_game` | `player_deck`, `opponent_deck`, `bot_type?`, `seed?` | Start a new game |
| `show_state` | - | Display current board (ASCII) |
| `show_hand` | - | Display cards in hand |
| `legal_actions` | - | List all legal moves with indices |
| `play_action` | `action_index` | Execute an action |
| `end_game` | - | End current session |

### AI Analysis

| Tool | Parameters | Description |
|------|------------|-------------|
| `ai_hint` | `use_mcts?`, `depth?`, `simulations?` | Get AI recommendation |

**Alpha-Beta (default):**
- Fast, deterministic analysis
- Configurable depth (default: 6)
- Returns ranked moves with scores and confidence

**MCTS (optional):**
- `use_mcts: true`
- Configurable simulations (default: 500)
- Better for deep tactical positions

### Score Interpretation

| Score | Meaning |
|-------|---------|
| > +200 | Winning position |
| +50 to +200 | Significant advantage |
| -50 to +50 | Roughly even |
| < -50 | Disadvantage |
| ±10000 | Forced win/loss |

### Education

| Tool | Parameters | Description |
|------|------------|-------------|
| `explain_rules` | `topic?` | Explain game rules (overview, turn, combat, etc.) |
| `explain_keywords` | `keyword?` | Explain keywords (Rush, Guard, Lethal, etc.) |
| `explain_card` | `card_id` | Get detailed card information |

### UI Sync

| Tool | Description |
|------|-------------|
| `sync_ui_state` | Push game state to Tauri desktop app |

## Typical Session

```
1. list_decks           → See available decks
2. list_bots            → See AI options
3. start_game           → Begin game
4. show_hand            → See your cards
5. legal_actions        → List possible moves
6. ai_hint              → Get recommendation
7. play_action 12       → Execute move
8. [repeat 4-7]
9. end_game             → Show final result
```

## Architecture

```
src/
├── main.rs           # Entry point
├── mcp.rs            # JSON-RPC protocol + tool dispatch
├── session.rs        # Game session management
├── tools/
│   ├── game.rs       # Game control tools
│   ├── ai.rs         # AI analysis
│   ├── discovery.rs  # Deck/bot listing
│   ├── explain.rs    # Educational tools
│   └── ui_sync.rs    # Tauri sync
└── ascii/
    ├── board.rs      # Board rendering
    ├── hand.rs       # Hand rendering
    └── card.rs       # Card formatting
```

## UI Sync

The MCP server can optionally sync game state to the Tauri desktop app:

```bash
# Start Tauri app first
cd crates/essence-wars-ui && pnpm tauri:dev

# Or use the launch script
./scripts/launch-ui.sh

# Health check
curl http://127.0.0.1:9999/health
```

Game state syncs automatically after `start_game` and `play_action`. Use `sync_ui_state` for manual sync.

**Note:** UI sync is non-fatal. If Tauri isn't running, the game continues normally.

## Protocol Details

### Transport

- **Protocol**: JSON-RPC 2.0 over stdio
- **Encoding**: UTF-8, one JSON object per line

### Request Flow

```
stdin  → Parse JSON-RPC request
       → Route to handler (initialize, tools/list, tools/call)
       → Execute tool
stdout ← JSON-RPC response
```

### Testing

```bash
# List available tools
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' | \
  ./target/release/essence-wars-mcp

# Start a game
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"start_game","arguments":{"player_deck":"iron_wall","opponent_deck":"swarm_aggro"}}}' | \
  ./target/release/essence-wars-mcp
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `cardgame` | Core game engine |
| `serde_json` | JSON serialization |
| `parking_lot` | Thread-safe state |
| `uuid` | Session IDs |
| `ureq` | HTTP client for UI sync |

## Card ID Ranges

| Range | Faction |
|-------|---------|
| 1000-1074 | Argentum Combine |
| 2000-2074 | Symbiote Circles |
| 3000-3074 | Obsidion Syndicate |
| 4000-4074 | Neutral |
| 5000-5011 | Commanders |

## License

MIT License - see [LICENSE](../../LICENSE) for details.
