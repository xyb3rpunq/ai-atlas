/**
 * Laboratorium Sesi 10 — Pemrosesan Bahasa Alami.
 *
 * Teks Bahasa Indonesia diproses tahap demi tahap dan setiap tahap
 * diperlihatkan apa adanya: pemenggalan, penghapusan kata henti, pencarian kata
 * dasar lengkap dengan imbuhan yang dikupas, lalu pembobotan TF-IDF dan
 * kemiripan antardokumen.
 *
 * Bagian yang paling layak diperhatikan adalah pencarian kata dasarnya.
 * Algoritma untuk Bahasa Inggris tidak bisa dipakai di sini karena sebagian
 * awalan Bahasa Indonesia meluluhkan huruf pertama kata dasarnya — `menyapu`
 * berasal dari `sapu`, bukan `nyapu`.
 */

import * as engine from "../engine.js";
import { T, bi, pick } from "../i18n.js";
import { buttonRow, card, clear, el, errorNote, fmt, slider, table } from "../ui.js";
import type { Lab } from "./registry.js";

type Tab = "pipeline" | "stem" | "tfidf" | "similarity";

const SAMPLE_TEXT =
  "Saya suka membaca buku di perpustakaan kampus. Pelayanannya bagus dan petugasnya ramah. Sayangnya bukunya tidak lengkap.";

const SAMPLE_DOCS = [
  "Kucing itu suka makan ikan segar",
  "Kucing kesayangan saya gemar makan ikan",
  "Mobil balap melaju sangat cepat di lintasan",
];

const SAMPLE_WORDS = [
  "menyapu",
  "beruang",
  "membacakan",
  "pembelajaran",
  "dituliskannya",
  "memukul",
  "anak-anak",
  "bukumu",
];

export const nlpLab: Lab = {
  slug: "nlp",
  session: 10,
  title: bi("Pemrosesan Bahasa Alami", "Natural Language Processing"),
  blurb: bi(
    "Teks Bahasa Indonesia diproses tahap demi tahap, dan tiap tahap diperlihatkan apa adanya. Perhatikan pencarian kata dasarnya: sebagian awalan meluluhkan huruf pertama kata dasarnya, sehingga algoritma untuk Bahasa Inggris tidak bisa dipakai begitu saja.",
    "Indonesian text processed stage by stage, with every stage shown as it is. Watch the stemmer in particular: some prefixes dissolve the first letter of the root, so an English algorithm cannot simply be reused.",
  ),

  mount(root: HTMLElement): () => void {
    let tab: Tab = "pipeline";
    let text = SAMPLE_TEXT;
    let removeStopwords = true;
    let stem = true;
    let docs = [...SAMPLE_DOCS];
    let wordA = "kucing";
    let wordB = "kucin";
    let ngramSize = 2;

    const controls = el("div");
    const output = el("div");

    function renderPipeline(): void {
      let result: engine.NlpPipeline;
      try {
        result = engine.nlpPipeline(text, removeStopwords, stem);
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }

      const removed = result.tokens.length - result.after_stopwords.length;
      let sentiment: engine.Sentiment | null = null;
      try {
        sentiment = engine.nlpSentiment(text);
      } catch {
        sentiment = null;
      }

      output.append(
        card(
          pick(bi("Tahap demi tahap", "Stage by stage")),
          table(
            [
              pick(bi("Tahap", "Stage")),
              pick(bi("Jumlah", "Count")),
              pick(bi("Hasil", "Result")),
            ],
            [
              [
                pick(bi("Kalimat", "Sentences")),
                String(result.sentences.length),
                result.sentences.join(" | "),
              ],
              [
                pick(bi("Token", "Tokens")),
                String(result.tokens.length),
                result.tokens.join(" · "),
              ],
              [
                pick(bi("Tanpa kata henti", "Stopwords removed")),
                String(result.after_stopwords.length),
                result.after_stopwords.join(" · "),
              ],
              [
                pick(bi("Kata dasar", "Stems")),
                String(result.final_tokens.length),
                result.final_tokens.join(" · "),
              ],
            ],
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                `${removed} kata hilang saat kata henti dibuang. Daftar kata henti di sini sengaja dijaga pendek: daftar yang panjang akan ikut membuang kata seperti “tidak”, dan itu membalik makna kalimat.`,
                `${removed} words disappeared when stopwords were removed. The stopword list here is deliberately short: a long one would also remove words like “not”, which reverses the meaning of a sentence.`,
              ),
            ),
          }),
        ),
      );

      if (result.stems.length > 0) {
        const berubah = result.stems.filter((s) => s.original !== s.stem);
        output.append(
          card(
            pick(bi("Imbuhan yang dikupas", "Affixes stripped")),
            berubah.length > 0
              ? table(
                  [
                    pick(bi("Kata", "Word")),
                    pick(bi("Kata dasar", "Stem")),
                    pick(bi("Langkah", "Steps")),
                    pick(bi("Di kamus", "In dictionary")),
                  ],
                  berubah.map((s) => [
                    s.original,
                    s.stem,
                    s.steps.map((st) => `${st.kind}: ${st.affix}`).join(" → "),
                    s.in_dictionary ? "✓" : "—",
                  ]),
                )
              : el("p", {
                  class: "note",
                  text: pick(
                    bi(
                      "Tidak ada kata berimbuhan pada teks ini.",
                      "No affixed words in this text.",
                    ),
                  ),
                }),
          ),
        );
      }

      if (sentiment) {
        output.append(
          card(
            pick(bi("Sentimen", "Sentiment")),
            table(
              [pick(bi("Ukuran", "Measure")), pick(bi("Nilai", "Value"))],
              [
                [pick(bi("Label", "Label")), sentiment.label],
                [pick(bi("Skor", "Score")), sentiment.score],
                [
                  pick(bi("Kata bermuatan", "Charged words")),
                  sentiment.matches
                    .map(([w, v]) => `${w} (${v > 0 ? "+" : ""}${fmt(v, 0)})`)
                    .join(", ") || "—",
                ],
              ],
            ),
            el("p", {
              class: "note",
              text: pick(
                bi(
                  "Pengingkaran diperhitungkan sampai dua kata ke belakang. Tanpa itu, “tidak bagus” akan dinilai positif — kesalahan yang membuat analisis sentimen tidak berguna pada ulasan berbahasa Indonesia.",
                  "Negation is tracked up to two words back. Without it, “not good” would score positive — a mistake that makes sentiment analysis useless on Indonesian reviews.",
                ),
              ),
            }),
          ),
        );
      }

      try {
        const grams = engine.nlpNgrams(text, ngramSize);
        output.append(
          card(
            `${ngramSize}-gram`,
            el("p", {
              class: "note",
              text: grams.slice(0, 40).join(" | ") || pick(bi("kosong", "empty")),
            }),
          ),
        );
      } catch {
        /* Ukuran n-gram tidak sah; bagian ini dilewati. */
      }
    }

    function renderStem(): void {
      const rows: (string | number)[][] = [];
      for (const word of SAMPLE_WORDS) {
        try {
          const r = engine.nlpStem(word);
          rows.push([
            r.original,
            r.stem,
            r.steps.map((s) => `${s.kind}: ${s.affix}`).join(" → ") || "—",
            r.in_dictionary ? "✓" : "—",
          ]);
        } catch {
          rows.push([word, "—", "—", "—"]);
        }
      }

      output.append(
        card(
          pick(bi("Kata uji", "Test words")),
          table(
            [
              pick(bi("Kata", "Word")),
              pick(bi("Kata dasar", "Stem")),
              pick(bi("Langkah pengupasan", "Stripping steps")),
              pick(bi("Di kamus", "In dictionary")),
            ],
            rows,
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Perhatikan dua baris: “menyapu” menjadi “sapu” karena awalan meny- meluluhkan huruf s, sedangkan “beruang” tidak dikupas sama sekali karena sudah ada di kamus. Tanpa pemeriksaan kamus, kata itu akan menjadi “uang” — dan tidak ada aturan pengupasan yang bisa mencegahnya.",
                "Note two rows: “menyapu” becomes “sapu” because the meny- prefix dissolves the letter s, while “beruang” (bear) is not stripped at all because it is already in the dictionary. Without that check it would become “uang” (money) — and no stripping rule could prevent it.",
              ),
            ),
          }),
        ),
      );
    }

    function renderTfIdf(): void {
      let result: engine.TfIdfResult;
      try {
        result = engine.nlpTfIdf(docs, removeStopwords, stem);
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }

      // Kata dengan IDF tertinggi adalah yang paling membedakan dokumen.
      const ranked = result.vocabulary
        .map((term, i) => ({ term, idf: result.idf[i] }))
        .sort((a, b) => b.idf - a.idf)
        .slice(0, 12);

      output.append(
        card(
          pick(bi("Kata paling membedakan", "Most distinguishing terms")),
          table(
            [pick(bi("Kata", "Term")), "IDF"],
            ranked.map((r) => [r.term, r.idf]),
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                `Kosakata berisi ${result.vocabulary.length} kata dari ${docs.length} dokumen. IDF memakai bentuk yang dihaluskan; bentuk mentah memberi nol untuk kata yang muncul di semua dokumen, sehingga kata itu lenyap sama sekali dari perhitungan.`,
                `The vocabulary holds ${result.vocabulary.length} terms across ${docs.length} documents. IDF uses the smoothed form; the raw form gives zero for terms appearing in every document, which erases them from the calculation entirely.`,
              ),
            ),
          }),
        ),
        card(
          pick(bi("Kemiripan antardokumen", "Document similarity")),
          table(
            ["", ...docs.map((_, i) => `D${i + 1}`)],
            result.similarity.map((row, i) => [
              `D${i + 1}`,
              ...row.map((v) => fmt(v, 3)),
            ]),
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Kemiripan kosinus tidak terpengaruh panjang dokumen, hanya arah vektornya. Dua dokumen yang membahas hal sama dengan panjang berbeda tetap dinilai mirip.",
                "Cosine similarity ignores document length and looks only at vector direction. Two documents about the same subject stay similar even at very different lengths.",
              ),
            ),
          }),
        ),
        card(
          pick(bi("Token tiap dokumen", "Tokens per document")),
          table(
            [
              pick(bi("Dokumen", "Document")),
              pick(bi("Jumlah", "Count")),
              pick(bi("Token", "Tokens")),
            ],
            result.documents.map((d, i) => [
              `D${i + 1}`,
              String(d.length),
              d.join(" · "),
            ]),
          ),
        ),
      );
    }

    function renderSimilarity(): void {
      let result: engine.EditDistance;
      try {
        result = engine.nlpLevenshtein(wordA, wordB);
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }
      const contoh: (string | number)[][] = [
        ["kitten", "sitting", 0],
        ["café", "cafe", 0],
        ["kucing", "kucin", 0],
        ["makan", "minum", 0],
      ].map(([a, b]) => {
        try {
          const r = engine.nlpLevenshtein(String(a), String(b));
          return [String(a), String(b), r.distance, fmt(r.similarity, 3)];
        } catch {
          return [String(a), String(b), "—", "—"];
        }
      });

      output.append(
        card(
          pick(T.result),
          table(
            [pick(bi("Ukuran", "Measure")), pick(bi("Nilai", "Value"))],
            [
              [pick(bi("Kata pertama", "First word")), wordA],
              [pick(bi("Kata kedua", "Second word")), wordB],
              [pick(bi("Jarak sunting", "Edit distance")), String(result.distance)],
              [pick(bi("Kemiripan", "Similarity")), fmt(result.similarity, 4)],
            ],
          ),
        ),
        card(
          pick(bi("Contoh pembanding", "Reference examples")),
          table(
            [
              pick(bi("Kata A", "Word A")),
              pick(bi("Kata B", "Word B")),
              pick(bi("Jarak", "Distance")),
              pick(bi("Kemiripan", "Similarity")),
            ],
            contoh,
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Baris “café” dan “cafe” berjarak satu, bukan dua. Jaraknya dihitung per karakter Unicode, bukan per bita — huruf beraksen memakai dua bita, dan menghitungnya per bita memberi jarak yang salah sekaligus berisiko memotong karakter di tengah.",
                "The “café” and “cafe” row is distance one, not two. Distance is counted in Unicode characters, not bytes — an accented letter takes two bytes, and counting bytes gives the wrong distance while risking a split mid-character.",
              ),
            ),
          }),
        ),
      );
    }

    function render(): void {
      clear(output);
      switch (tab) {
        case "pipeline":
          renderPipeline();
          break;
        case "stem":
          renderStem();
          break;
        case "tfidf":
          renderTfIdf();
          break;
        case "similarity":
          renderSimilarity();
          break;
      }
    }

    function renderControls(): void {
      clear(controls);

      const tabs = buttonRow(
        (
          [
            ["pipeline", bi("Alur proses", "Pipeline")],
            ["stem", bi("Kata dasar", "Stemming")],
            ["tfidf", bi("TF-IDF", "TF-IDF")],
            ["similarity", bi("Jarak sunting", "Edit distance")],
          ] as [Tab, { id: string; en: string }][]
        ).map(([t, label]) => ({
          label: pick(label),
          primary: t === tab,
          onClick: () => {
            tab = t;
            renderControls();
            render();
          },
        })),
      );

      const extras: HTMLElement[] = [];

      if (tab === "pipeline") {
        const area = el("textarea", {
          attrs: {
            rows: "5",
            "aria-label": pick(bi("Teks yang diproses", "Text to process")),
          },
        });
        area.value = text;
        area.addEventListener("input", () => {
          text = area.value;
          render();
        });
        extras.push(
          card(pick(bi("Teks", "Text")), area),
          card(
            pick(T.controls),
            buttonRow([
              {
                label: removeStopwords
                  ? pick(bi("Kata henti: dibuang", "Stopwords: removed"))
                  : pick(bi("Kata henti: dibiarkan", "Stopwords: kept")),
                onClick: () => {
                  removeStopwords = !removeStopwords;
                  renderControls();
                  render();
                },
              },
              {
                label: stem
                  ? pick(bi("Kata dasar: nyala", "Stemming: on"))
                  : pick(bi("Kata dasar: mati", "Stemming: off")),
                onClick: () => {
                  stem = !stem;
                  renderControls();
                  render();
                },
              },
            ]),
            slider({
              label: pick(bi("Ukuran n-gram", "N-gram size")),
              min: 1,
              max: 5,
              step: 1,
              value: ngramSize,
              format: (v) => String(v),
              onInput: (v) => {
                ngramSize = v;
                render();
              },
            }),
          ),
        );
      }

      if (tab === "tfidf") {
        const area = el("textarea", {
          attrs: {
            rows: "6",
            "aria-label": pick(
              bi("Satu dokumen per baris", "One document per line"),
            ),
          },
        });
        area.value = docs.join("\n");
        area.addEventListener("input", () => {
          docs = area.value
            .split("\n")
            .map((d) => d.trim())
            .filter((d) => d.length > 0);
          render();
        });
        extras.push(
          card(pick(bi("Korpus, satu dokumen per baris", "Corpus, one document per line")), area),
          card(
            pick(T.controls),
            buttonRow([
              {
                label: removeStopwords
                  ? pick(bi("Kata henti: dibuang", "Stopwords: removed"))
                  : pick(bi("Kata henti: dibiarkan", "Stopwords: kept")),
                onClick: () => {
                  removeStopwords = !removeStopwords;
                  renderControls();
                  render();
                },
              },
              {
                label: stem
                  ? pick(bi("Kata dasar: nyala", "Stemming: on"))
                  : pick(bi("Kata dasar: mati", "Stemming: off")),
                onClick: () => {
                  stem = !stem;
                  renderControls();
                  render();
                },
              },
              {
                label: pick(bi("Contoh awal", "Sample corpus")),
                onClick: () => {
                  docs = [...SAMPLE_DOCS];
                  renderControls();
                  render();
                },
              },
            ]),
          ),
        );
      }

      if (tab === "similarity") {
        const inputA = el("input", {
          attrs: { type: "text", value: wordA, "aria-label": "kata A" },
        });
        inputA.addEventListener("input", () => {
          wordA = inputA.value;
          render();
        });
        const inputB = el("input", {
          attrs: { type: "text", value: wordB, "aria-label": "kata B" },
        });
        inputB.addEventListener("input", () => {
          wordB = inputB.value;
          render();
        });
        extras.push(
          card(
            pick(bi("Dua kata", "Two words")),
            el("div", { class: "field", children: [inputA] }),
            el("div", { class: "field", children: [inputB] }),
          ),
        );
      }

      if (tab === "stem") {
        extras.push(
          card(
            pick(bi("Catatan", "Note")),
            el("p", {
              class: "note",
              text: pick(
                bi(
                  "Kata uji dipilih untuk memperlihatkan kasus yang sulit, termasuk yang sengaja tidak boleh dikupas.",
                  "The test words are chosen to show the hard cases, including ones that deliberately must not be stripped.",
                ),
              ),
            }),
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
  },
};
