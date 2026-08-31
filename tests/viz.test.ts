/**
 * @vitest-environment happy-dom
 *
 * Uji untuk perangkat visualisasi.
 *
 * Gambar adalah bagian yang paling mudah rusak tanpa ketahuan: sebuah bilah
 * yang panjangnya salah tetap terlihat seperti bilah, dan sebuah sisi yang
 * hilang dari graf tetap menyisakan graf yang tampak masuk akal. Karena itu
 * yang diuji di sini bukan "apakah ada gambarnya", melainkan angka-angka yang
 * menentukan letak dan panjangnya.
 *
 * Yang dijaga paling ketat justru padanan teksnya. Gambar tanpa keterangan
 * hanyalah kotak kosong bagi pengguna pembaca layar, dan tidak ada satu pun
 * uji lain di repositori ini yang akan menangkapnya.
 *
 * .Deckyx
 */

import { beforeEach, describe, expect, it } from "vitest";
import { bi, setLang } from "../web/src/i18n.js";
import {
  canvasSvg,
  cycle,
  dataDetails,
  figure,
  heatmap,
  nodeGraph,
  numberLine,
  pipeline,
  rankedBars,
  svg,
  svgText,
  waterfall,
} from "../web/src/viz.js";

/** Seluruh elemen `<rect>` di dalam sebuah simpul. */
function rects(node: Element): SVGRectElement[] {
  return [...node.querySelectorAll("rect")] as SVGRectElement[];
}

/** Seluruh teks yang muncul di dalam sebuah simpul. */
function texts(node: Element): string[] {
  return [...node.querySelectorAll("text")].map((t) => t.textContent ?? "");
}

beforeEach(() => {
  setLang("id");
});

describe("pembantu SVG", () => {
  it("membuat simpul pada ruang nama SVG", () => {
    const node = svg("rect", { x: 1, y: 2 });
    expect(node.namespaceURI).toBe("http://www.w3.org/2000/svg");
    expect(node.getAttribute("x")).toBe("1");
  });

  it("melewati atribut bernilai kosong alih-alih menulis 'null'", () => {
    // Atribut `stroke-dasharray="null"` bukan galat bagi peramban; ia hanya
    // diabaikan diam-diam, sehingga garis putus-putus berubah menjadi garis
    // penuh tanpa ada yang tahu.
    const node = svg("path", { d: "M0 0", "stroke-dasharray": null, fill: undefined });
    expect(node.hasAttribute("stroke-dasharray")).toBe(false);
    expect(node.hasAttribute("fill")).toBe(false);
  });

  it("menyisipkan teks sebagai teks, bukan markup", () => {
    const node = svgText(0, 0, "<script>x</script>");
    expect(node.textContent).toBe("<script>x</script>");
    expect(node.querySelector("script")).toBeNull();
  });

  it("kanvas memakai viewBox agar ikut mengecil bersama induknya", () => {
    const c = canvasSvg(640, 120);
    expect(c.getAttribute("viewBox")).toBe("0 0 640 120");
    expect(c.getAttribute("width")).toBe("100%");
  });
});

describe("figure", () => {
  it("menyertakan judul, keterangan, dan teks alternatif", () => {
    const f = figure({
      title: bi("Garis keyakinan", "Belief line"),
      summary: bi("Jarum berada di tengah.", "The needle sits in the middle."),
      body: canvasSvg(100, 40),
    });
    expect(f.querySelector(".viz__title")?.textContent).toBe("Garis keyakinan");
    expect(f.querySelector("figcaption")?.textContent).toBe("Jarum berada di tengah.");
    const gambar = f.querySelector("svg");
    expect(gambar?.getAttribute("role")).toBe("img");
    // Teks alternatifnya memuat judul dan keterangan sekaligus, karena pembaca
    // layar hanya membacakan satu di antaranya.
    expect(gambar?.getAttribute("aria-label")).toContain("Garis keyakinan");
    expect(gambar?.getAttribute("aria-label")).toContain("Jarum berada di tengah.");
  });

  it("mengikuti bahasa yang sedang aktif", () => {
    setLang("en");
    const f = figure({
      title: bi("Garis keyakinan", "Belief line"),
      summary: bi("Jarum di tengah.", "Needle in the middle."),
      body: canvasSvg(10, 10),
    });
    expect(f.querySelector(".viz__title")?.textContent).toBe("Belief line");
    expect(f.querySelector("figcaption")?.textContent).toBe("Needle in the middle.");
  });

  it("menuliskan keterangan simbol bila diberikan", () => {
    const f = figure({
      title: bi("A", "A"),
      summary: bi("B", "B"),
      body: canvasSvg(10, 10),
      legend: [{ color: "var(--accent)", label: bi("menyala", "fired") }],
    });
    expect(f.querySelectorAll(".viz__legend-item")).toHaveLength(1);
    expect(f.querySelector(".viz__legend-item")?.textContent).toBe("menyala");
    // Kotak warnanya semata hiasan dan harus disembunyikan dari pembaca layar,
    // yang sudah menerima keterangannya sebagai teks.
    expect(f.querySelector(".viz__swatch")?.getAttribute("aria-hidden")).toBe("true");
  });

  it("tidak menuliskan daftar simbol bila tidak ada", () => {
    const f = figure({ title: bi("A", "A"), summary: bi("B", "B"), body: canvasSvg(10, 10) });
    expect(f.querySelector(".viz__legend")).toBeNull();
  });
});

describe("numberLine", () => {
  const pita = [
    { from: -1, to: 0, label: bi("tidak", "no"), color: "var(--danger)" },
    { from: 0, to: 1, label: bi("ya", "yes"), color: "var(--ok)" },
  ];

  it("menempatkan jarum sebanding dengan nilainya", () => {
    const kiri = numberLine({ min: -1, max: 1, value: -1, bands: pita });
    const tengah = numberLine({ min: -1, max: 1, value: 0, bands: pita });
    const kanan = numberLine({ min: -1, max: 1, value: 1, bands: pita });
    const x = (n: SVGSVGElement): number =>
      Number([...n.querySelectorAll("line")].slice(-1)[0]?.getAttribute("x1") ?? "0");
    expect(x(kiri)).toBeLessThan(x(tengah));
    expect(x(tengah)).toBeLessThan(x(kanan));
  });

  it("menahan nilai di luar rentang di tepinya, bukan di luar gambar", () => {
    // Nilai di luar rentang datang dari data yang salah, bukan dari pengguna.
    // Menggambarnya di luar bingkai membuat gambarnya hilang tanpa penjelasan;
    // menahannya di tepi tetap memberi tahu ke arah mana nilainya melenceng.
    const jauh = numberLine({ min: -1, max: 1, value: 99, bands: pita });
    const tepi = numberLine({ min: -1, max: 1, value: 1, bands: pita });
    const x = (n: SVGSVGElement): string =>
      [...n.querySelectorAll("line")].slice(-1)[0]?.getAttribute("x1") ?? "";
    expect(x(jauh)).toBe(x(tepi));
  });

  it("menuliskan nilainya sebagai angka yang bisa dibaca", () => {
    const n = numberLine({ min: -1, max: 1, value: 0.7321, bands: pita });
    expect(texts(n)).toContain("0.732");
  });

  it("menggambar penanda tambahan bila diberikan", () => {
    const tanpa = numberLine({ min: -1, max: 1, value: 0, bands: pita });
    const dengan = numberLine({
      min: -1,
      max: 1,
      value: 0,
      bands: pita,
      marks: [{ value: 0.5, label: "bukti" }],
    });
    expect(dengan.querySelectorAll("circle").length).toBe(
      tanpa.querySelectorAll("circle").length + 1,
    );
    expect(texts(dengan)).toContain("bukti");
  });

  it("menggambar garis nol hanya bila nol ada di dalam rentangnya", () => {
    const bertanda = numberLine({ min: -1, max: 1, value: 0, bands: pita });
    const positif = numberLine({
      min: 0,
      max: 1,
      value: 0.5,
      bands: [{ from: 0, to: 1, label: bi("a", "a"), color: "var(--ok)" }],
    });
    expect(bertanda.querySelectorAll('line[stroke-dasharray]').length).toBe(1);
    expect(positif.querySelectorAll('line[stroke-dasharray]').length).toBe(0);
  });
});

describe("waterfall", () => {
  it("menggambar satu batang per langkah", () => {
    const w = waterfall(
      [
        { label: "a", value: 0.5 },
        { label: "b", value: 0.75 },
        { label: "c", value: 0.6 },
      ],
      -1,
      1,
    );
    // Satu batang per langkah; garis nol bukan batang.
    expect(rects(w)).toHaveLength(3);
    expect(texts(w)).toContain("a");
    expect(texts(w)).toContain("0.750");
  });

  it("mewarnai langkah yang menurun berbeda dari yang menaik", () => {
    const w = waterfall(
      [
        { label: "naik", value: 0.8 },
        { label: "turun", value: 0.3 },
      ],
      -1,
      1,
    );
    const warna = rects(w).map((r) => r.getAttribute("fill"));
    expect(warna[0]).toBe("var(--accent)");
    expect(warna[1]).toBe("var(--danger)");
  });

  it("memberi batang lebar terlihat walau langkahnya nyaris tidak bergerak", () => {
    // Batang selebar nol piksel terbaca sebagai "tidak ada data", padahal
    // artinya "bukti ini tidak mengubah apa pun" — dua hal yang berbeda.
    const w = waterfall(
      [
        { label: "a", value: 0.5 },
        { label: "b", value: 0.5 },
      ],
      -1,
      1,
    );
    for (const r of rects(w)) {
      expect(Number(r.getAttribute("width"))).toBeGreaterThan(0);
    }
  });
});

describe("nodeGraph", () => {
  const simpul = [
    { id: "a", label: "a", layer: 0 },
    { id: "b", label: "b", layer: 0 },
    { id: "c", label: "c", layer: 1, detail: "0.80", tone: "aktif" as const },
  ];

  it("menyusun simpul menurut lapisannya", () => {
    const g = nodeGraph(simpul, [{ from: "a", to: "c", active: true }]);
    const kotak = rects(g);
    expect(kotak).toHaveLength(3);
    const y = kotak.map((r) => Number(r.getAttribute("y")));
    // Dua simpul lapisan nol sejajar; simpul lapisan satu di bawahnya.
    expect(y[0]).toBe(y[1]);
    expect(y[2]).toBeGreaterThan(y[0]);
  });

  it("membedakan sisi yang menyala dari yang tidak", () => {
    const g = nodeGraph(simpul, [
      { from: "a", to: "c", active: true },
      { from: "b", to: "c", active: false },
    ]);
    const jalur = [...g.querySelectorAll("path")].filter((p) => p.getAttribute("marker-end"));
    expect(jalur).toHaveLength(2);
    expect(jalur[0].getAttribute("stroke")).toBe("var(--accent)");
    expect(jalur[0].hasAttribute("stroke-dasharray")).toBe(false);
    expect(jalur[1].hasAttribute("stroke-dasharray")).toBe(true);
  });

  it("menandai sisi ingkar dengan silang", () => {
    const polos = nodeGraph(simpul, [{ from: "a", to: "c" }]);
    const ingkar = nodeGraph(simpul, [{ from: "a", to: "c", negated: true }]);
    expect(ingkar.querySelectorAll("line").length).toBe(
      polos.querySelectorAll("line").length + 2,
    );
  });

  it("melewati sisi yang menunjuk simpul tak dikenal", () => {
    // Basis pengetahuan yang tidak lengkap tidak boleh membuat gambarnya gagal
    // sama sekali; yang benar adalah menggambar bagian yang memang diketahui.
    const g = nodeGraph(simpul, [{ from: "a", to: "hantu" }]);
    expect([...g.querySelectorAll("path")].filter((p) => p.getAttribute("marker-end"))).toHaveLength(0);
    expect(rects(g)).toHaveLength(3);
  });

  it("menyertakan keterangan simpul sebagai judul yang bisa dibacakan", () => {
    const g = nodeGraph(simpul, []);
    const judul = [...g.querySelectorAll("title")].map((t) => t.textContent);
    expect(judul).toContain("c — 0.80");
  });
});

describe("heatmap", () => {
  it("menggambar satu sel per pasangan baris dan kolom", () => {
    const h = heatmap({
      rows: ["D1", "D2"],
      cols: ["a", "b", "c"],
      values: [
        [1, 0, 0.5],
        [0, 2, 0.25],
      ],
    });
    expect(rects(h)).toHaveLength(6);
  });

  it("menormalkan kepekatan terhadap nilai terbesar yang ada", () => {
    const h = heatmap({ rows: ["r"], cols: ["a", "b"], values: [[1, 0.5]] });
    const buram = rects(h).map((r) => Number(r.getAttribute("opacity")));
    expect(buram[0]).toBeGreaterThan(buram[1]);
  });

  it("tidak membagi dengan nol saat seluruh nilainya nol", () => {
    const h = heatmap({ rows: ["r"], cols: ["a"], values: [[0]] });
    for (const r of rects(h)) {
      expect(Number.isFinite(Number(r.getAttribute("opacity")))).toBe(true);
    }
  });

  it("mewarnai nilai negatif berbeda dari yang positif", () => {
    const h = heatmap({ rows: ["r"], cols: ["a", "b"], values: [[-1, 1]] });
    const warna = rects(h).map((r) => r.getAttribute("fill"));
    expect(warna[0]).toBe("var(--danger)");
    expect(warna[1]).toBe("var(--accent)");
  });

  it("menyediakan judul sel untuk dibacakan dan disorot tetikus", () => {
    const h = heatmap({ rows: ["D1"], cols: ["kucing"], values: [[0.5]] });
    expect([...h.querySelectorAll("title")].map((t) => t.textContent)).toContain(
      "D1 × kucing = 0.500",
    );
  });

  it("menahan sel dari mengecil tanpa batas pada matriks lebar", () => {
    const kolom = Array.from({ length: 40 }, (_, i) => `k${i}`);
    const h = heatmap({ rows: ["r"], cols: kolom, values: [kolom.map(() => 1)] });
    const lebar = Number(rects(h)[0].getAttribute("width"));
    expect(lebar).toBeGreaterThan(20);
  });
});

describe("pipeline", () => {
  it("menggambar satu kotak per tahap", () => {
    const p = pipeline([
      { label: bi("Masukan", "Input"), value: "menyapu" },
      { label: bi("Kupas", "Strip"), value: "sapu", note: "meny-" },
    ]);
    expect(rects(p)).toHaveLength(2);
    expect(texts(p)).toContain("menyapu");
    expect(texts(p)).toContain("meny-");
  });

  it("memudarkan tahap yang tidak terjadi", () => {
    const p = pipeline([
      { label: bi("A", "A"), value: "x" },
      { label: bi("B", "B"), value: "y", skipped: true },
    ]);
    const kotak = rects(p);
    expect(kotak[0].hasAttribute("stroke-dasharray")).toBe(false);
    expect(kotak[1].getAttribute("stroke-dasharray")).toBe("4 4");
  });

  it("menggambar panah di antara tahap, bukan sesudah tahap terakhir", () => {
    const dua = pipeline([
      { label: bi("A", "A"), value: "x" },
      { label: bi("B", "B"), value: "y" },
    ]);
    const satu = pipeline([{ label: bi("A", "A"), value: "x" }]);
    expect(dua.querySelectorAll("path")).toHaveLength(1);
    expect(satu.querySelectorAll("path")).toHaveLength(0);
  });
});

describe("rankedBars", () => {
  it("memanjangkan bilah sebanding dengan nilainya", () => {
    const b = rankedBars([
      { label: "a", value: 1 },
      { label: "b", value: 0.5 },
    ]);
    const lebar = rects(b).map((r) => Number(r.getAttribute("width")));
    expect(lebar[0]).toBeGreaterThan(lebar[1] * 1.9);
  });

  it("menyorot baris yang ditandai", () => {
    const b = rankedBars([
      { label: "a", value: 1, highlight: true },
      { label: "b", value: 0.5 },
    ]);
    const warna = rects(b).map((r) => r.getAttribute("fill"));
    expect(warna[0]).toBe("var(--accent)");
    expect(warna[1]).toBe("var(--text-faint)");
  });

  it("tidak membagi dengan nol saat seluruh nilainya nol", () => {
    const b = rankedBars([
      { label: "a", value: 0 },
      { label: "b", value: 0 },
    ]);
    for (const r of rects(b)) {
      expect(Number.isFinite(Number(r.getAttribute("width")))).toBe(true);
    }
  });

  it("memakai pemformat yang diberikan", () => {
    const b = rankedBars([{ label: "a", value: 0.5 }], (v) => `${v} bit`);
    expect(texts(b)).toContain("0.5 bit");
  });

  it("menuliskan keterangan tambahan bila ada", () => {
    const b = rankedBars([{ label: "a", value: 1, detail: "30%" }]);
    expect(texts(b)).toContain("30%");
  });
});

describe("cycle", () => {
  it("menyusun tahap melingkar dan menutup lingkarannya", () => {
    const c = cycle([
      { label: bi("Indera", "Percept"), value: "kotor" },
      { label: bi("Tindakan", "Action"), value: "sedot" },
    ]);
    expect(rects(c)).toHaveLength(2);
    // Busur penghubungnya sebanyak tahapnya, bukan sebanyak tahap dikurangi
    // satu: siklus tertutup, sehingga tahap terakhir kembali ke yang pertama.
    expect(c.querySelectorAll("path[d^='M']").length).toBeGreaterThanOrEqual(2);
    expect(texts(c)).toContain("kotor");
  });

  it("menyorot tahap yang sedang berjalan", () => {
    const c = cycle(
      [
        { label: bi("A", "A"), value: "1" },
        { label: bi("B", "B"), value: "2" },
      ],
      1,
    );
    const kotak = rects(c);
    expect(kotak[0].getAttribute("stroke")).toBe("var(--border-strong)");
    expect(kotak[1].getAttribute("stroke")).toBe("var(--accent)");
  });

  it("tidak menyorot apa pun bila tidak ada tahap aktif", () => {
    const c = cycle([{ label: bi("A", "A"), value: "1" }]);
    expect(rects(c)[0].getAttribute("stroke")).toBe("var(--border-strong)");
  });

  it("tidak membagi dengan nol saat tahapnya kosong", () => {
    const c = cycle([]);
    expect(rects(c)).toHaveLength(0);
  });
});

describe("dataDetails", () => {
  it("menyembunyikan tabel di balik ringkasan yang tertutup", () => {
    const d = dataDetails(["a", "b"], [["x", 1]]);
    expect(d.tagName.toLowerCase()).toBe("details");
    expect(d.hasAttribute("open")).toBe(false);
    expect(d.querySelector("summary")?.textContent).toBe("Lihat angkanya");
  });

  it("memformat angka dan membiarkan teks apa adanya", () => {
    const d = dataDetails(["a", "b"], [["kucing", 1 / 3]]);
    const sel = [...d.querySelectorAll("td")].map((t) => t.textContent);
    expect(sel).toEqual(["kucing", "0.3333"]);
  });

  it("menandai sel angka agar bisa diratakan kanan", () => {
    const d = dataDetails(["a"], [[1]]);
    expect(d.querySelector("td")?.className).toBe("num");
  });
});
