import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [svelte()],

  // Vite options tailored for Tauri development; see https://vitejs.dev/config/
  clearScreen: false,
  server: {
    // Tauri expects a fixed port; if not available, error out.
    port: 1420,
    strictPort: true,
    host: "127.0.0.1",
    hmr: process.env.TAURI_DEV_PLATFORM
      ? { protocol: "ws", host: process.env.TAURI_DEV_PLATFORM, port: 1421 }
      : undefined,
    watch: { ignored: ["**/src-tauri/**", "**/Minecraft-Console-Client-master/**"] },
  },
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS/Linux.
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari13",
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
}));
