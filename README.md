<div align="center">

# AI ATLAS

**Laboratorium Kecerdasan Buatan Klasik**
*A Laboratory of Classical Artificial Intelligence*

Empat belas algoritma AI klasik, ditulis dari nol dengan **Rust**, dijalankan di peramban lewat **WebAssembly**, dan diverifikasi silang terhadap implementasi pembanding di **Go** dan **Oracle PL/SQL**.

[**→ Buka laboratoriumnya**](https://xyb3rpunq.github.io/ai-atlas/)

[![CI](https://github.com/xyb3rpunq/ai-atlas/actions/workflows/ci.yml/badge.svg)](https://github.com/xyb3rpunq/ai-atlas/actions/workflows/ci.yml)
[![Pages](https://img.shields.io/badge/GitHub%20Pages-live-4dd4c8)](https://xyb3rpunq.github.io/ai-atlas/)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-dea584)](https://www.rust-lang.org/)
[![WASM](https://img.shields.io/badge/WebAssembly-56%20KB%20gzip-654ff0)](https://webassembly.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

</div>

---

## Apa ini

Kebanyakan proyek "AI" hari ini adalah pemanggilan API milik orang lain. Proyek ini kebalikannya: **setiap algoritmanya ditulis sendiri, baris per baris**, mengikuti silabus mata kuliah *IND323 Artificial Intelligence* (Fakultas Ilmu Komputer, Universitas Esa Unggul).

Tidak ada `import tensorflow`. Tidak ada panggilan jaringan saat berjalan. Anda menggeser penggeser, dan yang menghitung adalah kode Rust yang dikompilasi ke WebAssembly dan berjalan di dalam tab Anda sendiri.

## Kenapa arsitekturnya begini

Rumus yang salah tidak akan membuat program *crash*. Ia hanya mengeluarkan angka keliru yang terlihat meyakinkan. Itu jenis kegagalan yang paling berbahaya dalam perangkat lunak numerik, dan satu-satunya pertahanan nyata adalah **menulis algoritma yang sama lebih dari sekali, secara independen, lalu mengadu hasilnya**.

```
                       ┌──────────────────────────────┐
                       │  crates/ai-core   (Rust)     │
                       │  sumber kebenaran            │
                       │  murni · tanpa I/O · teruji  │
                       └───────────────┬──────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              │                        │                        │
   ┌──────────▼─────────┐  ┌───────────▼──────────┐  ┌──────────▼──────────┐
   │  crates/ai-wasm    │  │  tools/conform (Go)  │  │  oracle/ (PL/SQL)   │
   │  wasm-bindgen      │  │  implementasi ke-2   │  │  implementasi ke-3  │
   │  → peramban        │  │  + harness pembanding│  │  + basis pengetahuan│
   └──────────┬─────────┘  └───────────┬──────────┘  └──────────┬──────────┘
              │                        │                        │
              │                        └───── bandingkan ───────┘
              │                          bit-eksak, ribuan kasus
              │                          selisih 1 ULP = build gagal
   ┌──────────▼─────────┐
   │  web/  TypeScript  │
   │  antarmuka + kanvas│
   └────────────────────┘
```

Tiga implementasi independen dari matematika yang sama. CI menjalankan ribuan kasus uji lewat ketiganya dan membandingkan **pola bit IEEE-754**, bukan desimal. Selisih satu ULP pun menggagalkan build.

### Catatan teknis: kenapa pola bit, bukan desimal

Saat membangun harness ini, pengukuran menemukan bahwa `serde_json::from_str::<f64>` **salah membulat sebesar 1 ULP pada 27.548 dari 200.000 nilai uji (13,8%)**, sementara `str::parse::<f64>` bawaan Rust nol kesalahan pada himpunan yang sama. Menulis `0.42000000000000004` lalu membacanya kembali bisa menghasilkan `0.42` — angka yang berbeda.

Karena itu seluruh vektor uji lintas bahasa memakai **16 digit heksadesimal pola bit**, bukan desimal. Lihat [`crates/ai-core/src/fx.rs`](crates/ai-core/src/fx.rs) dan uji `serde_json_bisa_meleset_satu_ulp` yang memagari temuan ini.

## Peta silabus

| Sesi | Topik | Yang bisa Anda mainkan | Status |
|:----:|-------|------------------------|:------:|
| 01 | Pengantar Kecerdasan Buatan | ELIZA (1966), uji Turing | ⏳ |
| 02 | Agen Cerdas & Ruang Keadaan | Dunia penyedot debu, empat jenis agen | ⏳ |
| 03 | Ketidakpastian | **Certainty Factor MYCIN**, MB/MD, kombinasi paralel & berantai | ✅ |
| 04 | Probabilitas Bayesian | **Teorema Bayes**, Naive Bayes, diagram 1.000 kasus | ✅ |
| 05 | Logika Fuzzy I | **Enam bentuk fungsi keanggotaan**, operasi Zadeh & produk, potongan alfa | ✅ |
| 06 | Logika Fuzzy II | **Mamdani, Sugeno, Tsukamoto** berdampingan, 5 metode defuzzifikasi | ✅ |
| 07 | Representasi Pengetahuan | Logika proposisi, resolusi, jaringan semantik, bingkai | ⏳ |
| 08 | Teknik Pencarian | **Sembilan algoritma** diadu di satu peta: BFS, DFS, DLS, IDDFS, UCS, Greedy, **A\***, hill climbing, annealing | ✅ |
| 09 | Jaringan Syaraf Tiruan | Perceptron, **backpropagation** yang diverifikasi gradien numerik, batas keputusan & kurva galat langsung | ✅ |
| 10 | Pemrosesan Bahasa Alami | Tokenisasi, stemming Bahasa Indonesia, TF-IDF | ⏳ |
| 11 | Sistem Pakar | **Forward & backward chaining**, fasilitas penjelasan | ⏳ |
| 12 | Sains Data & Big Data | Statistik, normalisasi, deteksi pencilan, matriks konfusi | ⏳ |
| 13 | Machine Learning | KNN, K-Means, pohon keputusan ID3, regresi | ⏳ |
| 14 | Robotika | Kinematika, kendali PID, medan potensial | ⏳ |

Sesi bertanda ⏳ sudah terpetakan di antarmuka tetapi mesinnya belum selesai. Statusnya ditampilkan apa adanya, bukan disembunyikan.

## Anggaran performa

Batas ini diperiksa otomatis di CI. Build gagal kalau terlampaui — bukan sekadar niat baik.

| Metrik | Anggaran | Terukur |
|--------|---------:|--------:|
| WebAssembly (gzip) | ≤ 400 KB | **107 KB** |
| JavaScript (gzip) | ≤ 60 KB | **16,4 KB** |
| CSS (gzip) | ≤ 20 KB | **2,8 KB** |
| Total muat pertama (gzip) | ≤ 460 KB | **132,0 KB** |
| Dependensi saat berjalan | 0 | **0** |

Tidak ada React, tidak ada kerangka kerja, tidak ada CDN. Seluruh antarmukanya hanya beberapa lusin simpul DOM, jadi membangunnya langsung lebih ringan daripada memuat pustaka mana pun.

## Menjalankan secara lokal

Prasyarat: [Rust](https://rustup.rs/) 1.75+, [Node.js](https://nodejs.org/) 20+, dan `wasm-pack`.

```bash
git clone https://github.com/xyb3rpunq/ai-atlas.git
cd ai-atlas

rustup target add wasm32-unknown-unknown
cargo install wasm-pack --locked
npm install

npm run wasm      # kompilasi Rust → WebAssembly
npm run dev       # buka http://localhost:5173
```

### Perintah yang tersedia

| Perintah | Fungsi |
|----------|--------|
| `npm run dev` | Peladen pengembangan dengan muat ulang panas |
| `npm run wasm` | Kompilasi ulang mesin Rust ke WebAssembly |
| `npm run build` | Build produksi ke `dist/` |
| `npm run test` | Uji sisi TypeScript |
| `npm run audit:all` | Periksa tipe + seluruh uji |
| `cargo test --workspace` | Seluruh uji Rust |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pemeriksaan gaya, peringatan dianggap galat |

## Cakupan pengujian

Setiap fungsi publik punya uji. Bukan uji jalur bahagia saja — uji nilai batas, masukan tak sah, sifat matematis (komutatif, monoton, jumlah probabilitas sama dengan satu), dan kasus dari lembar tugas asli.

| Berkas | Fungsi publik | Uji |
|--------|--------------:|----:|
| `certainty.rs` | 9 | 26 |
| `bayes.rs` | 21 | 36 |
| `fuzzy.rs` | 24 | 42 |
| `search.rs` | 22 | 44 |
| `neural.rs` | 30 | 40 |
| `fx.rs` | 8 | 17 |
| `rng.rs` | 9 | 16 |
| `lib.rs` | 2 | 4 |
| `ai-wasm/lib.rs` | 20 | 28 |
| `web/src/ui.ts` | 11 | 15 |
| **Total** | **156** | **283** |

Beberapa uji yang menahan seluruh proyek ini tetap jujur:

- **Pemeriksaan gradien** — perambatan balik dibandingkan dengan selisih hingga
  pada tiap bobot. Jaringan yang gradiennya salah tetap sering "belajar", hanya
  lebih lambat dan berhenti di tempat yang keliru; hanya uji inilah yang
  membedakannya dari yang benar.
- **Perbandingan optimalitas** — BFS, IDDFS, UCS, dan A\* harus menghasilkan
  biaya jalur yang identik pada peta yang sama.
- **Monotonisitas fuzzy** — menaikkan mutu masukan tidak boleh menurunkan
  keluaran. Uji inilah yang menemukan bug himpunan bahu.
- **Reproduktifitas** — benih yang sama harus menghasilkan bobot, labirin, dan
  jejak pencarian yang identik bit demi bit.

Contoh kasus yang dipakai sebagai uji berasal langsung dari lembar tugas mata kuliah:

- **Tugas Sesi 3** — MB[Cacar, Bintik] = 0,8 dan MD = 0,01 → CF = 0,79
- **Tugas Pertemuan 5** — 20% berita hoaks, 90% hoaks berjudul provokatif, 30% non-hoaks juga → P(hoaks \| provokatif) = 3/7 ≈ 42,86%

## Kesiapan produksi

| Aspek | Penerapan | Ditegakkan di |
|---|---|---|
| Kebijakan keamanan konten | Sidik SHA-256 dihitung otomatis untuk tiap skrip sebaris; `'unsafe-inline'` dan `'unsafe-eval'` ditolak | `scripts/csp.mjs` |
| Verifikasi hasil build | Berkas wajib, rujukan aset, jalur dasar, manifes, peta sumber | `scripts/verify-dist.mjs` |
| Anggaran ukuran | Build gagal bila terlampaui | `scripts/budget.mjs` |
| Audit kerentanan | `rustsec/audit-check` + `npm audit`, mingguan lewat cron | CI |
| Penggunaan luring | Pekerja layanan dengan strategi berbeda per jenis aset | `web/public/sw.js` |
| Penemuan mesin pencari | `sitemap.xml`, `robots.txt`, kanonik, data terstruktur `LearningResource` | `web/public/` |
| Pemasangan aplikasi | Manifes web + ikon biasa dan *maskable* | `web/public/` |
| Konsistensi akhiran baris | Dipaksa LF; CRLF merusak sidik kebijakan keamanan | `.gitattributes` |
| Tata kelola | `SECURITY.md`, `CONTRIBUTING.md`, `CHANGELOG.md`, `CODEOWNERS`, Dependabot | akar repositori |

Yang **tidak** dijanjikan, karena memang tidak bisa dipenuhi di GitHub Pages:
perlindungan penyematan lewat `frame-ancestors` memerlukan tajuk tanggapan HTTP
yang tidak bisa diatur di sana. Alasan mengapa risikonya dapat diterima ada di
[SECURITY.md](SECURITY.md) — bukan disembunyikan.

## Struktur direktori

```
ai-atlas/
├── crates/
│   ├── ai-core/        Algoritma murni. Tanpa I/O, tanpa WebAssembly.
│   │   └── src/
│   │       ├── certainty.rs   Sesi 3 — Certainty Factor
│   │       ├── bayes.rs       Sesi 4 — Bayesian
│   │       ├── fuzzy.rs       Sesi 5-6 — Logika Fuzzy
│   │       ├── search.rs      Sesi 8 — Teknik Pencarian
│   │       ├── neural.rs      Sesi 9 — Jaringan Syaraf Tiruan
│   │       ├── fx.rs          Pertukaran pecahan bit-eksak
│   │       └── rng.rs         SplitMix64 deterministik
│   └── ai-wasm/        Jembatan wasm-bindgen. Amplop JSON ok/err.
├── web/
│   ├── src/
│   │   ├── engine.ts   Pembungkus bertipe untuk WebAssembly
│   │   ├── labs/       Satu berkas per laboratorium
│   │   ├── i18n.ts     Dwibahasa ID/EN
│   │   └── ui.ts       Pembantu DOM, tanpa kerangka kerja
│   └── index.html
└── .github/workflows/  Uji, build, terbitkan
```

## Silabus sumber

Materi diambil dari modul resmi mata kuliah IND323, disusun oleh **Dr. Ir. Zulfiandri, M.Si.** dan diampu oleh **Ari Pambudi**, Universitas Esa Unggul. Dua rujukan primer yang dipakai:

- Zadeh, L. A. (2008). *Is there a need for fuzzy logic?* **Information Sciences**, 178(13), 2751–2779. [doi:10.1016/j.ins.2008.02.012](https://doi.org/10.1016/j.ins.2008.02.012)
- Taha, K. (2025). *Big Data Analytics in IoT, social media, NLP, and information security.* **Journal of Big Data**, 12(150). [doi:10.1186/s40537-025-01192-9](https://doi.org/10.1186/s40537-025-01192-9)
- Shortliffe, E. H., & Buchanan, B. G. (1975). *A model of inexact reasoning in medicine.* **Mathematical Biosciences**, 23(3–4), 351–379.

## Lisensi

Kode dilisensikan [MIT](LICENSE). Materi kuliah yang dirujuk tetap milik penyusunnya masing-masing.

---

<div align="center">

Dibangun oleh **[Daniel Hutajulu](https://github.com/xyb3rpunq)** — `.Deckyx`

[xyb3rpunq.github.io/whoami](https://xyb3rpunq.github.io/whoami/)

`.Deckyx` — inisial pencipta, tertanam di setiap repo, produk, dan proyek.

</div>
