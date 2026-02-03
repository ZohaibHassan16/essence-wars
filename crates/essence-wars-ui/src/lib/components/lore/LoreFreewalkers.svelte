<script lang="ts">
  import { Compass, Users, Mountain, Coins } from "lucide-svelte";

  // Featured card IDs for Free-Walkers/Neutral (4000-4074)
  const featuredCards = [
    { id: 4027, name: "Sniper Marksman", desc: "Precision from afar" },
    { id: 4060, name: "Warlord Titan", desc: "A walking earthquake" },
    { id: 4040, name: "War Drums", desc: "The rhythm of battle" },
    { id: 4015, name: "Caravan Guard", desc: "Protection for a price" },
  ];
</script>

<div class="space-y-8">
  <!-- Header -->
  <section class="text-center">
    <div class="w-20 h-20 mx-auto rounded-2xl bg-gradient-to-br from-amber-700 to-stone-600 flex items-center justify-center mb-4 shadow-lg">
      <Compass size={48} class="text-amber-400" />
    </div>
    <h2 class="text-3xl font-bold text-amber-400 mb-2">The Free-Walkers</h2>
    <p class="text-xl text-ui-text-dim italic">"No Flag. Just Gold."</p>
  </section>

  <!-- Card Art Gallery -->
  <section>
    <h3 class="text-xl font-semibold text-ui-text mb-4 flex items-center gap-2">
      <Compass size={20} class="text-amber-400" />
      Soldiers of Fortune
    </h3>
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      {#each featuredCards as card (card.id)}
        <div class="group relative">
          <div class="rounded-lg overflow-hidden border-2 border-amber-500/30 hover:border-amber-500 transition-all shadow-lg">
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
      <div class="w-10 h-10 rounded-lg bg-amber-500/20 flex items-center justify-center">
        <Coins size={24} class="text-amber-400" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">The Philosophy of Independence</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 border-l-4 border-amber-500 space-y-4">
      <p class="text-ui-text-dim leading-relaxed">
        The Free-Walkers are not a faction - they are an <span class="text-amber-400 font-semibold">absence</span> of faction.
        They are the people who looked at the Combine's order, the Circles' communion, and the Syndicate's bargains
        and said: <em>No thank you. I'll make my own way.</em>
      </p>
      <p class="text-ui-text-dim leading-relaxed">
        Some are exiles. Some are deserters. Some are simply souls born in the wrong place, who found their way
        to the Unclaimed Zones and discovered they could breathe for the first time. They have no unifying philosophy
        except <span class="text-ui-text">survival</span>, no shared culture except <span class="text-ui-text">pragmatism</span>,
        and no loyalty except to <span class="text-amber-400">whoever's paying</span>.
      </p>
      <blockquote class="border-l-2 border-amber-500/50 pl-4 italic text-ui-text-dim">
        "They call us mercenaries like it's an insult. But I've never burned a city because someone told me to.
        I've never injected my children with poison for 'evolution.' I've never tortured my soul for power.
        I just work for money. Who's the monster here?"
        <span class="block text-amber-400 text-sm mt-1">- Caravan Master Jenna Dustwalker</span>
      </blockquote>
    </div>
  </section>

  <!-- The Giants -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-amber-500/20 flex items-center justify-center">
        <Mountain size={24} class="text-amber-400" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">The Stone Giants</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 border-l-4 border-amber-500">
      <p class="text-ui-text-dim leading-relaxed mb-4">
        The most distinctive feature of Free-Walker society is the <span class="text-amber-400 font-semibold">Stone Giants</span> -
        massive humanoids who stand eight to twelve feet tall, remnants of a pre-Calamity civilization that the
        other factions have forgotten (or deliberately erased from history).
      </p>
      <p class="text-ui-text-dim leading-relaxed mb-4">
        The Giants are not a separate species; they are simply <em>large</em>. Their ancestors were workers -
        miners, builders, haulers - engineered in the ancient past. Too big for Combine cities. Too individualistic
        for Circle communion. Too simple (they say) for Syndicate schemes. So they walked. And kept walking
        until they found the Unclaimed Zones.
      </p>
      <blockquote class="border-l-2 border-amber-500/50 pl-4 italic text-ui-text-dim">
        "People assume I'm stupid because I'm big. That's fine. I assume they're small because they're petty.
        We're both usually right."
        <span class="block text-amber-400 text-sm mt-1">- Titan Ironhand, Warlord-for-Hire</span>
      </blockquote>
    </div>
  </section>

  <!-- Society -->
  <section>
    <div class="flex items-center gap-3 mb-4">
      <div class="w-10 h-10 rounded-lg bg-amber-500/20 flex items-center justify-center">
        <Users size={24} class="text-amber-400" />
      </div>
      <h3 class="text-xl font-semibold text-ui-text">The Guilds of the Zones</h3>
    </div>

    <div class="bg-ui-bg rounded-lg p-5 space-y-4">
      <p class="text-ui-text-dim leading-relaxed">
        The Free-Walkers have no government, no hierarchy, and no laws - except the ones they enforce themselves.
        What they have instead are <span class="text-amber-400">Guilds</span>: loose associations of professionals
        who share resources, reputation, and protection.
      </p>

      <!-- Guilds -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3 mt-4">
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-amber-400 font-semibold text-sm">The Iron Caravan</p>
          <p class="text-ui-text-dim text-xs">Traders with patchwork wagons and a mix of all faction tech</p>
        </div>
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-amber-400 font-semibold text-sm">The Long Rifles</p>
          <p class="text-ui-text-dim text-xs">Elite snipers in desert cloaks with massive custom rifles</p>
        </div>
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-amber-400 font-semibold text-sm">The Drum Corps</p>
          <p class="text-ui-text-dim text-xs">Battlefield support with mobile sound platforms and war-horns</p>
        </div>
        <div class="bg-ui-panel/50 rounded p-3 border border-gray-700">
          <p class="text-amber-400 font-semibold text-sm">The Reclaimers</p>
          <p class="text-ui-text-dim text-xs">Salvagers with maps to pre-Calamity ruins</p>
        </div>
      </div>
    </div>
  </section>

  <!-- Life in the Zones -->
  <section class="bg-ui-bg rounded-lg p-5 border border-gray-700">
    <h4 class="text-amber-400 font-semibold mb-3">Life in the Unclaimed Zones</h4>
    <p class="text-ui-text-dim leading-relaxed mb-3">
      The Unclaimed Zones are the spaces between: <span class="text-ui-text">deserts, badlands, mountain passes,
      and ruins</span> that no faction wants badly enough to fight for. They are harsh, dangerous, and <em>free</em>.
    </p>
    <p class="text-ui-text-dim leading-relaxed mb-3">
      Free-Walker "towns" are really <span class="text-amber-400">trade posts</span> - places where caravans stop,
      Guilds recruit, and contracts are negotiated. The largest is <span class="text-ui-text font-semibold">Last Hope</span>,
      a ramshackle city built in pre-Calamity ruins, where all three factions maintain quiet embassies
      and pretend not to see each other.
    </p>
    <blockquote class="border-l-2 border-amber-500/50 pl-4 italic text-ui-text-dim text-sm">
      "The Combine raises children to be components. The Circles raise children to be modified.
      The Syndicate raises children to be recruited. We raise children to be themselves.
      Dangerous, free, and ready for anything."
      <span class="block text-amber-400 text-xs mt-1">- Matriarch Chen, Last Hope Schoolkeeper</span>
    </blockquote>
  </section>

  <!-- Keywords -->
  <section class="bg-gradient-to-r from-amber-500/10 to-transparent rounded-lg p-5 border border-amber-500/30">
    <h4 class="text-ui-text font-semibold mb-3">Signature Keywords</h4>
    <div class="flex flex-wrap gap-3">
      <div class="flex items-center gap-2 bg-ui-bg px-3 py-2 rounded-lg">
        <span class="w-2 h-2 rounded-full bg-blue-400"></span>
        <span class="text-blue-400 font-semibold">Ranged</span>
        <span class="text-ui-text-dim text-sm">- Attacks without taking damage</span>
      </div>
      <div class="flex items-center gap-2 bg-ui-bg px-3 py-2 rounded-lg">
        <span class="w-2 h-2 rounded-full bg-amber-400"></span>
        <span class="text-amber-400 font-semibold">Charge</span>
        <span class="text-ui-text-dim text-sm">- Gains attack when played</span>
      </div>
    </div>
    <p class="text-ui-text-dim text-sm mt-3">
      Free-Walker cards are neutral and can be added to any faction deck, providing utility and filling gaps in your strategy.
    </p>
  </section>
</div>
