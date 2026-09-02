/**
 * Titik masuk aplikasi.
 *
 * Tugasnya tiga: memuat mesin WebAssembly, menggambar kerangka halaman, dan
 * menyalakan laboratorium yang sesuai dengan alamat saat ini. Perutean memakai
 * tanda pagar agar GitHub Pages bisa melayani seluruh rute dari satu berkas
 * tanpa perlu konfigurasi peladen.
 */

import "./style.css";
import * as engine from "./engine.js";
import { T, lang, onLangChange, pick, restoreLang, setLang } from "./i18n.js";
import { LABS, SYLLABUS, findLab, type Lab } from "./labs/registry.js";
import { notesFor } from "./labs/notes.js";
import { LABEL as EKSPOR_LABEL, bacaIsi, csv, download, fileName, reportRows } from "./ekspor";
import { clear, el } from "./ui.js";

const THEME_KEY = "ai-atlas:theme";
const app = document.querySelector<HTMLElement>("#app");

/** Pembersih laboratorium yang sedang tampil. */
let disposeLab: (() => void) | null = null;

/**
 * Nomor urut pemuatan laboratorium.
 *
 * Bertambah tiap kali sebuah laboratorium mulai dimuat. Pemuatan yang selesai
 * dengan nomor lama berarti penggunanya sudah berpindah halaman, dan hasilnya
 * harus dibuang alih-alih dipasang ke halaman yang sudah berganti.
 */
let muatKe = 0;

/** Tema yang sedang aktif. */
function theme(): "dark" | "light" {
  return document.documentElement.dataset.theme === "light" ? "light" : "dark";
}

/** Mengganti tema dan menyimpannya. */
function setTheme(next: "dark" | "light"): void {
  document.documentElement.dataset.theme = next;
  try {
    localStorage.setItem(THEME_KEY, next);
  } catch {
    /* Preferensi tidak tersimpan, tampilan sesi ini tetap berganti. */
  }
  render();
}

/** Slug yang diminta alamat saat ini. */
function currentSlug(): string {
  return globalThis.location.hash.replace(/^#\/?/, "").trim();
}

/** Lambang merek dalam bentuk SVG sebaris. */
function brandMark(): SVGSVGElement {
  const ns = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(ns, "svg");
  svg.setAttribute("viewBox", "0 0 32 32");
  svg.setAttribute("class", "brand__mark");
  svg.setAttribute("aria-hidden", "true");
  const nodes: [number, number][] = [
    [16, 8],
    [8, 23],
    [24, 23],
  ];
  const edges = document.createElementNS(ns, "path");
  edges.setAttribute("d", "M16 8 8 23M16 8l8 15M8 23h16");
  edges.setAttribute("stroke", "currentColor");
  edges.setAttribute("stroke-width", "1.4");
  edges.setAttribute("fill", "none");
  edges.setAttribute("opacity", "0.5");
  svg.append(edges);
  for (const [cx, cy] of nodes) {
    const c = document.createElementNS(ns, "circle");
    c.setAttribute("cx", String(cx));
    c.setAttribute("cy", String(cy));
    c.setAttribute("r", "3.2");
    c.setAttribute("fill", "currentColor");
    svg.append(c);
  }
  svg.style.color = "var(--accent)";
  return svg;
}

/** Bilah samping berisi navigasi silabus. */
function sidebar(active: string): HTMLElement {
  const items = SYLLABUS.map((entry) => {
    const ready = Boolean(entry.slug);
    const node = el("a", {
      class: ready ? "nav__item" : "nav__item nav__item--soon",
      attrs: {
        href: ready ? `#/${entry.slug}` : "#/",
        "aria-current": ready && entry.slug === active ? "page" : null,
        "aria-disabled": ready ? null : "true",
        tabindex: ready ? null : "-1",
      },
      children: [
        el("span", {
          class: "nav__num",
          text: String(entry.session).padStart(2, "0"),
        }),
        el("span", {
          text: ready
            ? pick(entry.title)
            : `${pick(entry.title)} · ${pick(T.soon)}`,
        }),
      ],
    });
    return node;
  });

  return el("aside", {
    class: "sidebar",
    children: [
      el("a", {
        class: "brand",
        attrs: { href: "#/", "aria-label": "AI ATLAS" },
        children: [
          brandMark(),
          el("span", {
            children: [
              el("div", { class: "brand__name", text: "AI ATLAS" }),
              el("div", { class: "brand__sub", text: "IND323 · Rust + WebAssembly" }),
            ],
          }),
        ],
      }),
      el("nav", {
        class: "nav",
        attrs: { "aria-label": pick(T.labs) },
        children: [
          el("div", { class: "nav__heading", text: pick(T.labs) }),
          ...items,
        ],
      }),
    ],
  });
}

/** Bilah atas berisi pengalih bahasa dan tema. */
function topbar(): HTMLElement {
  const langButtons = (["id", "en"] as const).map((code) =>
    el("button", {
      class: "pill",
      text: code === "id" ? "Bahasa Indonesia" : "English",
      attrs: { type: "button", "aria-pressed": lang() === code },
      on: { click: () => setLang(code) },
    }),
  );

  const themeButton = el("button", {
    class: "pill",
    text: theme() === "dark" ? "☾" : "☀",
    attrs: {
      type: "button",
      "aria-label": pick(T.theme),
      title: pick(T.theme),
    },
    on: { click: () => setTheme(theme() === "dark" ? "light" : "dark") },
  });

  return el("div", {
    class: "topbar",
    children: [
      el("div", { class: "toggles", children: langButtons }),
      el("div", { class: "toggles", children: [themeButton] }),
    ],
  });
}

/** Halaman depan: peta silabus. */
function homePage(): HTMLElement {
  return el("div", {
    children: [
      el("header", {
        class: "lab-head",
        children: [
          el("div", { class: "lab-head__eyebrow", text: "IND323 · UNIVERSITAS ESA UNGGUL" }),
          el("h1", { text: pick(T.tagline) }),
          el("p", { text: pick(T.subtitle) }),
        ],
      }),
      el("section", {
        class: "card",
        children: [
          el("h2", { class: "card__title", text: pick(T.labs) }),
          el("div", {
            class: "grid-2",
            children: SYLLABUS.map((entry) =>
              el("a", {
                class: entry.slug ? "nav__item" : "nav__item nav__item--soon",
                attrs: {
                  href: entry.slug ? `#/${entry.slug}` : "#/",
                  "aria-disabled": entry.slug ? null : "true",
                  tabindex: entry.slug ? null : "-1",
                },
                children: [
                  el("span", {
                    class: "nav__num",
                    text: String(entry.session).padStart(2, "0"),
                  }),
                  el("span", {
                    text: entry.slug
                      ? pick(entry.title)
                      : `${pick(entry.title)} · ${pick(T.soon)}`,
                  }),
                ],
              }),
            ),
          }),
        ],
      }),
    ],
  });
}

/**
 * Bagian catatan dan definisi, digambar seragam untuk seluruh laboratorium.
 *
 * Dirender terpusat di sini, bukan di tiap berkas laboratorium, supaya
 * susunannya tidak pelan-pelan menyimpang satu sama lain seiring waktu.
 */
function notesSection(slug: string): HTMLElement | null {
  const notes = notesFor(slug);
  if (!notes) return null;

  const bagian: HTMLElement[] = [
    el("p", { class: "note note--lead", text: pick(notes.summary) }),
  ];

  if (notes.definitions.length > 0) {
    bagian.push(
      el("h3", { class: "notes__heading", text: pick(T.definitions) }),
      el("dl", {
        class: "defs",
        children: notes.definitions.flatMap((d) => [
          el("dt", { text: d.term }),
          el("dd", { text: pick(d.meaning) }),
        ]),
      }),
    );
  }

  if (notes.formulas.length > 0) {
    bagian.push(
      el("h3", { class: "notes__heading", text: pick(T.formulas) }),
      el("div", {
        class: "formulas",
        children: notes.formulas.map((f) =>
          el("div", {
            class: "formula",
            children: [
              el("div", { class: "formula__name", text: f.name }),
              el("pre", { class: "formula__body", text: f.expression }),
              el("p", { class: "formula__note", text: pick(f.note) }),
            ],
          }),
        ),
      }),
    );
  }

  if (notes.pitfalls.length > 0) {
    bagian.push(
      el("h3", { class: "notes__heading", text: pick(T.pitfalls) }),
      el("ul", {
        class: "pitfalls",
        children: notes.pitfalls.map((p) => el("li", { text: pick(p) })),
      }),
    );
  }

  if (notes.references.length > 0) {
    bagian.push(
      el("h3", { class: "notes__heading", text: pick(T.references) }),
      el("ul", {
        class: "refs",
        children: notes.references.map((r) =>
          el("li", {
            children: [
              r.url
                ? el("a", {
                    attrs: { href: r.url, rel: "noopener", target: "_blank" },
                    text: r.text,
                  })
                : el("span", { text: r.text }),
            ],
          }),
        ),
      }),
    );
  }

  return el("section", {
    class: "card notes",
    children: [
      el("h2", { class: "card__title", text: pick(T.notes) }),
      ...bagian,
    ],
  });
}

/**
 * Halaman satu laboratorium.
 *
 * Judul, penjelasan, dan catatannya digambar seketika dari katalog; mesinnya
 * menyusul lewat `import()` yang berkas terpisah. Urutan ini disengaja:
 * pengunjung langsung melihat halaman yang benar dan bisa mulai membaca,
 * alih-alih menatap layar kosong sampai seluruh kodenya turun.
 *
 * Pemasangannya diperiksa dua kali terhadap `disposeLab`: pengguna bisa saja
 * berpindah halaman sementara modulnya masih diunduh, dan memasang
 * laboratorium ke dalam elemen yang sudah dibuang akan meninggalkan gelang
 * animasi yang terus berjalan tanpa ada yang bisa menghentikannya.
 */
function labPage(lab: Lab): HTMLElement {
  const body = el("div", {
    children: [
      el("p", { class: "note", text: pick(T.loading) }),
    ],
  });
  const page = el("div", {
    children: [
      el("header", {
        class: "lab-head",
        children: [
          el("div", {
            class: "lab-head__eyebrow",
            text: `${pick(T.syllabus)} ${String(lab.session).padStart(2, "0")}`,
          }),
          el("h1", { text: pick(lab.title) }),
          el("p", { text: pick(lab.blurb) }),
        ],
      }),
      body,
      exportSection(lab, body),
      notesSection(lab.slug),
    ],
  });

  const tokenSaatIni = ++muatKe;
  void lab
    .load()
    .then((modul) => {
      if (tokenSaatIni !== muatKe) return;
      clear(body);
      disposeLab = modul.mount(body);
    })
    .catch((error: unknown) => {
      if (tokenSaatIni !== muatKe) return;
      clear(body);
      body.append(
        el("p", {
          class: "error",
          attrs: { role: "alert" },
          text: `${pick(T.loadFailed)}: ${(error as Error).message}`,
        }),
      );
    });

  return page;
}

/**
 * Bagian ekspor di ujung tiap lab.
 *
 * Isinya dibaca dari `body` yang sudah tergambar, bukan dari keadaan lab.
 * Keempat belas lab tidak punya bentuk hasil bersama, dan membaca tampilannya
 * membuat satu kode ini berlaku untuk semuanya — termasuk lab yang belum ada.
 */
function exportSection(lab: Lab, body: HTMLElement): HTMLElement {
  const kabar = el("div", { class: "ekspor__kabar", attrs: { "aria-live": "polite" } });

  const tombolUnduh = el("button", {
    class: "btn",
    attrs: { type: "button" },
    text: pick(EKSPOR_LABEL.download),
    on: {
      click: () => {
        clear(kabar);
        try {
          const nama = fileName(lab.slug);
          const isi = bacaIsi(body);
          download(nama, csv(reportRows(pick(lab.title), lab.session, isi)), "text/csv;charset=utf-8");
          kabar.append(
            el("span", {
              class: "kabar kabar--ok",
              text: `${pick(EKSPOR_LABEL.downloaded)}: ${nama}`,
            }),
          );
        } catch (error: unknown) {
          // Diam bukan pilihan: pengguna yang menekan tombol dan tidak melihat
          // apa pun akan menekannya lagi, lalu menyimpulkan situsnya rusak.
          kabar.append(
            el("span", {
              class: "kabar",
              text: `${pick(EKSPOR_LABEL.refused)}: ${(error as Error).message}`,
            }),
          );
        }
      },
    },
  });

  const tombolCetak = el("button", {
    class: "btn",
    attrs: { type: "button" },
    text: pick(EKSPOR_LABEL.print),
    on: { click: () => globalThis.print() },
  });

  return el("section", {
    class: "ekspor",
    children: [
      el("p", { class: "note", text: pick(EKSPOR_LABEL.hint) }),
      el("div", { class: "btn-row", children: [tombolUnduh, tombolCetak] }),
      kabar,
    ],
  });
}

/** Halaman untuk alamat yang tidak dikenal. */
function notFoundPage(): HTMLElement {
  return el("div", {
    class: "lab-head",
    children: [
      el("h1", { text: pick(T.notFound) }),
      el("p", {
        children: [el("a", { attrs: { href: "#/" }, text: pick(T.backHome) })],
      }),
    ],
  });
}

/** Kaki halaman. */
function footer(): HTMLElement {
  let engineVersion = "—";
  try {
    engineVersion = engine.version();
  } catch {
    /* Mesin belum siap; versi ditampilkan sebagai tanda hubung. */
  }
  return el("footer", {
    class: "footer",
    children: [
      el("span", {
        children: [
          el("span", { text: `${pick(T.builtWith)} ` }),
          el("span", { class: "tag", text: ".Deckyx" }),
        ],
      }),
      el("span", {
        children: [
          el("span", { class: "tag", text: `${pick(T.engineVersion)} v${engineVersion}` }),
          document.createTextNode("  "),
          el("a", {
            attrs: {
              href: "https://github.com/xyb3rpunq/ai-atlas",
              rel: "noopener",
            },
            text: pick(T.sourceCode),
          }),
        ],
      }),
    ],
  });
}

/** Menggambar ulang seluruh halaman. */
function render(): void {
  if (!app) return;
  disposeLab?.();
  disposeLab = null;
  // Pemuatan yang masih berjalan ditandai basi sebelum halaman diganti.
  muatKe += 1;

  const slug = currentSlug();
  const lab = slug ? findLab(slug) : undefined;
  const content = slug === "" ? homePage() : lab ? labPage(lab) : notFoundPage();

  clear(app);
  app.removeAttribute("aria-busy");
  app.append(
    el("div", {
      class: "shell",
      children: [
        sidebar(slug),
        el("main", {
          class: "main",
          attrs: { id: "lab" },
          children: [topbar(), content, footer()],
        }),
      ],
    }),
  );
}

/** Layar sementara selama modul WebAssembly diunduh. */
function bootScreen(message: string, failed = false): void {
  if (!app) return;
  clear(app);
  app.append(
    el("div", {
      class: "boot",
      children: [
        el("p", { text: message }),
        failed
          ? el("button", {
              class: "btn btn--primary",
              text: pick(T.reload),
              attrs: { type: "button" },
              on: { click: () => globalThis.location.reload() },
            })
          : el("div", { class: "boot__bar", children: [el("span")] }),
      ],
    }),
  );
}

async function start(): Promise<void> {
  restoreLang();
  bootScreen(pick(T.loading));
  try {
    await engine.load();
  } catch (error) {
    bootScreen(`${pick(T.loadFailed)}: ${(error as Error).message}`, true);
    return;
  }
  globalThis.addEventListener("hashchange", render);
  onLangChange(render);
  render();
  prefetchLabs();
}

/**
 * Mengunduh modul laboratorium yang belum dipakai, setelah halaman pertama
 * selesai digambar.
 *
 * Pemecahan kode memperbaiki pemuatan pertama tetapi merusak dua hal lain:
 * berpindah laboratorium jadi menunggu unduhan, dan laboratorium yang belum
 * pernah dibuka tidak ada di singgahan sehingga hilang saat luring. Pengambilan
 * di belakang layar ini mengembalikan keduanya tanpa mengorbankan pemuatan
 * pertamanya.
 *
 * Dilewati sepenuhnya pada sambungan lambat atau saat penghemat data menyala:
 * pengguna yang menyalakan penghemat data sedang menyatakan sesuatu, dan
 * mengunduh sebelas modul yang mungkin tidak pernah dibuka adalah persis yang
 * ia minta jangan dilakukan.
 */
function prefetchLabs(): void {
  const koneksi = (
    navigator as Navigator & {
      connection?: { saveData?: boolean; effectiveType?: string };
    }
  ).connection;
  if (koneksi?.saveData === true) return;
  if (koneksi?.effectiveType && /2g/.test(koneksi.effectiveType)) return;

  const antre = LABS.filter((l) => l.slug !== currentSlug());
  const jalan = (): void => {
    const berikut = antre.shift();
    if (!berikut) return;
    // Berurutan, bukan serentak: sebelas permintaan sekaligus akan bersaing
    // dengan modul WebAssembly dan gambar yang mungkin masih dalam perjalanan.
    void berikut
      .load()
      .catch(() => {
        /* Gagal mengambil di muka bukan galat; modulnya diambil lagi saat dibuka. */
      })
      .finally(jalan);
  };

  const mulai = (): void => jalan();
  if ("requestIdleCallback" in globalThis) {
    (globalThis as unknown as { requestIdleCallback: (cb: () => void) => void })
      .requestIdleCallback(mulai);
  } else {
    globalThis.setTimeout(mulai, 1500);
  }
}

/**
 * Mendaftarkan pekerja layanan supaya laboratorium tetap bisa dibuka tanpa
 * jaringan. Kegagalan pendaftaran sengaja ditelan: situsnya tetap berfungsi
 * penuh saat daring, jadi ini peningkatan, bukan syarat.
 */
function registerServiceWorker(): void {
  if (!("serviceWorker" in navigator)) return;
  globalThis.addEventListener("load", () => {
    navigator.serviceWorker.register(`${import.meta.env.BASE_URL}sw.js`).catch(() => {
      /* Luring tidak tersedia; tidak ada yang perlu dilaporkan ke pengguna. */
    });
  });
}

registerServiceWorker();

void start();

/** Diekspor untuk keperluan pengujian rute. */
export { currentSlug, LABS };
