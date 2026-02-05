# essence-wars-mcp - MCP Server CLAUDE.md

<!-- Last verified: 2026-02-05 -->

## Overview

MCP (Model Context Protocol) server enabling Claude Code to play Essence Wars interactively.

## Architecture

```
Claude Code (MCP Client)
    │
    │ JSON-RPC (stdio)
    ▼
essence-wars-mcp (MCP Server)
    │
    │ HTTP sync → POST /sync_state (optional)
    ▼
essence-wars-ui (Tauri App)
```

## MCP Tools

| Tool | Description |
|------|-------------|
| `list_decks` | List available decks grouped by faction |
| `list_bots` | List AI opponent types |
| `start_game` | Start new game (player_deck, opponent_deck, bot_type, seed) |
| `show_state` | Display current board state |
| `show_hand` | Display cards in hand |
| `legal_actions` | List all legal moves with indices |
| `play_action` | Execute action by index |
| `ai_hint` | Get MCTS analysis and recommended move |
| `end_game` | End current game session |
| `sync_ui_state` | Push game state to Tauri UI (if running) |

## Typical Session

1. `list_decks` → choose decks
2. `start_game` → begins game
3. `show_hand` / `legal_actions` → see options
4. `play_action` → make moves
5. `end_game` when done

## UI Sync (Optional)

If Tauri UI is running, sync game state to it:

```bash
# Start the UI
./scripts/launch-ui.sh

# Health check
curl http://127.0.0.1:9999/health
```

Use `sync_ui_state` tool to push current game state to the UI.

## Configuration

MCP server configured in `.mcp.json` at repository root.
