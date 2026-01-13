# Essence Wars TUI - Design Specification

## Overview

A full-featured Terminal User Interface (TUI) for Essence Wars research workflows, built with `ratatui`. Dark theme, keyboard-driven, modular architecture.

---

## Technical Stack

### Core Dependencies
```toml
[dependencies]
# TUI framework
ratatui = "0.28"           # Modern terminal UI framework
crossterm = "0.28"         # Terminal manipulation (backend for ratatui)

# Existing deps
tokio = { version = "1", features = ["full"] }  # Async runtime for background tasks

# UI Components
tui-input = "0.10"         # Text input widget
tui-textarea = "0.6"       # Multi-line text editing
unicode-width = "0.1"      # Text width calculations

# Utility
chrono = "0.4"             # Already in project - time formatting
colored = "2"              # Already in project - terminal colors (for debug)
```

### Architecture Pattern

**Elm Architecture** (unidirectional data flow):
```
User Input → Message → Update State → Render UI
     ↑                                    ↓
     └────────────────────────────────────┘
```

This makes the app predictable, testable, and easy to extend.

---

## Application Architecture

### Module Structure

```
src/tui/
├── mod.rs                  # Public API, re-exports
├── app.rs                  # Main App state & update logic
├── ui.rs                   # Rendering coordinator
├── events.rs               # Event handling (keyboard, timers)
├── theme.rs                # Color scheme & styling
├── state.rs                # Shared state types
│
├── screens/                # Individual screen implementations
│   ├── mod.rs
│   ├── home.rs             # Main menu
│   ├── arena.rs            # Arena match builder
│   ├── tuning.rs           # Tuning wizard & progress
│   ├── analysis.rs         # Experiment analysis viewer
│   ├── weights.rs          # Weight manager
│   └── help.rs             # Help/documentation viewer
│
├── widgets/                # Reusable UI components
│   ├── mod.rs
│   ├── menu.rs             # Selectable menu list
│   ├── progress.rs         # Progress bars with ETA
│   ├── table.rs            # Data table with scrolling
│   ├── log_viewer.rs       # Scrollable log display
│   ├── status_bar.rs       # Bottom status bar
│   └── dialog.rs           # Modal dialogs
│
└── tasks/                  # Background task management
    ├── mod.rs
    ├── arena_task.rs       # Run arena matches
    ├── tuning_task.rs      # Run tuning experiments
    └── analysis_task.rs    # Parse logs & generate stats
```

### State Management

```rust
// src/tui/app.rs
pub struct App {
    /// Current active screen
    screen: Screen,
    
    /// Navigation history for back button
    history: Vec<Screen>,
    
    /// Shared application state
    state: AppState,
    
    /// Background task handle (if any)
    task: Option<TaskHandle>,
    
    /// Should quit?
    should_quit: bool,
}

pub enum Screen {
    Home(HomeScreen),
    Arena(ArenaScreen),
    Tuning(TuningScreen),
    Analysis(AnalysisScreen),
    Weights(WeightsScreen),
    Help(HelpScreen),
}

pub struct AppState {
    /// Card database (shared)
    card_db: Arc<CardDatabase>,
    
    /// Available decks
    decks: Vec<DeckInfo>,
    
    /// Available weight files
    weights: Vec<WeightInfo>,
    
    /// Recent experiments
    experiments: Vec<ExperimentInfo>,
    
    /// Current working directory
    cwd: PathBuf,
}
```

### Message-Driven Updates

```rust
pub enum Message {
    // Navigation
    Navigate(Screen),
    GoBack,
    Quit,
    
    // Input events
    KeyPress(KeyEvent),
    
    // Arena
    ArenaStartMatch(ArenaConfig),
    ArenaProgress(u32, u32),  // (current, total)
    ArenaComplete(ArenaResult),
    
    // Tuning
    TuningStart(TuningConfig),
    TuningProgress(TuningProgress),
    TuningComplete(TuningResult),
    
    // Analysis
    AnalysisLoad(PathBuf),
    AnalysisReady(AnalysisData),
    
    // Weights
    WeightsPromote(PathBuf),
    WeightsCompare(PathBuf, PathBuf),
    
    // Background tasks
    TaskUpdate(TaskMessage),
    TaskError(String),
}

impl App {
    pub fn update(&mut self, msg: Message) -> Result<()> {
        match msg {
            Message::Navigate(screen) => {
                self.history.push(self.screen.clone());
                self.screen = screen;
            }
            Message::ArenaStartMatch(config) => {
                self.task = Some(spawn_arena_task(config));
            }
            // ... handle all message types
        }
        Ok(())
    }
}
```

---

## UI Design

### Color Theme (Dark)

```rust
// src/tui/theme.rs
pub struct Theme {
    // Base colors
    pub bg: Color,                  // #1e1e2e (dark gray-blue)
    pub bg_alt: Color,              // #313244 (lighter panel)
    pub fg: Color,                  // #cdd6f4 (light text)
    pub fg_dim: Color,              // #6c7086 (dimmed text)
    
    // Accent colors
    pub primary: Color,             // #89b4fa (blue - for selections)
    pub success: Color,             // #a6e3a1 (green - for wins)
    pub warning: Color,             // #f9e2af (yellow - for warnings)
    pub error: Color,               // #f38ba8 (red - for errors)
    pub info: Color,                // #94e2d5 (cyan - for info)
    
    // Border colors
    pub border: Color,              // #45475a (border)
    pub border_focused: Color,      // #89b4fa (focused border)
    
    // Chart colors
    pub chart_line1: Color,         // #f38ba8 (red)
    pub chart_line2: Color,         // #89b4fa (blue)
    pub chart_line3: Color,         // #a6e3a1 (green)
}
```

Uses **Catppuccin Mocha** palette - professional, accessible, widely loved.

### Screen Layouts

#### 1. Home Screen
```
┌─ Essence Wars Research Lab ──────────────────────────────── v0.1.0 ─┐
│                                                                       │
│  What would you like to do?                                          │
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │ > 🎮  Arena Match        Run bot vs bot matches            │    │
│  │   🎲  Tune Weights       Optimize bot parameters           │    │
│  │   📊  Analysis           View experiment results           │    │
│  │   ⚖️   Weight Manager     Manage weight configurations      │    │
│  │   📖  Documentation      View guides and references        │    │
│  │   🚪  Exit               Quit application                  │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                       │
│  Recent Activity:                                                    │
│  • Tuning: 2026-01-13_1430_vs_mcts (completed)                      │
│  • Arena: Greedy vs MCTS - 1000 games (78% WR)                      │
│                                                                       │
├───────────────────────────────────────────────────────────────────────┤
│ [↑↓] Navigate  [Enter] Select  [Q] Quit  [?] Help                   │
└───────────────────────────────────────────────────────────────────────┘
```

#### 2. Arena Screen
```
┌─ Arena Match Builder ─────────────────────────────────────────────────┐
│                                                                        │
│  Player 1:                      Player 2:                             │
│  ┌──────────────────────────┐   ┌──────────────────────────┐         │
│  │ Bot:    [Greedy ▼]       │   │ Bot:    [MCTS ▼]         │         │
│  │ Deck:   [Aggro  ▼]       │   │ Deck:   [Control ▼]      │         │
│  │ Weights:[default ▼]      │   │ Weights:[default ▼]      │         │
│  └──────────────────────────┘   └──────────────────────────┘         │
│                                                                        │
│  Match Settings:                                                      │
│  ┌────────────────────────────────────────────────────────┐           │
│  │ Games:     [1000        ]                              │           │
│  │ Seed:      [42          ] (empty for random)           │           │
│  │ Debug:     [ ] Enable detailed logging                 │           │
│  │ Progress:  [✓] Show live progress bar                  │           │
│  └────────────────────────────────────────────────────────┘           │
│                                                                        │
│  [ Start Match ]  [ Save Config ]  [ Back ]                           │
│                                                                        │
├────────────────────────────────────────────────────────────────────────┤
│ [Tab] Next field  [Shift+Tab] Previous  [Enter] Activate  [Esc] Back │
└────────────────────────────────────────────────────────────────────────┘
```

#### 3. Tuning Screen (Active)
```
┌─ Tuning: 2026-01-13_1523_multi_opponent ──────────────────────────────┐
│                                                                        │
│  Mode: Multi-Opponent  │  Generations: 50  │  Games/Eval: 200         │
│                                                                        │
│  Progress:                                                             │
│  Generation 23/50 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━░░░░░░░ 46%    │
│  Games      4600/10000 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━░░░░░░░░░ 46%    │
│  Time Elapsed: 12m 34s  │  ETA: 14m 11s                               │
│                                                                        │
│  Current Generation Stats:                                            │
│  ┌────────────────────────────────────────────────────────────────┐   │
│  │ Best Fitness:     0.782  (↑ +0.018 from gen 22)               │   │
│  │ Mean Fitness:     0.654  (↑ +0.012)                            │   │
│  │ Convergence:      3.2%   (std deviation)                       │   │
│  │ Win Rate:         78.2%  vs Random                             │   │
│  │                   64.5%  vs Greedy                             │   │
│  │                   58.1%  vs MCTS                               │   │
│  └────────────────────────────────────────────────────────────────┘   │
│                                                                        │
│  Fitness History:                                                      │
│   1.0 ┤                                          ╭─────               │
│   0.8 ┤                               ╭──────────╯                    │
│   0.6 ┤                    ╭──────────╯                               │
│   0.4 ┤          ╭─────────╯                                          │
│   0.2 ┤ ─────────╯                                                    │
│   0.0 └┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┴──    │
│         0    5    10    15    20    25    30    35    40    45   50  │
│                                                                        │
├────────────────────────────────────────────────────────────────────────┤
│ [Space] Pause  [S] Save checkpoint  [Q] Stop & save  [L] View log    │
└────────────────────────────────────────────────────────────────────────┘
```

#### 4. Weight Manager
```
┌─ Weight Manager ──────────────────────────────────────────────────────┐
│                                                                        │
│  Available Weights:                                     Sort: [Name▼] │
│  ┌────────────────────────────────────────────────────────────────┐   │
│  │ > default.toml                             [ACTIVE]   157 KB   │   │
│  │   tuned_multi_opponent.toml                           156 KB   │   │
│  │   tuned_multi_opponent_long.toml                      156 KB   │   │
│  │   experiments/mcts/.../weights.toml                   155 KB   │   │
│  └────────────────────────────────────────────────────────────────┘   │
│                                                                        │
│  Preview: default.toml                                                │
│  ┌────────────────────────────────────────────────────────────────┐   │
│  │ Name:    tuned_multi-opponent                                  │   │
│  │ Version: 1                                                     │   │
│  │                                                                │   │
│  │ Key Weights:                                                   │   │
│  │   own_life:            2.10  ░░░░░░░░░░░░░░░░░░░░░░ (vs 1.5)  │   │
│  │   enemy_life_damage:   2.03  ░░░░░░░░░░░░░░░░░░░░░  (vs 2.0)  │   │
│  │   creature_count:      1.48  ░░░░░░░░░░░░░░░        (vs 3.0)  │   │
│  │   keyword_guard:       5.00  ░░░░░░░░░░░░░░░░░░░░░░ (vs 3.0)  │   │
│  │   keyword_lethal:      2.62  ░░░░░░░░░░░░░░░        (vs 4.0)  │   │
│  │   ... (15 more)                                                │   │
│  └────────────────────────────────────────────────────────────────┘   │
│                                                                        │
│  Actions: [P]romote to default  [C]ompare  [V]iew full  [D]elete     │
│                                                                        │
├────────────────────────────────────────────────────────────────────────┤
│ [↑↓] Navigate  [Enter] Select  [Esc] Back                            │
└────────────────────────────────────────────────────────────────────────┘
```

---

## Keyboard Navigation

### Global Hotkeys (work on all screens)
- `Q` - Quit (with confirmation if task running)
- `Esc` - Go back / Cancel
- `?` - Show help overlay
- `Tab` / `Shift+Tab` - Focus next/previous widget

### Screen-Specific
- **Home**: `↑↓` navigate, `Enter` select, `1-6` quick jump
- **Arena**: `Tab` cycle fields, `Space` toggle checkboxes, `Enter` activate
- **Tuning**: `Space` pause/resume, `S` save checkpoint, `L` view logs
- **Analysis**: `↑↓` scroll, `PgUp/PgDn` page, `Tab` switch plots
- **Weights**: `↑↓` navigate, `P` promote, `C` compare, `V` view, `D` delete

---

## File Organization

```
src/
├── bin/
│   ├── arena.rs           # Existing CLI
│   ├── tune.rs            # Existing CLI
│   ├── profile_mcts.rs    # Existing
│   └── lab.rs             # NEW: TUI entry point
│
├── tui/
│   ├── mod.rs             # Public API
│   ├── app.rs             # App state & message handling
│   ├── ui.rs              # Main render coordinator
│   ├── events.rs          # Event loop & input handling
│   ├── theme.rs           # Colors & styles
│   ├── state.rs           # Shared state types
│   │
│   ├── screens/           # Individual screens
│   │   ├── mod.rs
│   │   ├── home.rs
│   │   ├── arena.rs
│   │   ├── tuning.rs
│   │   ├── analysis.rs
│   │   ├── weights.rs
│   │   └── help.rs
│   │
│   ├── widgets/           # Reusable components
│   │   ├── mod.rs
│   │   ├── menu.rs
│   │   ├── progress.rs
│   │   ├── table.rs
│   │   ├── log_viewer.rs
│   │   ├── status_bar.rs
│   │   └── dialog.rs
│   │
│   └── tasks/             # Background operations
│       ├── mod.rs
│       ├── arena_task.rs
│       ├── tuning_task.rs
│       └── analysis_task.rs
│
└── lib.rs                 # Expose tui module
```

---

## Implementation Phases

### Phase 1: Foundation (Core TUI)
**Files**: `app.rs`, `ui.rs`, `events.rs`, `theme.rs`, `state.rs`
- [ ] Basic TUI setup with ratatui + crossterm
- [ ] Event loop with keyboard handling
- [ ] Message-driven architecture
- [ ] Theme system with dark colors
- [ ] Screen navigation framework

### Phase 2: Home & Navigation
**Files**: `screens/home.rs`, `screens/help.rs`, `widgets/menu.rs`
- [ ] Home screen with menu
- [ ] Help overlay
- [ ] Navigation between screens
- [ ] Status bar widget

### Phase 3: Arena Screen
**Files**: `screens/arena.rs`, `tasks/arena_task.rs`, `widgets/progress.rs`
- [ ] Arena configuration UI
- [ ] Form inputs (dropdowns, text fields)
- [ ] Background arena task execution
- [ ] Live progress display
- [ ] Results screen

### Phase 4: Tuning Screen
**Files**: `screens/tuning.rs`, `tasks/tuning_task.rs`
- [ ] Tuning wizard/config UI
- [ ] Background CMA-ES execution
- [ ] Live metrics display
- [ ] Line chart widget for fitness history
- [ ] Pause/resume/stop controls

### Phase 5: Analysis & Weights
**Files**: `screens/analysis.rs`, `screens/weights.rs`, `tasks/analysis_task.rs`
- [ ] Experiment browser
- [ ] Parse and display stats
- [ ] Weight file browser
- [ ] Weight diff viewer
- [ ] Promote weights action

### Phase 6: Polish
- [ ] Error handling & user feedback
- [ ] Confirmation dialogs
- [ ] Loading states
- [ ] Keyboard shortcuts help
- [ ] Config persistence (last used values)

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    // Test message handling
    #[test]
    fn test_navigate_updates_history() { }
    
    // Test state transitions
    #[test]
    fn test_arena_start_spawns_task() { }
    
    // Test widget rendering (snapshot tests)
    #[test]
    fn test_menu_renders_correctly() { }
}
```

### Integration Tests
```bash
# Manual testing checklist (tests/tui_manual_test.md)
- [ ] Can navigate all screens
- [ ] Arena starts and completes successfully
- [ ] Tuning shows progress updates
- [ ] Weights can be promoted
- [ ] Quit works from all screens
```

---

## Open Questions

1. **Async vs Sync Tasks?**
   - Use `tokio` channels for background tasks?
   - Or simple threads with `mpsc::channel`?
   - **Recommendation**: Start with threads, add tokio if needed

2. **Chart Library?**
   - `tui-rs-tree-widget` for simple sparklines?
   - Custom ASCII art charts?
   - **Recommendation**: Custom - more control, lighter

3. **Config Persistence?**
   - Save last-used arena config to `~/.config/essence-wars/lab.toml`?
   - **Recommendation**: Phase 6 feature, nice-to-have

4. **Log Viewer?**
   - Live tail of tuning logs?
   - **Recommendation**: Yes, use `widgets/log_viewer.rs`

---

## Next Steps

1. **Review this design** - Any changes needed?
2. **Add dependencies to Cargo.toml**
3. **Create module structure** (empty files with TODOs)
4. **Implement Phase 1** (Foundation)
5. **Iterate on remaining phases**

---

## References

- [ratatui docs](https://ratatui.rs/)
- [Catppuccin Mocha palette](https://github.com/catppuccin/catppuccin)
- [The Elm Architecture](https://guide.elm-lang.org/architecture/)
