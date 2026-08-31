# Kebijakan Keamanan

`.Deckyx`

## Permukaan serangan

AI ATLAS adalah situs statis tanpa peladen, tanpa basis data, dan tanpa akun
pengguna. Seluruh perhitungan terjadi di dalam peramban pengunjung. Tidak ada
data pengguna yang dikirim ke mana pun, dan satu-satunya hal yang disimpan
adalah preferensi tema dan bahasa di `localStorage`.

Meski begitu, ada beberapa hal yang tetap dijaga:

| Kendali | Penerapan |
|---|---|
| Kebijakan keamanan konten | Disegel otomatis saat build oleh `scripts/csp.mjs`. Sidik SHA-256 dihitung untuk tiap skrip sebaris; `'unsafe-inline'` dan `'unsafe-eval'` ditolak dan membuat build gagal. |
| Penyematan lintas situs | **Tidak terlindungi.** Lihat catatan di bawah. |
| Sumber luar | Tidak ada. Tidak ada CDN, tidak ada fon jarak jauh, tidak ada analitik. `connect-src 'self'`. |
| Penyisipan HTML | Seluruh teks masuk lewat `textContent`. Tidak ada jalur `innerHTML` di seluruh basis kode. |
| Kode tidak aman di Rust | `#![forbid(unsafe_code)]` pada kedua crate. |
| Dependensi saat berjalan | Nol. |

### Catatan: perlindungan penyematan

`frame-ancestors` dan `X-Frame-Options` hanya berlaku bila dikirim sebagai tajuk
tanggapan HTTP. GitHub Pages tidak mengizinkan tajuk khusus, dan arahan
`frame-ancestors` di dalam `<meta>` **diabaikan peramban** — mencantumkannya di
sana hanya menghasilkan galat konsol tanpa perlindungan apa pun. Karena itu
arahan tersebut sengaja tidak dipasang.

Risikonya dinilai dapat diterima karena situs ini tidak punya sesi, tidak punya
tombol yang mengubah keadaan di peladen, dan tidak menyimpan apa pun yang
bernilai dicuri. Serangan *clickjacking* memerlukan tindakan bernilai untuk
dibajak; di sini tidak ada.

Bila suatu saat situs ini dipindahkan ke hosting yang bisa mengirim tajuk
(Cloudflare Pages, Netlify, atau peladen sendiri), `frame-ancestors 'none'` dan
`X-Content-Type-Options: nosniff` harus dipasang di sana.

### Catatan: pekerja layanan

Berkas `web/public/sw.js` menyimpan aset agar laboratorium bisa dipakai tanpa
jaringan. Cakupannya dibatasi pada jalur `/ai-atlas/`, hanya menyinggahkan
permintaan `GET` ke asal sendiri, dan melewatkan seluruh permintaan lintas asal
tanpa menyentuhnya. Pendaftarannya gagal secara diam-diam bila peramban
menolaknya, karena situs tetap berfungsi penuh saat daring.

## Melaporkan kerentanan

Kalau Anda menemukan masalah keamanan, buka
[GitHub Security Advisory](https://github.com/xyb3rpunq/ai-atlas/security/advisories/new)
pada repositori ini. Mohon jangan membuka isu publik lebih dulu.

Perkiraan waktu tanggapan: 7 hari kerja untuk pengakuan pertama.

## Versi yang didukung

Hanya `main` yang menerima perbaikan keamanan. Situs ini diterbitkan ulang dari
`main` pada tiap perubahan yang lolos CI.
