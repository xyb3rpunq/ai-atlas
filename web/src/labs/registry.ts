/**
 * Daftar laboratorium.
 *
 * Setiap sesi kuliah IND323 punya satu entri. Sesi yang mesinnya belum siap
 * tetap tercantum dengan `mount: null` supaya peta silabusnya utuh dan
 * kemajuannya jujur terlihat, bukan disembunyikan.
 */

import type { Bilingual } from "../i18n.js";
import { bi } from "../i18n.js";
import { bayesLab } from "./bayes.js";
import { certaintyLab } from "./certainty.js";
import { fuzzyLab } from "./fuzzy.js";
import { expertLab } from "./expert.js";
import { neuralLab } from "./neural.js";
import { searchLab } from "./search.js";

/** Sebuah laboratorium yang bisa dipasang ke dalam halaman. */
export interface Lab {
  /** Bagian URL setelah tanda pagar, mis. `#/certainty-factor`. */
  slug: string;
  /** Nomor sesi pada silabus IND323. */
  session: number;
  /** Judul laboratorium. */
  title: Bilingual;
  /** Penjelasan singkat yang muncul di bawah judul. */
  blurb: Bilingual;
  /**
   * Memasang laboratorium ke dalam elemen yang diberikan.
   * Mengembalikan fungsi pembersih yang dipanggil saat pengguna berpindah.
   */
  mount: (root: HTMLElement) => () => void;
}

/** Entri silabus yang belum punya laboratorium. */
export interface PlannedLab {
  session: number;
  title: Bilingual;
  slug?: undefined;
}

/** Sesi yang sudah bisa dijalankan. */
export const LABS: Lab[] = [
  certaintyLab,
  bayesLab,
  fuzzyLab,
  searchLab,
  neuralLab,
  expertLab,
];

/** Seluruh sesi silabus, termasuk yang belum terimplementasi. */
export const SYLLABUS: { session: number; title: Bilingual; slug?: string }[] = [
  { session: 1, title: bi("Pengantar Kecerdasan Buatan", "Introduction to AI") },
  { session: 2, title: bi("Agen Cerdas & Ruang Keadaan", "Agents & State Space") },
  { session: 3, title: certaintyLab.title, slug: certaintyLab.slug },
  { session: 4, title: bayesLab.title, slug: bayesLab.slug },
  { session: 5, title: bi("Logika Fuzzy I", "Fuzzy Logic I"), slug: fuzzyLab.slug },
  { session: 6, title: bi("Logika Fuzzy II", "Fuzzy Logic II"), slug: fuzzyLab.slug },
  { session: 7, title: bi("Representasi Pengetahuan", "Knowledge Representation") },
  { session: 8, title: searchLab.title, slug: searchLab.slug },
  { session: 9, title: neuralLab.title, slug: neuralLab.slug },
  { session: 10, title: bi("Pemrosesan Bahasa Alami", "Natural Language Processing") },
  { session: 11, title: expertLab.title, slug: expertLab.slug },
  { session: 12, title: bi("Sains Data & Big Data", "Data Science & Big Data") },
  { session: 13, title: bi("Machine Learning", "Machine Learning") },
  { session: 14, title: bi("Robotika", "Robotics") },
];

/** Mencari laboratorium berdasarkan slug-nya. */
export function findLab(slug: string): Lab | undefined {
  return LABS.find((l) => l.slug === slug);
}
