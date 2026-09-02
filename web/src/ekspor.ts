/**
 * Menyusun isi sebuah laboratorium menjadi berkas yang bisa disimpan.
 *
 * # Kenapa isinya dibaca dari DOM, bukan diserahkan tiap lab
 *
 * Karena keempat belas lab membangun tampilannya sendiri-sendiri dan tidak
 * punya satu bentuk hasil bersama. Menambahkan ekspor lab demi lab berarti
 * empat belas potong kode yang akan menyimpang: yang satu lupa menyertakan
 * langkah, yang lain lupa menyertakan masukan, dan tidak seorang pun akan
 * menyadarinya sampai ada yang mengunduh lab yang jarang dibuka.
 *
 * Yang dibaca bukan nama kelas CSS melainkan atribut `data-ekspor` yang
 * dipasang komponen bersama di `ui.ts`. Bedanya penting: nama kelas ada untuk
 * penataan dan siapa pun berhak menggantinya demi tampilan, sementara
 * `data-ekspor` ada khusus sebagai kontrak. Sebuah lab yang memakai komponen
 * bersama karena itu bisa diekspor tanpa satu baris pun tambahan — dan lab
 * yang tidak memakainya akan terlihat kosong di berkasnya, yang merupakan
 * kabar yang benar.
 *
 * # Kenapa CSV dan bukan XLSX
 *
 * XLSX adalah arsip ZIP berisi beberapa berkas XML; menyusunnya di peramban
 * menuntut pustaka berukuran ratusan kilobyte, beberapa kali lipat modul
 * WebAssembly seluruh proyek ini. CSV dibuka Excel, Google Sheets, dan pandas
 * tanpa satu bita pun tambahan.
 *
 * Tiga hal yang wajib benar agar berkasnya sungguh terbuka rapi di Excel, dan
 * ketiganya sering terlewat: BOM UTF-8 di depan (tanpa itu Excel di Windows
 * membacanya sebagai ANSI), baris `sep=,` paling atas (Excel berwilayah
 * Indonesia memakai titik koma sebagai pemisah kolom), dan akhir baris CRLF
 * seperti dituntut RFC 4180.
 *
 * .Deckyx
 */

import { bi, pick, type Bilingual } from "./i18n";

const CR = String.fromCharCode(13);
const LF = String.fromCharCode(10);
const AKHIR_BARIS = CR + LF;
const BOM = "﻿";

export type Cell = string | number | null | undefined;

/** Satu sel CSV, dikutip bila perlu. */
export function cell(value: Cell): string {
  if (value === null || value === undefined) return "";
  const text = typeof value === "number" ? String(value) : value;
  if (/[",\r\n]/.test(text)) return `"${text.replace(/"/g, '""')}"`;
  return text;
}

/** Menyusun baris menjadi teks CSV yang siap diunduh. */
export function csv(rows: Cell[][]): string {
  const out = [`${BOM}sep=,`];
  for (const r of rows) out.push(r.map(cell).join(","));
  return out.join(AKHIR_BARIS) + AKHIR_BARIS;
}

/** Satu bagian laporan, sepadan dengan satu kartu di halaman. */
export interface Bagian {
  judul: string;
  masukan: { label: string; nilai: string }[];
  pilihan: { label: string; terpilih: string }[];
  hasil: { label: string; nilai: string }[];
  langkah: { label: string; rumus: string }[];
  tabel: { kepala: string[]; baris: string[][] }[];
}

/** Isi sebuah lab, seperti yang terbaca dari tampilannya. */
export interface IsiLab {
  bagian: Bagian[];
}

function teks(node: Element | null): string {
  return (node?.textContent ?? "").replace(/\s+/g, " ").trim();
}

/**
 * Isian teks bebas yang layak masuk laporan.
 *
 * Inilah data yang dibawa sendiri oleh penggunanya — kalimat yang diurai,
 * rumus logika yang dibuktikan, korpus yang ditimbang — dan justru itulah yang
 * paling ingin ia simpan. Dibaca tanpa perlu ditandai satu per satu, karena
 * menuntut tiap lab menandainya berarti menunggu satu lab lupa, lalu tidak ada
 * yang menyadari bahwa laporannya kehilangan satu-satunya bagian yang bukan
 * bawaan.
 *
 * Penggeser tidak ikut lewat jalur ini karena sudah tertangkap sebagai
 * `data-ekspor="masukan"`; tombol dan isian tersembunyi bukan data yang
 * diketik siapa pun.
 */
const BEBAS =
  "textarea, input:not([type=range]):not([type=button]):not([type=submit]):not([type=hidden])";

/** Nama sebuah isian, dicari dari yang paling tepat ke yang paling umum. */
function labelIsian(n: Element): string {
  const aria = n.getAttribute("aria-label");
  if (aria) return aria;
  const id = n.getAttribute("id");
  if (id) {
    const l = n.ownerDocument.querySelector(`label[for="${CSS.escape(id)}"]`);
    if (l) return teks(l);
  }
  return teks(n.closest("label")?.querySelector(".field__label") ?? null);
}

/**
 * Kartu terdekat yang memuat sebuah simpul, atau `null` bila di luar kartu.
 *
 * Kartu bisa bersarang — beberapa lab menaruh kartu hasil di dalam kartu
 * setelan. Tanpa pemilik terdekat, isi kartu dalam akan tercatat dua kali:
 * sekali di bawah judulnya sendiri dan sekali lagi di bawah judul induknya.
 */
function pemilik(node: Element): Element | null {
  return node.closest('[data-ekspor="kartu"]');
}

/** Bagian kosong yang siap diisi. */
function bagianKosong(judul: string): Bagian {
  return { judul, masukan: [], pilihan: [], hasil: [], langkah: [], tabel: [] };
}

/**
 * Membaca isi sebuah lab dari tampilannya yang sedang tergambar.
 *
 * Disusun per kartu dan mengikuti urutan halaman. Sebuah CSV berisi lima tabel
 * tanpa judul tidak memberi satu pun petunjuk tabel mana yang mana — padahal
 * yang mengunduhnya justru orang yang ingin membacanya lagi besok.
 *
 * Simpul yang berada di luar kartu mana pun tidak dibuang, melainkan
 * dikumpulkan ke satu bagian tanpa judul di akhir. Membuangnya berarti sebuah
 * lab yang tidak memakai `card()` akan menghasilkan berkas kosong tanpa
 * memberi tanda apa pun bahwa ada yang hilang.
 */
export function bacaIsi(root: ParentNode): IsiLab {
  const urut = new Map<Element | null, Bagian>();

  // Kartu didaftarkan lebih dulu supaya urutan bagian mengikuti urutan
  // halaman, bukan urutan kemunculan isi pertamanya.
  for (const k of root.querySelectorAll('[data-ekspor="kartu"]')) {
    urut.set(k, bagianKosong(k.getAttribute("data-judul") ?? ""));
  }
  const bagianUntuk = (node: Element): Bagian => {
    const kunci = pemilik(node);
    let b = urut.get(kunci);
    if (!b) {
      b = bagianKosong("");
      urut.set(kunci, b);
    }
    return b;
  };

  // Masukan ditelusuri dalam satu sapuan supaya urutannya di laporan sama
  // dengan urutannya di layar. Dua sapuan terpisah — penggeser dulu, lalu
  // isian teks — membuat nama sebuah bukti tercatat di bawah angkanya sendiri,
  // dan pembacanya harus menebak ke atas.
  for (const n of root.querySelectorAll(`[data-ekspor="masukan"], ${BEBAS}`)) {
    if (n.matches('[data-ekspor="masukan"]')) {
      const isian = n.querySelector("input");
      bagianUntuk(n).masukan.push({
        label: n.getAttribute("data-label") ?? "",
        // Dibaca dari isian saat ini, bukan dari atribut yang disalin waktu
        // digambar: salinan itu basi begitu penggesernya digerakkan.
        nilai: isian?.value ?? teks(n.querySelector(".field__value")),
      });
      continue;
    }
    // Isian yang berada di dalam komponen bertanda sudah terbaca di atas.
    if (n.closest('[data-ekspor="masukan"]')) continue;
    const nilai = (n as HTMLInputElement | HTMLTextAreaElement).value.trim();
    // Kotak yang belum diisi bukan masukan.
    if (nilai === "") continue;
    bagianUntuk(n).masukan.push({ label: labelIsian(n), nilai });
  }

  for (const n of root.querySelectorAll('[data-ekspor="pilihan"]')) {
    const terpilih = teks(n.querySelector('button[aria-pressed="true"]'));
    if (terpilih === "") continue;
    const b = bagianUntuk(n);
    b.pilihan.push({ label: b.judul, terpilih });
  }

  for (const n of root.querySelectorAll('[data-ekspor="hasil"]')) {
    bagianUntuk(n).hasil.push({
      label: n.getAttribute("data-label") ?? "",
      nilai: n.getAttribute("data-nilai") ?? teks(n.querySelector(".readout")),
    });
  }

  for (const n of root.querySelectorAll('[data-ekspor="langkah"] > li')) {
    bagianUntuk(n).langkah.push({
      label: n.getAttribute("data-label") ?? "",
      rumus: n.getAttribute("data-rumus") ?? teks(n),
    });
  }

  for (const t of root.querySelectorAll("table")) {
    bagianUntuk(t).tabel.push({
      kepala: [...t.querySelectorAll("thead th")].map(teks),
      baris: [...t.querySelectorAll("tbody tr")].map((tr) =>
        [...tr.querySelectorAll("td")].map(teks),
      ),
    });
  }

  const bagian = [...urut.values()].filter(
    (b) =>
      b.masukan.length > 0 ||
      b.pilihan.length > 0 ||
      b.hasil.length > 0 ||
      b.langkah.length > 0 ||
      b.tabel.some((t) => t.baris.length > 0),
  );
  return { bagian };
}

/** Label berkas dalam dua bahasa. */
export const LABEL = {
  title: bi("AI ATLAS — laporan laboratorium", "AI ATLAS — laboratory report"),
  generated: bi("dihasilkan", "generated by"),
  lab: bi("laboratorium", "laboratory"),
  session: bi("sesi kuliah", "course session"),
  inputs: bi("MASUKAN", "INPUTS"),
  results: bi("HASIL", "RESULTS"),
  steps: bi("LANGKAH PERHITUNGAN", "CALCULATION STEPS"),
  tables: bi("TABEL", "TABLES"),
  chosen: bi("dipilih", "chosen"),
  other: bi("LAIN-LAIN", "OTHER"),
  no: bi("no", "no"),
  step: bi("langkah", "step"),
  formula: bi("rumus", "formula"),
  notes: bi("CATATAN", "NOTES"),
  noteEngine: bi(
    "Seluruh perhitungan dilakukan di peramban Anda oleh mesin Rust yang " +
      "dikompilasi ke WebAssembly, dan sudah diadu terhadap 3.796 pernyataan " +
      "berpola bit lawan implementasi Go dan Oracle PL/SQL.",
    "Every calculation runs in your browser in Rust compiled to WebAssembly, " +
      "checked against 3,796 bit-pattern assertions versus the Go and Oracle " +
      "PL/SQL implementations.",
  ),
  noteDecimal: bi(
    "Angka memakai titik sebagai pemisah desimal. Excel berwilayah Indonesia " +
      "mungkin membacanya sebagai teks.",
    "Numbers use a full stop as the decimal separator.",
  ),
  download: bi("Unduh CSV (Excel)", "Download CSV (Excel)"),
  print: bi("Cetak / simpan PDF", "Print / save as PDF"),
  downloaded: bi("Diunduh", "Downloaded"),
  refused: bi("Peramban menolak unduhan", "The browser refused the download"),
  hint: bi(
    "Unduh seluruh isinya — masukan, hasil, langkah perhitungan, dan tabelnya " +
      "— sebagai satu berkas yang bisa dibuka Excel, atau cetak halamannya " +
      "menjadi PDF.",
    "Download everything on this page — inputs, results, calculation steps, " +
      "and tables — as one file Excel can open, or print the page to PDF.",
  ),
} as const;

/** Seluruh isi laporan sebagai baris CSV. */
export function reportRows(
  namaLab: string,
  sesi: number | undefined,
  isi: IsiLab,
): Cell[][] {
  const t = (p: Bilingual) => pick(p);
  const rows: Cell[][] = [];

  rows.push([t(LABEL.title)]);
  rows.push([t(LABEL.generated), "xyb3rpunq.github.io/ai-atlas (.Deckyx)"]);
  rows.push([t(LABEL.lab), namaLab]);
  if (sesi !== undefined) rows.push([t(LABEL.session), sesi]);
  rows.push([]);

  for (const b of isi.bagian) {
    rows.push([b.judul || t(LABEL.other)]);

    for (const p of b.pilihan) {
      if (p.terpilih) rows.push([t(LABEL.chosen), p.terpilih]);
    }
    for (const m of b.masukan) rows.push([m.label, m.nilai]);
    for (const h of b.hasil) rows.push([h.label, h.nilai]);

    if (b.langkah.length > 0) {
      rows.push([t(LABEL.no), t(LABEL.step), t(LABEL.formula)]);
      b.langkah.forEach((s, i) => rows.push([i + 1, s.label, s.rumus]));
    }

    for (const tab of b.tabel) {
      if (tab.kepala.length === 0 && tab.baris.length === 0) continue;
      rows.push(tab.kepala);
      for (const r of tab.baris) rows.push(r);
    }

    rows.push([]);
  }

  rows.push([t(LABEL.notes)]);
  rows.push([t(LABEL.noteEngine)]);
  rows.push([t(LABEL.noteDecimal)]);
  return rows;
}

/** Nama berkas yang menyebutkan labnya, bukan sekadar tanggalnya. */
export function fileName(slug: string, ext = "csv"): string {
  const now = new Date();
  const two = (n: number) => String(n).padStart(2, "0");
  const stamp = `${now.getFullYear()}${two(now.getMonth() + 1)}${two(now.getDate())}-${two(now.getHours())}${two(now.getMinutes())}`;
  const bersih = slug.replace(/[^0-9A-Za-z-]/g, "") || "lab";
  return `ai-atlas-${bersih}-${stamp}.${ext}`;
}

/**
 * Menyerahkan sebuah berkas kepada pengguna.
 *
 * Objek URL, bukan `data:` URI. Yang terakhir dibatasi panjang di sebagian
 * peramban, dan laporan dengan tabel besar melewati batas itu tanpa memberi
 * tanda apa pun — unduhannya sekadar tidak terjadi.
 */
export function download(name: string, text: string, mime: string): void {
  const blob = new Blob([text], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  document.body.append(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}
