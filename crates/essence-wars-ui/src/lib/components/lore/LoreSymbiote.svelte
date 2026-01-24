<script lang="ts">
  import { Bug, Dna, TreePine, Heart } from "lucide-svelte";

  // Featured card IDs for Symbiote (2000-2074)
  const featuredCards = [
    { id: 2000, name: "Toxic Spider", desc: "Small but deadly" },
    { id: 2060, name: "The Hive Mother", desc: "Queen of the swarm" },
    { id: 2011, name: "Centaur Charger", desc: "Swift and relentless" },
    { id: 2045, name: "Apex Predator", desc: "Evolution's masterpiece" },
  ];
</script>

<div class="space-y-8">
  <!-- Header -->
  <section class="text-center">
    <div class="w-20 h-20 mx-auto rounded-2xl bg-gradient-to-br from-green-800 to-purple-600 flex items-center justify-center mb-4 shadow-lg">
      <Bug size={48} class="text-green-400" />
    </div>
    <h2 class="text-3xl font-bold text-green-400 mb-2">The Symbiote Circles</h2>
    <p class="text-xl text-ui-text-dim italic">"Adapt or Perish."</p>
  </section>

  <!-- Card Art Gallery -->
  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4 flex items-center gap-2">
      <Bug size={20} class="text-green-400" />
      Children of Evolution
    </h3>
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      {#each featuredCards as card}
        <div class="group relative">
          <div class="rounded-lg overflow-hidden border-2 border-green-500/30 hover:border-green-500 transition-all shadow-lg">
            <img
              src="/cards/core_set/{card.id}.webp"
              alt={card.name}
              class="w-full aspect-[3/4] object-cover group-hover:scale-105 transition-transform duration-300"
            />
          </div>
          <div class="mt-2 text-center">
            <p class="text-ui-text font-semibold text-sm">{card.name}</p>
            <p class="text-ui-text-dim text-xs">{card.desc}</p>
          </div>
        </div>
      {/each}
    </div>
  </section>

  <!-- Philosophy -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-green-500/20 flex items-center justify-center">
        <Dna size={24} class="text-green-400" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">The Philosophy of Evolution</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 border-l-4 border-green-500 space-y-4">
      <p class="text-ui-text-dim leading-relaxed">
        The Symbiote Circles are not a horde of mindless beasts - they are a sophisticated civilization
        where the line between "person" and "creature" has been deliberately erased. Every humanoid member
        has undergone <span class="text-green-400 font-semibold">The Communion</span>, a ritual bonding that
        integrates them into the collective biological network.
      </p>
      <p class="text-ui-text-dim leading-relaxed">
        To outsiders, this appears horrifying. To the Circles, it is the natural next step of evolution.
        They do not see their mutations as deformities - they see themselves as <span class="text-ui-text">improved</span>.
        The baseline human body is considered an unfinished draft, waiting to be perfected through symbiosis.
      </p>
      <blockquote class="border-l-2 border-green-500/50 pl-4 italic text-ui-text-dim">
        "You ask if our beasts are intelligent. You ask the wrong question.
        Ask instead: are your 'individuals' still capable of being anything greater than alone?"
        <span class="block text-green-400 text-sm mt-1">- Evolution Priest Tyrak</span>
      </blockquote>
    </div>
  </section>

  <!-- Society -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-green-500/20 flex items-center justify-center">
        <TreePine size={24} class="text-green-400" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">Life in the Living Cities</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 space-y-4">
      <p class="text-ui-text-dim leading-relaxed">
        The cities of the Circles are <span class="text-green-400">grown, not built</span>. Massive Spire Trees
        form the central structures, their branches becoming walkways, their hollows becoming homes.
        Buildings are coral-like growths that expand to accommodate new residents.
        Streets are root networks that guide travelers and transport nutrients.
      </p>

      <!-- Castes -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mt-4">
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-green-400 font-semibold text-sm">Bio-Weavers</p>
          <p class="text-ui-text-dim text-xs">Genetic engineers with living fiber robes and symbiotic tools</p>
        </div>
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-green-400 font-semibold text-sm">Hive Shepherds</p>
          <p class="text-ui-text-dim text-xs">Beast commanders with chitin armor bonded to skin</p>
        </div>
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-green-400 font-semibold text-sm">Spore-Sowers</p>
          <p class="text-ui-text-dim text-xs">Healers in mycelium robes with mushroom familiars</p>
        </div>
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-green-400 font-semibold text-sm">Circle Warriors</p>
          <p class="text-ui-text-dim text-xs">Soldiers with living weapons fused to their arms</p>
        </div>
      </div>
    </div>
  </section>

  <!-- Relationship with Beasts -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-green-500/20 flex items-center justify-center">
        <Heart size={24} class="text-green-400" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">The Bond with Beasts</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 border-l-4 border-green-500">
      <p class="text-ui-text-dim leading-relaxed mb-4">
        The creatures of the Circles are not pets, mounts, or tools - they are <span class="text-ui-text font-semibold">extensions</span>
        of the collective. A Hive Shepherd does not "command" her beetle swarm; she <em>is</em> the swarm,
        perceiving through a thousand compound eyes while her humanoid body directs the whole.
      </p>
      <p class="text-ui-text-dim leading-relaxed mb-4">
        When a Circle Warrior bonds with a living blade-creature, the weapon can feel pain, hunger, and loyalty.
        The "Siege Beetle" that the Argentum Combine fears is not a mindless animal - it contains a Gene-Sculptor
        who has merged with it completely, their consciousness distributed throughout its massive form.
      </p>
      <blockquote class="border-l-2 border-green-500/50 pl-4 italic text-ui-text-dim">
        "The Combine builds walls of steel. We just dissolve the hinges."
        <span class="block text-green-400 text-sm mt-1">- Circle Bio-Weaver</span>
      </blockquote>
    </div>
  </section>

  <!-- Keywords -->
  <section class="bg-gradient-to-r from-green-500/10 to-transparent rounded-lg p-5 border border-green-500/30">
    <h4 class="text-ui-text font-semibold mb-3">Signature Keywords</h4>
    <div class="flex flex-wrap gap-3">
      <div class="flex items-center gap-2 bg-ui-bg px-3 py-2 rounded-lg">
        <span class="w-2 h-2 rounded-full bg-yellow-400"></span>
        <span class="text-yellow-400 font-semibold">Rush</span>
        <span class="text-ui-text-dim text-sm">- Can attack immediately</span>
      </div>
      <div class="flex items-center gap-2 bg-ui-bg px-3 py-2 rounded-lg">
        <span class="w-2 h-2 rounded-full bg-purple-400"></span>
        <span class="text-purple-400 font-semibold">Lethal</span>
        <span class="text-ui-text-dim text-sm">- Destroys any creature it damages</span>
      </div>
      <div class="flex items-center gap-2 bg-ui-bg px-3 py-2 rounded-lg">
        <span class="w-2 h-2 rounded-full bg-emerald-400"></span>
        <span class="text-emerald-400 font-semibold">Regenerate</span>
        <span class="text-ui-text-dim text-sm">- Heals at start of turn</span>
      </div>
    </div>
  </section>
</div>
