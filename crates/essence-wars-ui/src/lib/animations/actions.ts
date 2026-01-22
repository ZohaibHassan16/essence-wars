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
    animationRegistry.set(id, node);
  }

  return {
    destroy() {
      if (id) {
        animationRegistry.delete(id);
      }
    },
  };
};

// Global registry for animated elements
export const animationRegistry = new Map<string, HTMLElement>();

/**
 * Get an animated element by its ID
 */
export function getAnimatedElement(id: string): HTMLElement | undefined {
  return animationRegistry.get(id);
}

/**
 * Trigger a damage animation on an element
 */
export function triggerDamage(elementOrId: HTMLElement | string, amount: number): void {
  const element = typeof elementOrId === "string"
    ? animationRegistry.get(elementOrId)
    : elementOrId;

  if (!element) return;

  const intensity = Math.min(amount * 3, 20);

  // Flash red and shake
  const tl = gsap.timeline();

  tl.to(element, {
    filter: "brightness(1.5) sepia(1) hue-rotate(-50deg) saturate(2)",
    duration: 0.08,
  });

  tl.to(element, {
    x: intensity,
    duration: 0.04,
    yoyo: true,
    repeat: 5,
    ease: "none",
  }, "<");

  tl.to(element, {
    filter: "",
    x: 0,
    duration: 0.15,
  });

  // Show damage number
  showFloatingNumber(element, amount, false);
}

/**
 * Trigger a heal animation on an element
 */
export function triggerHeal(elementOrId: HTMLElement | string, amount: number): void {
  const element = typeof elementOrId === "string"
    ? animationRegistry.get(elementOrId)
    : elementOrId;

  if (!element) return;

  const tl = gsap.timeline();

  tl.to(element, {
    filter: "brightness(1.3) sepia(0.5) hue-rotate(80deg) saturate(1.5)",
    scale: 1.05,
    duration: 0.2,
    ease: "power2.out",
  });

  tl.to(element, {
    filter: "",
    scale: 1,
    duration: 0.3,
    ease: "power2.in",
  });

  // Show heal number
  showFloatingNumber(element, amount, true);
}

/**
 * Trigger a death animation on an element
 */
export function triggerDeath(elementOrId: HTMLElement | string): Promise<void> {
  return new Promise((resolve) => {
    const element = typeof elementOrId === "string"
      ? animationRegistry.get(elementOrId)
      : elementOrId;

    if (!element) {
      resolve();
      return;
    }

    const tl = gsap.timeline({
      onComplete: resolve,
    });

    // Flash white
    tl.to(element, {
      filter: "brightness(3) grayscale(1)",
      duration: 0.1,
    });

    // Shake violently
    tl.to(element, {
      x: 8,
      duration: 0.03,
      yoyo: true,
      repeat: 6,
    });

    // Fade out and shrink
    tl.to(element, {
      opacity: 0,
      scale: 0.5,
      y: 15,
      filter: "brightness(0.3) grayscale(1)",
      duration: 0.4,
      ease: "power2.in",
    });
  });
}

/**
 * Trigger a spawn animation on an element
 */
export function triggerSpawn(elementOrId: HTMLElement | string): Promise<void> {
  return new Promise((resolve) => {
    const element = typeof elementOrId === "string"
      ? animationRegistry.get(elementOrId)
      : elementOrId;

    if (!element) {
      resolve();
      return;
    }

    // Start state
    gsap.set(element, {
      opacity: 0,
      scale: 0.3,
      y: 20,
    });

    const tl = gsap.timeline({
      onComplete: resolve,
    });

    // Pop in with glow
    tl.to(element, {
      opacity: 1,
      scale: 1.1,
      y: 0,
      filter: "brightness(1.5)",
      duration: 0.25,
      ease: "back.out(1.7)",
    });

    // Settle
    tl.to(element, {
      scale: 1,
      filter: "",
      duration: 0.15,
      ease: "power2.out",
    });
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
    const attacker = animationRegistry.get(attackerId);
    const defender = animationRegistry.get(defenderId);

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

    const tl = gsap.timeline({
      onComplete: resolve,
    });

    // Wind up
    tl.to(attacker, {
      scale: 1.15,
      filter: "brightness(1.2)",
      duration: 0.15,
      ease: "power2.out",
    });

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
      filter: "",
      duration: 0.3,
      ease: "power2.out",
    });
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

  gsap.to(numberEl, {
    y: -50,
    opacity: 0,
    scale: 1.3,
    duration: 0.8,
    ease: "power2.out",
    onComplete: () => numberEl.remove(),
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
