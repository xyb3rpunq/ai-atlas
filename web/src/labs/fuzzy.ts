/**
 * Laboratorium Sesi 5 & 6 — Logika Fuzzy.
 *
 * Pengguna menggeser masukan tegas dan melihat tiga hal berubah serentak:
 * kurva keanggotaan beserta garis potong fuzzifikasi, daftar aturan dengan
 * derajat penyalaan masing-masing, dan daerah keluaran hasil agregasi lengkap
 * dengan titik defuzzifikasinya. Tiga mesin inferensi bisa dibandingkan
 * berdampingan pada masukan yang sama.
 */

import * as engine from "../engine.js";
import { T, bi, pick } from "../i18n.js";
import { buttonRow, card, clear, el, errorNote, fmt, readout, slider, table } from "../ui.js";

/** Sistem contoh: menentukan persen tip dari mutu pelayanan dan makanan. */
const SYSTEM: engine.FuzzySystem = {
  inputs: [
    {
      name: "Pelayanan",
      min: 0,
      max: 10,
      sets: [
        { name: "Buruk", membership: { kind: "trapezoidal", a: 0, b: 0, c: 2, d: 5 } },
        { name: "Baik", membership: { kind: "triangular", a: 0, b: 5, c: 10 } },
        { name: "Istimewa", membership: { kind: "trapezoidal", a: 5, b: 8, c: 10, d: 10 } },
      ],
    },
    {
      name: "Makanan",
      min: 0,
      max: 10,
      sets: [
        { name: "Hambar", membership: { kind: "trapezoidal", a: 0, b: 0, c: 2, d: 5 } },
        { name: "Lezat", membership: { kind: "trapezoidal", a: 5, b: 8, c: 10, d: 10 } },
      ],
    },
  ],
  output: {
    name: "Tip",
    min: 0,
    max: 25,
    sets: [
      { name: "Sedikit", membership: { kind: "triangular", a: 0, b: 5, c: 10 } },
      { name: "Sedang", membership: { kind: "triangular", a: 7.5, b: 12.5, c: 17.5 } },
      { name: "Banyak", membership: { kind: "triangular", a: 15, b: 20, c: 25 } },
    ],
  },
  rules: [
    {
      antecedents: [
        { variable: "Pelayanan", set: "Buruk" },
        { variable: "Makanan", set: "Hambar" },
      ],
      connective: "OR",
      consequent_set: "Sedikit",
      consequent_value: 5,
      weight: 1,
    },
    {
      antecedents: [{ variable: "Pelayanan", set: "Baik" }],
      connective: "AND",
      consequent_set: "Sedang",
      consequent_value: 12.5,
      weight: 1,
    },
    {
      antecedents: [
        { variable: "Pelayanan", set: "Istimewa" },
        { variable: "Makanan", set: "Lezat" },
      ],
      connective: "OR",
      consequent_set: "Banyak",
      consequent_value: 20,
      weight: 1,
    },
  ],
};

/** Warna deret, diambil dari token tema supaya ikut berganti mode. */
function palette(): string[] {
  const s = getComputedStyle(document.documentElement);
  const accent = s.getPropertyValue("--accent").trim() || "#4dd4c8";
  const warn = s.getPropertyValue("--warn").trim() || "#f0b429";
  const danger = s.getPropertyValue("--danger").trim() || "#f2686c";
  return [accent, warn, danger];
}

/** Menyiapkan kanvas pada resolusi peranti dan mengembalikan konteksnya. */
function prepare(
  canvas: HTMLCanvasElement,
  cssW: number,
  cssH: number,
): CanvasRenderingContext2D | null {
  const dpr = Math.min(globalThis.devicePixelRatio || 1, 2);
  canvas.width = Math.round(cssW * dpr);
  canvas.height = Math.round(cssH * dpr);
  canvas.style.aspectRatio = `${cssW} / ${cssH}`;
  const ctx = canvas.getContext("2d");
  if (!ctx) return null;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, cssW, cssH);
  return ctx;
}

/** Menggambar kurva-kurva keanggotaan sebuah variabel beserti garis masukan. */
function drawVariable(
  canvas: HTMLCanvasElement,
  variable: engine.FuzzyVariable,
  value: number,
): void {
  const W = 460;
  const H = 150;
  const padL = 8;
  const padB = 22;
  const ctx = prepare(canvas, W, H);
  if (!ctx) return;

  const style = getComputedStyle(document.documentElement);
  const grid = style.getPropertyValue("--border").trim() || "#1f2b3c";
  const text = style.getPropertyValue("--text-muted").trim() || "#8496ab";
  const colors = palette();

  const x2px = (x: number) =>
    padL + ((x - variable.min) / (variable.max - variable.min)) * (W - padL * 2);
  const y2px = (y: number) => H - padB - y * (H - padB - 10);

  ctx.strokeStyle = grid;
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(padL, y2px(0));
  ctx.lineTo(W - padL, y2px(0));
  ctx.stroke();

  const N = 160;
  variable.sets.forEach((set, i) => {
    ctx.strokeStyle = colors[i % colors.length];
    ctx.lineWidth = 2;
    ctx.beginPath();
    for (let k = 0; k <= N; k++) {
      const x = variable.min + ((variable.max - variable.min) * k) / N;
      const d = engine.fuzzyDegree(set.membership, x);
      const px = x2px(x);
      const py = y2px(d);
      if (k === 0) ctx.moveTo(px, py);
      else ctx.lineTo(px, py);
    }
    ctx.stroke();
  });

  // Garis tegak menandai masukan tegas, plus titik potongnya di tiap kurva.
  ctx.strokeStyle = text;
  ctx.setLineDash([3, 3]);
  ctx.beginPath();
  ctx.moveTo(x2px(value), y2px(0));
  ctx.lineTo(x2px(value), y2px(1));
  ctx.stroke();
  ctx.setLineDash([]);

  variable.sets.forEach((set, i) => {
    const d = engine.fuzzyDegree(set.membership, value);
    if (d < 0.001) return;
    ctx.fillStyle = colors[i % colors.length];
    ctx.beginPath();
    ctx.arc(x2px(value), y2px(d), 3.5, 0, Math.PI * 2);
    ctx.fill();
  });

  ctx.fillStyle = text;
  ctx.font = "11px ui-monospace, monospace";
  ctx.fillText(String(variable.min), padL, H - 6);
  ctx.textAlign = "right";
  ctx.fillText(String(variable.max), W - padL, H - 6);
  ctx.textAlign = "center";
  ctx.fillText(variable.name, W / 2, H - 6);
  ctx.textAlign = "left";
}

/** Menggambar daerah keluaran hasil agregasi dan titik defuzzifikasinya. */
function drawOutput(
  canvas: HTMLCanvasElement,
  xs: number[],
  ys: number[],
  crisp: number,
  min: number,
  max: number,
): void {
  const W = 460;
  const H = 170;
  const padL = 8;
  const padB = 24;
  const ctx = prepare(canvas, W, H);
  if (!ctx || xs.length === 0) return;

  const style = getComputedStyle(document.documentElement);
  const grid = style.getPropertyValue("--border").trim() || "#1f2b3c";
  const text = style.getPropertyValue("--text-muted").trim() || "#8496ab";
  const accent = style.getPropertyValue("--accent").trim() || "#4dd4c8";

  const x2px = (x: number) => padL + ((x - min) / (max - min)) * (W - padL * 2);
  const y2px = (y: number) => H - padB - y * (H - padB - 12);

  ctx.strokeStyle = grid;
  ctx.beginPath();
  ctx.moveTo(padL, y2px(0));
  ctx.lineTo(W - padL, y2px(0));
  ctx.stroke();

  // Daerah agregasi diarsir supaya terbaca sebagai luas, bukan sekadar garis.
  ctx.beginPath();
  ctx.moveTo(x2px(xs[0]), y2px(0));
  for (let i = 0; i < xs.length; i++) ctx.lineTo(x2px(xs[i]), y2px(ys[i]));
  ctx.lineTo(x2px(xs[xs.length - 1]), y2px(0));
  ctx.closePath();
  ctx.fillStyle = accent;
  ctx.globalAlpha = 0.22;
  ctx.fill();
  ctx.globalAlpha = 1;
  ctx.strokeStyle = accent;
  ctx.lineWidth = 2;
  ctx.stroke();

  ctx.strokeStyle = accent;
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(x2px(crisp), y2px(0));
  ctx.lineTo(x2px(crisp), y2px(1));
  ctx.stroke();

  ctx.fillStyle = text;
  ctx.font = "11px ui-monospace, monospace";
  ctx.textAlign = "center";
  ctx.fillText(fmt(crisp, 2), x2px(crisp), H - 8);
  ctx.textAlign = "left";
}

type EngineName = "mamdani" | "sugeno" | "tsukamoto";

const METHODS: engine.DefuzzMethod[] = [
  "centroid",
  "bisector",
  "mean_of_maximum",
  "smallest_of_maximum",
  "largest_of_maximum",
];

/**
 * Memasang laboratorium ke dalam elemen yang diberikan.
 *
 * Keterangannya -- judul, nomor sesi, penjelasan -- ada di
 * `labs/registry.ts`, bukan di sini, supaya daftar isi bisa ditampilkan
 * tanpa mengunduh mesin seluruh laboratorium lebih dulu.
 */
export function mount(root: HTMLElement): () => void {
    const values: Record<string, number> = { Pelayanan: 7, Makanan: 8 };
    let engineName: EngineName = "mamdani";
    let method: engine.DefuzzMethod = "centroid";

    const controls = el("div");
    const output = el("div");
    const inputCanvases = new Map<string, HTMLCanvasElement>();
    const outputCanvas = el("canvas", {
      attrs: {
        role: "img",
        "aria-label": pick(
          bi(
            "Daerah keluaran hasil agregasi aturan, dengan garis tegak menandai hasil defuzzifikasi.",
            "The aggregated output region, with a vertical line marking the defuzzified result.",
          ),
        ),
      },
    });

    function recompute(): void {
      clear(output);

      let result: engine.Inference;
      try {
        result = engine.fuzzyInfer(
          SYSTEM,
          Object.entries(values),
          engineName,
          method,
          201,
        );
      } catch (error) {
        output.append(
          errorNote(String((error as Error).message)),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Tidak ada aturan yang menyala pada masukan ini, jadi keluarannya tidak terdefinisi. Ini sengaja dilaporkan sebagai galat, bukan diam-diam diisi nilai tengah.",
                "No rule fires at these inputs, so the output is undefined. This is deliberately reported as an error rather than quietly filled with a midpoint.",
              ),
            ),
          }),
        );
        return;
      }

      const fired = result.rules.filter((r) => r.firing_strength > 0.001).length;

      output.append(
        card(
          pick(T.result),
          readout(
            `${SYSTEM.output.name} · ${engineName}${engineName === "mamdani" ? ` · ${method}` : ""}`,
            fmt(result.crisp, 3),
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                `${fired} dari ${result.rules.length} aturan menyala.`,
                `${fired} of ${result.rules.length} rules fired.`,
              ),
            ),
          }),
        ),
        card(
          pick(bi("Aturan & derajat penyalaan", "Rules & firing strength")),
          table(
            [
              "#",
              pick(bi("Aturan", "Rule")),
              pick(bi("Derajat premis", "Premise degrees")),
              "α",
            ],
            result.rules.map((r) => [
              String(r.index),
              r.text,
              r.degrees.map((d) => fmt(d, 3)).join(" · "),
              r.firing_strength,
            ]),
          ),
        ),
      );

      if (result.xs.length > 0) {
        output.append(
          card(pick(bi("Daerah keluaran", "Output region")), outputCanvas),
        );
        drawOutput(
          outputCanvas,
          result.xs,
          result.ys,
          result.crisp,
          SYSTEM.output.min,
          SYSTEM.output.max,
        );
      }

      // Perbandingan tiga mesin pada masukan yang sama.
      const rows: (string | number)[][] = [];
      for (const name of ["mamdani", "sugeno", "tsukamoto"] as EngineName[]) {
        try {
          const r = engine.fuzzyInfer(SYSTEM, Object.entries(values), name, method, 201);
          rows.push([name, r.crisp]);
        } catch {
          rows.push([name, "—"]);
        }
      }
      output.append(
        card(
          pick(bi("Tiga mesin, satu masukan", "Three engines, one input")),
          table([pick(bi("Mesin", "Engine")), pick(bi("Hasil", "Result"))], rows),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Ketiganya memakai basis aturan yang sama. Angkanya berbeda karena cara mereka mengubah himpunan keluaran menjadi satu bilangan memang berbeda — bukan karena salah satunya keliru.",
                "All three share the same rule base. The numbers differ because they turn the output set into a single value differently — not because one of them is wrong.",
              ),
            ),
          }),
        ),
      );

      for (const v of SYSTEM.inputs) {
        const c = inputCanvases.get(v.name);
        if (c) drawVariable(c, v, values[v.name]);
      }
    }

    function renderControls(): void {
      clear(controls);
      inputCanvases.clear();

      const inputCards = SYSTEM.inputs.map((v) => {
        const canvas = el("canvas", {
          attrs: {
            role: "img",
            "aria-label": pick(
              bi(
                `Kurva keanggotaan variabel ${v.name}.`,
                `Membership curves for ${v.name}.`,
              ),
            ),
          },
        });
        inputCanvases.set(v.name, canvas);
        return card(
          v.name,
          slider({
            label: v.name,
            min: v.min,
            max: v.max,
            step: 0.1,
            value: values[v.name],
            format: (x) => fmt(x, 1),
            onInput: (x) => {
              values[v.name] = x;
              recompute();
            },
          }),
          canvas,
        );
      });

      const engineButtons = buttonRow(
        (["mamdani", "sugeno", "tsukamoto"] as EngineName[]).map((name) => ({
          label: name,
          primary: name === engineName,
          onClick: () => {
            engineName = name;
            renderControls();
            recompute();
          },
        })),
      );

      const methodButtons = buttonRow(
        METHODS.map((m) => ({
          label: m.replaceAll("_", " "),
          primary: m === method,
          onClick: () => {
            method = m;
            renderControls();
            recompute();
          },
        })),
      );

      controls.append(
        ...inputCards,
        card(pick(bi("Mesin inferensi", "Inference engine")), engineButtons),
        card(
          pick(bi("Defuzzifikasi (Mamdani)", "Defuzzification (Mamdani)")),
          methodButtons,
        ),
      );
    }

    root.append(el("div", { class: "grid-2", children: [controls, output] }));
    renderControls();
    recompute();

    const observer = new MutationObserver(() => recompute());
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });

    return () => {
      observer.disconnect();
      clear(root);
    };
}
