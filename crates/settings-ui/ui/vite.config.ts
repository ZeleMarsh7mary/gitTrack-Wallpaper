import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// The Rust crate embeds ../ui/dist, so the output directory must stay "dist".
export default defineConfig({
  // Relative asset URLs keep the bundle working from the Tauri asset protocol
  // and from a plain file:// preview.
  base: "./",
  clearScreen: false,
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: "chrome110",
  },
  server: {
    port: 5173,
    strictPort: true,
  },
});
