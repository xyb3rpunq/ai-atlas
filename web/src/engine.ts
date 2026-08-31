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

// ---------------------------------------------------------------------------
// Sesi 9 — Jaringan Syaraf Tiruan
// ---------------------------------------------------------------------------

/** Fungsi aktivasi yang dikenali mesin. */
export type Activation = "step" | "sigmoid" | "tanh" | "relu" | "leaky_relu" | "linear";

/** Satu lapisan jaringan. */
export interface Layer {
  weights: number[][];
  biases: number[];
  activation: Activation;
}

/** Keadaan jaringan yang bisa disimpan dan dilanjutkan. */
export interface NeuralNetwork {
  layers: Layer[];
  learning_rate: number;
  momentum: number;
}

/** Ringkasan jaringan beserta angka yang layak ditampilkan. */
export interface NetworkSummary {
  network: NeuralNetwork;
  input_size: number;
  output_size: number;
  parameters: number;
  /**
   * Laju belajar setelah memperhitungkan momentum, kira-kira
   * `laju / (1 - momentum)`. Pada momentum 0,9 nilainya sepuluh kali lipat.
   */
  effective_learning_rate: number;
  /** Benar bila langkah efektifnya berada di wilayah yang cenderung menyimpang. */
  step_risky: boolean;
}

/** Catatan satu epoch pelatihan. */
export interface EpochRecord {
  epoch: number;
  loss: number;
  accuracy: number;
}

/** Hasil satu potongan pelatihan. */
export interface TrainResult {
  summary: NetworkSummary;
  history: EpochRecord[];
}

/** Kumpulan data latih. */
export interface Dataset {
  x: number[][];
  y: number[][];
}

/** Kisi keluaran jaringan, dipakai menggambar batas keputusan. */
export interface DecisionGrid {
  resolution: number;
  min: number;
  max: number;
  values: number[];
}

/** Membuat jaringan baru. */
export function neuralCreate(
  sizes: number[],
  hidden: Activation,
  output: Activation,
  learningRate: number,
  momentum: number,
  seed: number,
): NetworkSummary {
  return unwrap<NetworkSummary>(
    wasm.neural_create(
      JSON.stringify(sizes),
      hidden,
      output,
      learningRate,
      momentum,
      BigInt(seed),
    ),
  );
}

/** Kumpulan data bawaan. */
export function neuralDataset(
  name: "xor" | "and" | "or" | "spiral",
  points: number,
  noise: number,
  seed: number,
): Dataset {
  return unwrap<Dataset>(wasm.neural_dataset(name, points, noise, BigInt(seed)));
}

/** Melatih satu potongan epoch. */
export function neuralTrain(
  network: NeuralNetwork,
  data: Dataset,
  epochs: number,
  tolerance: number,
  seed: number,
): TrainResult {
  return unwrap<TrainResult>(
    wasm.neural_train(
      JSON.stringify(network),
      JSON.stringify(data.x),
      JSON.stringify(data.y),
      epochs,
      tolerance,
      BigInt(seed),
    ),
  );
}

/** Keluaran jaringan pada kisi seragam. */
export function neuralDecisionGrid(
  network: NeuralNetwork,
  min: number,
  max: number,
  resolution: number,
): DecisionGrid {
  return unwrap<DecisionGrid>(
    wasm.neural_decision_grid(JSON.stringify(network), min, max, resolution),
  );
}

// ---------------------------------------------------------------------------
// Sesi 11 — Sistem Pakar
// ---------------------------------------------------------------------------

/** Satu premis aturan; `expected: false` berarti premis ingkar. */
export interface ExpertPremise {
  fact: string;
  expected: boolean;
}

/** Satu aturan JIKA-MAKA. */
export interface ExpertRule {
  id: string;
  premises: ExpertPremise[];
  connective: "AND" | "OR";
  conclusion: string;
  certainty: number;
  rationale: string;
}

/** Basis pengetahuan lengkap. */
export interface KnowledgeBase {
  name: string;
  rules: ExpertRule[];
  /** Fakta yang hanya bisa diketahui dengan bertanya kepada pengguna. */
  askable: string[];
}

/** Laporan kesehatan basis pengetahuan. */
export interface KnowledgeBaseReport {
  name: string;
  rules: number;
  derivable: string[];
  leaf_facts: string[];
  /** Premis yang tidak bisa disimpulkan maupun ditanyakan. */
  unreachable_facts: string[];
  askable: string[];
}

/** Satu langkah penalaran runut maju. */
export interface ExpertStep {
  order: number;
  rule_id: string;
  text: string;
  conclusion: string;
  premise_certainty: number;
  conclusion_certainty: number;
  support: [string, number][];
}

/** Hasil penalaran runut maju. */
export interface ForwardResult {
  /** Fakta yang benar-benar disimpulkan sistem. */
  derived: [string, number][];
  /** Fakta yang berasal dari masukan pengguna. */
  given: [string, number][];
  all_facts: [string, number][];
  steps: ExpertStep[];
  passes: number;
}

/**
 * Bagaimana sebuah tujuan diselesaikan runut mundur.
 *
 * Bentuknya bertanda seragam, sehingga satu pemeriksaan `kind` cukup untuk
 * seluruh varian.
 */
export type ProofOutcome =
  | { kind: "known" }
  | { kind: "needs_asking" }
  | { kind: "unprovable" }
  | { kind: "derived"; rule_id: string };

/** Satu simpul pada pohon pembuktian. */
export interface ProofNode {
  goal: string;
  depth: number;
  certainty: number;
  outcome: ProofOutcome;
  children: ProofNode[];
}

/** Hasil penalaran runut mundur. */
export interface BackwardResult {
  goal: string;
  certainty: number;
  proof: ProofNode;
  /** Fakta yang masih perlu ditanyakan agar penelusuran bisa dilanjutkan. */
  questions: string[];
}

/** Basis pengetahuan contoh: diagnosis flu dari studi kasus modul. */
export function expertSampleKb(): KnowledgeBase {
  return unwrap<KnowledgeBase>(wasm.expert_sample_kb());
}

/** Memeriksa kesehatan sebuah basis pengetahuan. */
export function expertInspectKb(kb: KnowledgeBase): KnowledgeBaseReport {
  return unwrap<KnowledgeBaseReport>(wasm.expert_inspect_kb(JSON.stringify(kb)));
}

/** Penalaran runut maju dari fakta yang diketahui. */
export function expertForward(
  kb: KnowledgeBase,
  facts: [string, number][],
  threshold: number,
): ForwardResult {
  return unwrap<ForwardResult>(
    wasm.expert_forward(JSON.stringify(kb), JSON.stringify(facts), threshold),
  );
}

/** Penalaran runut mundur terhadap sebuah tujuan. */
export function expertBackward(
  kb: KnowledgeBase,
  facts: [string, number][],
  goal: string,
): BackwardResult {
  return unwrap<BackwardResult>(
    wasm.expert_backward(JSON.stringify(kb), JSON.stringify(facts), goal),
  );
}

/** Jawaban atas pertanyaan "kenapa aturan ini ada". */
export function expertWhy(kb: KnowledgeBase, ruleId: string): string {
  return unwrap<string>(wasm.expert_why(JSON.stringify(kb), ruleId));
}

/** Jawaban atas pertanyaan "bagaimana kesimpulan ini diperoleh". */
export function expertHow(
  kb: KnowledgeBase,
  facts: [string, number][],
  fact: string,
): string[] {
  return unwrap<string[]>(
    wasm.expert_how(JSON.stringify(kb), JSON.stringify(facts), fact),
  );
}
