// Animation system using GSAP
import { gsap } from "gsap";

// Animation durations (in seconds)
export const DURATIONS = {
  cardHover: 0.15,
  cardSelect: 0.2,
  cardPlay: 0.5,
  cardReturn: 0.3,
  attackDeclare: 0.2,
  attackExecute: 0.4,
  damage: 0.3,
  death: 0.5,
  heal: 0.4,
  turnTransition: 0.3,
};

// Easing functions
export const EASINGS = {
  cardPlay: "power2.out",
  attack: "power3.inOut",
  damage: "elastic.out(1, 0.5)",
  death: "power2.in",
  bounce: "bounce.out",
};

/**
 * Animate a card being played from hand to a board slot
 */
export function animateCardPlay(
  cardElement: HTMLElement,
  targetSlot: HTMLElement,
  onComplete?: () => void
): gsap.core.Tween {
  const cardRect = cardElement.getBoundingClientRect();
  const slotRect = targetSlot.getBoundingClientRect();

  // Calculate the arc path
  const deltaX = slotRect.left + slotRect.width / 2 - (cardRect.left + cardRect.width / 2);
  const deltaY = slotRect.top + slotRect.height / 2 - (cardRect.top + cardRect.height / 2);

  // Create a clone for the animation
  const clone = cardElement.cloneNode(true) as HTMLElement;
  clone.style.position = "fixed";
  clone.style.left = `${cardRect.left}px`;
  clone.style.top = `${cardRect.top}px`;
  clone.style.width = `${cardRect.width}px`;
  clone.style.height = `${cardRect.height}px`;
  clone.style.zIndex = "1000";
  clone.style.pointerEvents = "none";
  document.body.appendChild(clone);

  // Hide original
  cardElement.style.opacity = "0";

  return gsap.to(clone, {
    x: deltaX,
    y: deltaY,
    scale: 0.8,
    duration: DURATIONS.cardPlay,
    ease: EASINGS.cardPlay,
    onComplete: () => {
      clone.remove();
      cardElement.style.opacity = "";
      onComplete?.();
    },
  });
}

/**
 * Animate an attack from one creature to another
 */
export function animateAttack(
  attackerElement: HTMLElement,
  defenderElement: HTMLElement,
  onImpact?: () => void,
  onComplete?: () => void
): gsap.core.Timeline {
  const attackerRect = attackerElement.getBoundingClientRect();
  const defenderRect = defenderElement.getBoundingClientRect();

  const deltaX = defenderRect.left - attackerRect.left;
  const deltaY = defenderRect.top - attackerRect.top;

  // Normalize to create a "lunge" effect (move partway toward target)
  const lungeDistance = 0.4;

  const tl = gsap.timeline();

  // Wind up
  tl.to(attackerElement, {
    scale: 1.1,
    duration: DURATIONS.attackDeclare,
    ease: "power2.out",
  });

  // Lunge toward target
  tl.to(attackerElement, {
    x: deltaX * lungeDistance,
    y: deltaY * lungeDistance,
    duration: DURATIONS.attackExecute * 0.4,
    ease: "power3.in",
    onComplete: onImpact,
  });

  // Return to position
  tl.to(attackerElement, {
    x: 0,
    y: 0,
    scale: 1,
    duration: DURATIONS.attackExecute * 0.6,
    ease: "power2.out",
    onComplete,
  });

  return tl;
}

/**
 * Shake animation for taking damage
 */
export function animateDamage(
  element: HTMLElement,
  damageAmount: number,
  onComplete?: () => void
): gsap.core.Timeline {
  const intensity = Math.min(damageAmount * 2, 15);
  const tl = gsap.timeline();

  // Flash red
  tl.to(element, {
    filter: "brightness(1.5) sepia(1) hue-rotate(-50deg) saturate(2)",
    duration: 0.1,
  });

  // Shake
  tl.to(element, {
    x: intensity,
    duration: 0.05,
    yoyo: true,
    repeat: 5,
    ease: "none",
  });

  // Reset
  tl.to(element, {
    filter: "",
    x: 0,
    duration: 0.2,
    onComplete,
  });

  return tl;
}

/**
 * Floating damage number
 */
export function showDamageNumber(
  targetElement: HTMLElement,
  amount: number,
  isHeal: boolean = false
): void {
  const rect = targetElement.getBoundingClientRect();

  const numberEl = document.createElement("div");
  numberEl.className = `fixed font-black text-4xl pointer-events-none z-[1001] ${isHeal ? "text-health" : "text-damage"}`;
  numberEl.textContent = isHeal ? `+${amount}` : `-${amount}`;
  numberEl.style.left = `${rect.left + rect.width / 2}px`;
  numberEl.style.top = `${rect.top + rect.height / 2}px`;
  numberEl.style.transform = "translate(-50%, -50%)";
  numberEl.style.textShadow = "0 2px 4px rgba(0,0,0,0.8)";
  document.body.appendChild(numberEl);

  gsap.to(numberEl, {
    y: -60,
    opacity: 0,
    scale: 1.5,
    duration: 1,
    ease: "power2.out",
    onComplete: () => numberEl.remove(),
  });
}

/**
 * Creature death animation
 */
export function animateDeath(
  element: HTMLElement,
  onComplete?: () => void
): gsap.core.Timeline {
  const tl = gsap.timeline();

  // Shake and flash
  tl.to(element, {
    filter: "brightness(2) grayscale(1)",
    duration: 0.1,
  });

  tl.to(element, {
    x: 5,
    duration: 0.05,
    yoyo: true,
    repeat: 4,
  });

  // Fade out and scale down
  tl.to(element, {
    opacity: 0,
    scale: 0.5,
    y: 20,
    filter: "brightness(0.5) grayscale(1)",
    duration: DURATIONS.death,
    ease: EASINGS.death,
    onComplete,
  });

  return tl;
}

/**
 * Heal animation
 */
export function animateHeal(
  element: HTMLElement,
  amount: number,
  onComplete?: () => void
): gsap.core.Timeline {
  const tl = gsap.timeline();

  // Green glow pulse
  tl.to(element, {
    filter: "brightness(1.3) sepia(1) hue-rotate(60deg) saturate(1.5)",
    scale: 1.05,
    duration: DURATIONS.heal * 0.5,
    ease: "power2.out",
  });

  tl.to(element, {
    filter: "",
    scale: 1,
    duration: DURATIONS.heal * 0.5,
    ease: "power2.in",
    onComplete,
  });

  return tl;
}

/**
 * Card hover animation
 */
export function animateCardHover(element: HTMLElement, isHovering: boolean): gsap.core.Tween {
  return gsap.to(element, {
    scale: isHovering ? 1.08 : 1,
    y: isHovering ? -8 : 0,
    boxShadow: isHovering
      ? "0 20px 40px rgba(0,0,0,0.5)"
      : "0 4px 8px rgba(0,0,0,0.3)",
    duration: DURATIONS.cardHover,
    ease: "power2.out",
  });
}

/**
 * Card selection animation
 */
export function animateCardSelect(element: HTMLElement, isSelected: boolean): gsap.core.Tween {
  return gsap.to(element, {
    scale: isSelected ? 1.12 : 1,
    y: isSelected ? -20 : 0,
    duration: DURATIONS.cardSelect,
    ease: "power2.out",
  });
}

/**
 * Spawn animation for new creatures
 */
export function animateSpawn(element: HTMLElement, onComplete?: () => void): gsap.core.Timeline {
  const tl = gsap.timeline();

  // Start invisible and scaled down
  gsap.set(element, { opacity: 0, scale: 0.3 });

  // Pop in with glow
  tl.to(element, {
    opacity: 1,
    scale: 1.1,
    filter: "brightness(1.5)",
    duration: 0.3,
    ease: "back.out(1.7)",
  });

  // Settle
  tl.to(element, {
    scale: 1,
    filter: "",
    duration: 0.2,
    ease: "power2.out",
    onComplete,
  });

  return tl;
}

/**
 * Turn transition animation
 */
export function animateTurnTransition(element: HTMLElement, isYourTurn: boolean): gsap.core.Timeline {
  const tl = gsap.timeline();

  tl.fromTo(
    element,
    { opacity: 0, scale: 0.8 },
    {
      opacity: 1,
      scale: 1,
      duration: DURATIONS.turnTransition,
      ease: "back.out(1.5)",
    }
  );

  if (isYourTurn) {
    tl.to(element, {
      scale: 1.05,
      duration: 0.2,
      yoyo: true,
      repeat: 1,
    });
  }

  return tl;
}
