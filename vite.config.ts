import { defineConfig } from "vite";

/**
 * Situs dilayani dari https://xyb3rpunq.github.io/ai-atlas/, jadi seluruh
 * asetnya harus dirujuk relatif terhadap subdirektori itu. `base` bisa
 * ditimpa lewat variabel lingkungan agar pratinjau lokal dan pemeriksaan CI
 * bisa memakai akar.
 */
const base = process.env.AI_ATLAS_BASE ?? "/ai-atlas/";

export default defineConfig({
  base,
  root: "web",
  publicDir: "public",
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    target: "es2022",
    // Peta sumber tetap dibuat agar galat di produksi bisa ditelusuri, tetapi
    // tidak dirujuk dari berkas terbangun sehingga tidak menambah unduhan.
    sourcemap: "hidden",
    assetsInlineLimit: 4096,
    rollupOptions: {
      output: {
        // Nama berkas ber-hash supaya aman di-cache selamanya.
        entryFileNames: "assets/[name]-[hash].js",
        chunkFileNames: "assets/[name]-[hash].js",
        assetFileNames: "assets/[name]-[hash][extname]",
      },
    },
  },
  server: {
    port: 5173,
    strictPort: false,
  },
  // WebAssembly diambil saat berjalan lewat `fetch`, bukan di-bundel.
  assetsInclude: ["**/*.wasm"],
});
