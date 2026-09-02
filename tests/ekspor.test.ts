/**
 * @vitest-environment happy-dom
 *
 * Uji pengekspor laporan laboratorium.
 *
 * Dua hal berbeda diuji di sini dan keduanya gagal secara diam-diam.
 *
 * Yang pertama: pembacaan isi dari tampilan. Karena isinya dipungut dari DOM
 * lewat atribut `data-ekspor`, sebuah kelalaian di `ui.ts` — atribut yang
 * hilang saat komponennya ditata ulang — tidak membuat apa pun rusak. Situsnya
 * tetap jalan, tombol unduhnya tetap bekerja, dan berkasnya sekadar kehilangan
 * satu blok. Karena itu ujinya dibangun dari komponen bersama yang sungguhan,
 * bukan dari HTML tiruan: kalau kontraknya putus, uji ini yang memberi tahu.
 *
 * Yang kedua: bentuk CSV-nya. Berkas yang gagal terbuka rapi di Excel sama
 * tidak bergunanya dengan berkas yang tidak pernah diunduh, dan pengguna yang
 * melihat seluruh baris menumpuk di satu kolom akan menyimpulkan situsnya
 * rusak tanpa pernah melaporkannya.
 *
 * .Deckyx
 */

import { beforeEach, describe, expect, it } from "vitest";

import { LABEL, bacaIsi, cell, csv, fileName, reportRows } from "../web/src/ekspor.js";
import { setLang } from "../web/src/i18n.js";
import { buttonRow, card, el, readout, slider, stepList, table } from "../web/src/ui.js";

const CR = String.fromCharCode(13);
const LF = String.fromCharCode(10);

/** Halaman lab tiruan yang dibangun dari komponen bersama yang sungguhan. */
function halaman(): HTMLElement {
  return el("div", {
    children: [
      card(
        "Setelan",
        slider({
          label: "Kepercayaan awal",
          min: 0,
          max: 1,
          step: 0.01,
          value: 0.35,
          onInput: () => {},
        }),
        buttonRow([
          { label: "Konjungsi", selected: true, onClick: () => {} },
          { label: "Disjungsi", selected: false, onClick: () => {} },
        ]),
      ),
      card(
        "Hasil",
        readout("CF gabungan", "0.7250"),
        stepList([
          { label: "Langkah 1", formula: "CF = 0.35 + 0.60 x (1 - 0.35)" },
          { label: "Langkah 2", formula: "CF = 0.7400" },
        ]),
      ),
      card(
        "Rincian",
        table(
          ["aturan", "bobot"],
          [
            ["R1", 0.35],
            ["R2", 0.6],
          ],
        ),
      ),
    ],
  });
}

beforeEach(() => {
  setLang("id");
});

describe("cell", () => {
  it("mengutip sel yang memuat koma, kutip, atau baris baru", () => {
    expect(cell("ada, koma")).toBe('"ada, koma"');
    expect(cell('ada "kutip"')).toBe('"ada ""kutip"""');
    expect(cell(`dua${LF}baris`)).toBe(`"dua${LF}baris"`);
  });

  it("mengubah kosong menjadi sel kosong, bukan tulisan undefined", () => {
    expect(cell(null)).toBe("");
    expect(cell(undefined)).toBe("");
    // Nol adalah nilai yang sah dan sering justru yang paling penting; sebuah
    // penjagaan yang memakai `if (!value)` akan menghapusnya diam-diam.
    expect(cell(0)).toBe("0");
  });
});

describe("csv", () => {
  it("diawali BOM dan petunjuk pemisah, berakhiran CRLF", () => {
    const teks = csv([["a"], [1]]);
    expect(teks.startsWith("﻿")).toBe(true);
    expect(teks.split(CR + LF)[0]).toBe("﻿sep=,");
    expect(teks.endsWith(CR + LF)).toBe(true);
  });

  it("tidak meninggalkan baris dengan kutip ganjil", () => {
    // Kutip ganjil berarti sel yang terpotong, dan Excel akan menelan sisa
    // berkasnya tanpa satu pun pesan galat.
    const teks = csv(reportRows("Certainty Factor", 6, bacaIsi(halaman())));
    for (const baris of teks.split(CR + LF)) {
      expect((baris.match(/"/g) ?? []).length % 2, baris.slice(0, 60)).toBe(0);
    }
  });
});

describe("bacaIsi", () => {
  it("menyusun laporan mengikuti kartu di halaman", () => {
    const isi = bacaIsi(halaman());
    expect(isi.bagian.map((b) => b.judul)).toEqual(["Setelan", "Hasil", "Rincian"]);
  });

  it("memungut penggeser sebagai masukan, lengkap dengan labelnya", () => {
    const [setelan] = bacaIsi(halaman()).bagian;
    expect(setelan?.masukan).toEqual([{ label: "Kepercayaan awal", nilai: "0.35" }]);
  });

  it("membaca nilai penggeser yang berubah, bukan nilai awalnya", () => {
    // Inilah alasan nilainya tidak disalin ke atribut saat digambar: laporan
    // yang memuat setelan awal padahal penggunanya menggeser tiga kali adalah
    // laporan yang salah, dan salahnya tidak kelihatan.
    const root = halaman();
    const isian = root.querySelector("input");
    expect(isian).not.toBeNull();
    isian!.value = "0.87";
    expect(bacaIsi(root).bagian[0]?.masukan[0]?.nilai).toBe("0.87");
  });

  it("mencatat tombol yang sedang aktif sebagai pilihan", () => {
    const [setelan] = bacaIsi(halaman()).bagian;
    expect(setelan?.pilihan).toEqual([{ label: "Setelan", terpilih: "Konjungsi" }]);
  });

  it("mengabaikan baris aksi, termasuk aksi utama yang ditonjolkan", () => {
    // Tombol yang ditonjolkan belum tentu tombol yang terpilih. "Tambah
    // bukti" ditonjolkan karena ia aksi utama panelnya; mencatatnya sebagai
    // "dipilih: Tambah bukti" menghasilkan baris yang tidak berarti apa pun
    // bagi orang yang membaca laporannya besok.
    const root = el("div", {
      children: [
        card(
          "Aksi",
          buttonRow([
            { label: "Tambah bukti", primary: true, onClick: () => {} },
            { label: "Ulangi", onClick: () => {} },
          ]),
        ),
      ],
    });
    expect(bacaIsi(root).bagian).toEqual([]);
  });

  it("mencatat pilihan yang belum ditentukan sebagai tidak ada pilihan", () => {
    const root = el("div", {
      children: [
        card(
          "Mesin",
          buttonRow([
            { label: "Mamdani", selected: false, onClick: () => {} },
            { label: "Sugeno", selected: false, onClick: () => {} },
          ]),
        ),
      ],
    });
    expect(bacaIsi(root).bagian).toEqual([]);
  });

  it("memungut isian teks bebas beserta nama dari aria-label", () => {
    // Kalimat yang diurai dan rumus yang dibuktikan adalah data yang dibawa
    // sendiri oleh penggunanya, dan justru itulah yang paling ingin ia simpan.
    const area = el("textarea", { attrs: { "aria-label": "Teks yang diproses" } });
    area.value = "  saya sedang belajar kecerdasan buatan  ";
    const root = el("div", { children: [card("Teks", area)] });
    expect(bacaIsi(root).bagian[0]?.masukan).toEqual([
      { label: "Teks yang diproses", nilai: "saya sedang belajar kecerdasan buatan" },
    ]);
  });

  it("melewati isian teks yang masih kosong", () => {
    // Kotak yang belum diisi bukan masukan; mencatatnya hanya menambah baris
    // kosong yang membuat laporan lebih sulit dibaca.
    const area = el("textarea", { attrs: { "aria-label": "Teks" } });
    const root = el("div", { children: [card("Teks", area)] });
    expect(bacaIsi(root).bagian).toEqual([]);
  });

  it("tidak menghitung penggeser dua kali lewat jalur isian bebas", () => {
    const isi = bacaIsi(halaman());
    expect(isi.bagian[0]?.masukan).toHaveLength(1);
  });

  it("memakai label dari elemen label pembungkusnya bila aria-label tidak ada", () => {
    const isian = el("input", { attrs: { type: "text" } });
    isian.value = "Demam";
    const root = el("div", {
      children: [
        card(
          "Bukti 1",
          el("label", {
            class: "field",
            children: [
              el("span", { class: "field__label", text: "Nama bukti" }),
              isian,
            ],
          }),
        ),
      ],
    });
    expect(bacaIsi(root).bagian[0]?.masukan).toEqual([
      { label: "Nama bukti", nilai: "Demam" },
    ]);
  });

  it("menjaga urutan masukan sama dengan urutannya di layar", () => {
    // Dua sapuan terpisah — penggeser dulu, lalu isian teks — membuat nama
    // sebuah bukti tercatat di bawah angkanya sendiri, dan pembaca laporannya
    // harus menebak ke atas untuk tahu angka itu milik siapa.
    const nama = el("input", { attrs: { type: "text", "aria-label": "Bukti" } });
    nama.value = "Demam";
    const root = el("div", {
      children: [
        card(
          "Bukti 1",
          nama,
          slider({ label: "MB", min: 0, max: 1, step: 0.01, value: 0.6, onInput: () => {} }),
          slider({ label: "MD", min: 0, max: 1, step: 0.01, value: 0.1, onInput: () => {} }),
        ),
      ],
    });
    expect(bacaIsi(root).bagian[0]?.masukan.map((m) => m.label)).toEqual([
      "Bukti",
      "MB",
      "MD",
    ]);
  });

  it("memungut hasil dan langkah pada kartunya masing-masing", () => {
    const bagian = bacaIsi(halaman()).bagian;
    const hasil = bagian.find((b) => b.judul === "Hasil");
    expect(hasil?.hasil).toEqual([{ label: "CF gabungan", nilai: "0.7250" }]);
    expect(hasil?.langkah).toHaveLength(2);
    expect(hasil?.langkah[0]).toEqual({
      label: "Langkah 1",
      rumus: "CF = 0.35 + 0.60 x (1 - 0.35)",
    });
    // Kartu setelan tidak boleh ikut kebagian isi kartu hasil.
    expect(bagian.find((b) => b.judul === "Setelan")?.hasil).toEqual([]);
  });

  it("memungut tabel beserta kepala dan isinya", () => {
    const rincian = bacaIsi(halaman()).bagian.find((b) => b.judul === "Rincian");
    expect(rincian?.tabel).toHaveLength(1);
    expect(rincian?.tabel[0]?.kepala).toEqual(["aturan", "bobot"]);
    expect(rincian?.tabel[0]?.baris).toEqual([
      ["R1", "0.3500"],
      ["R2", "0.6000"],
    ]);
  });

  it("menghitung isi kartu bersarang sekali saja, di kartu terdekatnya", () => {
    // Beberapa lab menaruh kartu hasil di dalam kartu setelan. Tanpa aturan
    // pemilik terdekat, isinya muncul dua kali: sekali di bawah judulnya
    // sendiri dan sekali lagi di bawah judul induknya.
    const root = el("div", {
      children: [card("Luar", readout("a", "1"), card("Dalam", readout("b", "2")))],
    });
    const isi = bacaIsi(root);
    expect(isi.bagian.map((b) => b.judul)).toEqual(["Luar", "Dalam"]);
    expect(isi.bagian.find((b) => b.judul === "Luar")?.hasil).toEqual([
      { label: "a", nilai: "1" },
    ]);
    expect(isi.bagian.find((b) => b.judul === "Dalam")?.hasil).toEqual([
      { label: "b", nilai: "2" },
    ]);
  });

  it("tidak membuang isi yang berada di luar kartu mana pun", () => {
    // Sebuah lab yang tidak memakai `card()` akan menghasilkan berkas kosong
    // kalau isi tanpa kartu dibuang — dan kosongnya tidak memberi tanda apa
    // pun bahwa ada yang hilang.
    const root = el("div", { children: [readout("tanpa kartu", "9")] });
    const isi = bacaIsi(root);
    expect(isi.bagian).toHaveLength(1);
    expect(isi.bagian[0]?.judul).toBe("");
    expect(isi.bagian[0]?.hasil).toEqual([{ label: "tanpa kartu", nilai: "9" }]);
  });

  it("membuang kartu yang tidak memuat apa pun untuk diekspor", () => {
    const root = el("div", {
      children: [card("Penjelasan", el("p", { text: "Sekadar prosa." })), ...halaman().children],
    });
    expect(bacaIsi(root).bagian.map((b) => b.judul)).not.toContain("Penjelasan");
  });
});

describe("reportRows", () => {
  it("memberi judul pada tiap blok, bukan menumpuk tabel tanpa nama", () => {
    const rows = reportRows("Certainty Factor", 6, bacaIsi(halaman()));
    const tunggal = rows.filter((r) => r.length === 1).map((r) => String(r[0]));
    for (const judul of ["Setelan", "Hasil", "Rincian"]) {
      expect(tunggal).toContain(judul);
    }
  });

  it("menyebutkan lab dan sesinya di kepala berkas", () => {
    const rows = reportRows("Certainty Factor", 6, bacaIsi(halaman()));
    const kepala = rows.slice(0, 4).map((r) => r.map(String).join("|"));
    expect(kepala.join(" ")).toContain("Certainty Factor");
    expect(kepala.join(" ")).toContain("6");
  });

  it("menghilangkan baris sesi untuk halaman tanpa sesi kuliah", () => {
    const rows = reportRows("Bebas", undefined, bacaIsi(halaman()));
    const label = String(reportRows("Bebas", 1, bacaIsi(halaman()))[3]?.[0]);
    expect(rows.map((r) => String(r[0]))).not.toContain(label);
  });

  it("selalu menutup dengan catatan asal angkanya", () => {
    const rows = reportRows("Certainty Factor", 6, bacaIsi(halaman()));
    const ekor = rows.slice(-3).map((r) => String(r[0]));
    expect(ekor[0]).toBe(LABEL.notes.id);
    expect(ekor[1]).toContain("WebAssembly");
  });

  it("mengikuti bahasa yang sedang aktif", () => {
    const isi = bacaIsi(halaman());
    setLang("en");
    const en = reportRows("Certainty Factor", 6, isi).map((r) => String(r[0]));
    setLang("id");
    const id = reportRows("Certainty Factor", 6, isi).map((r) => String(r[0]));
    expect(en).toContain(LABEL.notes.en);
    expect(id).toContain(LABEL.notes.id);
    expect(en).not.toContain(LABEL.notes.id);
  });

  it("setiap label punya kedua bahasa dan keduanya berbeda", () => {
    for (const [kunci, pasangan] of Object.entries(LABEL)) {
      expect(pasangan.id.trim(), kunci).not.toBe("");
      expect(pasangan.en.trim(), kunci).not.toBe("");
      // "no" adalah singkatan nomor dan memang sama di kedua bahasa; sisanya
      // harus benar-benar diterjemahkan, bukan disalin.
      if (kunci !== "no") expect(pasangan.id, kunci).not.toBe(pasangan.en);
    }
  });
});

describe("fileName", () => {
  it("menyebutkan labnya, bukan sekadar tanggalnya", () => {
    const nama = fileName("certainty-factor");
    expect(nama.startsWith("ai-atlas-certainty-factor-")).toBe(true);
    expect(nama.endsWith(".csv")).toBe(true);
  });

  it("membuang aksara yang membuat unduhan gagal senyap di Windows", () => {
    expect(fileName('a/b\\c:d*e?"f<g>h|i')).not.toMatch(/[:/\\?*"<>|]/);
  });

  it("tidak menghasilkan nama kosong untuk slug yang seluruhnya terbuang", () => {
    expect(fileName("///")).toContain("ai-atlas-lab-");
  });
});
