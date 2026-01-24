<script lang="ts">
  import { Play, Hand, Swords, SkipForward } from "lucide-svelte";
</script>

<div class="space-y-8">
  <section>
    <h2 class="text-2xl font-bold text-ui-text mb-4">Turn Structure</h2>
    <p class="text-ui-text-dim leading-relaxed">
      Each turn follows a simple structure. You have <span class="text-gold font-semibold">3 Action Points</span>
      to spend on playing cards and attacking. Plan your moves carefully!
    </p>
  </section>

  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4">Turn Start</h3>
    <div class="bg-ui-bg rounded-lg p-4 space-y-3">
      <div class="flex items-start gap-3">
        <div class="w-8 h-8 rounded-full bg-mana/20 flex items-center justify-center flex-shrink-0">
          <span class="text-mana font-bold">1</span>
        </div>
        <div>
          <p class="text-ui-text font-semibold">Gain Essence</p>
          <p class="text-ui-text-dim text-sm">
            Your maximum Essence increases by 1 (up to 10), then refills completely.
          </p>
        </div>
      </div>
      <div class="flex items-start gap-3">
        <div class="w-8 h-8 rounded-full bg-mana/20 flex items-center justify-center flex-shrink-0">
          <span class="text-mana font-bold">2</span>
        </div>
        <div>
          <p class="text-ui-text font-semibold">Draw a Card</p>
          <p class="text-ui-text-dim text-sm">
            Draw one card from your deck. If you cannot draw, you lose the game.
          </p>
        </div>
      </div>
      <div class="flex items-start gap-3">
        <div class="w-8 h-8 rounded-full bg-mana/20 flex items-center justify-center flex-shrink-0">
          <span class="text-mana font-bold">3</span>
        </div>
        <div>
          <p class="text-ui-text font-semibold">Trigger Start-of-Turn Effects</p>
          <p class="text-ui-text-dim text-sm">
            Regenerate heals creatures, Charge increases attack, supports may trigger.
          </p>
        </div>
      </div>
    </div>
  </section>

  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4">Available Actions</h3>
    <p class="text-ui-text-dim mb-4">
      During your turn, you can take actions in any order. Each action costs <span class="text-gold">1 AP</span>.
    </p>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div class="bg-ui-bg rounded-lg p-4 border-l-4 border-mana">
        <div class="flex items-center gap-2 mb-2">
          <Hand size={18} class="text-mana" />
          <h4 class="text-ui-text font-semibold">Play a Card</h4>
        </div>
        <p class="text-ui-text-dim text-sm">
          Pay the card's Essence cost and place it on the battlefield.
          Creatures go in creature slots, supports in support slots.
          Spells resolve immediately.
        </p>
        <p class="text-xs text-ui-text-dim mt-2 italic">Costs: 1 AP + Essence cost</p>
      </div>

      <div class="bg-ui-bg rounded-lg p-4 border-l-4 border-damage">
        <div class="flex items-center gap-2 mb-2">
          <Swords size={18} class="text-damage" />
          <h4 class="text-ui-text font-semibold">Attack</h4>
        </div>
        <p class="text-ui-text-dim text-sm">
          Command a creature to attack an enemy creature or the enemy player directly
          (if no guards are in the way).
        </p>
        <p class="text-xs text-ui-text-dim mt-2 italic">Costs: 1 AP</p>
      </div>

      <div class="bg-ui-bg rounded-lg p-4 border-l-4 border-gold">
        <div class="flex items-center gap-2 mb-2">
          <Play size={18} class="text-gold" />
          <h4 class="text-ui-text font-semibold">Use Ability</h4>
        </div>
        <p class="text-ui-text-dim text-sm">
          Some creatures and supports have activated abilities. Click on them to see
          available actions.
        </p>
        <p class="text-xs text-ui-text-dim mt-2 italic">Costs: Varies</p>
      </div>

      <div class="bg-ui-bg rounded-lg p-4 border-l-4 border-gray-500">
        <div class="flex items-center gap-2 mb-2">
          <SkipForward size={18} class="text-gray-400" />
          <h4 class="text-ui-text font-semibold">End Turn</h4>
        </div>
        <p class="text-ui-text-dim text-sm">
          Pass the turn to your opponent. Any unused AP is lost.
          Press <kbd class="px-1.5 py-0.5 bg-ui-panel rounded text-xs">Space</kbd> to end turn.
        </p>
        <p class="text-xs text-ui-text-dim mt-2 italic">Costs: 0 AP</p>
      </div>
    </div>
  </section>

  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4">Turn End</h3>
    <div class="bg-ui-bg rounded-lg p-4 space-y-2">
      <p class="text-ui-text-dim">
        <span class="text-ui-text font-semibold">End-of-turn effects trigger:</span>
        Ephemeral creatures die, some abilities activate.
      </p>
      <p class="text-ui-text-dim">
        <span class="text-ui-text font-semibold">Summoning sickness clears:</span>
        Creatures that couldn't attack this turn will be ready next turn.
      </p>
    </div>
  </section>

  <section class="bg-ui-bg/50 rounded-lg p-4 border border-gray-700">
    <h4 class="text-ui-text font-semibold mb-2">Example Turn</h4>
    <ol class="text-ui-text-dim text-sm space-y-1 list-decimal list-inside">
      <li>Turn starts: Gain 4th Essence (now 4/4), draw a card</li>
      <li>Play a 3-cost creature to slot 2 (1 AP, 3 Essence spent)</li>
      <li>Attack with existing creature in slot 1 (1 AP)</li>
      <li>Play a 1-cost spell to deal damage (1 AP, 1 Essence)</li>
      <li>No more AP - end turn (or end early to pass)</li>
    </ol>
  </section>
</div>
