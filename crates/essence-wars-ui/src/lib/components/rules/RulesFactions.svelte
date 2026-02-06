<script lang="ts">
  import { assetUrl } from "$lib/utils/paths";

  // Faction emblems - computed lazily to avoid SSR/hydration issues
  const factionEmblems = $derived({
    argentum: assetUrl("/ui/decorations/emblems/emblem_argentum.png"),
    symbiote: assetUrl("/ui/decorations/emblems/emblem_symbiote.png"),
    obsidion: assetUrl("/ui/decorations/emblems/emblem_obsidion.png"),
    neutral: assetUrl("/ui/decorations/emblems/emblem_neutral.png"),
  });
</script>

<div class="space-y-8">
  <section>
    <h2 class="text-2xl font-bold text-ui-text mb-4">The Factions of Omyra</h2>
    <p class="text-ui-text-dim leading-relaxed">
      In the Year 302 of the Gilded Truce, four great powers compete for control of
      <span class="text-gold">Essence</span> - the crystallized memory of the planet itself.
      Each faction has a distinct playstyle and signature keywords.
    </p>
  </section>

  <!-- Argentum Combine -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-12 h-12 rounded-lg bg-gradient-to-br from-gray-200 to-gold/50 flex items-center justify-center p-1">
        <img src={factionEmblems.argentum} alt="Argentum" class="w-full h-full object-contain drop-shadow-md" />
      </div>
      <div>
        <h3 class="text-xl font-semibold text-ui-text">The Argentum Combine</h3>
        <p class="text-gold text-sm">"Precision. Protocol. Peace."</p>
      </div>
    </div>

    <div class="bg-ui-bg rounded-lg p-4 space-y-4 border-l-4 border-gold">
      <p class="text-ui-text-dim">
        A nation of engineers and soldiers who believe that <span class="text-ui-text">structure is safety</span>.
        They refine Essence through massive pressure engines, building gleaming cities of
        white marble and polished brass. Their automatons fight alongside human soldiers in
        perfect formation.
      </p>

      <div>
        <p class="text-ui-text font-semibold text-sm mb-2">Signature Keywords</p>
        <div class="flex flex-wrap gap-2">
          <span class="px-2 py-1 bg-cyan-400/20 text-cyan-400 rounded text-xs font-semibold">Guard</span>
          <span class="px-2 py-1 bg-orange-400/20 text-orange-400 rounded text-xs font-semibold">Piercing</span>
          <span class="px-2 py-1 bg-blue-300/20 text-blue-300 rounded text-xs font-semibold">Shield</span>
        </div>
      </div>

      <div>
        <p class="text-ui-text font-semibold text-sm mb-2">Playstyle</p>
        <p class="text-ui-text-dim text-sm">
          Defensive and methodical. Build walls of Guard creatures, outlast opponents,
          and grind them down with efficient trades. Strong late-game with powerful constructs.
        </p>
      </div>

      <p class="text-ui-text-dim text-xs italic border-t border-gray-700 pt-3">
        "The world is chaotic. We bring order. Every gear has its place,
        every soldier their purpose."
      </p>
    </div>
  </section>

  <!-- Symbiote Circles -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-12 h-12 rounded-lg bg-gradient-to-br from-green-800 to-purple-600 flex items-center justify-center p-1">
        <img src={factionEmblems.symbiote} alt="Symbiote" class="w-full h-full object-contain drop-shadow-md" />
      </div>
      <div>
        <h3 class="text-xl font-semibold text-ui-text">The Symbiote Circles</h3>
        <p class="text-green-400 text-sm">"The Pack Endures. Hunt as One."</p>
      </div>
    </div>

    <div class="bg-ui-bg rounded-lg p-4 space-y-4 border-l-4 border-green-500">
      <p class="text-ui-text-dim">
        A civilization where the bond between humanoid and beast has been elevated to sacred art.
        They don't build machines - they <span class="text-ui-text">grow with</span> the forest.
        Ancient groves and living tree-cities house warriors bonded with wolves, great cats, and serpents,
        their leather and fur armor marking their pack allegiance.
      </p>

      <div>
        <p class="text-ui-text font-semibold text-sm mb-2">Signature Keywords</p>
        <div class="flex flex-wrap gap-2">
          <span class="px-2 py-1 bg-yellow-400/20 text-yellow-400 rounded text-xs font-semibold">Rush</span>
          <span class="px-2 py-1 bg-purple-400/20 text-purple-400 rounded text-xs font-semibold">Lethal</span>
          <span class="px-2 py-1 bg-emerald-400/20 text-emerald-400 rounded text-xs font-semibold">Regenerate</span>
        </div>
      </div>

      <div>
        <p class="text-ui-text font-semibold text-sm mb-2">Playstyle</p>
        <p class="text-ui-text-dim text-sm">
          Aggressive tempo. Flood the board with pack creatures, push damage early with Rush,
          and use Lethal predators to remove threats. Overwhelm before opponents stabilize.
        </p>
      </div>

      <p class="text-ui-text-dim text-xs italic border-t border-gray-700 pt-3">
        "In the pack, there is no 'us' and 'them.' There is only the hunt, and those who run together."
      </p>
    </div>
  </section>

  <!-- Obsidion Syndicate -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-12 h-12 rounded-lg bg-gradient-to-br from-red-900 to-blue-900 flex items-center justify-center p-1">
        <img src={factionEmblems.obsidion} alt="Obsidion" class="w-full h-full object-contain drop-shadow-md" />
      </div>
      <div>
        <h3 class="text-xl font-semibold text-ui-text">The Obsidion Syndicate</h3>
        <p class="text-red-400 text-sm">"Ambition Unbound."</p>
      </div>
    </div>

    <div class="bg-ui-bg rounded-lg p-4 space-y-4 border-l-4 border-red-500">
      <p class="text-ui-text-dim">
        A conspiracy of scholars and visionaries who inject Essence directly into their veins,
        bind it to their souls, or drink it like wine. Their gothic spires rise from the frozen north,
        where <span class="text-ui-text">death is considered optional</span>.
        Hemomancers, storm mages, and the walking dead serve alongside the living.
      </p>

      <div>
        <p class="text-ui-text font-semibold text-sm mb-2">Signature Keywords</p>
        <div class="flex flex-wrap gap-2">
          <span class="px-2 py-1 bg-red-400/20 text-red-400 rounded text-xs font-semibold">Lifesteal</span>
          <span class="px-2 py-1 bg-violet-400/20 text-violet-400 rounded text-xs font-semibold">Stealth</span>
          <span class="px-2 py-1 bg-green-400/20 text-green-400 rounded text-xs font-semibold">Quick</span>
        </div>
      </div>

      <div>
        <p class="text-ui-text font-semibold text-sm mb-2">Playstyle</p>
        <p class="text-ui-text-dim text-sm">
          Burst damage and sustain. Set up devastating combos, drain life to stay alive,
          and strike from the shadows with Stealth. High risk, high reward.
        </p>
      </div>

      <p class="text-ui-text-dim text-xs italic border-t border-gray-700 pt-3">
        "Power has a price. We've simply learned not to flinch when paying it."
      </p>
    </div>
  </section>

  <!-- Free-Walkers -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-12 h-12 rounded-lg bg-gradient-to-br from-amber-700 to-stone-600 flex items-center justify-center p-1">
        <img src={factionEmblems.neutral} alt="Free-Walkers" class="w-full h-full object-contain drop-shadow-md" />
      </div>
      <div>
        <h3 class="text-xl font-semibold text-ui-text">The Free-Walkers</h3>
        <p class="text-amber-400 text-sm">"No Flag. Just Gold."</p>
      </div>
    </div>

    <div class="bg-ui-bg rounded-lg p-4 space-y-4 border-l-4 border-amber-500">
      <p class="text-ui-text-dim">
        Mercenaries, traders, and the massive Stone Giants who were too big for civilization.
        They live in the Unclaimed Zones between the great powers, wearing patchwork armor
        from all factions, <span class="text-ui-text">fighting for whoever pays the most</span>.
        The Free-Walkers have no loyalty except to gold and survival.
      </p>

      <div>
        <p class="text-ui-text font-semibold text-sm mb-2">Signature Keywords</p>
        <div class="flex flex-wrap gap-2">
          <span class="px-2 py-1 bg-blue-400/20 text-blue-400 rounded text-xs font-semibold">Ranged</span>
          <span class="px-2 py-1 bg-amber-400/20 text-amber-400 rounded text-xs font-semibold">Charge</span>
        </div>
      </div>

      <div>
        <p class="text-ui-text font-semibold text-sm mb-2">Playstyle</p>
        <p class="text-ui-text-dim text-sm">
          Neutral cards that fit any deck. Ranged snipers pick off threats, Giants dominate
          the late game, and supports provide utility. Splash into any faction strategy.
        </p>
      </div>

      <p class="text-ui-text-dim text-xs italic border-t border-gray-700 pt-3">
        "They call us mercenaries like it's an insult. But I've never burned a city
        because someone told me to. I just work for money."
      </p>
    </div>
  </section>

  <!-- The World -->
  <section class="bg-ui-bg/50 rounded-lg p-4 border border-gray-700">
    <h4 class="text-ui-text font-semibold mb-3">The World of Omyra</h4>
    <p class="text-ui-text-dim text-sm mb-3">
      Three centuries ago, the <span class="text-damage">Calamity of Silence</span> nearly
      destroyed the world. A massive Essence eruption from the planet's crystalline core
      erased technologies and collapsed civilizations. The survivors established the
      <span class="text-gold">Gilded Truce</span> - an agreement to limit warfare to
      controlled "Extraction Disputes" over Essence veins.
    </p>
    <p class="text-ui-text-dim text-sm">
      Now, in Year 302 of the Truce, the veins run thinner and the disputes grow bloodier.
      As a Commander, you direct your forces from high-orbit airships, using Farsight technology
      to see everything on the battlefield with perfect clarity.
    </p>
  </section>
</div>
