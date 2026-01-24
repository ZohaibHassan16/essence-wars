// Svelte actions for GSAP animations
import { gsap } from "gsap";
import type { Action } from "svelte/action";

interface AnimatableParams {
  onDamage?: (amount: number) => void;
  onHeal?: (amount: number) => void;
  onDeath?: () => void;
  onSpawn?: () => void;
  onAttack?: (targetEl: HTMLElement) => void;
}

/**
 * Svelte action that makes an element animatable
 * Returns methods to trigger various animations
 */
export const animatable: Action<HTMLElement, AnimatableParams | undefined> = (node, params) => {
  // Store the element reference in a global registry for cross-component animations
  const id = node.dataset.animateId;
  if (id) {
    animationRegistry.set(id, new WeakRef(node));
  }

  return {
    destroy() {
      if (id) {
        animationRegistry.delete(id);
      }
    },
  };
};

// Global registry for animated elements using WeakRef
const animationRegistry = new Map<string, WeakRef<HTMLElement>>();

// Track active floating elements for cleanup
const activeFloatingElements = new Set<HTMLElement>();

/**
 * Get an animated element by its ID (handles WeakRef deref and cleanup)
 */
export function getAnimatedElement(id: string): HTMLElement | undefined {
  const ref = animationRegistry.get(id);
  if (!ref) return undefined;

  const element = ref.deref();
  if (!element) {
    // Element was garbage collected, clean up the registry
    animationRegistry.delete(id);
    return undefined;
  }
  return element;
}

/**
 * Clean up stale animation refs (elements that were garbage collected)
 */
export function cleanupStaleAnimationRefs(): void {
  for (const [id, ref] of animationRegistry) {
    if (!ref.deref()) {
      animationRegistry.delete(id);
    }
  }
}

/**
 * Clean up all floating elements (call on scene transitions)
 */
export function cleanupAllFloatingElements(): void {
  for (const element of activeFloatingElements) {
    try {
      element.remove();
    } catch {
      // Element may already be removed
    }
  }
  activeFloatingElements.clear();
}

/**
 * Create a colored overlay for visual effects (more performant than filters)
 */
function createOverlay(element: HTMLElement, color: string, opacity: number): HTMLElement {
  const overlay = document.createElement("div");
  overlay.style.cssText = `
    position: absolute;
    inset: 0;
    background-color: ${color};
    opacity: ${opacity};
    pointer-events: none;
    border-radius: inherit;
    z-index: 10;
  `;
  element.style.position = "relative";
  element.appendChild(overlay);
  return overlay;
}

/**
 * Trigger a damage animation on an element
 */
export function triggerDamage(elementOrId: HTMLElement | string, amount: number): void {
  const element = typeof elementOrId === "string"
    ? getAnimatedElement(elementOrId)
    : elementOrId;

  if (!element) return;

  const intensity = Math.min(amount * 3, 20);

  // Create red overlay instead of filter (more performant)
  const overlay = createOverlay(element, "#ef4444", 0);

  const tl = gsap.timeline();

  // Flash overlay
  tl.to(overlay, {
    opacity: 0.4,
    duration: 0.08,
  });

  // Shake with transform (GPU accelerated)
  tl.to(element, {
    x: intensity,
    duration: 0.04,
    yoyo: true,
    repeat: 5,
    ease: "none",
  }, "<");

  // Fade out overlay and reset position
  tl.to(overlay, {
    opacity: 0,
    duration: 0.15,
    onComplete: () => overlay.remove(),
  });

  tl.to(element, {
    x: 0,
    duration: 0.15,
  }, "<");

  // Show damage number
  showFloatingNumber(element, amount, false);
}

/**
 * Trigger a heal animation on an element
 */
export function triggerHeal(elementOrId: HTMLElement | string, amount: number): void {
  const element = typeof elementOrId === "string"
    ? getAnimatedElement(elementOrId)
    : elementOrId;

  if (!element) return;

  // Create green overlay instead of filter (more performant)
  const overlay = createOverlay(element, "#22c55e", 0);

  const tl = gsap.timeline();

  // Flash green and scale up
  tl.to(overlay, {
    opacity: 0.3,
    duration: 0.2,
    ease: "power2.out",
  });

  tl.to(element, {
    scale: 1.05,
    duration: 0.2,
    ease: "power2.out",
  }, "<");

  // Fade overlay and scale back
  tl.to(overlay, {
    opacity: 0,
    duration: 0.3,
    ease: "power2.in",
    onComplete: () => overlay.remove(),
  });

  tl.to(element, {
    scale: 1,
    duration: 0.3,
    ease: "power2.in",
  }, "<");

  // Show heal number
  showFloatingNumber(element, amount, true);
}

/**
 * Trigger a death animation on an element
 */
export function triggerDeath(elementOrId: HTMLElement | string): Promise<void> {
  return new Promise((resolve) => {
    const element = typeof elementOrId === "string"
      ? getAnimatedElement(elementOrId)
      : elementOrId;

    if (!element) {
      resolve();
      return;
    }

    // Create white overlay for death flash
    const overlay = createOverlay(element, "#ffffff", 0);

    const tl = gsap.timeline({
      onComplete: () => {
        overlay.remove();
        // CRITICAL: Reset only the GSAP-applied properties so the slot is visible
        // when Svelte re-renders it as empty. We must NOT use clearProps: "all"
        // because that would also clear the CSS variable dimensions (width/height).
        gsap.set(element, { clearProps: "opacity,scale,x,y,transform" });
        resolve();
      },
    });

    // Flash white overlay
    tl.to(overlay, {
      opacity: 0.8,
      duration: 0.1,
    });

    // Shake violently
    tl.to(element, {
      x: 8,
      duration: 0.03,
      yoyo: true,
      repeat: 6,
    });

    // Fade overlay to dark and element out
    tl.to(overlay, {
      backgroundColor: "#000000",
      opacity: 0.6,
      duration: 0.2,
    });

    tl.to(element, {
      opacity: 0,
      scale: 0.5,
      y: 15,
      duration: 0.4,
      ease: "power2.in",
    }, "<");
  });
}

/**
 * Trigger a spawn animation on an element
 */
export function triggerSpawn(elementOrId: HTMLElement | string): Promise<void> {
  return new Promise((resolve) => {
    const element = typeof elementOrId === "string"
      ? getAnimatedElement(elementOrId)
      : elementOrId;

    if (!element) {
      resolve();
      return;
    }

    // Create bright overlay for spawn glow
    const overlay = createOverlay(element, "#ffffff", 0);

    // Start state
    gsap.set(element, {
      opacity: 0,
      scale: 0.3,
      y: 20,
    });

    const tl = gsap.timeline({
      onComplete: () => {
        overlay.remove();
        resolve();
      },
    });

    // Pop in with glow overlay
    tl.to(element, {
      opacity: 1,
      scale: 1.1,
      y: 0,
      duration: 0.25,
      ease: "back.out(1.7)",
    });

    tl.to(overlay, {
      opacity: 0.3,
      duration: 0.25,
      ease: "back.out(1.7)",
    }, "<");

    // Settle and fade overlay
    tl.to(element, {
      scale: 1,
      duration: 0.15,
      ease: "power2.out",
    });

    tl.to(overlay, {
      opacity: 0,
      duration: 0.15,
    }, "<");
  });
}

/**
 * Trigger an attack animation
 */
export function triggerAttack(
  attackerId: string,
  defenderId: string,
  onImpact?: () => void
): Promise<void> {
  return new Promise((resolve) => {
    const attacker = getAnimatedElement(attackerId);
    const defender = getAnimatedElement(defenderId);

    if (!attacker || !defender) {
      onImpact?.();
      resolve();
      return;
    }

    const attackerRect = attacker.getBoundingClientRect();
    const defenderRect = defender.getBoundingClientRect();

    const deltaX = defenderRect.left - attackerRect.left;
    const deltaY = defenderRect.top - attackerRect.top;
    const lungeDistance = 0.35;

    // Create overlay for attack glow
    const overlay = createOverlay(attacker, "#ffffff", 0);

    const tl = gsap.timeline({
      onComplete: () => {
        overlay.remove();
        resolve();
      },
    });

    // Wind up with overlay glow
    tl.to(attacker, {
      scale: 1.15,
      duration: 0.15,
      ease: "power2.out",
    });

    tl.to(overlay, {
      opacity: 0.2,
      duration: 0.15,
      ease: "power2.out",
    }, "<");

    // Lunge
    tl.to(attacker, {
      x: deltaX * lungeDistance,
      y: deltaY * lungeDistance,
      duration: 0.2,
      ease: "power3.in",
      onComplete: onImpact,
    });

    // Return
    tl.to(attacker, {
      x: 0,
      y: 0,
      scale: 1,
      duration: 0.3,
      ease: "power2.out",
    });

    tl.to(overlay, {
      opacity: 0,
      duration: 0.3,
    }, "<");
  });
}

/**
 * Show a floating damage/heal number
 */
function showFloatingNumber(element: HTMLElement, amount: number, isHeal: boolean): void {
  const rect = element.getBoundingClientRect();

  const numberEl = document.createElement("div");
  numberEl.style.cssText = `
    position: fixed;
    left: ${rect.left + rect.width / 2}px;
    top: ${rect.top + rect.height / 3}px;
    transform: translate(-50%, -50%);
    font-size: 2rem;
    font-weight: 900;
    color: ${isHeal ? "#4ade80" : "#f87171"};
    text-shadow: 0 2px 8px rgba(0,0,0,0.8), 0 0 20px ${isHeal ? "rgba(74,222,128,0.5)" : "rgba(248,113,113,0.5)"};
    pointer-events: none;
    z-index: 9999;
    font-family: system-ui, sans-serif;
  `;
  numberEl.textContent = isHeal ? `+${amount}` : `-${amount}`;
  document.body.appendChild(numberEl);

  // Track floating element for cleanup
  activeFloatingElements.add(numberEl);

  // Cleanup function
  const cleanup = () => {
    activeFloatingElements.delete(numberEl);
    try {
      numberEl.remove();
    } catch {
      // Already removed
    }
  };

  // Fallback cleanup in case animation doesn't complete
  const fallbackTimeout = setTimeout(cleanup, 1500);

  gsap.to(numberEl, {
    y: -50,
    opacity: 0,
    scale: 1.3,
    duration: 0.8,
    ease: "power2.out",
    onComplete: () => {
      clearTimeout(fallbackTimeout);
      cleanup();
    },
  });
}

/**
 * Play card animation from hand to board
 */
export function triggerPlayCard(
  cardElement: HTMLElement,
  targetSlot: HTMLElement
): Promise<void> {
  return new Promise((resolve) => {
    const cardRect = cardElement.getBoundingClientRect();
    const slotRect = targetSlot.getBoundingClientRect();

    // Create a flying clone
    const clone = cardElement.cloneNode(true) as HTMLElement;
    clone.style.cssText = `
      position: fixed;
      left: ${cardRect.left}px;
      top: ${cardRect.top}px;
      width: ${cardRect.width}px;
      height: ${cardRect.height}px;
      z-index: 9999;
      pointer-events: none;
    `;
    document.body.appendChild(clone);

    // Hide original briefly
    cardElement.style.opacity = "0";

    const deltaX = slotRect.left + slotRect.width / 2 - (cardRect.left + cardRect.width / 2);
    const deltaY = slotRect.top + slotRect.height / 2 - (cardRect.top + cardRect.height / 2);

    gsap.to(clone, {
      x: deltaX,
      y: deltaY,
      scale: 1.2,
      rotation: 5,
      duration: 0.3,
      ease: "power2.out",
      onComplete: () => {
        gsap.to(clone, {
          scale: 0.8,
          opacity: 0,
          duration: 0.15,
          ease: "power2.in",
          onComplete: () => {
            clone.remove();
            cardElement.style.opacity = "";
            resolve();
          },
        });
      },
    });
  });
}
