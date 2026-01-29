/**
 * Statistics Export Utilities
 *
 * Functions for exporting match statistics to JSON and CSV formats
 * for research and design analysis.
 */

import type { SpectatorMatch } from "$lib/api/types";
import type { MatchStatistics, ExportData, TimelineData } from "./types";

// =============================================================================
// JSON Export
// =============================================================================

/**
 * Export match statistics as JSON.
 */
export function exportToJson(
  match: SpectatorMatch,
  statistics: MatchStatistics
): string {
  const exportData: ExportData = {
    matchId: match.id,
    player1Deck: match.player1DeckName,
    player2Deck: match.player2DeckName,
    player1Bot: match.player1BotName,
    player2Bot: match.player2BotName,
    timestamp: new Date().toISOString(),
    statistics,
  };

  return JSON.stringify(exportData, mapReplacer, 2);
}

/**
 * JSON replacer function to handle Map objects.
 */
function mapReplacer(_key: string, value: unknown): unknown {
  if (value instanceof Map) {
    return Object.fromEntries(value);
  }
  return value;
}

// =============================================================================
// CSV Export
// =============================================================================

/**
 * Export match statistics as CSV (multiple sheets combined).
 */
export function exportToCsv(
  match: SpectatorMatch,
  statistics: MatchStatistics
): string {
  const sections: string[] = [];

  // Overview section
  sections.push("# MATCH OVERVIEW");
  sections.push(overviewToCsv(match, statistics));

  // Action economy section
  sections.push("\n# ACTION ECONOMY");
  sections.push(actionEconomyToCsv(statistics));

  // Combat statistics section
  sections.push("\n# COMBAT STATISTICS");
  sections.push(combatStatsToCsv(statistics));

  // Resource economy section
  sections.push("\n# RESOURCE ECONOMY");
  sections.push(resourceStatsToCsv(statistics));

  // Keyword statistics section
  sections.push("\n# KEYWORD STATISTICS");
  sections.push(keywordStatsToCsv(statistics));

  // Timeline data section
  sections.push("\n# TIMELINE DATA");
  sections.push(timelineToCsv(statistics.timeline));

  // AI analysis section (if available)
  if (statistics.aiAnalysis) {
    sections.push("\n# AI ANALYSIS");
    sections.push(aiAnalysisToCsv(statistics));
  }

  return sections.join("\n");
}

function overviewToCsv(match: SpectatorMatch, stats: MatchStatistics): string {
  const { overview } = stats;
  const rows = [
    ["Match ID", match.id],
    ["Player 1 Deck", match.player1DeckName],
    ["Player 2 Deck", match.player2DeckName],
    ["Player 1 Bot", match.player1BotName],
    ["Player 2 Bot", match.player2BotName],
    ["Winner", overview.winner?.toString() ?? "Draw"],
    ["Reason", overview.reason],
    ["Total Turns", overview.totalTurns.toString()],
    ["Total Actions", overview.totalActions.toString()],
    ["Game Duration (ms)", overview.gameDurationMs.toString()],
    ["P1 Final Life", overview.player1FinalLife.toString()],
    ["P2 Final Life", overview.player2FinalLife.toString()],
    ["Life Differential", overview.lifeDifferential.toString()],
    ["MVP Card", overview.mvpCard?.name ?? "N/A"],
    ["MVP Impact", overview.mvpCard?.impact?.toString() ?? "N/A"],
  ];

  return rows.map(row => row.join(",")).join("\n");
}

function actionEconomyToCsv(stats: MatchStatistics): string {
  const { actionEconomy } = stats;
  const headers = [
    "Metric", "Player 1", "Player 2"
  ];

  const rows = [
    headers,
    ["Cards Played", actionEconomy.player1.cardsPlayed, actionEconomy.player2.cardsPlayed],
    ["Creatures Played", actionEconomy.player1.creaturesPlayed, actionEconomy.player2.creaturesPlayed],
    ["Spells Cast", actionEconomy.player1.spellsCast, actionEconomy.player2.spellsCast],
    ["Supports Placed", actionEconomy.player1.supportsPlaced, actionEconomy.player2.supportsPlaced],
    ["Attacks Made", actionEconomy.player1.attacksMade, actionEconomy.player2.attacksMade],
    ["Face Attacks", actionEconomy.player1.faceAttacks, actionEconomy.player2.faceAttacks],
    ["Creature Attacks", actionEconomy.player1.creatureAttacks, actionEconomy.player2.creatureAttacks],
    ["Abilities Used", actionEconomy.player1.abilitiesUsed, actionEconomy.player2.abilitiesUsed],
    ["Turns Played", actionEconomy.player1.turnsPlayed, actionEconomy.player2.turnsPlayed],
    ["AP Spent", actionEconomy.player1.apSpent, actionEconomy.player2.apSpent],
    ["AP Available", actionEconomy.player1.apAvailable, actionEconomy.player2.apAvailable],
    ["AP Efficiency (%)", actionEconomy.player1.apEfficiency, actionEconomy.player2.apEfficiency],
  ];

  return rows.map(row => row.join(",")).join("\n");
}

function combatStatsToCsv(stats: MatchStatistics): string {
  const { combat } = stats;
  const headers = ["Metric", "Player 1", "Player 2"];

  const rows = [
    headers,
    ["Total Damage Dealt", combat.player1.totalDamageDealt, combat.player2.totalDamageDealt],
    ["Damage to Creatures", combat.player1.damageToCreatures, combat.player2.damageToCreatures],
    ["Damage to Face", combat.player1.damageToFace, combat.player2.damageToFace],
    ["Creatures Killed", combat.player1.creaturesKilled, combat.player2.creaturesKilled],
    ["Creatures Lost", combat.player1.creaturesLost, combat.player2.creaturesLost],
    ["K/D Ratio", combat.player1.kdRatio, combat.player2.kdRatio],
    ["Favorable Trades", combat.player1.favorableTrades, combat.player2.favorableTrades],
    ["Even Trades", combat.player1.evenTrades, combat.player2.evenTrades],
    ["Unfavorable Trades", combat.player1.unfavorableTrades, combat.player2.unfavorableTrades],
  ];

  return rows.map(row => row.join(",")).join("\n");
}

function resourceStatsToCsv(stats: MatchStatistics): string {
  const { resources } = stats;
  const headers = ["Metric", "Player 1", "Player 2"];

  const rows = [
    headers,
    ["Total Essence Spent", resources.player1.totalEssenceSpent, resources.player2.totalEssenceSpent],
    ["Avg Essence/Turn", resources.player1.avgEssencePerTurn, resources.player2.avgEssencePerTurn],
    ["Cards Drawn", resources.player1.cardsDrawn, resources.player2.cardsDrawn],
    ["Cards Discarded", resources.player1.cardsDiscarded, resources.player2.cardsDiscarded],
    ["Avg Hand Size", resources.player1.averageHandSize, resources.player2.averageHandSize],
    ["Max Hand Size", resources.player1.maxHandSize, resources.player2.maxHandSize],
    ["Empty Hand Turns", resources.player1.emptyHandTurns, resources.player2.emptyHandTurns],
  ];

  return rows.map(row => row.join(",")).join("\n");
}

function keywordStatsToCsv(stats: MatchStatistics): string {
  const { keywords } = stats;
  const headers = ["Keyword", "Player 1", "Player 2"];

  const rows = [
    headers,
    ["Rush Attacks", keywords.player1.rushAttacks, keywords.player2.rushAttacks],
    ["Guard Blocks", keywords.player1.guardBlocks, keywords.player2.guardBlocks],
    ["Lethal Kills", keywords.player1.lethalKills, keywords.player2.lethalKills],
    ["Lifesteal Healing", keywords.player1.lifestealHealing, keywords.player2.lifestealHealing],
    ["Piercing Damage", keywords.player1.piercingDamage, keywords.player2.piercingDamage],
    ["Shield Absorbed", keywords.player1.shieldAbsorbed, keywords.player2.shieldAbsorbed],
    ["Quick Strikes", keywords.player1.quickStrikes, keywords.player2.quickStrikes],
    ["Fortify Reduced", keywords.player1.fortifyReduced, keywords.player2.fortifyReduced],
    ["Ward Blocks", keywords.player1.wardBlocks, keywords.player2.wardBlocks],
    ["Ranged Attacks", keywords.player1.rangedAttacks, keywords.player2.rangedAttacks],
    ["Stealth Evades", keywords.player1.stealthEvades, keywords.player2.stealthEvades],
    ["Regenerate Healing", keywords.player1.regenerateHealing, keywords.player2.regenerateHealing],
    ["Volatile Damage", keywords.player1.volatileDamage, keywords.player2.volatileDamage],
    ["Frenzy Stacks", keywords.player1.frenzyStacks, keywords.player2.frenzyStacks],
    ["Charge Bonus", keywords.player1.chargeBonus, keywords.player2.chargeBonus],
  ];

  return rows.map(row => row.join(",")).join("\n");
}

function timelineToCsv(timeline: TimelineData): string {
  const headers = [
    "Turn",
    "P1 Life",
    "P2 Life",
    "P1 Win Prob",
    "P2 Win Prob",
    "P1 Board",
    "P2 Board",
    "P1 Essence",
    "P2 Essence",
    "P1 Hand",
    "P2 Hand",
  ];

  const rows: (string | number)[][] = [headers];

  for (let i = 0; i < timeline.turns.length; i++) {
    rows.push([
      timeline.turns[i],
      timeline.player1Life[i],
      timeline.player2Life[i],
      Math.round((timeline.player1WinProb[i] ?? 0.5) * 100) / 100,
      Math.round((timeline.player2WinProb[i] ?? 0.5) * 100) / 100,
      timeline.player1BoardPresence[i],
      timeline.player2BoardPresence[i],
      timeline.player1EssenceSpent[i],
      timeline.player2EssenceSpent[i],
      timeline.player1HandSize[i],
      timeline.player2HandSize[i],
    ]);
  }

  return rows.map(row => row.join(",")).join("\n");
}

function aiAnalysisToCsv(stats: MatchStatistics): string {
  const { aiAnalysis } = stats;
  if (!aiAnalysis) return "";

  const rows = [
    ["Total Simulations", aiAnalysis.totalSimulations],
    ["Avg Simulations/Move", aiAnalysis.avgSimulationsPerMove],
    ["Avg Thinking Time (ms)", aiAnalysis.avgThinkingTimeMs],
    ["Max Thinking Time (ms)", aiAnalysis.maxThinkingTimeMs],
    ["Starting Win Prob", Math.round(aiAnalysis.startingWinProb * 100) / 100],
    ["Final Win Prob", Math.round(aiAnalysis.finalWinProb * 100) / 100],
    ["Max Win Prob Swing", aiAnalysis.maxWinProbSwing],
    ["Critical Moments", aiAnalysis.criticalMoments.length],
  ];

  let csv = rows.map(row => row.join(",")).join("\n");

  // Add critical moments if any
  if (aiAnalysis.criticalMoments.length > 0) {
    csv += "\n\nCritical Moments:";
    csv += "\nTurn,Player,Action,Before,After,Swing";
    for (const moment of aiAnalysis.criticalMoments) {
      csv += `\n${moment.turn},${moment.player},"${moment.actionDescription}",${Math.round(moment.winProbBefore * 100)}%,${Math.round(moment.winProbAfter * 100)}%,${Math.round(moment.swing * 100)}%`;
    }
  }

  return csv;
}

// =============================================================================
// Download Functions
// =============================================================================

/**
 * Trigger a file download in the browser.
 */
export async function downloadFile(
  content: string,
  filename: string,
  mimeType: string
): Promise<void> {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);

  const link = document.createElement("a");
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);

  URL.revokeObjectURL(url);
}

/**
 * Download statistics as JSON file.
 */
export async function downloadJson(
  match: SpectatorMatch,
  statistics: MatchStatistics
): Promise<void> {
  const json = exportToJson(match, statistics);
  const filename = `essence-wars-${match.id}-stats.json`;
  await downloadFile(json, filename, "application/json");
}

/**
 * Download statistics as CSV file.
 */
export async function downloadCsv(
  match: SpectatorMatch,
  statistics: MatchStatistics
): Promise<void> {
  const csv = exportToCsv(match, statistics);
  const filename = `essence-wars-${match.id}-stats.csv`;
  await downloadFile(csv, filename, "text/csv");
}
