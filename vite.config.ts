import { defineConfig, type Plugin } from "vite";

/**
 * Situs dilayani dari https://xyb3rpunq.github.io/ai-atlas/, jadi seluruh
 * asetnya harus dirujuk relatif terhadap subdirektori itu. `base` bisa
 * ditimpa lewat variabel lingkungan agar pratinjau lokal dan pemeriksaan CI
 * bisa memakai akar.
 */
const base = process.env.AI_ATLAS_BASE ?? "/ai-atlas/";

/**
 * Melepas meta kebijakan keamanan konten saat pengembangan.
 *
 * Berkas sumber memuat penanda `REPLACE_BOOT_HASH` yang baru diganti sidik
 * sungguhan oleh `scripts/csp.mjs` setelah build. Penanda itu bukan sidik yang
 * sah, sehingga peladen pengembangan akan memblokir skrip pemulih tema dan
 * halaman berkedip putih di tiap muat ulang. Peladen pengembangan juga
 * menyuntikkan modul HMR yang memang tidak tercakup kebijakan produksi.
 *
 * Kebijakan tetap diuji: `scripts/verify-dist.mjs` memeriksa hasil build, dan
 * pemeriksaan itulah yang menentukan apa yang sampai ke pengguna.
 */
function stripCspInDev(): Plugin {
  return {
    name: "ai-atlas:strip-csp-in-dev",
    apply: "serve",
    transformIndexHtml(html) {
      return html.replace(
        /\s*<meta\s+http-equiv="Content-Security-Policy"[\s\S]*?\/>/i,
        "",
      );
    },
  };
}

export default defineConfig({
  plugins: [stripCspInDev()],
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
