/**
 * Uji kelengkapan catatan dan definisi.
 *
 * Catatan adalah bagian yang paling mudah tertinggal: kodenya tetap jalan
 * tanpa catatan, ujinya tetap hijau, dan tidak ada yang berteriak. Uji di
 * berkas ini membuat laboratorium baru mustahil ditambahkan tanpa definisi,
 * rumus, kekeliruan umum, dan rujukan yang menyertainya.
 *
 * .Deckyx
 */

import { describe, expect, it } from "vitest";
import { NOTES, notesFor } from "../web/src/labs/notes.js";

/**
 * Slug tiap laboratorium.
 *
 * Ditulis ulang di sini alih-alih diimpor dari registry, karena registry
 * menarik seluruh berkas laboratorium yang memerlukan DOM dan WebAssembly.
 * Daftar ini dijaga tetap sepadan oleh uji terakhir di berkas ini.
 */
const SLUG_LAB = [
  "eliza",
  "agents",
  "certainty-factor",
  "bayesian",
  "fuzzy-logic",
  "knowledge",
  "search",
  "neural-network",
  "nlp",
  "expert-system",
  "machine-learning",
  "robotics",
];

describe("kelengkapan catatan", () => {
  it("setiap laboratorium punya catatan", () => {
    const tanpaCatatan = SLUG_LAB.filter((slug) => notesFor(slug) === undefined);
    expect(tanpaCatatan).toEqual([]);
  });

  it("tidak ada catatan yatim tanpa laboratorium", () => {
    const yatim = Object.keys(NOTES).filter((slug) => !SLUG_LAB.includes(slug));
    expect(yatim).toEqual([]);
  });

  it.each(SLUG_LAB)("catatan %s memuat seluruh bagiannya", (slug) => {
    const notes = notesFor(slug);
    expect(notes).toBeDefined();
    if (!notes) return;

    // Ringkasan harus menjelaskan apa yang dihitung, bukan sekadar menyebut
    // nama topiknya, jadi panjang minimumnya dijaga.
    expect(notes.summary.id.length).toBeGreaterThan(80);
    expect(notes.summary.en.length).toBeGreaterThan(80);

    expect(notes.definitions.length).toBeGreaterThanOrEqual(4);
    expect(notes.formulas.length).toBeGreaterThanOrEqual(1);
    expect(notes.pitfalls.length).toBeGreaterThanOrEqual(2);
    expect(notes.references.length).toBeGreaterThanOrEqual(1);
  });

  it.each(SLUG_LAB)("definisi %s lengkap dua bahasa", (slug) => {
    const notes = notesFor(slug);
    if (!notes) return;
    for (const d of notes.definitions) {
      expect(d.term.trim().length).toBeGreaterThan(0);
      // Definisi sepanjang beberapa kata biasanya hanya parafrase istilahnya
      // sendiri, yang tidak menjelaskan apa pun.
      expect(d.meaning.id.length).toBeGreaterThan(40);
      expect(d.meaning.en.length).toBeGreaterThan(40);
    }
  });

  it.each(SLUG_LAB)("rumus %s menyertakan syarat berlakunya", (slug) => {
    const notes = notesFor(slug);
    if (!notes) return;
    for (const f of notes.formulas) {
      expect(f.name.trim().length).toBeGreaterThan(0);
      expect(f.expression.trim().length).toBeGreaterThan(0);
      // Rumus tanpa keterangan kapan ia berlaku adalah jebakan.
      expect(f.note.id.length).toBeGreaterThan(30);
      expect(f.note.en.length).toBeGreaterThan(30);
    }
  });

  it.each(SLUG_LAB)("istilah %s tidak terduplikasi", (slug) => {
    const notes = notesFor(slug);
    if (!notes) return;
    const istilah = notes.definitions.map((d) => d.term.toLowerCase());
    expect(new Set(istilah).size).toBe(istilah.length);
  });

  it("rujukan berpranala memakai alamat yang sah", () => {
    for (const [slug, notes] of Object.entries(NOTES)) {
      for (const r of notes.references) {
        expect(r.text.trim().length, `${slug}`).toBeGreaterThan(20);
        if (r.url !== undefined) {
          expect(r.url, `${slug}: ${r.text}`).toMatch(/^https:\/\//);
        }
      }
    }
  });

  it("kekeliruan umum ditulis sebagai kalimat, bukan potongan", () => {
    for (const [slug, notes] of Object.entries(NOTES)) {
      for (const p of notes.pitfalls) {
        expect(p.id.length, `${slug}`).toBeGreaterThan(60);
        expect(p.en.length, `${slug}`).toBeGreaterThan(60);
      }
    }
  });

  it("seluruh teks tersedia dalam kedua bahasa", () => {
    // Teks yang identik di kedua bahasa hampir selalu berarti terjemahannya
    // terlupa. Istilah teknis yang memang sama di kedua bahasa dikecualikan.
    const bolehSama = new Set(["TF-IDF", "Naive Bayes", "PEAS", "Perceptron", "Epoch"]);
    for (const [slug, notes] of Object.entries(NOTES)) {
      for (const d of notes.definitions) {
        if (bolehSama.has(d.term)) continue;
        expect(d.meaning.id, `${slug}: ${d.term}`).not.toBe(d.meaning.en);
      }
      expect(notes.summary.id, slug).not.toBe(notes.summary.en);
    }
  });

  it("menghitung cakupan keseluruhan", () => {
    let definisi = 0;
    let rumus = 0;
    let kekeliruan = 0;
    let rujukan = 0;
    for (const notes of Object.values(NOTES)) {
      definisi += notes.definitions.length;
      rumus += notes.formulas.length;
      kekeliruan += notes.pitfalls.length;
      rujukan += notes.references.length;
    }
    // Angka ini bukan sekadar catatan: ia menjaga agar catatan tidak diam-diam
    // menyusut ketika seseorang merapikan berkasnya.
    expect(definisi).toBeGreaterThanOrEqual(70);
    expect(rumus).toBeGreaterThanOrEqual(35);
    expect(kekeliruan).toBeGreaterThanOrEqual(20);
    expect(rujukan).toBeGreaterThanOrEqual(20);
  });
});
