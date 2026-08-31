/**
 * Perangkat visualisasi bersama.
 *
 * # Kenapa SVG, bukan canvas
 *
 * Laboratorium yang menggambar ribuan piksel tiap bingkai — penelusuran peta,
 * pelatihan jaringan syaraf, simulasi robot — tetap memakai canvas. Sisanya
 * memakai SVG, dan itu bukan selera:
 *
 * - Canvas tidak terlihat oleh pembaca layar. Sebuah diagram alur inferensi
 *   yang digambar di canvas sama saja dengan gambar kosong bagi pengguna yang
 *   memakai pembaca layar. Simpul SVG bisa diberi `<title>` dan `<desc>`, dan
 *   teksnya bisa dipilih serta dicari.
 * - Canvas harus digambar ulang setiap kali tema berganti dan setiap kali
 *   ukurannya berubah. SVG mewarisi warna lewat `currentColor` dan variabel
 *   CSS, jadi pergantian terang-gelap tidak memerlukan satu baris kode pun.
 * - Canvas harus mengurus `devicePixelRatio` sendiri, dan salah sedikit
 *   hasilnya buram di layar retina.
 *
 * # Setiap gambar wajib punya padanan teks
 *
 * Fungsi {@link figure} menuntut `summary`. Gambar tanpa keterangan hanya
 * berguna bagi yang sudah paham isinya, dan pengguna yang paling butuh gambar
 * justru yang belum paham. Keterangan itu sekaligus menjadi teks alternatif
 * bagi pembaca layar dan tetap terbaca saat gambarnya gagal dimuat.
 *
 * .Deckyx
 */

import { bi, lang, pick, type Bilingual } from "./i18n.js";
import { clamp, el, fmt } from "./ui.js";

const NS = "http://www.w3.org/2000/svg";

/** Atribut untuk {@link svg}. */
type SvgAttrs = Record<string, string | number | null | undefined>;

/** Membuat simpul SVG beserta atribut dan anaknya. */
export function svg<K extends keyof SVGElementTagNameMap>(
  tag: K,
  attrs: SvgAttrs = {},
  children: (Node | string)[] = [],
): SVGElementTagNameMap[K] {
  const node = document.createElementNS(NS, tag);
  for (const [k, v] of Object.entries(attrs)) {
    if (v === null || v === undefined) continue;
    node.setAttribute(k, String(v));
  }
  for (const child of children) node.append(child);
  return node;
}

/** Teks SVG dengan gaya bawaan yang sudah selaras dengan tema. */
export function svgText(
  x: number,
  y: number,
  text: string,
  attrs: SvgAttrs = {},
): SVGTextElement {
  const node = svg("text", {
    x,
    y,
    "font-size": 11,
    "font-family": "var(--font-sans)",
    fill: "var(--text-muted)",
    ...attrs,
  });
  node.textContent = text;
  return node;
}

/** Argumen untuk {@link figure}. */
export interface FigureOptions {
  /** Judul singkat di atas gambar. */
  title: Bilingual;
  /**
   * Keterangan yang menjelaskan isi gambar dengan kalimat biasa.
   *
   * Bukan pengulangan judul: yang dibutuhkan adalah apa yang sedang dilihat
   * dan apa artinya, sebab pembaca yang paling butuh gambar adalah yang belum
   * memahami topiknya.
   */
  summary: Bilingual;
  /** Isi gambar. */
  body: SVGSVGElement;
  /** Keterangan simbol, kalau gambarnya memakai lebih dari satu warna. */
  legend?: { color: string; label: Bilingual }[];
}

/**
 * Membungkus sebuah gambar dengan judul, keterangan, dan padanan teksnya.
 *
 * SVG-nya diberi `role="img"` dan `aria-label`, sehingga pembaca layar
 * membacakan keterangannya alih-alih menelusuri ratusan simpul gambar satu per
 * satu — yang hasilnya hanya deretan angka tanpa makna.
 */
export function figure(options: FigureOptions): HTMLElement {
  const ringkas = pick(options.summary);
  options.body.setAttribute("role", "img");
  options.body.setAttribute("aria-label", `${pick(options.title)}. ${ringkas}`);

  const legend = options.legend
    ? el("ul", {
        class: "viz__legend",
        children: options.legend.map((item) =>
          el("li", {
            class: "viz__legend-item",
            children: [
              el("span", {
                class: "viz__swatch",
                attrs: { style: `background:${item.color}`, "aria-hidden": "true" },
              }),
              el("span", { text: pick(item.label) }),
            ],
          }),
        ),
      })
    : null;

  return el("figure", {
    class: "viz",
    children: [
      el("h3", { class: "viz__title", text: pick(options.title) }),
      options.body,
      legend,
      el("figcaption", { class: "viz__caption", text: ringkas }),
    ],
  });
}

/**
 * Membuat kanvas SVG yang menyesuaikan lebar induknya.
 *
 * `viewBox` yang dipasangkan dengan lebar 100% membuat gambarnya ikut mengecil
 * di layar sempit tanpa satu baris pun kode ukur-mengukur.
 */
export function canvasSvg(width: number, height: number): SVGSVGElement {
  return svg("svg", {
    viewBox: `0 0 ${width} ${height}`,
    width: "100%",
    height: "auto",
    preserveAspectRatio: "xMidYMid meet",
    class: "viz__svg",
  });
}

// ---------------------------------------------------------------------------
// Garis bilangan bertingkat
// ---------------------------------------------------------------------------

/** Satu pita bernama pada {@link numberLine}. */
export interface Band {
  from: number;
  to: number;
  label: Bilingual;
  color: string;
}

/** Argumen untuk {@link numberLine}. */
export interface NumberLineOptions {
  min: number;
  max: number;
  value: number;
  bands: Band[];
  /** Penanda tambahan, mis. nilai tiap bukti sebelum digabung. */
  marks?: { value: number; label: string }[];
}

/**
 * Garis bilangan berpita, dengan jarum pada nilai saat ini.
 *
 * Dipakai untuk besaran yang angkanya kurang berarti dibandingkan letaknya:
 * CF sebesar 0,73 tidak berarti apa-apa sampai terlihat bahwa ia jatuh di pita
 * "hampir pasti" dan masih jauh dari "pasti".
 */
export function numberLine(options: NumberLineOptions): SVGSVGElement {
  const W = 640;
  const H = 118;
  const padX = 26;
  const trackY = 52;
  const trackH = 26;
  const root = canvasSvg(W, H);

  const x = (v: number): number =>
    padX + ((clamp(v, options.min, options.max) - options.min) / (options.max - options.min)) *
      (W - padX * 2);

  for (const band of options.bands) {
    const x1 = x(band.from);
    const x2 = x(band.to);
    root.append(
      svg("rect", {
        x: x1,
        y: trackY,
        width: Math.max(0, x2 - x1),
        height: trackH,
        fill: band.color,
        opacity: 0.28,
      }),
    );
    // Label pita hanya dicetak kalau pitanya cukup lebar; teks yang saling
    // menumpuk lebih menyesatkan daripada tidak ada teks sama sekali.
    if (x2 - x1 > 62) {
      root.append(
        svgText((x1 + x2) / 2, trackY + trackH + 15, pick(band.label), {
          "text-anchor": "middle",
          "font-size": 10,
        }),
      );
    }
  }

  root.append(
    svg("rect", {
      x: padX,
      y: trackY,
      width: W - padX * 2,
      height: trackH,
      fill: "none",
      stroke: "var(--border-strong)",
      rx: 4,
    }),
  );

  // Nol diberi garis sendiri: pada besaran bertanda, sisi mana sebuah nilai
  // berada jauh lebih penting daripada besarnya.
  if (options.min < 0 && options.max > 0) {
    root.append(
      svg("line", {
        x1: x(0),
        y1: trackY - 6,
        x2: x(0),
        y2: trackY + trackH + 6,
        stroke: "var(--text-faint)",
        "stroke-width": 1,
        "stroke-dasharray": "3 3",
      }),
    );
  }

  for (const mark of options.marks ?? []) {
    root.append(
      svg("circle", {
        cx: x(mark.value),
        cy: trackY + trackH / 2,
        r: 4,
        fill: "var(--surface)",
        stroke: "var(--text-muted)",
        "stroke-width": 1.5,
      }),
      svgText(x(mark.value), trackY - 10, mark.label, {
        "text-anchor": "middle",
        "font-size": 9,
        fill: "var(--text-faint)",
      }),
    );
  }

  const px = x(options.value);
  root.append(
    svg("path", {
      d: `M ${px} ${trackY - 4} l -6 -9 h 12 Z`,
      fill: "var(--accent)",
    }),
    svg("line", {
      x1: px,
      y1: trackY - 4,
      x2: px,
      y2: trackY + trackH + 4,
      stroke: "var(--accent)",
      "stroke-width": 2.5,
    }),
    svgText(px, 22, fmt(options.value, 3), {
      "text-anchor": "middle",
      "font-size": 15,
      "font-family": "var(--font-mono)",
      fill: "var(--text)",
      "font-weight": "600",
    }),
  );

  root.append(
    svgText(padX, trackY + trackH + 30, fmt(options.min, 1), { "font-size": 10 }),
    svgText(W - padX, trackY + trackH + 30, fmt(options.max, 1), {
      "text-anchor": "end",
      "font-size": 10,
    }),
  );

  return root;
}

// ---------------------------------------------------------------------------
// Air terjun
// ---------------------------------------------------------------------------

/** Satu langkah pada {@link waterfall}. */
export interface WaterfallStep {
  label: string;
  /** Nilai kumulatif setelah langkah ini. */
  value: number;
}

/**
 * Diagram air terjun: bagaimana sebuah nilai berpindah langkah demi langkah.
 *
 * Bentuk ini menjawab pertanyaan yang tidak bisa dijawab angka akhir sendirian
 * — bukti mana yang paling menggeser kesimpulan, dan apakah ada bukti yang
 * justru menariknya mundur.
 */
export function waterfall(steps: WaterfallStep[], min: number, max: number): SVGSVGElement {
  const W = 640;
  const rowH = 34;
  const padL = 132;
  const padT = 26;
  const H = padT + steps.length * rowH + 16;
  const root = canvasSvg(W, H);

  const x = (v: number): number =>
    padL + ((clamp(v, min, max) - min) / (max - min)) * (W - padL - 56);

  if (min < 0 && max > 0) {
    root.append(
      svg("line", {
        x1: x(0),
        y1: padT - 12,
        x2: x(0),
        y2: H - 10,
        stroke: "var(--text-faint)",
        "stroke-dasharray": "3 3",
      }),
      svgText(x(0), padT - 16, "0", { "text-anchor": "middle", "font-size": 9 }),
    );
  }

  let prev = 0;
  steps.forEach((step, i) => {
    const y = padT + i * rowH;
    const from = i === 0 ? Math.min(0, step.value) : prev;
    const x1 = x(Math.min(from, step.value));
    const x2 = x(Math.max(from, step.value));
    const naik = step.value >= prev;

    root.append(
      svgText(padL - 10, y + 15, step.label, {
        "text-anchor": "end",
        fill: "var(--text)",
      }),
      svg("rect", {
        x: x1,
        y: y + 4,
        width: Math.max(2, x2 - x1),
        height: 15,
        rx: 3,
        fill: naik ? "var(--accent)" : "var(--danger)",
        opacity: 0.75,
      }),
      svgText(Math.min(W - 6, x2 + 8), y + 16, fmt(step.value, 3), {
        "font-size": 10,
        "font-family": "var(--font-mono)",
        fill: "var(--text)",
      }),
    );

    // Garis penghubung ke langkah berikutnya, supaya terlihat bahwa tiap
    // langkah berangkat dari hasil langkah sebelumnya, bukan dari nol.
    if (i < steps.length - 1) {
      root.append(
        svg("line", {
          x1: x(step.value),
          y1: y + 19,
          x2: x(step.value),
          y2: y + rowH + 4,
          stroke: "var(--border-strong)",
          "stroke-width": 1,
        }),
      );
    }
    prev = step.value;
  });

  return root;
}

// ---------------------------------------------------------------------------
// Graf simpul dan sisi
// ---------------------------------------------------------------------------

/** Satu simpul pada {@link nodeGraph}. */
export interface GraphNode {
  id: string;
  label: string;
  /** Baris tempat simpul diletakkan; 0 adalah baris paling atas. */
  layer: number;
  /** Keterangan kecil di bawah label, mis. nilai keyakinannya. */
  detail?: string;
  tone?: "netral" | "aktif" | "mati" | "tujuan";
}

/** Satu sisi pada {@link nodeGraph}. */
export interface GraphEdge {
  from: string;
  to: string;
  label?: string;
  /** Sisi yang tidak aktif digambar putus-putus dan pudar. */
  active?: boolean;
  /** Sisi ingkar diberi tanda silang. */
  negated?: boolean;
}

const WARNA_SIMPUL: Record<string, { isi: string; garis: string }> = {
  netral: { isi: "var(--surface-2)", garis: "var(--border-strong)" },
  aktif: { isi: "var(--accent-glow)", garis: "var(--accent)" },
  mati: { isi: "transparent", garis: "var(--border)" },
  tujuan: { isi: "var(--accent-glow)", garis: "var(--accent)" },
};

/**
 * Graf berlapis: simpul disusun per baris, sisi digambar dari atas ke bawah.
 *
 * Tata letaknya sengaja ditentukan pemanggil lewat `layer`, bukan dihitung
 * mesin. Tata letak otomatis pada graf sekecil ini menghasilkan gambar yang
 * berpindah-pindah setiap kali datanya berubah sedikit, dan gambar yang tidak
 * bisa diingat letaknya lebih sulit dibaca daripada gambar yang sedikit tidak
 * rapi.
 */
export function nodeGraph(nodes: GraphNode[], edges: GraphEdge[]): SVGSVGElement {
  const W = 640;
  const nodeW = 116;
  const nodeH = 40;
  const gapX = 16;
  const gapY = 64;

  const perLayer = new Map<number, GraphNode[]>();
  for (const n of nodes) {
    const daftar = perLayer.get(n.layer) ?? [];
    daftar.push(n);
    perLayer.set(n.layer, daftar);
  }
  const layers = [...perLayer.keys()].sort((a, b) => a - b);
  const H = layers.length * (nodeH + gapY) + 12;
  const root = canvasSvg(W, H);

  const pos = new Map<string, { x: number; y: number }>();
  layers.forEach((layer, li) => {
    const daftar = perLayer.get(layer) ?? [];
    const lebar = daftar.length * nodeW + (daftar.length - 1) * gapX;
    const mulai = (W - lebar) / 2;
    daftar.forEach((n, i) => {
      pos.set(n.id, {
        x: mulai + i * (nodeW + gapX) + nodeW / 2,
        y: li * (nodeH + gapY) + nodeH / 2 + 6,
      });
    });
  });

  const panah = svg("marker", {
    id: "viz-panah",
    viewBox: "0 0 8 8",
    refX: 7,
    refY: 4,
    markerWidth: 6,
    markerHeight: 6,
    orient: "auto-start-reverse",
  });
  panah.append(svg("path", { d: "M 0 0 L 8 4 L 0 8 z", fill: "var(--border-strong)" }));
  root.append(svg("defs", {}, [panah]));

  for (const e of edges) {
    const a = pos.get(e.from);
    const b = pos.get(e.to);
    if (!a || !b) continue;
    const y1 = a.y + nodeH / 2;
    const y2 = b.y - nodeH / 2;
    const tengah = (y1 + y2) / 2;
    root.append(
      svg("path", {
        d: `M ${a.x} ${y1} C ${a.x} ${tengah}, ${b.x} ${tengah}, ${b.x} ${y2}`,
        fill: "none",
        stroke: e.active ? "var(--accent)" : "var(--border-strong)",
        "stroke-width": e.active ? 2 : 1,
        "stroke-dasharray": e.active ? null : "4 4",
        opacity: e.active ? 0.9 : 0.45,
        "marker-end": "url(#viz-panah)",
      }),
    );
    if (e.negated) {
      root.append(
        svg("line", {
          x1: (a.x + b.x) / 2 - 5,
          y1: tengah - 5,
          x2: (a.x + b.x) / 2 + 5,
          y2: tengah + 5,
          stroke: "var(--danger)",
          "stroke-width": 2,
        }),
        svg("line", {
          x1: (a.x + b.x) / 2 - 5,
          y1: tengah + 5,
          x2: (a.x + b.x) / 2 + 5,
          y2: tengah - 5,
          stroke: "var(--danger)",
          "stroke-width": 2,
        }),
      );
    }
    if (e.label) {
      root.append(
        svgText((a.x + b.x) / 2, tengah + (e.negated ? -10 : 4), e.label, {
          "text-anchor": "middle",
          "font-size": 9,
          "font-family": "var(--font-mono)",
          fill: e.active ? "var(--accent)" : "var(--text-faint)",
        }),
      );
    }
  }

  for (const n of nodes) {
    const p = pos.get(n.id);
    if (!p) continue;
    const warna = WARNA_SIMPUL[n.tone ?? "netral"] ?? WARNA_SIMPUL.netral;
    const g = svg("g", {});
    const judul = svg("title");
    judul.textContent = n.detail ? `${n.label} — ${n.detail}` : n.label;
    g.append(
      judul,
      svg("rect", {
        x: p.x - nodeW / 2,
        y: p.y - nodeH / 2,
        width: nodeW,
        height: nodeH,
        rx: 8,
        fill: warna.isi,
        stroke: warna.garis,
        "stroke-width": n.tone === "aktif" || n.tone === "tujuan" ? 2 : 1,
      }),
      svgText(p.x, p.y + (n.detail ? -1 : 4), n.label, {
        "text-anchor": "middle",
        "font-size": 11,
        fill: "var(--text)",
        "font-weight": n.tone === "tujuan" ? "600" : "400",
      }),
    );
    if (n.detail) {
      g.append(
        svgText(p.x, p.y + 12, n.detail, {
          "text-anchor": "middle",
          "font-size": 9,
          "font-family": "var(--font-mono)",
          fill: "var(--text-muted)",
        }),
      );
    }
    root.append(g);
  }

  return root;
}

// ---------------------------------------------------------------------------
// Peta panas
// ---------------------------------------------------------------------------

/** Argumen untuk {@link heatmap}. */
export interface HeatmapOptions {
  rows: string[];
  cols: string[];
  /** Nilai `values[baris][kolom]`. */
  values: number[][];
  /** Pemformat nilai pada tooltip dan sel. */
  format?: (v: number) => string;
  /** Menuliskan angkanya di dalam sel; matikan untuk matriks besar. */
  showValues?: boolean;
}

/**
 * Peta panas matriks.
 *
 * Skalanya dinormalkan terhadap nilai terbesar yang benar-benar ada, bukan
 * terhadap angka tetap. Skala tetap membuat matriks yang seluruh nilainya
 * kecil tampak kosong, padahal justru perbandingan antarnilainyalah yang
 * ingin dilihat.
 */
export function heatmap(options: HeatmapOptions): SVGSVGElement {
  const format = options.format ?? ((v: number) => fmt(v, 3));
  const padL = 104;
  const padT = 58;
  const cell = Math.max(34, Math.min(72, Math.floor(520 / Math.max(1, options.cols.length))));
  const W = padL + options.cols.length * cell + 12;
  const H = padT + options.rows.length * cell + 12;
  const root = canvasSvg(W, H);

  let maks = 0;
  for (const baris of options.values) {
    for (const v of baris) {
      if (Number.isFinite(v)) maks = Math.max(maks, Math.abs(v));
    }
  }
  if (maks === 0) maks = 1;

  options.cols.forEach((c, j) => {
    const x = padL + j * cell + cell / 2;
    root.append(
      svgText(x, padT - 10, c, {
        "text-anchor": "start",
        "font-size": 10,
        transform: `rotate(-42 ${x} ${padT - 10})`,
      }),
    );
  });

  options.rows.forEach((r, i) => {
    root.append(
      svgText(padL - 10, padT + i * cell + cell / 2 + 4, r, {
        "text-anchor": "end",
        "font-size": 10,
        fill: "var(--text)",
      }),
    );
    options.cols.forEach((c, j) => {
      const v = options.values[i]?.[j] ?? 0;
      const kuat = Math.abs(v) / maks;
      const g = svg("g", {});
      const judul = svg("title");
      judul.textContent = `${r} × ${c} = ${format(v)}`;
      g.append(
        judul,
        svg("rect", {
          x: padL + j * cell + 1,
          y: padT + i * cell + 1,
          width: cell - 2,
          height: cell - 2,
          rx: 4,
          fill: v < 0 ? "var(--danger)" : "var(--accent)",
          opacity: 0.08 + kuat * 0.72,
        }),
      );
      if (options.showValues !== false && cell >= 40) {
        g.append(
          svgText(padL + j * cell + cell / 2, padT + i * cell + cell / 2 + 4, format(v), {
            "text-anchor": "middle",
            "font-size": 9,
            "font-family": "var(--font-mono)",
            fill: kuat > 0.55 ? "var(--bg)" : "var(--text)",
          }),
        );
      }
      root.append(g);
    });
  });

  return root;
}

// ---------------------------------------------------------------------------
// Alur bertahap
// ---------------------------------------------------------------------------

/** Satu tahap pada {@link pipeline}. */
export interface Stage {
  label: Bilingual;
  /** Isi tahap; teks pendek yang menunjukkan hasil tahap ini. */
  value: string;
  /** Keterangan kecil, mis. aturan yang dipakai. */
  note?: string;
  /** Tahap yang tidak terjadi digambar pudar. */
  skipped?: boolean;
}

/**
 * Rantai tahap dengan panah di antaranya.
 *
 * Dipakai untuk proses yang wujudnya berupa perubahan bertahap atas satu benda
 * yang sama — sebuah kata yang dikupas imbuhannya, sebuah kalimat yang
 * dipecah lalu disusun ulang. Tabel bisa memuat data yang sama, tetapi tabel
 * tidak menunjukkan bahwa barisnya berurutan dan saling bergantung.
 */
export function pipeline(stages: Stage[]): SVGSVGElement {
  const W = 640;
  const rowH = 58;
  const boxW = 452;
  const padL = 96;
  const H = stages.length * rowH + 12;
  const root = canvasSvg(W, H);

  stages.forEach((s, i) => {
    const y = i * rowH + 8;
    const pudar = s.skipped === true;
    root.append(
      svgText(padL - 12, y + 24, pick(s.label), {
        "text-anchor": "end",
        "font-size": 10,
        opacity: pudar ? 0.45 : 1,
      }),
      svg("rect", {
        x: padL,
        y,
        width: boxW,
        height: 38,
        rx: 8,
        fill: pudar ? "transparent" : "var(--surface-2)",
        stroke: pudar ? "var(--border)" : "var(--border-strong)",
        "stroke-dasharray": pudar ? "4 4" : null,
      }),
      svgText(padL + 14, y + (s.note ? 17 : 23), s.value, {
        "font-size": 12,
        "font-family": "var(--font-mono)",
        fill: "var(--text)",
        opacity: pudar ? 0.5 : 1,
      }),
    );
    if (s.note) {
      root.append(
        svgText(padL + 14, y + 30, s.note, {
          "font-size": 9,
          fill: "var(--text-faint)",
        }),
      );
    }
    if (i < stages.length - 1) {
      root.append(
        svg("path", {
          d: `M ${padL + boxW / 2} ${y + 38} l 0 12 m -5 -5 l 5 5 l 5 -5`,
          stroke: "var(--border-strong)",
          "stroke-width": 1.5,
          fill: "none",
          "stroke-linecap": "round",
        }),
      );
    }
  });

  return root;
}

// ---------------------------------------------------------------------------
// Bilah pembanding
// ---------------------------------------------------------------------------

/** Satu baris pada {@link rankedBars}. */
export interface RankedBar {
  label: string;
  value: number;
  /** Baris yang menang diberi warna aksen. */
  highlight?: boolean;
  detail?: string;
}

/**
 * Bilah horizontal terurut, untuk membandingkan beberapa besaran sejenis.
 *
 * Bilahnya tidak diurutkan ulang di sini. Pada ID3 misalnya, yang ingin
 * dilihat adalah atribut mana yang menang di antara atribut yang urutannya
 * tetap; mengurutkan ulang tiap kali datanya berubah membuat mata kehilangan
 * jejak baris yang sedang diamati.
 */
export function rankedBars(bars: RankedBar[], format = (v: number) => fmt(v, 4)): SVGSVGElement {
  const W = 640;
  const rowH = 30;
  const padL = 128;
  const padR = 92;
  const H = bars.length * rowH + 14;
  const root = canvasSvg(W, H);

  const maks = Math.max(1e-12, ...bars.map((b) => Math.abs(b.value)));

  bars.forEach((b, i) => {
    const y = i * rowH + 8;
    const lebar = (Math.abs(b.value) / maks) * (W - padL - padR);
    root.append(
      svgText(padL - 10, y + 15, b.label, {
        "text-anchor": "end",
        fill: "var(--text)",
        "font-size": 11,
      }),
      svg("rect", {
        x: padL,
        y: y + 3,
        width: Math.max(2, lebar),
        height: 16,
        rx: 3,
        fill: b.highlight ? "var(--accent)" : "var(--text-faint)",
        opacity: b.highlight ? 0.9 : 0.45,
      }),
      svgText(padL + lebar + 8, y + 16, format(b.value), {
        "font-size": 10,
        "font-family": "var(--font-mono)",
        fill: "var(--text)",
      }),
    );
    if (b.detail) {
      root.append(
        svgText(W - 6, y + 16, b.detail, {
          "text-anchor": "end",
          "font-size": 9,
          fill: "var(--text-faint)",
        }),
      );
    }
  });

  return root;
}

// ---------------------------------------------------------------------------
// Lingkaran siklus
// ---------------------------------------------------------------------------

/** Satu tahap pada {@link cycle}. */
export interface CycleStage {
  label: Bilingual;
  value: string;
}

/**
 * Siklus tertutup: tahap-tahap yang berulang tanpa awal dan akhir.
 *
 * Agen cerdas adalah gelang, bukan garis: ia mengindera, memutuskan,
 * bertindak, lalu mengindera akibat tindakannya sendiri. Menggambarnya sebagai
 * daftar bernomor menyembunyikan justru bagian yang paling menentukan
 * perilakunya — bahwa tindakannya kembali menjadi masukannya.
 */
export function cycle(stages: CycleStage[], activeIndex = -1): SVGSVGElement {
  const W = 460;
  const H = 300;
  const cx = W / 2;
  const cy = H / 2;
  const r = 104;
  const root = canvasSvg(W, H);

  const n = Math.max(1, stages.length);
  const sudut = (i: number): number => (i / n) * Math.PI * 2 - Math.PI / 2;

  // Busur penghubung digambar lebih dulu supaya kotaknya menimpa, bukan
  // sebaliknya.
  for (let i = 0; i < n; i += 1) {
    const a1 = sudut(i) + 0.34;
    const a2 = sudut(i + 1) - 0.34;
    const x1 = cx + Math.cos(a1) * r;
    const y1 = cy + Math.sin(a1) * r;
    const x2 = cx + Math.cos(a2) * r;
    const y2 = cy + Math.sin(a2) * r;
    root.append(
      svg("path", {
        d: `M ${x1} ${y1} A ${r} ${r} 0 0 1 ${x2} ${y2}`,
        fill: "none",
        stroke: "var(--border-strong)",
        "stroke-width": 1.5,
        "marker-end": "url(#viz-panah-siklus)",
      }),
    );
  }

  const panah = svg("marker", {
    id: "viz-panah-siklus",
    viewBox: "0 0 8 8",
    refX: 7,
    refY: 4,
    markerWidth: 6,
    markerHeight: 6,
    orient: "auto-start-reverse",
  });
  panah.append(svg("path", { d: "M 0 0 L 8 4 L 0 8 z", fill: "var(--border-strong)" }));
  root.append(svg("defs", {}, [panah]));

  stages.forEach((s, i) => {
    const a = sudut(i);
    const x = cx + Math.cos(a) * r;
    const y = cy + Math.sin(a) * r;
    const aktif = i === activeIndex;
    root.append(
      svg("rect", {
        x: x - 62,
        y: y - 22,
        width: 124,
        height: 44,
        rx: 9,
        fill: aktif ? "var(--accent-glow)" : "var(--surface-2)",
        stroke: aktif ? "var(--accent)" : "var(--border-strong)",
        "stroke-width": aktif ? 2 : 1,
      }),
      svgText(x, y - 3, pick(s.label), {
        "text-anchor": "middle",
        "font-size": 10,
        fill: "var(--text-muted)",
      }),
      svgText(x, y + 12, s.value, {
        "text-anchor": "middle",
        "font-size": 11,
        "font-family": "var(--font-mono)",
        fill: "var(--text)",
      }),
    );
  });

  return root;
}

// ---------------------------------------------------------------------------
// Padanan teks
// ---------------------------------------------------------------------------

/**
 * Tabel ringkas yang memuat data yang sama dengan gambarnya.
 *
 * Ditempatkan di dalam `<details>` yang tertutup: tidak mengganggu yang sudah
 * paham gambarnya, tetapi tetap ada bagi yang perlu angkanya persis, yang
 * ingin menyalinnya, atau yang memakai pembaca layar.
 */
export function dataDetails(
  head: string[],
  rows: (string | number)[][],
  label: Bilingual = bi("Lihat angkanya", "Show the numbers"),
): HTMLElement {
  const isiBaris = rows.map((r) =>
    el("tr", {
      children: r.map((c) =>
        el("td", {
          class: typeof c === "number" ? "num" : "",
          text: typeof c === "number" ? fmt(c, 4) : c,
        }),
      ),
    }),
  );
  return el("details", {
    class: "viz__data",
    children: [
      el("summary", { text: pick(label) }),
      el("div", {
        class: "scroll-x",
        children: [
          el("table", {
            children: [
              el("thead", {
                children: [el("tr", { children: head.map((h) => el("th", { text: h })) })],
              }),
              el("tbody", { children: isiBaris }),
            ],
          }),
        ],
      }),
    ],
  });
}

/** Bahasa yang sedang dipakai, dipakai uji untuk memastikan gambar dwibahasa. */
export function vizLang(): string {
  return lang();
}
