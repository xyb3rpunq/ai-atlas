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

[🇮🇩 Bahasa Indonesia](#-bahasa-indonesia) · [🇬🇧 English](#-english)

</div>

---

## 🇮🇩 Bahasa Indonesia

### Ini apa, sih?

Kalau kamu punya kalkulator, kamu percaya angkanya karena orang lain sudah
memeriksanya berkali-kali. Tapi bagaimana kalau kalkulatornya baru, dan kamu
sendiri yang membuatnya?

Cara paling meyakinkan: **kerjakan soal yang sama dengan tiga cara berbeda,
lalu bandingkan.** Kalau ketiganya sepakat sampai angka paling belakang,
kemungkinan besar ketiganya benar. Kalau satu berbeda, kamu tahu ada yang salah
— dan tahu di mana harus mencari.

Itu yang dilakukan proyek ini. Empat belas algoritma kecerdasan buatan, masing-
masing ditulis **tiga kali dalam tiga bahasa pemrograman berbeda** (Rust, Go,
dan Oracle PL/SQL), lalu diadu satu sama lain secara otomatis. Kalau selisihnya
sekecil apa pun, buildnya gagal dan situsnya tidak terbit.

Kamu bisa geser penggesernya sendiri di situs itu. Yang menghitung adalah kode
Rust yang berjalan di dalam tab kamu — bukan server siapa pun.

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
   │  wasm-bindgen      │  │  implementasi ke-2 ✅│  │  implementasi ke-3 ✅│
   │  → peramban        │  │  + harness pembanding│  │  + basis pengetahuan│
   └──────────┬─────────┘  └───────────┬──────────┘  └──────────┬──────────┘
              │                        │                        │
              │                        └───── bandingkan ───────┘
              │                          2.266 vektor → 3.796 pernyataan
              │                          selisih 1 ULP = build gagal
   ┌──────────▼─────────┐
   │  web/  TypeScript  │
   │  antarmuka + kanvas│
   └────────────────────┘
```

Tiga implementasi independen dari matematika yang sama. CI menjalankan ribuan kasus uji lewat ketiganya dan membandingkan **pola bit IEEE-754**, bukan desimal. Selisih satu ULP pun menggagalkan build.

### Status konformansi

Setiap berkas vektor memuat beberapa keluaran per baris — satu baris Bayes memuat P(E), posterior, dan rasio kemungkinan sekaligus. Pemuat Oracle memecahnya menjadi **satu baris tabel per keluaran**, sehingga laporan ketidakcocokan menunjuk ke satu perhitungan tertentu dan bukan ke sekumpulan perhitungan yang kebetulan ditulis di baris yang sama.

| Berkas vektor | Baris | Tingkat | Rust ⟷ Go | Rust ⟷ PL/SQL |
|---|---:|---|:---:|:---:|
| `bayes.tsv` | 729 | BitExact | ✅ | ✅ 2.187 |
| `certainty.tsv` | 680 | BitExact | ✅ | ✅ 680 |
| `fuzzy_linear.tsv` | 520 | BitExact | ✅ | ✅ 520 |
| `fuzzy_transcendental.tsv` | 222 | NearlyEqual(4) | ✅ | ✅ 222 |
| `rng.tsv` | 72 | BitExact | ✅ | ✅ 144 |
| `ml_exact.tsv` | 18 | BitExact | ✅ | ✅ 18 |
| `fx.tsv` | 14 | BitExact | ✅ | ✅ 14 |
| `ml_entropy.tsv` | 7 | NearlyEqual(4) | ✅ | ✅ 7 |
| `ml_gain.tsv` | 4 | CancellingDifference(4) | ✅ | ✅ 4 |
| **Total** | **2.266** | | **cocok** | **3.796 cocok** |

Terukur pada jalan terakhir: **3.752 cocok bit demi bit, 44 berbeda hanya pada tanda nol, 0 gagal.** Penyimpangan terjauh 2 ULP — separuh dari batas yang dipasang.

Harness-nya sendiri punya uji: sebuah vektor yang sengaja dirusak sebesar satu ULP harus tertangkap, dilaporkan di baris yang benar, dengan kedua pola bitnya. Harness yang selalu lolos tidak berguna.

CI juga memeriksa bahwa vektor yang tersimpan masih sepadan dengan keluaran Rust. Kalau berbeda, berarti perilaku numerik berubah tanpa ada yang menyadarinya — dan perubahan seperti itu wajib disengaja.

### Catatan teknis: Oracle tidak punya nol negatif

Pengukuran terhadap Oracle Free 23ai menemukan bahwa `BINARY_DOUBLE` **mengubah `-0` menjadi `+0`**, bahkan pada konversi langsung dari pola bitnya:

```sql
UTL_RAW.CAST_TO_BINARY_DOUBLE(HEXTORAW('8000000000000000'))  -->  0000000000000000
```

Tiga tempat di vektor uji menghasilkan nol negatif — `entropy(['A'])`, `cf_sequential(-1, -1)`, dan nilai batas `fx.tsv` — dan seluruhnya mustahil direproduksi di Oracle. Jawabannya bukan melonggarkan perbandingan, melainkan **memberi status tersendiri yang bisa dihitung**: putusan `Z` hanya diberikan bila kedua nilainya benar-benar nol, sehingga selisih apa pun selain tanda nol tetap dihitung gagal. Jumlahnya dilaporkan tiap jalan (44), sehingga tidak bisa diam-diam bertambah.

Perilaku Oracle itu sendiri dikunci uji, supaya kelonggarannya bisa dicabut kalau suatu hari Oracle berubah.

### Catatan teknis: kenapa pola bit, bukan desimal

Saat membangun harness ini, pengukuran menemukan bahwa `serde_json::from_str::<f64>` **salah membulat sebesar 1 ULP pada 27.548 dari 200.000 nilai uji (13,8%)**, sementara `str::parse::<f64>` bawaan Rust nol kesalahan pada himpunan yang sama. Menulis `0.42000000000000004` lalu membacanya kembali bisa menghasilkan `0.42` — angka yang berbeda.

Karena itu seluruh vektor uji lintas bahasa memakai **16 digit heksadesimal pola bit**, bukan desimal. Lihat [`crates/ai-core/src/fx.rs`](crates/ai-core/src/fx.rs) dan uji `serde_json_bisa_meleset_satu_ulp` yang memagari temuan ini.

### Catatan teknis: apa yang sebenarnya bisa dibandingkan

Temuan kedua, dari CI: sebuah uji jaringan syaraf **lolos di Windows dan gagal di Linux**. Bukan flake, dan bukan bug di kodenya. IEEE-754 hanya mewajibkan enam operasi dibulatkan dengan benar — `+`, `−`, `×`, `÷`, `√`, dan perbandingan. Fungsi transendental seperti `exp`, `ln`, `tanh`, `sin`, dan `pow` **tidak diwajibkan**, sehingga pustaka matematika yang berbeda boleh menghasilkan nilai berbeda satu ULP untuk masukan yang sama. Pada pelatihan berlangkah besar yang sudah berayun, selisih sekecil itu membesar menjadi hasil akhir yang sama sekali berbeda.

Menuntut kesamaan bit pada perhitungan seperti itu akan menghasilkan uji yang gagal berselang-seling tanpa ada yang benar-benar salah. Karena itu tiap perhitungan digolongkan lebih dulu:

| Tingkat | Berlaku untuk | Yang dituntut |
|---|---|---|
| `BitExact` | Hanya `+ − × ÷ √` dan perbandingan | Identik bit demi bit |
| `NearlyEqual(n)` | Menyentuh `exp`, `ln`, `tanh`, `pow` | Selisih maksimal `n` ULP (bawaan 4) |
| `CancellingDifference(n)` | Hasil yang berupa selisih dua besaran hampir sama | Selisih maksimal `n` ULP **diukur pada skala masukannya** |
| `PropertyOnly` | Perhitungan kacau, mis. pelatihan divergen | Hanya sifatnya, mis. "yang wajar lebih baik daripada yang ekstrem" |

Tingkat keempat lahir dari kegagalan sungguhan. Perolehan informasi adalah `H(sebelum) − H(sesudah)`; pada dataset tenis nilainya 0,94 dikurangi 0,91. Galat dua ULP pada `H` — wajar, karena `log2` bukan operasi yang dibulatkan dengan benar menurut IEEE-754 — bernilai mutlak sekitar 2,2×10⁻¹⁶. Pada hasil sebesar 0,029, nilai itu sama dengan **64 ULP**. Menuntut `NearlyEqual(4)` di sana berarti menuntut implementasi `log2` yang lebih teliti daripada yang diwajibkan standar mana pun.

Yang benar adalah menyatakan toleransinya di tempat aritmetikanya sungguh-sungguh terjadi: `|a − b| ≤ n × ulp(skala)`, dengan skala disertakan berkas vektornya sebagai kolom `scale_hex`. Berkas yang menyatakan tingkat ini tanpa kolom itu ditolak harness — dan ditolak juga oleh `CHECK` di tabel Oracle.

Penggolongan ini menentukan bentuk harness Go dan PL/SQL. Tanpa itu, seluruh perbandingan tiga arah akan dibangun di atas asumsi yang salah.

## Visualisasi di setiap modul

Setiap laboratorium menampilkan besarannya sebagai gambar, bukan hanya tabel. Yang menentukan bentuk gambarnya adalah pertanyaan yang ingin dijawab, bukan variasi:

| Bentuk | Menjawab | Dipakai di |
|---|---|---|
| Garis bilangan berpita | "Angka ini letaknya di mana, dan apa artinya di sana" | Certainty factor, Bayesian |
| Air terjun | "Bukti mana yang paling menggeser kesimpulan" | Certainty factor |
| Graf berlapis | "Apa menyimpulkan apa, lewat aturan mana" | Sistem pakar, resolusi, jaringan semantik |
| Peta panas | "Nilai mana yang menonjol di dalam matriks" | TF-IDF, kemiripan dokumen, matriks konfusi, tabel kebenaran |
| Alur bertahap | "Benda yang sama berubah menjadi apa di tiap tahap" | Stemming, pipeline NLP, ELIZA |
| Bilah terurut | "Siapa menang, dan seberapa unggul" | Perolehan informasi ID3, biaya agen, keutamaan aturan, IDF |
| Siklus tertutup | "Kenapa ini gelang, bukan garis" | Agen cerdas |
| Kanvas | Piksel dan animasi | Peta pencarian, pelatihan jaringan, kurva kabur, robot |

**Gambar struktural memakai SVG, bukan kanvas.** Bukan selera: kanvas tidak terlihat oleh pembaca layar, harus digambar ulang tiap kali tema berganti, dan harus mengurus `devicePixelRatio` sendiri. SVG mewarisi warna lewat variabel CSS, sehingga pergantian terang-gelap tidak memerlukan satu baris kode pun. Kanvas tetap dipakai di tempat yang memang membutuhkannya — peta penelusuran yang menggambar ribuan petak tiap bingkai.

**Setiap gambar wajib punya padanan teks.** Fungsi `figure()` menuntut argumen `summary`, dan keterangannya menjadi `aria-label` gambar sekaligus tulisan di bawahnya. Keterangannya bukan pengulangan judul: ia menyebutkan apa yang sedang dilihat dan apa artinya, sebab pembaca yang paling butuh gambar adalah yang belum memahami topiknya.

## Data Anda sendiri, hasilnya bisa dibawa pulang

Tiap laboratorium menerima data yang Anda ketik sendiri — bukan hanya contoh bawaan. Isi bukti dan bobotnya di Certainty Factor, tempel kalimat Anda di NLP, tulis rumus Anda di representasi pengetahuan, gambar dinding Anda di peta pencarian. Seluruhnya dihitung di peramban Anda; tidak ada satu bita pun yang dikirim ke mana pun.

Di ujung tiap laboratorium ada dua tombol:

| Tombol | Menghasilkan | Untuk |
|---|---|---|
| **Unduh CSV (Excel)** | Satu berkas berisi setelan, hasil, langkah perhitungan, dan seluruh tabel di halaman | Diolah lagi di Excel, Google Sheets, atau pandas |
| **Cetak / simpan PDF** | Halaman yang sama tanpa kemudi, satu kolom, tabel utuh | Dilampirkan ke laporan tugas |

Laporannya mengikuti struktur halaman: tiap kartu menjadi satu bagian bernama, sehingga sebuah berkas berisi lima tabel tetap bisa dibaca ulang besok. Pilihan yang sedang aktif — algoritma mana, heuristik mana, mesin inferensi mana — ikut tercatat, karena angka tanpa setelan yang menghasilkannya tidak bisa diulang siapa pun.

**Kenapa CSV dan bukan XLSX.** XLSX adalah arsip ZIP berisi beberapa berkas XML; menyusunnya di peramban menuntut pustaka ratusan kilobyte — beberapa kali lipat modul WebAssembly seluruh proyek ini. CSV dibuka Excel, Sheets, dan pandas tanpa satu bita pun tambahan. Tiga hal yang membuatnya sungguh terbuka rapi, dan ketiganya sering terlewat: BOM UTF-8 di depan, baris `sep=,` paling atas (Excel berwilayah Indonesia memakai titik koma), dan akhir baris CRLF sesuai RFC 4180.

**Kenapa isinya dibaca dari tampilan.** Dua belas laboratorium tidak punya satu bentuk hasil bersama. Menambahkan ekspor satu per satu berarti dua belas potong kode yang akan menyimpang — yang satu lupa langkah, yang lain lupa masukan, dan tidak ada yang menyadarinya sampai seseorang mengunduh lab yang jarang dibuka. Karena itu isinya dipungut dari atribut `data-ekspor` yang dipasang komponen bersama di [`web/src/ui.ts`](web/src/ui.ts). Sebuah laboratorium yang memakai komponen bersama otomatis bisa diekspor tanpa satu baris pun tambahan. Lihat [`web/src/ekspor.ts`](web/src/ekspor.ts) dan [`tests/ekspor.test.ts`](tests/ekspor.test.ts) — ujinya dibangun dari komponen sungguhan, bukan HTML tiruan, supaya kontraknya yang putus terdeteksi, bukan hanya tiruannya.

## Peta silabus

| Sesi | Topik | Yang bisa Anda mainkan | Status |
|:----:|-------|------------------------|:------:|
| 01 | Pengantar Kecerdasan Buatan | **ELIZA (1966)** dengan mesinnya dibiarkan terbuka: aturan mana yang menang dan mengapa | ✅ |
| 02 | Agen Cerdas & Ruang Keadaan | **Empat jenis agen** diadu di satu dunia, teko air, misionaris & kanibal | ✅ |
| 03 | Ketidakpastian | **Certainty Factor MYCIN**, MB/MD, kombinasi paralel & berantai | ✅ |
| 04 | Probabilitas Bayesian | **Teorema Bayes**, Naive Bayes, diagram 1.000 kasus | ✅ |
| 05 | Logika Fuzzy I | **Enam bentuk fungsi keanggotaan**, operasi Zadeh & produk, potongan alfa | ✅ |
| 06 | Logika Fuzzy II | **Mamdani, Sugeno, Tsukamoto** berdampingan, 5 metode defuzzifikasi | ✅ |
| 07 | Representasi Pengetahuan | **Pengurai rumus**, tabel kebenaran, CNF, **pembuktian resolusi**, jaringan semantik, bingkai | ✅ |
| 08 | Teknik Pencarian | **Sembilan algoritma** diadu di satu peta: BFS, DFS, DLS, IDDFS, UCS, Greedy, **A\***, hill climbing, annealing | ✅ |
| 09 | Jaringan Syaraf Tiruan | Perceptron, **backpropagation** yang diverifikasi gradien numerik, batas keputusan & kurva galat langsung | ✅ |
| 10 | Pemrosesan Bahasa Alami | **Stemmer Nazief-Adriani** Bahasa Indonesia, TF-IDF, kemiripan kosinus, jarak sunting, sentimen berpengingkaran | ✅ |
| 11 | Sistem Pakar | **Runut maju & mundur** pada basis pengetahuan yang sama, pemangkasan pertanyaan, fasilitas penjelasan | ✅ |
| 12 | Sains Data & Big Data | Penskalaan, matriks konfusi, presisi/kepekaan/F1, **ketepatan pembanding** | ✅ |
| 13 | Machine Learning | **KNN** dengan wilayah keputusan, **K-Means++**, **pohon ID3** dengan entropi & perolehan informasi, regresi | ✅ |
| 14 | Robotika | Penggerak diferensial, **kendali PID** beserta ayunannya, kinematika balik, **minimum lokal** medan potensial | ✅ |

Seluruh empat belas sesi sudah terimplementasi dan bisa dijalankan. Tidak ada bagian yang berstatus "segera".

## Anggaran performa

Batas ini diperiksa otomatis di CI. Build gagal kalau terlampaui — bukan sekadar niat baik.

| Metrik | Anggaran | Terukur |
|--------|---------:|--------:|
| WebAssembly (gzip) | ≤ 400 KB | **229,1 KB** |
| JavaScript inti (gzip) | ≤ 60 KB | **33,9 KB** |
| Satu laboratorium (gzip) | ≤ 60 KB | **2,4 – 5,7 KB** |
| CSS (gzip) | ≤ 20 KB | **3,3 KB** |
| Total seluruh berkas (gzip) | ≤ 460 KB | **319,9 KB** |
| Dependensi saat berjalan | 0 | **0** |

Tidak ada React, tidak ada kerangka kerja, tidak ada CDN. Seluruh antarmukanya hanya beberapa lusin simpul DOM, jadi membangunnya langsung lebih ringan daripada memuat pustaka mana pun.

### Kode dipecah per laboratorium

Katalog laboratorium — judul, nomor sesi, penjelasan — dibutuhkan sejak halaman pertama dibuka. Mesinnya tidak: pengunjung yang membuka satu laboratorium tidak punya alasan mengunduh sebelas yang lain. Karena itu tiap laboratorium dimuat lewat `import()` tersendiri, dan berkas intinya menyusut dari 71 KB menjadi **33,9 KB gzip**.

Pemecahan kode memperbaiki pemuatan pertama tetapi merusak dua hal lain: berpindah laboratorium jadi menunggu unduhan, dan laboratorium yang belum pernah dibuka hilang saat luring. Keduanya dikembalikan dengan mengambil modul sisanya di belakang layar setelah halaman pertama selesai digambar — **dilewati sepenuhnya** saat penghemat data menyala atau sambungannya 2G, karena pengguna yang menyalakan penghemat data sedang meminta persis agar itu tidak dilakukan.

Ada satu jebakan yang mudah terlewat: pengguna bisa berpindah halaman sementara modulnya masih diunduh. Memasang laboratorium ke elemen yang sudah dibuang akan meninggalkan gelang animasi yang terus berjalan tanpa ada yang bisa menghentikannya, jadi tiap pemuatan diberi nomor urut dan hasil bernomor basi dibuang.

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
| `cargo run -p ai-core --bin export_vectors` | Menghasilkan ulang vektor uji lintas bahasa |
| `cd tools/conform && go test ./...` | Uji harness konformansi |
| `cd tools/conform && go run .` | Mengadu implementasi Rust terhadap Go |
| `bash oracle/run.sh` | Menyalakan Oracle, memasang skema, konformansi, dan uji PL/SQL |
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
| `neural.rs` | 32 | 42 |
| `expert.rs` | 26 | 30 |
| `ml.rs` | 30 | 50 |
| `nlp.rs` | 22 | 43 |
| `knowledge.rs` | 28 | 42 |
| `agent.rs` | 22 | 29 |
| `eliza.rs` | 10 | 17 |
| `robotics.rs` | 18 | 26 |
| `fx.rs` | 8 | 17 |
| `rng.rs` | 9 | 16 |
| `lib.rs` | 2 | 4 |
| `ai-wasm/lib.rs` | 55 | 70 |
| `web/src/ui.ts` | 11 | 15 |
| `web/src/viz.ts` | 11 | 42 |
| `web/src/labs/notes.ts` | 1 | 54 |
| `web/src/labs/registry.ts` | 2 | 21 |
| `web/src/ekspor.ts` | 7 | 29 |
| `tools/conform` (Go) | 23 | 12 |
| `oracle/` (PL/SQL) | 27 | 60 |
| **Total** | **413** | **788** |

Ditambah **3.796 pernyataan konformansi** yang mengadu ketiga implementasi terhadap vektor yang sama. Angka itu bukan bagian dari 759 di atas: uji unit membuktikan tiap implementasi konsisten dengan dirinya sendiri, sedangkan konformansi membuktikan ketiganya sepakat satu sama lain — dan hanya yang kedua yang bisa menangkap rumus yang salah tetapi konsisten.

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
- **Laporan ekspor dibangun dari komponen sungguhan** — ujinya menyusun kartu,
  penggeser, dan tabel lewat `ui.ts` yang sama seperti yang dipakai
  laboratorium, lalu memeriksa hasil bacaannya. Kalau suatu hari sebuah
  komponen ditata ulang dan atribut `data-ekspor`-nya hilang, situsnya tetap
  jalan dan tombol unduhnya tetap bekerja — berkasnya hanya kehilangan satu
  blok, diam-diam. Uji inilah satu-satunya yang menangkapnya.
- **Setiap gambar punya padanan teks** — `figure()` menuntut keterangan, dan
  ujinya memeriksa bahwa keterangan itu benar-benar sampai ke `aria-label`.
  Gambar tanpa keterangan adalah kotak kosong bagi pengguna pembaca layar, dan
  tidak ada uji lain di repositori ini yang akan menangkapnya.
- **Setiap `import()` benar-benar dipanggil** — jalur modul yang salah tulis
  tidak tertangkap pemeriksa tipe maupun build; Vite tetap menghasilkan berkas,
  dan kegagalannya baru muncul di peramban pengguna.
- **Lubang di basis pengetahuan terdeteksi** — data benih Oracle sengaja memuat
  satu fakta yang dipakai sebagai premis tetapi tidak bisa disimpulkan maupun
  ditanyakan. Basis seperti itu lolos seluruh batasan tabel dan tetap memberi
  jawaban; yang terjadi hanyalah satu aturan tidak pernah menyala. Ujinya
  menuntut lubang itu ditemukan, tepat satu.

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
| Akses tanpa penglihatan | Tiap gambar SVG memikul `role="img"` dan keterangan; angkanya juga tersedia sebagai tabel | `web/src/viz.ts` |
| Konformansi basis data | Oracle Free 23ai dijalankan sebagai service container, skema dipasang dari nol tiap kali | CI, `oracle/run.sh` |
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
│   │       ├── expert.rs      Sesi 11 — Sistem Pakar
│   │       ├── eliza.rs       Sesi 1 — ELIZA
│   │       ├── agent.rs       Sesi 2 — Agen & Ruang Keadaan
│   │       ├── knowledge.rs   Sesi 7 — Representasi Pengetahuan
│   │       ├── nlp.rs         Sesi 10 — Pemrosesan Bahasa Alami
│   │       ├── ml.rs          Sesi 12-13 — Sains Data & Machine Learning
│   │       ├── robotics.rs    Sesi 14 — Robotika
│   │       ├── fx.rs          Pertukaran pecahan bit-eksak
│   │       └── rng.rs         SplitMix64 deterministik
│   └── ai-wasm/        Jembatan wasm-bindgen. Amplop JSON ok/err.
├── tools/
│   └── conform/        Implementasi pembanding Go + harness konformansi
│       ├── aicore/     Algoritma ditulis ulang dari nol dalam Go
│       └── vectors/    2.266 vektor uji berpola bit, dihasilkan Rust
├── oracle/             Implementasi ketiga: PL/SQL di atas Oracle Free 23ai
│   ├── 01_schema.sql          Basis pengetahuan relasional, seluruhnya BINARY_DOUBLE
│   ├── 02_pkg_ai_core.pks     Spesifikasi paket
│   ├── 03_pkg_ai_core.pkb     Badan paket — urutan operasinya sama persis dengan Rust
│   ├── 04_seed_knowledge.sql  Aturan, himpunan kabur, dataset tenis
│   ├── 05_conformance.sql     Pemeriksa 3.796 pernyataan, per tingkat keterbandingan
│   ├── 06_tests.sql           60 uji unit, harness-nya ditulis sendiri
│   ├── run.sh                 Satu perintah: nyalakan, pasang, konformansi, uji
│   └── tools/                 Pengubah vektor TSV menjadi SQL pemuat
├── web/
│   ├── src/
│   │   ├── engine.ts   Pembungkus bertipe untuk WebAssembly
│   │   ├── viz.ts      Perangkat visualisasi SVG: garis bilangan, air terjun,
│   │   │               graf, peta panas, alur bertahap, siklus
│   │   ├── labs/       Satu berkas per laboratorium, dimuat lewat import()
│   │   │   ├── registry.ts  Katalog: keterangan eager, mesin lazy
│   │   │   └── notes.ts     81 definisi, 42 rumus, 26 kekeliruan, 28 rujukan
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

---

## 🇬🇧 English

### What is this?

You trust a pocket calculator because other people have checked it many times
over. But what if the calculator is new, and you built it yourself?

The most convincing approach: **work the same problem three different ways and
compare.** If all three agree down to the last digit, they are very probably
right. If one differs, you know something is wrong — and you know where to
look.

That is what this project does. Fourteen classical AI algorithms, each written
**three times in three different languages** (Rust, Go, and Oracle PL/SQL),
then checked against one another automatically. Any discrepancy at all fails
the build, and the site does not ship.

You can move the sliders yourself. The computation is Rust compiled to
WebAssembly, running inside your own tab — not on anyone's server.

### Why this architecture

A wrong formula does not crash the program. It quietly produces plausible wrong
numbers — the most dangerous failure mode in numerical software. The only real
defence is to **write the same algorithm more than once, independently, and
compare the results.**

```
                       ┌──────────────────────────────┐
                       │  crates/ai-core   (Rust)     │
                       │  source of truth             │
                       │  pure · no I/O · tested      │
                       └───────────────┬──────────────┘
              ┌────────────────────────┼────────────────────────┐
   ┌──────────▼─────────┐  ┌───────────▼──────────┐  ┌──────────▼──────────┐
   │  crates/ai-wasm    │  │  tools/conform (Go)  │  │  oracle/ (PL/SQL)   │
   │  → browser         │  │  2nd implementation  │  │  3rd implementation │
   └────────────────────┘  └───────────┬──────────┘  └──────────┬──────────┘
                                       └───── compare ──────────┘
                                  2,266 vectors → 3,796 assertions
                                  a 1 ULP difference fails the build
```

Numbers cross language boundaries as **16-digit hexadecimal bit patterns**,
never as decimal. That is not pedantry: measurement found
`serde_json::from_str::<f64>` mis-rounding by 1 ULP on **27,548 of 200,000**
test values (13.8%), while Rust's own `str::parse::<f64>` had zero errors.

### Three findings worth knowing

1. **IEEE-754 only requires `+ − × ÷ √` and comparison to be correctly
   rounded.** `exp`, `ln`, `log2`, `tanh`, and `pow` are not covered. One
   neural-network test passed on Windows and failed on Linux for exactly this
   reason.

2. **Oracle `BINARY_DOUBLE` has no negative zero.** Even
   `UTL_RAW.CAST_TO_BINARY_DOUBLE(HEXTORAW('8000000000000000'))` returns `+0`.
   44 of 3,796 assertions are affected, handled by a distinct `Z` verdict that
   applies only when both values are genuinely zero — and locked by a test.

3. **Subtracting two nearly equal quantities amplifies error.** Information
   gain is `H(before) − H(after)`; a 2 ULP error in `H` (≈ 0.94) becomes 64 ULP
   in the result (≈ 0.029). This produced a fourth comparability tier,
   `CancellingDifference(n)`, which measures tolerance at the **input** scale:
   `|a − b| ≤ n × ulp(scale)`.

### Visualisation in every module

Every lab carries figures built from a shared SVG toolkit — `numberLine`,
`waterfall`, `nodeGraph`, `heatmap`, `pipeline`, `rankedBars`, `cycle` — and
`figure()` **requires** a caption, which becomes the `aria-label`. A figure
without an explanation only helps readers who already understand it, and the
readers who most need a figure are precisely those who do not.

SVG is used for structure; canvas only where hundreds of pixels mean something
solely in aggregate. Canvas is invisible to screen readers, must be redrawn on
every theme change, and has to manage `devicePixelRatio` by hand.

### Your own data, and results you can take with you

Every lab accepts data you type yourself, not just the built-in examples: your
evidence and weights in Certainty Factor, your sentence in NLP, your formula in
knowledge representation, your walls on the search map. All of it is computed
in your browser — not one byte leaves the machine.

Each lab ends with two buttons. **Download CSV (Excel)** writes one file
containing the settings, the results, the calculation steps, and every table on
the page. **Print / save as PDF** gives you the same page without the chrome:
one column, tables intact, ready to staple to an assignment.

The report follows the page: each card becomes one named section, so a file
holding five tables is still readable tomorrow. The active selections —
which algorithm, which heuristic, which inference engine — are recorded too,
because a number without the settings that produced it cannot be reproduced by
anyone.

CSV rather than XLSX: XLSX is a ZIP of XML files, and writing one in the
browser costs a library several times the size of this project's entire
WebAssembly module. Three details make CSV actually open cleanly in Excel, and
all three are commonly missed — a UTF-8 BOM, a leading `sep=,` line for
locales that use semicolons, and CRLF line endings per RFC 4180.

The content is read from the rendered page via `data-ekspor` attributes set by
the shared components in [`web/src/ui.ts`](web/src/ui.ts), rather than
implemented twelve times. A lab built from shared components is exportable with
no extra code; see [`web/src/ekspor.ts`](web/src/ekspor.ts) and
[`tests/ekspor.test.ts`](tests/ekspor.test.ts), whose tests are built from the
real components so that a broken contract fails the suite.

### Bilingual by construction

Every user-visible string is a `Bilingual` value — `bi("Indonesian",
"English")` — so a missing translation is a **type error**, not a string that
silently falls back to the wrong language. The chosen language persists in
`localStorage` and sets `document.documentElement.lang`.

### Running it

```bash
npm install
npm run wasm       # build the Rust → WebAssembly bundle
npm run dev
npm run audit:all  # Rust tests, web tests, typecheck, build, budget
```

Requires Rust 1.75+, Node 22+, and `wasm-pack`. The Oracle implementation runs
under Docker; see `oracle/run.sh`.

---

<div align="center">

**Bagian dari empat situs IND323** · *Part of the four IND323 sites*

**ai-atlas** (Rust → WASM) ·
[kecerdasan-buatan](https://xyb3rpunq.github.io/kecerdasan-buatan/) (Lua) ·
[ind323-ai-lab](https://xyb3rpunq.github.io/ind323-ai-lab/) (Swift) ·
[neuronusa](https://xyb3rpunq.github.io/neuronusa/) (Brython)

</div>
