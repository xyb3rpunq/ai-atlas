/**
 * Laboratorium Sesi 3 — Certainty Factor.
 *
 * Pengguna menyusun sendiri daftar bukti, menggeser MB dan MD tiap bukti, lalu
 * melihat CF gabungan beserta seluruh langkah perhitungannya berubah seketika.
 */

import * as engine from "../engine.js";
import { T, bi, lang, pick } from "../i18n.js";
import {
  buttonRow,
  card,
  clear,
  el,
  errorNote,
  fmt,
  readout,
  slider,
  stepList,
  table,
} from "../ui.js";
import type { Lab } from "./registry.js";

/** Satu potong bukti yang dimasukkan pengguna. */
interface Evidence {
  name: string;
  mb: number;
  md: number;
}

/** Contoh kasus yang bisa dimuat sekali klik. */
interface Preset {
  label: { id: string; en: string };
  evidence: Evidence[];
}

const PRESETS: Preset[] = [
  {
    label: bi(
      "Tugas Sesi 3 — Cacar (MB 0,8 / MD 0,01)",
      "Assignment 3 — Chickenpox (MB 0.8 / MD 0.01)",
    ),
    evidence: [{ name: "Bintik-bintik", mb: 0.8, md: 0.01 }],
  },
  {
    label: bi(
      "Tugas Sesi 3 — Edema palpebra (MB 0,3 / MD 0)",
      "Assignment 3 — Palpebral edema (MB 0.3 / MD 0)",
    ),
    evidence: [{ name: "Peradangan mata", mb: 0.3, md: 0.0 }],
  },
  {
    label: bi("Tiga gejala menguatkan", "Three reinforcing symptoms"),
    evidence: [
      { name: "Demam", mb: 0.6, md: 0.1 },
      { name: "Batuk", mb: 0.5, md: 0.2 },
      { name: "Nyeri otot", mb: 0.4, md: 0.0 },
    ],
  },
  {
    label: bi("Bukti saling bertentangan", "Conflicting evidence"),
    evidence: [
      { name: "Gejala mendukung", mb: 0.8, md: 0.0 },
      { name: "Hasil lab menyangkal", mb: 0.0, md: 0.6 },
    ],
  },
];

/** Membuat salinan dalam agar preset tidak ikut berubah saat digeser. */
function clone(list: Evidence[]): Evidence[] {
  return list.map((e) => ({ ...e }));
}

export const certaintyLab: Lab = {
  slug: "certainty-factor",
  session: 3,
  title: bi("Certainty Factor", "Certainty Factor"),
  blurb: bi(
    "Cara MYCIN menakar keyakinan ketika buktinya tidak pasti. Setiap bukti punya ukuran kepercayaan (MB) dan ketidakpercayaan (MD); CF adalah selisihnya, lalu bukti-bukti digabungkan satu per satu.",
    "How MYCIN weighs belief when the evidence is uncertain. Each piece of evidence carries a measure of belief (MB) and disbelief (MD); CF is their difference, and pieces are then combined one at a time.",
  ),

  mount(root: HTMLElement): () => void {
    let evidence: Evidence[] = clone(PRESETS[2].evidence);

    const controls = el("div");
    const output = el("div");

    /** Menghitung ulang dan menggambar sisi hasil. */
    function recompute(): void {
      clear(output);
      if (evidence.length === 0) {
        output.append(
          errorNote(
            pick(
              bi(
                "Tambahkan minimal satu bukti untuk mulai menghitung.",
                "Add at least one piece of evidence to start.",
              ),
            ),
          ),
        );
        return;
      }

      let perEvidence: number[];
      try {
        perEvidence = evidence.map((e) => engine.cfFromMbMd(e.mb, e.md));
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }

      let combined: engine.CfResult;
      try {
        combined = engine.cfCombine(perEvidence);
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }

      const label = lang() === "id" ? combined.label_id : combined.label_en;
      const tone =
        combined.value >= 0.4 ? "" : combined.value <= -0.4 ? "danger" : "warn";

      output.append(
        card(
          pick(T.result),
          readout(
            `${pick(T.interpretation)}: ${label}`,
            fmt(combined.value, 4),
          ),
          el("div", {
            class: "note",
            text: pick(
              bi(
                "CF berada di rentang -1 (pasti tidak) sampai +1 (pasti ya). Nilai di sekitar nol berarti bukti yang ada belum memutuskan apa pun.",
                "CF ranges from -1 (definitely not) to +1 (definitely yes). Values near zero mean the evidence has not decided anything yet.",
              ),
            ),
          }),
        ),
        card(
          pick(bi("CF tiap bukti", "CF per piece of evidence")),
          table(
            [
              pick(T.evidence),
              "MB",
              "MD",
              "CF = MB - MD",
            ],
            evidence.map((e, i) => [e.name, e.mb, e.md, perEvidence[i]]),
          ),
        ),
        card(
          pick(T.steps),
          stepList(
            combined.steps.map((s) => ({
              label: s.op === "init" ? "CF0" : "",
              formula: s.formula,
            })),
          ),
        ),
      );
      // `tone` dipakai untuk mewarnai bilah ringkas di bawah angka utama.
      const summary = el("div", { class: "bar", children: [] });
      const fill = el("div", {
        class: tone ? `bar__fill bar__fill--${tone}` : "bar__fill",
      });
      fill.style.width = `${((combined.value + 1) / 2) * 100}%`;
      summary.append(fill);
      output.firstElementChild?.append(summary);
    }

    /** Menggambar ulang panel kontrol dari keadaan saat ini. */
    function renderControls(): void {
      clear(controls);

      const presetButtons = buttonRow(
        PRESETS.map((p) => ({
          label: pick(p.label),
          onClick: () => {
            evidence = clone(p.evidence);
            renderControls();
            recompute();
          },
        })),
      );

      const rows = evidence.map((item, index) =>
        el("div", {
          class: "card",
          children: [
            el("div", {
              class: "field",
              children: [
                el("span", { class: "field__label", text: pick(T.evidence) }),
                el("input", {
                  attrs: {
                    type: "text",
                    value: item.name,
                    "aria-label": pick(T.evidence),
                    maxlength: "40",
                  },
                  on: {
                    input: (event) => {
                      item.name = (event.target as HTMLInputElement).value;
                      recompute();
                    },
                  },
                }),
              ],
            }),
            slider({
              label: `MB — ${pick(bi("ukuran kepercayaan", "measure of belief"))}`,
              min: 0,
              max: 1,
              step: 0.01,
              value: item.mb,
              onInput: (v) => {
                item.mb = v;
                recompute();
              },
            }),
            slider({
              label: `MD — ${pick(bi("ukuran ketidakpercayaan", "measure of disbelief"))}`,
              min: 0,
              max: 1,
              step: 0.01,
              value: item.md,
              onInput: (v) => {
                item.md = v;
                recompute();
              },
            }),
            buttonRow([
              {
                label: pick(T.removeEvidence),
                onClick: () => {
                  evidence.splice(index, 1);
                  renderControls();
                  recompute();
                },
              },
            ]),
          ],
        }),
      );

      controls.append(
        card(pick(T.preset), presetButtons),
        ...rows,
        buttonRow([
          {
            label: pick(T.addEvidence),
            primary: true,
            onClick: () => {
              evidence.push({
                name: `${pick(T.evidence)} ${evidence.length + 1}`,
                mb: 0.5,
                md: 0.1,
              });
              renderControls();
              recompute();
            },
          },
        ]),
      );
    }

    root.append(
      el("div", {
        class: "grid-2",
        children: [controls, output],
      }),
    );

    renderControls();
    recompute();

    return () => {
      clear(root);
    };
  },
};
