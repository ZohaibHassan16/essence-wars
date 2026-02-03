<script lang="ts">
  import { Skull, FlaskConical, Building2, Sparkles } from "lucide-svelte";

  // Featured card IDs for Obsidion (3000-3074)
  const featuredCards = [
    { id: 3000, name: "Blood Cultist", desc: "Power at any price" },
    { id: 3060, name: "Archon Eternal", desc: "Death is merely optional" },
    { id: 3021, name: "Vampire Lord", desc: "Sustained by stolen life" },
    { id: 3034, name: "Storm Mage", desc: "Lightning made manifest" },
  ];
</script>

<div class="space-y-8">
  <!-- Header -->
  <section class="text-center">
    <div class="w-20 h-20 mx-auto rounded-2xl bg-gradient-to-br from-red-900 to-blue-900 flex items-center justify-center mb-4 shadow-lg">
      <Skull size={48} class="text-red-400" />
    </div>
    <h2 class="text-3xl font-bold text-red-400 mb-2">The Obsidion Syndicate</h2>
    <p class="text-xl text-ui-text-dim italic">"Ambition Unbound."</p>
  </section>

  <!-- Card Art Gallery -->
  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4 flex items-center gap-2">
      <Skull size={20} class="text-red-400" />
      Masters of Dark Arts
    </h3>
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      {#each featuredCards as card (card.id)}
        <div class="group relative">
          <div class="rounded-lg overflow-hidden border-2 border-red-500/30 hover:border-red-500 transition-all shadow-lg">
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
      <div class="w-10 h-10 rounded-lg bg-red-500/20 flex items-center justify-center">
        <FlaskConical size={24} class="text-red-400" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">The Philosophy of Transcendence</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 border-l-4 border-red-500 space-y-4">
      <p class="text-ui-text-dim leading-relaxed">
        The Obsidion Syndicate is not a nation - it is a <span class="text-red-400 font-semibold">conspiracy</span>.
        A network of scholars, nobles, and visionaries who long ago decided that the limitations of flesh,
        morality, and law were obstacles to be overcome, not boundaries to be respected.
      </p>
      <p class="text-ui-text-dim leading-relaxed">
        Every member has made <span class="text-ui-text">The Bargain</span>: the conscious choice to sacrifice
        something - health, years of life, pieces of their soul - in exchange for power. This is not coercion.
        This is <em>ambition</em>. The Syndicate does not recruit the desperate; it recruits the <span class="text-red-400">hungry</span>.
      </p>
      <blockquote class="border-l-2 border-red-500/50 pl-4 italic text-ui-text-dim">
        "The Combine fears what we do. The Circles are disgusted by it. But when the lights go out
        and the monsters come, who do they call? Us. Because we're not afraid of the dark.
        We learned to see in it."
        <span class="block text-red-400 text-sm mt-1">- Archon Severin Blackwood</span>
      </blockquote>
    </div>
  </section>

  <!-- Society -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-red-500/20 flex items-center justify-center">
        <Building2 size={24} class="text-red-400" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">Life in the Obsidion Spires</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 space-y-4">
      <p class="text-ui-text-dim leading-relaxed">
        The Syndicate's cities rise from the frozen north: <span class="text-red-400">impossibly tall black spires
        that seem to drink the light around them</span>. These towers are built not for efficiency or growth,
        but for <em>secrecy and power</em>. Gargoyles that might be alive, shadows that definitely move,
        and architecture that seems to shift when you're not looking.
      </p>

      <!-- Castes -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mt-4">
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-red-400 font-semibold text-sm">Archons</p>
          <p class="text-ui-text-dim text-xs">Ancient council leaders, often undead, with essence veins glowing beneath skin</p>
        </div>
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-red-400 font-semibold text-sm">Hemomancers</p>
          <p class="text-ui-text-dim text-xs">Blood magic specialists with tubes and syringes in formal wear</p>
        </div>
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-red-400 font-semibold text-sm">Storm Mages</p>
          <p class="text-ui-text-dim text-xs">Combat spellcasters with crackling gauntlets and burn scars</p>
        </div>
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-red-400 font-semibold text-sm">Shadow Walkers</p>
          <p class="text-ui-text-dim text-xs">Assassins in black, faces obscured by living shadows</p>
        </div>
      </div>
    </div>
  </section>

  <!-- Relationship with Death -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-red-500/20 flex items-center justify-center">
        <Sparkles size={24} class="text-red-400" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">Death is Optional</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 border-l-4 border-red-500">
      <p class="text-ui-text-dim leading-relaxed mb-4">
        The Syndicate has a unique relationship with mortality: they consider it <span class="text-ui-text font-semibold">optional</span>.
        Through essence infusion, soul-binding, hemomancy, and outright necromancy, members can extend their
        lives far beyond natural limits. Some merely age slowly. Others become vampiric hemomancers sustained
        by stolen life-force, or true undead whose souls are anchored to corpses through sheer will.
      </p>
      <p class="text-ui-text-dim leading-relaxed mb-4">
        The Syndicate's undead are not mindless thralls. They are <span class="text-red-400">citizens who refused to die</span>.
        They retain their memories, their personalities, their ambitions. They simply no longer breathe,
        age, or fear the passage of time.
      </p>
      <blockquote class="border-l-2 border-red-500/50 pl-4 italic text-ui-text-dim">
        "You call them 'monsters.' We call them 'senior faculty.' Show some respect -
        that lich was already ancient when your grandmother was born."
        <span class="block text-red-400 text-sm mt-1">- Aspirant Elena Voss</span>
      </blockquote>
    </div>
  </section>

  <!-- Keywords -->
  <section class="bg-gradient-to-r from-red-500/10 to-transparent rounded-lg p-5 border border-red-500/30">
    <h4 class="text-ui-text font-semibold mb-3">Signature Keywords</h4>
    <div class="flex flex-wrap gap-3">
      <div class="flex items-center gap-2 bg-ui-bg px-3 py-2 rounded-lg">
        <span class="w-2 h-2 rounded-full bg-red-400"></span>
        <span class="text-red-400 font-semibold">Lifesteal</span>
        <span class="text-ui-text-dim text-sm">- Heals when dealing damage</span>
      </div>
      <div class="flex items-center gap-2 bg-ui-bg px-3 py-2 rounded-lg">
        <span class="w-2 h-2 rounded-full bg-violet-400"></span>
        <span class="text-violet-400 font-semibold">Stealth</span>
        <span class="text-ui-text-dim text-sm">- Cannot be targeted until attacking</span>
      </div>
      <div class="flex items-center gap-2 bg-ui-bg px-3 py-2 rounded-lg">
        <span class="w-2 h-2 rounded-full bg-green-400"></span>
        <span class="text-green-400 font-semibold">Quick</span>
        <span class="text-ui-text-dim text-sm">- Attacks before enemy can retaliate</span>
      </div>
    </div>
  </section>
</div>
