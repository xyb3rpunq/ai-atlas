/**
 * Perakit kalimat aturan JIKA–MAKA, dalam bahasa pembacanya.
 *
 * # Kenapa di sini, bukan di mesinnya
 *
 * Karena mesinnya tidak tahu — dan tidak seharusnya tahu — siapa yang sedang
 * membaca. Selama kalimatnya dirakit di Rust, ia akan selalu berbunyi
 * "JIKA … DAN … MAKA …", termasuk di halaman yang seluruh sisanya berbahasa
 * Inggris. Bocoran seperti itu tidak bisa diperbaiki dari sisi antarmuka:
 * yang diterima sudah berupa kalimat jadi.
 *
 * Jadi mesinnya sekarang mengembalikan **bentuk** aturannya — premis,
 * penghubung, kesimpulan — dan kalimatnya dirakit di sini. Akibat yang
 * diinginkan: kedua bahasa berasal dari satu sumber yang sama, dan menambah
 * bahasa ketiga tidak menyentuh Rust sama sekali.
 *
 * # Kenapa nama faktanya lewat kamus terpisah
 *
 * Karena di dalam mesin, nama fakta adalah **kunci**: "demam" menghubungkan
 * premis satu aturan dengan kesimpulan aturan lain. Menerjemahkannya di dalam
 * basis pengetahuan akan memutus hubungan itu — aturan yang menyimpulkan
 * "fever" tidak akan pernah menyalakan aturan yang menuntut "demam".
 *
 * Karena itu kuncinya tetap, dan nama yang dibaca manusia diambil dari kamus
 * saat menampilkannya. Nama yang belum punya terjemahan tampil apa adanya —
 * itu memang yang diinginkan untuk fakta yang diketik sendiri oleh pengguna.
 *
 * .Deckyx
 */

import type * as engine from "./engine.js";
import { bi, pick } from "./i18n.js";
import type { Bilingual } from "./i18n.js";

/** Kata penghubung aturan. Satu-satunya bagian kalimat yang bukan nama. */
const K = {
  jika: bi("JIKA", "IF"),
  maka: bi("MAKA", "THEN"),
  dan: bi("DAN", "AND"),
  atau: bi("ATAU", "OR"),
  bukan: bi("BUKAN", "NOT"),
} as const;

/** Kamus nama yang dibaca manusia: kunci mesin → sepasang teks. */
export type KamusNama = Readonly<Record<string, Bilingual>>;

/**
 * Nama sebuah kunci, atau kuncinya sendiri bila belum ada di kamus.
 *
 * Mengembalikan kuncinya alih-alih untai kosong: fakta yang diketik sendiri
 * oleh pengguna memang tidak akan pernah ada di kamus mana pun, dan yang benar
 * untuk fakta seperti itu adalah menampilkannya persis seperti yang diketik.
 */
export function nama(kamus: KamusNama, kunci: string): string {
  const pasangan = kamus[kunci];
  return pasangan === undefined ? kunci : pick(pasangan);
}

/** Kata sambung yang sesuai dengan penghubung aturan. */
function sambung(penghubung: "AND" | "OR"): string {
  return ` ${pick(penghubung === "AND" ? K.dan : K.atau)} `;
}

/**
 * Kalimat sebuah aturan kabur: `JIKA a = A ATAU b = B MAKA c = C`.
 *
 * Bentuk `variabel = himpunan` dipertahankan di kedua bahasa. Ia bukan
 * kalimat, melainkan notasi — dan notasi yang berubah bentuk antarbahasa
 * memutus hubungannya dengan gambar kurva keanggotaan di sebelahnya.
 */
export function kalimatAturanKabur(jejak: engine.RuleTrace, kamus: KamusNama = {}): string {
  const premis = jejak.antecedents
    .map((a) => `${nama(kamus, a.variable)} = ${nama(kamus, a.set)}`)
    .join(sambung(jejak.connective));
  const keluaran = `${nama(kamus, jejak.output)} = ${nama(kamus, jejak.consequent_set)}`;
  return `${pick(K.jika)} ${premis} ${pick(K.maka)} ${keluaran}`;
}

/**
 * Kalimat sebuah aturan pakar: `JIKA a DAN BUKAN b MAKA c`.
 *
 * Premis yang dinegasikan diawali "BUKAN"/"NOT". Menghilangkannya akan
 * membuat aturan "pilek tanpa demam berarti alergi" terbaca sebagai "pilek dan
 * demam berarti alergi" — kebalikannya, dan tetap masuk akal dibaca.
 */
export function kalimatAturanPakar(
  langkah: { premises: engine.ExpertPremise[]; connective: "AND" | "OR"; conclusion: string },
  kamus: KamusNama = {},
): string {
  const premis = langkah.premises
    .map((p) => {
      const teks = nama(kamus, p.fact);
      return p.expected ? teks : `${pick(K.bukan)} ${teks}`;
    })
    .join(sambung(langkah.connective));
  return `${pick(K.jika)} ${premis} ${pick(K.maka)} ${nama(kamus, langkah.conclusion)}`;
}

/**
 * Jawaban atas "kenapa aturan ini ada".
 *
 * Dirakit dari basis pengetahuan yang memang sudah dipegang sisi antarmuka.
 * Sebelumnya ini satu panggilan lagi ke mesin, yang mengembalikan kalimat jadi
 * berbahasa Indonesia — perjalanan bolak-balik hanya untuk merangkai kata dari
 * data yang tidak pernah meninggalkan halaman ini.
 */
export function kenapaAturan(
  kb: engine.KnowledgeBase,
  ruleId: string,
  kamus: KamusNama = {},
  alasan: Readonly<Record<string, Bilingual>> = {},
): string {
  const aturan = kb.rules.find((r) => r.id === ruleId);
  if (aturan === undefined) return "";
  const kalimat = kalimatAturanPakar(aturan, kamus);
  const pola = bi(
    "%A dipakai untuk menyimpulkan %K.",
    "%A is used to conclude %K.",
  );
  const inti = pick(pola)
    .replace("%A", kalimat)
    .replace("%K", nama(kamus, aturan.conclusion));
  const tambahan = alasan[ruleId];
  return tambahan === undefined ? inti : `${inti} ${pick(tambahan)}`;
}

/**
 * Jawaban atas "bagaimana kesimpulan ini diperoleh", satu baris per langkah.
 *
 * Angkanya dibiarkan apa adanya di sini; pemanggilnya yang memformat, karena
 * ia yang tahu berapa desimal yang pantas untuk kolomnya.
 */
export function bagaimanaKesimpulan(
  langkah: engine.ExpertStep[],
  fakta: string,
  format: (n: number) => string,
  kamus: KamusNama = {},
): string[] {
  const pola = bi(
    "Langkah %N: %R [%D] menghasilkan %K dengan keyakinan %C",
    "Step %N: %R [%D] yields %K with certainty %C",
  );
  return langkah
    .filter((s) => s.conclusion === fakta)
    .map((s) => {
      const dukungan = s.support
        .map(([f, cf]) => `${nama(kamus, f)} (${format(cf)})`)
        .join(", ");
      return pick(pola)
        .replace("%N", String(s.order))
        .replace("%R", s.rule_id)
        .replace("%D", dukungan)
        .replace("%K", nama(kamus, s.conclusion))
        .replace("%C", format(s.conclusion_certainty));
    });
}
