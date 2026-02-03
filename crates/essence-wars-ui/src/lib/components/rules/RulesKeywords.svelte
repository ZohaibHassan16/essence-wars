<script lang="ts">
  import KeywordIcon from "../KeywordIcon.svelte";

  // All keywords with detailed descriptions
  const keywords = [
    {
      name: "Rush",
      effect: "Can attack immediately when played",
      detail: "Bypasses summoning sickness. Great for aggressive plays and surprise damage.",
      faction: "Symbiote Circles"
    },
    {
      name: "Ranged",
      effect: "Can attack any enemy creature",
      detail: "Ignores positional restrictions. Can pick off high-value targets regardless of slot.",
      faction: "Free-Walkers"
    },
    {
      name: "Piercing",
      effect: "Excess damage hits the enemy player",
      detail: "When this creature kills an enemy with overkill damage, the excess goes to face.",
      faction: "Argentum Combine"
    },
    {
      name: "Guard",
      effect: "Must be attacked before other creatures",
      detail: "Protects your other creatures and life total. Enemy must clear guards first.",
      faction: "Argentum Combine"
    },
    {
      name: "Lifesteal",
      effect: "Heals your hero equal to damage dealt",
      detail: "Sustain through combat. Particularly powerful on high-attack creatures.",
      faction: "Obsidion Syndicate"
    },
    {
      name: "Lethal",
      effect: "Destroys any creature it damages",
      detail: "Even 1 damage kills. Great for removing large threats with small creatures.",
      faction: "Symbiote Circles"
    },
    {
      name: "Shield",
      effect: "Blocks the first damage instance",
      detail: "Absorbs one hit completely, whether it's 1 damage or 10. Then breaks.",
      faction: "Argentum Combine"
    },
    {
      name: "Quick",
      effect: "Can attack twice per turn",
      detail: "Double the attacks means double the value. Synergizes with attack buffs.",
      faction: "Obsidion Syndicate"
    },
    {
      name: "Ephemeral",
      effect: "Dies at end of turn",
      detail: "Powerful but temporary. Get maximum value immediately.",
      faction: "Various"
    },
    {
      name: "Regenerate",
      effect: "Heals 1 HP at start of your turn",
      detail: "Gradual healing makes this creature sticky and hard to remove efficiently.",
      faction: "Symbiote Circles"
    },
    {
      name: "Stealth",
      effect: "Cannot be targeted until it attacks",
      detail: "Safe from removal and targeted effects until you choose to reveal it.",
      faction: "Obsidion Syndicate"
    },
    {
      name: "Charge",
      effect: "Gains +1 attack each turn",
      detail: "Grows stronger over time. A long-lived Charge creature becomes a major threat.",
      faction: "Free-Walkers"
    },
    {
      name: "Frenzy",
      effect: "Gains +1 attack when damaged",
      detail: "Gets angrier when hurt. Can snowball if it survives multiple combats.",
      faction: "Symbiote Circles"
    },
    {
      name: "Volatile",
      effect: "Deals damage to all enemies when it dies",
      detail: "Board-wide effect on death. Consider both offense and defense.",
      faction: "Obsidion Syndicate"
    }
  ];

  // Group by rough category
  const offensiveKeywords = ["Rush", "Ranged", "Piercing", "Lethal", "Quick", "Charge", "Frenzy"];
  const defensiveKeywords = ["Guard", "Shield", "Regenerate", "Stealth"];
  const utilityKeywords = ["Lifesteal", "Ephemeral", "Volatile"];
</script>

<div class="space-y-8">
  <section>
    <h2 class="text-2xl font-bold text-ui-text mb-4">Keywords</h2>
    <p class="text-ui-text-dim leading-relaxed">
      Keywords are special abilities that modify how creatures behave.
      Hover over a keyword icon in-game to see its effect.
    </p>
  </section>

  <!-- Offensive Keywords -->
  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4 text-damage">Offensive Keywords</h3>
    <div class="space-y-3">
      {#each keywords.filter(k => offensiveKeywords.includes(k.name)) as keyword (keyword.name)}
        <div class="bg-ui-bg rounded-lg p-4 flex items-start gap-4">
          <div class="flex-shrink-0 mt-1">
            <KeywordIcon keyword={keyword.name} size={20} showLabel={false} />
          </div>
          <div class="flex-1">
            <div class="flex items-center gap-2 mb-1">
              <span class="text-ui-text font-semibold">{keyword.name}</span>
              <span class="text-xs text-ui-text-dim bg-ui-panel px-2 py-0.5 rounded">{keyword.faction}</span>
            </div>
            <p class="text-ui-text-dim text-sm">{keyword.effect}</p>
            <p class="text-ui-text-dim text-xs mt-1 italic">{keyword.detail}</p>
          </div>
        </div>
      {/each}
    </div>
  </section>

  <!-- Defensive Keywords -->
  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4 text-mana">Defensive Keywords</h3>
    <div class="space-y-3">
      {#each keywords.filter(k => defensiveKeywords.includes(k.name)) as keyword (keyword.name)}
        <div class="bg-ui-bg rounded-lg p-4 flex items-start gap-4">
          <div class="flex-shrink-0 mt-1">
            <KeywordIcon keyword={keyword.name} size={20} showLabel={false} />
          </div>
          <div class="flex-1">
            <div class="flex items-center gap-2 mb-1">
              <span class="text-ui-text font-semibold">{keyword.name}</span>
              <span class="text-xs text-ui-text-dim bg-ui-panel px-2 py-0.5 rounded">{keyword.faction}</span>
            </div>
            <p class="text-ui-text-dim text-sm">{keyword.effect}</p>
            <p class="text-ui-text-dim text-xs mt-1 italic">{keyword.detail}</p>
          </div>
        </div>
      {/each}
    </div>
  </section>

  <!-- Special Keywords -->
  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4 text-gold">Special Keywords</h3>
    <div class="space-y-3">
      {#each keywords.filter(k => utilityKeywords.includes(k.name)) as keyword (keyword.name)}
        <div class="bg-ui-bg rounded-lg p-4 flex items-start gap-4">
          <div class="flex-shrink-0 mt-1">
            <KeywordIcon keyword={keyword.name} size={20} showLabel={false} />
          </div>
          <div class="flex-1">
            <div class="flex items-center gap-2 mb-1">
              <span class="text-ui-text font-semibold">{keyword.name}</span>
              <span class="text-xs text-ui-text-dim bg-ui-panel px-2 py-0.5 rounded">{keyword.faction}</span>
            </div>
            <p class="text-ui-text-dim text-sm">{keyword.effect}</p>
            <p class="text-ui-text-dim text-xs mt-1 italic">{keyword.detail}</p>
          </div>
        </div>
      {/each}
    </div>
  </section>

  <!-- Quick Reference -->
  <section class="bg-ui-bg/50 rounded-lg p-4 border border-gray-700">
    <h4 class="text-ui-text font-semibold mb-3">Quick Reference</h4>
    <div class="flex flex-wrap gap-2">
      {#each keywords as keyword (keyword.name)}
        <KeywordIcon keyword={keyword.name} size={14} />
      {/each}
    </div>
  </section>
</div>
