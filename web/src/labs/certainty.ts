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
import { dataDetails, figure, numberLine, waterfall } from "../viz.js";
import type { Band } from "../viz.js";

/**
 * Pita tafsir CF, sama persis dengan `certainty::interpret` di Rust.
 *
 * Batas-batasnya disalin, bukan dihitung ulang di sini: kalau suatu hari
 * batasnya berubah di mesin, gambarnya harus ikut berubah, dan menyalinnya
 * membuat perbedaan itu terlihat pada satu tempat.
 */
const PITA: Band[] = [
  { from: -1, to: -0.8, label: bi("pasti tidak", "definitely not"), color: "var(--danger)" },
  { from: -0.8, to: -0.4, label: bi("hampir pasti tidak", "almost certainly not"), color: "var(--danger)" },
  { from: -0.4, to: -0.2, label: bi("mungkin tidak", "probably not"), color: "var(--warn)" },
  { from: -0.2, to: 0.2, label: bi("tidak diketahui", "unknown"), color: "var(--text-faint)" },
  { from: 0.2, to: 0.4, label: bi("mungkin", "maybe"), color: "var(--warn)" },
  { from: 0.4, to: 0.8, label: bi("hampir pasti", "almost certainly"), color: "var(--ok)" },
  { from: 0.8, to: 1, label: bi("pasti", "definitely"), color: "var(--ok)" },
];

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

/**
 * Memasang laboratorium ke dalam elemen yang diberikan.
 *
 * Keterangannya -- judul, nomor sesi, penjelasan -- ada di
 * `labs/registry.ts`, bukan di sini, supaya daftar isi bisa ditampilkan
 * tanpa mengunduh mesin seluruh laboratorium lebih dulu.
 */
export function mount(root: HTMLElement): () => void {
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

      const panel: (Node | null)[] = [
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
          pick(bi("Letak kesimpulan", "Where the conclusion sits")),
          figure({
            title: bi("Garis keyakinan", "Belief line"),
            summary: bi(
              `Jarum berada di ${fmt(combined.value, 3)}, yaitu pita "${combined.label_id}". ` +
                `Lingkaran kecil menandai CF tiap bukti sebelum digabung, sehingga terlihat ` +
                `apakah kesimpulan akhirnya lebih kuat daripada bukti terkuatnya sendiri.`,
              `The needle sits at ${fmt(combined.value, 3)}, inside the "${combined.label_en}" band. ` +
                `Small circles mark each piece of evidence before combining, so you can see ` +
                `whether the conclusion ends up stronger than its strongest single piece.`,
            ),
            body: numberLine({
              min: -1,
              max: 1,
              value: combined.value,
              bands: PITA,
              marks: perEvidence.map((v, i) => ({
                value: v,
                label: evidence[i]?.name.slice(0, 10) ?? "",
              })),
            }),
            legend: [
              { color: "var(--accent)", label: bi("CF gabungan", "combined CF") },
              { color: "var(--text-muted)", label: bi("CF tiap bukti", "per-evidence CF") },
            ],
          }),
        ),
        // Air terjun hanya bermakna kalau ada lebih dari satu bukti; dengan
        // satu bukti ia cuma menggambar ulang angka yang sudah terbaca di atas.
        combined.steps.length > 1
          ? card(
              pick(bi("Jalannya penggabungan", "How the combination unfolds")),
              figure({
                title: bi("Air terjun bukti", "Evidence waterfall"),
                summary: bi(
                  "Tiap batang berangkat dari hasil batang sebelumnya, bukan dari nol. " +
                    "Batang hijau menaikkan keyakinan, batang merah menurunkannya. " +
                    "Perhatikan bahwa bukti yang datang belakangan menggeser lebih sedikit: " +
                    "makin yakin sebuah kesimpulan, makin sulit digeser bukti tambahan.",
                  "Each bar starts from the previous result, not from zero. Green bars raise " +
                    "the belief, red bars lower it. Notice that later evidence moves the needle " +
                    "less: the more certain a conclusion already is, the harder extra evidence " +
                    "can shift it.",
                ),
                body: waterfall(
                  combined.steps.map((s, i) => ({
                    label: i === 0 ? (evidence[0]?.name ?? "CF0") : (evidence[i]?.name ?? `CF${i}`),
                    value: s.value,
                  })),
                  -1,
                  1,
                ),
                legend: [
                  { color: "var(--accent)", label: bi("menguatkan", "reinforces") },
                  { color: "var(--danger)", label: bi("melemahkan", "weakens") },
                ],
              }),
              dataDetails(
                [pick(T.evidence), pick(bi("CF setelah langkah ini", "CF after this step"))],
                combined.steps.map((s, i) => [
                  evidence[i]?.name ?? `CF${i}`,
                  s.value,
                ]),
              ),
            )
          : null,
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
      ];
      output.append(...panel.filter((n): n is Node => n !== null));
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
}
