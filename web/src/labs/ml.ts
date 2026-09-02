/**
 * Laboratorium Sesi 12 & 13 — Sains Data dan Machine Learning.
 *
 * Empat algoritma dalam satu halaman, dipilih lewat tab:
 *
 * - **KNN** — klik untuk menaruh titik, lalu lihat wilayah keputusannya
 *   berubah saat `k` digeser. Nilai `k` yang terlalu kecil menghafal derau;
 *   yang terlalu besar melumatkan batas antarkelas.
 * - **K-Means** — pengelompokan tanpa label, dengan pusat yang bergerak.
 * - **Pohon Keputusan** — entropi dan perolehan informasi tiap atribut
 *   ditampilkan apa adanya, sehingga terlihat mengapa satu atribut dipilih.
 * - **Regresi** — seret titik dan lihat garis kuadrat terkecil menyesuaikan.
 */

import * as engine from "../engine.js";
import { T, bi, pick } from "../i18n.js";
import { buttonRow, card, clear, el, errorNote, fmt, pct, slider, table } from "../ui.js";
import { figure, heatmap, rankedBars } from "../viz.js";

type Tab = "knn" | "kmeans" | "tree" | "regression";

/** Satu titik data berlabel. */
interface Point {
  x: number;
  y: number;
  label: string;
}

const CLASSES = ["A", "B", "C"];
const SIZE = 440;
const RANGE: [number, number] = [0, 10];

/** Warna kelas, tetap agar tetap terbedakan pada mode terang maupun gelap. */
const CLASS_COLOURS = ["#4dd4c8", "#f0b429", "#c084f5"];

/** Titik awal: dua gugus yang jelas terpisah plus sedikit tumpang tindih. */
function seedPoints(): Point[] {
  const pts: Point[] = [];
  const grid: [number, number, string][] = [
    [2, 2, "A"],
    [2.6, 2.4, "A"],
    [1.6, 2.8, "A"],
    [2.2, 3.4, "A"],
    [3.0, 1.8, "A"],
    [7.5, 7.5, "B"],
    [8.1, 7.1, "B"],
    [7.0, 8.2, "B"],
    [8.4, 8.0, "B"],
    [6.9, 7.0, "B"],
    [5.0, 5.0, "A"],
    [5.6, 5.4, "B"],
  ];
  for (const [x, y, label] of grid) pts.push({ x, y, label });
  return pts;
}

/**
 * Memasang laboratorium ke dalam elemen yang diberikan.
 *
 * Keterangannya -- judul, nomor sesi, penjelasan -- ada di
 * `labs/registry.ts`, bukan di sini, supaya daftar isi bisa ditampilkan
 * tanpa mengunduh mesin seluruh laboratorium lebih dulu.
 */
export function mount(root: HTMLElement): () => void {
    let tab: Tab = "knn";
    let points = seedPoints();
    let brush = "A";
    let k = 3;
    let distance: engine.Distance = "euclidean";
    let weighted = false;
    let clusters = 2;
    let treeDepth = 6;
    let seed = 42;

    const canvas = el("canvas", {
      attrs: {
        role: "img",
        "aria-label": pick(
          bi(
            "Bidang data dua dimensi beserta wilayah keputusan model.",
            "A two dimensional data plane with the model decision regions.",
          ),
        ),
      },
    });
    const controls = el("div");
    const output = el("div");

    const tennis = engine.mlTennisDataset();

    /** Mengubah koordinat data menjadi piksel. */
    const toPx = (v: number): number =>
      ((v - RANGE[0]) / (RANGE[1] - RANGE[0])) * SIZE;

    function ctx2d(): CanvasRenderingContext2D | null {
      const dpr = Math.min(globalThis.devicePixelRatio || 1, 2);
      canvas.width = Math.round(SIZE * dpr);
      canvas.height = Math.round(SIZE * dpr);
      canvas.style.aspectRatio = "1 / 1";
      const c = canvas.getContext("2d");
      if (!c) return null;
      c.setTransform(dpr, 0, 0, dpr, 0, 0);
      return c;
    }

    function drawPoints(c: CanvasRenderingContext2D, assignments?: number[]): void {
      points.forEach((p, i) => {
        const colour =
          assignments !== undefined
            ? CLASS_COLOURS[assignments[i] % CLASS_COLOURS.length]
            : CLASS_COLOURS[Math.max(CLASSES.indexOf(p.label), 0) % CLASS_COLOURS.length];
        c.beginPath();
        c.arc(toPx(p.x), SIZE - toPx(p.y), 5, 0, Math.PI * 2);
        c.fillStyle = colour;
        c.fill();
        c.lineWidth = 1.2;
        c.strokeStyle = "rgb(0 0 0 / 60%)";
        c.stroke();
      });
    }

    function drawPlane(): void {
      const c = ctx2d();
      if (!c) return;
      const style = getComputedStyle(document.documentElement);
      c.fillStyle = style.getPropertyValue("--bg-grid").trim() || "#0d131c";
      c.fillRect(0, 0, SIZE, SIZE);

      if (tab === "knn" && points.length >= k && k >= 1) {
        try {
          const regions = engine.mlKnnRegions(
            points.map((p) => [p.x, p.y]),
            points.map((p) => p.label),
            k,
            distance,
            weighted,
            RANGE[0],
            RANGE[1],
            60,
          );
          const cell = SIZE / regions.resolution;
          c.globalAlpha = 0.24;
          for (let j = 0; j < regions.resolution; j++) {
            for (let i = 0; i < regions.resolution; i++) {
              const idx = regions.cells[j * regions.resolution + i];
              const label = regions.classes[idx] ?? "A";
              c.fillStyle =
                CLASS_COLOURS[Math.max(CLASSES.indexOf(label), 0) % CLASS_COLOURS.length];
              c.fillRect(i * cell, SIZE - (j + 1) * cell, cell + 0.6, cell + 0.6);
            }
          }
          c.globalAlpha = 1;
        } catch {
          /* Wilayah keputusan hanya hiasan; kegagalannya tidak menghalangi titik. */
        }
        drawPoints(c);
        return;
      }

      if (tab === "kmeans" && points.length >= clusters) {
        try {
          const result = engine.mlKmeans(
            points.map((p) => [p.x, p.y]),
            clusters,
            distance,
            100,
            seed,
          );
          drawPoints(c, result.assignments);
          // Pusat kelompok digambar sebagai silang agar tidak tertukar dengan data.
          c.lineWidth = 2.5;
          result.centroids.forEach((centroid, i) => {
            const cx = toPx(centroid[0]);
            const cy = SIZE - toPx(centroid[1]);
            c.strokeStyle = CLASS_COLOURS[i % CLASS_COLOURS.length];
            c.beginPath();
            c.moveTo(cx - 8, cy);
            c.lineTo(cx + 8, cy);
            c.moveTo(cx, cy - 8);
            c.lineTo(cx, cy + 8);
            c.stroke();
          });
          return;
        } catch {
          /* Jatuh ke gambar titik biasa. */
        }
      }

      if (tab === "regression") {
        drawPoints(c);
        try {
          const model = engine.mlFitLinear(
            points.map((p) => p.x),
            points.map((p) => p.y),
          );
          const style2 = getComputedStyle(document.documentElement);
          c.strokeStyle = style2.getPropertyValue("--accent").trim() || "#4dd4c8";
          c.lineWidth = 2.5;
          c.beginPath();
          c.moveTo(toPx(RANGE[0]), SIZE - toPx(model.intercept + model.slope * RANGE[0]));
          c.lineTo(toPx(RANGE[1]), SIZE - toPx(model.intercept + model.slope * RANGE[1]));
          c.stroke();
        } catch {
          /* Kurang dari dua titik; tidak ada garis yang bisa digambar. */
        }
        return;
      }

      drawPoints(c);
    }

    function renderKnn(): void {
      if (points.length < k) {
        output.append(
          errorNote(
            pick(
              bi(
                `Butuh minimal ${k} titik untuk k = ${k}.`,
                `At least ${k} points are needed for k = ${k}.`,
              ),
            ),
          ),
        );
        return;
      }
      // Ketepatan pada data latihnya sendiri, sebagai bahan diskusi tentang
      // mengapa angka itu tidak boleh dipakai menilai model.
      const predicted = points.map((p) => {
        try {
          return engine.mlKnnPredict(
            points.map((q) => [q.x, q.y]),
            points.map((q) => q.label),
            [p.x, p.y],
            k,
            distance,
            weighted,
          ).label;
        } catch {
          return p.label;
        }
      });
      const evaluation = engine.mlEvaluate(
        points.map((p) => p.label),
        predicted,
      );

      output.append(
        card(
          pick(bi("Evaluasi pada data latih", "Evaluation on the training data")),
          table(
            [pick(bi("Ukuran", "Measure")), pick(bi("Nilai", "Value"))],
            [
              [pick(bi("Ketepatan", "Accuracy")), pct(evaluation.accuracy, 1)],
              [
                pick(bi("Tebakan kelas terbanyak", "Majority-class baseline")),
                pct(evaluation.baseline_accuracy, 1),
              ],
              [pick(bi("F1 makro", "Macro F1")), fmt(evaluation.macro_f1, 3)],
            ],
          ),
          el("p", {
            class: "note",
            text:
              evaluation.accuracy <= evaluation.baseline_accuracy + 1e-9
                ? pick(
                    bi(
                      "Model ini tidak melampaui tebakan kelas terbanyak. Ketepatannya boleh terlihat tinggi, tetapi ia belum mempelajari apa pun.",
                      "This model does not beat guessing the majority class. Its accuracy may look high, but it has learned nothing.",
                    ),
                  )
                : pick(
                    bi(
                      "Angka ini diukur pada data yang dipakai melatih, jadi selalu terlalu optimistis. Itu sebabnya evaluasi yang sungguhan memerlukan data uji terpisah.",
                      "These numbers are measured on the very data used for training, so they are always too optimistic. That is why real evaluation needs a held-out test set.",
                    ),
                  ),
          }),
        ),
        card(
          pick(bi("Matriks konfusi", "Confusion matrix")),
          figure({
            title: bi("Salah tebak jatuh ke mana", "Where the mistakes land"),
            summary: bi(
              "Baris adalah kelas sebenarnya, kolom adalah tebakan model. Sel diagonal " +
                "adalah tebakan yang benar; sel di luar diagonal adalah kesalahannya, dan " +
                "letaknya jauh lebih berguna daripada jumlahnya. Model yang salah 10 kali " +
                "pada satu pasangan kelas punya masalah yang bisa diperbaiki; model yang " +
                "salah 10 kali tersebar merata hanya belum belajar.",
              "Rows are the true class, columns are the model's guess. Diagonal cells are " +
                "correct guesses; off-diagonal cells are the mistakes, and where they land " +
                "matters far more than how many there are. A model wrong 10 times on one pair " +
                "of classes has a fixable problem; one wrong 10 times spread evenly has simply " +
                "not learned.",
            ),
            body: heatmap({
              rows: evaluation.labels,
              cols: evaluation.labels,
              values: evaluation.matrix,
              format: (v) => String(Math.round(v)),
            }),
          }),
          table(
            [pick(bi("Sebenarnya \\ Ramalan", "Actual \\ Predicted")), ...evaluation.labels],
            evaluation.matrix.map((row, i) => [
              evaluation.labels[i],
              ...row.map((v) => String(v)),
            ]),
          ),
        ),
      );
    }

    function renderKmeans(): void {
      if (points.length < clusters) {
        output.append(
          errorNote(
            pick(
              bi(
                `Butuh minimal ${clusters} titik untuk ${clusters} kelompok.`,
                `At least ${clusters} points are needed for ${clusters} clusters.`,
              ),
            ),
          ),
        );
        return;
      }
      let result: engine.Clustering;
      try {
        result = engine.mlKmeans(
          points.map((p) => [p.x, p.y]),
          clusters,
          distance,
          100,
          seed,
        );
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }

      // Kurva siku: inertia untuk beberapa nilai k sekaligus.
      const elbow: (string | number)[][] = [];
      for (let kk = 1; kk <= Math.min(6, points.length); kk++) {
        try {
          const r = engine.mlKmeans(
            points.map((p) => [p.x, p.y]),
            kk,
            distance,
            100,
            seed,
          );
          elbow.push([String(kk), r.inertia, r.converged ? "✓" : "—"]);
        } catch {
          /* Nilai k ini tidak berlaku untuk jumlah titik saat ini. */
        }
      }

      output.append(
        card(
          pick(T.result),
          table(
            [pick(bi("Ukuran", "Measure")), pick(bi("Nilai", "Value"))],
            [
              [pick(bi("Kelompok", "Clusters")), String(result.centroids.length)],
              [pick(bi("Inertia", "Inertia")), fmt(result.inertia, 3)],
              [pick(bi("Sapuan", "Iterations")), String(result.iterations)],
              [
                pick(bi("Mencapai keadaan tetap", "Converged")),
                result.converged ? pick(bi("ya", "yes")) : pick(bi("tidak", "no")),
              ],
            ],
          ),
        ),
        card(
          pick(bi("Kurva siku", "Elbow curve")),
          table(
            ["k", pick(bi("Inertia", "Inertia")), pick(bi("Tetap", "Converged"))],
            elbow,
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Inertia selalu menurun saat k dinaikkan, jadi memilih k dengan mencari inertia terkecil akan selalu menjawab “sebanyak titiknya”. Yang dicari adalah tempat penurunannya mulai melandai.",
                "Inertia always falls as k rises, so choosing k by minimising inertia always answers “as many as there are points”. What you look for is where the fall flattens out.",
              ),
            ),
          }),
        ),
      );
    }

    function renderTree(): void {
      let built: engine.TreeResult;
      try {
        built = engine.mlBuildTree(tennis.x, tennis.y, tennis.names, treeDepth);
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }

      /** Meratakan pohon menjadi baris-baris tabel berindentasi. */
      const rows: (string | number)[][] = [];
      const walk = (node: engine.TreeNode, depth: number, edge: string): void => {
        const indent = "· ".repeat(depth);
        if (node.kind === "leaf") {
          rows.push([
            `${indent}${edge}`,
            `→ ${node.label}`,
            String(node.samples),
            fmt(node.purity, 2),
          ]);
          return;
        }
        rows.push([
          `${indent}${edge}`,
          `${node.attribute_name}?`,
          "",
          fmt(node.gain, 3),
        ]);
        for (const [value, child] of Object.entries(node.children)) {
          walk(child, depth + 1, value);
        }
      };
      walk(built.tree, 0, pick(bi("akar", "root")));

      const terurut = built.gains.slice().sort((a, b) => b[1] - a[1]);
      const juara = terurut[0];

      output.append(
        card(
          pick(bi("Perolehan informasi tiap atribut", "Information gain per attribute")),
          figure({
            title: bi("Siapa yang dipilih ID3, dan seberapa unggul", "What ID3 picks, and by how much"),
            summary: bi(
              `Entropi sebelum dipecah ${fmt(built.root_entropy, 3)} bit. ID3 memilih ` +
                `"${juara?.[0] ?? "—"}" karena ia paling banyak mengurangi ketidakpastian. ` +
                `Yang layak diperhatikan bukan pemenangnya melainkan jaraknya: kalau dua batang ` +
                `teratas nyaris sama panjang, pilihan pohon ini rapuh — data latih yang sedikit ` +
                `berbeda akan menghasilkan pohon yang sama sekali lain.`,
              `Entropy before splitting is ${fmt(built.root_entropy, 3)} bits. ID3 picks ` +
                `"${juara?.[0] ?? "—"}" because it removes the most uncertainty. What matters is ` +
                `not the winner but the margin: if the top two bars are nearly equal, this tree's ` +
                `choice is fragile — slightly different training data would grow a very ` +
                `different tree.`,
            ),
            body: rankedBars(
              terurut.map(([name, gain], i) => ({
                label: name,
                value: gain,
                highlight: i === 0,
                detail: `${pct(gain / Math.max(1e-12, built.root_entropy), 0)} ${pick(bi("dari entropi", "of entropy"))}`,
              })),
              (v) => `${fmt(v, 4)} bit`,
            ),
          }),
          table(
            [
              pick(bi("Atribut", "Attribute")),
              pick(bi("Perolehan (bit)", "Gain (bits)")),
            ],
            built.gains
              .slice()
              .sort((a, b) => b[1] - a[1])
              .map(([name, gain]) => [name, gain]),
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                `Entropi sebelum dipecah: ${fmt(built.root_entropy, 3)} bit. ID3 memilih atribut dengan perolehan tertinggi, yaitu yang paling banyak mengurangi ketidakpastian.`,
                `Entropy before splitting: ${fmt(built.root_entropy, 3)} bits. ID3 picks the attribute with the highest gain — the one that removes the most uncertainty.`,
              ),
            ),
          }),
        ),
        card(
          pick(bi("Pohon keputusan", "Decision tree")),
          table(
            [
              pick(bi("Cabang", "Branch")),
              pick(bi("Uji / hasil", "Test / outcome")),
              pick(bi("Data", "Samples")),
              pick(bi("Perolehan / kemurnian", "Gain / purity")),
            ],
            rows,
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                `Kedalaman ${built.depth}, ${built.leaves} daun. Turunkan batas kedalaman dan perhatikan pohonnya memangkas diri — pohon yang lebih kecil sering lebih tahan terhadap data baru meski lebih sering salah pada data latihnya.`,
                `Depth ${built.depth}, ${built.leaves} leaves. Lower the depth limit and watch the tree prune itself — a smaller tree often generalises better even though it makes more mistakes on the training data.`,
              ),
            ),
          }),
        ),
      );
    }

    function renderRegression(): void {
      if (points.length < 2) {
        output.append(
          errorNote(pick(bi("Butuh minimal dua titik.", "At least two points are needed."))),
        );
        return;
      }
      let model: engine.LinearRegression;
      try {
        model = engine.mlFitLinear(
          points.map((p) => p.x),
          points.map((p) => p.y),
        );
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }
      output.append(
        card(
          pick(T.result),
          table(
            [pick(bi("Besaran", "Quantity")), pick(bi("Nilai", "Value"))],
            [
              [pick(bi("Kemiringan", "Slope")), model.slope],
              [pick(bi("Titik potong", "Intercept")), model.intercept],
              ["R²", model.r_squared],
              [
                pick(bi("Persamaan", "Equation")),
                `y = ${fmt(model.intercept, 3)} + ${fmt(model.slope, 3)}x`,
              ],
            ],
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "R² menyatakan berapa bagian keragaman y yang dijelaskan garis ini. Nilainya tinggi bukan berarti hubungannya sebab-akibat, dan garis lurus tetap bisa terlihat cocok pada data yang sebenarnya melengkung.",
                "R² states how much of the variation in y this line explains. A high value does not mean causation, and a straight line can still look like a good fit on data that actually curves.",
              ),
            ),
          }),
        ),
      );
    }

    function render(): void {
      clear(output);
      drawPlane();
      switch (tab) {
        case "knn":
          renderKnn();
          break;
        case "kmeans":
          renderKmeans();
          break;
        case "tree":
          renderTree();
          break;
        case "regression":
          renderRegression();
          break;
      }
    }

    canvas.addEventListener("pointerdown", (event) => {
      if (tab === "tree") return;
      const rect = canvas.getBoundingClientRect();
      const x = ((event.clientX - rect.left) / rect.width) * (RANGE[1] - RANGE[0]) + RANGE[0];
      const y =
        (1 - (event.clientY - rect.top) / rect.height) * (RANGE[1] - RANGE[0]) + RANGE[0];
      if (x < RANGE[0] || x > RANGE[1] || y < RANGE[0] || y > RANGE[1]) return;
      points.push({ x, y, label: brush });
      render();
    });

    function renderControls(): void {
      clear(controls);

      const tabs = buttonRow(
        (
          [
            ["knn", bi("KNN", "KNN")],
            ["kmeans", bi("K-Means", "K-Means")],
            ["tree", bi("Pohon Keputusan", "Decision Tree")],
            ["regression", bi("Regresi", "Regression")],
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

      const shared: HTMLElement[] = [];

      if (tab === "knn") {
        shared.push(
          card(
            pick(T.controls),
            slider({
              label: "k",
              min: 1,
              max: Math.max(1, Math.min(15, points.length)),
              step: 1,
              value: Math.min(k, Math.max(1, points.length)),
              format: (v) => String(v),
              onInput: (v) => {
                k = v;
                render();
              },
            }),
            buttonRow(
              (["euclidean", "manhattan", "chebyshev"] as engine.Distance[]).map((d) => ({
                label: d,
                selected: d === distance,
                onClick: () => {
                  distance = d;
                  renderControls();
                  render();
                },
              })),
            ),
            buttonRow([
              {
                label: weighted
                  ? pick(bi("Suara berbobot: nyala", "Weighted votes: on"))
                  : pick(bi("Suara berbobot: mati", "Weighted votes: off")),
                onClick: () => {
                  weighted = !weighted;
                  renderControls();
                  render();
                },
              },
            ]),
            el("p", {
              class: "note",
              text: pick(
                bi(
                  "k = 1 menghafal setiap titik, termasuk deraunya. k sebesar jumlah data melumatkan seluruh batas menjadi satu kelas. Yang menarik ada di antaranya.",
                  "k = 1 memorises every point, noise included. k as large as the dataset flattens every boundary into one class. The interesting behaviour lives in between.",
                ),
              ),
            }),
          ),
        );
      }

      if (tab === "kmeans") {
        shared.push(
          card(
            pick(T.controls),
            slider({
              label: pick(bi("Jumlah kelompok", "Clusters")),
              min: 1,
              max: Math.max(1, Math.min(6, points.length)),
              step: 1,
              value: Math.min(clusters, Math.max(1, points.length)),
              format: (v) => String(v),
              onInput: (v) => {
                clusters = v;
                render();
              },
            }),
            buttonRow([
              {
                label: pick(bi("Benih baru", "New seed")),
                onClick: () => {
                  seed = Math.floor(Math.random() * 100000) + 1;
                  render();
                },
              },
            ]),
            el("p", {
              class: "note",
              text: pick(
                bi(
                  "K-Means tidak pernah melihat label. Warnanya di sini adalah kelompok yang ia temukan sendiri, bukan kelas yang Anda berikan.",
                  "K-Means never sees the labels. The colours here are the clusters it found on its own, not the classes you assigned.",
                ),
              ),
            }),
          ),
        );
      }

      if (tab === "tree") {
        shared.push(
          card(
            pick(T.controls),
            slider({
              label: pick(bi("Batas kedalaman", "Depth limit")),
              min: 1,
              max: 6,
              step: 1,
              value: treeDepth,
              format: (v) => String(v),
              onInput: (v) => {
                treeDepth = v;
                render();
              },
            }),
            el("p", {
              class: "note",
              text: pick(
                bi(
                  "Memakai kumpulan data “bermain tenis” klasik: 14 baris, empat atribut. Bidang gambar tidak dipakai pada tab ini karena atributnya kategorikal, bukan koordinat.",
                  "Uses the classic “play tennis” dataset: 14 rows, four attributes. The plotting area is unused on this tab because the attributes are categorical, not coordinates.",
                ),
              ),
            }),
          ),
        );
      }

      if (tab === "regression") {
        shared.push(
          card(
            pick(T.controls),
            el("p", {
              class: "note",
              text: pick(
                bi(
                  "Klik di mana saja untuk menambah titik. Label kelas diabaikan pada tab ini; yang dicocokkan adalah hubungan antara sumbu mendatar dan tegak.",
                  "Click anywhere to add a point. Class labels are ignored on this tab; what is fitted is the relation between the horizontal and vertical axes.",
                ),
              ),
            }),
          ),
        );
      }

      controls.append(
        card(pick(bi("Algoritma", "Algorithm")), tabs),
        ...shared,
        card(
          pick(bi("Data", "Data")),
          tab === "tree"
            ? el("p", {
                class: "note",
                text: pick(
                  bi(
                    "Tab ini memakai kumpulan data tetap.",
                    "This tab uses a fixed dataset.",
                  ),
                ),
              })
            : buttonRow(
                CLASSES.map((c) => ({
                  label: `${pick(bi("Kelas", "Class"))} ${c}`,
                  selected: c === brush,
                  onClick: () => {
                    brush = c;
                    renderControls();
                  },
                })),
              ),
          buttonRow([
            {
              label: pick(bi("Contoh awal", "Seed data")),
              onClick: () => {
                points = seedPoints();
                render();
              },
            },
            {
              label: pick(bi("Kosongkan", "Clear")),
              onClick: () => {
                points = [];
                render();
              },
            },
          ]),
          el("p", {
            class: "note",
            text: pick(
              bi(
                `${points.length} titik. Klik pada bidang di sebelah kanan untuk menambah.`,
                `${points.length} points. Click the plane on the right to add more.`,
              ),
            ),
          }),
        ),
      );
    }

    root.append(
      el("div", {
        class: "grid-2",
        children: [controls, card(pick(bi("Bidang data", "Data plane")), canvas)],
      }),
      output,
    );
    renderControls();
    render();

    const observer = new MutationObserver(() => drawPlane());
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });

    return () => {
      observer.disconnect();
      clear(root);
    };
}
