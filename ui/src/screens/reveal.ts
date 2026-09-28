import { useEffect, useRef } from "react";

/**
 * Reveal-on-scroll, as on pieceofmind.cloud: every `.pb-reveal` inside the
 * returned ref gets `.pb-in` the first time it enters the viewport. The CSS
 * owns the motion (and turns it off for reduced motion); without
 * IntersectionObserver everything is shown at once.
 */
export function useReveal<T extends HTMLElement>() {
  const ref = useRef<T>(null);
  useEffect(() => {
    const container = ref.current;
    if (!container) return;
    const items = Array.from(container.querySelectorAll<HTMLElement>(".pb-reveal"));
    if (typeof IntersectionObserver === "undefined") {
      items.forEach((item) => item.classList.add("pb-in"));
      return;
    }
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (!entry.isIntersecting) continue;
          entry.target.classList.add("pb-in");
          observer.unobserve(entry.target);
        }
      },
      { threshold: 0.12, rootMargin: "0px 0px -6% 0px" },
    );
    items.forEach((item) => observer.observe(item));
    return () => observer.disconnect();
  }, []);
  return ref;
}
