# To Do

## ✅ (A) Deck Builder [DONE]

- Custom Deck Builder
- Choose Faction -> Choose Commander
- Choose Cards from Faction + Neutral Cards 
- Save as Deck, add Description
- App should automatically detect which playstyle the deck falls into 
- Be able to play with custom decks
- Cannot edit or overwrite the prebuilt 12 Starter Decks, only custom decks

## ✅ (B) Update Lore and Other Documentation [DONE]

Update `lore.md` with the new direction for Argentum, Symbiote and the Free Walkers in our Art Direction `docs/art-direction.md` and the prompts(Artwork). Free Walkers not using guns, Symbiote not being bio punk engineers but naturalists and druids etc. Make sure the whole Document is in sync, including `docs/essence-wars-design.md` Faction Descriptions.

Review all other existing Documentation and look for out of date statements/Info.

## ✅ (C) Update and Enhance Diagnosis and Analysis Capabilities of Spectator Mode [DONE]

Review existing Analysis (Chris will have to supply Screenshots).

Update and Enhance the Diagnosis/Analysis Screen in Spectator Mode, using the now fully enhanced Rust binary Diagnose/Analysis we have at our hand, with plots, graphs, tables, statistics, probabilities etc.

Review and enhance AI Visualization Layout and integrate better with existing features, streamline and polish the UI/UX Exerience.

## (D) Review and Update MCP Server

- Bring up to date if needed
- Review `explain_rules()` , so that LLM Agents can play with confidence
- Review and Audit complete Package
- Discuss if AI Hint function needs to be Improved/Enhanced (Test it in a life game first) - does it use Alpha-Beta Depth 6 (it is the best bot we have)

## (F) Review and Update Python Gym

- Look into the changes, update and integrate everything
- review Github Action workflows for Maturin/PyPI for Publishing to Github Packages / PyPI

## (G) Review if Asset Pipeline is still in sync

Search and review all Asset related Scripts, Data, Prompts, Artwork, Documention etc, and bring them up to date and in sync.

## (H) Tutorials

- Update Human vs AI Tutorial
- Use the Replay System to be sure each step is 100% the same each time

## (I) Expansion Planning

| Question | Decision |
|----------|----------|
| Scripting vs Pure Rust | Pure Rust for performance |
| Custom YAML cards | Yes, sandboxed for casual play |
| Bot/Agent compatibility | Accept retuning; keep observation/action space stable |
| Expansion structure | 1 commander + deck per faction per expansion |

## (J) Essential Audit Checklist

Work through `essential-audit-checklist.md`.

### (K) Test Suite

Improve Test Coverage for vital Systems (Crates and Python)

## (L) ✅ Win Conditions [DONE]

Review Win Conditions implementation and Discussion in Code Base and `win-discussion.md`. Update Essence Wars Design Doc and other Documentation, including Game App `crates/essence-wars-ui`.

**Decision:** Keep life as "Tactical Stability" (zero = forced retreat), add Essence Extraction as parallel win condition.

**Implementation Details:**
- Renamed `GameMode::EssenceDuel` → `GameMode::EssenceWar` (now default)
- Renamed `WinReason::VictoryPointsReached` → `WinReason::EssenceExtractionReached`
- Renamed `WinReason::TurnLimitHigherLife` → `WinReason::TurnLimitTiebreaker`
- Turn limit tiebreaker uses VP for EssenceWar mode, life for Attrition mode
- Backwards compatibility maintained (CLI accepts "essence-duel", "essenceduel" etc.)
- All 759 tests passing

**Thresholds:** 50 essence extracted to win (unchanged from VictoryPoints).


## (M) ✅ Deck Library Visualization [DONE]

Add Player 1 and Player 2 Libraries to Spectator Mode. THe Spectator should be able to scroll through all available cards in the library at any time.

For Human vs AI Mode, let's discuss if we should keep it hidden as it is currently, or make it the same (More of a Game Design Discussion).

## (N) Web-playable demo - WASM build where people can try it instantly

Playing the Game online should be easy and quick, not require an .exe or Linux Binary / .deb download. Discuss hosting Options (huggingface or github pages, or other solutions?) and Infrastructure Concerns/Specs.

## (Z) Create new Documentation Suite

- Each Crate/Python gets its onw Readme.
- Organize `docs/` better
- Target Audience mostly human contributors (AI Developers read the source code)
- Sorted by Game Design, Art Asset Pipeline, ML/AI Infrastructure, ML/AI Research/Experiments, etc.
- One good Main README.md that is top notch and polished (with gifs and pictures etc)