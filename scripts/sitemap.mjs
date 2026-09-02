/**
 * Menyusun peta situs dari katalog laboratorium.
 *
 * # Kenapa dihasilkan, bukan ditulis tangan
 *
 * Karena yang ditulis tangan tidak pernah ikut bertambah. Peta situs ini
 * menyebut tiga laboratorium selama sembilan laboratorium berikutnya
 * ditambahkan satu per satu — dan tidak ada satu pun yang gagal karenanya:
 * berkasnya tetap sah, mesin pencari tetap menerimanya, dan sembilan halaman
 * sekadar tidak pernah disebutkan. Kegagalan yang tidak menggagalkan apa pun
 * adalah kegagalan yang bertahan paling lama.
 *
 * Sumbernya `web/src/labs/registry.ts`, dibaca sebagai teks. Node tidak bisa
 * mengimpor TypeScript, dan menambahkan langkah kompilasi hanya untuk membaca
 * daftar slug akan lebih rapuh daripada membacanya. Yang menjaga pembacaan ini
 * tetap benar adalah uji: `tests/sitemap.test.ts` mengimpor katalognya
 * sungguhan lalu menuntut tiap slug ada di berkas keluarannya.
 *
 * .Deckyx
 */

import { readFileSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const AKAR = join(dirname(fileURLToPath(import.meta.url)), "..");
const KATALOG = join(AKAR, "web", "src", "labs", "registry.ts");
const TUJUAN = join(AKAR, "web", "public", "sitemap.xml");
const ASAL = "https://xyb3rpunq.github.io/ai-atlas/";

/** Halaman yang bukan laboratorium. */
const LAIN = [{ slug: "enam-bahasa", prioritas: "0.9" }];

const sumber = readFileSync(KATALOG, "utf8");

// Hanya slug di dalam larik LABS yang diambil. `SYLLABUS` di bawahnya menunjuk
// slug yang sama dan akan menghasilkan rangkap kalau ikut terbaca.
const awal = sumber.indexOf("export const LABS");
if (awal < 0) throw new Error("registry.ts: LABS tidak ditemukan");
const akhir = sumber.indexOf("\n];", awal);
if (akhir < 0) throw new Error("registry.ts: akhir LABS tidak ditemukan");

const slug = [...sumber.slice(awal, akhir).matchAll(/slug:\s*"([a-z0-9-]+)"/g)].map((m) => m[1]);
if (slug.length === 0) throw new Error("registry.ts: tidak ada satu pun slug terbaca");

const baris = [
  '<?xml version="1.0" encoding="UTF-8"?>',
  "<!-- Dihasilkan `npm run sitemap` dari web/src/labs/registry.ts. -->",
  "<!-- Jangan disunting tangan: yang disunting tangan tidak ikut bertambah. -->",
  '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"',
  '        xmlns:xhtml="http://www.w3.org/1999/xhtml">',
  "  <url>",
  `    <loc>${ASAL}</loc>`,
  "    <changefreq>weekly</changefreq>",
  "    <priority>1.0</priority>",
  // Kedua bahasa menunjuk alamat yang sama: pilihannya disimpan di peramban,
  // bukan di alamat. Menyebutkannya tetap memberi tahu mesin pencari bahwa
  // halaman ini melayani keduanya.
  `    <xhtml:link rel="alternate" hreflang="id" href="${ASAL}"/>`,
  `    <xhtml:link rel="alternate" hreflang="en" href="${ASAL}"/>`,
  "  </url>",
];

for (const s of LAIN) {
  baris.push(
    "  <url>",
    `    <loc>${ASAL}#/${s.slug}</loc>`,
    "    <changefreq>monthly</changefreq>",
    `    <priority>${s.prioritas}</priority>`,
    "  </url>",
  );
}

for (const s of slug) {
  baris.push(
    "  <url>",
    `    <loc>${ASAL}#/${s}</loc>`,
    "    <changefreq>monthly</changefreq>",
    "    <priority>0.8</priority>",
    "  </url>",
  );
}

baris.push("</urlset>");

writeFileSync(TUJUAN, baris.join("\n") + "\n", "utf8");
console.log(`Peta situs: ${slug.length + LAIN.length + 1} alamat → ${TUJUAN}`);
