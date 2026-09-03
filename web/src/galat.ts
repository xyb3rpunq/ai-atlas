/**
 * Kalimat untuk setiap kegagalan mesin, dalam bahasa pembacanya.
 *
 * # Kenapa di sini, bukan di mesinnya
 *
 * Karena mesin tidak tahu — dan tidak seharusnya tahu — siapa yang sedang
 * membaca. Selama kalimatnya dirakit di Rust, setiap kegagalan akan berbunyi
 * dalam Bahasa Indonesia, termasuk di halaman yang seluruh sisanya berbahasa
 * Inggris; dan sisi antarmuka tidak punya cara memperbaikinya, karena yang ia
 * terima sudah berupa kalimat jadi.
 *
 * Pesan kegagalan justru yang paling buruk untuk salah bahasa. Yang membacanya
 * sedang mengalami sesuatu yang tidak ia harapkan, dan pada saat itu ia paling
 * membutuhkan kalimat yang bisa ia baca.
 *
 * Jadi mesin menyerahkan **kode dan nilai** — `{"kode": "cf.daftar_kosong",
 * "arg": []}` — dan kalimatnya dirakit di sini. Perlakuan yang sama persis
 * dengan kalimat aturan JIKA–MAKA di `aturan.ts`.
 *
 * # Kenapa penandanya bernomor
 *
 * Karena urutan kata berbeda antarbahasa. "diberi %2, harus 1 sampai %1" dan
 * "must be 1 to %1, got %2" menyisipkan nilai yang sama di tempat yang
 * berbeda; penanda tak bernomor akan memaksa keduanya punya urutan yang sama,
 * dan terjemahan yang dipaksa berurutan seperti itu selalu terbaca janggal.
 *
 * # Kelengkapannya dijaga uji, bukan ingatan
 *
 * `error_codes()` di sisi mesin menyerahkan seluruh kode yang bisa datang
 * beserta jumlah argumennya. Uji menuntut kamus ini memuat tepat kode-kode itu
 * dengan jumlah penanda yang sepadan — sehingga kode baru yang belum
 * diterjemahkan gagal di CI, bukan ditemukan pengguna yang sedang mengalami
 * kegagalan.
 *
 * .Deckyx
 */

import { bi, pick } from "./i18n.js";
import type { Bilingual } from "./i18n.js";

/** Bentuk galat yang datang dari mesin. */
export interface GalatMesin {
  kode: string;
  arg: string[];
}

/**
 * Kalimat tiap kode, dalam kedua bahasa.
 *
 * Penandanya `%1`, `%2`, `%3`, mengikuti urutan `argumen()` di sisi Rust.
 */
export const PESAN: Readonly<Record<string, Bilingual>> = {
  // Sesi 2 — agen cerdas
  "agen.jumlah_rombongan": bi(
    "Jumlah rombongan harus 1 sampai %1, diberi %2.",
    "The party size must be 1 to %1; %2 was given.",
  ),
  "agen.jumlah_ruangan": bi(
    "Jumlah ruangan harus 1 sampai %1, diberi %2.",
    "The number of rooms must be 1 to %1; %2 was given.",
  ),
  "agen.kapasitas_teko": bi(
    "Kapasitas teko tidak sah: %1 dan %2.",
    "Invalid jug capacities: %1 and %2.",
  ),
  "agen.penyeberangan_tak_aman": bi(
    "Tidak ada urutan penyeberangan yang aman untuk rombongan sebesar itu.",
    "No safe crossing order exists for a party that size.",
  ),
  "agen.posisi_awal": bi(
    "Posisi awal %1 berada di luar %2 ruangan.",
    "Starting position %1 lies outside the %2 rooms.",
  ),
  "agen.ruang_keadaan_habis": bi(
    "Seluruh ruang keadaan sudah ditelusuri tanpa menemukan sasarannya.",
    "The whole state space was explored without reaching the target.",
  ),
  "agen.sasaran_bukan_kelipatan": bi(
    "Sasaran %1 bukan kelipatan pembagi bersama terbesar kedua teko (%2), jadi ia mustahil dicapai.",
    "Target %1 is not a multiple of the jugs' greatest common divisor (%2), so it cannot be reached.",
  ),
  "agen.sasaran_melebihi_teko": bi(
    "Sasaran %1 melebihi teko terbesar (%2).",
    "Target %1 exceeds the largest jug (%2).",
  ),

  // Sesi 10 — pengolahan bahasa
  "bahasa.korpus_kosong": bi(
    "Korpusnya kosong. Isi setidaknya satu dokumen lebih dulu.",
    "The corpus is empty. Enter at least one document first.",
  ),
  "bahasa.panjang_tak_sepadan": bi(
    "Panjang kedua vektor berbeda: %1 dan %2.",
    "The two vectors differ in length: %1 and %2.",
  ),
  "bahasa.ukuran_ngram": bi(
    "Ukuran n-gram harus minimal 1, diberi %1.",
    "The n-gram size must be at least 1; %1 was given.",
  ),

  // Sesi 4 — probabilitas Bayesian
  "bayes.belum_dilatih": bi(
    "Modelnya belum dilatih.",
    "The model has not been trained yet.",
  ),
  "bayes.bukti_nol": bi(
    "P(E) = 0, sehingga posteriornya tidak terdefinisi. Bukti yang mustahil tidak bisa memberi informasi apa pun.",
    "P(E) = 0, so the posterior is undefined. Impossible evidence carries no information.",
  ),
  "bayes.indeks_di_luar_jangkauan": bi(
    "Indeks %1 berada di luar jangkauan 0..%2.",
    "Index %1 is outside the range 0..%2.",
  ),
  "bayes.masukan_kosong": bi("Masukannya kosong.", "The input is empty."),
  "bayes.panjang_tak_sepadan": bi(
    "Panjang larik tidak sepadan: %1 dan %2.",
    "The arrays differ in length: %1 and %2.",
  ),
  "bayes.prior_tak_berjumlah_satu": bi(
    "Seluruh prior harus berjumlah 1, diperoleh %1.",
    "The priors must sum to 1; they sum to %1.",
  ),
  "bayes.probabilitas_di_luar_rentang": bi(
    "Probabilitas harus berada di rentang [0, 1], diberi %1.",
    "A probability must lie in [0, 1]; %1 was given.",
  ),

  // Sesi 8 — pencarian
  "cari.awal_terhalang": bi(
    "Titik awalnya berdiri di atas dinding.",
    "The start point stands on a wall.",
  ),
  "cari.di_luar_kisi": bi(
    "Titik (%1, %2) berada di luar kisi.",
    "Point (%1, %2) lies outside the grid.",
  ),
  "cari.kisi_tak_sah": bi(
    "Ukuran kisi tidak sah: %1 × %2.",
    "Invalid grid size: %1 × %2.",
  ),
  "cari.panjang_dinding": bi(
    "Data dinding harus %1 sel, diberi %2.",
    "The wall data must have %1 cells; %2 were given.",
  ),
  "cari.tujuan_terhalang": bi(
    "Titik tujuannya berdiri di atas dinding.",
    "The goal point stands on a wall.",
  ),

  // Sesi 3 — certainty factor
  "cf.cf_di_luar_rentang": bi(
    "CF harus berada di rentang [-1, 1], diberi %1.",
    "A CF must lie in [-1, 1]; %1 was given.",
  ),
  "cf.daftar_kosong": bi(
    "Daftar CF-nya kosong. Tambahkan setidaknya satu bukti.",
    "The list of CFs is empty. Add at least one piece of evidence.",
  ),
  "cf.mb_md_di_luar_rentang": bi(
    "MB dan MD harus berada di rentang [0, 1], diberi %1.",
    "MB and MD must lie in [0, 1]; %1 was given.",
  ),

  // Sesi 1 — ELIZA
  "eliza.aturan_tanpa_balasan": bi(
    "Aturan %1 tidak punya satu pun kalimat balasan.",
    "Rule %1 has no reply lines at all.",
  ),
  "eliza.masukan_terlalu_panjang": bi(
    "Masukan %1 karakter melebihi batas %2.",
    "An input of %1 characters exceeds the limit of %2.",
  ),
  "eliza.naskah_kosong": bi(
    "Naskahnya tidak punya satu pun aturan.",
    "The script has no rules at all.",
  ),

  // Pertukaran pecahan bit-eksak
  "fx.bukan_digit_heksadesimal": bi(
    "Bukan digit heksadesimal: %1.",
    "Not a hexadecimal digit: %1.",
  ),
  "fx.panjang_salah": bi(
    "Pola bit harus %1 digit, diberi %2.",
    "A bit pattern must be %1 digits; %2 were given.",
  ),

  // Sesi 5 & 6 — logika kabur
  "kabur.basis_aturan_kosong": bi(
    "Basis aturannya kosong.",
    "The rule base is empty.",
  ),
  "kabur.cuplikan_terlalu_sedikit": bi(
    "Butuh minimal 2 cuplikan, diberi %1.",
    "At least 2 samples are needed; %1 was given.",
  ),
  "kabur.derajat_di_luar_rentang": bi(
    "Derajat keanggotaan harus berada di [0, 1], diberi %1.",
    "A degree of membership must lie in [0, 1]; %1 was given.",
  ),
  "kabur.himpunan_tak_dikenal": bi(
    "Himpunan tidak dikenal: %1.",
    "Unknown set: %1.",
  ),
  "kabur.semesta_tak_sah": bi(
    "Semesta tidak sah: batas bawah %1 tidak kurang dari batas atas %2.",
    "Invalid universe: the lower bound %1 is not below the upper bound %2.",
  ),
  "kabur.tidak_ada_aturan_menyala": bi(
    "Tidak ada satu pun aturan yang menyala pada masukan itu. Melaporkannya jauh lebih jujur daripada mengembalikan titik tengah semesta.",
    "No rule fires on that input. Reporting it is far more honest than returning the midpoint of the universe.",
  ),
  "kabur.titik_tak_terurut": bi(
    "Titik fungsi keanggotaan tidak terurut menaik: %1.",
    "The membership-function points are not in ascending order: %1.",
  ),
  "kabur.variabel_tak_dikenal": bi(
    "Variabel tidak dikenal: %1.",
    "Unknown variable: %1.",
  ),

  // Sesi 7 — representasi pengetahuan
  "logika.basis_kosong": bi(
    "Basis pengetahuannya kosong.",
    "The knowledge base is empty.",
  ),
  "logika.batas_pembuktian": bi(
    "Melampaui %1 langkah pembuktian.",
    "Exceeded %1 proof steps.",
  ),
  "logika.simpul_tak_dikenal": bi(
    "Simpul tidak dikenal: %1.",
    "Unknown node: %1.",
  ),
  "logika.terlalu_banyak_variabel": bi(
    "%1 proposisi menghasilkan tabel yang terlalu besar; batasnya %2.",
    "%1 propositions make the table too large; the limit is %2.",
  ),
  "logika.urai_karakter_tak_dikenal": bi(
    "Karakter yang tidak dikenal pada posisi %1: %2.",
    "Unrecognised character at position %1: %2.",
  ),
  "logika.urai_kurung_tutup_hilang": bi(
    "Kurung tutup hilang pada posisi %1.",
    "A closing parenthesis is missing at position %1.",
  ),
  "logika.urai_operator_tanpa_operand": bi(
    "Operator muncul tanpa operand pada posisi %1.",
    "An operator appears without an operand at position %1.",
  ),
  "logika.urai_rumus_kosong": bi(
    "Rumusnya kosong.",
    "The formula is empty.",
  ),
  "logika.urai_rumus_terputus": bi(
    "Rumusnya berakhir lebih cepat daripada yang dituntut tata bahasanya, pada posisi %1.",
    "The formula ends sooner than the grammar requires, at position %1.",
  ),
  "logika.urai_sisa_masukan": bi(
    "Rumusnya sudah lengkap tetapi masih ada sisa masukan pada posisi %1.",
    "The formula is already complete, but input remains at position %1.",
  ),

  // Jembatan WebAssembly
  "mesin.aktivasi_tak_dikenal": bi(
    "Fungsi aktivasi tidak dikenal: %1.",
    "Unknown activation function: %1.",
  ),
  "mesin.batas_keputusan_dua_masukan": bi(
    "Batas keputusan hanya bisa digambar untuk jaringan berdua masukan.",
    "A decision boundary can only be drawn for a network with two inputs.",
  ),
  "mesin.dataset_tak_dikenal": bi(
    "Kumpulan data tidak dikenal: %1.",
    "Unknown dataset: %1.",
  ),
  "mesin.defuzzifikasi_tak_dikenal": bi(
    "Metode defuzzifikasi tidak dikenal: %1.",
    "Unknown defuzzification method: %1.",
  ),
  "mesin.inferensi_tak_dikenal": bi(
    "Mesin inferensi tidak dikenal: %1.",
    "Unknown inference engine: %1.",
  ),
  "mesin.jenis_agen_tak_dikenal": bi(
    "Jenis agen tidak dikenal: %1.",
    "Unknown agent kind: %1.",
  ),
  "mesin.json_tak_sah": bi(
    "Masukannya tidak bisa dibaca mesin: %1.",
    "The engine could not read the input: %1.",
  ),
  "mesin.operator_tak_dikenal": bi(
    "Operator tidak dikenal: %1.",
    "Unknown operator: %1.",
  ),
  "mesin.rentang_tak_sah": bi(
    "Rentang tidak sah: %1 sampai %2.",
    "Invalid range: %1 to %2.",
  ),
  "mesin.resolusi_di_luar_rentang": bi(
    "Resolusi harus antara %1 dan %2, diberi %3.",
    "The resolution must be between %1 and %2; %3 was given.",
  ),
  "mesin.serialisasi_gagal": bi(
    "Hasilnya gagal diserialisasi: %1.",
    "The result could not be serialised: %1.",
  ),

  // Sesi 12 & 13 — machine learning
  "ml.baris_tak_rata": bi(
    "Setiap baris harus punya %1 fitur, ditemukan %2.",
    "Every row must have %1 features; %2 were found.",
  ),
  "ml.belum_dilatih": bi(
    "Modelnya belum dilatih.",
    "The model has not been trained yet.",
  ),
  "ml.data_kosong": bi(
    "Kumpulan datanya kosong.",
    "The dataset is empty.",
  ),
  "ml.kelompok_terlalu_banyak": bi(
    "%1 kelompok diminta untuk %2 titik.",
    "%1 clusters were requested for %2 points.",
  ),
  "ml.nilai_bukan_bilangan": bi(
    "Nilai yang bukan bilangan pada baris %1 kolom %2.",
    "A non-numeric value at row %1, column %2.",
  ),
  "ml.panjang_tak_sepadan": bi(
    "%1 baris fitur tetapi %2 label.",
    "%1 feature rows but %2 labels.",
  ),
  "ml.parameter_tak_sah": bi(
    "Parameter %1 tidak sah: %2.",
    "Invalid parameter %1: %2.",
  ),

  // Sesi 11 — sistem pakar
  "pakar.aturan_tanpa_premis": bi(
    "Aturan %1 tidak punya satu pun premis.",
    "Rule %1 has no premises at all.",
  ),
  "pakar.basis_aturan_kosong": bi(
    "Basis aturannya kosong.",
    "The rule base is empty.",
  ),
  "pakar.batas_langkah": bi(
    "Melampaui %1 langkah penalaran.",
    "Exceeded %1 reasoning steps.",
  ),
  "pakar.keyakinan_di_luar_rentang": bi(
    "Certainty factor %1 berada di luar [-1, 1]: %2.",
    "The certainty factor of %1 lies outside [-1, 1]: %2.",
  ),
  "pakar.penalaran_melingkar": bi(
    "Penalaran melingkar: %1.",
    "Circular reasoning: %1.",
  ),

  // Sesi 14 — robotika
  "robot.di_luar_jangkauan": bi(
    "Jarak %1 berada di luar jangkauan lengan, yaitu %2 sampai %3.",
    "A distance of %1 lies outside the arm's reach of %2 to %3.",
  ),
  "robot.parameter_tak_sah": bi(
    "Parameter %1 tidak sah: %2.",
    "Invalid parameter %1: %2.",
  ),
  "robot.tidak_konvergen": bi(
    "Tidak mencapai tujuannya dalam %1 langkah.",
    "Did not reach the goal within %1 steps.",
  ),

  // Sesi 9 — jaringan syaraf
  "syaraf.arsitektur_tak_sah": bi(
    "Arsitektur jaringan tidak sah: %1.",
    "Invalid network architecture: %1.",
  ),
  "syaraf.data_kosong": bi(
    "Kumpulan datanya kosong.",
    "The dataset is empty.",
  ),
  "syaraf.data_tak_sepadan": bi(
    "%1 baris masukan tetapi %2 baris target.",
    "%1 input rows but %2 target rows.",
  ),
  "syaraf.laju_belajar": bi(
    "Laju belajar harus positif dan berhingga, diberi %1.",
    "The learning rate must be positive and finite; %1 was given.",
  ),
  "syaraf.masukan_tak_sepadan": bi(
    "Masukan harus %1 nilai, diberi %2.",
    "The input must have %1 values; %2 were given.",
  ),
  "syaraf.menyimpang": bi(
    "Pelatihannya menyimpang pada epoch %1.",
    "Training diverged at epoch %1.",
  ),
  "syaraf.target_tak_sepadan": bi(
    "Target harus %1 nilai, diberi %2.",
    "The target must have %1 values; %2 were given.",
  ),
};

/** Penanda bernomor di dalam sebuah kalimat, mis. `%1`. */
export const PENANDA = /%([1-9])/g;

/**
 * Kalimat untuk sebuah galat mesin, dalam bahasa yang sedang aktif.
 *
 * Kode yang tidak dikenal tetap menghasilkan sesuatu yang bisa dilaporkan —
 * kodenya sendiri beserta nilainya — bukan untai kosong. Kegagalan yang tampil
 * sebagai ruang kosong adalah kegagalan yang tidak bisa ditelusuri siapa pun.
 */
export function kalimatGalat(galat: GalatMesin): string {
  const pasangan = PESAN[galat.kode];
  if (pasangan === undefined) {
    const nilai = galat.arg.length > 0 ? `: ${galat.arg.join(", ")}` : "";
    return `${galat.kode}${nilai}`;
  }
  return pick(pasangan).replace(PENANDA, (utuh, nomor: string) => {
    const nilai = galat.arg[Number(nomor) - 1];
    return nilai === undefined ? utuh : nilai;
  });
}

/**
 * Membaca bentuk galat dari amplop mesin.
 *
 * Mengembalikan `null` untuk apa pun yang bukan bentuk itu, supaya pemanggilnya
 * bisa memilih kalimat cadangannya sendiri alih-alih menabrak.
 */
export function bacaGalat(nilai: unknown): GalatMesin | null {
  if (typeof nilai !== "object" || nilai === null) return null;
  const kode = (nilai as { kode?: unknown }).kode;
  const arg = (nilai as { arg?: unknown }).arg;
  if (typeof kode !== "string") return null;
  if (!Array.isArray(arg) || !arg.every((a) => typeof a === "string")) return null;
  return { kode, arg };
}
