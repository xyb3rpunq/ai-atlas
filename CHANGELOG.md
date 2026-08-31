# Riwayat Perubahan

Format mengikuti [Keep a Changelog](https://keepachangelog.com/id/1.1.0/);
penomoran mengikuti [Semantic Versioning](https://semver.org/lang/id/).

`.Deckyx`

## [Belum dirilis]

### Ditambahkan
- Sesi 8 — Teknik Pencarian. Sembilan algoritma di atas satu peta yang bisa
  digambar sendiri: BFS, DFS, DLS, IDDFS, UCS, greedy best-first, A\*, hill
  climbing, dan simulated annealing. Yang dianimasikan adalah urutan sel yang
  diperiksa, bukan jalurnya, ditambah tabel perbandingan sembilan algoritma
  sekaligus. Termasuk pembangkit labirin yang dijamin punya jalan keluar.
- Sesi 5 & 6 — Logika Fuzzy. Enam bentuk fungsi keanggotaan, operasi himpunan
  kabur Zadeh dan produk, potongan alfa, lima metode defuzzifikasi, serta tiga
  mesin inferensi (Mamdani, Sugeno, Tsukamoto) yang bisa dibandingkan
  berdampingan pada masukan yang sama.
- Pengerasan produksi: kebijakan keamanan konten yang disegel otomatis saat
  build, pekerja layanan untuk penggunaan luring, manifes aplikasi web, peta
  situs, `robots.txt`, data terstruktur `LearningResource`, dan halaman 404.

### Diperbaiki
- **A\* melebar percuma di ruang terbuka.** Tanpa pemutus seri, ribuan simpul
  bernilai `f` identik sehingga A\* membuka seluruh kisi 21x21 — 441 sel, sama
  banyak dengan pencarian tanpa heuristik. Seri kini diputus dengan taksiran
  sisa terkecil. Pemutus itu **tidak** dipakai pada pencarian biaya seragam,
  karena UCS menurut definisinya tidak mengenal heuristik; memakainya di sana
  diam-diam mengubah UCS menjadi algoritma lain.
- **Urutan prioritas rusak untuk bilangan negatif.** Pemetaan pecahan ke bilangan
  bulat bercabang pada `v >= 0.0`, padahal `-0.0 >= 0.0` bernilai benar pada
  IEEE-754. Nol negatif karenanya masuk cabang bilangan positif. Kini bercabang
  pada bit tanda.
- **Kebijakan keamanan konten memblokir skrip di peladen pengembangan.** Penanda
  pengganti sidik bukan nilai yang sah, sehingga tema berkedip putih tiap muat
  ulang. Meta kebijakan kini dilepas saat pengembangan; hasil build tetap
  diverifikasi penuh.
- **Himpunan bahu bernilai nol di tepi semesta.** Pemeriksaan tepi berjalan
  sebelum pemeriksaan puncak datar, sehingga trapesium seperti `(5, 8, 10, 10)`
  menghasilkan derajat nol tepat di `x = 10` — mematikan seluruh aturan di ujung
  atas semesta. Bentuk seperti ini justru yang paling lazim dipakai untuk
  himpunan paling kiri dan paling kanan. Ditemukan oleh uji monotonisitas.
- Nomor urut pada tabel muncul sebagai `1.0000`. Bilangan bulat kini ditulis
  apa adanya.
- Rumus Bayes menampilkan `0.42000000000000004` di layar. Nilai internal tetap
  presisi penuh; hanya tampilannya yang dibulatkan.

## [1.0.0] — 2026-08-31

### Ditambahkan
- Fondasi Rust ke WebAssembly dengan antarmuka TypeScript tanpa kerangka kerja
  dan tanpa dependensi saat berjalan.
- Sesi 3 — Certainty Factor MYCIN: CF dari MB/MD, kombinasi paralel, berantai,
  premis `AND`/`OR`, interpretasi linguistik dwibahasa, jejak langkah.
- Sesi 4 — Probabilitas Bayesian: teorema Bayes, probabilitas total, rasio
  kemungkinan, Naive Bayes kategorikal dan Gaussian, diagram frekuensi alami.
- Anggaran ukuran yang ditegakkan CI.

### Catatan teknis
- Ditemukan bahwa `serde_json::from_str::<f64>` salah membulat sebesar 1 ULP
  pada 27.548 dari 200.000 nilai uji (13,8%), sementara `str::parse::<f64>`
  bawaan Rust nol kesalahan. Akibatnya, seluruh vektor uji lintas bahasa
  memakai pola bit heksadesimal, bukan desimal. Lihat `crates/ai-core/src/fx.rs`.
