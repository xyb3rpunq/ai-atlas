/**
 * Laboratorium Sesi 8 — Teknik Pencarian dan Pelacakan.
 *
 * Pengguna menggambar sendiri dindingnya, memindahkan titik awal dan tujuan,
 * lalu menjalankan sembilan algoritma di atas peta yang sama. Yang dianimasikan
 * bukan hanya jalurnya, melainkan **urutan simpul yang dibuka** — di situlah
 * perbedaan antaralgoritma sebenarnya terlihat. Dua algoritma bisa menemukan
 * jalur sama panjang sambil memeriksa jumlah sel yang berbeda berkali lipat.
 */

import * as engine from "../engine.js";
import { T, bi, pick } from "../i18n.js";
import { buttonRow, card, clear, el, errorNote, fmt, slider, table } from "../ui.js";

const WIDTH = 31;
const HEIGHT = 21;

/** Alat yang sedang dipegang saat menyeret di atas kanvas. */
type Tool = "wall" | "erase" | "start" | "goal";

const ALGORITHMS: { slug: engine.Algorithm; label: string }[] = [
  { slug: "breadth_first", label: "BFS" },
  { slug: "depth_first", label: "DFS" },
  { slug: "depth_limited", label: "DLS" },
  { slug: "iterative_deepening", label: "IDDFS" },
  { slug: "uniform_cost", label: "UCS" },
  { slug: "greedy_best_first", label: "Greedy" },
  { slug: "a_star", label: "A*" },
  { slug: "hill_climbing", label: "Hill Climbing" },
  { slug: "simulated_annealing", label: "Annealing" },
];

const HEURISTICS: engine.Heuristic[] = ["manhattan", "euclidean", "chebyshev", "zero"];

/**
 * Memasang laboratorium ke dalam elemen yang diberikan.
 *
 * Keterangannya -- judul, nomor sesi, penjelasan -- ada di
 * `labs/registry.ts`, bukan di sini, supaya daftar isi bisa ditampilkan
 * tanpa mengunduh mesin seluruh laboratorium lebih dulu.
 */
export function mount(root: HTMLElement): () => void {
    let grid: engine.Grid = engine.searchMaze(WIDTH, HEIGHT, 2026);
    let start = { x: 0, y: 0 };
    let goal = { x: WIDTH - 1, y: HEIGHT - 1 };
    let algorithm: engine.Algorithm = "a_star";
    let heuristic: engine.Heuristic = "manhattan";
    let depthLimit = 200;
    let speed = 6;
    let result: engine.SearchResult | null = null;
    let revealed = 0;
    let animation = 0;
    let tool: Tool = "wall";
    let painting = false;

    const canvas = el("canvas", {
      attrs: {
        role: "img",
        "aria-label": pick(
          bi(
            "Peta pencarian. Sel yang sudah diperiksa diwarnai, jalur akhir ditandai garis terang.",
            "The search map. Examined cells are shaded and the final path is highlighted.",
          ),
        ),
      },
    });
    const controls = el("div");
    const output = el("div");

    /** Ukuran satu sel dalam piksel CSS. */
    const CELL = 18;

    function draw(): void {
      const W = WIDTH * CELL;
      const H = HEIGHT * CELL;
      const dpr = Math.min(globalThis.devicePixelRatio || 1, 2);
      canvas.width = Math.round(W * dpr);
      canvas.height = Math.round(H * dpr);
      canvas.style.aspectRatio = `${W} / ${H}`;
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

      const s = getComputedStyle(document.documentElement);
      const bg = s.getPropertyValue("--bg-grid").trim() || "#0d131c";
      const wall = s.getPropertyValue("--border-strong").trim() || "#2c3d54";
      const accent = s.getPropertyValue("--accent").trim() || "#4dd4c8";
      const warn = s.getPropertyValue("--warn").trim() || "#f0b429";
      const danger = s.getPropertyValue("--danger").trim() || "#f2686c";

      ctx.fillStyle = bg;
      ctx.fillRect(0, 0, W, H);

      ctx.fillStyle = wall;
      for (let y = 0; y < HEIGHT; y++) {
        for (let x = 0; x < WIDTH; x++) {
          if (grid.walls[y * WIDTH + x]) {
            ctx.fillRect(x * CELL, y * CELL, CELL - 1, CELL - 1);
          }
        }
      }

      // Sel yang sudah diperiksa memudar dari yang terbaru ke yang terlama,
      // sehingga arah rambat pencarian terbaca sekali lihat.
      if (result) {
        const shown = Math.min(revealed, result.expanded.length);
        for (let i = 0; i < shown; i++) {
          const p = result.expanded[i];
          const age = shown === 0 ? 0 : i / shown;
          ctx.globalAlpha = 0.18 + age * 0.42;
          ctx.fillStyle = warn;
          ctx.fillRect(p.x * CELL, p.y * CELL, CELL - 1, CELL - 1);
        }
        ctx.globalAlpha = 1;

        if (shown >= result.expanded.length && result.found) {
          ctx.strokeStyle = accent;
          ctx.lineWidth = 3;
          ctx.lineJoin = "round";
          ctx.lineCap = "round";
          ctx.beginPath();
          result.path.forEach((p, i) => {
            const cx = p.x * CELL + CELL / 2 - 0.5;
            const cy = p.y * CELL + CELL / 2 - 0.5;
            if (i === 0) ctx.moveTo(cx, cy);
            else ctx.lineTo(cx, cy);
          });
          ctx.stroke();
        }
      }

      ctx.fillStyle = accent;
      ctx.fillRect(start.x * CELL, start.y * CELL, CELL - 1, CELL - 1);
      ctx.fillStyle = danger;
      ctx.fillRect(goal.x * CELL, goal.y * CELL, CELL - 1, CELL - 1);
    }

    function stopAnimation(): void {
      if (animation) {
        cancelAnimationFrame(animation);
        animation = 0;
      }
    }

    function animate(): void {
      stopAnimation();
      if (!result) return;
      revealed = 0;
      const step = () => {
        if (!result) return;
        // Kecepatan diukur dalam sel per bingkai, bukan penundaan waktu, supaya
        // animasinya tetap halus di layar berfrekuensi tinggi maupun rendah.
        revealed = Math.min(revealed + speed, result.expanded.length);
        draw();
        if (revealed < result.expanded.length) {
          animation = requestAnimationFrame(step);
        } else {
          animation = 0;
        }
      };
      animation = requestAnimationFrame(step);
    }

    function run(): void {
      clear(output);
      try {
        result = engine.searchRun(grid, start, goal, {
          algorithm,
          heuristic,
          depth_limit: depthLimit,
          seed: 0x5eed,
          max_expansions: 200_000,
        });
      } catch (error) {
        result = null;
        output.append(errorNote(String((error as Error).message)));
        draw();
        return;
      }

      const r = result;
      output.append(
        card(
          pick(T.result),
          table(
            [pick(bi("Ukuran", "Measure")), pick(bi("Nilai", "Value"))],
            [
              [
                pick(bi("Tujuan tercapai", "Goal reached")),
                r.found ? pick(bi("ya", "yes")) : pick(bi("tidak", "no")),
              ],
              [
                pick(bi("Panjang jalur", "Path length")),
                r.found ? String(Math.max(r.path.length - 1, 0)) : "—",
              ],
              [pick(bi("Biaya jalur", "Path cost")), r.found ? fmt(r.cost, 2) : "—"],
              [pick(bi("Sel diperiksa", "Cells examined")), String(r.expansions)],
              [
                pick(bi("Daftar tunggu terbesar", "Peak frontier")),
                String(r.peak_frontier),
              ],
            ],
          ),
          el("p", {
            class: "note",
            text: r.found
              ? pick(
                  bi(
                    `Jalur ditemukan setelah memeriksa ${r.expansions} sel. Coba algoritma lain pada peta yang sama dan perhatikan angka itu, bukan panjang jalurnya.`,
                    `A path was found after examining ${r.expansions} cells. Try another algorithm on the same map and watch that number, not the path length.`,
                  ),
                )
              : pick(
                  bi(
                    "Tidak ada jalur yang ditemukan. Pada hill climbing hal ini wajar walau jalurnya ada — algoritma itu berhenti begitu tidak ada langkah yang memperkecil jarak.",
                    "No path was found. For hill climbing this happens even when a path exists — it stops as soon as no step reduces the distance.",
                  ),
                ),
          }),
        ),
      );

      let rows: engine.CompareRow[] = [];
      try {
        rows = engine.searchCompare(grid, start, goal, {
          algorithm,
          heuristic,
          depth_limit: depthLimit,
          seed: 0x5eed,
          max_expansions: 200_000,
        });
      } catch {
        /* Perbandingan bersifat tambahan; kegagalannya tidak menghalangi hasil utama. */
      }

      if (rows.length > 0) {
        output.append(
          card(
            pick(bi("Sembilan algoritma, satu peta", "Nine algorithms, one map")),
            table(
              [
                pick(bi("Algoritma", "Algorithm")),
                pick(bi("Optimal", "Optimal")),
                pick(bi("Ketemu", "Found")),
                pick(bi("Langkah", "Steps")),
                pick(bi("Diperiksa", "Examined")),
              ],
              rows.map((row) => [
                row.name,
                row.optimal ? "✓" : "—",
                row.found ? "✓" : "—",
                row.found ? String(row.steps) : "—",
                row.expansions,
              ]),
            ),
            el("p", {
              class: "note",
              text: pick(
                bi(
                  "Kolom “Optimal” menyatakan jaminan teoretis, bukan hasil sekali jalan. Algoritma tak optimal kadang beruntung menemukan jalur terpendek — yang membedakan adalah apakah keberuntungan itu bisa diandalkan.",
                  "The “Optimal” column states a theoretical guarantee, not the outcome of one run. A non-optimal algorithm sometimes gets lucky — what differs is whether that luck can be relied upon.",
                ),
              ),
            }),
          ),
        );
      }

      animate();
    }

    /** Mengubah posisi penunjuk menjadi koordinat sel. */
    function cellAt(event: PointerEvent): { x: number; y: number } | null {
      const rect = canvas.getBoundingClientRect();
      const x = Math.floor(((event.clientX - rect.left) / rect.width) * WIDTH);
      const y = Math.floor(((event.clientY - rect.top) / rect.height) * HEIGHT);
      if (x < 0 || y < 0 || x >= WIDTH || y >= HEIGHT) return null;
      return { x, y };
    }

    function applyTool(cell: { x: number; y: number }): void {
      const i = cell.y * WIDTH + cell.x;
      const isEndpoint =
        (cell.x === start.x && cell.y === start.y) || (cell.x === goal.x && cell.y === goal.y);
      switch (tool) {
        case "wall":
          if (!isEndpoint) grid.walls[i] = true;
          break;
        case "erase":
          grid.walls[i] = false;
          break;
        case "start":
          if (!grid.walls[i]) start = cell;
          break;
        case "goal":
          if (!grid.walls[i]) goal = cell;
          break;
      }
      draw();
    }

    canvas.addEventListener("pointerdown", (event) => {
      const cell = cellAt(event);
      if (!cell) return;
      painting = true;
      canvas.setPointerCapture(event.pointerId);
      applyTool(cell);
    });
    canvas.addEventListener("pointermove", (event) => {
      if (!painting) return;
      const cell = cellAt(event);
      if (cell) applyTool(cell);
    });
    const endPaint = (): void => {
      if (!painting) return;
      painting = false;
      run();
    };
    canvas.addEventListener("pointerup", endPaint);
    canvas.addEventListener("pointercancel", endPaint);

    function renderControls(): void {
      clear(controls);
      controls.append(
        card(
          pick(bi("Algoritma", "Algorithm")),
          buttonRow(
            ALGORITHMS.map((a) => ({
              label: a.label,
              primary: a.slug === algorithm,
              onClick: () => {
                algorithm = a.slug;
                renderControls();
                run();
              },
            })),
          ),
        ),
        card(
          pick(bi("Heuristik", "Heuristic")),
          buttonRow(
            HEURISTICS.map((h) => ({
              label: h,
              primary: h === heuristic,
              onClick: () => {
                heuristic = h;
                renderControls();
                run();
              },
            })),
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Hanya berpengaruh pada Greedy, A*, hill climbing, dan annealing. Pilih “zero” untuk melihat A* merosot menjadi UCS.",
                "Only affects Greedy, A*, hill climbing, and annealing. Choose “zero” to watch A* degrade into UCS.",
              ),
            ),
          }),
        ),
        card(
          pick(bi("Alat gambar", "Drawing tool")),
          buttonRow(
            (
              [
                ["wall", bi("Dinding", "Wall")],
                ["erase", bi("Hapus", "Erase")],
                ["start", bi("Titik awal", "Start")],
                ["goal", bi("Tujuan", "Goal")],
              ] as [Tool, { id: string; en: string }][]
            ).map(([t, label]) => ({
              label: pick(label),
              primary: t === tool,
              onClick: () => {
                tool = t;
                renderControls();
              },
            })),
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Seret di atas peta untuk menggambar. Pencarian dijalankan ulang setiap kali Anda melepas.",
                "Drag across the map to draw. The search re-runs each time you release.",
              ),
            ),
          }),
        ),
        card(
          pick(T.controls),
          slider({
            label: pick(bi("Kecepatan animasi", "Animation speed")),
            min: 1,
            max: 60,
            step: 1,
            value: speed,
            format: (v) => `${v}×`,
            onInput: (v) => {
              speed = v;
            },
          }),
          slider({
            label: pick(bi("Batas kedalaman (DLS)", "Depth limit (DLS)")),
            min: 1,
            max: 400,
            step: 1,
            value: depthLimit,
            format: (v) => String(v),
            onInput: (v) => {
              depthLimit = v;
              if (algorithm === "depth_limited") run();
            },
          }),
          buttonRow([
            {
              label: pick(bi("Labirin baru", "New maze")),
              primary: true,
              onClick: () => {
                grid = engine.searchMaze(
                  WIDTH,
                  HEIGHT,
                  Math.floor(Math.random() * 0xffffff),
                );
                start = { x: 0, y: 0 };
                goal = { x: WIDTH - 1, y: HEIGHT - 1 };
                run();
              },
            },
            {
              label: pick(bi("Peta kosong", "Empty map")),
              onClick: () => {
                grid = engine.searchEmptyGrid(WIDTH, HEIGHT);
                run();
              },
            },
            {
              label: pick(bi("Jalankan lagi", "Run again")),
              onClick: run,
            },
          ]),
        ),
      );
    }

    root.append(
      card(null, canvas),
      el("div", { class: "grid-2", children: [controls, output] }),
    );
    renderControls();
    run();

    const observer = new MutationObserver(() => draw());
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });

    return () => {
      stopAnimation();
      observer.disconnect();
      clear(root);
    };
}
