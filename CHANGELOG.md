# Riwayat Perubahan

Format mengikuti [Keep a Changelog](https://keepachangelog.com/id/1.1.0/);
penomoran mengikuti [Semantic Versioning](https://semver.org/lang/id/).

`.Deckyx`

## [Belum dirilis]

### Diubah
- **Galat mesin membawa kode dan nilai, bukan kalimat.** Seluruh 69 pesan galat
  di pustaka inti dan 11 di jembatan WebAssembly dulu berupa kalimat Bahasa
  Indonesia yang dirakit di Rust, dan sampai ke pengguna apa adanya — termasuk
  di halaman yang seluruh sisanya berbahasa Inggris. Sisi antarmuka tidak punya
  cara memperbaikinya: yang ia terima sudah berupa kalimat jadi.

  Sekarang amplopnya berbunyi `{"err": {"kode": "cf.daftar_kosong", "arg": []}}`
  dan kalimatnya dirakit di `web/src/galat.ts`, dalam bahasa pembacanya.
  `Display` di sisi Rust tetap ada dan tetap berbahasa Indonesia — pembacanya
  pengembang yang sedang menatap kegagalan uji, dan pesan uji yang berbunyi
  "cari.awal_terhalang" jauh lebih sulit dibaca daripada kalimatnya.
- Tiga galat yang menyimpan kalimat Indonesia di dalam **nilainya** dipecah
  menjadi varian tersendiri: sebab teko air yang tak terjangkau, sebab
  kegagalan penguraian rumus, dan ujung pencarian yang terhalang. Untai
  penjelasan di dalam sebuah galat hanya bisa punya satu bahasa.
- Penandanya bernomor (`%1`, `%2`), bukan berurutan. Urutan kata berbeda
  antarbahasa: "diberi %2, harus 1 sampai %1" dan "must be 1 to %1, got %2"
  menyisipkan nilai yang sama di tempat yang berbeda.

### Diubah
- **Kalimat aturan tidak lagi dirakit di mesinnya.** `RuleTrace` kabur dan
  `Step` pakar dulu mengembalikan kalimat jadi berbahasa Indonesia — "JIKA
  Pelayanan = Buruk ATAU …", "JIKA demam DAN pilek MAKA flu" — sehingga
  halaman berbahasa Inggris menampilkannya apa adanya, dan tidak ada yang bisa
  dilakukan sisi antarmuka tentang itu. Keduanya kini mengembalikan **bentuk**
  aturannya, dan kalimatnya dirakit di `web/src/aturan.ts` dalam bahasa
  pembacanya.
- Nama gejala, variabel, dan himpunan tetap menjadi kunci di dalam mesin —
  menerjemahkannya di sana akan memutus rantai penalarannya — dan nama yang
  dibaca manusia diambil dari kamus saat ditampilkan.
- `rationale` dibuang dari basis pengetahuan. Ia satu-satunya bidangnya yang
  berupa kalimat untuk dibaca manusia, dan kalimat yang disimpan di mesin hanya
  bisa punya satu bahasa. Alasan pakar kini tersimpan sisi antarmuka, berkunci
  `id` aturan, dalam kedua bahasa.
- `expert_why` dan `expert_how` dihapus dari jembatan WebAssembly. Keduanya
  hanya merangkai kata dari data yang tidak pernah meninggalkan halaman, jadi
  keduanya kini dirakit di TypeScript — satu perjalanan bolak-balik lebih
  sedikit, dan dua bahasa alih-alih satu.

### Diubah
- 42 nama rumus dan 81 istilah di catatan laboratorium menjadi dwibahasa.
  Seluruhnya sebelumnya hanya berbahasa Indonesia, di halaman yang setiap
  bagian lainnya sudah berganti bahasa.
- Sebelas ekspresi rumus ditulis ulang sebagai lambang yang sama di kedua
  bahasa — `∧`, `∨`, `⟺`, `[syarat]` — alih-alih "dan", "atau", "bila".
  Sebuah substitusi angka bukan kalimat, dan lambang yang berubah bentuk
  antarbahasa memutus hubungannya dengan gambar di sebelahnya.
- Nama simpul dan relasi jaringan semantik lewat kamus saat ditampilkan
  (`adalah` → `is-a`, `burung` → `bird`), sementara kuncinya tetap. Kunci
  pewarisannya sekarang tetapan bernama, bukan untai lepas yang diulang tiga
  kali.
- Judul tab dan tautan lompat ikut berganti bahasa. Keduanya diterapkan di
  fungsi yang sama dengan atribut `lang`, supaya tidak ada jalan mengganti
  bahasa yang melewatinya.

### Ditambahkan
- **Naskah ELIZA berbahasa Inggris**, mengikuti DOCTOR asli Weizenbaum: sebelas
  aturan dengan keutamaan yang sepadan dengan naskah Indonesia, tabel penukaran
  kata ganti tersendiri, dan balasan cadangannya sendiri. Mesinnya tidak
  berubah sama sekali — dan itu justru yang diperlihatkan laboratorium ini:
  seluruh "kecerdasan" ELIZA ada di dalam datanya.
- Jawaban **"bagaimana"** pada laboratorium sistem pakar: dari mana sebuah
  kesimpulan datang, satu baris per langkah. Sebuah kesimpulan bisa datang
  lewat lebih dari satu aturan, dan hanya jawaban ini yang memperlihatkannya.
- **Uji yang benar-benar memasang tiap laboratorium**, lalu membaca layarnya
  dalam Bahasa Inggris dan menolak kata Indonesia yang tersisa — termasuk pada
  atribut yang dibaca manusia. Mesin WebAssembly-nya benar-benar dijalankan,
  karena sebagian besar teks laboratorium baru muncul sesudah ada hasil, dan
  halaman yang hampir kosong lolos setiap pemeriksaan bocoran.
- `error_codes()` di jembatan WebAssembly: seluruh kode galat yang bisa datang
  dari mesin beserta jumlah argumennya. Uji sisi antarmuka menuntut kamusnya
  memuat tepat kode-kode itu, dengan jumlah penanda yang sepadan — sehingga
  kode baru yang belum diterjemahkan gagal di CI, bukan ditemukan pengguna yang
  sedang mengalami kegagalan.
- **Uji yang membaca kode antarmukanya sendiri** dan menolak untai berbahasa
  Indonesia yang tidak menjadi argumen pertama sebuah `bi(...)`. Ia menutup
  celah yang tidak bisa dilihat uji lain: `render.test.ts` memasang tiap
  laboratorium ke wadah lepas, sehingga kerangka aplikasi, catatan, kepala, dan
  kaki halaman tidak pernah ikut terbaca. Bocoran terbesarnya ada persis di
  sana.
- Dua kesepakatan penamaan yang membuat pengecualiannya terbaca dari kodenya:
  `KORPUS_*` untuk bahan yang memang berbahasa Indonesia dengan sengaja, dan
  `KUNCI_*` untuk untai yang merupakan kunci mesin dan bukan teks.
- Penandaan `data-korpus` untuk bahan yang memang berbahasa Indonesia dengan
  sengaja — kalimat contoh di laboratorium pengolahan bahasa, yang morfologi
  Indonesianya justru sedang dibedah. Ditandai di tempatnya, bukan dikecualikan
  berdasarkan nama laboratorium, sehingga bagian lain mana pun tetap dituntut
  diterjemahkan. Alasannya juga dinyatakan di layar.
- Sesi 9 — Jaringan Syaraf Tiruan. Perceptron satu lapis, jaringan berlapis
  dengan perambatan balik dan momentum, enam fungsi aktivasi, inisialisasi
  Xavier, dan empat kumpulan data (XOR, AND, OR, spiral). Batas keputusan
  digambar ulang secara langsung selama pelatihan; kisi 72x72 berarti 5.184
  perambatan maju dan selesai dalam 18 milidetik.
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
- **Uji divergensi jaringan lolos di Windows tetapi gagal di Linux.** Bukan
  flake. IEEE-754 hanya mewajibkan `+ - x / sqrt` dan perbandingan dibulatkan
  dengan benar; `exp`, `ln`, dan `tanh` tidak, sehingga pustaka matematika yang
  berbeda boleh meleset satu ULP. Pada pelatihan berlangkah besar yang sudah
  berayun, selisih itu membesar menjadi hasil yang sama sekali berbeda. Uji
  diganti menjadi langkah yang jauh lebih ekstrem, tempat aktivasinya jenuh
  sepenuhnya, ditambah uji pembanding relatif yang tidak goyah oleh pembulatan.
  Penggolong `Comparability` ditambahkan ke `fx` untuk menetapkan sejak awal
  perhitungan mana yang boleh dituntut identik bit demi bit dan mana yang tidak.
- **Jaringan hasil deserialisasi langsung meledak saat dilatih.** Kecepatan
  momentum tidak ikut diserialisasi, sehingga jaringan yang dibaca kembali dari
  JSON datang dengan penyangga kosong dan pembaruan bobot mengindeks di luar
  batas. Ini bukan kasus pinggiran: antarmuka menyimpan jaringan sebagai JSON di
  antara potongan pelatihan supaya tampilan tidak membeku, jadi setiap potongan
  setelah yang pertama melewatinya. Penyangga kini dibangun ulang bila bentuknya
  tidak cocok.
- **Panel kontrol digambar ulang enam puluh kali sedetik selama pelatihan.**
  Penggeser yang sedang diseret terlepas dari jari pengguna dan fokus papan
  ketik kembali ke awal. Kini hanya gambar dan angka hasil yang disegarkan.
- **Kumpulan data spiral mustahil dipisahkan sempurna.** Kedua lengan dimulai
  dari titik asal, sehingga titik terdalam kedua kelas berimpit dan angka
  ketepatan menjadi menyesatkan. Jari-jari terdalam kini bukan nol.
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
