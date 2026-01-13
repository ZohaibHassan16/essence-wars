# 📜 Essence Wars: The "New Horizons" Action Plan

**Version:** 2.0 (Updated for Phase 1.5 & 2)
**Current Engine Version:** 0.2.x -> **Target:** 0.3.0

---

## 🚧 Phase 1.5: Engine Overhaul & Mechanics (The "Deep Dive")

**Goal:** Upgrade the DNA of the engine to support complex mechanics and verify them with a "Proving Grounds" card set.

### Step 1: The `u16` Migration (Foundation)

The current `u8` bitfield is full. We must expand it to `u16` to allow up to 16 keywords.

* [ ] **1.1 Modify `src/core/keywords.rs`:**
* Change struct to `pub struct Keywords(pub u16)`.
* Update constants (e.g., `RUSH = 0x0001`, `QUICK = 0x0080`).
* Add the 4 New Keyword constants (Bit positions 9-12):
* `EPHEMERAL = 0x0100` (Dies at end of turn)
* `REGENERATE = 0x0200` (Heals at start of turn)
* `STEALTH = 0x0400` (Untargetable by attacks/target-spells)
* `CHARGE = 0x0800` (+Attack when attacking)




* [ ] **1.2 Update `src/core/types.rs` / `state.rs`:**
* Verify `Serialize`/`Deserialize` works (Serde handles u16 automatically, but check for custom bit-packing logic if any exists).


* [ ] **1.3 Update AI Tensor (`src/tensor.rs`):**
* The state tensor currently reads bits. Ensure it loops 16 times (or specifically checks the new flags) so the AI "sees" the new keywords.



### Step 2: Implement Keyword Logic (The Physics)

Now that the bits exist, we must teach the engine what they *do*. Add four cards to the `/home/chris/ai-cardgame/data/cards/sets/starter.yaml` so that 

* [ ] **2.1 Implement `Ephemeral`:**
* **File:** `src/core/engine/game_engine.rs` (in `end_turn_step`)
* **Logic:** Before passing the turn, iterate through `active_player`'s creatures. If `has_ephemeral()`, trigger death.


* [ ] **2.2 Implement `Regenerate`:**
* **File:** `src/core/engine/game_engine.rs` (in `start_turn_step`)
* **Logic:** Iterate `active_player`'s creatures. If `has_regenerate()` AND `health < max_health`, heal for X (define X, usually full heal or fixed amount. Let's start with **2** or **Full**).


* [ ] **2.3 Implement `Stealth`:**
* **File:** `src/core/combat.rs` (in `get_valid_attack_targets`)
* **Logic:** Filter out enemies with `has_stealth()`.


* **File:** `src/core/legal.rs` (in `get_legal_spell_targets`)
* **Logic:** Filter out enemies with `has_stealth()` for targeted spells.


* **File:** `src/core/combat.rs` (in `resolve_combat`)
* **Logic:** "Stealth Break" — When a creature attacks or deals damage, remove the `STEALTH` bit.




* [ ] **2.4 Implement `Charge`:**
* **File:** `src/core/combat.rs` (in `calculate_damage`)
* **Logic:** If `attacker.has_charge()`, add bonus damage (e.g., +2) to the attack value *during calculation only*.



### Step 3: Documentation & Versioning

* [ ] **3.1 Update `design-engine.md`:**
* Add the 4 new keywords to the "Keywords" section with exact rules.
* Update the `u16` technical spec.


* [ ] **3.2 Bump Version:**
* Update `Cargo.toml` to `version = "0.3.0"`.
* Update `src/version.rs` (if you have one).



### Step 4: The "Proving Grounds" (Verification)

We will not build 300 cards yet. We will add **4 Test Cards** (one per mechanic) to the existing starter decks to prove they work.

* [ ] **4.1 Update `aggressive_assault.toml`:**
* Add **"Ghost Wolf"** (1 Cost, 3/3, **Ephemeral**, **Rush**). *Tests Ephemeral + Aggro.*
* Add **"Frenzied Berserker"** (3 Cost, 2/3, **Charge**). *Tests conditional attack power.*


* [ ] **4.2 Update `defensive_control.toml`:**
* Add **"Swamp Troll"** (3 Cost, 2/4, **Regenerate**). *Tests survival.*
* Add **"Shadow Agent"** (2 Cost, 3/2, **Stealth**). *Tests targeting protection.*


* [ ] **4.3 Verification Run:**
* Run `cargo run --bin arena -- --bot1 greedy --bot2 greedy --games 10 --verbose`.
* *Manual Check:* Watch the logs. Did the Ghost Wolf die at end of turn? Did the Troll heal?



---

## 🚀 Phase 2: Card Expansion (The "Industrial Revolution")

**Goal:** Scale from ~34 cards to ~250 cards using automated balance testing. This will be the new Standard Edition Card Pool, /home/chris/ai-cardgame/data/cards/sets/.yaml

### Step 5: The "Grinder" Tooling

* [ ] **5.1 Create `scripts/balance_batch.py`:**
* A script that automates the testing loop:
1. Accepts a path to a `new_batch.yaml`.
2. Runs `arena` (MCTS vs MCTS or Greedy vs Greedy).
3. Parses logs to calculate Win Rate for the deck containing new cards.
4. Outputs a "Verdict": OP (>55%), UP (<45%), or Balanced.




* [ ] **5.2 Create `cards/sets/new-horizons.yaml`:**
* Keep expansion cards separate from the starter set.



### Step 6: Faction Batches (Design & Test)

* [ ] **6.1 Batch A: Argentum Combine (The Wall)**
* **Focus:** `Guard`, `Shield`, `Armor` (High Health).
* **Task:** Design 15 cards -> Run Grinder -> Tune Stats -> Commit.


* [ ] **6.2 Batch B: Symbiote Circles (The Swarm)**
* **Focus:** `Regenerate`, `Lethal`, `Rush`.
* **Task:** Design 15 cards -> Run Grinder -> Tune Stats -> Commit.


* [ ] **6.3 Batch C: Obsidion Syndicate (The Glass Cannon)**
* **Focus:** `Ephemeral`, `Stealth`, Spells.
* **Task:** Design 15 cards -> Run Grinder -> Tune Stats -> Commit.


* [ ] **6.4 Batch D: Free-Walkers (The Toolbox)**
* **Focus:** `Charge`, `Ranged`, Tech cards.
* **Task:** Design 15 cards -> Run Grinder -> Tune Stats -> Commit.



### Step 7: The Meta-Tuning (Retraining the Brain)

Once 60+ new cards are in:

* [ ] **7.1 Run `src/bin/tune.rs`:**
* Re-optimize the AI evaluation weights. The value of `Health` might change if `Lethal` becomes more common!


* [ ] **7.2 Commit `default.toml`:**
* Save the new weights as the standard for the 0.3.0 engine.



---

## ✅ Phase 1.5 Checklist (Copy-Pasteable)

### 🛠️ Engine Upgrade (v0.3.0)
- [ ] **Refactor:** `Keywords(u8)` -> `Keywords(u16)` in `keywords.rs`.
- [ ] **Refactor:** Update `Keywords` constants (shift bits).
- [ ] **Feature:** Implement `Ephemeral` logic (End of Turn death).
- [ ] **Feature:** Implement `Regenerate` logic (Start of Turn heal).
- [ ] **Feature:** Implement `Stealth` logic (Targeting filters).
- [ ] **Feature:** Implement `Charge` logic (Damage calculation bonus).
- [ ] **AI:** Verify `StateTensor` includes new keyword bits.
- [ ] **Docs:** Update `design-engine.md` with new mechanics.
- [ ] **Version:** Bump `Cargo.toml` to `0.3.0`.

### 🧪 Verification (The Proving Grounds)
- [ ] **Content:** Add `Ghost Wolf` (Ephemeral) to `aggressive_assault.toml`.
- [ ] **Content:** Add `Frenzied Berserker` (Charge) to `aggressive_assault.toml`.
- [ ] **Content:** Add `Swamp Troll` (Regenerate) to `defensive_control.toml`.
- [ ] **Content:** Add `Shadow Agent` (Stealth) to `defensive_control.toml`.
- [ ] **Test:** Run `arena` (verbose) and confirm mechanics trigger correctly.
