/**
 * Jembatan bertipe ke mesin WebAssembly yang ditulis dengan Rust.
 *
 * Setiap fungsi Rust mengembalikan JSON berbentuk `{"ok": ...}` atau
 * `{"err": "..."}`. Modul ini membongkar amplop itu satu kali di sini supaya
 * kode laboratorium tidak perlu mengurusnya berulang kali.
 *
 * Catatan ketelitian: arah Rust ke JavaScript aman karena `JSON.parse` pada
 * mesin V8 membulatkan pecahan dengan benar. Arah sebaliknya melewati parser
 * `serde_json` yang diketahui meleset satu ULP pada sebagian nilai, jadi nilai
 * yang dikirim ke Rust selalu berupa argumen numerik langsung, bukan pecahan
 * yang disisipkan ke dalam teks JSON.
 */

import init, * as wasm from "../pkg/ai_wasm.js";
import wasmUrl from "../pkg/ai_wasm_bg.wasm?url";

/** Amplop hasil dari sisi Rust. */
type Envelope<T> = { ok: T } | { err: string };

/** Kegagalan yang berasal dari mesin, dibedakan dari galat pemrograman biasa. */
export class EngineError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "EngineError";
  }
}

let ready: Promise<void> | null = null;

/**
 * Memuat modul WebAssembly. Aman dipanggil berkali-kali: pemuatan hanya terjadi
 * sekali dan pemanggil berikutnya menunggu janji yang sama.
 */
export function load(): Promise<void> {
  if (ready === null) {
    ready = init({ module_or_path: wasmUrl }).then(() => undefined);
  }
  return ready;
}

/** Membongkar amplop JSON, melempar {@link EngineError} bila berisi galat. */
function unwrap<T>(raw: string): T {
  let parsed: Envelope<T>;
  try {
    parsed = JSON.parse(raw) as Envelope<T>;
  } catch {
    throw new EngineError(`mesin mengembalikan JSON tidak sah: ${raw.slice(0, 120)}`);
  }
  if ("err" in parsed) throw new EngineError(parsed.err);
  return parsed.ok;
}

/** Satu langkah perhitungan certainty factor. */
export interface CfStep {
  op: string;
  formula: string;
  value: number;
}

/** Hasil penggabungan certainty factor beserta jejaknya. */
export interface CfResult {
  value: number;
  steps: CfStep[];
  label_id: string;
  label_en: string;
}

/** Satu langkah perhitungan Bayes. */
export interface BayesStep {
  label: string;
  formula: string;
  value: number;
}

/** Hasil lengkap kasus Bayes dua hipotesis. */
export interface BayesResult {
  posterior: number;
  posterior_complement: number;
  evidence: number;
  likelihood_ratio: number;
  steps: BayesStep[];
}

/** Hasil klasifikasi Naive Bayes. */
export interface Prediction {
  label: string;
  probabilities: Record<string, number>;
}

/** Ringkasan satu sesi kuliah yang sudah terimplementasi di mesin. */
export interface SessionInfo {
  session: number;
  module: string;
  title_id: string;
  title_en: string;
}

/** Versi pustaka inti Rust. */
export function version(): string {
  return wasm.version();
}

/** Daftar sesi yang sudah punya implementasi. */
export function sessions(): SessionInfo[] {
  return unwrap<SessionInfo[]>(wasm.sessions());
}

/** `CF = MB - MD`. */
export function cfFromMbMd(mb: number, md: number): number {
  return unwrap<number>(wasm.cf_from_mb_md(mb, md));
}

/** Menggabungkan daftar certainty factor secara paralel, dengan jejak langkah. */
export function cfCombine(cfs: number[]): CfResult {
  return unwrap<CfResult>(wasm.cf_combine(JSON.stringify(cfs)));
}

/** CF gabungan premis `AND` (minimum) atau `OR` (maksimum). */
export function cfPremise(cfs: number[], operator: "AND" | "OR"): number {
  return unwrap<number>(wasm.cf_premise(JSON.stringify(cfs), operator));
}

/** CF berantai: CF aturan dikali CF bukti. */
export function cfSequential(cfRule: number, cfEvidence: number): number {
  return unwrap<number>(wasm.cf_sequential(cfRule, cfEvidence));
}

/** Kasus Bayes dua hipotesis lengkap dengan langkahnya. */
export function bayesBinary(
  prior: number,
  likelihoodH: number,
  likelihoodNotH: number,
): BayesResult {
  return unwrap<BayesResult>(wasm.bayes_binary(prior, likelihoodH, likelihoodNotH));
}

/** Posterior seluruh hipotesis dari larik prior dan likelihood. */
export function bayesPosteriorAll(priors: number[], likelihoods: number[]): number[] {
  return unwrap<number[]>(
    wasm.bayes_posterior_all(JSON.stringify(priors), JSON.stringify(likelihoods)),
  );
}

/** Satu baris data latih kategorikal. */
export interface CategoricalSample {
  features: string[];
  label: string;
}

/** Melatih Naive Bayes kategorikal lalu memprediksi satu baris. */
export function naiveBayesPredict(
  samples: CategoricalSample[],
  query: string[],
  alpha = 1,
): Prediction {
  return unwrap<Prediction>(
    wasm.naive_bayes_predict(JSON.stringify(samples), JSON.stringify(query), alpha),
  );
}

// ---------------------------------------------------------------------------
// Sesi 5 & 6 — Logika Fuzzy
// ---------------------------------------------------------------------------

/** Bentuk fungsi keanggotaan, sepadan dengan enum bertanda di sisi Rust. */
export type Membership =
  | { kind: "triangular"; a: number; b: number; c: number }
  | { kind: "trapezoidal"; a: number; b: number; c: number; d: number }
  | { kind: "gaussian"; mean: number; sigma: number }
  | { kind: "sigmoid"; a: number; c: number }
  | { kind: "scurve"; a: number; b: number }
  | { kind: "zcurve"; a: number; b: number };

/** Satu himpunan kabur bernama. */
export interface NamedSet {
  name: string;
  membership: Membership;
}

/** Variabel linguistik beserta semesta dan himpunan-himpunannya. */
export interface FuzzyVariable {
  name: string;
  min: number;
  max: number;
  sets: NamedSet[];
}

/** Satu premis aturan. */
export interface Antecedent {
  variable: string;
  set: string;
}

/** Satu aturan JIKA-MAKA. */
export interface FuzzyRule {
  antecedents: Antecedent[];
  connective: "AND" | "OR";
  consequent_set: string;
  consequent_value: number;
  weight: number;
}

/** Sistem inferensi kabur lengkap. */
export interface FuzzySystem {
  inputs: FuzzyVariable[];
  output: FuzzyVariable;
  rules: FuzzyRule[];
}

/** Jejak satu aturan setelah dievaluasi. */
export interface RuleTrace {
  index: number;
  degrees: number[];
  firing_strength: number;
  text: string;
}

/** Hasil inferensi kabur. */
export interface Inference {
  crisp: number;
  rules: RuleTrace[];
  xs: number[];
  ys: number[];
}

/** Metode defuzzifikasi yang dikenali mesin. */
export type DefuzzMethod =
  | "centroid"
  | "bisector"
  | "mean_of_maximum"
  | "smallest_of_maximum"
  | "largest_of_maximum";

/** Derajat keanggotaan sebuah nilai pada satu fungsi keanggotaan. */
export function fuzzyDegree(set: Membership, x: number): number {
  return unwrap<number>(wasm.fuzzy_degree(JSON.stringify(set), x));
}

/** Kurva sebuah fungsi keanggotaan, tercuplik seragam pada semesta. */
export function fuzzyCurve(
  set: Membership,
  min: number,
  max: number,
  samples: number,
): { xs: number[]; ys: number[] } {
  return unwrap<{ xs: number[]; ys: number[] }>(
    wasm.fuzzy_curve(JSON.stringify(set), min, max, samples),
  );
}

/** Inferensi kabur lengkap dengan salah satu dari tiga mesin. */
export function fuzzyInfer(
  system: FuzzySystem,
  inputs: [string, number][],
  engineName: "mamdani" | "sugeno" | "tsukamoto",
  method: DefuzzMethod,
  samples = 201,
): Inference {
  return unwrap<Inference>(
    wasm.fuzzy_infer(
      JSON.stringify(system),
      JSON.stringify(inputs),
      engineName,
      method,
      samples,
    ),
  );
}

// ---------------------------------------------------------------------------
// Sesi 8 — Teknik Pencarian
// ---------------------------------------------------------------------------

/** Sebuah titik pada kisi. */
export interface Point {
  x: number;
  y: number;
}

/** Kisi tempat pencarian berlangsung. */
export interface Grid {
  width: number;
  height: number;
  /** `true` berarti dinding. Panjangnya `width * height`, baris demi baris. */
  walls: boolean[];
  diagonal: boolean;
}

/** Algoritma pencarian yang tersedia. */
export type Algorithm =
  | "breadth_first"
  | "depth_first"
  | "depth_limited"
  | "iterative_deepening"
  | "uniform_cost"
  | "greedy_best_first"
  | "a_star"
  | "hill_climbing"
  | "simulated_annealing";

/** Fungsi heuristik yang tersedia. */
export type Heuristic = "manhattan" | "euclidean" | "chebyshev" | "zero";

/** Pengaturan sebuah pencarian. */
export interface SearchOptions {
  algorithm: Algorithm;
  heuristic: Heuristic;
  depth_limit: number;
  seed: number;
  max_expansions: number;
}

/** Hasil sebuah pencarian. */
export interface SearchResult {
  path: Point[];
  /** Urutan sel yang dibuka, dipakai untuk menganimasikan pencarian. */
  expanded: Point[];
  cost: number;
  found: boolean;
  expansions: number;
  peak_frontier: number;
}

/** Satu baris pada tabel perbandingan antaralgoritma. */
export interface CompareRow {
  algorithm: Algorithm;
  name: string;
  optimal: boolean;
  uses_heuristic: boolean;
  found: boolean;
  /** Biaya jalur; bernilai `-1` bila tujuan tidak tercapai. */
  cost: number;
  steps: number;
  expansions: number;
  peak_frontier: number;
}

/** Kisi kosong tanpa dinding. */
export function searchEmptyGrid(width: number, height: number): Grid {
  return unwrap<Grid>(wasm.search_empty_grid(width, height));
}

/** Labirin acak yang dijamin punya jalur keluar. */
export function searchMaze(width: number, height: number, seed: number): Grid {
  return unwrap<Grid>(wasm.search_maze(width, height, BigInt(seed)));
}

/** Menjalankan satu pencarian. */
export function searchRun(
  grid: Grid,
  start: Point,
  goal: Point,
  options: SearchOptions,
): SearchResult {
  return unwrap<SearchResult>(
    wasm.search_run(
      JSON.stringify(grid),
      start.x,
      start.y,
      goal.x,
      goal.y,
      JSON.stringify(options),
    ),
  );
}

/** Menjalankan seluruh algoritma pada kisi yang sama untuk dibandingkan. */
export function searchCompare(
  grid: Grid,
  start: Point,
  goal: Point,
  options: SearchOptions,
): CompareRow[] {
  return unwrap<CompareRow[]>(
    wasm.search_compare(
      JSON.stringify(grid),
      start.x,
      start.y,
      goal.x,
      goal.y,
      JSON.stringify(options),
    ),
  );
}
