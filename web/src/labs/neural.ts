/**
 * Laboratorium Sesi 9 — Jaringan Syaraf Tiruan.
 *
 * Jaringan dilatih sungguhan di dalam peramban, dan Anda melihatnya belajar:
 * batas keputusan bergeser, kurva galat menurun, bobot berubah tebal-tipis.
 *
 * Pelatihan dipotong menjadi beberapa epoch per bingkai lalu dikembalikan ke
 * penjadwal tampilan. Tanpa itu, seribu epoch akan membekukan tab sampai
 * selesai — jaringan tetap terlatih, tetapi tidak ada yang bisa dilihat.
 */

import * as engine from "../engine.js";
import { T, bi, pick } from "../i18n.js";
import { buttonRow, card, clear, el, errorNote, fmt, pct, slider, table } from "../ui.js";

type DatasetName = "xor" | "and" | "or" | "spiral";

/** Susunan lapisan tersembunyi yang bisa dipilih. */
const ARCHITECTURES: { label: string; hidden: number[] }[] = [
  { label: "4", hidden: [4] },
  { label: "8", hidden: [8] },
  { label: "16-16", hidden: [16, 16] },
  { label: "24-16-8", hidden: [24, 16, 8] },
];

/** Berapa epoch dikerjakan sebelum tampilan diberi kesempatan menggambar. */
const EPOCHS_PER_FRAME = 8;

/** Resolusi kisi batas keputusan. Lebih halus berarti lebih berat. */
const GRID_RESOLUTION = 72;

/**
 * Memasang laboratorium ke dalam elemen yang diberikan.
 *
 * Keterangannya -- judul, nomor sesi, penjelasan -- ada di
 * `labs/registry.ts`, bukan di sini, supaya daftar isi bisa ditampilkan
 * tanpa mengunduh mesin seluruh laboratorium lebih dulu.
 */
export function mount(root: HTMLElement): () => void {
    let datasetName: DatasetName = "spiral";
    let points = 60;
    let noise = 0.04;
    let hidden = [16, 16];
    let learningRate = 0.08;
    let momentum = 0.9;
    let seed = 5;

    let data: engine.Dataset = engine.neuralDataset(datasetName, points, noise, seed);
    let summary: engine.NetworkSummary | null = null;
    let history: engine.EpochRecord[] = [];
    let running = false;
    let frame = 0;
    let targetEpochs = 2000;

    const boundaryCanvas = el("canvas", {
      attrs: {
        role: "img",
        "aria-label": pick(
          bi(
            "Batas keputusan jaringan beserta titik-titik data latih.",
            "The network decision boundary with the training points.",
          ),
        ),
      },
    });
    const lossCanvas = el("canvas", {
      attrs: {
        role: "img",
        "aria-label": pick(
          bi("Kurva galat dan ketepatan per epoch.", "Loss and accuracy per epoch."),
        ),
      },
    });
    const controls = el("div");
    const output = el("div");

    /** Rentang tampilan; spiral memakai koordinat sekitar -1 sampai 1. */
    function range(): [number, number] {
      return datasetName === "spiral" ? [-1.15, 1.15] : [-0.2, 1.2];
    }

    function prepare(
      canvas: HTMLCanvasElement,
      w: number,
      h: number,
    ): CanvasRenderingContext2D | null {
      const dpr = Math.min(globalThis.devicePixelRatio || 1, 2);
      canvas.width = Math.round(w * dpr);
      canvas.height = Math.round(h * dpr);
      canvas.style.aspectRatio = `${w} / ${h}`;
      const ctx = canvas.getContext("2d");
      if (!ctx) return null;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.clearRect(0, 0, w, h);
      return ctx;
    }

    /** Warna dari nilai keluaran: satu ujung ke ujung lain lewat netral. */
    function tone(v: number, accent: [number, number, number], warn: [number, number, number]) {
      const t = Math.min(Math.max(v, 0), 1);
      const mix = (a: number, b: number) => Math.round(a + (b - a) * t);
      return `rgb(${mix(warn[0], accent[0])} ${mix(warn[1], accent[1])} ${mix(warn[2], accent[2])})`;
    }

    function drawBoundary(): void {
      const SIZE = 460;
      const ctx = prepare(boundaryCanvas, SIZE, SIZE);
      if (!ctx) return;
      const [min, max] = range();

      const style = getComputedStyle(document.documentElement);
      const bg = style.getPropertyValue("--bg-grid").trim() || "#0d131c";
      ctx.fillStyle = bg;
      ctx.fillRect(0, 0, SIZE, SIZE);

      if (summary) {
        let grid: engine.DecisionGrid | null = null;
        try {
          grid = engine.neuralDecisionGrid(summary.network, min, max, GRID_RESOLUTION);
        } catch {
          grid = null;
        }
        if (grid) {
          const cell = SIZE / grid.resolution;
          // Palet tetap dipakai di sini agar kedua kelas selalu terbedakan,
          // termasuk pada mode terang.
          const accent: [number, number, number] = [77, 212, 200];
          const warn: [number, number, number] = [240, 180, 41];
          ctx.globalAlpha = 0.4;
          for (let j = 0; j < grid.resolution; j++) {
            for (let i = 0; i < grid.resolution; i++) {
              ctx.fillStyle = tone(grid.values[j * grid.resolution + i], accent, warn);
              ctx.fillRect(i * cell, SIZE - (j + 1) * cell, cell + 0.6, cell + 0.6);
            }
          }
          ctx.globalAlpha = 1;
        }
      }

      // Titik data digambar di atas batas keputusan.
      const toPx = (v: number) => ((v - min) / (max - min)) * SIZE;
      data.x.forEach((p, i) => {
        const target = data.y[i];
        const first = target.length === 1 ? target[0] >= 0.5 : target[0] > target[1];
        ctx.beginPath();
        ctx.arc(toPx(p[0]), SIZE - toPx(p[1]), 3.6, 0, Math.PI * 2);
        ctx.fillStyle = first ? "#4dd4c8" : "#f0b429";
        ctx.fill();
        ctx.lineWidth = 1;
        ctx.strokeStyle = "rgb(0 0 0 / 55%)";
        ctx.stroke();
      });
    }

    function drawLoss(): void {
      const W = 460;
      const H = 150;
      const ctx = prepare(lossCanvas, W, H);
      if (!ctx || history.length < 2) return;

      const style = getComputedStyle(document.documentElement);
      const accent = style.getPropertyValue("--accent").trim() || "#4dd4c8";
      const warn = style.getPropertyValue("--warn").trim() || "#f0b429";
      const grid = style.getPropertyValue("--border").trim() || "#1f2b3c";

      ctx.strokeStyle = grid;
      ctx.lineWidth = 1;
      ctx.strokeRect(0.5, 0.5, W - 1, H - 1);

      const maxLoss = Math.max(...history.map((h) => h.loss), 1e-6);
      const n = history.length;
      const xAt = (i: number) => (i / Math.max(n - 1, 1)) * (W - 8) + 4;

      // Galat digambar pada skala logaritmik: penurunan dari 0,25 ke 0,001
      // tidak akan terlihat sama sekali pada skala linear.
      const logAt = (v: number) => {
        const lo = Math.log10(1e-5);
        const hi = Math.log10(Math.max(maxLoss, 1e-4));
        const t = (Math.log10(Math.max(v, 1e-5)) - lo) / Math.max(hi - lo, 1e-9);
        return H - 6 - Math.min(Math.max(t, 0), 1) * (H - 12);
      };

      ctx.strokeStyle = accent;
      ctx.lineWidth = 2;
      ctx.beginPath();
      history.forEach((h, i) => {
        const y = logAt(h.loss);
        if (i === 0) ctx.moveTo(xAt(i), y);
        else ctx.lineTo(xAt(i), y);
      });
      ctx.stroke();

      ctx.strokeStyle = warn;
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      history.forEach((h, i) => {
        const y = H - 6 - h.accuracy * (H - 12);
        if (i === 0) ctx.moveTo(xAt(i), y);
        else ctx.lineTo(xAt(i), y);
      });
      ctx.stroke();
    }

    function renderOutput(): void {
      clear(output);
      if (!summary) return;
      const last = history[history.length - 1];

      output.append(
        card(
          pick(T.result),
          table(
            [pick(bi("Ukuran", "Measure")), pick(bi("Nilai", "Value"))],
            [
              [pick(bi("Epoch dijalankan", "Epochs run")), String(history.length)],
              [
                pick(bi("Galat", "Loss")),
                last ? last.loss.toExponential(3) : "—",
              ],
              [pick(bi("Ketepatan", "Accuracy")), last ? pct(last.accuracy, 2) : "—"],
              [pick(bi("Parameter", "Parameters")), String(summary.parameters)],
              [
                pick(bi("Laju belajar efektif", "Effective learning rate")),
                fmt(summary.effective_learning_rate, 3),
              ],
            ],
          ),
          summary.step_risky
            ? errorNote(
                pick(
                  bi(
                    `Langkah efektifnya ${fmt(summary.effective_learning_rate, 2)}. Momentum memperbesar laju belajar kira-kira 1/(1-momentum) kali, jadi momentum 0,9 mengalikannya sepuluh kali. Di atas 1,0 pelatihan cenderung berayun alih-alih menurun — pengukuran pada spiral menunjukkan jaringan yang sama macet di 50 persen pada langkah 2,0, padahal tuntas dalam 70 epoch pada langkah 0,8.`,
                    `The effective step is ${fmt(summary.effective_learning_rate, 2)}. Momentum amplifies the learning rate by roughly 1/(1-momentum), so 0.9 multiplies it tenfold. Above 1.0 training tends to oscillate rather than descend — on the spiral, the same network stalls at 50 percent with a step of 2.0 but finishes in 70 epochs at 0.8.`,
                  ),
                ),
              )
            : el("p", {
                class: "note",
                text: pick(
                  bi(
                    "Garis hijau adalah galat pada skala logaritmik; garis kuning adalah ketepatan. Naikkan momentum dan perhatikan angka laju efektif di atas.",
                    "The green line is loss on a logarithmic scale; the amber line is accuracy. Raise the momentum and watch the effective rate above.",
                  ),
                ),
              }),
        ),
        card(pick(bi("Kurva belajar", "Learning curve")), lossCanvas),
      );
      drawLoss();
    }

    function stop(): void {
      running = false;
      if (frame) {
        cancelAnimationFrame(frame);
        frame = 0;
      }
    }

    function rebuild(): void {
      stop();
      history = [];
      data = engine.neuralDataset(datasetName, points, noise, seed);
      const outputs = data.y[0]?.length ?? 1;
      try {
        summary = engine.neuralCreate(
          [2, ...hidden, outputs],
          "tanh",
          "sigmoid",
          learningRate,
          momentum,
          seed,
        );
      } catch (error) {
        summary = null;
        clear(output);
        output.append(errorNote(String((error as Error).message)));
        return;
      }
      renderOutput();
      drawBoundary();
    }

    function step(): void {
      if (!running || !summary) return;
      try {
        const result = engine.neuralTrain(
          summary.network,
          data,
          EPOCHS_PER_FRAME,
          1e-5,
          seed,
        );
        summary = result.summary;
        // Nomor epoch datang ulang dari nol tiap potongan, jadi diberi nomor
        // menerus di sini supaya kurvanya bersambung.
        const base = history.length;
        for (const record of result.history) {
          history.push({ ...record, epoch: base + record.epoch });
        }
      } catch (error) {
        stop();
        clear(output);
        output.append(errorNote(String((error as Error).message)));
        return;
      }

      drawBoundary();
      renderOutput();
      // Panel kontrol sengaja TIDAK digambar ulang di sini. Menggambarnya tiap
      // bingkai berarti membuang dan membuat ulang seluruh penggeser enam
      // puluh kali sedetik, yang membuat penggeser yang sedang diseret
      // terlepas dari jari pengguna dan mengembalikan fokus papan ketik ke
      // awal. Yang berubah tiap bingkai hanya gambar dan angka hasil.
      updateRunLabel();

      const done =
        history.length >= targetEpochs ||
        (history[history.length - 1]?.loss ?? 1) <= 1e-5;
      if (done) {
        stop();
        updateRunLabel();
        return;
      }
      frame = requestAnimationFrame(step);
    }

    /** Tombol jalan/berhenti, disimpan agar labelnya bisa diperbarui sendiri. */
    let runButton: HTMLButtonElement | null = null;

    /** Menyegarkan label tombol tanpa menggambar ulang seluruh panel. */
    function updateRunLabel(): void {
      if (!runButton) return;
      const label = running ? pick(bi("Berhenti", "Stop")) : pick(bi("Latih", "Train"));
      if (runButton.textContent !== label) runButton.textContent = label;
    }

    function renderControls(): void {
      clear(controls);
      runButton = null;
      controls.append(
        card(
          pick(bi("Kumpulan data", "Dataset")),
          buttonRow(
            (["spiral", "xor", "and", "or"] as DatasetName[]).map((name) => ({
              label: name.toUpperCase(),
              selected: name === datasetName,
              onClick: () => {
                datasetName = name;
                rebuild();
                renderControls();
              },
            })),
          ),
          datasetName === "spiral"
            ? slider({
                label: pick(bi("Titik per kelas", "Points per class")),
                min: 20,
                max: 150,
                step: 10,
                value: points,
                format: (v) => String(v),
                onInput: (v) => {
                  points = v;
                  rebuild();
                },
              })
            : el("p", {
                class: "note",
                text: pick(
                  bi(
                    "Gerbang logika hanya punya empat titik. XOR adalah yang tidak bisa dipisahkan satu garis — itulah alasan lapisan tersembunyi ada.",
                    "Logic gates have only four points. XOR is the one no single line can separate — which is why hidden layers exist.",
                  ),
                ),
              }),
        ),
        card(
          pick(bi("Lapisan tersembunyi", "Hidden layers")),
          buttonRow(
            ARCHITECTURES.map((a) => ({
              label: a.label,
              selected: a.hidden.join("-") === hidden.join("-"),
              onClick: () => {
                hidden = [...a.hidden];
                rebuild();
                renderControls();
              },
            })),
          ),
        ),
        card(
          pick(T.controls),
          slider({
            label: pick(bi("Laju belajar", "Learning rate")),
            min: 0.005,
            max: 0.4,
            step: 0.005,
            value: learningRate,
            format: (v) => fmt(v, 3),
            onInput: (v) => {
              learningRate = v;
              rebuild();
              renderControls();
            },
          }),
          slider({
            label: pick(bi("Momentum", "Momentum")),
            min: 0,
            max: 0.95,
            step: 0.05,
            value: momentum,
            format: (v) => fmt(v, 2),
            onInput: (v) => {
              momentum = v;
              rebuild();
              renderControls();
            },
          }),
          slider({
            label: pick(bi("Batas epoch", "Epoch budget")),
            min: 200,
            max: 5000,
            step: 100,
            value: targetEpochs,
            format: (v) => String(v),
            onInput: (v) => {
              targetEpochs = v;
            },
          }),
          buttonRow([
            {
              label: running
                ? pick(bi("Berhenti", "Stop"))
                : pick(bi("Latih", "Train")),
              primary: true,
              onClick: () => {
                if (running) {
                  stop();
                  updateRunLabel();
                  return;
                }
                if (!summary) rebuild();
                running = true;
                updateRunLabel();
                frame = requestAnimationFrame(step);
              },
            },
            {
              label: pick(bi("Ulang dari awal", "Reset")),
              onClick: () => {
                rebuild();
                renderControls();
              },
            },
            {
              label: pick(bi("Benih baru", "New seed")),
              onClick: () => {
                seed = Math.floor(Math.random() * 100000) + 1;
                rebuild();
                renderControls();
              },
            },
          ]),
        ),
      );

      // Tombol jalan/berhenti adalah tombol utama pertama di panel.
      runButton = controls.querySelector<HTMLButtonElement>("button.btn--primary");
    }

    root.append(
      el("div", {
        class: "grid-2",
        children: [card(pick(bi("Batas keputusan", "Decision boundary")), boundaryCanvas), controls],
      }),
      output,
    );

    rebuild();
    renderControls();

    const observer = new MutationObserver(() => {
      drawBoundary();
      drawLoss();
    });
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });

    return () => {
      stop();
      observer.disconnect();
      clear(root);
    };
}
