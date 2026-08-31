#!/usr/bin/env node
/**
 * Menyegel kebijakan keamanan konten setelah build.
 *
 * Halaman ini memuat skrip sebaris yang harus berjalan sebelum gaya dilukis:
 * pemulih tema, yang mencegah kedipan warna. Membolehkannya dengan
 * `'unsafe-inline'` akan membuka pintu bagi skrip sisipan mana pun, jadi yang
 * dilakukan di sini sebaliknya: tiap skrip sebaris dihitung sidik SHA-256-nya,
 * lalu hanya sidik itu yang diizinkan.
 *
 * Skrip ini juga memverifikasi hasilnya. Bila masih ada penanda pengganti yang
 * tertinggal, atau arahannya melonggar, prosesnya keluar dengan galat sehingga
 * CI menolak build.
 *
 * .Deckyx
 */

import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("..", import.meta.url));
const INDEX = join(ROOT, "dist", "index.html");

/** Menemukan seluruh skrip sebaris yang benar-benar dieksekusi. */
function inlineScripts(html) {
  const out = [];
  // Skrip dengan atribut `src` diambil dari berkas, jadi tidak perlu sidik.
  const re = /<script(?![^>]*\bsrc=)([^>]*)>([\s\S]*?)<\/script>/gi;
  let m;
  while ((m = re.exec(html)) !== null) {
    const attrs = m[1] ?? "";
    const body = m[2] ?? "";
    // Blok `application/ld+json` adalah data, bukan skrip yang dieksekusi,
    // sehingga tidak diatur oleh `script-src`.
    if (/type\s*=\s*["']application\/ld\+json["']/i.test(attrs)) continue;
    out.push(body);
  }
  return out;
}

/**
 * Menyamakan akhiran baris dengan yang dilihat peramban.
 *
 * Peramban menghitung sidik dari teks skrip **setelah** praproses aliran
 * masukan HTML, yang mengubah CRLF dan CR menjadi LF. Berkas yang ditulis di
 * Windows menyimpan CRLF, sehingga sidik yang dihitung dari bita mentah tidak
 * akan pernah cocok — skripnya diblokir dan halaman berkedip putih sebelum
 * tema terpasang. Kegagalan ini hanya muncul pada build produksi, tidak pada
 * peladen pengembangan, jadi mudah lolos tanpa pemeriksaan ini.
 */
function normaliseNewlines(source) {
  return source.split("\r\n").join("\n").split("\r").join("\n");
}

/** Sidik SHA-256 dalam bentuk yang dipahami kebijakan keamanan konten. */
function sha256(source) {
  const digest = createHash("sha256")
    .update(normaliseNewlines(source), "utf8")
    .digest("base64");
  return `'sha256-${digest}'`;
}

async function main() {
  let html;
  try {
    html = await readFile(INDEX, "utf8");
  } catch {
    console.error("dist/index.html belum ada. Jalankan `npm run build` lebih dulu.");
    process.exit(1);
  }

  const scripts = inlineScripts(html);
  const hashes = scripts.map(sha256);

  const cspRe = /(<meta\s+http-equiv="Content-Security-Policy"\s+content=")([^"]*)(")/i;
  const match = html.match(cspRe);
  if (!match) {
    console.error("Meta Content-Security-Policy tidak ditemukan di dist/index.html.");
    process.exit(1);
  }

  const directives = match[2]
    .split(";")
    .map((d) => d.trim())
    .filter(Boolean)
    .map((directive) => {
      if (!directive.startsWith("script-src")) return directive;
      // Membuang penanda pengganti dan sidik lama, lalu memasang yang baru.
      const kept = directive
        .split(/\s+/)
        .filter((token) => !token.startsWith("'sha256-"))
        .join(" ");
      return hashes.length > 0 ? `${kept} ${hashes.join(" ")}` : kept;
    });

  const updated = html.replace(cspRe, `$1${directives.join("; ")}$3`);

  if (updated.includes("REPLACE_BOOT_HASH")) {
    console.error("Penanda pengganti masih tertinggal di kebijakan keamanan.");
    process.exit(1);
  }

  await writeFile(INDEX, updated, "utf8");

  console.log(`Kebijakan keamanan konten disegel untuk ${scripts.length} skrip sebaris:`);
  for (const h of hashes) console.log(`  ${h}`);

  // Pemeriksaan akhir: pastikan tidak ada arahan yang longgar.
  //
  // `frame-ancestors` sengaja tidak diperiksa. Arahan itu diabaikan bila
  // dikirim lewat <meta> dan hanya menghasilkan galat konsol; perlindungan
  // penyematan memerlukan tajuk tanggapan, yang tidak bisa diatur di GitHub
  // Pages. Lihat SECURITY.md untuk alasan mengapa risikonya dapat diterima.
  const final = directives.join("; ");
  const scriptSrc = directives.find((d) => d.startsWith("script-src")) ?? "";
  for (const token of ["'unsafe-inline'", "'unsafe-eval'", "*"]) {
    if (scriptSrc.split(/\s+/).includes(token)) {
      console.error(`script-src memuat ${token}, yang membatalkan gunanya kebijakan ini.`);
      process.exit(1);
    }
  }
  for (const directive of ["object-src 'none'", "base-uri 'none'", "form-action 'none'"]) {
    if (!final.includes(directive)) {
      console.error(`Kebijakan kehilangan ${directive}.`);
      process.exit(1);
    }
  }
  console.log("Kebijakan lolos pemeriksaan.");
}

await main();
