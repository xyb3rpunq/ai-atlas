/**
 * Laboratorium Sesi 7 — Representasi Pengetahuan.
 *
 * Tiga bagian yang menjawab pertanyaan berbeda tentang pengetahuan yang sama:
 *
 * - **Tabel kebenaran** menjawab "apakah benar" dengan mencoba seluruh
 *   kemungkinan. Jujur, tetapi jumlah barisnya berlipat dua tiap proposisi
 *   ditambahkan.
 * - **Resolusi** menjawab pertanyaan yang sama dengan *membuktikan*: ia
 *   menyangkal kesimpulan lalu mencari kontradiksi, tanpa perlu menyentuh
 *   sebagian besar kemungkinan.
 * - **Jaringan semantik** menyimpan pengetahuan sebagai hubungan, sehingga
 *   sifat yang ditulis sekali berlaku untuk seluruh turunannya.
 */

import * as engine from "../engine.js";
import { T, bi, pick } from "../i18n.js";
import { buttonRow, card, clear, el, errorNote, table } from "../ui.js";
import { figure, heatmap, nodeGraph } from "../viz.js";
import type { GraphEdge, GraphNode } from "../viz.js";

/** Batas kedalaman pewarisan yang digambar. */
const MAX_LAPIS = 5;

/**
 * Menyusun graf jaringan semantik dengan simpul terpilih sebagai jangkar.
 *
 * Lapisan dihitung dari relasi `adalah`: leluhur diletakkan di atas, keturunan
 * di bawah. Relasi sifat — `punya`, `bisa`, dan sejenisnya — sengaja tidak ikut
 * menentukan lapisan, karena sifat bukan hubungan pewarisan dan memasukkannya
 * membuat gambar yang seharusnya berupa silsilah berubah menjadi kusut.
 */
function grafSemantik(view: engine.SemanticNetworkView): {
  nodes: GraphNode[];
  edges: GraphEdge[];
} {
  const naik = new Map<string, string[]>();
  for (const r of view.relations) {
    if (r.label !== "adalah") continue;
    const daftar = naik.get(r.from) ?? [];
    daftar.push(r.to);
    naik.set(r.from, daftar);
  }

  /** Berapa langkah `adalah` dari sebuah simpul ke akar terjauhnya. */
  function tinggi(simpul: string, sisa: number): number {
    if (sisa <= 0) return 0;
    const induk = naik.get(simpul);
    if (!induk || induk.length === 0) return 0;
    return 1 + Math.max(...induk.map((p) => tinggi(p, sisa - 1)));
  }

  const terpakai = new Set<string>([view.selected, ...view.ancestors]);
  for (const r of view.relations) {
    if (r.label === "adalah" && (r.to === view.selected || r.from === view.selected)) {
      terpakai.add(r.from);
      terpakai.add(r.to);
    }
  }
  // Sifat yang diwarisi ikut digambar, karena justru dari sanalah terlihat
  // bahwa sebuah sifat datang dari leluhur dan bukan dari simpulnya sendiri.
  for (const r of view.properties) terpakai.add(r.from);

  const tinggiMaks = Math.max(...[...terpakai].map((n) => tinggi(n, MAX_LAPIS)), 0);
  const nodes: GraphNode[] = [...terpakai].sort().map((n) => ({
    id: n,
    label: n,
    layer: tinggiMaks - tinggi(n, MAX_LAPIS),
    tone:
      n === view.selected ? "tujuan" : view.ancestors.includes(n) ? "aktif" : "netral",
  }));

  const edges: GraphEdge[] = view.relations
    .filter((r) => r.label === "adalah" && terpakai.has(r.from) && terpakai.has(r.to))
    .map((r) => ({
      from: r.to,
      to: r.from,
      label: r.label,
      active: r.from === view.selected || view.ancestors.includes(r.from),
    }));

  return { nodes, edges };
}

type Tab = "truth" | "resolution" | "network";

const FORMULA_PRESETS: { label: { id: string; en: string }; formula: string }[] = [
  { label: bi("Modus ponens", "Modus ponens"), formula: "((P -> Q) & P) -> Q" },
  { label: bi("Modus tollens", "Modus tollens"), formula: "((P -> Q) & ~Q) -> ~P" },
  { label: bi("De Morgan", "De Morgan"), formula: "~(P & Q) <-> (~P | ~Q)" },
  { label: bi("Kontradiksi", "Contradiction"), formula: "P & ~P" },
  { label: bi("Silogisme", "Syllogism"), formula: "((P -> Q) & (Q -> R)) -> (P -> R)" },
];

const PROOF_PRESETS: {
  label: { id: string; en: string };
  knowledge: string[];
  conclusion: string;
}[] = [
  {
    label: bi("Modus ponens", "Modus ponens"),
    knowledge: ["P -> Q", "P"],
    conclusion: "Q",
  },
  {
    label: bi("Rantai implikasi", "Implication chain"),
    knowledge: ["P -> Q", "Q -> R", "R -> S", "P"],
    conclusion: "S",
  },
  {
    label: bi("Modus tollens", "Modus tollens"),
    knowledge: ["P -> Q", "~Q"],
    conclusion: "~P",
  },
  {
    label: bi("Kekeliruan menegaskan konsekuen", "Affirming the consequent"),
    knowledge: ["P -> Q", "Q"],
    conclusion: "P",
  },
];

/**
 * Memasang laboratorium ke dalam elemen yang diberikan.
 *
 * Keterangannya -- judul, nomor sesi, penjelasan -- ada di
 * `labs/registry.ts`, bukan di sini, supaya daftar isi bisa ditampilkan
 * tanpa mengunduh mesin seluruh laboratorium lebih dulu.
 */
export function mount(root: HTMLElement): () => void {
    let tab: Tab = "truth";
    let formula = FORMULA_PRESETS[0].formula;
    let compareWith = "~P | Q";
    let knowledge = [...PROOF_PRESETS[0].knowledge];
    let conclusion = PROOF_PRESETS[0].conclusion;
    let node = "pinguin";

    const controls = el("div");
    const output = el("div");

    function renderTruth(): void {
      let result: engine.TruthTable;
      try {
        result = engine.logicTruthTable(formula);
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }

      const verdict = result.tautology
        ? pick(bi("tautologi — benar pada semua baris", "tautology — true on every row"))
        : result.contradiction
          ? pick(bi("kontradiksi — salah pada semua baris", "contradiction — false on every row"))
          : pick(bi("kontingen — kadang benar, kadang salah", "contingent — sometimes true"));

      let equivalent: boolean | null = null;
      try {
        equivalent = engine.logicEquivalent(formula, compareWith);
      } catch {
        equivalent = null;
      }

      output.append(
        card(
          pick(bi("Seluruh dunia yang mungkin", "Every possible world")),
          figure({
            title: bi("Peta tabel kebenaran", "Truth table map"),
            summary: bi(
              `${result.rows.length} baris adalah seluruh dunia yang mungkin bagi ` +
                `${result.variables.length} proposisi. Kolom terakhir adalah nilai rumusnya. ` +
                `Rumus ini ${verdict}. Perhatikan jumlah barisnya: tiap proposisi baru ` +
                `melipatduakannya, sehingga sepuluh proposisi sudah berarti 1.024 baris — ` +
                `itulah sebabnya pembuktian yang tidak perlu memeriksa semua baris jauh lebih ` +
                `berharga daripada tabel ini.`,
              `${result.rows.length} rows are every possible world for ${result.variables.length} ` +
                `propositions. The last column is the formula's value. This formula is ${verdict}. ` +
                `Note the row count: each new proposition doubles it, so ten propositions already ` +
                `mean 1,024 rows — which is why a proof that need not check every row is worth ` +
                `far more than this table.`,
            ),
            body: heatmap({
              rows: result.rows.map((_, i) => String(i + 1)),
              cols: [...result.variables, pick(bi("hasil", "result"))],
              values: result.rows.map((r) => [
                ...r.values.map((v) => (v ? 1 : 0)),
                r.result ? 1 : 0,
              ]),
              format: (v) => (v === 1 ? "B" : "S"),
            }),
            legend: [
              { color: "var(--accent)", label: bi("benar", "true") },
              { color: "var(--surface-2)", label: bi("salah", "false") },
            ],
          }),
        ),
        card(
          pick(T.result),
          table(
            [pick(bi("Ukuran", "Measure")), pick(bi("Nilai", "Value"))],
            [
              [pick(bi("Rumus terbaca", "Parsed as")), result.text],
              [pick(bi("Proposisi", "Propositions")), result.variables.join(", ")],
              [pick(bi("Baris tabel", "Table rows")), String(result.rows.length)],
              [pick(bi("Kesimpulan", "Verdict")), verdict],
              [
                pick(bi("Setara dengan pembanding", "Equivalent to comparison")),
                equivalent === null
                  ? "—"
                  : equivalent
                    ? pick(bi("ya", "yes"))
                    : pick(bi("tidak", "no")),
              ],
            ],
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                `${result.variables.length} proposisi menghasilkan ${result.rows.length} baris. Setiap proposisi tambahan melipatduakannya — enam belas proposisi sudah berarti 65.536 baris, dan di situlah tabel kebenaran berhenti berguna.`,
                `${result.variables.length} propositions produce ${result.rows.length} rows. Each additional proposition doubles that — sixteen propositions already means 65,536 rows, and that is where truth tables stop being useful.`,
              ),
            ),
          }),
        ),
        card(
          pick(bi("Tabel kebenaran", "Truth table")),
          table(
            [...result.variables, result.text],
            result.rows.map((r) => [
              ...r.values.map((v) => (v ? "T" : "F")),
              r.result ? "T" : "F",
            ]),
          ),
        ),
        card(
          pick(bi("Bentuk normal konjungtif", "Conjunctive normal form")),
          table(
            [pick(bi("Klausa", "Clause"))],
            result.cnf.map((c) => [c]),
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Bentuk ini setara dengan rumus aslinya — ada uji yang membandingkan tabel kebenaran keduanya baris demi baris. Resolusi hanya bekerja pada bentuk ini.",
                "This form is equivalent to the original — a test compares both truth tables row by row. Resolution only works on this form.",
              ),
            ),
          }),
        ),
      );
    }

    function renderResolution(): void {
      let proof: engine.ResolutionProof;
      try {
        proof = engine.logicResolve(knowledge, conclusion);
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }

      output.append(
        card(
          pick(T.result),
          table(
            [pick(bi("Ukuran", "Measure")), pick(bi("Nilai", "Value"))],
            [
              [
                pick(bi("Kesimpulan terbukti", "Conclusion proved")),
                proof.proved ? pick(bi("ya", "yes")) : pick(bi("tidak", "no")),
              ],
              [pick(bi("Klausa awal", "Initial clauses")), String(proof.initial_clauses.length)],
              [pick(bi("Langkah resolusi", "Resolution steps")), String(proof.steps.length)],
              [pick(bi("Klausa dihasilkan", "Clauses generated")), String(proof.generated)],
            ],
          ),
          el("p", {
            class: "note",
            text: proof.proved
              ? pick(
                  bi(
                    "Klausa kosong ditemukan. Ingkaran kesimpulan mustahil benar, jadi kesimpulannya pasti mengikuti dari basis pengetahuan.",
                    "The empty clause was derived. The negated conclusion cannot hold, so the conclusion must follow from the knowledge base.",
                  ),
                )
              : pick(
                  bi(
                    "Tidak ada lagi klausa baru yang bisa dihasilkan, dan klausa kosong tidak pernah muncul. Kesimpulan ini tidak mengikuti dari basis pengetahuannya — bukan berarti salah, hanya tidak terbuktikan dari sini.",
                    "No new clauses can be produced and the empty clause never appeared. This conclusion does not follow from the knowledge base — not that it is false, only that it is not provable from here.",
                  ),
                ),
          }),
        ),
        card(
          pick(bi("Jalannya pembuktian", "How the proof unfolds")),
          figure({
            title: bi("Pohon resolusi", "Resolution tree"),
            summary: bi(
              proof.proved
                ? `Klausa awal ada di baris atas. Tiap panah adalah satu langkah resolusi: ` +
                  `dua klausa bertemu, proposisi yang berlawanan tanda saling meniadakan, dan ` +
                  `sisanya menjadi klausa baru. Pembuktian berakhir ketika yang tersisa tidak ` +
                  `ada sama sekali — klausa kosong, yang berarti ingkaran kesimpulan mustahil benar.`
                : `Klausa awal ada di baris atas, dan ${proof.steps.length} langkah resolusi ` +
                  `dijalankan tanpa pernah menghasilkan klausa kosong. Pohon yang berhenti tanpa ` +
                  `klausa kosong bukan berarti kesimpulannya salah, hanya berarti ia tidak bisa ` +
                  `dibuktikan dari basis pengetahuan ini.`,
              proof.proved
                ? `Initial clauses sit on the top row. Each arrow is one resolution step: two ` +
                  `clauses meet, oppositely signed propositions cancel, and what remains becomes a ` +
                  `new clause. The proof ends when nothing remains at all — the empty clause, ` +
                  `meaning the negated conclusion cannot possibly hold.`
                : `Initial clauses sit on the top row, and ${proof.steps.length} resolution steps ` +
                  `ran without ever producing the empty clause. A tree that stops without it does ` +
                  `not mean the conclusion is false, only that it cannot be proved from this ` +
                  `knowledge base.`,
            ),
            body: nodeGraph(
              [
                ...proof.initial_clauses.map((c, i) => ({
                  id: `k${i}`,
                  label: c,
                  layer: 0,
                })),
                ...proof.steps.map((s) => ({
                  id: s.result,
                  // Klausa kosong tidak punya teks, dan kotak tanpa isi tidak
                  // memberi tahu apa pun. Ia diberi nama agar terlihat bahwa
                  // justru simpul inilah tujuan seluruh pembuktiannya.
                  label: s.result === "" ? "□" : s.result,
                  layer: Math.min(4, s.order),
                  detail: s.result === "" ? pick(bi("klausa kosong", "empty clause")) : `~${s.pivot}`,
                  tone: s.result === "" ? ("tujuan" as const) : ("aktif" as const),
                })),
              ],
              proof.steps.flatMap((s) => {
                const cari = (teks: string): string => {
                  const i = proof.initial_clauses.indexOf(teks);
                  return i >= 0 ? `k${i}` : teks;
                };
                return [
                  { from: cari(s.left), to: s.result, label: s.pivot, active: true },
                  { from: cari(s.right), to: s.result, active: true },
                ];
              }),
            ),
            legend: [
              { color: "var(--accent)", label: bi("klausa hasil resolusi", "resolved clause") },
              { color: "var(--surface-2)", label: bi("klausa awal", "initial clause") },
            ],
          }),
        ),
        card(
          pick(bi("Klausa awal", "Initial clauses")),
          table(
            ["#", pick(bi("Klausa", "Clause"))],
            proof.initial_clauses.map((c, i) => [String(i + 1), c]),
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Termasuk ingkaran kesimpulan. Resolusi tidak membuktikan secara langsung; ia menyangkal apa yang ingin dibuktikan lalu menunjukkan bahwa penyangkalan itu mustahil.",
                "This includes the negated conclusion. Resolution does not prove directly; it denies what you want to prove and then shows the denial is impossible.",
              ),
            ),
          }),
        ),
      );

      if (proof.steps.length > 0) {
        output.append(
          card(
            pick(bi("Jejak pembuktian", "Proof trace")),
            table(
              [
                "#",
                pick(bi("Klausa A", "Clause A")),
                pick(bi("Klausa B", "Clause B")),
                pick(bi("Dihapuskan", "Pivot")),
                pick(bi("Hasil", "Result")),
              ],
              proof.steps.map((s) => [String(s.order), s.left, s.right, s.pivot, s.result]),
            ),
          ),
        );
      }
    }

    function renderNetwork(): void {
      let view: engine.SemanticNetworkView;
      try {
        view = engine.logicSemanticNetwork(node);
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }

      const langsung = view.properties.filter((r) => r.from === view.selected);
      const warisan = view.properties.filter((r) => r.from !== view.selected);
      const graf = grafSemantik(view);

      output.append(
        card(
          pick(bi("Silsilah pewarisan", "Inheritance chain")),
          figure({
            title: bi("Jaringan semantik", "Semantic network"),
            summary: bi(
              `Panah menunjuk dari yang umum ke yang khusus. Simpul bertepi tebal adalah ` +
                `"${view.selected}" beserta ${view.ancestors.length} leluhurnya, yaitu jalur ` +
                `yang dilalui pewarisan sifat. Dari ${langsung.length + warisan.length} sifat ` +
                `yang dimiliki "${view.selected}", hanya ${langsung.length} yang benar-benar ` +
                `dituliskan padanya; sisanya datang menuruni panah-panah ini.`,
              `Arrows point from the general to the specific. Thick-edged nodes are ` +
                `"${view.selected}" and its ${view.ancestors.length} ancestors — the path ` +
                `inheritance travels. Of the ${langsung.length + warisan.length} properties ` +
                `"${view.selected}" has, only ${langsung.length} are actually written on it; ` +
                `the rest arrive down these arrows.`,
            ),
            body: nodeGraph(graf.nodes, graf.edges),
            legend: [
              { color: "var(--accent)", label: bi("jalur pewarisan", "inheritance path") },
              { color: "var(--surface-2)", label: bi("simpul lain", "other nodes") },
            ],
          }),
        ),
        card(
          `${pick(bi("Sifat", "Properties"))}: ${view.selected}`,
          table(
            [
              pick(bi("Asal", "From")),
              pick(bi("Relasi", "Relation")),
              pick(bi("Tujuan", "To")),
              pick(bi("Sumber", "Source")),
            ],
            [
              ...langsung.map((r) => [
                r.from,
                r.label,
                r.to,
                pick(bi("langsung", "direct")),
              ]),
              ...warisan.map((r) => [
                r.from,
                r.label,
                r.to,
                pick(bi("warisan", "inherited")),
              ]),
            ],
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                `${langsung.length} sifat langsung, ${warisan.length} diwarisi dari ${view.ancestors.join(", ") || "—"}. Inilah gunanya jaringan semantik: menuliskan bahwa hewan punya sel satu kali sudah cukup untuk seluruh turunannya.`,
                `${langsung.length} direct properties, ${warisan.length} inherited from ${view.ancestors.join(", ") || "—"}. This is the point of a semantic network: writing once that animals have cells covers every descendant.`,
              ),
            ),
          }),
        ),
        card(
          pick(bi("Seluruh relasi", "All relations")),
          table(
            [
              pick(bi("Asal", "From")),
              pick(bi("Relasi", "Relation")),
              pick(bi("Tujuan", "To")),
            ],
            view.relations.map((r) => [r.from, r.label, r.to]),
          ),
        ),
      );
    }

    function render(): void {
      clear(output);
      switch (tab) {
        case "truth":
          renderTruth();
          break;
        case "resolution":
          renderResolution();
          break;
        case "network":
          renderNetwork();
          break;
      }
    }

    function textField(
      label: string,
      value: string,
      onChange: (v: string) => void,
    ): HTMLElement {
      const input = el("input", {
        attrs: { type: "text", value, "aria-label": label },
      });
      input.addEventListener("input", () => {
        onChange(input.value);
        render();
      });
      return el("label", {
        class: "field",
        children: [el("span", { class: "field__label", text: label }), input],
      });
    }

    function renderControls(): void {
      clear(controls);

      const tabs = buttonRow(
        (
          [
            ["truth", bi("Tabel kebenaran", "Truth table")],
            ["resolution", bi("Resolusi", "Resolution")],
            ["network", bi("Jaringan semantik", "Semantic network")],
          ] as [Tab, { id: string; en: string }][]
        ).map(([t, label]) => ({
          label: pick(label),
          selected: t === tab,
          onClick: () => {
            tab = t;
            renderControls();
            render();
          },
        })),
      );

      const extras: HTMLElement[] = [];

      if (tab === "truth") {
        extras.push(
          card(
            pick(T.preset),
            buttonRow(
              FORMULA_PRESETS.map((p) => ({
                label: pick(p.label),
                onClick: () => {
                  formula = p.formula;
                  renderControls();
                  render();
                },
              })),
            ),
          ),
          card(
            pick(bi("Rumus", "Formula")),
            textField(pick(bi("Rumus utama", "Main formula")), formula, (v) => {
              formula = v;
            }),
            textField(pick(bi("Pembanding", "Comparison")), compareWith, (v) => {
              compareWith = v;
            }),
            el("p", {
              class: "note",
              text: pick(
                bi(
                  "Diterima: ¬ ~ ! not · ∧ & && and · ∨ | || or · → -> => implies · ↔ <-> <=> iff",
                  "Accepted: ¬ ~ ! not · ∧ & && and · ∨ | || or · → -> => implies · ↔ <-> <=> iff",
                ),
              ),
            }),
          ),
        );
      }

      if (tab === "resolution") {
        const area = el("textarea", {
          attrs: {
            rows: "5",
            "aria-label": pick(bi("Satu rumus per baris", "One formula per line")),
          },
        });
        area.value = knowledge.join("\n");
        area.addEventListener("input", () => {
          knowledge = area.value
            .split("\n")
            .map((l) => l.trim())
            .filter((l) => l.length > 0);
          render();
        });
        extras.push(
          card(
            pick(T.preset),
            buttonRow(
              PROOF_PRESETS.map((p) => ({
                label: pick(p.label),
                onClick: () => {
                  knowledge = [...p.knowledge];
                  conclusion = p.conclusion;
                  renderControls();
                  render();
                },
              })),
            ),
          ),
          card(
            pick(bi("Basis pengetahuan, satu rumus per baris", "Knowledge base, one formula per line")),
            area,
            textField(pick(bi("Kesimpulan", "Conclusion")), conclusion, (v) => {
              conclusion = v;
            }),
          ),
        );
      }

      if (tab === "network") {
        let nodes: string[] = [];
        try {
          nodes = engine.logicSemanticNetwork(node).nodes;
        } catch {
          nodes = [];
        }
        extras.push(
          card(
            pick(bi("Simpul", "Node")),
            buttonRow(
              nodes.map((n) => ({
                label: n,
                selected: n === node,
                onClick: () => {
                  node = n;
                  renderControls();
                  render();
                },
              })),
            ),
          ),
        );
      }

      controls.append(card(pick(bi("Bagian", "Section")), tabs), ...extras);
    }

    root.append(el("div", { class: "grid-2", children: [controls, output] }));
    renderControls();
    render();

    return () => {
      clear(root);
    };
}
