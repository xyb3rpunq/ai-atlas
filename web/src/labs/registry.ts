/**
 * Daftar laboratorium.
 *
 * Setiap sesi kuliah IND323 punya satu entri. Sesi yang mesinnya belum siap
 * tetap tercantum dengan `mount: null` supaya peta silabusnya utuh dan
 * kemajuannya jujur terlihat, bukan disembunyikan.
 */

import type { Bilingual } from "../i18n.js";
import { bi } from "../i18n.js";
import { agentLab } from "./agent.js";
import { bayesLab } from "./bayes.js";
import { certaintyLab } from "./certainty.js";
import { fuzzyLab } from "./fuzzy.js";
import { elizaLab } from "./eliza.js";
import { expertLab } from "./expert.js";
import { knowledgeLab } from "./knowledge.js";
import { mlLab } from "./ml.js";
import { nlpLab } from "./nlp.js";
import { roboticsLab } from "./robotics.js";
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
  elizaLab,
  agentLab,
  certaintyLab,
  bayesLab,
  fuzzyLab,
  searchLab,
  neuralLab,
  expertLab,
  mlLab,
  nlpLab,
  knowledgeLab,
  roboticsLab,
];

/** Seluruh sesi silabus, termasuk yang belum terimplementasi. */
export const SYLLABUS: { session: number; title: Bilingual; slug?: string }[] = [
  { session: 1, title: elizaLab.title, slug: elizaLab.slug },
  { session: 2, title: agentLab.title, slug: agentLab.slug },
  { session: 3, title: certaintyLab.title, slug: certaintyLab.slug },
  { session: 4, title: bayesLab.title, slug: bayesLab.slug },
  { session: 5, title: bi("Logika Fuzzy I", "Fuzzy Logic I"), slug: fuzzyLab.slug },
  { session: 6, title: bi("Logika Fuzzy II", "Fuzzy Logic II"), slug: fuzzyLab.slug },
  { session: 7, title: knowledgeLab.title, slug: knowledgeLab.slug },
  { session: 8, title: searchLab.title, slug: searchLab.slug },
  { session: 9, title: neuralLab.title, slug: neuralLab.slug },
  { session: 10, title: nlpLab.title, slug: nlpLab.slug },
  { session: 11, title: expertLab.title, slug: expertLab.slug },
  { session: 12, title: bi("Sains Data & Big Data", "Data Science & Big Data"), slug: mlLab.slug },
  { session: 13, title: mlLab.title, slug: mlLab.slug },
  { session: 14, title: roboticsLab.title, slug: roboticsLab.slug },
];

/** Mencari laboratorium berdasarkan slug-nya. */
export function findLab(slug: string): Lab | undefined {
  return LABS.find((l) => l.slug === slug);
}
