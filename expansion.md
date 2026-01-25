# Expansion Roadmap

Discuss the following points:

1. Should we add scripting for custom keywords and effects, or prefer to add mechanics, effects and keywords to the engine in pure rust code to preserve performance?
2. Should we add scripting to support custom cards that can be loaded from custom YAMLs?
3. How to future Proof our Engine and ML/AI and Algo(Bots) Infrastructure for future expansion sets?
  - Should we simply accept that there will be breaking changes for agents and bots with new sets?

Goal: Each Expansion Set should be able to introduce a new keyword without rendering previous expansions or sets obsolote, avoiding power creep (like in MTG). Should only the basic `New Horizons` Edition must be 100% compatible with the python gym, no breaking changes, or instead risk breaking changes requiring agents to retune or retrain?

Random, Greedy, MCTS and Alpha-Beta Bots should be able to work with new expansions (with tuning for each addition)

Each Expansion set should get one new Commander and new Decks per Faction, with the needed cards added to each faction. Neutral does not get a commmander or deck, but also new cards.