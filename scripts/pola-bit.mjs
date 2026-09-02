/**
 * Menyusun data halaman "Enam bahasa, satu angka".
 *
 * # Apa yang dikumpulkan
 *
 * Enam implementasi menghitung algoritma yang sama, dan CI membuktikan
 * ketiganya sepakat. Yang belum pernah bisa dilihat siapa pun adalah **pola
 * bit yang sebenarnya dihasilkan masing-masing** untuk satu vektor tertentu.
 * Berkas ini menyatukannya.
 *
 * Rust tidak punya berkas tersendiri: vektor di `tools/conform/vectors/`
 * **adalah** keluaran Rust, dihasilkan `cargo run -p ai-core --bin
 * export_vectors`. Kolom `expected` di sana jawaban Rust menurut definisi.
 *
 * Kelima lainnya memancarkan berkasnya sendiri, masing-masing dari jalan
 * konformansinya sendiri:
 *
 *   go.tsv      go run ./tools/conform --pancar        (artefak CI)
 *   plsql.tsv   bash oracle/run.sh                     (artefak CI)
 *   lua.tsv     npm run pancar        di kecerdasan-buatan
 *   python.tsv  python conformance/pancar.py  di neuronusa
 *   swift.tsv   aikit-cli pancar      di ind323-ai-lab (artefak CI)
 *
 * # Kenapa hanya sebagian yang diterbitkan
 *
 * Karena 3.796 pernyataan dikali enam bahasa berukuran sekitar 1,4 MB, dan
 * seluruh situs ini beranggaran 460 KB. Yang diterbitkan adalah tengara —
 * vektor yang punya cerita — beserta ringkasan seluruh sisanya. Ringkasan itu
 * yang menjawab "apakah keenamnya sepakat"; tengaranya yang memperlihatkan
 * bagaimana rupa kesepakatan itu sampai ke bitnya.
 *
 * # Kenapa bahasa yang hilang tidak disembunyikan
 *
 * Kolom kosong terbaca sebagai perbedaan. Berkas keluarannya mencatat bahasa
 * mana yang datanya ada dan mana yang tidak, dan halamannya menyebutkannya —
 * bukan membiarkan pembaca menyimpulkan sendiri.
 *
 * .Deckyx
 */

import { readFileSync, readdirSync, writeFileSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const AKAR = join(dirname(fileURLToPath(import.meta.url)), "..");
const VEKTOR = join(AKAR, "tools", "conform", "vectors");
const PANCARAN = join(AKAR, "pola-bit");
const TUJUAN = join(AKAR, "web", "src", "data", "pola-bit.json");

/**
 * Bahasa yang ikut dibandingkan, berurutan menurut kemunculannya di proyek.
 *
 * Rust lebih dulu karena ia yang menghasilkan vektornya; sisanya menyusul
 * menurut urutan ditulisnya.
 */
const BAHASA = [
  { kode: "rust", nama: "Rust", berkas: null },
  { kode: "go", nama: "Go", berkas: "go.tsv" },
  { kode: "plsql", nama: "Oracle PL/SQL", berkas: "plsql.tsv" },
  { kode: "lua", nama: "Lua", berkas: "lua.tsv" },
  { kode: "swift", nama: "Swift", berkas: "swift.tsv" },
  { kode: "python", nama: "Python", berkas: "python.tsv" },
];

/**
 * Vektor yang diterbitkan lengkap dengan pola bit tiap bahasanya.
 *
 * Dipilih karena masing-masing punya cerita, bukan karena mewakili secara
 * statistik. `id` dipakai halaman untuk memasangkan penjelasannya.
 */
const TENGARA = [
  { id: "nol-negatif-cf", berkas: "certainty.tsv", baris: 2 },
  { id: "nol-negatif-entropi", berkas: "ml_entropy.tsv", baris: 1 },
  { id: "perolehan-informasi", berkas: "ml_gain.tsv", baris: 1 },
  { id: "transendental", berkas: "fuzzy_transcendental.tsv", baris: 1 },
  { id: "pembangkit-acak", berkas: "rng.tsv", baris: 1 },
  { id: "bolak-balik", berkas: "fx.tsv", baris: 6 },
  { id: "laju-dasar", berkas: "bayes.tsv", baris: 74 },
  { id: "akar-kuadrat", berkas: "ml_exact.tsv", baris: 4 },
];

/** Membaca sebuah TSV menjadi komentar dan baris data. */
function bacaTsv(jalur) {
  const semua = readFileSync(jalur, "utf8").split(/\r?\n/).filter((b) => b.length > 0);
  return {
    komentar: semua.filter((b) => b.startsWith("#")),
    data: semua.filter((b) => !b.startsWith("#")).map((b) => b.split("\t")),
  };
}

/** Mengambil nilai sebuah baris komentar `# kunci: nilai`. */
function kepala(komentar, kunci) {
  const b = komentar.find((x) => x.startsWith(`# ${kunci}:`));
  return b === undefined ? null : b.slice(`# ${kunci}:`.length).trim();
}

// ---------------------------------------------------------------------------
// Vektor: masukan, tingkat keterbandingan, dan jawaban Rust
// ---------------------------------------------------------------------------

/**
 * Kolom hasil tiap berkas, berurutan.
 *
 * Sama persis dengan peta yang dipakai kelima harness. Ditulis ulang di sini
 * dan bukan diimpor karena keenamnya bahasa berbeda; yang menjaga keduanya
 * sepadan adalah uji, bukan berbagi berkas.
 */
const KOLOM_HASIL = {
  "fx.tsv": ["hex"],
  "rng.tsv": ["next_u64_hex", "next_f64_hex"],
  "bayes.tsv": ["evidence_hex", "posterior_hex", "likelihood_ratio_hex"],
  "certainty.tsv": ["result_hex"],
  "fuzzy_linear.tsv": ["degree_hex"],
  "fuzzy_transcendental.tsv": ["degree_hex"],
  "ml_exact.tsv": ["result_hex"],
  "ml_entropy.tsv": ["result_hex"],
  "ml_gain.tsv": ["result_hex"],
};

const vektor = new Map();
const berkasVektor = readdirSync(VEKTOR).filter((f) => f.endsWith(".tsv")).sort();

for (const nama of berkasVektor) {
  const { komentar, data } = bacaTsv(join(VEKTOR, nama));
  const tingkat = kepala(komentar, "keterbandingan");
  const kolom = (kepala(komentar, "kolom") ?? "").split("\t");
  if (tingkat === null) throw new Error(`${nama}: tidak menyebutkan tingkat keterbandingan`);

  vektor.set(nama, { nama, tingkat, kolom, data });
}

// ---------------------------------------------------------------------------
// Membandingkan pola bit menurut tingkatnya
// ---------------------------------------------------------------------------

/** Menafsirkan 16 digit heksadesimal sebagai bilangan pecahan. */
function keAngka(hex) {
  const b = new ArrayBuffer(8);
  const v = new DataView(b);
  for (let i = 0; i < 8; i += 1) v.setUint8(i, parseInt(hex.slice(i * 2, i * 2 + 2), 16));
  return v.getFloat64(0);
}

/**
 * Jarak dua pola bit dalam ULP.
 *
 * Dihitung pada bilangan bulat dari pola bitnya, bukan lewat pengurangan
 * pecahan: pengurangan kehilangan ketelitian persis di daerah yang diukur.
 */
function jarakUlp(a, b) {
  const ke = (h) => {
    const u = BigInt("0x" + h);
    // Ditafsirkan sebagai bilangan bulat bertanda lebih dulu; besarannya saja
    // memetakan nol negatif ke 2⁶³ dan melaporkan jarak sembilan triliun ULP
    // untuk dua nilai yang sebenarnya sama.
    const bertanda = u >= 0x8000000000000000n ? u - 0x10000000000000000n : u;
    return bertanda < 0n ? -0x8000000000000000n - bertanda : bertanda;
  };
  const d = ke(a) - ke(b);
  return d < 0n ? -d : d;
}

/** Jarak ke pecahan berikutnya pada besaran `x`. */
function langkahUlp(x) {
  const b = new ArrayBuffer(8);
  const v = new DataView(b);
  v.setFloat64(0, Math.abs(x));
  const u = v.getBigUint64(0);
  v.setBigUint64(0, u + 1n);
  return v.getFloat64(0) - Math.abs(x);
}

/**
 * Menggolongkan sebuah jawaban terhadap acuan Rust.
 *
 * Empat kemungkinan, dan bedanya menentukan segalanya:
 *
 *   `identik`         pola bitnya sama persis
 *   `tandaNol`        keduanya nol, tandanya berbeda
 *   `dalamToleransi`  pola bitnya berbeda, tetapi masih di dalam tingkatnya
 *   `luarToleransi`   berbeda melebihi tingkatnya — ini kegagalan sungguhan
 *
 * Golongan ketiga yang paling sering disalahbaca. Pola bit yang berbeda pada
 * `exp` bukan cacat: IEEE-754 tidak mewajibkan `exp` dibulatkan dengan benar,
 * jadi dua pustaka matematika boleh berbeda satu ULP dan keduanya tetap benar.
 * Menyebutnya "gagal" akan membuat halaman ini menuduh implementasi yang
 * sebenarnya lolos.
 */
function golongkan(acuan, hasil, tingkat, skalaHex) {
  if (acuan === hasil) return "identik";

  const a = keAngka(acuan);
  const h = keAngka(hasil);
  if (a === 0 && h === 0) return "tandaNol";
  if (!Number.isFinite(a) || !Number.isFinite(h)) return "luarToleransi";

  if (tingkat === "BitExact") return "luarToleransi";

  const cocokNearly = /^NearlyEqual\((\d+)\)$/.exec(tingkat);
  if (cocokNearly) {
    return jarakUlp(acuan, hasil) <= BigInt(cocokNearly[1]) ? "dalamToleransi" : "luarToleransi";
  }

  const cocokBatal = /^CancellingDifference\((\d+)\)$/.exec(tingkat);
  if (cocokBatal) {
    // Toleransinya diukur pada skala tempat aritmetikanya sungguh-sungguh
    // terjadi, bukan pada hasilnya. Berkas yang menyatakan tingkat ini tanpa
    // kolom skala adalah salah tulis, bukan alasan melonggarkan periksa.
    if (skalaHex === undefined) throw new Error(`${tingkat} menuntut kolom scale_hex`);
    const batas = Number(cocokBatal[1]) * langkahUlp(keAngka(skalaHex));
    return Math.abs(a - h) <= batas ? "dalamToleransi" : "luarToleransi";
  }

  throw new Error(`tingkat tidak dikenal: ${tingkat}`);
}

/** Jawaban Rust untuk satu (berkas, baris, kolom). */
function jawabanRust(berkas, baris, kolomHasil) {
  const v = vektor.get(berkas);
  const kol = v.data[baris - 1];
  if (kol === undefined) throw new Error(`${berkas} baris ${baris} tidak ada`);
  const i = v.kolom.indexOf(kolomHasil);
  if (i < 0) throw new Error(`${berkas}: tidak punya kolom ${kolomHasil}`);
  return kol[i];
}

// ---------------------------------------------------------------------------
// Pancaran tiap bahasa
// ---------------------------------------------------------------------------

const pancaran = new Map();
const keterangan = [];

for (const b of BAHASA) {
  if (b.berkas === null) {
    keterangan.push({
      kode: b.kode,
      nama: b.nama,
      ada: true,
      versi: null,
      dihasilkan: null,
      perintah: "cargo run -p ai-core --bin export_vectors",
      pernyataan: null,
      catatan: "acuan",
    });
    continue;
  }

  const jalur = join(PANCARAN, b.berkas);
  if (!existsSync(jalur)) {
    keterangan.push({
      kode: b.kode,
      nama: b.nama,
      ada: false,
      versi: null,
      dihasilkan: null,
      perintah: null,
      pernyataan: 0,
      catatan: "belum terkumpul",
    });
    continue;
  }

  const { komentar, data } = bacaTsv(jalur);
  const peta = new Map();
  for (const [berkas, baris, kolom, hasil] of data) {
    peta.set(`${berkas}|${baris}|${kolom}`, hasil);
  }
  pancaran.set(b.kode, peta);
  keterangan.push({
    kode: b.kode,
    nama: b.nama,
    ada: true,
    versi: kepala(komentar, "versi"),
    dihasilkan: kepala(komentar, "dihasilkan"),
    perintah: kepala(komentar, "perintah"),
    pernyataan: data.length,
    catatan: null,
  });
}

// ---------------------------------------------------------------------------
// Tengara: satu entri per pernyataan
// ---------------------------------------------------------------------------

const tengara = TENGARA.map((t) => {
  const v = vektor.get(t.berkas);
  if (v === undefined) throw new Error(`tengara menunjuk berkas tak dikenal: ${t.berkas}`);
  const kol = v.data[t.baris - 1];
  if (kol === undefined) throw new Error(`${t.berkas} baris ${t.baris} tidak ada`);

  const iSkala = v.kolom.indexOf("scale_hex");
  const skala = iSkala >= 0 ? kol[iSkala] : undefined;

  const pernyataan = KOLOM_HASIL[t.berkas].map((kolomHasil) => {
    const acuan = jawabanRust(t.berkas, t.baris, kolomHasil);
    const hasil = { rust: acuan };
    const golongan = { rust: "identik" };
    for (const [kode, peta] of pancaran) {
      const h = peta.get(`${t.berkas}|${t.baris}|${kolomHasil}`);
      if (h === undefined) continue;
      hasil[kode] = h;
      golongan[kode] = golongkan(acuan, h, v.tingkat, skala);
    }
    return { kolom: kolomHasil, hasil, golongan };
  });

  return {
    id: t.id,
    berkas: t.berkas,
    baris: t.baris,
    tingkat: v.tingkat,
    kolom: v.kolom,
    masukan: kol,
    pernyataan,
  };
});

// ---------------------------------------------------------------------------
// Ringkasan seluruh vektor, per berkas per bahasa
// ---------------------------------------------------------------------------

const ringkasan = berkasVektor.map((nama) => {
  const v = vektor.get(nama);
  const kolomHasil = KOLOM_HASIL[nama];
  const total = v.data.length * kolomHasil.length;

  const iSkala = v.kolom.indexOf("scale_hex");

  const perBahasa = {};
  for (const [kode, peta] of pancaran) {
    const hitung = { identik: 0, tandaNol: 0, dalamToleransi: 0, luarToleransi: 0, hilang: 0 };
    let ulpTerjauh = 0;
    for (let i = 1; i <= v.data.length; i += 1) {
      const skala = iSkala >= 0 ? v.data[i - 1][iSkala] : undefined;
      for (const kh of kolomHasil) {
        const h = peta.get(`${nama}|${i}|${kh}`);
        if (h === undefined) {
          hitung.hilang += 1;
          continue;
        }
        const acuan = jawabanRust(nama, i, kh);
        const g = golongkan(acuan, h, v.tingkat, skala);
        hitung[g] += 1;
        if (g === "dalamToleransi") {
          const d = Number(jarakUlp(acuan, h));
          if (d > ulpTerjauh) ulpTerjauh = d;
        }
      }
    }
    perBahasa[kode] = { ...hitung, ulpTerjauh };
  }

  return { berkas: nama, tingkat: v.tingkat, pernyataan: total, perBahasa };
});

// ---------------------------------------------------------------------------

const keluaran = {
  disusun: new Date().toISOString().replace(/\.\d+Z$/, "Z"),
  bahasa: keterangan,
  tengara,
  ringkasan,
};

writeFileSync(TUJUAN, JSON.stringify(keluaran, null, 2) + "\n", "utf8");

const ada = keterangan.filter((k) => k.ada).length;
const ukuran = (JSON.stringify(keluaran).length / 1024).toFixed(1);
console.log(`Pola bit lintas bahasa: ${ada}/${BAHASA.length} bahasa, ${ukuran} KB → ${TUJUAN}`);
for (const k of keterangan) {
  const isi = k.ada ? `${k.pernyataan ?? "—"} pernyataan` : "BELUM TERKUMPUL";
  console.log(`  ${k.nama.padEnd(16)}${isi}`);
}
