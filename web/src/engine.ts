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
import type { Lang } from "./i18n.js";

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

/**
 * Jejak satu aturan setelah dievaluasi.
 *
 * Membawa **bentuk** aturannya, bukan kalimatnya. Kalimat yang dirakit mesin
 * akan selalu berbahasa Indonesia, sedangkan yang membacanya belum tentu;
 * merakitnya di sini membuat "JIKA … ATAU … MAKA …" dan "IF … OR … THEN …"
 * berasal dari satu sumber yang sama. Lihat {@link kalimatAturanKabur}.
 */
export interface RuleTrace {
  index: number;
  degrees: number[];
  firing_strength: number;
  antecedents: Antecedent[];
  connective: "AND" | "OR";
  output: string;
  consequent_set: string;
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
  /** Premis aturan, apa adanya — termasuk premis yang dinegasikan. */
  premises: ExpertPremise[];
  connective: "AND" | "OR";
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

// ---------------------------------------------------------------------------
// Sesi 12 & 13 — Sains Data dan Machine Learning
// ---------------------------------------------------------------------------

/** Ukuran jarak antartitik. */
export type Distance = "euclidean" | "manhattan" | "chebyshev";

/** Satu tetangga beserta jaraknya. */
export interface Neighbour {
  index: number;
  distance: number;
  label: string;
}

/** Hasil klasifikasi KNN. */
export interface KnnResult {
  label: string;
  neighbours: Neighbour[];
  votes: Record<string, number>;
}

/** Wilayah keputusan KNN pada kisi seragam. */
export interface KnnRegions {
  classes: string[];
  resolution: number;
  /** Indeks kelas tiap sel, baris demi baris dari bawah ke atas. */
  cells: number[];
}

/** Hasil pengelompokan K-Means. */
export interface Clustering {
  centroids: number[][];
  assignments: number[];
  inertia: number;
  iterations: number;
  converged: boolean;
}

/** Satu simpul pohon keputusan. */
export type TreeNode =
  | { kind: "leaf"; label: string; samples: number; purity: number }
  | {
      kind: "branch";
      attribute: number;
      attribute_name: string;
      gain: number;
      children: Record<string, TreeNode>;
      fallback: string;
    };

/** Pohon keputusan beserta angka yang menjelaskan pembentukannya. */
export interface TreeResult {
  tree: TreeNode;
  depth: number;
  leaves: number;
  root_entropy: number;
  gains: [string, number][];
}

/** Model regresi linear satu peubah. */
export interface LinearRegression {
  intercept: number;
  slope: number;
  r_squared: number;
}

/** Matriks konfusi beserta ukuran turunannya. */
export interface Evaluation {
  labels: string[];
  matrix: number[][];
  accuracy: number;
  precision: Record<string, number>;
  recall: Record<string, number>;
  f1: Record<string, number>;
  macro_f1: number;
  /** Ketepatan yang dicapai dengan selalu menebak kelas terbanyak. */
  baseline_accuracy: number;
}

/** Kumpulan data tenis klasik untuk pohon keputusan. */
export interface TennisDataset {
  names: string[];
  values: string[][];
  x: string[][];
  y: string[];
}

/** Klasifikasi satu titik dengan K-Nearest Neighbours. */
export function mlKnnPredict(
  x: number[][],
  y: string[],
  query: number[],
  k: number,
  distance: Distance,
  weighted: boolean,
): KnnResult {
  return unwrap<KnnResult>(
    wasm.ml_knn_predict(
      JSON.stringify(x),
      JSON.stringify(y),
      JSON.stringify(query),
      k,
      distance,
      weighted,
    ),
  );
}

/** Wilayah keputusan KNN pada kisi seragam. */
export function mlKnnRegions(
  x: number[][],
  y: string[],
  k: number,
  distance: Distance,
  weighted: boolean,
  min: number,
  max: number,
  resolution: number,
): KnnRegions {
  return unwrap<KnnRegions>(
    wasm.ml_knn_regions(
      JSON.stringify(x),
      JSON.stringify(y),
      k,
      distance,
      weighted,
      min,
      max,
      resolution,
    ),
  );
}

/** Pengelompokan K-Means. */
export function mlKmeans(
  x: number[][],
  k: number,
  distance: Distance,
  maxIterations: number,
  seed: number,
): Clustering {
  return unwrap<Clustering>(
    wasm.ml_kmeans(JSON.stringify(x), k, distance, maxIterations, BigInt(seed)),
  );
}

/** Membangun pohon keputusan ID3 dari data kategorikal. */
export function mlBuildTree(
  x: string[][],
  y: string[],
  names: string[],
  maxDepth: number,
): TreeResult {
  return unwrap<TreeResult>(
    wasm.ml_build_tree(
      JSON.stringify(x),
      JSON.stringify(y),
      JSON.stringify(names),
      maxDepth,
    ),
  );
}

/** Memprediksi label sebuah baris dengan pohon yang sudah dibangun. */
export function mlTreePredict(tree: TreeNode, row: string[]): string {
  return unwrap<string>(wasm.ml_tree_predict(JSON.stringify(tree), JSON.stringify(row)));
}

/** Regresi linear satu peubah. */
export function mlFitLinear(x: number[], y: number[]): LinearRegression {
  return unwrap<LinearRegression>(
    wasm.ml_fit_linear(JSON.stringify(x), JSON.stringify(y)),
  );
}

/** Matriks konfusi beserta ukuran turunannya. */
export function mlEvaluate(actual: string[], predicted: string[]): Evaluation {
  return unwrap<Evaluation>(
    wasm.ml_evaluate(JSON.stringify(actual), JSON.stringify(predicted)),
  );
}

/** Kumpulan data tenis klasik. */
export function mlTennisDataset(): TennisDataset {
  return unwrap<TennisDataset>(wasm.ml_tennis_dataset());
}

// ---------------------------------------------------------------------------
// Sesi 10 — Pemrosesan Bahasa Alami
// ---------------------------------------------------------------------------

/** Satu langkah pengupasan imbuhan. */
export interface StemStep {
  kind: string;
  affix: string;
  result: string;
}

/** Hasil pencarian kata dasar beserta jejaknya. */
export interface StemResult {
  original: string;
  stem: string;
  steps: StemStep[];
  in_dictionary: boolean;
}

/** Hasil seluruh tahap pemrosesan teks. */
export interface NlpPipeline {
  sentences: string[];
  tokens: string[];
  after_stopwords: string[];
  stems: StemResult[];
  final_tokens: string[];
}

/** Bobot TF-IDF sebuah korpus beserta kemiripan antardokumen. */
export interface TfIdfResult {
  vocabulary: string[];
  idf: number[];
  vectors: number[][];
  similarity: number[][];
  documents: string[][];
}

/** Jarak sunting dan kemiripannya. */
export interface EditDistance {
  distance: number;
  similarity: number;
}

/** Hasil analisis sentimen. */
export interface Sentiment {
  score: number;
  label: string;
  matches: [string, number][];
}

/** Memenggal, membuang kata henti, lalu mencari kata dasarnya. */
export function nlpPipeline(
  text: string,
  removeStopwords: boolean,
  stem: boolean,
): NlpPipeline {
  return unwrap<NlpPipeline>(wasm.nlp_pipeline(text, removeStopwords, stem));
}

/** Pencarian kata dasar satu kata beserta jejaknya. */
export function nlpStem(word: string): StemResult {
  return unwrap<StemResult>(wasm.nlp_stem(word));
}

/** Bobot TF-IDF sebuah korpus. */
export function nlpTfIdf(
  documents: string[],
  removeStopwords: boolean,
  stem: boolean,
): TfIdfResult {
  return unwrap<TfIdfResult>(
    wasm.nlp_tfidf(JSON.stringify(documents), removeStopwords, stem),
  );
}

/** Jarak sunting antara dua kata. */
export function nlpLevenshtein(a: string, b: string): EditDistance {
  return unwrap<EditDistance>(wasm.nlp_levenshtein(a, b));
}

/** N-gram kata dari sebuah teks. */
export function nlpNgrams(text: string, n: number): string[] {
  return unwrap<string[]>(wasm.nlp_ngrams(text, n));
}

/** Analisis sentimen berbasis leksikon. */
export function nlpSentiment(text: string): Sentiment {
  return unwrap<Sentiment>(wasm.nlp_sentiment(text));
}

// ---------------------------------------------------------------------------
// Sesi 7 — Representasi Pengetahuan
// ---------------------------------------------------------------------------

/** Satu baris tabel kebenaran. */
export interface TruthRow {
  values: boolean[];
  result: boolean;
}

/** Tabel kebenaran lengkap beserta bentuk normal konjungtifnya. */
export interface TruthTable {
  text: string;
  variables: string[];
  rows: TruthRow[];
  tautology: boolean;
  satisfiable: boolean;
  contradiction: boolean;
  cnf: string[];
}

/** Satu langkah resolusi. */
export interface ResolutionStep {
  order: number;
  left: string;
  right: string;
  pivot: string;
  result: string;
}

/** Hasil pembuktian dengan resolusi. */
export interface ResolutionProof {
  proved: boolean;
  initial_clauses: string[];
  steps: ResolutionStep[];
  generated: number;
}

/** Sebuah relasi berarah pada jaringan semantik. */
export interface Relation {
  from: string;
  label: string;
  to: string;
}

/** Jaringan semantik beserta sifat warisan simpul terpilih. */
export interface SemanticNetworkView {
  relations: Relation[];
  nodes: string[];
  selected: string;
  properties: Relation[];
  ancestors: string[];
}

/** Tabel kebenaran sebuah rumus proposisi. */
export function logicTruthTable(formula: string): TruthTable {
  return unwrap<TruthTable>(wasm.logic_truth_table(formula));
}

/** Apakah dua rumus setara secara logika. */
export function logicEquivalent(a: string, b: string): boolean {
  return unwrap<boolean>(wasm.logic_equivalent(a, b));
}

/** Membuktikan kesimpulan dari basis pengetahuan dengan resolusi. */
export function logicResolve(knowledge: string[], conclusion: string): ResolutionProof {
  return unwrap<ResolutionProof>(
    wasm.logic_resolve(JSON.stringify(knowledge), conclusion),
  );
}

/** Jaringan semantik contoh beserta sifat warisan sebuah simpul. */
export function logicSemanticNetwork(node: string): SemanticNetworkView {
  return unwrap<SemanticNetworkView>(wasm.logic_semantic_network(node));
}

// ---------------------------------------------------------------------------
// Sesi 1 — Pengantar Kecerdasan Buatan (ELIZA)
// ---------------------------------------------------------------------------

/** Satu aturan pencocokan ELIZA. */
export interface ElizaRule {
  keyword: string;
  priority: number;
  responses: string[];
}

/** Naskah ELIZA lengkap. */
export interface ElizaScript {
  name: string;
  rules: ElizaRule[];
  fallbacks: string[];
  reflections: [string, string][];
}

/** Balasan ELIZA beserta penjelasan bagaimana ia dihasilkan. */
export interface ElizaReply {
  text: string;
  matched_keyword: string;
  priority: number;
  reflected_fragment: string;
  used_fallback: boolean;
}

/** Ringkasan naskah, untuk membongkar ukuran sebenarnya. */
export interface ElizaSummary {
  name: string;
  rules: number;
  total_responses: number;
  fallbacks: number;
  reflections: number;
  keywords: [string, number][];
}

/**
 * Balasan ELIZA untuk sebuah masukan, memakai naskah bahasa yang diminta.
 *
 * Bahasanya diteruskan ke mesin, bukan diterjemahkan sesudahnya, karena ELIZA
 * mencocokkan **kata kunci**: "saya merasa" tidak akan pernah cocok dengan
 * kalimat berbahasa Inggris, dan menerjemahkan balasannya sesudah dicocokkan
 * berarti menerjemahkan jawaban atas kalimat yang tidak pernah dipahami.
 */
export function elizaRespond(input: string, seed: number, bahasa: Lang): ElizaReply {
  return unwrap<ElizaReply>(wasm.eliza_respond(input, BigInt(seed), bahasa));
}

/** Ringkasan naskah ELIZA untuk sebuah bahasa. */
export function elizaScriptSummary(bahasa: Lang): ElizaSummary {
  return unwrap<ElizaSummary>(wasm.eliza_script_summary(bahasa));
}

/** Naskah ELIZA lengkap untuk sebuah bahasa. */
export function elizaScript(bahasa: Lang): ElizaScript {
  return unwrap<ElizaScript>(wasm.eliza_script(bahasa));
}

// ---------------------------------------------------------------------------
// Sesi 2 — Agen Cerdas dan Ruang Keadaan
// ---------------------------------------------------------------------------

/** Tindakan yang bisa diambil agen. */
export type AgentAction = "suck" | "move_left" | "move_right" | "idle";

/** Jenis agen. */
export type AgentKind =
  | "simple_reflex"
  | "model_based"
  | "goal_based"
  | "utility_based";

/** Satu langkah simulasi agen. */
export interface AgentStep {
  step: number;
  position: number;
  perceived_dirty: boolean;
  action: AgentAction;
  dirty_after: number;
}

/** Hasil menjalankan seorang agen. */
export interface AgentRun {
  kind: AgentKind;
  steps: AgentStep[];
  finished: boolean;
  cost: number;
  actions_taken: number;
  wasted_moves: number;
}

/** Satu langkah penyelesaian teko air. */
export interface JugStep {
  action: string;
  a: number;
  b: number;
}

/** Satu langkah penyeberangan misionaris dan kanibal. */
export interface CrossingStep {
  action: string;
  missionaries_left: number;
  cannibals_left: number;
  boat_left: boolean;
}

/** Menjalankan seluruh jenis agen pada dunia yang sama. */
export function agentCompare(
  dirty: boolean[],
  position: number,
  maxSteps: number,
): AgentRun[] {
  return unwrap<AgentRun[]>(
    wasm.agent_compare(JSON.stringify(dirty), position, maxSteps),
  );
}

/** Menjalankan satu jenis agen. */
export function agentRun(
  dirty: boolean[],
  position: number,
  kind: AgentKind,
  maxSteps: number,
): AgentRun {
  return unwrap<AgentRun>(
    wasm.agent_run(JSON.stringify(dirty), position, kind, maxSteps),
  );
}

/** Menyelesaikan masalah teko air. */
export function agentWaterJug(a: number, b: number, target: number): JugStep[] {
  return unwrap<JugStep[]>(wasm.agent_water_jug(a, b, target));
}

/** Menyelesaikan masalah misionaris dan kanibal. */
export function agentMissionaries(
  missionaries: number,
  cannibals: number,
  boat: number,
): CrossingStep[] {
  return unwrap<CrossingStep[]>(
    wasm.agent_missionaries(missionaries, cannibals, boat),
  );
}

// ---------------------------------------------------------------------------
// Sesi 14 — Robotika
// ---------------------------------------------------------------------------

/** Satu langkah simulasi kendali. */
export interface ControlStep {
  time: number;
  value: number;
  error: number;
  output: number;
}

/** Hasil simulasi kendali PID. */
export interface ControlRun {
  steps: ControlStep[];
  settled: boolean;
  /** Kosong bila sistemnya tidak pernah menetap. */
  settling_time: number | null;
  overshoot_percent: number;
  final_error: number;
}

/** Sudut kedua sendi lengan. */
export interface ArmAngles {
  theta1: number;
  theta2: number;
}

/** Posisi ujung lengan beserta titik sikunya. */
export interface ForwardKinematics {
  x: number;
  y: number;
  elbow_x: number;
  elbow_y: number;
}

/** Sebuah rintangan berbentuk lingkaran. */
export interface Obstacle {
  x: number;
  y: number;
  radius: number;
}

/** Hasil perencanaan lintasan dengan medan potensial. */
export interface PotentialPath {
  points: [number, number][];
  reached: boolean;
  stuck_in_local_minimum: boolean;
  length: number;
}

/** Mensimulasikan kendali PID pada sistem orde pertama. */
export function roboticsPid(
  kp: number,
  ki: number,
  kd: number,
  outputLimit: number,
  setpoint: number,
  timeConstant: number,
  steps: number,
): ControlRun {
  return unwrap<ControlRun>(
    wasm.robotics_pid(kp, ki, kd, outputLimit, setpoint, timeConstant, steps),
  );
}

/** Kinematika maju lengan dua sendi. */
export function roboticsForward(
  theta1: number,
  theta2: number,
  length1: number,
  length2: number,
): ForwardKinematics {
  return unwrap<ForwardKinematics>(
    wasm.robotics_forward(theta1, theta2, length1, length2),
  );
}

/** Kinematika balik; mengembalikan kedua penyelesaiannya. */
export function roboticsInverse(
  x: number,
  y: number,
  length1: number,
  length2: number,
): ArmAngles[] {
  return unwrap<ArmAngles[]>(wasm.robotics_inverse(x, y, length1, length2));
}

/** Merencanakan lintasan dengan medan potensial. */
export function roboticsPath(
  goalX: number,
  goalY: number,
  obstacles: Obstacle[],
  repulsiveGain: number,
  maxSteps: number,
): PotentialPath {
  return unwrap<PotentialPath>(
    wasm.robotics_path(goalX, goalY, JSON.stringify(obstacles), repulsiveGain, maxSteps),
  );
}
