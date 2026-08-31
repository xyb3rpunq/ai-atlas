/**
 * Uji untuk pembantu sisi TypeScript.
 *
 * Fungsi-fungsi di sini kecil, tetapi persis jenis yang gagal secara diam-diam:
 * pemformat angka yang salah membulat tidak membuat apa pun rusak, ia hanya
 * menampilkan angka keliru dengan penuh percaya diri.
 *
 * .Deckyx
 */

import { describe, expect, it } from "vitest";
import { clamp, fmt, pct } from "../web/src/ui.js";

describe("fmt", () => {
  it("membulatkan ke jumlah desimal yang diminta", () => {
    expect(fmt(0.42, 4)).toBe("0.4200");
    expect(fmt(1 / 3, 4)).toBe("0.3333");
    expect(fmt(2 / 3, 4)).toBe("0.6667");
    expect(fmt(3, 0)).toBe("3");
  });

  it("membuang derau biner dari perkalian pecahan", () => {
    // 0.9*0.2 + 0.3*0.8 = 0.42000000000000004 pada IEEE-754.
    expect(fmt(0.9 * 0.2 + 0.3 * 0.8, 4)).toBe("0.4200");
  });

  it("tidak pernah menampilkan nol negatif", () => {
    expect(fmt(-0)).toBe("0.0000");
    expect(fmt(-0.00001, 3)).toBe("0.000");
  });

  it("menangani bilangan negatif", () => {
    expect(fmt(-1.5, 2)).toBe("-1.50");
    expect(fmt(-0.005, 2)).toBe("-0.01");
  });

  it("menandai nilai yang tidak berhingga alih-alih menampilkan NaN", () => {
    expect(fmt(Number.POSITIVE_INFINITY)).toBe("∞");
    expect(fmt(Number.NEGATIVE_INFINITY)).toBe("-∞");
    expect(fmt(Number.NaN)).toBe("—");
  });

  it("memakai empat desimal sebagai bawaan", () => {
    expect(fmt(1)).toBe("1.0000");
  });
});

describe("pct", () => {
  it("mengubah pecahan menjadi persen", () => {
    expect(pct(0.4285714285714286, 2)).toBe("42.86%");
    expect(pct(1, 0)).toBe("100%");
    expect(pct(0, 1)).toBe("0.0%");
  });

  it("menangani nilai di luar nol sampai satu apa adanya", () => {
    expect(pct(1.5, 0)).toBe("150%");
    expect(pct(-0.25, 0)).toBe("-25%");
  });

  it("menandai nilai tak berhingga", () => {
    expect(pct(Number.NaN)).toBe("—");
    expect(pct(Number.POSITIVE_INFINITY)).toBe("—");
  });
});

describe("clamp", () => {
  it("mengembalikan nilai yang sudah di dalam rentang", () => {
    expect(clamp(0.5, 0, 1)).toBe(0.5);
  });

  it("memotong di kedua ujung", () => {
    expect(clamp(-3, 0, 1)).toBe(0);
    expect(clamp(9, 0, 1)).toBe(1);
  });

  it("menghormati batas yang sama persis", () => {
    expect(clamp(0, 0, 1)).toBe(0);
    expect(clamp(1, 0, 1)).toBe(1);
  });

  it("bekerja pada rentang negatif", () => {
    expect(clamp(-5, -3, -1)).toBe(-3);
    expect(clamp(0, -3, -1)).toBe(-1);
  });
});

describe("sel angka pada tabel", () => {
  it("menulis bilangan bulat tanpa ekor desimal", () => {
    // Nomor urut yang muncul sebagai "1.0000" terbaca seperti cacat.
    expect(fmt(1, 0)).toBe("1");
  });

  it("tetap memberi presisi pada pecahan", () => {
    expect(fmt(0.6, 4)).toBe("0.6000");
  });
});
