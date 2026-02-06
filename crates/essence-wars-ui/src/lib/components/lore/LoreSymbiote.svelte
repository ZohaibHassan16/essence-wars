<script lang="ts">
  import { Leaf, TreePine, Heart } from "lucide-svelte";

  const emblem = "/ui/decorations/emblems/emblem_symbiote.png";

  // Featured card IDs for Symbiote (2000-2074)
  const featuredCards = [
    { id: 2000, name: "Spore Crawler", desc: "Small but deadly" },
    { id: 2060, name: "Pack Matriarch", desc: "Alpha of the pack" },
    { id: 2011, name: "Swift Stalker", desc: "Swift and relentless" },
    { id: 2045, name: "Apex Predator", desc: "Nature's perfect hunter" },
  ];
</script>

<div class="space-y-8">
  <!-- Header -->
  <section class="text-center">
    <div class="w-20 h-20 mx-auto rounded-2xl bg-gradient-to-br from-green-800 to-purple-600 flex items-center justify-center mb-4 shadow-lg p-2">
      <img src={emblem} alt="Symbiote" class="w-full h-full object-contain drop-shadow-md" />
    </div>
    <h2 class="text-3xl font-bold text-green-400 mb-2">The Symbiote Circles</h2>
    <p class="text-xl text-ui-text-dim italic">"The Pack Endures. Hunt as One."</p>
  </section>

  <!-- Card Art Gallery -->
  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4 flex items-center gap-2">
      <img src={emblem} alt="" class="w-6 h-6 object-contain" />
      Children of the Pack
    </h3>
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      {#each featuredCards as card (card.id)}
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
        <Leaf size={24} class="text-green-400" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">The Philosophy of the Pack</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 border-l-4 border-green-500 space-y-4">
      <p class="text-ui-text-dim leading-relaxed">
        The Symbiote Circles are not savages - they are an ancient civilization where the bond between
        humanoid and nature has been elevated to sacred art. Every member has undergone
        <span class="text-green-400 font-semibold">The Pack Bond</span>, a ritual communion that
        links them spiritually to the great forests and the creatures within.
      </p>
      <p class="text-ui-text-dim leading-relaxed">
        To outsiders, the Circles appear wild and dangerous. To the Circles, they are simply <span class="text-ui-text">whole</span>.
        The Combine locks itself in metal boxes. The Syndicate poisons itself with dark magic.
        Only the Circles remember what it means to be part of the living world.
      </p>
      <blockquote class="border-l-2 border-green-500/50 pl-4 italic text-ui-text-dim">
        "You ask if our beasts obey us. You ask the wrong question.
        In the pack, there is no 'us' and 'them.' There is only the hunt, and those who run together."
        <span class="block text-green-400 text-sm mt-1">- Pack Alpha Kira Wolfborn</span>
      </blockquote>
    </div>
  </section>

  <!-- Society -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-green-500/20 flex items-center justify-center">
        <TreePine size={24} class="text-green-400" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">Life in the Primordial Forests</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 space-y-4">
      <p class="text-ui-text-dim leading-relaxed">
        The Circles do not build cities - they <span class="text-green-400">grow with them</span>. Their communities
        exist within and around enormous ancient trees, in groves where Essence flows so strongly the forest itself seems alive.
        Walkways are woven from living branches. Homes are hollows shaped by generations of careful tending.
      </p>

      <!-- Roles -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mt-4">
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-green-400 font-semibold text-sm">Pack Alphas</p>
          <p class="text-ui-text-dim text-xs">War leaders in wolf pelts with dire wolf companions</p>
        </div>
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-green-400 font-semibold text-sm">Beast Wardens</p>
          <p class="text-ui-text-dim text-xs">Hunters bonded to great cats and wolf packs</p>
        </div>
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-green-400 font-semibold text-sm">Grove Keepers</p>
          <p class="text-ui-text-dim text-xs">Forest guardians with healing magic and living wood staffs</p>
        </div>
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-green-400 font-semibold text-sm">Circle Shamans</p>
          <p class="text-ui-text-dim text-xs">Spiritual leaders with ancestor spirits and bone masks</p>
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
        of the pack. A Beast Warden does not "command" her wolf pack; she <em>runs with</em> them,
        sensing through their keen senses while her humanoid form guides the hunt.
      </p>
      <p class="text-ui-text-dim leading-relaxed mb-4">
        When a Circle Warrior bonds with a great cat, the predator can feel loyalty, hunger, and purpose.
        The "Ancient Treant" that the Argentum Combine fears is not a mindless plant - it contains a Grove Keeper
        who has merged with it completely, their consciousness flowing through root and branch.
      </p>
      <blockquote class="border-l-2 border-green-500/50 pl-4 italic text-ui-text-dim">
        "The Combine builds walls of steel. The forest grows through the cracks."
        <span class="block text-green-400 text-sm mt-1">- Circle Grove Keeper</span>
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
