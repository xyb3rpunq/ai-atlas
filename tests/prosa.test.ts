/**
 * Uji yang membaca kode antarmukanya sendiri dan menolak prosa tanpa pasangan.
 *
 * # Kenapa memeriksa kamus dan data saja tidak cukup
 *
 * Uji yang ada memeriksa isi `T` dan isi `NOTES`: tiap pasangan punya kedua
 * bahasa, keduanya berbeda. Seluruhnya hijau — dan seluruhnya tetap hijau
 * sementara tautan lompat di `index.html` bertuliskan "Lompat ke laboratorium"
 * apa adanya, dan 42 nama rumus di `notes.ts` tidak punya sisi Inggris sama
 * sekali. Keduanya nyata, keduanya lolos setiap uji sebelum ini, dan keduanya
 * baru terlihat setelah halamannya dibuka dan dibaca.
 *
 * Uji `render.test.ts` memasang tiap laboratorium dan membaca layarnya, tetapi
 * ia memasangnya ke wadah lepas — kerangka aplikasi, catatan, kepala, dan kaki
 * halaman tidak pernah ikut. Yang di sini menutup celah itu dari arah kode:
 * setiap untai berbahasa Indonesia di seluruh `web/src` harus berupa argumen
 * pertama sebuah `bi(...)`, yaitu harus punya pasangan Inggris di baris yang
 * sama.
 *
 * .Deckyx
 */

import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import { NOTES } from "../web/src/labs/notes.js";

/**
 * Kata Indonesia yang tidak mungkin muncul di kode.
 *
 * Sengaja kata fungsi — kata sambung, kata depan, kata ganti. Kata benda
 * seperti "data" dan "model" dipakai sebagai nama peubah di seluruh berkas
 * ini, dan memasukkannya akan membuat ujinya berteriak pada kode yang benar.
 */
const KATA = new RegExp(
  String.raw`\b(yang|dengan|adalah|tidak|untuk|dari|pada|karena|jadi|bisa|akan|sebuah|supaya|sehingga|kalau|tetapi|atau|dan|ini|itu|lebih|sudah|masih|hanya|setiap|tiap|bila|juga|belum|harus|lompat)\b`,
  "i",
);

/**
 * Tetapan yang isinya memang bahan berbahasa Indonesia, bukan teks antarmuka.
 *
 * Hanya satu awalan nama, dan hanya di laboratorium pengolahan bahasa: bahan
 * yang dibedah di sana adalah morfologi Bahasa Indonesia, dan menjalankan
 * pengupas imbuhan "me-", "di-", "-kan" pada kalimat Inggris tidak
 * mengajarkan apa pun. Alasannya juga dinyatakan di layar, dan wadahnya
 * ditandai `data-korpus` supaya `render.test.ts` tahu bagian mana yang
 * dikecualikan.
 */
const AWALAN_KORPUS = "KORPUS_";

/**
 * Tetapan yang isinya kunci mesin, bukan teks yang ditampilkan.
 *
 * Nama relasi seperti `"adalah"` menghubungkan premis satu simpul dengan
 * kesimpulan simpul lain; menerjemahkannya akan memutus penalarannya.
 * Menaruhnya di tetapan bernama membuat niatnya terbaca — dan membuat salah
 * ketik menjadi galat, bukan pewarisan yang diam-diam berhenti bekerja.
 */
const AWALAN_KUNCI = "KUNCI_";

/** Berkas sumber antarmuka, tanpa data yang sudah diuji tersendiri. */
function berkasSumber(): string[] {
  const keluar: string[] = [];
  for (const dir of [join("web", "src"), join("web", "src", "labs")]) {
    for (const f of readdirSync(dir, { withFileTypes: true })) {
      if (f.isFile() && f.name.endsWith(".ts")) keluar.push(join(dir, f.name));
    }
  }
  return keluar;
}

/** Sisipan di dalam untai templat. */
const SISIPAN = new RegExp(String.raw`\$\{[^}]*\}`, "g");

/**
 * Awalan yang menandakan untainya memang punya pasangan Inggris.
 *
 * `bi(` untuk potongan pertamanya. `+`, `?`, dan `:` untuk lanjutannya:
 * kalimat panjang ditulis sebagai beberapa untai yang disambung, dan kalimat
 * yang berubah menurut keadaan ditulis sebagai percabangan di dalam `bi(...)`.
 * Hanya potongan pertama yang didahului `bi(`.
 */
const BERPASANGAN = new RegExp(String.raw`(\bbi\(|[+?:])\s*$`);

/**
 * Untai literal regex, dikenali dari apa yang mendahuluinya.
 *
 * Sebuah `/` yang menyusul `(`, `=`, `,`, `:` dan sejenisnya memulai regex,
 * bukan pembagian. Regex seperti `/[",` + "`" + `]/` memuat petik ganda, dan tanpa
 * membuangnya lebih dulu petik itu akan dikira pembuka untai baru — seluruh
 * sisa berkas lalu terbaca sebagai satu untai raksasa, dan bocoran yang
 * sesungguhnya tertelan di dalamnya tanpa jejak.
 */
const REGEX_LITERAL = new RegExp(
  String.raw`([(,=:[!&|?{};]\s*)/[^/\n]*/[gimsuy]*`,
  "g",
);

/**
 * Membuang komentar dan literal regex.
 *
 * Komentar dibuang karena berkas ini memang berkomentar panjang dalam Bahasa
 * Indonesia, dan itu disengaja: yang membacanya pengembang, bukan pengunjung.
 */
function tanpaKomentar(isi: string): string {
  return isi
    .replace(/\/\*[\s\S]*?\*\//g, "")
    .replace(/^\s*\/\/.*$/gm, "")
    .replace(REGEX_LITERAL, "$1/RE/");
}

/**
 * Untai berbahasa Indonesia yang tidak menjadi argumen pertama sebuah `bi()`.
 *
 * Bentuknya selalu `bi("…", "…")` di seluruh proyek ini, jadi sisi Indonesia
 * selalu didahului `bi(` — dengan spasi atau baris baru di antaranya.
 */
function prosaTanpaPasangan(isi: string): string[] {
  const bersih = tanpaKomentar(isi);
  const keluar: string[] = [];
  // Ketiga bentuk petik ikut, supaya pemasangannya tidak tergeser. Untai
  // berpetik tunggal seperti `'""'` di dalam pengganti CSV memuat petik
  // ganda; tanpa memasangkannya lebih dulu, petik itu akan dikira pembuka
  // untai baru dan seluruh sisa berkas terbaca sebagai satu untai raksasa.
  // Yang berpetik tunggal dibatasi satu baris: tanda kutip pada "Bayes'"
  // di dalam prosa tidak boleh menelan apa pun.
  const untai = /"[^"]*"|'[^'\n]*'|`[^`]*`/g;
  for (const c of bersih.matchAll(untai)) {
    // Petiknya dibuang; polanya tidak lagi punya grup tangkap.
    const teks = c[0].slice(1, -1);
    // Sisipan `${...}` dibuang lebih dulu. Nama peubah seperti `jugA` cocok
    // dengan kata "juga" tanpa peduli besar-kecil huruf, dan menuduh nama
    // peubah sebagai prosa membuat ujinya berteriak pada kode yang benar.
    const isi = teks.replace(SISIPAN, " ");
    if (!KATA.test(isi)) continue;
    const sebelum = bersih.slice(Math.max(0, c.index - 40), c.index);
    // Argumen pertama sebuah `bi(...)`, atau lanjutan sebuah rangkaian:
    // kalimat panjang ditulis sebagai beberapa untai yang disambung `+`, dan
    // hanya potongan pertamanya yang didahului `bi(`.
    if (BERPASANGAN.test(sebelum)) continue;
    // Bahan yang memang berbahasa Indonesia dengan sengaja, dikenali dari
    // nama tetapan yang menampungnya.
    const tetapan = bersih.slice(0, c.index).lastIndexOf("const ");
    if (tetapan !== -1) {
      const nama = bersih.slice(tetapan + 6, tetapan + 6 + 8);
      const dikecualikan =
        nama.startsWith(AWALAN_KORPUS) || nama.startsWith(AWALAN_KUNCI);
      if (dikecualikan) {
        const larik = bersih.indexOf("];", tetapan);
        const tunggal = bersih.indexOf(";", tetapan);
        const akhir = larik === -1 ? tunggal : Math.min(larik + 1, tunggal);
        if (akhir === -1 || c.index < akhir) continue;
      }
    }
    // Yang dicari prosa, bukan penanda. Potongan seperti `)) {` tidak punya
    // satu pun kata, dan hanya muncul kalau pemasangan petiknya tergeser.
    if ((isi.match(/[A-Za-z]{3,}/g) ?? []).length < 2) continue;
    keluar.push(teks.slice(0, 70));
  }
  return keluar;
}

describe("prosa antarmuka selalu berpasangan", () => {
  it("polanya sendiri bisa gagal", () => {
    // Pemeriksaan yang tidak bisa gagal adalah kegagalan yang paling mahal,
    // karena ia terlihat persis seperti jaminan.
    expect(prosaTanpaPasangan('const x = "Lompat ke laboratorium";')).toEqual([
      "Lompat ke laboratorium",
    ]);
    expect(prosaTanpaPasangan('bi("Lompat ke laboratorium", "Skip")')).toEqual([]);
    expect(prosaTanpaPasangan('const c = "formula__name";')).toEqual([]);
    // Tanpa batas kata, "dan" akan cocok di dalam "standar".
    expect(prosaTanpaPasangan('const s = "standar deviasi";')).toEqual([]);
  });

  it("tidak ada untai Indonesia tanpa pasangan di web/src", () => {
    const pelanggaran: string[] = [];
    for (const berkas of berkasSumber()) {
      for (const teks of prosaTanpaPasangan(readFileSync(berkas, "utf8"))) {
        pelanggaran.push(`${berkas}: ${teks}`);
      }
    }
    expect(pelanggaran, pelanggaran.join("\n")).toEqual([]);
  });

  it("pemindainya memang membaca berkas yang cukup", () => {
    // Pemindai yang tidak membuka apa pun membuat uji di atas hijau karena
    // tidak memeriksa apa-apa, bukan karena kodenya bersih.
    expect(berkasSumber().length).toBeGreaterThan(15);
  });
});

describe("catatan laboratorium", () => {
  it("sisi Inggrisnya benar-benar berbahasa Inggris", () => {
    // Menyalin kolom Indonesia ke kolom Inggris lolos setiap pemeriksaan
    // panjang dan pemeriksaan "keduanya berbeda" yang ada — cukup dengan
    // mengubah satu kata.
    const pelanggaran: string[] = [];
    for (const [slug, notes] of Object.entries(NOTES)) {
      const periksa = (teks: string, di: string) => {
        if (KATA.test(teks)) pelanggaran.push(`${slug} ${di}: ${teks.slice(0, 60)}`);
      };
      periksa(notes.summary.en, "summary");
      for (const d of notes.definitions) {
        periksa(d.term.en, "term");
        periksa(d.meaning.en, `meaning ${d.term.id}`);
      }
      for (const f of notes.formulas) {
        periksa(f.name.en, "formula");
        periksa(f.note.en, `note ${f.name.id}`);
        // Rumusnya tidak pernah diterjemahkan, jadi ia diperiksa di kedua
        // arah: lambang yang sama di kedua bahasa, bukan kalimat pendek dalam
        // samaran rumus.
        periksa(f.expression, `expression ${f.name.id}`);
      }
      for (const p of notes.pitfalls) periksa(p.en, "pitfall");
    }
    expect(pelanggaran, pelanggaran.join("\n")).toEqual([]);
  });
});
