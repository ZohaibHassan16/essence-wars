<script lang="ts">
  import { Cog, Users, Building, Wrench } from "lucide-svelte";

  // Featured card IDs for Argentum (1000-1074)
  const featuredCards = [
    { id: 1000, name: "Brass Sentinel", desc: "The backbone of Combine defense" },
    { id: 1060, name: "The Architect", desc: "Legendary commander and visionary" },
    { id: 1020, name: "Siege Breaker", desc: "When walls must fall" },
    { id: 1045, name: "Clockwork Colossus", desc: "Walking fortress of brass and steam" },
  ];
</script>

<div class="space-y-8">
  <!-- Header -->
  <section class="text-center">
    <div class="w-20 h-20 mx-auto rounded-2xl bg-gradient-to-br from-gray-200 to-gold/50 flex items-center justify-center mb-4 shadow-lg">
      <Cog size={48} class="text-gray-800" />
    </div>
    <h2 class="text-3xl font-bold text-gold mb-2">The Argentum Combine</h2>
    <p class="text-xl text-ui-text-dim italic">"Precision. Protocol. Peace."</p>
  </section>

  <!-- Card Art Gallery -->
  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4 flex items-center gap-2">
      <Cog size={20} class="text-gold" />
      Champions of Order
    </h3>
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      {#each featuredCards as card (card.id)}
        <div class="group relative">
          <div class="rounded-lg overflow-hidden border-2 border-gold/30 hover:border-gold transition-all shadow-lg">
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
      <div class="w-10 h-10 rounded-lg bg-gold/20 flex items-center justify-center">
        <Building size={24} class="text-gold" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">The Philosophy of Structure</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 border-l-4 border-gold space-y-4">
      <p class="text-ui-text-dim leading-relaxed">
        The Argentum Combine is not merely a nation - it is a <span class="text-gold font-semibold">system</span>.
        Every citizen, from the lowliest apprentice to the highest Architect, understands their place in the
        grand machinery of civilization. To outsiders, this rigid hierarchy appears oppressive.
        To the Combine, it is liberation from chaos.
      </p>
      <p class="text-ui-text-dim leading-relaxed">
        They believe in <span class="text-ui-text">The Covenant of Precision</span>: the idea that every individual
        has an optimal role, and society achieves perfection when each person fulfills their function flawlessly.
        A gear that tries to be a spring serves no one. A gear that embraces being a gear becomes essential.
      </p>
      <blockquote class="border-l-2 border-gold/50 pl-4 italic text-ui-text-dim">
        "Chaos is not freedom. Chaos is simply suffering that hasn't found you yet."
        <span class="block text-gold text-sm mt-1">- First Architect Valeria Steelwright</span>
      </blockquote>
    </div>
  </section>

  <!-- Society -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-gold/20 flex items-center justify-center">
        <Users size={24} class="text-gold" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">The People of the Combine</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 space-y-4">
      <p class="text-ui-text-dim leading-relaxed">
        The Combine operates through a meritocratic bureaucracy where advancement is earned through
        demonstrated competence. At age sixteen, every citizen takes <span class="text-gold">The Placement Trials</span> -
        a rigorous series of tests measuring aptitude in engineering, combat, administration, and social coordination.
      </p>

      <!-- Castes -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mt-4">
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-gold font-semibold text-sm">Architects</p>
          <p class="text-ui-text-dim text-xs">Master engineers and leaders in white greatcoats with gold filigree</p>
        </div>
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-gold font-semibold text-sm">Sentinels</p>
          <p class="text-ui-text-dim text-xs">Elite soldiers in gleaming plate, faces hidden beneath standardized helms</p>
        </div>
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-gold font-semibold text-sm">Artificers</p>
          <p class="text-ui-text-dim text-xs">Machine designers with brass goggles and tool-belts</p>
        </div>
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-gold font-semibold text-sm">Wardens</p>
          <p class="text-ui-text-dim text-xs">Guards in bronze shield-armor with geometric patterns</p>
        </div>
      </div>
    </div>
  </section>

  <!-- Constructs -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-gold/20 flex items-center justify-center">
        <Wrench size={24} class="text-gold" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">Relationship with Constructs</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 border-l-4 border-gold">
      <p class="text-ui-text-dim leading-relaxed mb-4">
        The Combine's automatons are not servants - they are <span class="text-ui-text font-semibold">colleagues</span>.
        Every Construct is treated with a peculiar respect that outsiders find confusing. Artificers speak to
        their creations politely. Sentinels fight alongside mechanical units as trusted partners.
        There are even memorial halls for Constructs that fell in battle.
      </p>
      <blockquote class="border-l-2 border-gold/50 pl-4 italic text-ui-text-dim">
        "The Iron Defender doesn't feel fear. But according to the manual, it does feel 'Pride in a job well done.'
        I've worked with this unit for thirty years. The manual is correct."
        <span class="block text-gold text-sm mt-1">- Senior Artificer Brennan Cole</span>
      </blockquote>
    </div>
  </section>

  <!-- Keywords -->
  <section class="bg-gradient-to-r from-gold/10 to-transparent rounded-lg p-5 border border-gold/30">
    <h4 class="text-ui-text font-semibold mb-3">Signature Keywords</h4>
    <div class="flex flex-wrap gap-3">
      <div class="flex items-center gap-2 bg-ui-bg px-3 py-2 rounded-lg">
        <span class="w-2 h-2 rounded-full bg-cyan-400"></span>
        <span class="text-cyan-400 font-semibold">Guard</span>
        <span class="text-ui-text-dim text-sm">- Protects allies from attacks</span>
      </div>
      <div class="flex items-center gap-2 bg-ui-bg px-3 py-2 rounded-lg">
        <span class="w-2 h-2 rounded-full bg-orange-400"></span>
        <span class="text-orange-400 font-semibold">Piercing</span>
        <span class="text-ui-text-dim text-sm">- Excess damage hits face</span>
      </div>
      <div class="flex items-center gap-2 bg-ui-bg px-3 py-2 rounded-lg">
        <span class="w-2 h-2 rounded-full bg-blue-300"></span>
        <span class="text-blue-300 font-semibold">Shield</span>
        <span class="text-ui-text-dim text-sm">- Blocks first damage</span>
      </div>
    </div>
  </section>
</div>
