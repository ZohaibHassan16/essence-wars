// Tutorial step definitions and configuration
// Custom-tailored for Seed 77 game flow with sovereign_lifesteal vs broodmother_pack
import type { TutorialStep } from '$lib/stores/tutorialState.svelte';

// Tutorial game configuration
// Seed 77 produces this starting hand for sovereign_lifesteal:
// - Contract Killer (3c) - 3/2 Lethal - first creature to play (Turn 3)
// - Supply Depot (3c) - Support with durability 4
// - Silent Assassin (4c) - 4/3 Stealth, Quick
// - Hired Blade (3c) - 3/3 vanilla creature
// - Life Tap (1c) - Spell
//
// Blood Sovereign commander grants: Lifesteal + +0/+1 to all creatures
// Broodmother commander grants: Rush to all creatures
//
// Game Flow:
// - Turn 1: Only 1 essence, can play Life Tap but better to save, end turn
// - Turn 2: Opponent plays Rush creature and attacks face
// - Turn 3: Play Contract Killer (shows Lifesteal from commander!)
// - Turn 4: Combat demonstrates Lethal + Lifesteal
// - Turn 5: Play Silent Assassin (Stealth + Quick) and Supply Depot
export const TUTORIAL_SEED = 77;
export const TUTORIAL_PLAYER_DECK = 'sovereign_lifesteal';
export const TUTORIAL_OPPONENT_DECK = 'broodmother_pack';
export const TUTORIAL_BOT = 'random';

// Tutorial steps - friendly, engaging tone
// Designed to teach mechanics without tutorial fatigue (~17 steps)
export const tutorialSteps: TutorialStep[] = [
  // === PHASE 1: WELCOME & BOARD TOUR (4 steps) ===
  {
    id: 'welcome',
    title: 'Welcome to Essence Wars!',
    message: `You command the Blood Sovereign in battle! Your goal: reduce your opponent's life to zero before they do the same to you. Let's learn the basics!`,
    advanceCondition: { type: 'click_next' },
  },
  {
    id: 'your_commander',
    title: 'Your Commander',
    message: `This is your Commander - The Blood Sovereign. Commanders grant powerful abilities to ALL your creatures. Yours gives Lifesteal (heal when dealing damage) and +1 Health!`,
    targetElementId: 'player-commander',
    arrowDirection: 'top',
    advanceCondition: { type: 'click_next' },
  },
  {
    id: 'resources_intro',
    title: 'Your Resources',
    message: `You have 30 Life (lose when it hits 0), Essence to play cards (gains +1 max each turn), and 3 Action Points per turn for playing cards and attacking.`,
    targetElementId: 'player-life',
    arrowDirection: 'top',
    advanceCondition: { type: 'click_next' },
  },
  {
    id: 'hand_intro',
    title: 'Your Hand',
    message: `These are your cards! You have Creatures (fight on the board), Spells (one-time effects), and Supports (ongoing bonuses). Hover over cards to see details.`,
    targetElementId: 'player-hand',
    arrowDirection: 'top',
    advanceCondition: { type: 'click_next' },
  },

  // === PHASE 2: FIRST TURNS (3 steps) ===
  {
    id: 'turn_1_end',
    title: 'End Your First Turn',
    message: `With only 1 Essence, you can't afford most cards yet. End your turn to gain more Essence next turn. Click "End Turn" or press Space.`,
    targetElementId: 'end-turn-btn',
    arrowDirection: 'top',
    advanceCondition: { type: 'turn_ended' },
  },
  {
    id: 'opponent_turn_1',
    title: "Opponent's Turn",
    message: `Your opponent (The Broodmother) also has limited Essence. Watch what they do...`,
    advanceCondition: { type: 'auto', delayMs: 2000 },
  },
  {
    id: 'opponent_rush_attack',
    title: 'Rush Attack!',
    message: `The opponent played a creature with Rush - it can attack immediately! Their Commander gives ALL creatures Rush. You took damage to your life total.`,
    advanceCondition: { type: 'auto', delayMs: 3500 },
  },

  // === PHASE 3: PLAY YOUR FIRST CREATURE (3 steps) ===
  {
    id: 'play_creature',
    title: 'Fight Back!',
    message: `Now you have 3 Essence. Find "Contract Killer" in your hand (costs 3) and click it, then click an empty slot on your battlefield to summon it!`,
    targetElementId: 'player-hand',
    arrowDirection: 'top',
    advanceCondition: { type: 'card_played' },
  },
  {
    id: 'commander_passive_demo',
    title: 'Commander Power!',
    message: `Look at your creature - it shows Lifesteal AND Lethal! The card only has Lethal, but your Commander automatically granted Lifesteal. It also has +1 Health from your Commander!`,
    advanceCondition: { type: 'click_next' },
  },
  {
    id: 'summoning_sickness',
    title: 'Summoning Sickness',
    message: `See the "~" on your creature? That's Summoning Sickness - creatures can't attack the turn they're played. End your turn, and next turn you can fight!`,
    targetElementId: 'end-turn-btn',
    arrowDirection: 'top',
    advanceCondition: { type: 'turn_ended' },
  },

  // === PHASE 4: COMBAT & KEYWORDS (4 steps) ===
  {
    id: 'opponent_attacks_again',
    title: 'Under Attack!',
    message: `The enemy is attacking again! Watch how their Rush creatures keep hitting you. You need to fight back!`,
    advanceCondition: { type: 'auto', delayMs: 3500 },
  },
  {
    id: 'attack_intro',
    title: 'Attack!',
    message: `Your creature is ready! Click on it, then click an enemy creature to attack. Combat is simultaneous - both creatures deal damage to each other.`,
    advanceCondition: { type: 'creature_attacked' },
  },
  {
    id: 'lethal_lifesteal_explain',
    title: 'Powerful Keywords!',
    message: `Amazing! Lethal killed their creature instantly (any damage = death). Lifesteal healed you for the damage dealt! These keywords work together beautifully.`,
    advanceCondition: { type: 'click_next' },
  },
  {
    id: 'play_stealth_quick',
    title: 'More Keywords',
    message: `Play "Silent Assassin" from your hand - it has Stealth (can't be targeted by attacks while hidden) and Quick (attacks first in combat, avoiding counter-damage)!`,
    targetElementId: 'player-hand',
    arrowDirection: 'top',
    advanceCondition: { type: 'card_played' },
  },

  // === PHASE 5: ADVANCED & FREEDOM (3 steps) ===
  {
    id: 'supports_intro',
    title: 'Support Cards',
    message: `You also have "Supply Depot" - a Support card! Supports go in special slots and provide ongoing effects. Play it to see how Supports work.`,
    targetElementId: 'player-hand',
    arrowDirection: 'top',
    advanceCondition: { type: 'card_played' },
  },
  {
    id: 'ai_hint_intro',
    title: 'Need Help?',
    message: `Feeling stuck? Click "Get Hint" anytime to see what the AI recommends! It analyzes the board and suggests the best move. Use it whenever you need guidance.`,
    targetElementId: 'ai-hint-panel',
    arrowDirection: 'left',
    advanceCondition: { type: 'click_next' },
  },
  {
    id: 'tutorial_complete',
    title: "You're Ready!",
    message: `You've learned the essentials: Commanders, Resources, Combat, and Keywords like Rush, Lethal, Lifesteal, Stealth, and Quick. Now finish this battle on your own! Good luck, Commander!`,
    advanceCondition: { type: 'click_next' },
  },
];
