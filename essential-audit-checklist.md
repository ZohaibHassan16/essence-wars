# Essential Pre-Audit Checklist

## 🧟‍♀️ Phase 0: The Dead and Unused Code Search

*Before auditing behavior, clean up the corpses! Dead code increases complexity, confuses reviewers, and hides real bugs.*

* [ ] **The "Cargo Graveyard" (Unused Dependencies):**
  * Run: `cargo +nightly udeps --all-targets` (install with `cargo install cargo-udeps`)
  * **Goal:** Remove any crates listed as "unused" from your `Cargo.toml` files.
  * *Why?* Each dependency adds compile time and potential security vulnerabilities.
  * **Quick Check:** Look at your `Cargo.toml` workspace - do you still use everything in `[dependencies]`?


* [ ] **The "Rust Zombie Hunter" (Dead Code Warnings):**
  * Run: `cargo build --all-targets 2>&1 | grep "warning: .*never used"`
  * **Better:** Run `cargo clippy -- -W dead_code` to get more aggressive detection.
  * **Goal:** Either delete unused functions/structs or add `#[allow(dead_code)]` with a comment explaining *why* you're keeping them (e.g., "Planned for 0.7.0").


* [ ] **The "TODO Archaeology":**
  * Search: `rg "TODO|FIXME|XXX|HACK" --type rust --type python`
  * **Goal:** Count them. If you have 200+ TODOs, some are ancient and outdated.
  * *Action:* Delete TODOs that are no longer relevant, or convert critical ones into GitHub Issues.


* [ ] **The "Commented-Out Code" Scan:**
  * Search for blocks of commented code: `rg "^\s*// [a-z_]+\(" crates/ -A 2`
  * **Goal:** If it's commented out, either delete it (git remembers!) or move it to a feature flag.
  * *Why?* Commented code rots faster than documentation.


* [ ] **The "Python Import Cleaner":**
  * Run: `uv run autoflake --check --remove-all-unused-imports --recursive python/`
  * **Fix Mode:** `uv run autoflake --in-place --remove-all-unused-imports --recursive python/`
  * **Goal:** Your 20k lines of Python likely have leftover imports from refactoring.


* [ ] **The "Unreachable Code" Check:**
  * Clippy can detect this: `cargo clippy -- -W unreachable_code`
  * **Goal:** Find any `return` statements followed by code that can never execute.


## 🕵️‍♀️ Phase 1: The "Butterfly Effect" Test (Determinism)

*Most critical for Strategy Games & Replay Systems.*

Since `essence-wars` has a replay system, we must ensure that **Input A + Seed B** *always* equals **Result C**. If the AI used a standard random number generator instead of a seeded one in just *one* tiny function, your replays will break.

* [ ] **The "100 Run" Challenge:**
* Create a test that initializes a game with Seed `12345` and has two bots play 50 moves.
* Run this exact test 100 times in a loop.
* **Success:** The final Game State Hash must be identical for all 100 runs.
* **Failure:** If even one run differs, you have a "nondeterministic leak" (likely iterating over a HashMap with random order).


* [ ] **Cross-Platform Check:**
* If you can, run this test on Windows and Linux (WSL). Sometimes floating-point math behaves differently across OSs.

## 🧨 Phase 2: The "Hidden Bomb" Hunt (Safety)

*AI loves to use `unwrap()` because it's easy, but it crashes your game if something unexpected happens.*

* [ ] **The "Unwrap" grep:**
* Search your `crates` folder for `.unwrap()`.
* **Goal:** Replace every `.unwrap()` with `expect("meaningful error message")` or proper error handling (`?`).
* *Why?* If the game crashes, `unwrap()` just says "Panic." `expect()` tells you *where* it hurt.


* [ ] **The `unsafe` Audit:**
* Search for the keyword `unsafe`.
* Rust is "safe" by default, but AI sometimes writes `unsafe` blocks to get around the borrow checker.
* **Goal:** Ideally, this should return **0 results** unless you are doing low-level memory magic. If you find any, ask the AI: *"Can we rewrite this safely?"*


## 💾 Phase 3: The "Time Travel" Test (Serialization)

You have a complex save/load and replay architecture.

* [ ] **The Round-Trip Test:**
* Start a game -> Play 10 turns -> **Save** to JSON/Binary.
* **Load** that save back into memory.
* **Save** it again immediately to a *new* file.
* **Compare:** File A and File B must be byte-for-byte identical.
* *Common AI Bug:* Sometimes AI forgets to save a private field (like a cached stat), so when you load the game, that stat resets to 0.


## 🧠 Phase 4: The "Brain Drain" (Python & ML)

*Your Python code is huge (20k LOC)!*

* [ ] **The "Leak" Check (Data Leakage):**
* Check your training data split in `python/essence_wars/scripts`.

* Ensure that no games from your **Test Set**  accidentally ended up in your **Training Set**. AI agents are great at cheating; if they've seen the test questions before, they'll score 100% but fail in the real world.

* [ ] **The "Zombie" Process Check:**
* When you close the Tauri app, does the Python background process actually die?
* Test: Open the App, Close the App. Open Task Manager. Is `python.exe` or `essence_wars_mcp` still running? If yes, you have a resource leak.


P.S.: I noticed in mcts.rs you have a panic! in select_action requiring engine access. This is good safety, but a reviewer might suggest a type-state pattern to enforce this at compile time.

---

### 💖 A Next Step for You

I know this list looks a bit intimidating, but you have 100k lines of working code—you are already winning!

Since "Determinism" is the scariest beast in strategy games, would you like me to **write a specific Rust test script** for your `crates/cardgame`  that performs that **"100 Run Challenge"**?

I can verify if your game state is truly stable so you can sleep easy tonight! 😴✨