/**
 * Mengubah vektor uji TSV hasil Rust menjadi berkas SQL pemuat.
 *
 * Berkas TSV memuat beberapa keluaran per baris — satu baris Bayes memuat
 * P(E), posterior, dan rasio kemungkinan sekaligus. Skrip ini memecahnya
 * menjadi satu baris tabel per keluaran, sehingga laporan ketidakcocokan
 * menunjuk ke satu perhitungan tertentu dan bukan ke sekumpulan perhitungan
 * yang kebetulan ditulis di baris yang sama.
 *
 * Keluarannya sengaja tidak dikomit. Ia sepenuhnya turunan dari berkas di
 * `tools/conform/vectors/`, dan berkas itulah yang sudah dijaga CI agar tetap
 * sepadan dengan implementasi Rust.
 *
 * .Deckyx
 */

import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const DIR = dirname(fileURLToPath(import.meta.url));
const AKAR = resolve(DIR, "..", "..");
const SUMBER = join(AKAR, "tools", "conform", "vectors");
const TUJUAN = join(AKAR, "oracle", "generated");

/** Banyaknya baris per pernyataan `INSERT ALL`. */
const UKURAN_TUMPUKAN = 100;

/** Membaca satu berkas TSV menjadi header dan baris data. */
function bacaTsv(namaBerkas) {
  const teks = readFileSync(join(SUMBER, namaBerkas), "utf8");
  const semua = teks.split(/\r?\n/).filter((b) => b.length > 0);
  const komentar = semua.filter((b) => b.startsWith("#"));
  const data = semua.filter((b) => !b.startsWith("#")).map((b) => b.split("\t"));

  const barisKeterbandingan = komentar.find((b) => b.startsWith("# keterbandingan:"));
  if (barisKeterbandingan === undefined) {
    throw new Error(`${namaBerkas}: tidak menyebutkan tingkat keterbandingan`);
  }
  const keterbandingan = barisKeterbandingan.slice("# keterbandingan:".length).trim();

  return { keterbandingan, data };
}

/** Mengutip teks untuk SQL, atau `NULL` bila kosong. */
function q(v) {
  if (v === undefined || v === null || v === "") return "NULL";
  return `'${String(v).replace(/'/g, "''")}'`;
}

/**
 * Semua pernyataan yang akan diperiksa, satu objek per pernyataan.
 *
 * `operation` menentukan fungsi PL/SQL mana yang dipanggil pemeriksanya, jadi
 * namanya harus sepadan dengan `pkg_ai_conform.jalankan_satu`.
 */
function bangunPernyataan() {
  const keluar = [];
  const berkas = readdirSync(SUMBER).filter((f) => f.endsWith(".tsv")).sort();

  for (const nama of berkas) {
    const { keterbandingan, data } = bacaTsv(nama);
    data.forEach((kol, i) => {
      const baris = i + 1;
      const dasar = { source_file: nama, line_no: baris, comparability: keterbandingan };

      switch (nama) {
        case "rng.tsv": {
          const [benih, indeks, u64, f64] = kol;
          keluar.push({ ...dasar, operation: "splitmix64_u64", arg_text1: benih, arg_text2: indeks, expected_hex: u64 });
          keluar.push({ ...dasar, operation: "splitmix64_f64", arg_text1: benih, arg_text2: indeks, expected_hex: f64 });
          break;
        }
        case "certainty.tsv": {
          const [op, a, b, hasil] = kol;
          keluar.push({ ...dasar, operation: `cf_${op}`, arg1_hex: a, arg2_hex: b, expected_hex: hasil });
          break;
        }
        case "bayes.tsv": {
          const [prior, lh, lnh, ev, post, lr] = kol;
          const args = { arg1_hex: prior, arg2_hex: lh, arg3_hex: lnh };
          keluar.push({ ...dasar, operation: "bayes_evidence", ...args, expected_hex: ev });
          keluar.push({ ...dasar, operation: "bayes_posterior", ...args, expected_hex: post });
          keluar.push({ ...dasar, operation: "bayes_likelihood_ratio", ...args, expected_hex: lr });
          break;
        }
        case "fuzzy_linear.tsv": {
          const [bentuk, p1, p2, p3, p4, x, derajat] = kol;
          keluar.push({
            ...dasar,
            operation: `fuzzy_${bentuk}`,
            arg1_hex: p1, arg2_hex: p2, arg3_hex: p3, arg4_hex: p4, arg5_hex: x,
            expected_hex: derajat,
          });
          break;
        }
        case "fuzzy_transcendental.tsv": {
          const [bentuk, p1, p2, x, derajat] = kol;
          keluar.push({
            ...dasar,
            operation: `fuzzy_${bentuk}`,
            arg1_hex: p1, arg2_hex: p2, arg3_hex: x,
            expected_hex: derajat,
          });
          break;
        }
        case "ml_exact.tsv": {
          const [op, a1, a2, a3, a4, hasil] = kol;
          if (op === "gini") {
            // Gini memakai daftar label, bukan koordinat; kolom pertamanya teks.
            keluar.push({ ...dasar, operation: "gini", arg_text1: a1, expected_hex: hasil });
          } else {
            keluar.push({
              ...dasar,
              operation: `distance_${op}`,
              arg1_hex: a1, arg2_hex: a2, arg3_hex: a3, arg4_hex: a4,
              expected_hex: hasil,
            });
          }
          break;
        }
        case "ml_entropy.tsv": {
          const [, label, , hasil] = kol;
          keluar.push({ ...dasar, operation: "entropy", arg_text1: label, expected_hex: hasil });
          break;
        }
        case "ml_gain.tsv": {
          const [, label, nilai, skala, hasil] = kol;
          // Kolom nilai ditulis Rust sebagai `nama_atribut=v1,v2,...`; nama
          // atributnya hanya untuk dibaca manusia dan tidak ikut dihitung.
          const pisah = nilai.indexOf("=");
          if (pisah < 0) throw new Error(`${nama} baris ${baris}: kolom nilai tanpa nama atribut`);
          keluar.push({
            ...dasar,
            operation: "information_gain",
            arg_text1: label,
            arg_text2: nilai.slice(pisah + 1),
            scale_hex: skala,
            expected_hex: hasil,
          });
          break;
        }
        case "fx.tsv": {
          const [label, heks] = kol;
          keluar.push({ ...dasar, operation: "roundtrip", arg_text1: label, arg1_hex: heks, expected_hex: heks });
          break;
        }
        default:
          throw new Error(`berkas vektor tak dikenal: ${nama}`);
      }
    });
  }
  return keluar;
}

const KOLOM = [
  "vec_id",
  "source_file", "line_no", "comparability", "operation",
  "arg1_hex", "arg2_hex", "arg3_hex", "arg4_hex", "arg5_hex",
  "arg_text1", "arg_text2", "scale_hex", "expected_hex",
];

function tulisSql(pernyataan) {
  const bagian = [
    "-- Dihasilkan oleh oracle/tools/make-load-sql.mjs. Jangan disunting tangan.",
    "-- Sumbernya tools/conform/vectors/*.tsv, keluaran implementasi Rust.",
    "--",
    "-- .Deckyx",
    "SET DEFINE OFF",
    "WHENEVER SQLERROR EXIT SQL.SQLCODE",
    "DELETE FROM ai_conformance_result;",
    "DELETE FROM ai_conformance_vector;",
    "",
  ];

  for (let i = 0; i < pernyataan.length; i += UKURAN_TUMPUKAN) {
    const tumpukan = pernyataan.slice(i, i + UKURAN_TUMPUKAN);
    bagian.push("INSERT ALL");
    for (const p of tumpukan) {
      const nilai = KOLOM.map((k) => (k === "line_no" || k === "vec_id" ? p[k] : q(p[k]))).join(", ");
      bagian.push(`  INTO ai_conformance_vector (${KOLOM.join(", ")}) VALUES (${nilai})`);
    }
    bagian.push("SELECT 1 FROM dual;");
    bagian.push("");
  }

  bagian.push("COMMIT;");
  bagian.push(
    `PROMPT ${pernyataan.length} pernyataan konformansi dimuat.`,
  );
  bagian.push("");
  return bagian.join("\n");
}

const pernyataan = bangunPernyataan();
// Nomor diberikan di sini, bukan oleh basis data, supaya sebuah pernyataan uji
// selalu bernomor sama selama berkas vektornya tidak berubah.
pernyataan.forEach((p, i) => {
  p.vec_id = i + 1;
});
mkdirSync(TUJUAN, { recursive: true });
const keluaran = join(TUJUAN, "load_vectors.sql");
writeFileSync(keluaran, tulisSql(pernyataan), "utf8");

const perBerkas = new Map();
for (const p of pernyataan) {
  perBerkas.set(p.source_file, (perBerkas.get(p.source_file) ?? 0) + 1);
}
console.log(`${pernyataan.length} pernyataan ditulis ke ${keluaran}`);
for (const [berkas, n] of [...perBerkas].sort()) {
  console.log(`  ${berkas.padEnd(28)} ${String(n).padStart(5)}`);
}
