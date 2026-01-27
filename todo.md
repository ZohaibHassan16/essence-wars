### Deck Builder

Custom Deck Builder

Choose Faction -> Choose Commander

Choose Cards from Faction + Neutral Cards 

Save as Deck, add Description

Be able to play with it


## LateGame/Endgame Buff

The Idea: "Essence Overflow" (Solving Starvation) Instead of a "Second Wind" hard rule, use your Commander system!

Since you already have Commanders, give them a generic, expensive active ability available to everyone once the game goes late.

Mechanic Idea: Commander's Insight

Cost: 4 or 5 Essence.

Effect: Draw a card.

Trigger: Only usable if you have 0 or 1 card in hand (or after turn 8).

This prevents the "top-decking mode" (where you play one card and pass) without flooding the hand, and it gives players something to do with excess Essence in the late game.

## Tuning Weights

1. The Bot Tuning Conundrum 🤖
You asked if you should retune per specific Deck/Commander, keep it general, or switch to Archetypes.

My advice: Do NOT tune for every single specific deck. Here is why: It creates a maintenance nightmare. Every time you tweak a card’s cost in data/cards, you would invalidate dozens of weight files. Plus, "over-fitting" a bot to a specific deck often makes it brittle—it becomes great at playing that list but fails if the player swaps two cards.

The "Retune in Client" Feature:

Should we offer a "Retune your Bots" function for the game client? Hard No. 🙅‍♀️ Running a full CMA-ES (Covariance Matrix Adaptation Evolution Strategy) or MCTS tuning loop on a user's laptop is a bad user experience. It turns their computer into a space heater, drains battery, and takes hours (as you noted). Players want to play, not wait for a compiler.

The Solution: The "Archetype Buckets" Approach You mentioned moving towards "Archetypes" (Aggro, Control, etc.). This is the brilliant middle ground.

Instead of binding weights to a Faction (which is arbitrary), bind them to a Playstyle Profile.

How to implement this:

Train 3-4 Master Profiles:

The Rusher (Aggro): Tune this bot using only low-cost, high-tempo decks. It learns to value Face Damage and Tempo over Card Advantage.

The Wall (Control): Tune this using removal-heavy decks. It learns to value Board Clears and Hand Size.

The Generalist (Midrange): Your current baseline.

Dynamic Weight Selection: When the user builds a custom deck, your engine analyzes it instantly (0.1ms) to assign a "Profile."

Plaintext
       User's Custom Deck
             │
             ▼
    [ Deck Analyzer ]
    (Calculates Avg Cost, Unit Count, Keywords)
             │
    ┌────────┼────────┐
    ▼        ▼        ▼
 Is Aggro? Is Control? Is Midrange?
    │        │        │
    ▼        ▼        ▼
 [Load]   [Load]   [Load]
 Rush.toml Wall.toml Std.toml
Why this works: It makes the AI feel "smart" without needing client-side training. An AI playing an Aggro deck should know not to trade efficiently but to hit the face!