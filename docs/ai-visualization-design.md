# AI Visualization Design Document

**Status:** Approved
**Author:** Claude (with Chris)
**Created:** 2026-02-04
**Updated:** 2026-02-04
**Version:** 0.2.0

## Executive Summary

This document describes the design for AI decision-making visualizations in Essence Wars' Spectator Mode. The goal is to create an "X-Ray Mode" that reveals how AI agents think, making the game both visually compelling and genuinely insightful for ML/AI researchers and engineers.

---

## Table of Contents

1. [Goals and Non-Goals](#1-goals-and-non-goals)
2. [Target Audience](#2-target-audience)
3. [Use Cases](#3-use-cases)
4. [Visualization Layers](#4-visualization-layers)
5. [Bot-Specific Visualizations](#5-bot-specific-visualizations)
6. [Architecture](#6-architecture)
7. [Data Structures](#7-data-structures)
8. [API Design](#8-api-design)
9. [UI Components](#9-ui-components)
10. [Implementation Phases](#10-implementation-phases)
11. [Future: Neural Agent Support](#11-future-neural-agent-support)
12. [Open Questions](#12-open-questions)

---

## 1. Goals and Non-Goals

### Goals

1. **Insight over Flash**: Visualizations should provide genuine understanding of AI decision-making, not just eye candy
2. **Researcher-Friendly**: Enable ML/AI researchers to understand, debug, and compare agent behaviors
3. **Visually Compelling**: Look good enough for demo videos, GIFs, and social media sharing
4. **Extensible**: Architecture must support future neural network agents (policy/value heads, attention maps)
5. **Non-Intrusive**: Visualizations should enhance, not obstruct, the game viewing experience
6. **Progressive Disclosure**: Casual viewers see highlights; researchers can drill into details

### Non-Goals

1. **Real-time Training Visualization**: This is for inference/play, not training loops
2. **General-Purpose ML Dashboard**: Focused on game decision-making, not arbitrary metrics
3. **Human Player Assistance**: This is for spectating AI vs AI, not helping humans play

---

## 2. Target Audience

### Primary: ML/AI Researchers & Engineers

- **Who**: Academic researchers, hobbyist ML enthusiasts, game AI developers
- **What they want**:
  - Understand why the AI made a specific decision
  - Compare different algorithms (MCTS vs AlphaBeta vs Neural)
  - Identify weaknesses or biases in evaluation functions
  - Validate that their agent is "thinking correctly"
- **Technical comfort**: High - can interpret probability distributions, search trees, evaluation scores

### Secondary: Educators & Content Creators

- **Who**: People making videos/tutorials about game AI, MCTS, neural networks
- **What they want**:
  - Clear visual explanations of how algorithms work
  - Dramatic moments ("the AI just found a winning line!")
  - Shareable screenshots and clips

### Tertiary: Curious Spectators

- **Who**: People watching AI vs AI games for entertainment
- **What they want**:
  - See that "something interesting" is happening
  - Understand which side is winning and why
  - Not be overwhelmed by technical details

---

## 3. Use Cases

### UC1: Debugging a Bot

**Scenario**: An engineer notices their MCTS bot makes a seemingly bad move.

**Visualization needs**:
- See all moves considered and their visit counts
- See win rate estimates for each move
- Drill into the search tree to understand why a bad move got high visits
- See the evaluation breakdown at leaf nodes

### UC2: Comparing Algorithms

**Scenario**: A researcher wants to understand how MCTS and AlphaBeta differ on the same position.

**Visualization needs**:
- Side-by-side move rankings from each algorithm
- Different confidence/certainty representations
- Search statistics (nodes, depth, time)

### UC3: Demo Video

**Scenario**: Chris wants to create a 30-second GIF showing the AI "thinking".

**Visualization needs**:
- Animated move candidates appearing on the board
- Probability bars that update as search progresses
- Clean, readable overlays that look good at low resolution

### UC4: Educational Content

**Scenario**: Someone is learning how MCTS works and wants to see it in action.

**Visualization needs**:
- Step-by-step tree expansion (slow mode)
- Clear labeling of UCB scores, visit counts, win rates
- Ability to pause and explore the tree

---

## 4. Visualization Layers

The visualization system has three progressive layers, from least to most detailed.

### Layer 1: Board Overlays (Always Visible)

Lightweight visual indicators directly on the game board.

```
┌─────────────────────────────────────────────────────────────┐
│                                                             │
│    Your Creatures:                                          │
│    ┌───────┐   ┌───────┐   ┌───────┐                       │
│    │ 3/4   │   │ 2/2   │   │ 5/3   │                       │
│    │ Guard │   │  47%  │   │  12%  │  ← Move probability   │
│    │       │   │  ▼▼▼  │   │   ▼   │    badges on sources  │
│    └───────┘   └───┬───┘   └───┬───┘                       │
│                    │           │                            │
│         ┌─────────┴───┐       │      ← Attack arrows       │
│         ▼             ▼       ▼         (thickness = prob) │
│    ┌───────┐   ┌───────┐   ┌───────┐                       │
│    │ 2/3   │   │ 4/1   │   │ FACE  │                       │
│    │Target │   │Target │   │  41%  │  ← Target highlights  │
│    │  6%   │   │  53%  │   │       │                       │
│    └───────┘   └───────┘   └───────┘                       │
│                                                             │
│    Enemy Creatures                                          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Elements**:

| Element | Description | Visual |
|---------|-------------|--------|
| **Move Probability Badge** | % chance this move is selected | Small pill badge on card |
| **Attack Arrow** | Shows attack source → target | Animated arrow, thickness ∝ probability |
| **Target Highlight** | Creature being considered as target | Glow/border effect |
| **Best Move Indicator** | Marks the chosen/best move | Star or checkmark icon |
| **Card Play Ghost** | Shows where a card might be played | Semi-transparent card preview |

**Interaction**: Hover over any indicator to see more details in tooltip.

### Layer 2: Stats Bar (Toggle-able)

Compact horizontal bar showing key metrics.

```
┌─────────────────────────────────────────────────────────────┐
│ MCTS │ Eval: +2.3 │ Nodes: 12,847 │ Depth: 8 │ Time: 1.2s │
│      │ ████████░░ │               │          │ Conf: 89%  │
│      │  P1 favored                                         │
└─────────────────────────────────────────────────────────────┘
```

**Metrics**:

| Metric | MCTS | AlphaBeta | Neural |
|--------|------|-----------|--------|
| Evaluation | Win rate % | Minimax score | Value head output |
| Search Size | Nodes visited | Nodes evaluated | N/A |
| Search Depth | Max depth reached | Fixed depth | N/A |
| Time | Think time | Think time | Inference time |
| Confidence | Visit concentration | N/A | Policy entropy |

### Layer 3: Research Sidepanel (Expandable)

Detailed analysis panel that slides out from the right side.

```
┌────────────────────────────────────────┐
│  AI Insights                      [×]  │
├────────────────────────────────────────┤
│                                        │
│  ┌──────────────────────────────────┐  │
│  │ MOVE RANKINGS                    │  │
│  ├──────────────────────────────────┤  │
│  │ 1. Attack Slot2 → Face     47%  │  │
│  │    ████████████████████░░░░░░░  │  │
│  │ 2. Play "Flame Imp" → S3   23%  │  │
│  │    ██████████░░░░░░░░░░░░░░░░░  │  │
│  │ 3. Attack Slot1 → Slot0    15%  │  │
│  │    ██████░░░░░░░░░░░░░░░░░░░░░  │  │
│  │ 4. End Turn                 8%  │  │
│  │    ███░░░░░░░░░░░░░░░░░░░░░░░░  │  │
│  │ 5. Play "Dark Rit..." → S4  7%  │  │
│  │    ██░░░░░░░░░░░░░░░░░░░░░░░░░  │  │
│  └──────────────────────────────────┘  │
│                                        │
│  ┌──────────────────────────────────┐  │
│  │ EVALUATION BREAKDOWN             │  │
│  ├──────────────────────────────────┤  │
│  │ Factor           P1    P2    Δ   │  │
│  │ ─────────────────────────────────│  │
│  │ Life             24    18  +0.6  │  │
│  │ Board Power      12     8  +0.8  │  │
│  │ Card Advantage    4     3  +0.3  │  │
│  │ Guard Presence    2     0  +0.4  │  │
│  │ Essence Progress 23    15  +0.2  │  │
│  │ ─────────────────────────────────│  │
│  │ TOTAL                      +2.3  │  │
│  └──────────────────────────────────┘  │
│                                        │
│  ┌──────────────────────────────────┐  │
│  │ SEARCH TREE          [Expand ↗]  │  │
│  ├──────────────────────────────────┤  │
│  │         [Root 52%]               │  │
│  │        /     |     \             │  │
│  │    [Atk]  [Play]  [End]          │  │
│  │    54%    48%     41%            │  │
│  │   2341    847     112  visits    │  │
│  └──────────────────────────────────┘  │
│                                        │
│  ┌──────────────────────────────────┐  │
│  │ SEARCH STATS                     │  │
│  ├──────────────────────────────────┤  │
│  │ Algorithm:    MCTS (UCB1)        │  │
│  │ Simulations:  3,300              │  │
│  │ Max Depth:    12                 │  │
│  │ Avg Depth:    7.3                │  │
│  │ Think Time:   1,247ms            │  │
│  │ Nodes/sec:    2,645              │  │
│  │ Cache Hits:   892 (27%)          │  │
│  └──────────────────────────────────┘  │
│                                        │
└────────────────────────────────────────┘
```

**Sections**:

1. **Move Rankings**: All legal moves ranked by score/probability
2. **Evaluation Breakdown**: Factor-by-factor position analysis
3. **Search Tree**: Interactive tree visualization (collapsible)
4. **Search Stats**: Performance metrics and algorithm details

---

## 5. Bot-Specific Visualizations

### 5.1 MCTS Bot

MCTS naturally provides rich visualization data through its tree structure.

**Available Data**:
- Visit counts per node → move probabilities
- Win rate estimates per node → position evaluation
- UCB scores → exploration vs exploitation balance
- Tree structure → decision process

**Unique Visualizations**:

```
MCTS Tree Node
┌─────────────────────────┐
│ Action: Attack S2→Face  │
│ ─────────────────────── │
│ Visits: 2,341 (71%)     │
│ Wins: 1,287 (55.0%)     │
│ UCB: 0.583              │
│ ─────────────────────── │
│ Children: 5             │
│ Max Depth Below: 8      │
└─────────────────────────┘
```

**Progressive Search Animation** (optional):
- Show tree growing in real-time during search
- Highlight currently expanding node
- Useful for educational/demo purposes

### 5.2 AlphaBeta Bot

AlphaBeta provides deterministic evaluation but less "probabilistic" feel.

**Available Data**:
- Best move and its evaluation score
- Evaluation at each depth
- Alpha/beta bounds (for visualization of pruning)
- Principal variation (best line found)

**Unique Visualizations**:

```
AlphaBeta Principal Variation
┌────────────────────────────────────────┐
│ Depth 6 Search - Score: +2.3          │
│                                        │
│ Move 1: Attack Slot2 → Face  (+1.8)   │
│   └─ Opp: Play "Guard" → S1  (+1.5)   │
│       └─ Us: Attack S1 → S1  (+2.0)   │
│           └─ Opp: Attack S0  (+1.8)   │
│               └─ Us: Play... (+2.3)   │
│                   └─ [depth limit]    │
└────────────────────────────────────────┘
```

**Pruning Visualization** (advanced):
- Show which branches were pruned
- Indicate efficiency: "Evaluated 12,847 of 89,000 possible nodes (14%)"

### 5.3 Greedy Bot

Greedy bot is simple but can still show evaluation breakdown.

**Available Data**:
- Score for each legal move
- Evaluation factor contributions

**Visualization**:
- Move ranking with absolute scores (not probabilities)
- Evaluation breakdown showing weight contributions

### 5.4 Neural Agent (Future)

Neural agents have different but equally interesting visualization potential.

**Available Data**:
- Policy head output: probability distribution over moves
- Value head output: single position evaluation
- (Optional) Attention weights: where the model "looks"
- (Optional) Embedding: latent representation of game state

**Unique Visualizations**:

```
Neural Policy Distribution
┌────────────────────────────────────────┐
│ Policy Head Output (softmax)           │
│                                        │
│ Attack S2→Face  ████████████████  34%  │
│ Play Card #3    ████████          21%  │
│ Attack S1→S0    ██████            15%  │
│ Play Card #1    █████             11%  │
│ End Turn        ███                8%  │
│ [12 more]       ████              11%  │
│                                        │
│ Entropy: 2.31 (moderate confidence)    │
├────────────────────────────────────────┤
│ Value Head: 0.67 → P1 67% to win      │
└────────────────────────────────────────┘
```

**Attention Map** (if transformer-based):
```
Card Attention Weights
┌──────────────────────────────────────┐
│ Query: Move decision                 │
│                                      │
│ Our Cards:    [0.8] [0.2] [0.1] [·]  │
│ Our Board:    [0.9] [0.7] [0.3] [·]  │
│ Their Board:  [0.6] [0.4] [0.2] [·]  │
│ Commander:    [0.5] [0.3]            │
│                                      │
│ High attention = model "focuses" on  │
└──────────────────────────────────────┘
```

---

## 6. Architecture

### 6.1 High-Level Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                           RUST CORE                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │
│   │  MctsBot    │  │AlphaBetaBot │  │ NeuralAgent │               │
│   │             │  │             │  │  (future)   │               │
│   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘               │
│          │                │                │                       │
│          ▼                ▼                ▼                       │
│   ┌─────────────────────────────────────────────────────────┐     │
│   │              trait BotIntrospection                     │     │
│   │  ──────────────────────────────────────────────────────│     │
│   │  fn get_last_decision() -> Option<DecisionInsights>    │     │
│   │  fn get_move_scores() -> Vec<MoveScore>                │     │
│   │  fn get_eval_breakdown() -> Option<EvalBreakdown>      │     │
│   │  fn get_search_stats() -> Option<SearchStats>          │     │
│   │  fn get_tree_snapshot(depth: u8) -> Option<TreeNode>   │     │
│   │  fn supports_introspection() -> IntrospectionCaps      │     │
│   └─────────────────────────────────────────────────────────┘     │
│                              │                                     │
│                              ▼                                     │
│   ┌─────────────────────────────────────────────────────────┐     │
│   │                  IntrospectionCache                     │     │
│   │  ────────────────────────────────────────────────────── │     │
│   │  Stores last decision data for UI retrieval             │     │
│   │  Thread-safe (Arc<RwLock<...>>)                        │     │
│   └─────────────────────────────────────────────────────────┘     │
│                              │                                     │
└──────────────────────────────┼──────────────────────────────────────┘
                               │
                               │ Tauri Commands (IPC)
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         TAURI BRIDGE                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│   #[tauri::command]                                                │
│   fn get_ai_insights(player: PlayerId) -> AIInsights              │
│                                                                     │
│   #[tauri::command]                                                │
│   fn get_search_tree(player: PlayerId, depth: u8) -> TreeSnapshot │
│                                                                     │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               │ JSON over IPC
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        SVELTE UI                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│   ┌─────────────────────────────────────────────────────────┐     │
│   │                 aiInsightsStore.svelte.ts               │     │
│   │  ──────────────────────────────────────────────────────│     │
│   │  $state: AIInsights | null                             │     │
│   │  $state: TreeSnapshot | null                           │     │
│   │  $state: isLoading                                     │     │
│   │  ──────────────────────────────────────────────────────│     │
│   │  async fetchInsights(player)                           │     │
│   │  async fetchTree(player, depth)                        │     │
│   │  clearInsights()                                       │     │
│   └─────────────────────────────────────────────────────────┘     │
│                              │                                     │
│          ┌───────────────────┼───────────────────┐                │
│          ▼                   ▼                   ▼                │
│   ┌────────────┐    ┌──────────────┐    ┌──────────────┐        │
│   │BoardOverlay│    │  StatsBar    │    │ResearchPanel │        │
│   │            │    │              │    │              │        │
│   │-MoveArrows │    │-EvalMeter    │    │-MoveRankings │        │
│   │-ProbBadges │    │-SearchStats  │    │-EvalBreakdown│        │
│   │-Highlights │    │-Confidence   │    │-TreeView     │        │
│   └────────────┘    └──────────────┘    └──────────────┘        │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 6.2 Key Design Decisions

**1. Pull-based, not Push-based**

The UI requests insights after each move, rather than the bot pushing updates continuously.

Rationale:
- Simpler architecture
- No need for streaming/websockets
- Bot performance not impacted during search
- UI controls refresh rate

**2. Introspection is Optional**

Bots implement `BotIntrospection` trait optionally. A bot that doesn't support it returns `None`.

```rust
pub trait BotIntrospection {
    fn supports_introspection(&self) -> IntrospectionCaps {
        IntrospectionCaps::none()  // Default: no introspection
    }

    fn get_last_decision(&self) -> Option<DecisionInsights> {
        None  // Default implementation
    }
    // ... etc
}
```

**3. Cached Last Decision**

Bots cache their most recent decision data. The UI can query it anytime.

```rust
pub struct MctsBot {
    config: MctsConfig,
    weights: BotWeights,
    last_decision: Option<MctsDecisionData>,  // Cached for introspection
}
```

**4. Depth-Limited Tree Snapshots**

Full search trees can be huge. UI requests trees with a depth limit.

```rust
fn get_tree_snapshot(&self, max_depth: u8) -> Option<TreeNode> {
    // Returns tree pruned to max_depth levels
    // Deeper nodes summarized as "N children, best: X%"
}
```

---

## 7. Data Structures

### 7.1 Rust Types

```rust
// crates/cardgame/src/bots/introspection.rs

use serde::{Deserialize, Serialize};
use crate::core::Action;

/// Capabilities a bot supports for introspection
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct IntrospectionCaps {
    pub move_scores: bool,
    pub eval_breakdown: bool,
    pub search_stats: bool,
    pub tree_snapshot: bool,
    pub attention_weights: bool,  // Future: neural agents
}

impl IntrospectionCaps {
    pub fn none() -> Self {
        Self {
            move_scores: false,
            eval_breakdown: false,
            search_stats: false,
            tree_snapshot: false,
            attention_weights: false,
        }
    }

    pub fn mcts() -> Self {
        Self {
            move_scores: true,
            eval_breakdown: true,
            search_stats: true,
            tree_snapshot: true,
            attention_weights: false,
        }
    }

    pub fn alphabeta() -> Self {
        Self {
            move_scores: true,
            eval_breakdown: true,
            search_stats: true,
            tree_snapshot: true,  // Principal variation
            attention_weights: false,
        }
    }

    pub fn neural() -> Self {
        Self {
            move_scores: true,  // Policy output
            eval_breakdown: false,  // Black box
            search_stats: false,
            tree_snapshot: false,
            attention_weights: true,
        }
    }
}

/// Score/probability for a single move
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveScore {
    pub action: Action,
    pub action_index: u8,
    pub score: f32,           // Raw score (interpretation varies by bot)
    pub probability: f32,     // Normalized 0-1 (visits% for MCTS, softmax for neural)
    pub visits: Option<u32>,  // MCTS only
    pub depth: Option<u8>,    // AlphaBeta only
}

/// Breakdown of position evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalBreakdown {
    pub total_score: f32,
    pub factors: Vec<EvalFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalFactor {
    pub name: String,
    pub player1_value: f32,
    pub player2_value: f32,
    pub weight: f32,
    pub contribution: f32,  // weighted delta
}

/// Search performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    pub algorithm: String,
    pub nodes_evaluated: u64,
    pub max_depth: u8,
    pub avg_depth: f32,
    pub time_ms: u64,
    pub nodes_per_sec: f32,
    pub cache_hits: Option<u64>,
    pub cache_hit_rate: Option<f32>,
    pub early_terminations: Option<u64>,  // MCTS optimization
}

/// A node in the search tree (for visualization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub action: Option<Action>,  // None for root
    pub action_str: String,      // Human-readable action description
    pub visits: u32,
    pub score: f32,              // Win rate (MCTS) or eval (AlphaBeta)
    pub children: Vec<TreeNode>,
    pub is_truncated: bool,      // True if children were cut off by depth limit
    pub truncated_child_count: u32,
}

/// Complete decision insights package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionInsights {
    pub chosen_action: Action,
    pub chosen_action_str: String,
    pub move_scores: Vec<MoveScore>,
    pub eval_breakdown: Option<EvalBreakdown>,
    pub search_stats: Option<SearchStats>,
    pub tree_root: Option<TreeNode>,
}

/// Trait for bots that support introspection
pub trait BotIntrospection {
    fn supports_introspection(&self) -> IntrospectionCaps {
        IntrospectionCaps::none()
    }

    fn get_last_decision(&self) -> Option<DecisionInsights> {
        None
    }

    fn get_tree_snapshot(&self, max_depth: u8) -> Option<TreeNode> {
        None
    }
}
```

### 7.2 TypeScript Types

```typescript
// src/lib/types/aiInsights.ts

export interface IntrospectionCaps {
  moveScores: boolean;
  evalBreakdown: boolean;
  searchStats: boolean;
  treeSnapshot: boolean;
  attentionWeights: boolean;
}

export interface MoveScore {
  action: Action;
  actionIndex: number;
  score: number;
  probability: number;
  visits?: number;
  depth?: number;
}

export interface EvalFactor {
  name: string;
  player1Value: number;
  player2Value: number;
  weight: number;
  contribution: number;
}

export interface EvalBreakdown {
  totalScore: number;
  factors: EvalFactor[];
}

export interface SearchStats {
  algorithm: string;
  nodesEvaluated: number;
  maxDepth: number;
  avgDepth: number;
  timeMs: number;
  nodesPerSec: number;
  cacheHits?: number;
  cacheHitRate?: number;
  earlyTerminations?: number;
}

export interface TreeNode {
  action?: Action;
  actionStr: string;
  visits: number;
  score: number;
  children: TreeNode[];
  isTruncated: boolean;
  truncatedChildCount: number;
}

export interface DecisionInsights {
  chosenAction: Action;
  chosenActionStr: string;
  moveScores: MoveScore[];
  evalBreakdown?: EvalBreakdown;
  searchStats?: SearchStats;
  treeRoot?: TreeNode;
}

export interface AIInsights {
  player1?: DecisionInsights;
  player2?: DecisionInsights;
  capabilities: {
    player1: IntrospectionCaps;
    player2: IntrospectionCaps;
  };
}
```

---

## 8. API Design

### 8.1 Rust Introspection Trait

```rust
// Extend existing Bot trait or create separate trait

pub trait BotIntrospection: Bot {
    /// What introspection features this bot supports
    fn introspection_caps(&self) -> IntrospectionCaps {
        IntrospectionCaps::none()
    }

    /// Get insights from the last decision made
    fn last_decision(&self) -> Option<&DecisionInsights>;

    /// Get a depth-limited snapshot of the search tree
    fn tree_snapshot(&self, max_depth: u8) -> Option<TreeNode> {
        self.last_decision()
            .and_then(|d| d.tree_root.as_ref())
            .map(|tree| prune_tree(tree, max_depth))
    }
}
```

### 8.2 Tauri Commands

```rust
// crates/essence-wars-ui/src-tauri/src/commands/ai_insights.rs

use tauri::State;
use crate::GameState;

#[tauri::command]
pub async fn get_ai_insights(
    state: State<'_, GameState>,
) -> Result<AIInsights, String> {
    let game = state.game.read().await;

    Ok(AIInsights {
        player1: game.bot1.as_ref()
            .and_then(|b| b.last_decision().cloned()),
        player2: game.bot2.as_ref()
            .and_then(|b| b.last_decision().cloned()),
        capabilities: Capabilities {
            player1: game.bot1.as_ref()
                .map(|b| b.introspection_caps())
                .unwrap_or_default(),
            player2: game.bot2.as_ref()
                .map(|b| b.introspection_caps())
                .unwrap_or_default(),
        },
    })
}

#[tauri::command]
pub async fn get_search_tree(
    state: State<'_, GameState>,
    player: u8,
    max_depth: u8,
) -> Result<Option<TreeNode>, String> {
    let game = state.game.read().await;
    let bot = if player == 1 { &game.bot1 } else { &game.bot2 };

    Ok(bot.as_ref().and_then(|b| b.tree_snapshot(max_depth)))
}
```

### 8.3 TypeScript Store

```typescript
// src/lib/stores/aiInsightsStore.svelte.ts

import { invoke } from '@tauri-apps/api/core';

class AIInsightsStore {
  private _insights = $state<AIInsights | null>(null);
  private _isLoading = $state(false);
  private _error = $state<string | null>(null);

  get insights() { return this._insights; }
  get isLoading() { return this._isLoading; }
  get error() { return this._error; }

  // Derived: current player's insights
  get currentPlayerInsights() {
    if (!this._insights) return null;
    // Determine current player from game state
    return this._insights.player1; // or player2
  }

  async fetchInsights(): Promise<void> {
    this._isLoading = true;
    this._error = null;

    try {
      this._insights = await invoke<AIInsights>('get_ai_insights');
    } catch (e) {
      this._error = e instanceof Error ? e.message : 'Failed to fetch insights';
    } finally {
      this._isLoading = false;
    }
  }

  async fetchTree(player: 1 | 2, maxDepth: number = 4): Promise<TreeNode | null> {
    try {
      return await invoke<TreeNode | null>('get_search_tree', {
        player,
        maxDepth
      });
    } catch (e) {
      console.error('Failed to fetch tree:', e);
      return null;
    }
  }

  clear(): void {
    this._insights = null;
    this._error = null;
  }
}

export const aiInsights = new AIInsightsStore();
```

---

## 9. UI Components

### 9.1 Component Hierarchy

```
SpectatorMode/
├── GameBoard/
│   └── AIOverlay/                 # Layer 1: Board overlays
│       ├── MoveArrows.svelte      # Attack/play arrows
│       ├── ProbabilityBadges.svelte
│       ├── TargetHighlights.svelte
│       └── BestMoveIndicator.svelte
├── AIStatsBar.svelte              # Layer 2: Compact stats
│   ├── EvalMeter.svelte
│   ├── SearchStatsCompact.svelte
│   └── ConfidenceIndicator.svelte
└── AIResearchPanel.svelte         # Layer 3: Detailed sidepanel
    ├── MoveRankings.svelte
    ├── EvalBreakdown.svelte
    ├── SearchTreeView.svelte
    └── SearchStatsDetailed.svelte
```

### 9.2 Key Component Specs

#### MoveArrows.svelte

```svelte
<script lang="ts">
  import { aiInsights } from '$lib/stores/aiInsightsStore.svelte';

  // Props
  let { boardLayout, showThreshold = 0.05 } = $props<{
    boardLayout: BoardLayout;
    showThreshold?: number;
  }>();

  // Filter moves above threshold
  const visibleMoves = $derived(
    aiInsights.currentPlayerInsights?.moveScores
      .filter(m => m.probability >= showThreshold)
      .filter(m => isAttackAction(m.action)) ?? []
  );
</script>

<svg class="move-arrows-overlay">
  {#each visibleMoves as move}
    <Arrow
      from={getSlotPosition(move.action.source, boardLayout)}
      to={getSlotPosition(move.action.target, boardLayout)}
      thickness={move.probability * 10}
      opacity={0.3 + move.probability * 0.7}
      color={move === bestMove ? 'gold' : 'white'}
    />
  {/each}
</svg>
```

#### EvalMeter.svelte

```svelte
<script lang="ts">
  let { score, range = 10 } = $props<{ score: number; range?: number }>();

  // Clamp and normalize to 0-100%
  const normalized = $derived(
    Math.max(0, Math.min(100, (score / range + 1) * 50))
  );

  const favoredPlayer = $derived(score > 0.5 ? 'P1' : score < -0.5 ? 'P2' : 'Even');
</script>

<div class="eval-meter">
  <div class="eval-bar">
    <div class="p1-side" style:width="{normalized}%"></div>
    <div class="p2-side" style:width="{100 - normalized}%"></div>
  </div>
  <span class="eval-label">
    {score > 0 ? '+' : ''}{score.toFixed(1)} ({favoredPlayer})
  </span>
</div>
```

#### SearchTreeView.svelte

```svelte
<script lang="ts">
  import { aiInsights } from '$lib/stores/aiInsightsStore.svelte';

  let expandedNodes = $state(new Set<string>(['root']));

  function toggleNode(nodeId: string) {
    if (expandedNodes.has(nodeId)) {
      expandedNodes.delete(nodeId);
    } else {
      expandedNodes.add(nodeId);
    }
    expandedNodes = new Set(expandedNodes); // Trigger reactivity
  }
</script>

<div class="tree-view">
  {#if aiInsights.currentPlayerInsights?.treeRoot}
    <TreeNodeView
      node={aiInsights.currentPlayerInsights.treeRoot}
      depth={0}
      {expandedNodes}
      onToggle={toggleNode}
    />
  {:else}
    <p class="no-tree">No search tree available</p>
  {/if}
</div>
```

---

## 10. Implementation Phases

### Phase 1: Foundation (Week 1-2)

**Goal**: Wrapper architecture + basic move scores overlay + stats bar

**Rust Tasks** (in `crates/essence-wars-ui/src-tauri/`):
- [ ] Create `src/bots/` module for UI-only wrappers
- [ ] Define `BotIntrospection` trait and data types
- [ ] Create `IntrospectableGreedyBot` wrapper (simplest, for testing)
- [ ] Create `IntrospectableMctsBot` wrapper
- [ ] Implement `MoveScore` extraction (visit counts → probabilities)
- [ ] Add Tauri command `get_ai_insights`
- [ ] Wire up Spectator Mode to use introspectable wrappers

**UI Tasks**:
- [ ] Create `aiInsightsStore.svelte.ts`
- [ ] Create `AIOverlay` component with probability badges
- [ ] Create `AIStatsBar` with basic metrics (eval, nodes, time)
- [ ] Integrate into existing Spectator Mode game board

**Deliverable**: Spectators see move probabilities on cards + a stats bar

**Verification**: Run `cargo bench -p cardgame` before/after to confirm zero perf impact on core

### Phase 2: Evaluation Breakdown + History (Week 3)

**Goal**: Show what factors influence evaluation + historical timeline

**Rust Tasks**:
- [ ] Create `EvalBreakdown` struct with factor contributions
- [ ] Extract breakdown from GreedyBot's evaluation (already has weighted factors)
- [ ] Create `IntrospectableAlphaBetaBot` wrapper
- [ ] Add `TurnSnapshot` and `GameInsightsHistory` to game session
- [ ] Store snapshot after each move in Spectator Mode

**UI Tasks**:
- [ ] Create `AIResearchPanel` shell (expandable sidepanel)
- [ ] Create `EvalBreakdown.svelte` factor table
- [ ] Create `MoveRankings.svelte` with full move list
- [ ] Create `EvalTimeline.svelte` line chart component
- [ ] Add "Key Moments" detection (eval swings > 1.5)

**Deliverable**: Full evaluation breakdown + game history timeline

### Phase 3: Search Tree Visualization (Week 4-5)

**Goal**: Interactive tree visualization for MCTS/AlphaBeta

**Rust Tasks**:
- [ ] Implement `TreeNode` serialization from MCTS tree structure
- [ ] Add depth-limited tree extraction (default 3, max 6)
- [ ] Implement principal variation extraction for AlphaBeta
- [ ] Add `get_search_tree` Tauri command with depth parameter

**UI Tasks**:
- [ ] Create `SearchTreeView.svelte` container
- [ ] Create `TreeNodeView.svelte` (recursive, collapsible)
- [ ] Add node detail tooltip (visits, win%, UCB score)
- [ ] Add visual distinction for best path vs alternatives
- [ ] Add depth slider control

**Deliverable**: Full interactive search tree exploration

### Phase 4: Polish & Neural Prep (Week 6)

**Goal**: Visual polish + architecture for future neural agents

**Rust Tasks**:
- [ ] Define `NeuralAgentIntrospection` trait extension
- [ ] Define `PolicyOutput` struct (action → probability mapping)
- [ ] Define `AttentionWeights` struct (for future transformer agents)
- [ ] Add placeholder `IntrospectableNeuralBot` wrapper

**UI Tasks**:
- [ ] Add animated move arrows (thickness ∝ probability)
- [ ] Add confidence indicator (entropy-based for MCTS/Neural)
- [ ] Create `AttentionHeatmap.svelte` placeholder component
- [ ] Performance optimization: lazy load trees, virtualize long lists
- [ ] Add keyboard shortcuts (T: toggle tree, H: toggle history)

**Deliverable**: Production-ready visualization system, neural-agent-ready

### Phase 5 (Future): Neural Agent Integration

**Goal**: Full neural agent visualization when models are ready

**Tasks**:
- [ ] Implement policy head extraction in neural inference
- [ ] Implement value head extraction
- [ ] Implement attention weight extraction (if transformer)
- [ ] Create attention heatmap visualization
- [ ] Add embedding trajectory visualization (t-SNE/UMAP)

---

## 11. Future: Neural Agent Support

### 11.1 Policy/Value Heads

Neural agents typically output:
- **Policy**: Probability distribution over all legal actions
- **Value**: Single scalar estimating win probability

These map directly to existing visualizations:
- Policy → `MoveScore.probability`
- Value → `EvalBreakdown.total_score`

### 11.2 Attention Visualization

For transformer-based agents, attention weights reveal what the model "looks at".

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionWeights {
    pub layer: u8,
    pub head: u8,
    pub weights: Vec<AttentionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionEntry {
    pub source: AttentionSource,  // Card, creature, global
    pub target: AttentionSource,
    pub weight: f32,
}
```

UI would render as:
- Heatmap over cards/creatures
- Lines connecting attended elements
- Slider to explore different layers/heads

### 11.3 Embedding Visualization

Show game state embeddings in 2D/3D using dimensionality reduction.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingSnapshot {
    pub game_state_embedding: Vec<f32>,  // Full embedding
    pub projected_2d: [f32; 2],          // t-SNE/UMAP projection
    pub similar_states: Vec<SimilarState>,  // Nearest neighbors
}
```

This enables:
- "This position is similar to turn 5 of game X"
- Trajectory visualization over a game
- Clustering of game states

---

## 12. Design Decisions

### Resolved Questions

#### D1: Real-time Search Animation
**Decision**: Show final result only, no streaming.

Rationale:
- Streaming adds significant complexity
- Risk of performance degradation during search
- Final result is sufficient for research/debugging purposes
- Can revisit later if there's strong demand

#### D2: Comparison Mode
**Decision**: Defer to future version, avoid scope creep.

Rationale:
- Nice-to-have but not essential for v1
- Can be added later without architectural changes
- Focus on making single-bot visualization excellent first

#### D3: Historical Insights
**Decision**: Yes, include a simple evaluation timeline.

Implementation:
```rust
// Minimal data stored per turn
pub struct TurnSnapshot {
    pub turn: u8,
    pub player: PlayerId,
    pub eval_score: f32,
    pub chosen_action: Action,
    pub action_str: String,
}

// Stored in game session, not in bot
pub struct GameInsightsHistory {
    pub turns: Vec<TurnSnapshot>,
}
```

UI visualization:
```
Evaluation Over Time
┌────────────────────────────────────────────────────────────┐
│  +5 ┤                              ╭──╮                    │
│     │                         ╭───╯  │                    │
│   0 ┼─────╮    ╭─────────────╯       ╰──╮                 │
│     │     ╰────╯                        ╰───              │
│  -5 ┤                                                      │
│     └──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──        │
│        1  2  3  4  5  6  7  8  9  10 11 12 13 14          │
│                          Turn                              │
│                                                            │
│  Key Moments: (click to jump to turn)                      │
│  • Turn 5: P2 plays Guard, eval swings -2.1               │
│  • Turn 9: P1 clears board, eval swings +3.4              │
└────────────────────────────────────────────────────────────┘
```

Scope: Low - just append to a Vec after each move.

#### D4: Tree Serialization Depth
**Decision**: Default depth 3, user-configurable up to 6.

Rationale:
- Depth 3 captures immediate decision context
- Deeper trees exponentially larger (branching factor ~20-50)
- Power users can request more if needed

#### D5: Performance Isolation (CRITICAL)
**Decision**: Use wrapper pattern - core bots unchanged, introspection in UI-only wrapper.

**Architectural principle**: The core engine, bots, and Python gym must have ZERO performance impact from visualization features. Speed is our distinguishing feature.

```rust
// ============================================
// CORE BOTS (unchanged, used by arena/benchmark/gym)
// ============================================

/// Core MCTS bot - pure performance, no introspection
pub struct MctsBot {
    config: MctsConfig,
    weights: BotWeights,
    rng: SmallRng,
}

impl Bot for MctsBot {
    fn select_action(&mut self, state: &[f32; 328],
                     mask: &[f32; 256], actions: &[Action]) -> Action {
        // Unchanged - pure MCTS, maximum performance
    }
}

// ============================================
// UI WRAPPERS (Tauri UI only, never used by core)
// ============================================

/// Wrapper that adds introspection for UI visualization
/// Lives in: crates/essence-wars-ui/src-tauri/src/bots/
pub struct IntrospectableMctsBot {
    inner: MctsBot,
    last_insights: Option<DecisionInsights>,
}

impl Bot for IntrospectableMctsBot {
    fn select_action(&mut self, state: &[f32; 328],
                     mask: &[f32; 256], actions: &[Action]) -> Action {
        // 1. Run the actual bot (unchanged performance)
        let action = self.inner.select_action(state, mask, actions);

        // 2. AFTER search completes, extract insights
        //    This cost is paid ONLY by UI, never by benchmarks
        self.last_insights = Some(self.extract_insights(&action, actions));

        action
    }
}

impl BotIntrospection for IntrospectableMctsBot {
    fn last_decision(&self) -> Option<&DecisionInsights> {
        self.last_insights.as_ref()
    }
}
```

**File organization**:
```
crates/cardgame/src/bots/
├── mcts.rs              # Core MctsBot (unchanged)
├── alphabeta.rs         # Core AlphaBetaBot (unchanged)
├── greedy.rs            # Core GreedyBot (unchanged)
└── mod.rs               # No introspection here

crates/essence-wars-ui/src-tauri/src/
├── bots/
│   ├── introspectable.rs    # IntrospectableMctsBot, etc.
│   └── mod.rs
└── commands/
    └── ai_insights.rs       # Tauri commands for UI
```

**Who uses what**:
| Context | Bot Type | Introspection |
|---------|----------|---------------|
| `cargo run --bin arena` | `MctsBot` | None |
| `cargo run --bin benchmark` | `MctsBot` | None |
| Python `EssenceWarsEnv` | `MctsBot` | None |
| Tauri UI Spectator Mode | `IntrospectableMctsBot` | Full |

#### D6: Thread Safety
**Decision**: Not needed - sequential access pattern.

The data flow is strictly sequential:
```
Bot.select_action() → returns Action → UI receives → UI calls get_insights()
```

No concurrent access occurs. The wrapper owns `last_insights`, Tauri command clones it for serialization. Simple ownership, no locks needed.

---

## Appendix A: Visual Design Guidelines

### Color Palette

| Element | Color | Usage |
|---------|-------|-------|
| P1 Advantage | `#4CAF50` (green) | Eval bars, scores |
| P2 Advantage | `#F44336` (red) | Eval bars, scores |
| Neutral | `#9E9E9E` (gray) | Even positions |
| Best Move | `#FFD700` (gold) | Highlighting chosen action |
| Candidates | `#FFFFFF` @ 60% | Other considered moves |
| Tree Nodes | Faction colors | Match creature factions |

### Typography

- **Stats numbers**: Monospace, tabular figures
- **Labels**: System sans-serif
- **Tree nodes**: Condensed font for space efficiency

### Animation

- **Probability badges**: Fade in, 200ms ease-out
- **Attack arrows**: Draw animation, 300ms
- **Tree expansion**: Slide down, 200ms
- **Eval bar**: Smooth transition, 500ms

---

## Appendix B: Example Scenarios

### Scenario: MCTS Considers Attack vs Play Card

```
Board State:
- P1 has creature [3/2] in slot 1
- P1 has "Flame Imp" (2/1 Rush) in hand
- P2 has creature [2/4 Guard] in slot 0
- P2 at 8 life

MCTS runs 3000 simulations:

Move Rankings:
1. Play "Flame Imp" → Slot 2    41% (1230 visits, 54% win)
   - Rush allows immediate attack
   - Expected damage: 5 to face next turn

2. Attack Slot1 → Slot0         35% (1050 visits, 51% win)
   - Removes guard
   - Our creature dies (3 vs 4 health)

3. Attack Slot1 → Face          18% (540 visits, 48% win)
   - Blocked by guard (0 damage)
   - Wastes action point

4. End Turn                      6% (180 visits, 43% win)
   - Passing is rarely optimal

Chosen: Play "Flame Imp" → Slot 2
```

This breakdown helps researchers understand:
- MCTS correctly values the Rush keyword
- Guard is properly blocking face attacks
- Visit distribution shows exploration vs exploitation

---

*End of Design Document*
