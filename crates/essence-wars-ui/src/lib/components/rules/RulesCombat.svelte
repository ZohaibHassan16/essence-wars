<script lang="ts">
  import { Swords, Shield, ArrowRight, Heart, Skull } from "lucide-svelte";
</script>

<div class="space-y-8">
  <section>
    <h2 class="text-2xl font-bold text-ui-text mb-4">Combat</h2>
    <p class="text-ui-text-dim leading-relaxed">
      Combat is how you deal damage to enemy creatures and the enemy player.
      Understanding combat rules is essential to victory.
    </p>
  </section>

  <!-- Basic Combat -->
  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4 flex items-center gap-2">
      <Swords size={20} class="text-damage" />
      Attack Resolution
    </h3>
    <div class="bg-ui-bg rounded-lg p-4 space-y-4">
      <div class="flex items-center gap-4">
        <div class="flex items-center gap-2">
          <div class="w-16 h-20 bg-ui-panel rounded border border-gray-600 flex flex-col items-center justify-center">
            <span class="text-damage font-bold">3</span>
            <span class="text-health font-bold">4</span>
          </div>
          <span class="text-ui-text-dim">Your Creature</span>
        </div>
        <ArrowRight size={24} class="text-damage" />
        <div class="flex items-center gap-2">
          <div class="w-16 h-20 bg-ui-panel rounded border border-gray-600 flex flex-col items-center justify-center">
            <span class="text-damage font-bold">2</span>
            <span class="text-health font-bold">3</span>
          </div>
          <span class="text-ui-text-dim">Enemy Creature</span>
        </div>
      </div>

      <ol class="text-ui-text-dim text-sm space-y-2 list-decimal list-inside">
        <li>Your creature deals its attack (3) as damage to the enemy</li>
        <li>The enemy creature deals its attack (2) as damage back</li>
        <li>Both creatures take damage simultaneously</li>
        <li>Result: Enemy has 0 HP (dies), yours has 2 HP (survives)</li>
      </ol>
    </div>
  </section>

  <!-- Guard Mechanic -->
  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4 flex items-center gap-2">
      <Shield size={20} class="text-cyan-400" />
      Guard Mechanic
    </h3>
    <div class="bg-ui-bg rounded-lg p-4 space-y-4">
      <p class="text-ui-text-dim">
        Creatures with <span class="text-cyan-400 font-semibold">Guard</span> must be attacked
        before you can attack other creatures or the enemy player.
      </p>

      <div class="bg-ui-panel rounded p-3">
        <p class="text-ui-text font-semibold text-sm mb-2">Guard Rules:</p>
        <ul class="text-ui-text-dim text-sm space-y-1 list-disc list-inside">
          <li>If enemy has any Guard creatures, you must attack one of them</li>
          <li>You can choose which Guard to attack if there are multiple</li>
          <li>Ranged creatures can bypass Guard and attack any target</li>
          <li>Spells are not blocked by Guard - they can target anything</li>
        </ul>
      </div>
    </div>
  </section>

  <!-- Direct Damage -->
  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4 flex items-center gap-2">
      <Heart size={20} class="text-health" />
      Attacking the Player
    </h3>
    <div class="bg-ui-bg rounded-lg p-4 space-y-4">
      <p class="text-ui-text-dim">
        When a creature attacks the enemy player directly, it deals damage equal to its attack
        without taking damage back.
      </p>

      <div class="bg-ui-panel rounded p-3">
        <p class="text-ui-text font-semibold text-sm mb-2">Direct Attack Rules:</p>
        <ul class="text-ui-text-dim text-sm space-y-1 list-disc list-inside">
          <li>Can only attack player if no Guards are present</li>
          <li>Ranged creatures can always attack player (ignores Guards)</li>
          <li>Player has 30 life - reduce it to 0 to win</li>
          <li>Players don't deal damage back to attackers</li>
        </ul>
      </div>
    </div>
  </section>

  <!-- Combat Keywords -->
  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4">Combat Keywords</h3>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <div class="bg-ui-bg rounded-lg p-4">
        <p class="text-purple-400 font-semibold text-sm mb-1">Lethal</p>
        <p class="text-ui-text-dim text-xs">
          Any damage destroys the target. A 1/1 Lethal kills an 8/8.
        </p>
      </div>
      <div class="bg-ui-bg rounded-lg p-4">
        <p class="text-orange-400 font-semibold text-sm mb-1">Piercing</p>
        <p class="text-ui-text-dim text-xs">
          Overkill damage goes to face. Kill a 2 HP creature with 5 ATK = 3 face damage.
        </p>
      </div>
      <div class="bg-ui-bg rounded-lg p-4">
        <p class="text-red-400 font-semibold text-sm mb-1">Lifesteal</p>
        <p class="text-ui-text-dim text-xs">
          Heal equal to damage dealt. 5 damage = 5 healing.
        </p>
      </div>
      <div class="bg-ui-bg rounded-lg p-4">
        <p class="text-green-400 font-semibold text-sm mb-1">Quick</p>
        <p class="text-ui-text-dim text-xs">
          Strikes first in combat. If it kills the enemy, takes no damage back.
        </p>
      </div>
      <div class="bg-ui-bg rounded-lg p-4">
        <p class="text-blue-300 font-semibold text-sm mb-1">Shield</p>
        <p class="text-ui-text-dim text-xs">
          First damage instance is blocked completely, then Shield breaks.
        </p>
      </div>
      <div class="bg-ui-bg rounded-lg p-4">
        <p class="text-orange-500 font-semibold text-sm mb-1">Frenzy</p>
        <p class="text-ui-text-dim text-xs">
          Gains +1 attack when damaged. Gets stronger during combat.
        </p>
      </div>
    </div>
  </section>

  <!-- Combat Tips -->
  <section class="bg-ui-bg/50 rounded-lg p-4 border border-gray-700">
    <h4 class="text-ui-text font-semibold mb-3 flex items-center gap-2">
      <Skull size={16} class="text-damage" />
      Combat Tips
    </h4>
    <ul class="text-ui-text-dim text-sm space-y-2 list-disc list-inside">
      <li>
        <span class="text-ui-text">Trade efficiently:</span> Use smaller creatures to kill
        larger threats when possible (especially with Lethal).
      </li>
      <li>
        <span class="text-ui-text">Value Guards:</span> A well-placed Guard protects your
        valuable creatures and buys time.
      </li>
      <li>
        <span class="text-ui-text">Face is the place:</span> Sometimes ignoring enemy creatures
        and racing face damage is the right play.
      </li>
      <li>
        <span class="text-ui-text">Health is a resource:</span> Taking damage to develop your
        board is often correct in the early game.
      </li>
    </ul>
  </section>
</div>
