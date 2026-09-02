// @vitest-environment happy-dom
/**
 * Uji yang benar-benar memasang tiap laboratorium, lalu membaca layarnya.
 *
 * # Kenapa membaca kamus saja tidak cukup
 *
 * Uji dwibahasa yang sudah ada memeriksa isi kamus: tiap pasangan punya kedua
 * bahasa dan keduanya berbeda. Seluruhnya hijau — dan seluruhnya akan tetap
 * hijau sementara sebuah tombol di dalam laboratorium bertuliskan "Hitung"
 * apa adanya, karena teks itu memang bukan bagian dari kamus mana pun.
 *
 * Kegagalan seperti itu hanya bisa dilihat dari arah yang berlawanan: pasang
 * laboratoriumnya dalam Bahasa Inggris, lalu baca setiap teks yang benar-benar
 * sampai ke layar.
 *
 * # Kenapa mesin WebAssembly-nya benar-benar dijalankan
 *
 * Karena sebagian besar teks laboratorium ini baru muncul **sesudah** ada
 * hasil: label langkah, keterangan pita, dan tabel rincian. Memasangnya tanpa
 * mesin menghasilkan halaman yang hampir kosong — dan halaman yang hampir
 * kosong lolos setiap pemeriksaan bocoran, karena ia tidak menampilkan apa pun
 * untuk bocor.
 *
 * Berkas `.wasm`-nya diambil dari cakram lewat pengganti `fetch`, karena di
 * luar peramban tidak ada yang bisa mengunduhnya.
 *
 * .Deckyx
 */

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { beforeAll, describe, expect, it } from "vitest";

import * as engine from "../web/src/engine.js";
import { setLang } from "../web/src/i18n.js";
import { LABS } from "../web/src/labs/registry.js";

const AKAR = join(dirname(fileURLToPath(import.meta.url)), "..");

/**
 * Kata Indonesia yang tidak mungkin muncul di halaman berbahasa Inggris.
 *
 * Sengaja kata fungsi. Kata benda seperti "data" dan "premis" sama atau nyaris
 * sama di kedua bahasa, dan memasukkannya akan membuat ujinya berteriak pada
 * halaman yang benar.
 */
const KATA_INDONESIA = new RegExp(
  String.raw`\b(yang|dengan|adalah|tidak|untuk|dari|pada|karena|jadi|bisa|akan|sebuah|supaya|sehingga|kalau|tetapi|atau|dan|ini|itu|lebih|sudah|masih|hanya|setiap|tiap|bila|juga|nilai|bukan)\b`,
  "i",
);

beforeAll(async () => {
  // Pengganti `fetch` yang membaca berkas dari cakram. wasm-bindgen memuat
  // modulnya lewat alamat, dan di luar peramban tidak ada yang melayani
  // alamat itu.
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

  // Jalur `instantiateStreaming` dimatikan. wasm-bindgen mencobanya lebih dulu
  // dan meneruskan galatnya apa adanya ketika content-type-nya sudah benar —
  // dan di luar peramban ia memang selalu gagal, karena `Response` milik
  // lingkungan uji bukan `Response` yang dikenal `WebAssembly`. Tanpanya,
  // wasm-bindgen jatuh ke `arrayBuffer()` yang bekerja di mana saja.
  const bertahap = WebAssembly.instantiateStreaming;
  delete (WebAssembly as { instantiateStreaming?: unknown }).instantiateStreaming;
  try {
    await engine.load();
  } finally {
    (WebAssembly as { instantiateStreaming?: unknown }).instantiateStreaming = bertahap;
  }
}, 60_000);

/**
 * Apakah sebuah simpul berada di dalam bahan yang sedang dianalisis.
 *
 * Laboratorium pengolahan bahasa membedah **morfologi bahasa Indonesia**:
 * kalimat contohnya tetap Indonesia di halaman berbahasa Inggris, karena
 * menjalankan pengupas imbuhan "me-", "di-", "-kan" pada kalimat Inggris tidak
 * mengajarkan apa pun. Bagian seperti itu ditandai `data-korpus` di tempatnya,
 * bukan dikecualikan dari sini dengan nama laboratorium — sehingga bagian lain
 * mana pun di laboratorium yang sama tetap dituntut diterjemahkan.
 */
function didalamKorpus(simpul: Node): boolean {
  let n: Node | null = simpul;
  while (n !== null) {
    if (n.nodeType === 1 && (n as Element).hasAttribute("data-korpus")) return true;
    n = n.parentNode;
  }
  return false;
}

/** Seluruh teks antarmuka yang sampai ke layar, di dalam sebuah wadah. */
function teksTampil(wadah: HTMLElement): string[] {
  const keluar: string[] = [];
  const jalan = wadah.ownerDocument.createTreeWalker(wadah, 4 /* SHOW_TEXT */);
  for (let n = jalan.nextNode(); n !== null; n = jalan.nextNode()) {
    if (didalamKorpus(n)) continue;
    const t = (n.textContent ?? "").trim();
    if (t !== "") keluar.push(t);
  }
  return keluar;
}

/** Atribut yang isinya dibaca manusia, dari seluruh elemen di dalam wadah. */
function atributTerbaca(wadah: HTMLElement): string[] {
  const keluar: string[] = [];
  for (const el of wadah.querySelectorAll("*")) {
    for (const nama of ["aria-label", "placeholder", "title", "alt"]) {
      const nilai = el.getAttribute(nama);
      if (nilai !== null && nilai.trim() !== "") keluar.push(`${nama}="${nilai}"`);
    }
  }
  return keluar;
}

/** Memasang satu laboratorium tanpa membacanya, mengembalikan wadahnya. */
async function pasangMentah(
  slug: string,
  b: "id" | "en",
): Promise<{ wadah: HTMLElement; lepas: () => void }> {
  const lab = LABS.find((l) => l.slug === slug);
  if (lab === undefined) throw new Error(`laboratorium tidak dikenal: ${slug}`);
  setLang(b);
  const wadah = document.createElement("div");
  document.body.appendChild(wadah);
  const modul = await lab.load();
  const bersihkan = modul.mount(wadah);
  return {
    wadah,
    lepas: () => {
      bersihkan();
      wadah.remove();
    },
  };
}

/** Memasang satu laboratorium dalam bahasa yang diminta, lalu membacanya. */
async function pasang(
  slug: string,
  b: "id" | "en",
): Promise<{ teks: string[]; atribut: string[]; lepas: () => void }> {
  const lab = LABS.find((l) => l.slug === slug);
  if (lab === undefined) throw new Error(`laboratorium tidak dikenal: ${slug}`);
  setLang(b);
  const wadah = document.createElement("div");
  document.body.appendChild(wadah);
  const modul = await lab.load();
  const bersihkan = modul.mount(wadah);
  const hasil = { teks: teksTampil(wadah), atribut: atributTerbaca(wadah) };
  return {
    ...hasil,
    lepas: () => {
      bersihkan();
      wadah.remove();
    },
  };
}

/**
 * Ambang jumlah simpul teks yang menandakan laboratoriumnya benar-benar
 * terpasang. Laboratorium yang gagal memasang dirinya tidak menampilkan apa
 * pun untuk bocor, jadi ia akan lolos setiap pemeriksaan bocoran.
 */
const AMBANG_TERISI = 12;

describe("laboratorium yang benar-benar terpasang", () => {
  it("mesin WebAssembly-nya benar-benar hidup", () => {
    // Kalau tidak, seluruh uji di bawah ini sedang membaca halaman kosong.
    expect(engine.version().length).toBeGreaterThan(0);
  });

  for (const lab of LABS) {
    it(`${lab.slug} tidak menyisakan kata Indonesia dalam Bahasa Inggris`, async () => {
      const { teks, atribut, lepas } = await pasang(lab.slug, "en");
      try {
        expect(teks.length, `${lab.slug} tidak terpasang`).toBeGreaterThan(AMBANG_TERISI);
        const bocor = [...teks, ...atribut].filter((t) => KATA_INDONESIA.test(t));
        expect(bocor, `${lab.slug}\n${bocor.join("\n")}`).toEqual([]);
      } finally {
        lepas();
      }
    });
  }

  it("dalam Bahasa Indonesia memang berbahasa Indonesia", async () => {
    // Pemeriksaan arah sebaliknya: tanpa ini, pemasangan yang gagal total
    // terlihat sama persis seperti terjemahan yang sempurna.
    for (const lab of LABS) {
      const { teks, lepas } = await pasang(lab.slug, "id");
      try {
        expect(teks.length, `${lab.slug} tidak terpasang`).toBeGreaterThan(AMBANG_TERISI);
        expect(
          teks.some((t) => KATA_INDONESIA.test(t)),
          `${lab.slug} tidak memuat satu pun kata Indonesia`,
        ).toBe(true);
      } finally {
        lepas();
      }
    }
  });
});

describe("penandaan bahan yang dianalisis", () => {
  it("hanya laboratorium pengolahan bahasa yang memakainya", async () => {
    // Penandaan ini melubangi pemeriksaan bocoran, jadi setiap pemakaiannya
    // harus disengaja. Laboratorium lain yang mulai memakainya — biasanya
    // untuk membungkam uji yang menemukan bocoran sungguhan — akan gagal di
    // sini, bukan diam-diam lolos.
    const DIIZINKAN = new Set(["nlp"]);
    for (const lab of LABS) {
      const { wadah, lepas } = await pasangMentah(lab.slug, "en");
      try {
        const bertanda = wadah.querySelectorAll("[data-korpus]").length;
        if (!DIIZINKAN.has(lab.slug)) {
          expect(bertanda, `${lab.slug} menandai bahan tanpa alasan`).toBe(0);
        }
      } finally {
        lepas();
      }
    }
  });

  it("laboratorium pengolahan bahasa menyatakan alasannya di layar", async () => {
    // Bahan berbahasa Indonesia di halaman berbahasa Inggris harus terlihat
    // disengaja. Tanpa kalimat itu ia tampak seperti terjemahan yang terlupa,
    // dan pembacanya tidak punya cara membedakan keduanya.
    const { teks, lepas } = await pasang("nlp", "en");
    try {
      const semua = teks.join(" ");
      expect(semua).toContain("Indonesian");
    } finally {
      lepas();
    }
  });
});
