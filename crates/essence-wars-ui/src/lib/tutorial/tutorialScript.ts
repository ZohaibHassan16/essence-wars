// Tutorial step definitions - simplified and card-agnostic
import type { TutorialStep } from '$lib/stores/tutorialState.svelte';

// Tutorial game configuration
// Uses fixed decks for consistent commander abilities demonstration
export const TUTORIAL_SEED = 42;
export const TUTORIAL_PLAYER_DECK = 'sovereign_lifesteal';
export const TUTORIAL_OPPONENT_DECK = 'broodmother_pack';
export const TUTORIAL_BOT = 'random';

// Simplified tutorial - 11 steps, no specific card references
export const tutorialSteps: TutorialStep[] = [
  // === WELCOME & BASICS (4 steps) ===
  {
    id: 'welcome',
    title: 'Welcome to Essence Wars!',
    message: `Battle as the Blood Sovereign! Win by reducing your opponent's life to 0, or by extracting 50 essence (dealing 50 total face damage). Let's learn the basics!`,
    advanceCondition: { type: 'click_next' },
  },
  {
    id: 'commander',
    title: 'Your Commander',
    message: `Your Commander grants abilities to ALL your creatures. The Blood Sovereign gives Lifesteal (heal when dealing damage) and +1 Health. The enemy Broodmother gives Rush (attack immediately)!`,
    targetElementId: 'player-commander',
    arrowDirection: 'top',
    advanceCondition: { type: 'click_next' },
  },
  {
    id: 'essence',
    title: 'Essence',
    message: `This is your Essence - the resource for playing cards. You start with 1 and gain +1 maximum each turn (up to 10). Spend wisely!`,
    targetElementId: 'player-essence',
    arrowDirection: 'right',
    advanceCondition: { type: 'click_next' },
  },
  {
    id: 'life_and_ap',
    title: 'Life & Action Points',
    message: `Your Commander shows your Life (30 - lose at 0) and Action Points (3 per turn). Each card or attack costs 1 AP.`,
    targetElementId: 'player-commander',
    arrowDirection: 'top',
    advanceCondition: { type: 'click_next' },
  },

  // === FIRST TURN (2 steps) ===
  {
    id: 'hand',
    title: 'Your Cards',
    message: `Your hand has Creatures (fighters), Spells (one-time effects), and Supports (ongoing bonuses). Hover to see details. The number in the corner is the Essence cost.`,
    targetElementId: 'player-hand',
    arrowDirection: 'top',
    advanceCondition: { type: 'click_next' },
  },
  {
    id: 'end_turn_1',
    title: 'End Your Turn',
    message: `With only 1 Essence on turn 1, you can't play most cards. Click "End Turn" to pass. You'll gain more Essence next turn!`,
    targetElementId: 'end-turn-btn',
    arrowDirection: 'top',
    advanceCondition: { type: 'turn_ended' },
  },

  // === PLAY A CREATURE (3 steps) ===
  {
    id: 'opponent_turn',
    title: "Opponent's Turn",
    message: `The Broodmother's creatures have Rush, so they can attack immediately! Watch their turn, then we'll fight back.`,
    advanceCondition: { type: 'auto', delayMs: 2500 },
  },
  {
    id: 'play_creature',
    title: 'Play a Creature',
    message: `Now you have more Essence! Click a creature card in your hand, then click an empty slot on your battlefield to summon it.`,
    targetElementId: 'player-hand',
    arrowDirection: 'top',
    advanceCondition: { type: 'card_played' },
  },
  {
    id: 'summoning_sickness',
    title: 'Summoning Sickness',
    message: `New creatures have Summoning Sickness (the "~" icon) and can't attack this turn. End your turn - next turn you can fight!`,
    targetElementId: 'end-turn-btn',
    arrowDirection: 'top',
    advanceCondition: { type: 'turn_ended' },
  },

  // === COMBAT (2 steps) ===
  {
    id: 'attack',
    title: 'Attack!',
    message: `Your creature is ready! Click it, then click an enemy creature or the enemy commander to attack. Combat is simultaneous - both deal damage!`,
    advanceCondition: { type: 'creature_attacked' },
  },
  {
    id: 'complete',
    title: "You're Ready!",
    message: `You've learned the essentials! Remember: Lifesteal heals you, Rush attacks immediately, Lethal kills instantly, and Stealth can't be targeted. Good luck, Commander!`,
    advanceCondition: { type: 'click_next' },
  },
];
