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
import { figure, heatmap, pipeline as vizPipeline, rankedBars } from "../viz.js";
import type { Stage } from "../viz.js";

type Tab = "pipeline" | "stem" | "tfidf" | "similarity";

/*
 * # Kenapa contohnya tetap berbahasa Indonesia di halaman berbahasa Inggris
 *
 * Karena yang diperagakan laboratorium ini adalah **morfologi bahasa
 * Indonesia**. `stem_id` mengupas awalan "me-", "di-", "ber-" dan akhiran
 * "-kan", "-an", "-nya" mengikuti algoritme Nazief–Adriani; menjalankannya
 * pada kalimat berbahasa Inggris tidak menghasilkan pelajaran apa pun, hanya
 * kata yang dipotong sembarangan.
 *
 * Jadi yang dwibahasa di sini antarmukanya, sedangkan bahan yang dianalisis
 * tetap Indonesia — dan itu dinyatakan terang-terangan di layar, bukan
 * dibiarkan tampak seperti terjemahan yang terlupa. Wadah yang memuat bahan
 * itu ditandai `data-korpus="id"`, sehingga uji yang menolak kata Indonesia di
 * halaman berbahasa Inggris tahu persis bagian mana yang memang dikecualikan —
 * dan bagian lain mana pun tetap dituntut diterjemahkan.
 */

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

/**
 * Memasang laboratorium ke dalam elemen yang diberikan.
 *
 * Keterangannya -- judul, nomor sesi, penjelasan -- ada di
 * `labs/registry.ts`, bukan di sini, supaya daftar isi bisa ditampilkan
 * tanpa mengunduh mesin seluruh laboratorium lebih dulu.
 */
export function mount(root: HTMLElement): () => void {
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
          figure({
            title: bi("Penyusutan teks di tiap tahap", "How the text shrinks at each stage"),
            summary: bi(
              `Teks masuk sebagai ${result.sentences.length} kalimat dan keluar sebagai ` +
                `${result.final_tokens.length} kata dasar. Tiap tahap membuang sesuatu, dan ` +
                `yang dibuang tidak pernah kembali — itulah sebabnya urutannya menentukan: ` +
                `membuang kata henti sebelum mencari kata dasar memberi hasil berbeda dari ` +
                `sesudahnya.`,
              `The text enters as ${result.sentences.length} sentence(s) and leaves as ` +
                `${result.final_tokens.length} stems. Each stage discards something, and what is ` +
                `discarded never comes back — which is why the order matters: removing stopwords ` +
                `before stemming gives a different result than after.`,
            ),
            body: vizPipeline([
              {
                label: bi("Kalimat", "Sentences"),
                value: result.sentences.join(" | ").slice(0, 62) || "—",
                korpus: "id",
                note: pick(bi(`${result.sentences.length} kalimat`, `${result.sentences.length} sentence(s)`)),
              },
              {
                label: bi("Token", "Tokens"),
                value: result.tokens.join(" · ").slice(0, 62) || "—",
                korpus: "id",
                note: pick(bi(`${result.tokens.length} token`, `${result.tokens.length} tokens`)),
              },
              {
                label: bi("Buang kata henti", "Remove stopwords"),
                value: result.after_stopwords.join(" · ").slice(0, 62) || "—",
                korpus: "id",
                note: pick(bi(`${removed} kata dibuang`, `${removed} words dropped`)),
                skipped: !removeStopwords,
              },
              {
                label: bi("Cari kata dasar", "Stem"),
                value: result.final_tokens.join(" · ").slice(0, 62) || "—",
                korpus: "id",
                note: pick(
                  bi(
                    `${result.stems.filter((s) => s.original !== s.stem).length} kata dikupas`,
                    `${result.stems.filter((s) => s.original !== s.stem).length} words stripped`,
                  ),
                ),
                skipped: !stem,
              },
            ]),
          }),
          figure({
            title: bi("Berapa yang tersisa di tiap tahap", "How much survives each stage"),
            summary: bi(
              "Batang yang memendek tajam adalah tahap yang paling banyak membuang. " +
                "Tahap seperti itu paling berkuasa sekaligus paling berbahaya: satu kata henti " +
                'yang salah masuk daftar — misalnya "tidak" — sudah cukup untuk membalik makna ' +
                "seluruh kalimat tanpa satu pun galat muncul.",
              "A bar that drops sharply marks the stage that discards the most. Such stages are " +
                'the most powerful and the most dangerous: one wrong entry in the stopword list — ' +
                '"not", say — is enough to reverse a whole sentence\'s meaning without a single ' +
                "error appearing anywhere.",
            ),
            body: rankedBars(
              [
                { label: pick(bi("Token mentah", "Raw tokens")), value: result.tokens.length },
                {
                  label: pick(bi("Tanpa kata henti", "After stopwords")),
                  value: result.after_stopwords.length,
                },
                {
                  label: pick(bi("Kata dasar akhir", "Final stems")),
                  value: result.final_tokens.length,
                  highlight: true,
                },
              ],
              (v) => String(Math.round(v)),
            ),
          }),
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
            "id",
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
                  "id",
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
              "id",
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
              attrs: { "data-korpus": "id" },
            }),
          ),
        );
      } catch {
        /* Ukuran n-gram tidak sah; bagian ini dilewati. */
      }
    }

    /**
     * Menyusun alur pengupasan satu kata menjadi tahap-tahap yang terlihat.
     *
     * Tahap pemeriksaan kamus selalu ditampilkan, termasuk ketika ia yang
     * menghentikan pengupasan. Justru tahap itulah yang membedakan pencari
     * kata dasar Bahasa Indonesia dari yang Bahasa Inggris, dan tahap yang
     * tidak terlihat tidak akan dipelajari siapa pun.
     */
    function tahapStem(r: engine.StemResult): Stage[] {
      const tahap: Stage[] = [
        {
          label: bi("Kata masukan", "Input word"),
          value: r.original,
        },
      ];
      let berjalan = r.original;
      for (const s of r.steps) {
        tahap.push({
          label: bi(`Kupas ${s.kind}`, `Strip ${s.kind}`),
          value: s.result,
          note: pick(
            bi(`membuang "${s.affix}" dari ${berjalan}`, `removes "${s.affix}" from ${berjalan}`),
          ),
        });
        berjalan = s.result;
      }
      tahap.push({
        label: bi("Periksa kamus", "Dictionary check"),
        value: r.stem,
        note: pick(
          r.in_dictionary
            ? bi("ditemukan di kamus — pengupasan berhenti", "found in dictionary — stripping stops")
            : bi("tidak ada di kamus — hasil terbaik yang didapat", "not in dictionary — best effort result"),
        ),
        skipped: false,
      });
      return tahap;
    }

    function renderStem(): void {
      // Dua kata yang paling menjelaskan algoritmanya digambar alurnya; sisanya
      // cukup ditabelkan. Menggambar keempat belasnya justru mengubur bagian
      // yang ingin ditunjukkan.
      const sorot = ["menyapu", "beruang"].filter((w) => SAMPLE_WORDS.includes(w));
      const alur = sorot.length > 0 ? sorot : SAMPLE_WORDS.slice(0, 2);
      for (const word of alur) {
        try {
          const r = engine.nlpStem(word);
          output.append(
            card(
              pick(bi(`Pengupasan "${word}"`, `Stripping "${word}"`)),
              figure({
                title: bi("Alur pencarian kata dasar", "Stemming pipeline"),
                summary: bi(
                  r.steps.length === 0
                    ? `Tidak ada imbuhan yang dikupas dari "${r.original}": kata itu sudah ada di kamus. ` +
                      `Tanpa pemeriksaan kamus, aturan pengupasan akan tetap berjalan dan menghasilkan kata lain sama sekali.`
                    : `"${r.original}" dikupas ${r.steps.length} kali menjadi "${r.stem}". ` +
                      `Tiap kotak adalah keadaan kata setelah satu imbuhan dibuang; ` +
                      `kotak terakhir adalah pemeriksaan kamus yang menentukan apakah pengupasan boleh berhenti.`,
                  r.steps.length === 0
                    ? `No affix was stripped from "${r.original}": the word is already in the dictionary. ` +
                      `Without that check the stripping rules would run anyway and produce a different word entirely.`
                    : `"${r.original}" was stripped ${r.steps.length} time(s) down to "${r.stem}". ` +
                      `Each box is the word after one affix is removed; the last box is the dictionary ` +
                      `check that decides whether stripping may stop.`,
                ),
                body: vizPipeline(tahapStem(r)),
              }),
            ),
          );
        } catch {
          /* Kata yang gagal diproses cukup dilewati; tabel di bawah tetap memuatnya. */
        }
      }

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

      // Peta panas hanya memuat kata yang benar-benar membedakan. Menggambar
      // seluruh kosakata menghasilkan matriks yang sebagian besar kosong, dan
      // kolom kosong tidak menjelaskan apa pun.
      const kolom = ranked.slice(0, 10).map((r) => r.term);
      const indeks = kolom.map((t) => result.vocabulary.indexOf(t));
      const matriks = result.vectors.map((v) => indeks.map((i) => v[i] ?? 0));

      output.append(
        card(
          pick(bi("Bobot TF-IDF", "TF-IDF weights")),
          figure({
            title: bi("Peta panas kata lawan dokumen", "Term-by-document heatmap"),
            summary: bi(
              "Makin pekat sebuah sel, makin besar bobot kata itu pada dokumen tersebut. " +
                "Kata yang muncul di semua dokumen tampak pucat di mana-mana walau sering diucapkan — " +
                "itulah inti TF-IDF: yang sering muncul di mana-mana justru tidak informatif.",
              "The darker a cell, the larger that term's weight in that document. Terms that appear " +
                "in every document look pale everywhere even when frequent — that is the whole point " +
                "of TF-IDF: what appears everywhere carries no information.",
            ),
            body: heatmap({
              rows: docs.map((_, i) => `D${i + 1}`),
              cols: kolom,
              values: matriks,
              format: (v) => (v === 0 ? "" : fmt(v, 2)),
            }),
          }),
        ),
        card(
          pick(bi("Kata paling membedakan", "Most distinguishing terms")),
          figure({
            title: bi("Peringkat IDF", "IDF ranking"),
            summary: bi(
              `Kata dengan IDF tertinggi hanya muncul di sedikit dokumen, jadi kehadirannya ` +
                `banyak bercerita. Batang teratas adalah kata yang paling berguna untuk memisahkan ` +
                `${docs.length} dokumen ini satu sama lain.`,
              `Terms with the highest IDF appear in few documents, so their presence says a lot. ` +
                `The top bar is the term most useful for telling these ${docs.length} documents apart.`,
            ),
            body: rankedBars(
              ranked.map((r, i) => ({
                label: r.term,
                value: r.idf,
                highlight: i === 0,
              })),
              (v) => fmt(v, 3),
            ),
          }),
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
          figure({
            title: bi("Matriks kemiripan kosinus", "Cosine similarity matrix"),
            summary: bi(
              "Diagonalnya selalu satu — setiap dokumen mirip sempurna dengan dirinya sendiri — " +
                "dan matriksnya setangkup terhadap diagonal itu. Yang layak diperhatikan adalah " +
                "sel di luar diagonal: di sanalah terlihat dokumen mana yang sebenarnya sepasang.",
              "The diagonal is always one — every document matches itself perfectly — and the " +
                "matrix is symmetric about it. What matters are the off-diagonal cells: that is " +
                "where you see which documents actually belong together.",
            ),
            body: heatmap({
              rows: docs.map((_, i) => `D${i + 1}`),
              cols: docs.map((_, i) => `D${i + 1}`),
              values: result.similarity,
              format: (v) => fmt(v, 2),
            }),
          }),
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
          selected: t === tab,
          onClick: () => {
            tab = t;
            renderControls();
            render();
          },
        })),
      );

      const extras: HTMLElement[] = [];

      // Dinyatakan terang-terangan, bukan dibiarkan tampak seperti terjemahan
      // yang terlupa: bahan yang dianalisis di sini tetap Bahasa Indonesia,
      // karena itulah bahasa yang morfologinya sedang dibedah.
      extras.push(
        el("p", {
          class: "note",
          text: pick(
            bi(
              "Bahan yang dianalisis di laboratorium ini berbahasa Indonesia, termasuk ketika antarmukanya berbahasa Inggris. Pengupas imbuhannya mengikuti algoritme Nazief–Adriani untuk morfologi Bahasa Indonesia — awalan “me-”, “di-”, “ber-” dan akhiran “-kan”, “-an”, “-nya”. Menjalankannya pada kalimat Bahasa Inggris hanya memotong kata secara sembarangan. Silakan ganti teksnya dengan kalimat Bahasa Indonesia Anda sendiri.",
              "The material analysed in this lab is Indonesian, even when the interface is in English. Its stemmer implements the Nazief–Adriani algorithm for Indonesian morphology — prefixes “me-”, “di-”, “ber-” and suffixes “-kan”, “-an”, “-nya”. Running it on English sentences merely chops words at random. Feel free to replace the text with your own Indonesian sentences.",
            ),
          ),
        }),
      );

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
}
