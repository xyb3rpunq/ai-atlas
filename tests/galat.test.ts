// @vitest-environment happy-dom
/**
 * Uji yang menuntut setiap kegagalan mesin punya kalimat di kedua bahasa.
 *
 * # Kenapa daftarnya diambil dari mesin, bukan ditulis di sini
 *
 * Karena daftar yang ditulis tangan akan tertinggal. Kode galat baru
 * ditambahkan di Rust, bukan di sini, dan tidak ada yang mengingatkan bahwa
 * terjemahannya belum ada — sampai seseorang menemuinya. Yang menemuinya
 * adalah pengguna yang sedang mengalami kegagalan, dan pada saat itu ia paling
 * membutuhkan kalimat yang bisa ia baca.
 *
 * Jadi `error_codes()` di sisi mesin menyerahkan seluruh kode beserta jumlah
 * argumennya, dan uji ini menuntut kamusnya memuat tepat kode-kode itu.
 *
 * .Deckyx
 */

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { beforeAll, describe, expect, it } from "vitest";

import * as engine from "../web/src/engine.js";
import { PENANDA, PESAN, bacaGalat, kalimatGalat } from "../web/src/galat.js";
import { setLang } from "../web/src/i18n.js";

const AKAR = join(dirname(fileURLToPath(import.meta.url)), "..");

/**
 * Kata Indonesia yang tidak mungkin muncul di kalimat berbahasa Inggris.
 *
 * Sengaja kata fungsi — kata sambung, kata depan, kata ganti.
 */
const KATA_INDONESIA = new RegExp(
  String.raw`\b(yang|dengan|adalah|tidak|untuk|dari|pada|karena|jadi|bisa|akan|sebuah|supaya|sehingga|kalau|tetapi|atau|dan|ini|itu|lebih|sudah|masih|hanya|setiap|tiap|bila|juga|belum|harus|diberi|kosong)\b`,
  "i",
);

let kodeMesin: { kode: string; argumen: number }[] = [];

beforeAll(async () => {
  const aslinya = globalThis.fetch;
  globalThis.fetch = (async (masukan: RequestInfo | URL, init?: RequestInit) => {
    const alamat = String(masukan instanceof Request ? masukan.url : masukan);
    if (alamat.endsWith(".wasm")) {
      const nama = alamat.split(/[?#]/)[0]!.split("/").pop()!;
      const isi = readFileSync(join(AKAR, "web", "pkg", nama));
      return new Response(isi, { headers: { "content-type": "application/wasm" } });
    }
    return aslinya(masukan, init);
  }) as typeof fetch;

  const bertahap = WebAssembly.instantiateStreaming;
  delete (WebAssembly as { instantiateStreaming?: unknown }).instantiateStreaming;
  try {
    await engine.load();
  } finally {
    (WebAssembly as { instantiateStreaming?: unknown }).instantiateStreaming = bertahap;
  }
  kodeMesin = engine.errorCodes();
}, 60_000);

/** Jumlah penanda bernomor tertinggi di dalam sebuah kalimat. */
function penandaTertinggi(teks: string): number {
  let tertinggi = 0;
  for (const c of teks.matchAll(PENANDA)) tertinggi = Math.max(tertinggi, Number(c[1]));
  return tertinggi;
}

describe("kamus pesan galat", () => {
  it("mesin benar-benar menyerahkan daftar kodenya", () => {
    // Kalau daftarnya kosong, seluruh uji di bawah ini hijau karena tidak
    // memeriksa apa-apa.
    expect(kodeMesin.length).toBeGreaterThan(60);
  });

  it("setiap kode dari mesin punya kalimatnya", () => {
    const hilang = kodeMesin.filter((k) => PESAN[k.kode] === undefined).map((k) => k.kode);
    expect(hilang, hilang.join("\n")).toEqual([]);
  });

  it("tidak ada kalimat yang tidak punya kode", () => {
    // Kalimat tanpa kode adalah terjemahan yang dirawat tanpa satu pun galat
    // yang menghasilkannya — biasanya sisa kode yang sudah dihapus di Rust.
    const dikenal = new Set(kodeMesin.map((k) => k.kode));
    const berlebih = Object.keys(PESAN).filter((k) => !dikenal.has(k));
    expect(berlebih, berlebih.join("\n")).toEqual([]);
  });

  it("jumlah penandanya sepadan dengan jumlah argumen di sisi mesin", () => {
    // Penanda yang lebih banyak daripada argumennya akan tampil apa adanya
    // sebagai "%2" di tengah kalimat; yang lebih sedikit membuang nilai yang
    // justru menjelaskan kegagalannya.
    const salah: string[] = [];
    for (const { kode, argumen } of kodeMesin) {
      const pasangan = PESAN[kode];
      if (pasangan === undefined) continue;
      for (const b of ["id", "en"] as const) {
        const n = penandaTertinggi(pasangan[b]);
        if (n !== argumen) {
          salah.push(`${kode} (${b}): ${n} penanda, ${argumen} argumen`);
        }
      }
    }
    expect(salah, salah.join("\n")).toEqual([]);
  });

  it("setiap kalimat terisi di kedua bahasa dan keduanya berbeda", () => {
    for (const [kode, pasangan] of Object.entries(PESAN)) {
      expect(pasangan.id.trim(), kode).not.toBe("");
      expect(pasangan.en.trim(), kode).not.toBe("");
      expect(pasangan.id, kode).not.toBe(pasangan.en);
    }
  });

  it("sisi Inggrisnya benar-benar berbahasa Inggris", () => {
    const bocor = Object.entries(PESAN)
      .filter(([, p]) => KATA_INDONESIA.test(p.en))
      .map(([kode, p]) => `${kode}: ${p.en}`);
    expect(bocor, bocor.join("\n")).toEqual([]);
  });
});

describe("perakit kalimat galat", () => {
  it("menyisipkan nilai menurut nomornya, bukan urutannya", () => {
    // Urutan kata berbeda antarbahasa, dan justru itu sebabnya penandanya
    // bernomor: nilai yang sama muncul di tempat yang berbeda.
    setLang("id");
    const id = kalimatGalat({ kode: "agen.jumlah_ruangan", arg: ["8", "99"] });
    expect(id).toContain("1 sampai 8");
    expect(id).toContain("99");
    setLang("en");
    const en = kalimatGalat({ kode: "agen.jumlah_ruangan", arg: ["8", "99"] });
    expect(en).toContain("1 to 8");
    expect(en).toContain("99");
  });

  it("kode yang tidak dikenal tetap bisa dilaporkan", () => {
    // Kegagalan yang tampil sebagai ruang kosong tidak bisa ditelusuri siapa
    // pun — termasuk oleh yang menulis kodenya.
    const teks = kalimatGalat({ kode: "belum.ada", arg: ["7"] });
    expect(teks).toContain("belum.ada");
    expect(teks).toContain("7");
  });

  it("penanda tanpa nilai dibiarkan apa adanya, bukan menjadi undefined", () => {
    setLang("id");
    const teks = kalimatGalat({ kode: "agen.jumlah_ruangan", arg: ["8"] });
    expect(teks).not.toContain("undefined");
    expect(teks).toContain("%2");
  });

  it("membaca bentuk galat, dan menolak yang bukan", () => {
    expect(bacaGalat({ kode: "cf.daftar_kosong", arg: [] })).toEqual({
      kode: "cf.daftar_kosong",
      arg: [],
    });
    expect(bacaGalat("kalimat lama")).toBeNull();
    expect(bacaGalat(null)).toBeNull();
    expect(bacaGalat({ kode: 1, arg: [] })).toBeNull();
    expect(bacaGalat({ kode: "x" })).toBeNull();
    expect(bacaGalat({ kode: "x", arg: [1] })).toBeNull();
  });
});

describe("kegagalan sungguhan dari mesin", () => {
  it("sampai ke antarmuka sebagai kalimat berbahasa pembacanya", () => {
    // Jalan lengkapnya, bukan hanya kamusnya: mesin menolak masukan, amplopnya
    // dibongkar, kalimatnya dirakit.
    setLang("en");
    expect(() => engine.cfCombine([])).toThrow(/empty/i);
    setLang("id");
    expect(() => engine.cfCombine([])).toThrow(/kosong/i);
  });

  it("kegagalan berargumen membawa nilainya", () => {
    setLang("en");
    let pesan = "";
    try {
      engine.fuzzyInfer(
        { inputs: [], output: { name: "x", min: 0, max: 1, sets: [] }, rules: [] },
        [],
        "mamdani",
        "centroid",
        201,
      );
    } catch (e) {
      pesan = (e as Error).message;
    }
    expect(pesan.length).toBeGreaterThan(0);
    expect(KATA_INDONESIA.test(pesan), pesan).toBe(false);
  });
});
