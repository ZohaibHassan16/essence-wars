// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info

// Disable SSR - this is a client-only SPA
export const ssr = false;

// Enable prerendering for static site generation
export const prerender = true;
