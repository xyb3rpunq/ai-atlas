/**
 * Dwibahasa Indonesia dan Inggris.
 *
 * Kamusnya sengaja berupa objek biasa, bukan berkas terpisah yang diambil saat
 * berjalan: seluruh teks masuk ke dalam bundel sehingga pergantian bahasa
 * terjadi seketika tanpa permintaan jaringan tambahan.
 */

/** Bahasa yang didukung. */
export type Lang = "id" | "en";

/** Sepasang teks untuk kedua bahasa. */
export type Bilingual = Readonly<Record<Lang, string>>;

const KEY = "ai-atlas:lang";

/** Bahasa yang sedang aktif. */
let current: Lang = "id";

/** Pelanggan yang ingin tahu saat bahasa berganti. */
const listeners = new Set<(lang: Lang) => void>();

/** Membaca preferensi bahasa yang tersimpan, jatuh ke bawaan bila tidak ada. */
export function restoreLang(): Lang {
  try {
    const saved = localStorage.getItem(KEY);
    if (saved === "id" || saved === "en") current = saved;
  } catch {
    /* Penyimpanan bisa diblokir; bahasa bawaan tetap dipakai. */
  }
  document.documentElement.lang = current;
  return current;
}

/** Bahasa aktif saat ini. */
export function lang(): Lang {
  return current;
}

/** Mengganti bahasa aktif dan memberi tahu seluruh pelanggan. */
export function setLang(next: Lang): void {
  if (next === current) return;
  current = next;
  document.documentElement.lang = next;
  try {
    localStorage.setItem(KEY, next);
  } catch {
    /* Preferensi tidak tersimpan, tetapi sesi ini tetap berganti bahasa. */
  }
  for (const fn of listeners) fn(next);
}

/** Mendaftar untuk diberi tahu saat bahasa berganti. Mengembalikan pembatal. */
export function onLangChange(fn: (lang: Lang) => void): () => void {
  listeners.add(fn);
  return () => listeners.delete(fn);
}

/** Memilih teks sesuai bahasa aktif. */
export function pick(pair: Bilingual): string {
  return pair[current];
}

/** Membuat pasangan teks dwibahasa. */
export function bi(id: string, en: string): Bilingual {
  return { id, en };
}

/** Kamus teks antarmuka. */
export const T = {
  tagline: bi(
    "Laboratorium Kecerdasan Buatan Klasik",
    "A Laboratory of Classical Artificial Intelligence",
  ),
  subtitle: bi(
    "Empat belas algoritma, ditulis dari nol dengan Rust, dijalankan di peramban Anda.",
    "Fourteen algorithms, written from scratch in Rust, running in your browser.",
  ),
  labs: bi("Laboratorium", "Laboratories"),
  soon: bi("Segera", "Soon"),
  lintasBahasa: bi("Lintas bahasa", "Across languages"),
  enamBahasa: bi("Enam bahasa, satu angka", "Six languages, one number"),
  loading: bi("Memuat mesin WebAssembly…", "Loading the WebAssembly engine…"),
  loadFailed: bi("Mesin gagal dimuat", "The engine failed to load"),
  reload: bi("Muat ulang", "Reload"),
  theme: bi("Tema", "Theme"),
  steps: bi("Langkah perhitungan", "Calculation steps"),
  controls: bi("Kontrol", "Controls"),
  result: bi("Hasil", "Result"),
  preset: bi("Contoh kasus", "Worked examples"),
  reset: bi("Atur ulang", "Reset"),
  engineVersion: bi("Mesin", "Engine"),
  sourceCode: bi("Kode sumber", "Source code"),
  builtWith: bi(
    "Rust, WebAssembly, dan TypeScript. Tanpa dependensi saat berjalan.",
    "Rust, WebAssembly, and TypeScript. Zero runtime dependencies.",
  ),
  notFound: bi("Laboratorium tidak ditemukan.", "Laboratory not found."),
  backHome: bi("Kembali ke daftar", "Back to the index"),
  syllabus: bi("Sesi", "Session"),
  addEvidence: bi("Tambah bukti", "Add evidence"),
  removeEvidence: bi("Hapus", "Remove"),
  evidence: bi("Bukti", "Evidence"),
  interpretation: bi("Interpretasi", "Interpretation"),
  notes: bi("Catatan & Definisi", "Notes & Definitions"),
  whatThisComputes: bi("Apa yang dihitung di sini", "What is computed here"),
  definitions: bi("Definisi", "Definitions"),
  formulas: bi("Rumus", "Formulas"),
  pitfalls: bi("Kekeliruan yang sering terjadi", "Common pitfalls"),
  references: bi("Rujukan", "References"),
  term: bi("Istilah", "Term"),
  meaning: bi("Arti", "Meaning"),
  formula: bi("Bentuk", "Expression"),
  whenItApplies: bi("Kapan berlaku", "When it applies"),
} as const;
