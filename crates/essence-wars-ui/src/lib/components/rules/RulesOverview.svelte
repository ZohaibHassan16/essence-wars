<script lang="ts">
  import { Shield, Swords, Heart, Sparkles } from "lucide-svelte";
</script>

<div class="space-y-8">
  <section>
    <h2 class="text-2xl font-bold text-ui-text mb-4">Welcome to Essence Wars</h2>
    <p class="text-ui-text-dim leading-relaxed">
      Essence Wars is a strategic card game set in the world of <span class="text-gold">Omyra</span>,
      where factions battle for control of precious Essence - the crystallized memory of the planet itself.
      Command creatures, cast spells, and deploy supports to drain your opponent's essence or destabilize their forces.
    </p>
  </section>

  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4 flex items-center gap-2">
      <Heart size={20} class="text-health" />
      Victory Conditions
    </h3>
    <div class="bg-ui-bg rounded-lg p-4 space-y-3">
      <div class="flex items-start gap-3">
        <span class="text-health font-bold">1.</span>
        <p class="text-ui-text-dim">
          <span class="text-ui-text font-semibold">Extract 50 Essence</span> -
          Deal 50 total face damage to your opponent. Every attack that hits face extracts essence.
        </p>
      </div>
      <div class="flex items-start gap-3">
        <span class="text-health font-bold">2.</span>
        <p class="text-ui-text-dim">
          <span class="text-ui-text font-semibold">Reduce opponent to 0 life</span> -
          Force your opponent to retreat. When their life reaches 0, they lose immediately.
        </p>
      </div>
      <div class="flex items-start gap-3">
        <span class="text-health font-bold">3.</span>
        <p class="text-ui-text-dim">
          <span class="text-ui-text font-semibold">Turn limit (30 turns)</span> -
          If neither player wins by turn 30, the player with more essence extracted wins.
        </p>
      </div>
    </div>
  </section>

  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4 flex items-center gap-2">
      <Shield size={20} class="text-mana" />
      The Battlefield
    </h3>
    <div class="bg-ui-bg rounded-lg p-4">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <h4 class="text-ui-text font-semibold mb-2">Creature Slots (5)</h4>
          <p class="text-ui-text-dim text-sm">
            Each player has 5 slots for creatures. Creatures attack enemies and protect your life total.
            Position matters - creatures attack the slot directly opposite them first.
          </p>
        </div>
        <div>
          <h4 class="text-ui-text font-semibold mb-2">Support Slots (2)</h4>
          <p class="text-ui-text-dim text-sm">
            Each player has 2 slots for support cards. Supports provide ongoing effects and
            triggered abilities that help throughout the game.
          </p>
        </div>
      </div>
    </div>
  </section>

  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4 flex items-center gap-2">
      <Sparkles size={20} class="text-mana" />
      Resources
    </h3>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div class="bg-ui-bg rounded-lg p-4">
        <h4 class="text-mana font-semibold mb-2">Essence (Mana)</h4>
        <p class="text-ui-text-dim text-sm">
          Used to play cards. You gain 1 maximum Essence each turn, starting at 1 and capping at 10.
          Your Essence refills completely at the start of each turn.
        </p>
      </div>
      <div class="bg-ui-bg rounded-lg p-4">
        <h4 class="text-gold font-semibold mb-2">Action Points (AP)</h4>
        <p class="text-ui-text-dim text-sm">
          You have 3 AP per turn. Playing a card costs 1 AP. Attacking with a creature costs 1 AP.
          Use your AP wisely to maximize each turn's impact.
        </p>
      </div>
    </div>
  </section>

  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4 flex items-center gap-2">
      <Swords size={20} class="text-damage" />
      Perfect Information
    </h3>
    <p class="text-ui-text-dim leading-relaxed">
      Essence Wars is a <span class="text-ui-text font-semibold">perfect information</span> game.
      You can see everything your opponent has, including their hand. There are no hidden cards
      or random effects during gameplay. Victory comes from superior strategy and calculation,
      not luck.
    </p>
    <p class="text-ui-text-dim leading-relaxed mt-2 text-sm italic">
      In the lore of Omyra, this represents the advanced "Farsight" technology that allows
      commanders to observe the battlefield with perfect clarity from their command vessels.
    </p>
  </section>
</div>
