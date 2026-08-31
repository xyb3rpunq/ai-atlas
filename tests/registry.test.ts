/**
 * @vitest-environment happy-dom
 *
 * Uji untuk katalog laboratorium.
 *
 * Sejak mesinnya dimuat lewat `import()`, katalog di `labs/registry.ts` menjadi
 * satu-satunya penghubung antara alamat yang diketik pengguna dan kode yang
 * dijalankan. Salah ketik pada jalur `import()` tidak akan tertangkap pemeriksa
 * tipe maupun proses build — Vite tetap menghasilkan berkas, dan kegagalannya
 * baru muncul ketika seseorang membuka laboratorium itu di peramban.
 *
 * Karena itu uji di sini benar-benar memanggil tiap `load()`.
 *
 * .Deckyx
 */

import { describe, expect, it } from "vitest";
import { LABS, SYLLABUS, findLab } from "../web/src/labs/registry.js";
import { notesFor } from "../web/src/labs/notes.js";

describe("katalog laboratorium", () => {
  it("memuat dua belas laboratorium", () => {
    expect(LABS).toHaveLength(12);
  });

  it("slug-nya unik dan aman dipakai di alamat", () => {
    const slugs = LABS.map((l) => l.slug);
    expect(new Set(slugs).size).toBe(slugs.length);
    for (const s of slugs) {
      // Slug dipakai apa adanya sebagai `#/<slug>`; karakter di luar himpunan
      // ini akan tersandi ulang dan alamatnya tidak lagi cocok dengan dirinya.
      expect(s, s).toMatch(/^[a-z0-9-]+$/);
    }
  });

  it("nomor sesinya berada di dalam silabus empat belas pertemuan", () => {
    for (const l of LABS) {
      expect(l.session, l.slug).toBeGreaterThanOrEqual(1);
      expect(l.session, l.slug).toBeLessThanOrEqual(14);
    }
  });

  it("silabus memuat keempat belas sesi tanpa lompatan", () => {
    expect(SYLLABUS.map((s) => s.session)).toEqual([
      1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
    ]);
  });

  it("setiap pranala silabus menunjuk laboratorium yang benar-benar ada", () => {
    for (const entry of SYLLABUS) {
      if (entry.slug === undefined) continue;
      expect(findLab(entry.slug), `sesi ${entry.session}`).toBeDefined();
    }
  });

  it("setiap laboratorium tercantum di silabus", () => {
    const tercantum = new Set(SYLLABUS.map((s) => s.slug));
    for (const l of LABS) {
      expect(tercantum.has(l.slug), l.slug).toBe(true);
    }
  });

  it("findLab menolak slug yang tidak dikenal", () => {
    expect(findLab("tidak-ada")).toBeUndefined();
    expect(findLab("")).toBeUndefined();
  });

  it("judul dan penjelasannya lengkap dua bahasa", () => {
    for (const l of LABS) {
      expect(l.title.id.length, l.slug).toBeGreaterThan(3);
      expect(l.title.en.length, l.slug).toBeGreaterThan(3);
      // Penjelasan sepanjang satu kalimat pendek biasanya hanya mengulang
      // judulnya, yang tidak membantu siapa pun memilih laboratorium.
      expect(l.blurb.id.length, l.slug).toBeGreaterThan(120);
      expect(l.blurb.en.length, l.slug).toBeGreaterThan(120);
      expect(l.blurb.id, l.slug).not.toBe(l.blurb.en);
    }
  });

  it("setiap laboratorium punya catatan dan definisi", () => {
    for (const l of LABS) {
      expect(notesFor(l.slug), l.slug).toBeDefined();
    }
  });

  // Inilah uji yang sesungguhnya menjaga pemecahan kode: kalau sebuah jalur
  // `import()` salah tulis, hanya panggilan ini yang akan memberitahunya.
  it.each(LABS.map((l) => [l.slug, l] as const))(
    "modul %s bisa dimuat dan menyediakan mount",
    async (_slug, lab) => {
      const modul = await lab.load();
      expect(typeof modul.mount).toBe("function");
      // Satu argumen: elemen tempat laboratorium dipasang.
      expect(modul.mount.length).toBe(1);
    },
  );
});
