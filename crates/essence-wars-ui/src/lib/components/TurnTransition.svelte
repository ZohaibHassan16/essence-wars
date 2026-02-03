<script lang="ts">
  import { gsap } from "gsap";

  let {
    isYourTurn,
    turnNumber,
    visible = false,
  }: {
    isYourTurn: boolean;
    turnNumber: number;
    visible: boolean;
  } = $props();

  let containerEl: HTMLDivElement | undefined = $state();
  let textEl: HTMLDivElement | undefined = $state();

  $effect(() => {
    if (visible && containerEl && textEl) {
      // Animate in
      const tl = gsap.timeline();

      gsap.set(containerEl, { opacity: 0 });
      gsap.set(textEl, { scale: 0.5, opacity: 0 });

      tl.to(containerEl, {
        opacity: 1,
        duration: 0.2,
      });

      tl.to(textEl, {
        scale: 1,
        opacity: 1,
        duration: 0.3,
        ease: "back.out(1.5)",
      }, "-=0.1");

      // Hold
      tl.to({}, { duration: 0.8 });

      // Animate out
      tl.to(textEl, {
        scale: 1.2,
        opacity: 0,
        duration: 0.2,
        ease: "power2.in",
      });

      tl.to(containerEl, {
        opacity: 0,
        duration: 0.2,
      }, "-=0.1");
    }
  });
</script>

{#if visible}
  <div
    bind:this={containerEl}
    class="fixed inset-0 z-50 flex items-center justify-center pointer-events-none"
    style="background: radial-gradient(ellipse at center, rgba(0,0,0,0.7) 0%, transparent 70%)"
  >
    <div
      bind:this={textEl}
      class="text-center"
    >
      <div class="text-6xl font-black tracking-wider mb-2
                  {isYourTurn ? 'text-health' : 'text-damage'}
                  drop-shadow-[0_0_30px_rgba(255,255,255,0.3)]"
      >
        {isYourTurn ? "YOUR TURN" : "ENEMY TURN"}
      </div>
      <div class="text-2xl text-ui-text-dim font-semibold">
        Turn {turnNumber}
      </div>
    </div>
  </div>
{/if}
