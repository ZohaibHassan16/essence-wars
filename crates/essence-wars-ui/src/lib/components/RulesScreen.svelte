<script lang="ts">
  import { playSound } from "$lib/audio";
  import RulesOverview from "./rules/RulesOverview.svelte";
  import RulesTurnStructure from "./rules/RulesTurnStructure.svelte";
  import RulesCardTypes from "./rules/RulesCardTypes.svelte";
  import RulesKeywords from "./rules/RulesKeywords.svelte";
  import RulesCombat from "./rules/RulesCombat.svelte";
  import RulesFactions from "./rules/RulesFactions.svelte";

  let { onBack }: { onBack: () => void } = $props();

  type Tab = "overview" | "turns" | "cards" | "keywords" | "combat" | "factions";

  let activeTab = $state<Tab>("overview");

  const tabs: { id: Tab; label: string }[] = [
    { id: "overview", label: "Overview" },
    { id: "turns", label: "Turns" },
    { id: "cards", label: "Card Types" },
    { id: "keywords", label: "Keywords" },
    { id: "combat", label: "Combat" },
    { id: "factions", label: "Factions" },
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
    <h1 class="text-3xl font-bold text-ui-text">Rules & Guide</h1>
  </div>

  <!-- Tab navigation -->
  <div class="flex-shrink-0 flex flex-wrap gap-2 mb-6 border-b border-gray-700 pb-4">
    {#each tabs as tab (tab.id)}
      <button
        class="px-4 py-2 rounded-lg font-semibold transition-all
               {activeTab === tab.id
                 ? 'bg-ui-action text-white'
                 : 'bg-ui-panel text-ui-text border border-gray-600 hover:border-gray-500 hover:text-ui-text'}"
        onclick={() => selectTab(tab.id)}
        onmouseenter={handleButtonHover}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <!-- Tab content (scrollable) -->
  <div class="flex-1 min-h-0 overflow-y-auto">
    <div class="max-w-4xl mx-auto bg-ui-panel rounded-xl p-6 md:p-8 shadow-2xl mb-4">
      {#if activeTab === "overview"}
        <RulesOverview />
      {:else if activeTab === "turns"}
        <RulesTurnStructure />
      {:else if activeTab === "cards"}
        <RulesCardTypes />
      {:else if activeTab === "keywords"}
        <RulesKeywords />
      {:else if activeTab === "combat"}
        <RulesCombat />
      {:else if activeTab === "factions"}
        <RulesFactions />
      {/if}
    </div>
  </div>
</div>
