<script lang="ts">
  import { Users, Sparkles, Wrench, Heart, Swords } from "lucide-svelte";
</script>

<div class="space-y-8">
  <section>
    <h2 class="text-2xl font-bold text-ui-text mb-4">Card Types</h2>
    <p class="text-ui-text-dim leading-relaxed">
      There are three types of cards in Essence Wars. Each serves a different strategic purpose.
    </p>
  </section>

  <!-- Creatures -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-health/20 flex items-center justify-center">
        <Users size={24} class="text-health" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">Creatures</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-4 space-y-4">
      <p class="text-ui-text-dim">
        Creatures are your primary fighters. They occupy creature slots, attack enemies,
        and defend your life total.
      </p>

      <div class="grid grid-cols-2 gap-4">
        <div class="flex items-center gap-2">
          <Swords size={16} class="text-damage" />
          <div>
            <p class="text-ui-text font-semibold text-sm">Attack</p>
            <p class="text-ui-text-dim text-xs">Damage dealt when attacking</p>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <Heart size={16} class="text-health" />
          <div>
            <p class="text-ui-text font-semibold text-sm">Health</p>
            <p class="text-ui-text-dim text-xs">Damage it can take before dying</p>
          </div>
        </div>
      </div>

      <div class="border-t border-gray-700 pt-4">
        <h4 class="text-ui-text font-semibold text-sm mb-2">Key Rules</h4>
        <ul class="text-ui-text-dim text-sm space-y-1 list-disc list-inside">
          <li><span class="text-ui-text">Summoning Sickness:</span> Creatures cannot attack the turn they're played (unless they have Rush)</li>
          <li><span class="text-ui-text">One Attack Per Turn:</span> Each creature can attack once per turn (unless they have Quick or Frenzy)</li>
          <li><span class="text-ui-text">Persistent Damage:</span> Damage stays on creatures until they're healed or die</li>
          <li><span class="text-ui-text">Death:</span> When health reaches 0, the creature is destroyed</li>
        </ul>
      </div>
    </div>
  </section>

  <!-- Spells -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-mana/20 flex items-center justify-center">
        <Sparkles size={24} class="text-mana" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">Spells</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-4 space-y-4">
      <p class="text-ui-text-dim">
        Spells are one-time effects that resolve immediately when played.
        They don't occupy board slots.
      </p>

      <div class="border-t border-gray-700 pt-4">
        <h4 class="text-ui-text font-semibold text-sm mb-2">Common Spell Effects</h4>
        <ul class="text-ui-text-dim text-sm space-y-1 list-disc list-inside">
          <li><span class="text-ui-text">Damage:</span> Deal direct damage to creatures or the enemy player</li>
          <li><span class="text-ui-text">Healing:</span> Restore health to your creatures or yourself</li>
          <li><span class="text-ui-text">Buffs:</span> Increase a creature's attack or health</li>
          <li><span class="text-ui-text">Card Draw:</span> Draw additional cards from your deck</li>
          <li><span class="text-ui-text">Removal:</span> Destroy or disable enemy creatures</li>
        </ul>
      </div>

      <div class="bg-ui-panel rounded p-3 text-sm">
        <p class="text-ui-text-dim">
          <span class="text-mana font-semibold">Targeting:</span> Some spells require a target
          (a specific creature or player). Others affect all creatures or have no target.
        </p>
      </div>
    </div>
  </section>

  <!-- Supports -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-gold/20 flex items-center justify-center">
        <Wrench size={24} class="text-gold" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">Supports</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-4 space-y-4">
      <p class="text-ui-text-dim">
        Supports are persistent cards that provide ongoing effects or triggered abilities.
        Each player has 2 support slots.
      </p>

      <div class="flex items-center gap-2">
        <Wrench size={16} class="text-gold" />
        <div>
          <p class="text-ui-text font-semibold text-sm">Durability</p>
          <p class="text-ui-text-dim text-xs">How many times the support can trigger before breaking</p>
        </div>
      </div>

      <div class="border-t border-gray-700 pt-4">
        <h4 class="text-ui-text font-semibold text-sm mb-2">Support Types</h4>
        <ul class="text-ui-text-dim text-sm space-y-1 list-disc list-inside">
          <li><span class="text-ui-text">Triggered:</span> Activate on specific events (start of turn, when a creature dies, etc.)</li>
          <li><span class="text-ui-text">Passive:</span> Provide constant bonuses while in play</li>
          <li><span class="text-ui-text">Activated:</span> Can be used manually for an effect</li>
        </ul>
      </div>

      <div class="bg-ui-panel rounded p-3 text-sm">
        <p class="text-ui-text-dim">
          <span class="text-gold font-semibold">Strategic Note:</span> Supports are valuable
          for long-term advantage but use a limited resource (support slots).
          Choose carefully which supports fit your strategy.
        </p>
      </div>
    </div>
  </section>

  <!-- Card Anatomy -->
  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4">Reading a Card</h3>
    <div class="bg-ui-bg rounded-lg p-4">
      <div class="grid grid-cols-2 gap-4 text-sm">
        <div>
          <p class="text-mana font-semibold">Top Left: Cost</p>
          <p class="text-ui-text-dim text-xs">Essence required to play this card</p>
        </div>
        <div>
          <p class="text-ui-text font-semibold">Top: Name</p>
          <p class="text-ui-text-dim text-xs">The card's name</p>
        </div>
        <div>
          <p class="text-ui-text font-semibold">Center: Art</p>
          <p class="text-ui-text-dim text-xs">Visual representation</p>
        </div>
        <div>
          <p class="text-ui-text font-semibold">Below Art: Keywords</p>
          <p class="text-ui-text-dim text-xs">Special abilities (see Keywords tab)</p>
        </div>
        <div>
          <p class="text-damage font-semibold">Bottom Left: Attack</p>
          <p class="text-ui-text-dim text-xs">Damage dealt (creatures only)</p>
        </div>
        <div>
          <p class="text-health font-semibold">Bottom Right: Health/Durability</p>
          <p class="text-ui-text-dim text-xs">HP for creatures, uses for supports</p>
        </div>
      </div>
    </div>
  </section>
</div>
