/**
 * Laboratorium Sesi 11 — Sistem Pakar.
 *
 * Pengguna menyalakan gejala dengan tingkat keyakinan, lalu melihat dua arah
 * penalaran bekerja pada basis pengetahuan yang sama:
 *
 * - **Runut maju** memaparkan setiap aturan yang menyala, berurut, lengkap
 *   dengan dukungan dan keyakinan yang dihasilkan.
 * - **Runut mundur** menelusuri satu hipotesis dan menunjukkan pohon
 *   pembuktiannya, termasuk pertanyaan mana yang sebenarnya masih perlu
 *   diajukan — dan mana yang tidak, karena tidak akan mengubah jawaban.
 *
 * Fasilitas penjelasan bukan hiasan. Sistem pakar yang tidak bisa menjawab
 * "kenapa" hanyalah tebakan bercangkang komputer.
 */

import * as engine from "../engine.js";
import { T, bi, pick } from "../i18n.js";
import { buttonRow, card, clear, el, errorNote, fmt, slider, table } from "../ui.js";
import type { Lab } from "./registry.js";

/** Ambang agar sebuah fakta dianggap kesimpulan yang layak ditampilkan. */
const THRESHOLD = 0.2;

/** Contoh kasus siap pakai. */
const PRESETS: { label: { id: string; en: string }; facts: Record<string, number> }[] = [
  {
    label: bi("Flu khas", "Classic influenza"),
    facts: { demam: 1, pilek: 1, batuk: 1 },
  },
  {
    label: bi("Alergi, bukan flu", "Allergy, not influenza"),
    facts: { pilek: 1, demam: -1, "bersin berulang": 1 },
  },
  {
    label: bi("Perlu dirujuk", "Needs referral"),
    facts: { demam: 1, pilek: 1, batuk: 1, "sesak napas": 1 },
  },
  {
    label: bi("Gejala meragukan", "Uncertain symptoms"),
    facts: { demam: 0.5, pilek: 0.6, batuk: 0.4 },
  },
];

export const expertLab: Lab = {
  slug: "expert-system",
  session: 11,
  title: bi("Sistem Pakar", "Expert Systems"),
  blurb: bi(
    "Basis pengetahuan “Dokter Virtual” dari studi kasus modul, dijalankan dua arah. Runut maju bertanya “apa yang bisa disimpulkan”; runut mundur bertanya “benarkah dugaan ini, dan gejala mana yang masih perlu saya tanyakan”.",
    "The “Virtual Doctor” knowledge base from the course case study, run in both directions. Forward chaining asks “what follows”; backward chaining asks “is this hypothesis true, and which symptoms do I still need to ask about”.",
  ),

  mount(root: HTMLElement): () => void {
    const kb = engine.expertSampleKb();
    const inspection = engine.expertInspectKb(kb);
    let facts: Record<string, number> = { ...PRESETS[0].facts };
    let goal = "flu";
    let whyRule: string | null = null;

    const controls = el("div");
    const output = el("div");

    /** Memori kerja dalam bentuk yang diterima mesin. */
    function factPairs(): [string, number][] {
      return Object.entries(facts).filter(([, cf]) => Math.abs(cf) > 1e-9);
    }

    function render(): void {
      clear(output);
      const pairs = factPairs();

      let forward: engine.ForwardResult;
      try {
        forward = engine.expertForward(kb, pairs, THRESHOLD);
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }

      output.append(
        card(
          pick(bi("Runut maju — kesimpulan", "Forward chaining — conclusions")),
          forward.derived.length > 0
            ? table(
                [pick(bi("Kesimpulan", "Conclusion")), pick(bi("Keyakinan", "Certainty"))],
                forward.derived.map(([f, cf]) => [f, cf]),
              )
            : el("p", {
                class: "note",
                text: pick(
                  bi(
                    "Belum ada kesimpulan. Nyalakan beberapa gejala di sebelah kiri.",
                    "No conclusions yet. Switch on a few symptoms on the left.",
                  ),
                ),
              }),
          el("p", {
            class: "note",
            text: pick(
              bi(
                `Basis aturan disapu ${forward.passes} kali sampai tidak ada lagi yang berubah. ${forward.given.length} fakta berasal dari masukan Anda dan tidak dihitung sebagai kesimpulan.`,
                `The rule base was swept ${forward.passes} times until nothing changed. ${forward.given.length} facts came from your input and do not count as conclusions.`,
              ),
            ),
          }),
        ),
      );

      if (forward.steps.length > 0) {
        output.append(
          card(
            pick(bi("Jejak penalaran", "Inference trace")),
            table(
              [
                "#",
                pick(bi("Aturan", "Rule")),
                pick(bi("Dukungan", "Support")),
                pick(bi("Menghasilkan", "Concludes")),
                "CF",
              ],
              forward.steps.map((s) => [
                String(s.order),
                s.text,
                s.support.map(([f, cf]) => `${f} ${fmt(cf, 2)}`).join(" · "),
                s.conclusion,
                s.conclusion_certainty,
              ]),
            ),
            buttonRow(
              forward.steps.map((s) => ({
                label: `${pick(bi("Kenapa", "Why"))} ${s.rule_id}`,
                onClick: () => {
                  whyRule = s.rule_id;
                  render();
                },
              })),
            ),
            whyRule
              ? el("p", {
                  class: "note",
                  text: (() => {
                    try {
                      return engine.expertWhy(kb, whyRule);
                    } catch {
                      return "";
                    }
                  })(),
                })
              : null,
          ),
        );
      }

      // Runut mundur terhadap hipotesis terpilih.
      let backward: engine.BackwardResult;
      try {
        backward = engine.expertBackward(kb, pairs, goal);
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }

      const outcomeLabel = (o: engine.ProofOutcome): string => {
        switch (o.kind) {
          case "known":
            return pick(bi("sudah diketahui", "already known"));
          case "needs_asking":
            return pick(bi("perlu ditanyakan", "needs asking"));
          case "unprovable":
            return pick(bi("tak terbuktikan", "unprovable"));
          case "derived":
            return `${pick(bi("dari aturan", "from rule"))} ${o.rule_id}`;
        }
      };

      /** Meratakan pohon pembuktian menjadi baris-baris tabel. */
      const flatten = (
        node: engine.ProofNode,
        rows: (string | number)[][] = [],
      ): (string | number)[][] => {
        const indent = "· ".repeat(node.depth);
        rows.push([`${indent}${node.goal}`, outcomeLabel(node.outcome), node.certainty]);
        for (const child of node.children) flatten(child, rows);
        return rows;
      };

      output.append(
        card(
          pick(bi("Runut mundur — pohon pembuktian", "Backward chaining — proof tree")),
          buttonRow(
            inspection.derivable.map((f) => ({
              label: f,
              primary: f === goal,
              onClick: () => {
                goal = f;
                render();
              },
            })),
          ),
          table(
            [
              pick(bi("Tujuan / premis", "Goal / premise")),
              pick(bi("Asal", "Source")),
              "CF",
            ],
            flatten(backward.proof),
          ),
          backward.questions.length > 0
            ? el("p", {
                class: "note",
                text: pick(
                  bi(
                    `Masih perlu ditanyakan: ${backward.questions.join(", ")}. Dari ${inspection.askable.length} gejala yang bisa ditanyakan, hanya ini yang relevan — dan yang tidak akan mengubah jawaban sudah dipangkas.`,
                    `Still needs asking: ${backward.questions.join(", ")}. Of ${inspection.askable.length} askable symptoms, only these are relevant — those that cannot change the answer are already pruned.`,
                  ),
                ),
              })
            : el("p", {
                class: "note",
                text: pick(
                  bi(
                    "Tidak ada lagi yang perlu ditanyakan; seluruh premis yang menentukan sudah diketahui.",
                    "Nothing more needs asking; every deciding premise is already known.",
                  ),
                ),
              }),
        ),
        card(
          pick(bi("Kesehatan basis pengetahuan", "Knowledge base health")),
          table(
            [pick(bi("Ukuran", "Measure")), pick(bi("Nilai", "Value"))],
            [
              [pick(bi("Jumlah aturan", "Rules")), String(inspection.rules)],
              [
                pick(bi("Bisa disimpulkan", "Derivable")),
                inspection.derivable.join(", "),
              ],
              [
                pick(bi("Harus ditanyakan", "Leaf facts")),
                inspection.leaf_facts.join(", "),
              ],
              [
                pick(bi("Tak terjangkau", "Unreachable")),
                inspection.unreachable_facts.length > 0
                  ? inspection.unreachable_facts.join(", ")
                  : pick(bi("tidak ada", "none")),
              ],
            ],
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Fakta “tak terjangkau” adalah premis yang tidak bisa disimpulkan maupun ditanyakan. Fakta seperti itu diam-diam dianggap tidak berlaku, sehingga sebagian aturan tidak akan pernah menyala tanpa pesan galat apa pun — jenis cacat yang paling sulit ditemukan pada sistem pakar.",
                "An “unreachable” fact is a premise that can neither be derived nor asked. Such facts are silently treated as false, so some rules never fire and nothing reports an error — the hardest kind of defect to find in an expert system.",
              ),
            ),
          }),
        ),
      );
    }

    function renderControls(): void {
      clear(controls);

      const symptomCards = inspection.askable.map((symptom) =>
        slider({
          label: symptom,
          min: -1,
          max: 1,
          step: 0.1,
          value: facts[symptom] ?? 0,
          format: (v) => {
            if (Math.abs(v) < 1e-9) return pick(bi("tidak tahu", "unknown"));
            if (v > 0) return `${pick(bi("ya", "yes"))} ${fmt(v, 1)}`;
            return `${pick(bi("tidak", "no"))} ${fmt(-v, 1)}`;
          },
          onInput: (v) => {
            facts[symptom] = v;
            render();
          },
        }),
      );

      controls.append(
        card(
          pick(T.preset),
          buttonRow(
            PRESETS.map((p) => ({
              label: pick(p.label),
              onClick: () => {
                facts = { ...p.facts };
                whyRule = null;
                renderControls();
                render();
              },
            })),
          ),
        ),
        card(
          pick(bi("Gejala", "Symptoms")),
          ...symptomCards,
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Penggeser di tengah berarti “tidak tahu”, ke kiri berarti gejalanya tidak ada. Menjawab “tidak” berbeda dari tidak menjawab — dan sistem pakar yang baik memperlakukan keduanya berbeda.",
                "The middle means “unknown”, the left means the symptom is absent. Answering “no” is not the same as not answering — and a good expert system treats them differently.",
              ),
            ),
          }),
          buttonRow([
            {
              label: pick(T.reset),
              onClick: () => {
                facts = {};
                whyRule = null;
                renderControls();
                render();
              },
            },
          ]),
        ),
      );
    }

    root.append(el("div", { class: "grid-2", children: [controls, output] }));
    renderControls();
    render();

    return () => {
      clear(root);
    };
  },
};
