import { defineConfig } from "vitest/config";

/**
 * Konfigurasi uji dipisah dari `vite.config.ts` karena build memakai `web/`
 * sebagai akar, sedangkan uji perlu melihat seluruh repositori.
 *
 * .Deckyx
 */
export default defineConfig({
  test: {
    root: ".",
    include: ["tests/**/*.test.ts"],
    environment: "node",
    reporters: ["default"],
    coverage: {
      provider: "v8",
      include: ["web/src/**/*.ts"],
      exclude: ["web/src/main.ts", "web/src/labs/**"],
    },
  },
});
