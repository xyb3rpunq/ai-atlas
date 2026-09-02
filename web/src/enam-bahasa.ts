/**
 * Halaman "Enam bahasa, satu angka".
 *
 * # Pertanyaan yang dijawab halaman ini
 *
 * Enam implementasi menghitung algoritma yang sama, dan CI membuktikan
 * keenamnya sepakat. Kalimat itu sudah tertulis di README keempat situs —
 * tetapi tidak ada satu pun tempat yang memperlihatkan **rupa** kesepakatan
 * itu. "Sepakat" pada bilangan pecahan bukan satu hal melainkan empat, dan
 * bedanya menentukan apakah sebuah selisih itu cacat atau bukan.
 *
 * Halaman ini menaruh keenam jawaban berdampingan, sampai ke bitnya, untuk
 * satu vektor yang bisa dipilih pengunjung.
 *
 * # Kenapa polanya digambar per bit
 *
 * Karena `3fee16d2942c1b98` dan `3fee16d2942c1b99` terlihat sama sekilas, dan
 * justru selisih sekecil itulah yang sedang dibicarakan. Digambar sebagai 64
 * kotak dengan tanda, eksponen, dan mantisa yang terpisah warnanya, satu bit
 * yang berbeda tidak bisa lagi terlewat — dan letaknya di dalam mantisa
 * langsung memberi tahu seberapa besar selisihnya.
 *
 * # Kenapa angkanya tidak dihitung di sini
 *
 * Karena halaman yang menghitung sendiri jawaban enam bahasa lain hanya
 * menampilkan pendapat JavaScript tentang keenamnya. Setiap pola bit di sini
 * datang dari berkas yang dipancarkan bahasa itu sendiri, dari jalan
 * konformansinya sendiri — Go dan PL/SQL dari CI, karena keduanya tidak
 * terpasang di mesin tempat halaman ini disusun.
 *
 * .Deckyx
 */

import data from "./data/pola-bit.json";
import { bi, pick, type Bilingual } from "./i18n.js";
import { card, el, table } from "./ui.js";

/** Alamat halaman ini setelah tanda pagar. */
export const SLUG_ENAM_BAHASA = "enam-bahasa";

/** Golongan sebuah jawaban terhadap acuan Rust. */
type Golongan = "identik" | "tandaNol" | "dalamToleransi" | "luarToleransi";

/** Satu pernyataan di dalam sebuah tengara. */
interface Pernyataan {
  kolom: string;
  hasil: Record<string, string>;
  golongan: Record<string, Golongan>;
}

/** Sebuah vektor tengara beserta jawaban tiap bahasa. */
interface Tengara {
  id: string;
  berkas: string;
  baris: number;
  tingkat: string;
  kolom: string[];
  masukan: string[];
  pernyataan: Pernyataan[];
}

interface KeteranganBahasa {
  kode: string;
  nama: string;
  ada: boolean;
  versi: string | null;
  dihasilkan: string | null;
  perintah: string | null;
  pernyataan: number | null;
  catatan: string | null;
}

interface HitungGolongan {
  identik: number;
  tandaNol: number;
  dalamToleransi: number;
  luarToleransi: number;
  hilang: number;
  ulpTerjauh: number;
}

interface RingkasanBerkas {
  berkas: string;
  tingkat: string;
  pernyataan: number;
  perBahasa: Record<string, HitungGolongan>;
}

export const TENGARA = data.tengara as Tengara[];
export const BAHASA = data.bahasa as KeteranganBahasa[];
export const RINGKASAN = data.ringkasan as RingkasanBerkas[];

const T = {
  judul: bi("Enam bahasa, satu angka", "Six Languages, One Number"),
  ringkas: bi(
    "Algoritma yang sama ditulis enam kali, dalam enam bahasa, oleh orang yang sama tetapi dari rumusnya — bukan diterjemahkan dari kode mana pun. Halaman ini menaruh keenam jawabannya berdampingan, sampai ke bitnya.",
    "The same algorithms written six times, in six languages, by one person but each from the formula rather than translated from any existing code. This page puts all six answers side by side, down to the bit.",
  ),
  kenapa: bi("Kenapa halaman ini ada", "Why this page exists"),
  kenapaIsi: bi(
    "Sebuah rumus yang salah tetap konsisten dengan dirinya sendiri. Uji terhadap satu implementasi tidak bisa menangkapnya — ia hanya membuktikan kodenya sepakat dengan dirinya sendiri, yang memang selalu benar. Yang bisa menangkapnya adalah implementasi kedua yang ditulis terpisah, dan yang ketiga, dan seterusnya: hampir mustahil enam orang yang bekerja dari rumus yang sama salah dengan cara yang persis sama.",
    "A wrong formula is still perfectly consistent with itself. Testing one implementation cannot catch that — it only proves the code agrees with itself, which it always does. What catches it is a second implementation written independently, and a third, and so on: it is very nearly impossible for six separate readings of the same formula to be wrong in exactly the same way.",
  ),
  kenapaDua: bi(
    "Kalimat “keenamnya sepakat” sudah tertulis di README keempat situs. Yang belum pernah bisa dilihat siapa pun adalah rupa kesepakatan itu — dan “sepakat” pada bilangan pecahan bukan satu hal melainkan empat.",
    "The sentence “all six agree” already appears in four READMEs. What has never been visible is what that agreement looks like — and “agreement” between floating-point numbers is not one thing but four.",
  ),
  tingkatJudul: bi("Empat tingkat keterbandingan", "Four tiers of comparability"),
  tingkatIsi: bi(
    "IEEE-754 hanya mewajibkan enam operasi dibulatkan dengan benar: tambah, kurang, kali, bagi, akar, dan perbandingan. Fungsi transendental seperti exp, ln, log₂, dan pow tidak termasuk — pustaka matematika yang berbeda boleh menghasilkan nilai yang berbeda satu ULP untuk masukan yang sama, dan keduanya tetap benar menurut standar. Menuntut kesamaan bit di sana berarti menuntut sesuatu yang tidak dijanjikan siapa pun.",
    "IEEE-754 requires correct rounding for only six operations: add, subtract, multiply, divide, square root, and comparison. Transcendental functions such as exp, ln, log₂, and pow are not among them — different maths libraries may return values one ULP apart for the same input, and both remain correct by the standard. Demanding bit equality there is demanding something nobody promised.",
  ),
  kolomTingkat: bi("Tingkat", "Tier"),
  kolomBerlaku: bi("Berlaku untuk", "Applies to"),
  kolomTuntutan: bi("Yang dituntut", "What is required"),
  tengaraJudul: bi("Delapan vektor yang punya cerita", "Eight vectors with a story"),
  tengaraIsi: bi(
    "Dipilih karena masing-masing memperlihatkan sesuatu, bukan karena mewakili secara statistik. Pilih salah satu, lalu bandingkan pola bit keenam bahasanya.",
    "Chosen because each one shows something, not because they are statistically representative. Pick one and compare the bit patterns across all six languages.",
  ),
  masukan: bi("Masukan", "Inputs"),
  jawaban: bi("Jawaban tiap bahasa", "What each language answered"),
  kolomBahasa: bi("Bahasa", "Language"),
  kolomPola: bi("Pola bit", "Bit pattern"),
  kolomDesimal: bi("Sebagai desimal", "As a decimal"),
  kolomSelisih: bi("Selisih dari Rust", "Distance from Rust"),
  acuan: bi("acuan", "reference"),
  sama: bi("identik", "identical"),
  belumAda: bi("belum terkumpul", "not yet collected"),
  ulp: bi("%n ULP", "%n ULP"),
  tandaNol: bi("tanda nol saja", "sign of zero only"),
  dalamToleransi: bi("%n ULP — di dalam %t", "%n ULP — within %t"),
  luarToleransi: bi("%n ULP — MELEBIHI %t", "%n ULP — EXCEEDS %t"),
  angkaJudul: bi("Angkanya", "The numbers"),
  angkaIsi: bi(
    "Enam implementasi, 3.796 pernyataan masing-masing. Tidak satu pun selisih yang melebihi tingkat keterbandingannya — dan itu bukan hal yang sama dengan “seluruhnya identik”, yang justru tidak benar.",
    "Six implementations, 3,796 statements each. Not one difference exceeds its comparability tier — which is not the same as “all identical”, and that stronger claim would be false.",
  ),
  kolomIdentik: bi("Identik", "Identical"),
  kolomTandaNol: bi("Tanda nol", "Zero sign"),
  kolomDalam: bi("Beda, masih lolos", "Differs, still passes"),
  kolomLuar: bi("Melebihi tingkat", "Exceeds tier"),
  kolomUlpTerjauh: bi("ULP terjauh", "Widest ULP"),
  tigaIdentik: bi(
    "Lua, Swift, dan Python menghasilkan pola bit yang sama persis dengan Rust pada ketiga ribu tujuh ratus sembilan puluh enam pernyataan — tanpa satu bit pun berbeda. Go berbeda pada 47, seluruhnya menyentuh exp atau log₂. Oracle berbeda pada 49: 44 di antaranya hanya soal tanda nol, dan lima sisanya selisih yang tingkatnya memang mengizinkan.",
    "Lua, Swift, and Python produce bit patterns identical to Rust across all 3,796 statements — not one bit differs. Go differs on 47, every one of them touching exp or log₂. Oracle differs on 49: 44 are purely the sign of zero, and the remaining five are differences its tier permits.",
  ),
  petaBit: bi("Peta bit jawabannya", "Bit map of the answer"),
  petaBitIsi: bi(
    "Satu kotak satu bit, dibaca dari kiri: tanda, sebelas bit eksponen, lalu lima puluh dua bit mantisa. Kotak yang berbeda dari jawaban Rust diberi bingkai.",
    "One square per bit, read from the left: sign, eleven exponent bits, then fifty-two mantissa bits. Squares that differ from Rust's answer are outlined.",
  ),
  tanda: bi("tanda", "sign"),
  eksponen: bi("eksponen", "exponent"),
  mantisa: bi("mantisa", "mantissa"),
  seluruhnya: bi("Seluruh 3.796 pernyataan", "All 3,796 statements"),
  seluruhnyaIsi: bi(
    "Tengara di atas hanya delapan. Tabel ini menghitung sisanya: untuk tiap berkas vektor, berapa pernyataan yang pola bitnya sama persis dengan Rust di tiap bahasa.",
    "The landmarks above are only eight. This table counts the rest: for each vector file, how many statements match Rust's bit pattern exactly, in each language.",
  ),
  kolomBerkas: bi("Berkas", "File"),
  kolomPernyataan: bi("Pernyataan", "Statements"),
  asalJudul: bi("Dari mana angkanya", "Where the numbers come from"),
  asalIsi: bi(
    "Setiap pola bit di halaman ini dipancarkan bahasa itu sendiri, dari jalan konformansinya sendiri, lewat panggilan yang sama yang membandingkannya. Tidak ada satu pun angka yang dihitung ulang di sini — halaman yang menghitung sendiri jawaban enam bahasa lain hanya menampilkan pendapat JavaScript tentang keenamnya.",
    "Every bit pattern on this page was emitted by that language itself, from its own conformance run, through the same call that compares it. Not one number is recomputed here — a page that computed six other languages' answers itself would only be showing JavaScript's opinion of all six.",
  ),
  kolomVersi: bi("Versi", "Version"),
  kolomPerintah: bi("Dihasilkan oleh", "Produced by"),
  kolomDihasilkan: bi("Kapan", "When"),
  tidakLengkap: bi(
    "Pola bit %s belum terkumpul, jadi kolomnya kosong — bukan berarti jawabannya berbeda.",
    "Bit patterns for %s have not been collected, so those columns are empty — which does not mean the answers differ.",
  ),
} as const;

/** Penjelasan tiap tengara, dipasangkan lewat `id`. */
export const CERITA: Record<string, { judul: Bilingual; isi: Bilingual }> = {
  "nol-negatif-cf": {
    judul: bi("Nol yang punya tanda", "The zero that has a sign"),
    isi: bi(
      "Menggabungkan dua bukti yang sama-sama menyangkal sepenuhnya menghasilkan nol — tetapi nol negatif. IEEE-754 memang punya dua nol, dan keduanya berbeda pola bit meski −0 = +0 bernilai benar. Oracle tidak bisa menghasilkan nol negatif sama sekali: BINARY_DOUBLE-nya menyimpannya, tetapi tidak ada jalan aritmetika yang sampai ke sana. Ketimbang melonggarkan perbandingannya, konformansinya memberi kasus ini putusan tersendiri yang dihitung terpisah — 44 kasus, dilaporkan tiap jalan, sehingga tidak bisa diam-diam bertambah.",
      "Combining two pieces of evidence that both deny completely gives zero — but negative zero. IEEE-754 really does have two zeroes, with different bit patterns, even though −0 = +0 is true. Oracle cannot produce a negative zero at all: its BINARY_DOUBLE stores one, but no arithmetic path reaches it. Rather than loosening the comparison, the conformance harness gives this case its own verdict, counted separately — 44 cases, reported on every run, so the number cannot quietly grow.",
    ),
  },
  "nol-negatif-entropi": {
    judul: bi("Entropi yang juga negatif nol", "Entropy that is also negative zero"),
    isi: bi(
      "Entropi satu label tunggal adalah nol: tidak ada ketidakpastian sama sekali. Tetapi rumusnya −Σ p log₂ p, dan −(1 × 0) menghasilkan nol negatif. Tanda itu benar secara aritmetika dan tidak berarti apa-apa secara ilmu — dan justru karena itu ia gampang dianggap tidak penting sampai dua implementasi menghasilkan pola bit berbeda.",
      "The entropy of a single label is zero: no uncertainty at all. But the formula is −Σ p log₂ p, and −(1 × 0) is negative zero. That sign is arithmetically correct and scientifically meaningless — which is exactly why it is easy to dismiss until two implementations produce different bit patterns.",
    ),
  },
  "perolehan-informasi": {
    judul: bi("Selisih dua angka yang hampir sama", "The difference of two nearly equal numbers"),
    isi: bi(
      "Perolehan informasi adalah H(sebelum) − H(sesudah). Pada data tenis, 0,94 dikurangi 0,91. Galat dua ULP pada H — wajar, karena log₂ bukan operasi yang dibulatkan dengan benar menurut IEEE-754 — bernilai mutlak sekitar 2,2×10⁻¹⁶. Pada hasil sebesar 0,029, nilai itu sama dengan 64 ULP. Menuntut NearlyEqual(4) di sana berarti menuntut log₂ yang lebih teliti daripada yang diwajibkan standar mana pun. Karena itu tingkat keempat lahir: toleransinya diukur pada skala tempat aritmetikanya sungguh-sungguh terjadi, bukan pada hasilnya.",
      "Information gain is H(before) − H(after). On the tennis dataset, 0.94 minus 0.91. A two-ULP error in H — entirely reasonable, since log₂ is not a correctly-rounded operation under IEEE-754 — is about 2.2×10⁻¹⁶ in absolute terms. Against a result of 0.029, that is 64 ULP. Demanding NearlyEqual(4) there would demand a log₂ more accurate than any standard requires. That is why the fourth tier exists: its tolerance is measured at the scale where the arithmetic actually happened, not at the scale of the result.",
    ),
  },
  transendental: {
    judul: bi("Fungsi yang standarnya tidak jamin", "The function the standard does not guarantee"),
    isi: bi(
      "Keanggotaan Gauss memakai exp. IEEE-754 tidak mewajibkan exp dibulatkan dengan benar, jadi dua pustaka matematika boleh berbeda satu ULP untuk masukan yang sama dan keduanya tetap benar. Sebuah uji jaringan syaraf di proyek ini pernah lolos di Windows dan gagal di Linux karena persis alasan ini — bukan flake, dan bukan bug di kodenya.",
      "Gaussian membership uses exp. IEEE-754 does not require exp to be correctly rounded, so two maths libraries may differ by one ULP on the same input and both remain correct. A neural-network test in this project once passed on Windows and failed on Linux for exactly this reason — not a flake, and not a bug in the code.",
    ),
  },
  "pembangkit-acak": {
    judul: bi("Bilangan bulat tidak punya alasan berbeda", "Integers have no excuse"),
    isi: bi(
      "SplitMix64 hanya memakai perkalian, geseran, dan XOR pada bilangan bulat 64 bit. Tidak ada pembulatan yang terlibat sama sekali, jadi keenam bahasa wajib menghasilkan bit yang sama persis — dan kegagalan di sini jauh lebih serius daripada selisih pecahan mana pun. Di Lua, membaca 18446744073709551615 dengan tonumber diam-diam mengubahnya menjadi pecahan dan menghilangkan digit terakhirnya; benihnya harus diuraikan per digit supaya luapan perkaliannya berputar dengan benar.",
      "SplitMix64 uses only multiplication, shifts, and XOR on 64-bit integers. No rounding is involved at all, so all six languages must produce identical bits — and a failure here is far more serious than any floating-point difference. In Lua, reading 18446744073709551615 with tonumber silently turns it into a float and loses the last digit; the seed has to be parsed digit by digit so that the multiplication overflow wraps correctly.",
    ),
  },
  "bolak-balik": {
    judul: bi("Angka yang membuat proyek ini memakai pola bit", "The number that made this project use bit patterns"),
    isi: bi(
      "0,42000000000000004 adalah hasil 0,9×0,2 + 0,3×0,8. Saat harness ini dibangun, pengukuran menemukan serde_json::from_str::<f64> salah membulat sebesar 1 ULP pada 27.548 dari 200.000 nilai uji — sementara str::parse::<f64> bawaan Rust nol kesalahan pada himpunan yang sama. Menulis angka ini sebagai desimal lalu membacanya kembali bisa menghasilkan 0,42: angka yang berbeda. Sejak temuan itu, seluruh vektor lintas bahasa memakai 16 digit heksadesimal pola bit, yang tidak punya ruang tafsir sama sekali.",
      "0.42000000000000004 is what 0.9×0.2 + 0.3×0.8 gives. While this harness was being built, measurement found that serde_json::from_str::<f64> mis-rounds by 1 ULP on 27,548 of 200,000 test values — while Rust's own str::parse::<f64> gets none wrong on the same set. Write this number as a decimal and read it back and you may get 0.42: a different number. Since that finding, every cross-language vector uses 16 hex digits of bit pattern, which leaves no room for interpretation.",
    ),
  },
  "laju-dasar": {
    judul: bi("Tes 99% benar yang lebih sering salah", "The 99%-accurate test that is usually wrong"),
    isi: bi(
      "Penyakit menimpa 1 dari 1.000 orang; tesnya mendeteksi 99% yang sakit dan salah pada 2% yang sehat. Hasil positif berarti peluang sakit sebesar 4,7% — bukan 99%. Yang mengecoh bukan tesnya melainkan laju dasarnya, dan pembalikan arah pertanyaan yang tampak sepele di rumus hampir selalu keliru di kepala. Perhitungannya sendiri hanya memakai kali, bagi, dan tambah, jadi keenam bahasa wajib sepakat bit demi bit.",
      "A disease affects 1 in 1,000; the test detects 99% of the ill and errs on 2% of the healthy. A positive result means a 4.7% chance of illness — not 99%. What misleads is not the test but the base rate, and the reversal of the question that looks trivial on paper is nearly always got wrong in the head. The computation itself uses only multiplication, division, and addition, so all six languages must agree bit for bit.",
    ),
  },
  "akar-kuadrat": {
    judul: bi("Akar kuadrat, yang justru dijamin", "Square root, which is guaranteed"),
    isi: bi(
      "Jarak Euclidean memakai akar kuadrat — dan akar kuadrat adalah satu dari enam operasi yang IEEE-754 wajibkan dibulatkan dengan benar. Karena itu ia bertingkat BitExact bersama tambah, kurang, kali, bagi, dan perbandingan, sementara exp dan log₂ tidak. Perbedaan itu bukan selera melainkan isi standarnya.",
      "Euclidean distance uses a square root — and square root is one of the six operations IEEE-754 requires to be correctly rounded. That is why it sits at the BitExact tier alongside add, subtract, multiply, divide, and comparison, while exp and log₂ do not. The distinction is not taste; it is what the standard says.",
    ),
  },
};

/** Nama tingkat keterbandingan beserta artinya. */
const TINGKAT: { nama: string; berlaku: Bilingual; tuntutan: Bilingual }[] = [
  {
    nama: "BitExact",
    berlaku: bi("Hanya + − × ÷ √ dan perbandingan", "Only + − × ÷ √ and comparison"),
    tuntutan: bi("Identik bit demi bit", "Identical bit for bit"),
  },
  {
    nama: "NearlyEqual(4)",
    berlaku: bi("Menyentuh exp, ln, log₂, pow", "Touches exp, ln, log₂, pow"),
    tuntutan: bi("Selisih paling banyak 4 ULP", "At most 4 ULP apart"),
  },
  {
    nama: "CancellingDifference(4)",
    berlaku: bi(
      "Hasil berupa selisih dua besaran yang hampir sama",
      "Results that are the difference of two nearly equal quantities",
    ),
    tuntutan: bi(
      "Paling banyak 4 ULP, diukur pada skala masukannya",
      "At most 4 ULP, measured at the scale of the inputs",
    ),
  },
  {
    nama: "PropertyOnly",
    berlaku: bi("Perhitungan kacau, mis. pelatihan yang divergen", "Chaotic computations, e.g. diverging training"),
    tuntutan: bi("Hanya sifatnya, bukan angkanya", "Only the property, not the number"),
  },
];

// ---------------------------------------------------------------------------
// Pola bit
// ---------------------------------------------------------------------------

/** Mengubah 16 digit heksadesimal menjadi 64 bit sebagai untai "0"/"1". */
export function keBit(hex: string): string {
  let keluar = "";
  for (const d of hex) {
    keluar += parseInt(d, 16).toString(2).padStart(4, "0");
  }
  return keluar;
}

/**
 * Menafsirkan pola bit sebagai bilangan pecahan.
 *
 * Dipakai hanya untuk **menampilkan**, tidak pernah untuk membandingkan.
 * Perbandingannya dilakukan pada untai heksadesimalnya, karena dua pola bit
 * yang berbeda bisa menghasilkan untai desimal yang sama — nol positif dan nol
 * negatif keduanya tertulis "0".
 */
export function keDesimal(hex: string): string {
  const penyangga = new ArrayBuffer(8);
  const tampilan = new DataView(penyangga);
  for (let i = 0; i < 8; i += 1) {
    tampilan.setUint8(i, parseInt(hex.slice(i * 2, i * 2 + 2), 16));
  }
  const v = tampilan.getFloat64(0);
  if (Number.isNaN(v)) return "NaN";
  if (!Number.isFinite(v)) return v > 0 ? "∞" : "−∞";
  // Nol negatif ditulis apa adanya: itulah seluruh pokok beberapa tengara di
  // halaman ini, dan menuliskannya sebagai "0" akan menghapus justru yang
  // sedang ditunjukkan.
  if (v === 0) return Object.is(v, -0) ? "−0" : "0";
  return String(v);
}

/**
 * Jarak dua pola bit dalam ULP, atau `null` bila tidak terdefinisi.
 *
 * Dihitung pada bilangan bulat 64 bit dari pola bitnya, bukan lewat
 * pengurangan pecahan: pengurangan kehilangan ketelitian persis di daerah yang
 * sedang diukur.
 */
export function jarakUlp(a: string, b: string): number | null {
  const ke = (h: string): bigint => {
    const u = BigInt("0x" + h);
    // Pola bitnya ditafsirkan sebagai bilangan bulat **bertanda** lebih dulu,
    // lalu bagian negatifnya dicerminkan. Memakai besarannya saja — `u &
    // 0x7fff…` — memetakan nol negatif ke 2⁶³ dan nol positif ke 0, sehingga
    // keduanya dilaporkan berjarak sembilan triliun ULP padahal berjarak nol.
    const bertanda = u >= 0x8000000000000000n ? u - 0x10000000000000000n : u;
    return bertanda < 0n ? -0x8000000000000000n - bertanda : bertanda;
  };
  const eksponenPenuh = 0x7ffn;
  const cacat = (h: string) => (BigInt("0x" + h) >> 52n & eksponenPenuh) === eksponenPenuh;
  if (cacat(a) || cacat(b)) return null;
  const d = ke(a) - ke(b);
  return Number(d < 0n ? -d : d);
}

/** Gambar 64 bit sebuah pola, dengan bit yang berbeda diberi bingkai. */
function petaBit(hex: string, acuan: string): HTMLElement {
  const bit = keBit(hex);
  const bitAcuan = keBit(acuan);
  const kotak: HTMLElement[] = [];
  for (let i = 0; i < 64; i += 1) {
    const bagian = i === 0 ? "tanda" : i < 12 ? "eksponen" : "mantisa";
    const beda = bit[i] !== bitAcuan[i];
    kotak.push(
      el("span", {
        class: `bit bit--${bagian}${bit[i] === "1" ? " bit--nyala" : ""}${beda ? " bit--beda" : ""}`,
        text: bit[i],
      }),
    );
  }
  return el("div", { class: "peta-bit", attrs: { "aria-hidden": "true" }, children: kotak });
}

// ---------------------------------------------------------------------------
// Bagian halaman
// ---------------------------------------------------------------------------

function bagianTingkat(): HTMLElement {
  return card(
    pick(T.tingkatJudul),
    el("p", { class: "note", text: pick(T.tingkatIsi) }),
    table(
      [pick(T.kolomTingkat), pick(T.kolomBerlaku), pick(T.kolomTuntutan)],
      TINGKAT.map((t) => [t.nama, pick(t.berlaku), pick(t.tuntutan)]),
    ),
  );
}

/** Kartu satu tengara: ceritanya, masukannya, dan jawaban tiap bahasa. */
function kartuTengara(t: Tengara): HTMLElement {
  const cerita = CERITA[t.id];
  const anak: (HTMLElement | null)[] = [];

  if (cerita) {
    anak.push(el("p", { text: pick(cerita.isi) }));
  }

  anak.push(
    el("p", {
      class: "note",
      text: `${t.berkas}:${t.baris} · ${t.tingkat}`,
    }),
  );

  // Masukannya ditampilkan apa adanya dari berkas vektornya: nama kolom di
  // atas, isinya di bawah. Menerjemahkannya menjadi angka desimal akan
  // menyembunyikan bahwa yang dipertukarkan memang pola bit.
  anak.push(
    table(
      t.kolom,
      [t.masukan.map((m) => m)],
    ),
  );

  for (const p of t.pernyataan) {
    const acuan = p.hasil["rust"] ?? "";
    const baris = BAHASA.map((b) => {
      const h = p.hasil[b.kode];
      if (h === undefined) return [b.nama, "—", "—", pick(T.belumAda)];
      if (b.kode === "rust") return [b.nama, h, keDesimal(h), pick(T.acuan)];

      const g = p.golongan[b.kode];
      const d = jarakUlp(h, acuan);
      const n = d === null ? "—" : String(d);
      const selisih =
        g === "identik"
          ? pick(T.sama)
          : g === "tandaNol"
            ? pick(T.tandaNol)
            : pick(g === "luarToleransi" ? T.luarToleransi : T.dalamToleransi)
                .replace("%n", n)
                .replace("%t", t.tingkat);
      return [b.nama, h, keDesimal(h), selisih];
    });

    anak.push(
      el("h3", { class: "sub", text: `${p.kolom} · ${pick(T.jawaban)}` }),
      table(
        [pick(T.kolomBahasa), pick(T.kolomPola), pick(T.kolomDesimal), pick(T.kolomSelisih)],
        baris,
      ),
      el("p", { class: "note", text: pick(T.petaBitIsi) }),
      ...BAHASA.filter((b) => p.hasil[b.kode] !== undefined).map((b) =>
        el("div", {
          class: "peta-bit__baris",
          children: [
            el("span", { class: "peta-bit__nama", text: b.nama }),
            petaBit(p.hasil[b.kode]!, acuan),
          ],
        }),
      ),
    );
  }

  return card(cerita ? pick(cerita.judul) : `${t.berkas}:${t.baris}`, ...anak);
}

/** Menjumlahkan satu golongan di seluruh berkas untuk sebuah bahasa. */
function total(kode: string, bidang: keyof HitungGolongan): number {
  return RINGKASAN.reduce((n, r) => n + (r.perBahasa[kode]?.[bidang] ?? 0), 0);
}

function bagianAngka(): HTMLElement {
  const hadir = BAHASA.filter((b) => b.ada && b.kode !== "rust");
  const baris = hadir.map((b) => [
    b.nama,
    total(b.kode, "identik"),
    total(b.kode, "tandaNol"),
    total(b.kode, "dalamToleransi"),
    total(b.kode, "luarToleransi"),
    total(b.kode, "ulpTerjauh") === 0
      ? "0"
      : String(Math.max(...RINGKASAN.map((r) => r.perBahasa[b.kode]?.ulpTerjauh ?? 0))),
  ]);

  return card(
    pick(T.angkaJudul),
    el("p", { text: pick(T.angkaIsi) }),
    table(
      [
        pick(T.kolomBahasa),
        pick(T.kolomIdentik),
        pick(T.kolomTandaNol),
        pick(T.kolomDalam),
        pick(T.kolomLuar),
        pick(T.kolomUlpTerjauh),
      ],
      baris,
    ),
    el("p", { class: "note", text: pick(T.tigaIdentik) }),
  );
}

function bagianRingkasan(): HTMLElement {
  const hadir = BAHASA.filter((b) => b.ada && b.kode !== "rust");
  const kepala = [pick(T.kolomBerkas), pick(T.kolomTingkat), pick(T.kolomPernyataan)];
  for (const b of hadir) kepala.push(b.nama);

  const baris = RINGKASAN.map((r) => {
    const kol: (string | number)[] = [r.berkas, r.tingkat, r.pernyataan];
    for (const b of hadir) {
      const s = r.perBahasa[b.kode];
      if (s === undefined) {
        kol.push("—");
      } else if (s.luarToleransi > 0) {
        kol.push(`${s.luarToleransi} ✗`);
      } else if (s.identik === r.pernyataan) {
        kol.push("identik");
      } else {
        // Yang ditampilkan berapa yang berbeda, bukan berapa yang sama:
        // angka kecil lebih mudah dinilai daripada selisih dua angka besar.
        kol.push(`${s.tandaNol + s.dalamToleransi} ≈`);
      }
    }
    return kol;
  });

  return card(
    pick(T.seluruhnya),
    el("p", { class: "note", text: pick(T.seluruhnyaIsi) }),
    table(kepala, baris),
  );
}

function bagianAsal(): HTMLElement {
  const belum = BAHASA.filter((b) => !b.ada).map((b) => b.nama);
  const anak: (HTMLElement | null)[] = [
    el("p", { class: "note", text: pick(T.asalIsi) }),
    table(
      [
        pick(T.kolomBahasa),
        pick(T.kolomVersi),
        pick(T.kolomPerintah),
        pick(T.kolomDihasilkan),
        pick(T.kolomPernyataan),
      ],
      BAHASA.map((b) => [
        b.nama,
        b.versi ?? "—",
        b.perintah ?? "—",
        b.dihasilkan ?? "—",
        b.ada ? (b.pernyataan === null ? pick(T.acuan) : b.pernyataan) : pick(T.belumAda),
      ]),
    ),
  ];

  if (belum.length > 0) {
    anak.push(
      el("p", {
        class: "error",
        attrs: { role: "note" },
        text: pick(T.tidakLengkap).replace("%s", belum.join(", ")),
      }),
    );
  }

  return card(pick(T.asalJudul), ...anak);
}

/**
 * Nama halaman untuk kepala berkas ekspor.
 *
 * Berupa fungsi, bukan tetapan: bahasanya bisa berganti setelah modul ini
 * dimuat, dan nama yang dibekukan saat impor akan tetap memakai bahasa yang
 * aktif waktu itu.
 */
export function NAMA_HALAMAN(): string {
  return pick(T.judul);
}

/** Seluruh halaman. */
export function halamanEnamBahasa(): HTMLElement {
  return el("div", {
    children: [
      el("header", {
        class: "lab-head",
        children: [
          el("div", { class: "lab-head__eyebrow", text: "IND323 · KONFORMANSI" }),
          el("h1", { text: pick(T.judul) }),
          el("p", { text: pick(T.ringkas) }),
        ],
      }),
      card(
        pick(T.kenapa),
        el("p", { text: pick(T.kenapaIsi) }),
        el("p", { text: pick(T.kenapaDua) }),
      ),
      bagianAngka(),
      bagianTingkat(),
      card(
        pick(T.tengaraJudul),
        el("p", { class: "note", text: pick(T.tengaraIsi) }),
      ),
      ...TENGARA.map(kartuTengara),
      bagianRingkasan(),
      bagianAsal(),
    ],
  });
}
