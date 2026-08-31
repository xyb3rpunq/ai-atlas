# Riwayat Perubahan

Format mengikuti [Keep a Changelog](https://keepachangelog.com/id/1.1.0/);
penomoran mengikuti [Semantic Versioning](https://semver.org/lang/id/).

`.Deckyx`

## [Belum dirilis]

### Ditambahkan
- Sesi 5 & 6 — Logika Fuzzy. Enam bentuk fungsi keanggotaan, operasi himpunan
  kabur Zadeh dan produk, potongan alfa, lima metode defuzzifikasi, serta tiga
  mesin inferensi (Mamdani, Sugeno, Tsukamoto) yang bisa dibandingkan
  berdampingan pada masukan yang sama.
- Pengerasan produksi: kebijakan keamanan konten yang disegel otomatis saat
  build, pekerja layanan untuk penggunaan luring, manifes aplikasi web, peta
  situs, `robots.txt`, data terstruktur `LearningResource`, dan halaman 404.

### Diperbaiki
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
