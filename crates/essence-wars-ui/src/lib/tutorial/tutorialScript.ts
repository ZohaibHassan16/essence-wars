// Tutorial step definitions and configuration
// Custom-tailored for Seed 123 game flow
import type { TutorialStep } from '$lib/stores/tutorialState.svelte';

// Tutorial game configuration
// Seed 123 produces this starting hand:
// - Iron Bastion (4c) - 1/6 Guard, Fortify - teaches Guard keyword
// - Fortified Sentinel (2c) - 2/3 - first creature to play (Turn 2)
// - The Grand Architect (6c) - 2/6 Fortify - late-game creature
// - Lockdown Protocol (2c) - Spell
// - Armor Plating (1c) - Spell (but we guide to play creature instead)
//
// Game Flow:
// - Turn 1: Only 1 essence, can't play 2c creature, end turn
// - Turn 2: Play Fortified Sentinel (2/3)
// - Turn 3: Opponent plays Rush creature, attacks face. Player retaliates.
// - Later: Iron Bastion's Guard forces enemy attacks
export const TUTORIAL_SEED = 123;
export const TUTORIAL_PLAYER_DECK = 'architect_fortify';
export const TUTORIAL_OPPONENT_DECK = 'broodmother_pack';
export const TUTORIAL_BOT = 'random';

// Tutorial steps - friendly, helpful tone
// Custom-tailored for seed 123 game flow
export const tutorialSteps: TutorialStep[] = [
  // === INTRODUCTION ===
  {
    id: 'welcome',
    title: 'Welcome to Essence Wars!',
    message: `You're about to learn a strategic card game where you command creatures to battle. Your goal: reduce your opponent's life to zero before they do the same to you!`,
    advanceCondition: { type: 'click_next' },
  },

  // === BOARD TOUR ===
  {
    id: 'your_life',
    title: 'Your Life Total',
    message: `This is your life. You start with 30. If it reaches 0, you lose the game!`,
    targetElementId: 'player-life',
    arrowDirection: 'top',
    advanceCondition: { type: 'click_next' },
  },
  {
    id: 'opponent_life',
    title: "Opponent's Life",
    message: `Your opponent also starts at 30. Every point of damage counts toward victory!`,
    targetElementId: 'opponent-life',
    arrowDirection: 'bottom',
    advanceCondition: { type: 'click_next' },
  },
  {
    id: 'essence_intro',
    title: 'Essence - Your Resource',
    message: `Essence is spent to play cards. You gain 1 max each turn (up to 10) and it refills completely each turn. Right now you have 1 essence - not enough for most cards!`,
    targetElementId: 'player-essence',
    arrowDirection: 'top',
    advanceCondition: { type: 'click_next' },
  },

  // === HAND INTRODUCTION ===
  {
    id: 'hand_intro',
    title: 'Your Hand',
    message: `These are your cards. Hover over them to see details. Notice the cost in the top-left corner - your cheapest creature costs 2 essence, but you only have 1 right now.`,
    targetElementId: 'player-hand',
    arrowDirection: 'top',
    advanceCondition: { type: 'click_next' },
  },

  // === TURN 1: ACTION POINTS & END TURN ===
  {
    id: 'action_points',
    title: 'Action Points',
    message: `You have 3 Action Points (AP) per turn. Playing cards and attacking each cost 1 AP. Since you can't afford any creatures this turn, let's end your turn to get more essence.`,
    targetElementId: 'player-ap',
    arrowDirection: 'top',
    advanceCondition: { type: 'click_next' },
  },
  {
    id: 'end_turn_first',
    title: 'End Your Turn',
    message: `Click "End Turn" to pass. You can also press Space as a shortcut. Next turn you'll have 2 essence - enough to play a creature!`,
    targetElementId: 'end-turn-btn',
    arrowDirection: 'top',
    advanceCondition: { type: 'turn_ended' },
  },

  // === OPPONENT TURN 1 ===
  {
    id: 'opponent_turn_1',
    title: "Opponent's Turn",
    message: `Watch your opponent. They also only have 1 essence, so they'll likely pass too.`,
    advanceCondition: { type: 'auto', delayMs: 2000 },
  },

  // === TURN 2: PLAY FIRST CREATURE ===
  {
    id: 'turn_2_intro',
    title: 'Your Turn - Play a Creature!',
    message: `Now you have 2 essence! Look for "Fortified Sentinel" in your hand (costs 2). Click it, then click an empty slot on your battlefield to summon it!`,
    targetElementId: 'player-hand',
    arrowDirection: 'top',
    advanceCondition: { type: 'card_played' },
  },

  // === CREATURE ON BOARD ===
  {
    id: 'creature_stats',
    title: 'Your First Creature!',
    message: `Excellent! The red number is Attack (damage it deals), green is Health. When health reaches 0, the creature dies.`,
    advanceCondition: { type: 'click_next' },
  },
  {
    id: 'summoning_sickness',
    title: 'Summoning Sickness',
    message: `Creatures can't attack on the turn they're summoned - they need time to get ready. Some creatures have "Rush" which lets them attack immediately. End your turn to continue.`,
    targetElementId: 'end-turn-btn',
    arrowDirection: 'top',
    advanceCondition: { type: 'turn_ended' },
  },

  // === OPPONENT PLAYS RUSH CREATURE ===
  {
    id: 'opponent_rush',
    title: 'Watch Out!',
    message: `The opponent played a creature with Rush and attacked you! Rush creatures can attack immediately - they're dangerous early threats.`,
    advanceCondition: { type: 'auto', delayMs: 3000 },
  },

  // === TURN 3: COMBAT ===
  {
    id: 'attack_intro',
    title: 'Time to Fight Back!',
    message: `Your creature is ready now. Click on it, then click an enemy creature to attack. Creatures deal damage to each other simultaneously!`,
    advanceCondition: { type: 'creature_attacked' },
  },
  {
    id: 'combat_result',
    title: 'Combat Resolved',
    message: `Both creatures dealt their attack damage to each other. If either reaches 0 health, it dies. Keep playing cards and attacking to build your advantage!`,
    advanceCondition: { type: 'click_next' },
  },

  // === KEYWORDS & GUARD ===
  {
    id: 'keywords_intro',
    title: 'Card Keywords',
    message: `Cards have special keywords. Look for "Iron Bastion" in your hand (4 cost) - it has Guard. Guard forces enemies to attack that creature first, protecting your other creatures and your life!`,
    advanceCondition: { type: 'click_next' },
  },

  // === WINNING ===
  {
    id: 'winning',
    title: 'How to Win',
    message: `Reduce your opponent's life to 0 to win! If they have no Guard creatures, you can attack them directly. Use creatures, spells, and timing to outmaneuver your opponent.`,
    advanceCondition: { type: 'click_next' },
  },

  // === TUTORIAL COMPLETE ===
  {
    id: 'tutorial_complete',
    title: 'Tutorial Complete!',
    message: `You know the basics of Essence Wars! Continue this game to practice, or return to the menu for a fresh match. Good luck, Commander!`,
    advanceCondition: { type: 'click_next' },
  },
];
