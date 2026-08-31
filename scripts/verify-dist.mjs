#!/usr/bin/env node
/**
 * Memeriksa hasil build sebelum diterbitkan.
 *
 * Build yang "berhasil" tetap bisa menghasilkan situs rusak: manifes hilang,
 * kebijakan keamanan longgar, rujukan ke berkas yang tidak ikut terkirim, atau
 * jalur dasar yang salah sehingga seluruh aset gagal dimuat di GitHub Pages.
 * Kegagalan seperti itu baru ketahuan setelah pengguna membuka halamannya —
 * terlambat. Skrip ini memindahkan pemeriksaan itu ke dalam CI.
 *
 * .Deckyx
 */

import { readdir, readFile, stat } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("..", import.meta.url));
const DIST = join(ROOT, "dist");
const BASE = process.env.AI_ATLAS_BASE ?? "/ai-atlas/";

const problems = [];

/** Mencatat sebuah kegagalan pemeriksaan. */
function fail(message) {
  problems.push(message);
}

/** Mengumpulkan seluruh berkas di dalam sebuah direktori secara rekursif. */
async function walk(dir, prefix = "") {
  const out = [];
  let entries;
  try {
    entries = await readdir(dir, { withFileTypes: true });
  } catch {
    return out;
  }
  for (const entry of entries) {
    const rel = prefix ? `${prefix}/${entry.name}` : entry.name;
    if (entry.isDirectory()) {
      out.push(...(await walk(join(dir, entry.name), rel)));
    } else {
      out.push(rel);
    }
  }
  return out;
}

async function main() {
  try {
    await stat(DIST);
  } catch {
    console.error("dist/ belum ada. Jalankan `npm run build` lebih dulu.");
    process.exit(1);
  }

  const files = await walk(DIST);
  const has = (name) => files.includes(name);

  // 1. Berkas yang wajib ikut terkirim.
  const required = [
    "index.html",
    "404.html",
    "robots.txt",
    "sitemap.xml",
    "manifest.webmanifest",
    "sw.js",
    "icon.svg",
    "icon-maskable.svg",
  ];
  for (const name of required) {
    if (!has(name)) fail(`berkas wajib hilang: ${name}`);
  }

  // 2. Tepat satu modul WebAssembly.
  const wasm = files.filter((f) => f.endsWith(".wasm"));
  if (wasm.length !== 1) {
    fail(`diharapkan tepat satu berkas .wasm, ditemukan ${wasm.length}`);
  }

  const html = await readFile(join(DIST, "index.html"), "utf8");

  // 3. Kebijakan keamanan konten sudah disegel dan tetap ketat.
  if (!html.includes("Content-Security-Policy")) {
    fail("meta Content-Security-Policy hilang dari index.html");
  }
  if (html.includes("REPLACE_BOOT_HASH")) {
    fail("penanda pengganti kebijakan keamanan masih tertinggal");
  }
  const csp = html.match(/content="([^"]*default-src[^"]*)"/)?.[1] ?? "";
  const scriptSrc = csp
    .split(";")
    .map((d) => d.trim())
    .find((d) => d.startsWith("script-src"));
  if (!scriptSrc) {
    fail("arahan script-src hilang");
  } else {
    if (scriptSrc.split(/\s+/).includes("'unsafe-inline'")) {
      fail("script-src memuat 'unsafe-inline'");
    }
    if (scriptSrc.split(/\s+/).includes("'unsafe-eval'")) {
      fail("script-src memuat 'unsafe-eval'");
    }
    if (!scriptSrc.includes("'sha256-")) {
      fail("script-src tidak memuat sidik SHA-256 untuk skrip sebaris");
    }
    // WebAssembly memerlukan izin ini secara khusus; tanpa itu mesin gagal dimuat.
    if (!scriptSrc.includes("'wasm-unsafe-eval'")) {
      fail("script-src tidak mengizinkan 'wasm-unsafe-eval', mesin tidak akan jalan");
    }
  }
  // `frame-ancestors` tidak diperiksa: tidak berlaku lewat <meta>.
  for (const directive of ["object-src 'none'", "base-uri 'none'", "form-action 'none'"]) {
    if (!csp.includes(directive)) fail(`kebijakan keamanan kehilangan ${directive}`);
  }

  // 4. Metadata yang dipakai mesin pencari dan pemasangan aplikasi.
  for (const needle of [
    'rel="canonical"',
    'rel="manifest"',
    'property="og:title"',
    'name="description"',
    "application/ld+json",
    ".Deckyx",
  ]) {
    if (!html.includes(needle)) fail(`index.html kehilangan ${needle}`);
  }

  // 5. Seluruh aset yang dirujuk memang ikut terkirim, dan jalur dasarnya benar.
  const refs = [...html.matchAll(/(?:src|href)="([^"]+)"/g)].map((m) => m[1]);
  for (const ref of refs) {
    if (/^(https?:|data:|mailto:|#)/.test(ref)) continue;
    const path = ref.startsWith(BASE) ? ref.slice(BASE.length) : ref.replace(/^\.?\//, "");
    if (!has(path)) fail(`index.html merujuk berkas yang tidak ada: ${ref}`);
    if (ref.startsWith("/") && !ref.startsWith(BASE)) {
      fail(`rujukan mutlak di luar jalur dasar ${BASE}: ${ref}`);
    }
  }

  // 6. Manifes bisa dibaca dan menunjuk ikon yang ada.
  try {
    const manifest = JSON.parse(await readFile(join(DIST, "manifest.webmanifest"), "utf8"));
    for (const icon of manifest.icons ?? []) {
      const path = String(icon.src).replace(/^\.?\//, "");
      if (!has(path)) fail(`manifes menunjuk ikon yang tidak ada: ${icon.src}`);
    }
    if (!manifest.name || !manifest.start_url) {
      fail("manifes kehilangan name atau start_url");
    }
  } catch (error) {
    fail(`manifes tidak bisa dibaca: ${error.message}`);
  }

  // 7. Peta sumber tidak boleh dirujuk dari berkas yang terkirim.
  for (const file of files.filter((f) => f.endsWith(".js"))) {
    const source = await readFile(join(DIST, file), "utf8");
    if (source.includes("//# sourceMappingURL=")) {
      fail(`${file} merujuk peta sumber; harusnya tersembunyi`);
    }
  }

  if (problems.length > 0) {
    console.error(`Verifikasi gagal dengan ${problems.length} masalah:\n`);
    for (const p of problems) console.error(`  - ${p}`);
    process.exit(1);
  }

  console.log(`Verifikasi lolos. ${files.length} berkas siap terbit.`);
}

await main();
