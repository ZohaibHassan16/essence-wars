// Tauri doesn't have a Node.js server to do proper SSR
// so we use adapter-static with a fallback to index.html to put the site in SPA mode
// See: https://svelte.dev/docs/kit/single-page-apps
// See: https://v2.tauri.app/start/frontend/sveltekit/ for more info
import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

// For GitHub Pages deployment, set BASE_PATH=/essence-wars
// @ts-expect-error process is a nodejs global
const basePath = process.env.BASE_PATH || "";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      fallback: "index.html",
    }),
    alias: {
      // WASM package for web builds
      $wasm: "./src-wasm/pkg",
    },
    paths: {
      // Set base path for GitHub Pages (empty for Tauri/local dev)
      base: basePath,
    },
  },
};

export default config;
