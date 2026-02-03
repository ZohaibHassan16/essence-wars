<script lang="ts">
  import { playSound } from "$lib/audio";
  import LoreWorld from "./lore/LoreWorld.svelte";
  import LoreArgentum from "./lore/LoreArgentum.svelte";
  import LoreSymbiote from "./lore/LoreSymbiote.svelte";
  import LoreObsidion from "./lore/LoreObsidion.svelte";
  import LoreFreewalkers from "./lore/LoreFreewalkers.svelte";
  import { Cog, Bug, Skull, Compass, Globe } from "lucide-svelte";

  let { onBack }: { onBack: () => void } = $props();

  type Tab = "world" | "argentum" | "symbiote" | "obsidion" | "freewalkers";

  let activeTab = $state<Tab>("world");

  const tabs: { id: Tab; label: string; icon: typeof Globe; color: string }[] = [
    { id: "world", label: "Omyra", icon: Globe, color: "text-blue-400" },
    { id: "argentum", label: "Argentum", icon: Cog, color: "text-gold" },
    { id: "symbiote", label: "Symbiote", icon: Bug, color: "text-green-400" },
    { id: "obsidion", label: "Obsidion", icon: Skull, color: "text-red-400" },
    { id: "freewalkers", label: "Free-Walkers", icon: Compass, color: "text-amber-400" },
  ];

  function handleButtonHover() {
    playSound('buttonHover');
  }

  function handleButtonClick() {
    playSound('buttonClick');
  }

  function selectTab(tab: Tab) {
    handleButtonClick();
    activeTab = tab;
  }
</script>

<div class="h-screen flex flex-col p-4 md:p-8 overflow-hidden">
  <!-- Header with back button -->
  <div class="flex-shrink-0 flex items-center gap-4 mb-6">
    <button
      class="px-4 py-2 bg-ui-panel text-ui-text rounded-lg font-semibold
             border border-gray-600 hover:border-ui-action hover:text-ui-action transition-all"
      onclick={() => {
        handleButtonClick();
        onBack();
      }}
      onmouseenter={handleButtonHover}
    >
      Back
    </button>
    <h1 class="text-3xl font-bold text-ui-text">Lore & World</h1>
  </div>

  <!-- Tab navigation with icons -->
  <div class="flex-shrink-0 flex flex-wrap gap-2 mb-6 border-b border-gray-700 pb-4">
    {#each tabs as tab (tab.id)}
      {@const Icon = tab.icon}
      <button
        class="px-4 py-2 rounded-lg font-semibold transition-all flex items-center gap-2
               {activeTab === tab.id
                 ? 'bg-ui-action text-white'
                 : 'bg-ui-panel text-ui-text border border-gray-600 hover:border-gray-500 hover:text-ui-text'}"
        onclick={() => selectTab(tab.id)}
        onmouseenter={handleButtonHover}
      >
        <Icon size={18} class={activeTab === tab.id ? 'text-white' : tab.color} />
        {tab.label}
      </button>
    {/each}
  </div>

  <!-- Tab content (scrollable) -->
  <div class="flex-1 min-h-0 overflow-y-auto">
    <div class="max-w-5xl mx-auto bg-ui-panel rounded-xl p-6 md:p-8 shadow-2xl mb-4">
      {#if activeTab === "world"}
        <LoreWorld />
      {:else if activeTab === "argentum"}
        <LoreArgentum />
      {:else if activeTab === "symbiote"}
        <LoreSymbiote />
      {:else if activeTab === "obsidion"}
        <LoreObsidion />
      {:else if activeTab === "freewalkers"}
        <LoreFreewalkers />
      {/if}
    </div>
  </div>
</div>
