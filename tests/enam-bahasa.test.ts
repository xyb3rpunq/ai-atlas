/**
 * @vitest-environment happy-dom
 *
 * Uji halaman "Enam bahasa, satu angka" beserta datanya.
 *
 * Halaman ini menampilkan angka yang tidak dihitungnya sendiri: pola bit itu
 * dipancarkan enam harness di lima repositori, tiga di antaranya hanya bisa
 * dijalankan di CI. Karena itu kegagalannya senyap dalam dua arah sekaligus.
 *
 * Sebuah bahasa yang berkasnya belum terkumpul akan muncul sebagai kolom
 * kosong, dan kolom kosong terbaca sebagai perbedaan. Sebaliknya, sebuah
 * selisih yang **melebihi** tingkat keterbandingannya adalah kegagalan
 * konformansi sungguhan — dan kalau halaman ini menggolongkannya sebagai
 * "beda, masih lolos", ia akan menutupi persis hal yang seluruh proyek ini
 * dibangun untuk menemukan.
 *
 * .Deckyx
 */

import { describe, expect, it } from "vitest";

import {
  BAHASA,
  CERITA,
  RINGKASAN,
  TENGARA,
  jarakUlp,
  keBit,
  keDesimal,
} from "../web/src/enam-bahasa.js";

describe("pembacaan pola bit", () => {
  it("membentangkan 16 digit heksadesimal menjadi 64 bit", () => {
    expect(keBit("0000000000000000")).toBe("0".repeat(64));
    expect(keBit("ffffffffffffffff")).toBe("1".repeat(64));
    // 1,0 adalah eksponen bias penuh dengan mantisa nol.
    expect(keBit("3ff0000000000000")).toBe("0" + "01111111111" + "0".repeat(52));
  });

  it("membedakan nol positif dari nol negatif saat ditulis", () => {
    // Inilah pokok beberapa tengara di halaman ini. Menuliskan keduanya
    // sebagai "0" akan menghapus justru yang sedang ditunjukkan.
    expect(keDesimal("0000000000000000")).toBe("0");
    expect(keDesimal("8000000000000000")).toBe("−0");
  });

  it("menuliskan nilai yang tidak berhingga sebagai lambang", () => {
    expect(keDesimal("7ff0000000000000")).toBe("∞");
    expect(keDesimal("fff0000000000000")).toBe("−∞");
    expect(keDesimal("7ff8000000000000")).toBe("NaN");
  });

  it("menghitung jarak ULP sebagai langkah, bukan selisih pecahan", () => {
    // Dua pola bit yang berurutan berjarak tepat satu langkah, berapa pun
    // besarnya nilainya. Pengurangan pecahan kehilangan ketelitian persis di
    // daerah yang sedang diukur.
    expect(jarakUlp("3ff0000000000000", "3ff0000000000001")).toBe(1);
    expect(jarakUlp("3ff0000000000000", "3ff0000000000000")).toBe(0);
    expect(jarakUlp("0000000000000001", "0000000000000000")).toBe(1);
  });

  it("mengukur jarak yang melintasi nol tanpa melompat", () => {
    // Bentuk bertanda-besaran IEEE-754 membuat −0 dan +0 berjarak 2⁶³ kalau
    // dibaca sebagai bilangan bulat apa adanya. Yang benar: satu langkah dari
    // pecahan terkecil negatif ke nol negatif, satu lagi ke nol positif.
    expect(jarakUlp("8000000000000000", "0000000000000000")).toBe(0);
    expect(jarakUlp("8000000000000001", "0000000000000001")).toBe(2);
  });

  it("mengembalikan null untuk nilai yang tidak berhingga", () => {
    expect(jarakUlp("7ff0000000000000", "3ff0000000000000")).toBeNull();
    expect(jarakUlp("7ff8000000000000", "3ff0000000000000")).toBeNull();
  });
});

describe("data lintas bahasa", () => {
  it("memuat keenam bahasa, dan keenamnya benar-benar terkumpul", () => {
    // Kolom kosong terbaca sebagai perbedaan. Halaman yang menampilkan lima
    // bahasa dengan judul "enam bahasa" salah dua kali sekaligus.
    expect(BAHASA).toHaveLength(6);
    const belum = BAHASA.filter((b) => !b.ada).map((b) => b.nama);
    expect(belum, "berkas pola bit belum terkumpul").toEqual([]);
  });

  it("mencatat asal tiap berkas pancaran", () => {
    for (const b of BAHASA) {
      expect(b.perintah, b.nama).toBeTruthy();
      if (b.kode === "rust") continue;
      // Tanpa versi dan cap waktu, sebuah pola bit tidak bisa diulang: pustaka
      // matematika berubah antar rilis, dan itulah yang justru diukur di sini.
      expect(b.versi, b.nama).toBeTruthy();
      expect(b.dihasilkan, b.nama).toMatch(/^\d{4}-\d{2}-\d{2}T/);
      expect(b.pernyataan, b.nama).toBe(3796);
    }
  });

  it("tidak ada satu pun selisih yang melebihi tingkatnya", () => {
    // Uji terpenting di berkas ini. Selisih yang melebihi tingkat
    // keterbandingannya adalah kegagalan konformansi — dan halaman ini
    // menampilkannya di kolom "melebihi tingkat", yang harus selalu nol.
    // Kalau suatu hari tidak, yang dibutuhkan adalah penyelidikan, bukan
    // pelonggaran.
    for (const r of RINGKASAN) {
      for (const [kode, h] of Object.entries(r.perBahasa)) {
        expect(h.luarToleransi, `${r.berkas} ${kode}`).toBe(0);
        expect(h.hilang, `${r.berkas} ${kode}`).toBe(0);
      }
    }
  });

  it("menjumlahkan 3.796 pernyataan per bahasa", () => {
    const total = RINGKASAN.reduce((n, r) => n + r.pernyataan, 0);
    expect(total).toBe(3796);
    for (const b of BAHASA.filter((x) => x.kode !== "rust")) {
      const jumlah = RINGKASAN.reduce((n, r) => {
        const h = r.perBahasa[b.kode];
        return n + (h ? h.identik + h.tandaNol + h.dalamToleransi + h.luarToleransi : 0);
      }, 0);
      expect(jumlah, b.nama).toBe(3796);
    }
  });

  it("menyebut tingkat keterbandingan yang dikenal untuk tiap berkas", () => {
    const DIKENAL = /^(BitExact|NearlyEqual\(\d+\)|CancellingDifference\(\d+\)|PropertyOnly)$/;
    for (const r of RINGKASAN) {
      expect(r.tingkat, r.berkas).toMatch(DIKENAL);
    }
  });
});

describe("tengara", () => {
  it("setiap tengara punya ceritanya", () => {
    // Tengara tanpa cerita hanyalah baris data yang kebetulan dipilih. Yang
    // membuatnya layak diterbitkan justru penjelasannya.
    for (const t of TENGARA) {
      expect(Object.keys(CERITA), t.id).toContain(t.id);
    }
    // Dan sebaliknya: cerita tanpa tengara adalah prosa yang dirawat tanpa
    // satu pun halaman yang menampilkannya.
    const id = new Set(TENGARA.map((t) => t.id));
    for (const kunci of Object.keys(CERITA)) {
      expect(id.has(kunci), kunci).toBe(true);
    }
  });

  it("ceritanya dwibahasa, dan keduanya berbeda", () => {
    for (const [kunci, c] of Object.entries(CERITA)) {
      for (const bagian of [c.judul, c.isi]) {
        expect(bagian.id.trim(), kunci).not.toBe("");
        expect(bagian.en.trim(), kunci).not.toBe("");
        expect(bagian.id, kunci).not.toBe(bagian.en);
      }
      // Penjelasan sepanjang satu kalimat pendek tidak menjelaskan apa pun.
      expect(c.isi.id.length, kunci).toBeGreaterThan(200);
      expect(c.isi.en.length, kunci).toBeGreaterThan(200);
    }
  });

  it("membawa jawaban keenam bahasa untuk tiap pernyataannya", () => {
    for (const t of TENGARA) {
      expect(t.pernyataan.length, t.id).toBeGreaterThan(0);
      for (const p of t.pernyataan) {
        for (const b of BAHASA) {
          expect(Object.keys(p.hasil), `${t.id} ${p.kolom} ${b.kode}`).toContain(b.kode);
          expect(p.hasil[b.kode], `${t.id} ${b.kode}`).toMatch(/^[0-9a-f]{16}$/);
        }
      }
    }
  });

  it("menggolongkan tiap jawaban, dan tidak satu pun melebihi tingkatnya", () => {
    for (const t of TENGARA) {
      for (const p of t.pernyataan) {
        for (const b of BAHASA) {
          const g = p.golongan[b.kode];
          expect(g, `${t.id} ${b.kode}`).toBeDefined();
          expect(g, `${t.id} ${b.kode}`).not.toBe("luarToleransi");
        }
      }
    }
  });

  it("tengara nol negatif benar-benar bernilai nol negatif", () => {
    // Tengara yang ceritanya tentang tanda nol tetapi angkanya bukan nol
    // negatif adalah penjelasan yang tidak menggambarkan apa pun. Dua tengara
    // di halaman ini berdiri di atas fakta itu.
    for (const id of ["nol-negatif-cf", "nol-negatif-entropi"]) {
      const t = TENGARA.find((x) => x.id === id);
      expect(t, id).toBeDefined();
      expect(t!.pernyataan[0]!.hasil["rust"], id).toBe("8000000000000000");
      expect(keDesimal(t!.pernyataan[0]!.hasil["rust"]!), id).toBe("−0");
    }
  });

  it("tengara pembangkit acak membandingkan bilangan bulat dan pecahannya", () => {
    const t = TENGARA.find((x) => x.id === "pembangkit-acak");
    expect(t?.pernyataan.map((p) => p.kolom)).toEqual(["next_u64_hex", "next_f64_hex"]);
    // Tingkatnya wajib BitExact: tidak ada pembulatan yang terlibat sama
    // sekali, jadi tidak ada alasan bahasa mana pun berbeda.
    expect(t?.tingkat).toBe("BitExact");
    for (const p of t!.pernyataan) {
      for (const b of BAHASA) {
        expect(p.golongan[b.kode], `${b.kode} ${p.kolom}`).toBe("identik");
      }
    }
  });
});
