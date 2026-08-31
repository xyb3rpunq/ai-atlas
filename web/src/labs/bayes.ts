/**
 * Laboratorium Sesi 4 — Probabilitas Bayesian.
 *
 * Kasus dua hipotesis: pengguna menggeser prevalensi, tingkat deteksi benar,
 * dan tingkat positif palsu, lalu melihat posterior berubah beserta diagram
 * frekuensi alaminya. Diagram itu penting karena intuisi orang biasanya keliru
 * di sini — sebuah tes yang "akurat 90%" bisa saja lebih sering salah daripada
 * benar ketika kejadiannya jarang.
 */

import * as engine from "../engine.js";
import { T, bi, pick } from "../i18n.js";
import {
  buttonRow,
  card,
  clear,
  el,
  errorNote,
  fmt,
  pct,
  readout,
  slider,
  stepList,
  table,
} from "../ui.js";
import type { Lab } from "./registry.js";

/** Keadaan penggeser. */
interface State {
  prior: number;
  sensitivity: number;
  falsePositive: number;
  hypothesis: { id: string; en: string };
  evidence: { id: string; en: string };
}

const PRESETS: { label: { id: string; en: string }; state: State }[] = [
  {
    label: bi("Tugas Sesi 5 — deteksi hoaks", "Assignment 5 — hoax detection"),
    state: {
      prior: 0.2,
      sensitivity: 0.9,
      falsePositive: 0.3,
      hypothesis: bi("berita hoaks", "the article is a hoax"),
      evidence: bi("judul provokatif", "a provocative headline"),
    },
  },
  {
    label: bi("Tes medis penyakit langka", "Rare disease screening"),
    state: {
      prior: 0.001,
      sensitivity: 0.99,
      falsePositive: 0.05,
      hypothesis: bi("pasien sakit", "the patient is ill"),
      evidence: bi("hasil tes positif", "a positive test result"),
    },
  },
  {
    label: bi("Deteksi spam surel", "Email spam filter"),
    state: {
      prior: 0.4,
      sensitivity: 0.95,
      falsePositive: 0.02,
      hypothesis: bi("surel spam", "the email is spam"),
      evidence: bi("mengandung kata pemicu", "contains a trigger word"),
    },
  },
];

/**
 * Menggambar diagram frekuensi alami: 1.000 kasus sebagai kisi titik.
 *
 * Kanvas dilukis pada resolusi peranti agar tetap tajam di layar rapat, dan
 * hanya digambar ulang saat nilai berubah — bukan pada setiap bingkai.
 */
function drawFrequencyGrid(
  canvas: HTMLCanvasElement,
  prior: number,
  sensitivity: number,
  falsePositive: number,
): void {
  const cols = 40;
  const rows = 25;
  const total = cols * rows; // 1000 kasus
  const cell = 14;
  const gap = 3;
  const pad = 10;
  const cssW = cols * (cell + gap) - gap + pad * 2;
  const cssH = rows * (cell + gap) - gap + pad * 2;
  const dpr = Math.min(globalThis.devicePixelRatio || 1, 2);

  canvas.width = Math.round(cssW * dpr);
  canvas.height = Math.round(cssH * dpr);
  canvas.style.aspectRatio = `${cssW} / ${cssH}`;

  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, cssW, cssH);

  const style = getComputedStyle(document.documentElement);
  const accent = style.getPropertyValue("--accent").trim() || "#4dd4c8";
  const warn = style.getPropertyValue("--warn").trim() || "#f0b429";
  const faint = style.getPropertyValue("--border").trim() || "#1f2b3c";

  // Empat golongan, digambar berurutan supaya mudah dibaca:
  // positif benar, positif palsu, lalu sisanya yang tidak memicu bukti.
  const positiveTrue = Math.round(total * prior * sensitivity);
  const positiveFalse = Math.round(total * (1 - prior) * falsePositive);

  for (let i = 0; i < total; i++) {
    const x = pad + (i % cols) * (cell + gap);
    const y = pad + Math.floor(i / cols) * (cell + gap);
    ctx.fillStyle =
      i < positiveTrue ? accent : i < positiveTrue + positiveFalse ? warn : faint;
    ctx.beginPath();
    ctx.roundRect(x, y, cell, cell, 3);
    ctx.fill();
  }
}

export const bayesLab: Lab = {
  slug: "bayesian",
  session: 4,
  title: bi("Probabilitas Bayesian", "Bayesian Probability"),
  blurb: bi(
    "Teorema Bayes membalik arah pertanyaan: dari “seberapa sering gejala muncul pada yang sakit” menjadi “seberapa mungkin sakit bila gejalanya muncul”. Geser penggesernya dan perhatikan betapa jauh hasilnya dari tebakan intuitif.",
    "Bayes' theorem reverses the question: from “how often does the symptom appear in the ill” to “how likely is illness given the symptom”. Move the sliders and watch how far the answer drifts from intuition.",
  ),

  mount(root: HTMLElement): () => void {
    let state: State = { ...PRESETS[0].state };

    const controls = el("div");
    const output = el("div");
    const canvas = el("canvas", {
      attrs: {
        role: "img",
        "aria-label": pick(
          bi(
            "Diagram seribu kasus: hijau positif benar, kuning positif palsu, abu-abu tidak memicu bukti.",
            "A thousand-case diagram: green true positives, amber false positives, grey cases that do not trigger the evidence.",
          ),
        ),
      },
    });

    function recompute(): void {
      clear(output);
      let result: engine.BayesResult;
      try {
        result = engine.bayesBinary(
          state.prior,
          state.sensitivity,
          state.falsePositive,
        );
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }

      const h = pick(state.hypothesis);
      const e = pick(state.evidence);

      output.append(
        card(
          pick(T.result),
          readout(
            pick(bi(`P(${h} | ${e})`, `P(${h} | ${e})`)),
            pct(result.posterior, 2),
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                `Dari setiap 1.000 kasus, ${Math.round(1000 * result.evidence)} menunjukkan ${e}. Di antaranya, ${Math.round(1000 * state.prior * state.sensitivity)} memang ${h}.`,
                `Out of every 1,000 cases, ${Math.round(1000 * result.evidence)} show ${e}. Among those, ${Math.round(1000 * state.prior * state.sensitivity)} genuinely are ${h}.`,
              ),
            ),
          }),
        ),
        card(
          pick(bi("Ringkasan angka", "Numeric summary")),
          table(
            [pick(bi("Besaran", "Quantity")), pick(bi("Nilai", "Value"))],
            [
              [`P(H) — ${pick(bi("prior", "prior"))}`, state.prior],
              [`P(E|H) — ${pick(bi("deteksi benar", "true positive rate"))}`, state.sensitivity],
              [`P(E|~H) — ${pick(bi("positif palsu", "false positive rate"))}`, state.falsePositive],
              [`P(E) — ${pick(bi("probabilitas bukti", "evidence"))}`, result.evidence],
              [`P(H|E) — ${pick(bi("posterior", "posterior"))}`, result.posterior],
              [`P(~H|E)`, result.posterior_complement],
              [`LR+ — ${pick(bi("rasio kemungkinan", "likelihood ratio"))}`, result.likelihood_ratio],
            ],
          ),
        ),
        card(
          pick(T.steps),
          stepList(result.steps.map((s) => ({ label: s.label, formula: s.formula }))),
        ),
      );

      drawFrequencyGrid(
        canvas,
        state.prior,
        state.sensitivity,
        state.falsePositive,
      );
    }

    function renderControls(): void {
      clear(controls);
      controls.append(
        card(
          pick(T.preset),
          buttonRow(
            PRESETS.map((p) => ({
              label: pick(p.label),
              onClick: () => {
                state = { ...p.state };
                renderControls();
                recompute();
              },
            })),
          ),
        ),
        card(
          pick(T.controls),
          slider({
            label: `P(H) — ${pick(bi("seberapa sering hipotesis benar", "how often the hypothesis holds"))}`,
            min: 0,
            max: 1,
            step: 0.001,
            value: state.prior,
            format: (v) => pct(v, 1),
            onInput: (v) => {
              state.prior = v;
              recompute();
            },
          }),
          slider({
            label: `P(E|H) — ${pick(bi("bukti muncul saat hipotesis benar", "evidence appears when the hypothesis holds"))}`,
            min: 0,
            max: 1,
            step: 0.01,
            value: state.sensitivity,
            format: (v) => pct(v, 1),
            onInput: (v) => {
              state.sensitivity = v;
              recompute();
            },
          }),
          slider({
            label: `P(E|~H) — ${pick(bi("bukti muncul padahal hipotesis salah", "evidence appears when it does not"))}`,
            min: 0,
            max: 1,
            step: 0.01,
            value: state.falsePositive,
            format: (v) => pct(v, 1),
            onInput: (v) => {
              state.falsePositive = v;
              recompute();
            },
          }),
        ),
        card(pick(bi("Seribu kasus", "One thousand cases")), canvas),
      );
    }

    root.append(el("div", { class: "grid-2", children: [controls, output] }));
    renderControls();
    recompute();

    // Warna diambil dari token CSS, jadi diagram digambar ulang saat tema berganti.
    const observer = new MutationObserver(() => {
      drawFrequencyGrid(canvas, state.prior, state.sensitivity, state.falsePositive);
    });
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });

    return () => {
      observer.disconnect();
      clear(root);
    };
  },
};

/** Diekspor terpisah supaya bisa diuji tanpa DOM penuh. */
export const _internal = { fmt, drawFrequencyGrid };
