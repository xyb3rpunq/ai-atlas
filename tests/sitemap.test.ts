/**
 * Uji peta situs.
 *
 * Peta situs yang tertinggal tidak menggagalkan apa pun. Berkasnya tetap sah,
 * mesin pencari tetap menerimanya, dan halaman yang tidak disebutkan sekadar
 * tidak pernah disebutkan — sampai ada yang bertanya kenapa sembilan dari dua
 * belas laboratorium tidak muncul di hasil pencarian.
 *
 * Persis itu yang terjadi: peta situsnya menyebut tiga laboratorium selama
 * sembilan berikutnya ditambahkan satu per satu. Uji ini menutup jalannya.
 *
 * .Deckyx
 */

import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

import { LABS } from "../web/src/labs/registry.js";

const PETA = readFileSync("web/public/sitemap.xml", "utf8");
const ASAL = "https://xyb3rpunq.github.io/ai-atlas/";

/** Seluruh alamat di dalam peta situs. */
const alamat = [...PETA.matchAll(/<loc>([^<]+)<\/loc>/g)].map((m) => m[1]!);

describe("peta situs", () => {
  it("memuat halaman depan", () => {
    expect(alamat).toContain(ASAL);
  });

  it("memuat setiap laboratorium yang sudah bisa dijalankan", () => {
    for (const lab of LABS) {
      expect(alamat, lab.slug).toContain(`${ASAL}#/${lab.slug}`);
    }
  });

  it("memuat halaman lintas-bahasa", () => {
    // Bukan sesi silabus, jadi ia tidak ikut terbawa oleh uji di atas — dan
    // justru karena itu ia yang paling mudah terlupakan.
    expect(alamat).toContain(`${ASAL}#/enam-bahasa`);
  });

  it("tidak menyebut alamat yang tidak menuju ke mana-mana", () => {
    const slugSah = new Set([...LABS.map((l) => l.slug), "enam-bahasa"]);
    for (const a of alamat) {
      if (a === ASAL) continue;
      const slug = a.slice(`${ASAL}#/`.length);
      expect(slugSah.has(slug), a).toBe(true);
    }
  });

  it("tidak menyebut satu alamat dua kali", () => {
    // `SYLLABUS` menunjuk slug yang sama dengan `LABS`, dan dua sesi fuzzy
    // berbagi satu laboratorium. Pembacaan yang tidak hati-hati menghasilkan
    // rangkap, dan rangkap di peta situs terbaca sebagai isi yang digandakan.
    expect(new Set(alamat).size).toBe(alamat.length);
  });

  it("menyebut kedua bahasa pada halaman depan", () => {
    expect(PETA).toContain('hreflang="id"');
    expect(PETA).toContain('hreflang="en"');
  });
});
