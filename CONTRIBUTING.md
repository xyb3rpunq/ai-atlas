# Berkontribusi

`.Deckyx`

## Aturan yang tidak bisa ditawar

1. **Setiap fungsi publik punya uji.** Bukan hanya jalur bahagia — uji nilai
   batas, masukan tidak sah, dan sifat matematis yang seharusnya berlaku
   (komutatif, monoton, jumlah probabilitas sama dengan satu).
2. **Setiap item publik punya dokumentasi.** `#![warn(missing_docs)]` menegakkan
   ini pada sisi Rust.
3. **Seluruh suite dijalankan ulang pada tiap perubahan**, bukan sekali di akhir.
4. **Perilaku numerik tidak boleh berubah diam-diam.** Bila sebuah nilai
   berubah, ubahannya harus disertai penjelasan di pesan komit dan uji yang
   memagarinya.

## Sebelum mengirim perubahan

Perbarui rantai alat lebih dulu. `rust-toolchain.toml` mengunci kanal, bukan
nomor versi, jadi clippy lokal yang tertinggal akan meloloskan kode yang
ditolak CI:

```bash
rustup update stable
```

Lalu:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run audit:all
```

Keempatnya harus bersih. CI menjalankan hal yang sama dan akan menolak yang
tidak lolos.

## Gaya kode

- **Rust** — `rustfmt` bawaan. Nama fungsi dan uji dalam Bahasa Indonesia agar
  sejalan dengan materi kuliahnya; nama tipe dan istilah teknis tetap Inggris.
- **TypeScript** — mode ketat, tanpa `any`, tanpa `innerHTML`.
- **Komentar** menjelaskan *kenapa*, bukan *apa*. Komentar yang hanya mengulang
  isi baris di bawahnya akan diminta dihapus.

## Menambah laboratorium baru

1. Tulis mesinnya di `crates/ai-core/src/`, lengkap dengan ujinya.
2. Ekspos lewat `crates/ai-wasm/src/lib.rs`, lengkap dengan uji jembatannya.
3. Tambahkan pembungkus bertipe di `web/src/engine.ts`.
4. Buat berkas laboratorium di `web/src/labs/`.
5. Daftarkan di `web/src/labs/registry.ts` dan `crates/ai-core/src/lib.rs`.
6. Perbarui `web/public/sitemap.xml` dan tabel silabus di `README.md`.

## Anggaran ukuran

Perubahan yang membuat bundel melewati anggaran di `scripts/budget.mjs` akan
menggagalkan CI. Kalau sebuah fitur memang memerlukan ruang lebih, naikkan
anggarannya dalam komit yang sama dan jelaskan alasannya.
