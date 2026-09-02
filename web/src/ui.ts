/**
 * Pembantu DOM seperlunya.
 *
 * Tidak ada kerangka kerja di sini dan itu disengaja: seluruh antarmuka hanya
 * berupa beberapa lusin simpul, sehingga membangunnya langsung lebih ringan
 * daripada memuat pustaka mana pun. Semua teks masuk lewat `textContent`,
 * jadi tidak ada jalur yang menyisipkan HTML mentah.
 */

/** Atribut dan properti yang bisa diberikan ke {@link el}. */
export interface ElOptions {
  /** Kelas CSS, dipisah spasi. */
  class?: string;
  /** Isi teks. Selalu dimasukkan sebagai teks, tidak pernah sebagai markup. */
  text?: string;
  /** Atribut HTML biasa. Nilai `false` atau `null` berarti atribut dilewati. */
  attrs?: Record<string, string | number | boolean | null | undefined>;
  /** Penangan peristiwa. */
  on?: Partial<Record<keyof HTMLElementEventMap, EventListener>>;
  /** Anak-anak yang langsung ditambahkan. */
  children?: (Node | string | null | undefined)[];
}

/** Membuat elemen beserta atribut, penangan, dan anaknya dalam satu panggilan. */
export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  options: ElOptions = {},
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (options.class) node.className = options.class;
  if (options.text !== undefined) node.textContent = options.text;
  if (options.attrs) {
    for (const [k, v] of Object.entries(options.attrs)) {
      if (v === null || v === undefined || v === false) continue;
      node.setAttribute(k, v === true ? "" : String(v));
    }
  }
  if (options.on) {
    for (const [event, handler] of Object.entries(options.on)) {
      if (handler) node.addEventListener(event, handler as EventListener);
    }
  }
  if (options.children) {
    for (const child of options.children) {
      if (child === null || child === undefined) continue;
      node.append(child);
    }
  }
  return node;
}

/** Mengosongkan sebuah elemen. */
export function clear(node: Element): void {
  node.replaceChildren();
}

/**
 * Memformat bilangan untuk ditampilkan.
 *
 * Memakai pemisah desimal titik di kedua bahasa supaya angka pada layar sama
 * persis dengan yang dihasilkan mesin, dan supaya bisa disalin ke kalkulator
 * tanpa diterjemahkan lebih dulu.
 */
export function fmt(value: number, digits = 4): string {
  if (!Number.isFinite(value)) {
    return value > 0 ? "∞" : Number.isNaN(value) ? "—" : "-∞";
  }
  const rounded = Number(value.toFixed(digits));
  return Object.is(rounded, -0) ? (0).toFixed(digits) : rounded.toFixed(digits);
}

/** Memformat sebagai persen. */
export function pct(value: number, digits = 2): string {
  if (!Number.isFinite(value)) return "—";
  return `${(value * 100).toFixed(digits)}%`;
}

/** Membatasi nilai ke rentang tertentu. */
export function clamp(value: number, lo: number, hi: number): number {
  return Math.min(hi, Math.max(lo, value));
}

/** Argumen untuk membuat penggeser bernilai. */
export interface SliderOptions {
  label: string;
  min: number;
  max: number;
  step: number;
  value: number;
  /** Pemformat nilai yang ditampilkan di sebelah label. */
  format?: (v: number) => string;
  onInput: (value: number) => void;
}

/** Penggeser dengan label dan pembacaan nilai yang ikut berubah. */
export function slider(options: SliderOptions): HTMLElement {
  const format = options.format ?? ((v: number) => fmt(v, 2));
  const readout = el("span", { class: "field__value", text: format(options.value) });
  const input = el("input", {
    attrs: {
      type: "range",
      min: options.min,
      max: options.max,
      step: options.step,
      value: options.value,
      "aria-label": options.label,
    },
    on: {
      input: (event) => {
        const v = Number((event.target as HTMLInputElement).value);
        readout.textContent = format(v);
        options.onInput(v);
      },
    },
  });
  return el("label", {
    class: "field",
    // Ditandai sebagai masukan yang bisa diekspor. Nilainya dibaca dari
    // `input` saat ekspor dijalankan, bukan disalin ke atribut di sini —
    // menyalinnya berarti dua tempat yang harus dijaga tetap sepadan, dan
    // yang tertinggal adalah salinan yang tidak pernah dilihat siapa pun.
    attrs: { "data-ekspor": "masukan", "data-label": options.label },
    children: [
      el("span", {
        class: "field__label",
        children: [el("span", { text: options.label }), readout],
      }),
      input,
    ],
  });
}

/**
 * Kartu bersudut membulat dengan judul kecil di atasnya.
 *
 * Anak bernilai `null` dilewati, sehingga bagian yang muncul bersyarat bisa
 * ditulis sebagai ekspresi biasa tanpa menyusun larik terpisah lebih dulu.
 */
export function card(
  title: string | null,
  ...children: (Node | string | null | undefined)[]
): HTMLElement {
  return el("section", {
    class: "card",
    // Judulnya ikut ditandai supaya laporan ekspor mengikuti struktur
    // halaman. Tanpa itu, sebuah CSV yang memuat lima tabel tidak memberi
    // satu pun petunjuk tabel mana yang mana.
    attrs: title ? { "data-ekspor": "kartu", "data-judul": title } : {},
    children: [
      title ? el("h2", { class: "card__title", text: title }) : null,
      ...children,
    ],
  });
}

/** Angka besar dengan keterangan di bawahnya. */
export function readout(label: string, value: string): HTMLElement {
  return el("div", {
    attrs: { "data-ekspor": "hasil", "data-label": label, "data-nilai": value },
    children: [
      el("div", { class: "readout", text: value }),
      el("div", { class: "readout__label", text: label }),
    ],
  });
}

/** Bilah proporsi horizontal. */
export function bar(fraction: number, tone: "" | "warn" | "danger" = ""): HTMLElement {
  const fill = el("div", {
    class: tone ? `bar__fill bar__fill--${tone}` : "bar__fill",
  });
  fill.style.width = `${clamp(fraction, 0, 1) * 100}%`;
  return el("div", { class: "bar", children: [fill] });
}

/** Daftar langkah perhitungan bernomor. */
export function stepList(steps: { label: string; formula: string }[]): HTMLElement {
  return el("ol", {
    class: "steps",
    attrs: { "data-ekspor": "langkah" },
    children: steps.map((s) =>
      el("li", {
        attrs: { "data-label": s.label, "data-rumus": s.formula },
        children: [
          el("div", {
            children: [
              el("span", { class: "steps__label", text: `${s.label} ` }),
              el("span", { text: s.formula }),
            ],
          }),
        ],
      }),
    ),
  });
}

/**
 * Memformat sel angka pada tabel.
 *
 * Bilangan bulat ditulis apa adanya — nomor urut yang muncul sebagai `1.0000`
 * terbaca seperti cacat, bukan seperti data.
 */
function cellNumber(v: number): string {
  return Number.isInteger(v) ? String(v) : fmt(v);
}

/** Tabel sederhana dari kepala kolom dan baris. */
export function table(head: string[], rows: (string | number)[][]): HTMLElement {
  return el("div", {
    class: "scroll-x",
    children: [
      el("table", {
        children: [
          el("thead", {
            children: [
              el("tr", {
                children: head.map((h) => el("th", { text: h })),
              }),
            ],
          }),
          el("tbody", {
            children: rows.map((row) =>
              el("tr", {
                children: row.map((cell) =>
                  el("td", {
                    class: typeof cell === "number" ? "num" : "",
                    text: typeof cell === "number" ? cellNumber(cell) : cell,
                  }),
                ),
              }),
            ),
          }),
        ],
      }),
    ],
  });
}

/** Satu tombol pada sebuah baris tombol. */
export interface Tombol {
  label: string;
  /** Penekanan visual. Dipakai untuk aksi utama sebuah panel. */
  primary?: boolean;
  /**
   * Menyatakan bahwa tombol ini bagian dari sekelompok pilihan, dan apakah
   * ia yang sedang terpilih.
   *
   * Sengaja dipisah dari {@link primary}. Keduanya kebetulan tampak sama,
   * tetapi artinya berbeda: "Tambah bukti" ditonjolkan karena ia aksi utama,
   * bukan karena ia sedang terpilih. Menyamakannya membuat pembaca layar
   * mengumumkan tombol aksi sebagai tombol yang sedang aktif, dan membuat
   * laporan ekspor mencatat "dipilih: Tambah bukti" — kalimat yang tidak
   * berarti apa-apa bagi yang membacanya besok.
   */
  selected?: boolean;
  onClick: () => void;
}

/** Baris tombol; bisa berupa sekelompok pilihan atau sederet aksi. */
export function buttonRow(buttons: Tombol[]): HTMLElement {
  const kelompokPilihan = buttons.some((b) => b.selected !== undefined);
  return el("div", {
    class: "btn-row",
    // Hanya kelompok pilihan yang masuk laporan: yang terpilih adalah bagian
    // dari setelan yang menghasilkan angkanya, dan laporan tanpa itu tidak
    // bisa diulang siapa pun. Sederet aksi tidak punya "nilai" untuk dicatat.
    attrs: kelompokPilihan
      ? { "data-ekspor": "pilihan", role: "group" }
      : {},
    children: buttons.map((b) =>
      el("button", {
        class: (b.selected ?? b.primary) ? "btn btn--primary" : "btn",
        text: b.label,
        attrs: {
          type: "button",
          // Ditulis sebagai `aria-pressed`, bukan sekadar nama kelas: pembaca
          // layar perlu tahu tombol mana yang sedang aktif, dan pengekspor
          // membacanya dari sumber yang sama alih-alih menebak dari gaya.
          "aria-pressed": b.selected === undefined ? null : String(b.selected),
        },
        on: { click: b.onClick },
      }),
    ),
  });
}

/** Pesan galat yang bisa dibaca pengguna, sekaligus diumumkan pembaca layar. */
export function errorNote(message: string): HTMLElement {
  return el("p", {
    class: "error",
    text: message,
    attrs: { role: "alert" },
  });
}
