<script lang="ts">
  import { Globe, Sparkles, Shield, Swords } from "lucide-svelte";
  import { assetUrl } from "$lib/utils/paths";
</script>

<div class="space-y-8">
  <!-- Hero section with world map -->
  <section class="relative">
    <div class="rounded-xl overflow-hidden border border-gray-700 shadow-2xl">
      <img
        src={assetUrl("/backgrounds/omyra_world_map.webp")}
        alt="The World of Omyra"
        class="w-full h-64 md:h-96 object-cover"
      />
      <div class="absolute inset-0 bg-gradient-to-t from-ui-panel via-transparent to-transparent"></div>
      <div class="absolute bottom-0 left-0 right-0 p-6">
        <h2 class="text-3xl font-bold text-white drop-shadow-lg">The World of Omyra</h2>
        <p class="text-gray-200 text-lg drop-shadow">Year 302 of the Gilded Truce</p>
      </div>
    </div>
  </section>

  <!-- Essence explanation -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
        <Sparkles size={24} class="text-white" />
      </div>
      <h3 class="text-2xl font-bold text-ui-text">The Nature of Essence</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 border-l-4 border-blue-500">
      <p class="text-ui-text-dim leading-relaxed mb-4">
        In Omyra, Essence isn't just magical energy - it is <span class="text-gold font-semibold">Geological Memory</span>.
        The planet's crystalline core records history itself. Every few centuries, tectonic shifts push
        "Raw Essence" to the surface in the form of <span class="text-blue-400">Prismatic Geodes</span>.
      </p>
      <p class="text-ui-text-dim leading-relaxed mb-4">
        This isn't merely fuel - it's <span class="text-ui-text">information</span>. Refining Essence allows
        civilizations to not only power machines but to "remember" lost technologies from the ancient past.
        Whoever controls the Essence veins controls the future.
      </p>
      <p class="text-ui-text-dim leading-relaxed">
        The conflicts you witness aren't total war - they are <span class="text-amber-400">"Extraction Disputes"</span>.
        When a new Essence vein erupts, factions rush to secure the site. The fighting is regulated,
        professional, and often treated like a high-stakes sport by civilians back home.
      </p>
    </div>
  </section>

  <!-- The Gilded Truce -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-gradient-to-br from-gold to-amber-600 flex items-center justify-center">
        <Shield size={24} class="text-white" />
      </div>
      <h3 class="text-2xl font-bold text-ui-text">The Gilded Truce</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 border-l-4 border-gold">
      <p class="text-ui-text-dim leading-relaxed mb-4">
        We are in the <span class="text-gold font-semibold">Year 302 of the Gilded Truce</span>.
        The massive wars of the past - the terrifying <span class="text-damage">"Calamity of Silence"</span> -
        almost broke the planet itself. Entire technologies were lost. Languages vanished.
        The Giants forgot why they were made.
      </p>
      <p class="text-ui-text-dim leading-relaxed mb-4">
        The survivors looked at the devastation and agreed: <span class="text-ui-text font-semibold">never again</span>.
        The factions were formalized, territorial boundaries drawn, and the rules of "civilized conflict" established.
        Wars would be fought over Essence veins, not ideology. Cities would be spared.
      </p>
      <p class="text-ui-text-dim leading-relaxed">
        For 302 years, the Truce has held. But the Essence veins are running thinner.
        The Extraction Disputes are getting bloodier. And somewhere in the frozen north,
        the Obsidion Syndicate has been digging deeper than anyone knows...
      </p>
    </div>
  </section>

  <!-- The Factions overview -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-gradient-to-br from-gray-600 to-gray-800 flex items-center justify-center">
        <Swords size={24} class="text-white" />
      </div>
      <h3 class="text-2xl font-bold text-ui-text">The Four Powers</h3>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <!-- Argentum -->
      <div class="bg-ui-bg rounded-lg p-4 border border-gray-700 hover:border-gold transition-colors">
        <div class="flex items-center gap-2 mb-2">
          <div class="w-3 h-3 rounded-full bg-gold"></div>
          <h4 class="font-semibold text-gold">The Argentum Combine</h4>
        </div>
        <p class="text-ui-text-dim text-sm">
          Western Plateaus. Cities of brass and white marble. Order, industry, and clockwork precision.
        </p>
      </div>

      <!-- Symbiote -->
      <div class="bg-ui-bg rounded-lg p-4 border border-gray-700 hover:border-green-500 transition-colors">
        <div class="flex items-center gap-2 mb-2">
          <div class="w-3 h-3 rounded-full bg-green-500"></div>
          <h4 class="font-semibold text-green-400">The Symbiote Circles</h4>
        </div>
        <p class="text-ui-text-dim text-sm">
          Eastern Jungles. Living cities grown from coral and Spire Trees. Evolution made manifest.
        </p>
      </div>

      <!-- Obsidion -->
      <div class="bg-ui-bg rounded-lg p-4 border border-gray-700 hover:border-red-500 transition-colors">
        <div class="flex items-center gap-2 mb-2">
          <div class="w-3 h-3 rounded-full bg-red-500"></div>
          <h4 class="font-semibold text-red-400">The Obsidion Syndicate</h4>
        </div>
        <p class="text-ui-text-dim text-sm">
          Northern Glacial Peaks. Gothic spires of black stone. Knowledge, ambition, and dark bargains.
        </p>
      </div>

      <!-- Free-Walkers -->
      <div class="bg-ui-bg rounded-lg p-4 border border-gray-700 hover:border-amber-500 transition-colors">
        <div class="flex items-center gap-2 mb-2">
          <div class="w-3 h-3 rounded-full bg-amber-500"></div>
          <h4 class="font-semibold text-amber-400">The Free-Walkers</h4>
        </div>
        <p class="text-ui-text-dim text-sm">
          The Trade Roads between. Mercenaries, Giants, and those who answer to no flag but gold.
        </p>
      </div>
    </div>
  </section>

  <!-- Commander context -->
  <section class="bg-gradient-to-r from-ui-bg to-ui-panel rounded-lg p-5 border border-gray-700">
    <h4 class="text-ui-text font-semibold mb-3 flex items-center gap-2">
      <Globe size={18} class="text-blue-400" />
      Your Role as Commander
    </h4>
    <p class="text-ui-text-dim text-sm leading-relaxed">
      You are a <span class="text-ui-text">Commander</span> - directing forces from high-orbit airships or using
      "Farsight Tables" to observe the battlefield. In this era of high science, sensors are perfect.
      You can see exactly what the enemy has deployed. The strategy isn't about <em>hiding</em> -
      it's about <span class="text-gold">calculating</span> the outcome better than they can.
    </p>
  </section>
</div>
