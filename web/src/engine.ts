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
