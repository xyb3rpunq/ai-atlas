#!/usr/bin/env node
/**
 * Penjaga anggaran ukuran berkas.
 *
 * Janji "tidak lambat" hanya bermakna kalau ada yang menegakkannya. Skrip ini
 * mengukur ukuran terkompresi tiap aset hasil build dan keluar dengan kode
 * galat bila ada yang melewati batas, sehingga CI menolak perubahan yang
 * diam-diam menggemukkan situs.
 *
 * .Deckyx
 */

import { readdir, readFile, stat } from "node:fs/promises";
import { gzipSync } from "node:zlib";
import { join, extname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("..", import.meta.url));
const DIST = join(ROOT, "dist");

/** Batas ukuran terkompresi (gzip) per jenis aset, dalam kilobita. */
const BUDGET_KB = {
  ".wasm": 400,
  ".js": 60,
  ".css": 20,
  ".html": 12,
};

/** Batas total seluruh aset yang diunduh pada muat pertama. */
const TOTAL_BUDGET_KB = 460;

/** Mengumpulkan seluruh berkas di dalam sebuah direktori secara rekursif. */
async function walk(dir) {
  const out = [];
  let entries;
  try {
    entries = await readdir(dir, { withFileTypes: true });
  } catch {
    return out;
  }
  for (const entry of entries) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      out.push(...(await walk(full)));
    } else {
      out.push(full);
    }
  }
  return out;
}

/** Ukuran gzip sebuah berkas, dalam bita. */
async function gzipSize(path) {
  const buf = await readFile(path);
  return gzipSync(buf, { level: 9 }).byteLength;
}

const kb = (bytes) => (bytes / 1024).toFixed(1);

async function main() {
  try {
    await stat(DIST);
  } catch {
    console.error("dist/ belum ada. Jalankan `npm run build` lebih dulu.");
    process.exit(1);
  }

  const files = await walk(DIST);
  // Peta sumber tidak pernah diunduh peramban, jadi tidak dihitung.
  const shipped = files.filter((f) => !f.endsWith(".map"));

  const rows = [];
  let total = 0;
  let failures = 0;

  for (const file of shipped.sort()) {
    const ext = extname(file);
    const size = await gzipSize(file);
    total += size;
    const limit = BUDGET_KB[ext];
    const name = file.slice(DIST.length + 1).replaceAll("\\", "/");
    const over = limit !== undefined && size / 1024 > limit;
    if (over) failures++;
    rows.push({
      name,
      gzip: `${kb(size)} KB`,
      budget: limit === undefined ? "—" : `${limit} KB`,
      status: limit === undefined ? "—" : over ? "LEWAT" : "ok",
    });
  }

  console.table(rows);

  const totalOver = total / 1024 > TOTAL_BUDGET_KB;
  console.log(
    `\nTotal terkirim: ${kb(total)} KB gzip  (anggaran ${TOTAL_BUDGET_KB} KB) — ${
      totalOver ? "LEWAT" : "ok"
    }`,
  );

  if (failures > 0 || totalOver) {
    console.error(
      `\nAnggaran terlampaui pada ${failures} berkas${totalOver ? " dan pada total" : ""}.`,
    );
    process.exit(1);
  }
  console.log("Seluruh anggaran terpenuhi.");
}

await main();
