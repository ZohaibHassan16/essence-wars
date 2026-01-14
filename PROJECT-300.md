# 🗺️ The "Project 300" Roadmap

> "Developers, what is your profession?" — "GIT PUSH! GIT PULL! GIT COMMIT!"

## 🛠️ Milestone 1: The Foundation Refactor (Technical Cleanup)

**Goal:** Eliminate technical debt, modularize the data, and prepare the engine for scale. No new cards are added yet; we are just reorganizing what we have to support growth.

### 1.1 Data Restructuring
We will move away from monolithic files.

* **Action:** Split `new-horizons.yaml` into faction-specific files.
* **Action:** Organize decks into subfolders to prevent clutter in `data/decks/`.
* **Target Structure:**
```text
data/
├── cards/
│   └── core_set/           <-- New "Set" folder
│       ├── argentum.yaml   (IDs 1000+)
│       ├── symbiote.yaml   (IDs 2000+)
│       ├── obsidion.yaml   (IDs 3000+)
│       └── neutral.yaml    (IDs 4000+)
└── decks/
    ├── argentum/           <-- Organized by faction
    │   ├── control.toml
    │   └── midrange.toml
    ├── symbiote/
    │   └── aggro.toml
    └── ...

```



### 1.2 Engine Update (Rust)

* **Action:** Update `src/core/config.rs` (or similar loader) to iterate through the `cards/core_set/` directory and load *all* `.yaml` files found, rather than looking for a single filename.
* **Action:** Update the deck loader to scan subdirectories recursively.

**1.3 Validation**

* **Action:** Run `./scripts/run-tests.sh` to ensure the refactor didn't break existing mechanics.
* **Action:** Verify that `cargo run --bin arena -- --list-decks` still correctly identifies all decks in their new folders.

---

## 🌿 Milestone 2: Wave 1 - "The Living Jungle" (Symbiote Fix)

**Goal:** Address the current meta imbalance (106-69 loss for Symbiote) by introducing ~30 new cards focused on Symbiote utility and "Anti-Structure" tools.

### 2.1 Design & Implementation

* **Action:** Create `data/cards/core_set/symbiote_wave1.yaml` (or append to the main `symbiote.yaml`).
* **Focus:** "Sticky" minions (Deathrattle tokens) and "Corrosive" effects (reducing enemy Attack/HP) to counter Argentum walls.
* **Target:** +20 Symbiote cards, +10 Neutral cards that help Aggro decks.

### 2.2 Deck Integration

* **Action:** Create new decks: `symbiote/swarm_v2.toml` and `symbiote/corrosion_control.toml`.
* **Action:** Update existing decks (`symbiote_aggro.toml`) to include the new cards.

### 2.3 Tuning & Balancing

* **Action:** Run the `tune` binary (CMA-ES) to generate new weights for `Agent-Symbiote` specifically against Argentum.
* **Action:** Run `arena` matches (100+ games) to verify the win rate has shifted closer to 50%.

---

## 🛡️ Milestone 3: Wave 2 - "Tactical Evolution" (Tech & Utility)

**Goal:** Expand the card pool to ~180 cards. This wave focuses on "Tech" cards (situational answers) for Argentum and Obsidion, plus a massive influx of Neutral "Free-Walkers."

### 3.1 Design & Implementation

* **Focus:**
* **Argentum:** "Anti-Magic" shielding (to counter Obsidion burst).
* **Obsidion:** "Disruption" (messing with enemy hands or costs).
* **Neutral:** "Utility" (card draw, silence, movement).


* **Target:** +30 Argentum, +30 Obsidion, +30 Neutral.

### 3.2 The "Drafteable" Standard

* **Action:** Ensure there are enough "Vanilla" or simple cards to make the card pool feel grounded. Not every card needs complex scripts.

**3.3 Meta-Tuning**

* **Action:** Run a full tournament suite (Generalist vs Specialists) using the `arena` tool to ensure no single faction has become oppressive with the new tools.

---

## 👑 Milestone 4: Wave 3 - "Legends of Omyra" (Completion)

**Goal:** Reach the full 300 cards. Introduce "Legendary" (Unique) cards and final polish.

### 4.1 The Commanders 

* **Action:** Implement the Faction Leaders from the Lore file (e.g., The High Artificer for Argentum, The Broodmother for Symbiote).
* **Mechanic:** These cards will have unique, complex effects that might require custom Rust implementation in `src/core/effects.rs`.

### 4.2 Final Polish

* **Action:** Review all 300 cards for "Flavor" consistency.
* **Action:** Final "Golden Master" balance pass.

---

### 🚦 Immediate Next Steps (The Plan)

To kick this off, I recommend we start with **Milestone 1 (The Refactor)** immediately. It sets the stage for everything else.