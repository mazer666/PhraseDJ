import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri's dev server host (the Tauri backend connects here).
const TAURI_DEV_HOST = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],

  // Vite options tailored for Tauri development and production.
  clearScreen: false,

  server: {
    // Tauri expects a fixed port; fail if it is already in use.
    port: 1420,
    strictPort: true,
    host: TAURI_DEV_HOST || false,
    hmr: TAURI_DEV_HOST
      ? {
          protocol: "ws",
          host: TAURI_DEV_HOST,
          port: 1421,
        }
      : undefined,
    watch: {
      // Tell Vite to ignore changes in the Rust src-tauri directory.
      ignored: ["**/src-tauri/**"],
    },
  },

  // Build settings for Tauri production builds.
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: {
    // Tauri supports es2021.
    target:
      process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari13",
    // Don't minify for debug builds.
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    // Produce source maps for debug builds.
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },

  // Test configuration (vitest).
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test-setup.ts"],
  },
});
