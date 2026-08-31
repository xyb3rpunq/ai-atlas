//! Pertukaran bilangan pecahan antarbahasa secara bit-eksak.
//!
//! # Kenapa modul ini ada
//!
//! Proyek ini menjalankan algoritma yang sama di Rust, Go, dan PL/SQL lalu
//! membandingkan hasilnya. Perbandingan itu hanya bermakna kalau angkanya
//! berpindah tanpa berubah sedikit pun.
//!
//! Desimal tidak memenuhi syarat itu. Pengukuran pada `serde_json` 1.0.151
//! menunjukkan `from_str::<f64>` salah membulat sebesar 1 ULP pada **27.548
//! dari 200.000** nilai uji (13,8%), sementara `str::parse::<f64>` bawaan Rust
//! nol kesalahan pada himpunan yang sama. Menulis `0.42000000000000004` lalu
//! membacanya kembali bisa menghasilkan `0.42` — angka yang berbeda.
//!
//! Karena itu semua vektor uji lintas bahasa memakai pola bit 64-bit dalam
//! heksadesimal. Bentuk ini tidak punya ruang tafsir: `f64::to_bits` di Rust,
//! `math.Float64bits` di Go, dan konversi `RAW`/`NUMBER` di PL/SQL merujuk pola
//! bit IEEE-754 yang sama persis.
//!
//! JSON tetap dipakai pada batas Rust ke JavaScript, karena `JSON.parse` pada
//! mesin V8 membulatkan dengan benar; yang bermasalah hanya parser sisi Rust.

/// Panjang representasi heksadesimal sebuah `f64`: 16 digit.
pub const HEX_LEN: usize = 16;

/// Seberapa jauh sebuah perhitungan bisa dituntut sama antarbahasa.
///
/// Menyamakan hasil Rust, Go, dan PL/SQL hanya masuk akal bila targetnya
/// ditetapkan lebih dulu. Tidak semua perhitungan bisa dituntut sama persis,
/// dan menuntut yang mustahil hanya menghasilkan uji yang gagal berselang-
/// seling tanpa ada yang salah.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparability {
    /// Hasilnya wajib identik bit demi bit.
    ///
    /// Hanya berlaku untuk perhitungan yang seluruhnya memakai penjumlahan,
    /// pengurangan, perkalian, pembagian, akar kuadrat, dan perbandingan.
    /// IEEE-754 mewajibkan keenamnya dibulatkan dengan benar, sehingga
    /// hasilnya sama di prosesor dan bahasa mana pun.
    BitExact,
    /// Hasilnya boleh berbeda beberapa ULP.
    ///
    /// Berlaku untuk perhitungan yang menyentuh fungsi transendental —
    /// `exp`, `ln`, `tanh`, `sin`, `cos`, `pow`. IEEE-754 **tidak** mewajibkan
    /// fungsi-fungsi ini dibulatkan dengan benar, jadi pustaka matematika yang
    /// berbeda boleh menghasilkan nilai yang berbeda satu ULP untuk masukan
    /// yang sama.
    ///
    /// Ini bukan kekhawatiran teoretis. Sebuah uji jaringan syaraf di proyek
    /// ini lolos di Windows dan gagal di Linux karena persis hal ini: pada
    /// pelatihan berlangkah besar, selisih satu ULP pada `tanh` membesar
    /// menjadi hasil akhir yang sama sekali berbeda.
    NearlyEqual(u64),
    /// Hasilnya adalah selisih dua besaran yang hampir sama besar, sehingga
    /// toleransinya harus diukur pada skala kedua besaran itu — bukan pada
    /// hasilnya.
    ///
    /// Perolehan informasi adalah contohnya: `gain = H(sebelum) - H(sesudah)`
    /// dengan `H(sebelum)` sekitar 0,94 dan `H(sesudah)` sekitar 0,91. Galat
    /// dua ULP pada `H` — yang wajar, karena `log2` bukan operasi yang
    /// dibulatkan dengan benar menurut IEEE-754 — bernilai mutlak sekitar
    /// 2,2e-16. Pada hasil sebesar 0,029 nilai itu sama dengan 64 ULP.
    ///
    /// Menuntut [`Comparability::NearlyEqual`] di sini berarti menuntut
    /// implementasi `log2` yang lebih teliti daripada yang diwajibkan standar.
    /// Yang benar adalah menyatakan toleransinya pada tempat aritmetikanya
    /// terjadi: `|a - b| <= n x ulp(skala)`.
    ///
    /// Diukur pada dataset tenis: selisih Rust lawan PL/SQL paling besar
    /// 2 ULP pada skalanya, jauh di dalam batas 4 yang dipakai.
    CancellingDifference(u64),
    /// Hanya sifatnya yang bisa dituntut, bukan angkanya.
    ///
    /// Berlaku untuk perhitungan yang berperilaku kacau, seperti pelatihan
    /// dengan laju belajar terlalu besar. Yang bisa diuji hanyalah pernyataan
    /// seperti "yang wajar menghasilkan galat lebih kecil daripada yang
    /// ekstrem", bukan nilai tertentu.
    PropertyOnly,
}

impl Comparability {
    /// Apakah dua nilai memenuhi tingkat keterbandingan ini.
    ///
    /// [`Comparability::PropertyOnly`] selalu mengembalikan `true`: pada
    /// tingkat itu perbandingan angka memang tidak bermakna, dan yang menjaga
    /// kebenaran adalah uji sifat yang ditulis terpisah.
    pub fn holds(self, a: f64, b: f64) -> bool {
        match self {
            Comparability::BitExact => bit_equal(a, b),
            Comparability::NearlyEqual(max_ulp) => match ulp_distance(a, b) {
                Some(d) => d <= max_ulp,
                // Tak hingga yang berlawanan tanda, atau satu NaN satu bukan.
                None => bit_equal(a, b),
            },
            // Tingkat ini menuntut skala, dan skalanya tidak ada di sini.
            // Yang dikembalikan adalah pemeriksaan paling ketat, bukan paling
            // longgar: pemanggil yang lupa memberi skala akan melihat
            // kegagalan, bukan kelolosan palsu.
            Comparability::CancellingDifference(_) => bit_equal(a, b),
            Comparability::PropertyOnly => true,
        }
    }

    /// Seperti [`Comparability::holds`], tetapi menyertakan skala tempat
    /// aritmetikanya terjadi.
    ///
    /// Hanya [`Comparability::CancellingDifference`] yang memakai `scale`;
    /// tingkat lain mengabaikannya.
    pub fn holds_scaled(self, a: f64, b: f64, scale: f64) -> bool {
        match self {
            Comparability::CancellingDifference(max_ulp) => {
                if bit_equal(a, b) {
                    return true;
                }
                if !a.is_finite() || !b.is_finite() || !scale.is_finite() {
                    return false;
                }
                (a - b).abs() <= max_ulp as f64 * ulp_step(scale)
            }
            _ => self.holds(a, b),
        }
    }

    /// Penjelasan singkat, dipakai pada laporan ketidakcocokan.
    pub fn describe(self) -> &'static str {
        match self {
            Comparability::BitExact => "wajib identik bit demi bit",
            Comparability::NearlyEqual(_) => "boleh berbeda beberapa ULP",
            Comparability::CancellingDifference(_) => "beberapa ULP diukur pada skala masukannya",
            Comparability::PropertyOnly => "hanya sifatnya yang diuji",
        }
    }
}

/// Toleransi bawaan untuk perhitungan yang menyentuh fungsi transendental.
///
/// Empat ULP cukup longgar untuk menampung perbedaan pustaka matematika yang
/// wajar, tetapi masih jauh lebih ketat daripada toleransi relatif seperti
/// `1e-9` yang akan meloloskan cacat sungguhan pada nilai besar.
pub const TRANSCENDENTAL_ULP: u64 = 4;

/// Kesalahan saat membaca pola bit dari teks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FxError {
    /// Panjang teks bukan 16 digit heksadesimal.
    BadLength(usize),
    /// Teks memuat karakter yang bukan digit heksadesimal.
    BadDigit(char),
}

impl core::fmt::Display for FxError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FxError::BadLength(n) => write!(f, "panjang harus {HEX_LEN} digit, diberi {n}"),
            FxError::BadDigit(c) => write!(f, "bukan digit heksadesimal: {c:?}"),
        }
    }
}

/// Mengubah `f64` menjadi 16 digit heksadesimal huruf kecil.
///
/// ```
/// use ai_core::fx::to_hex;
/// assert_eq!(to_hex(1.0), "3ff0000000000000");
/// assert_eq!(to_hex(0.0), "0000000000000000");
/// ```
pub fn to_hex(v: f64) -> String {
    format!("{:016x}", v.to_bits())
}

/// Membaca kembali `f64` dari 16 digit heksadesimal.
///
/// Menerima huruf besar maupun kecil, tetapi menolak panjang yang salah agar
/// kesalahan pengetikan tidak diam-diam menghasilkan angka lain.
pub fn from_hex(s: &str) -> Result<f64, FxError> {
    let t = s.trim();
    if t.len() != HEX_LEN {
        return Err(FxError::BadLength(t.len()));
    }
    if let Some(c) = t.chars().find(|c| !c.is_ascii_hexdigit()) {
        return Err(FxError::BadDigit(c));
    }
    // Panjang dan himpunan karakter sudah divalidasi, jadi ini tidak bisa gagal.
    let bits = u64::from_str_radix(t, 16).map_err(|_| FxError::BadLength(t.len()))?;
    Ok(f64::from_bits(bits))
}

/// Mengubah sebaris `f64` menjadi satu teks, dipisahkan spasi.
pub fn row_to_hex(values: &[f64]) -> String {
    values
        .iter()
        .map(|v| to_hex(*v))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Membaca sebaris `f64` dari teks berpisah spasi.
pub fn row_from_hex(s: &str) -> Result<Vec<f64>, FxError> {
    s.split_whitespace().map(from_hex).collect()
}

/// Jarak dua `f64` dalam satuan ULP (*unit in the last place*).
///
/// Dipakai untuk melaporkan seberapa jauh dua implementasi menyimpang.
/// Mengembalikan `None` bila salah satu nilai bukan bilangan.
pub fn ulp_distance(a: f64, b: f64) -> Option<u64> {
    if a.is_nan() || b.is_nan() {
        return None;
    }
    if a == b {
        return Some(0);
    }
    if a.is_infinite() || b.is_infinite() {
        return None;
    }
    // Memetakan pola bit ke bilangan bulat bertanda yang terurut monoton,
    // sehingga selisihnya langsung menyatakan jumlah f64 di antara keduanya.
    let key = |v: f64| -> i64 {
        let bits = v.to_bits() as i64;
        if bits < 0 {
            i64::MIN - bits
        } else {
            bits
        }
    };
    Some(key(a).abs_diff(key(b)))
}

/// Jarak antara `x` dan bilangan `f64` terdekat berikutnya yang lebih besar
/// nilai mutlaknya.
///
/// Dipakai untuk menyatakan toleransi pada skala tertentu, bukan pada hasil
/// akhirnya. Lihat [`Comparability::CancellingDifference`].
///
/// ```
/// use ai_core::fx::ulp_step;
/// assert_eq!(ulp_step(1.0), f64::EPSILON);
/// // Nol tidak punya "ULP" yang bermakna, jadi dipakai bilangan subnormal
/// // terkecil — langkah sesungguhnya dari nol ke bilangan berikutnya.
/// assert_eq!(ulp_step(0.0), f64::from_bits(1));
/// ```
///
/// Mengembalikan `NaN` untuk masukan yang tidak berhingga, dan tak hingga
/// untuk [`f64::MAX`] karena bilangan berikutnya memang sudah tak hingga.
pub fn ulp_step(x: f64) -> f64 {
    if !x.is_finite() {
        return f64::NAN;
    }
    let a = x.abs();
    if a == 0.0 {
        return f64::from_bits(1);
    }
    f64::from_bits(a.to_bits() + 1) - a
}

/// Apakah dua nilai sama persis pada tingkat bit, dengan `NaN` dianggap sama.
///
/// Perbandingan `==` biasa menyatakan `NaN != NaN`, padahal untuk membandingkan
/// dua implementasi kita justru ingin "sama-sama menghasilkan NaN" dinilai lolos.
pub fn bit_equal(a: f64, b: f64) -> bool {
    if a.is_nan() && b.is_nan() {
        return true;
    }
    a.to_bits() == b.to_bits()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn langkah_ulp_sepadan_dengan_epsilon() {
        assert_eq!(ulp_step(1.0), f64::EPSILON);
        assert_eq!(ulp_step(-1.0), f64::EPSILON);
        assert_eq!(ulp_step(0.0), f64::from_bits(1));
        assert_eq!(ulp_step(-0.0), f64::from_bits(1));
        assert!(ulp_step(f64::NAN).is_nan());
        assert!(ulp_step(f64::INFINITY).is_nan());
    }

    #[test]
    fn langkah_ulp_membesar_bersama_nilainya() {
        // Satu ULP pada 1024 seribu kali lebih besar daripada pada 1. Justru
        // inilah alasan toleransi ULP harus disebut skalanya.
        assert!(ulp_step(1024.0) > ulp_step(1.0) * 1000.0);
    }

    #[test]
    fn selisih_yang_saling_meniadakan_diukur_pada_skalanya() {
        let tingkat = Comparability::CancellingDifference(4);
        let skala = 0.9402859586706311_f64;
        // Galat dua ULP pada skalanya.
        let galat = 2.0 * ulp_step(skala);
        let a = 0.02922256565895487_f64;
        let b = a + galat;
        assert!(tingkat.holds_scaled(a, b, skala));
        // Pada hasil sekecil itu, galat yang sama berjarak puluhan ULP,
        // sehingga tingkat NearlyEqual(4) menolaknya.
        assert!(!Comparability::NearlyEqual(4).holds(a, b));
        // Lima ULP pada skalanya sudah di luar batas.
        assert!(!tingkat.holds_scaled(a, a + 5.0 * ulp_step(skala), skala));
    }

    #[test]
    fn selisih_tanpa_skala_dinilai_paling_ketat() {
        // Lupa memberi skala harus berujung kegagalan, bukan kelolosan palsu.
        let tingkat = Comparability::CancellingDifference(4);
        assert!(tingkat.holds(1.0, 1.0));
        assert!(!tingkat.holds(1.0, 1.0 + f64::EPSILON));
        assert!(!tingkat.holds_scaled(1.0, 2.0, f64::NAN));
        assert!(!tingkat.holds_scaled(1.0, 2.0, f64::INFINITY));
        assert!(!tingkat.describe().is_empty());
    }

    #[test]
    fn hex_nilai_yang_dikenal() {
        assert_eq!(to_hex(1.0), "3ff0000000000000");
        assert_eq!(to_hex(0.0), "0000000000000000");
        assert_eq!(to_hex(-0.0), "8000000000000000");
        assert_eq!(to_hex(2.0), "4000000000000000");
        assert_eq!(to_hex(-1.0), "bff0000000000000");
        assert_eq!(to_hex(f64::INFINITY), "7ff0000000000000");
    }

    #[test]
    fn hex_selalu_enam_belas_digit() {
        for v in [0.0, 1.0, -1.0, 1e-300, 1e300, f64::MIN_POSITIVE] {
            assert_eq!(to_hex(v).len(), HEX_LEN, "gagal untuk {v}");
        }
    }

    #[test]
    fn bolak_balik_eksak() {
        let nilai = [
            0.0,
            -0.0,
            1.0,
            -1.0,
            0.1,
            0.42,
            0.9 * 0.2 + 0.3 * 0.8,
            core::f64::consts::PI,
            f64::MIN_POSITIVE,
            f64::MAX,
            f64::MIN,
            1e-300,
            1e300,
        ];
        for v in nilai {
            let h = to_hex(v);
            let back = from_hex(&h).unwrap();
            assert!(bit_equal(v, back), "{v} -> {h} -> {back}");
        }
    }

    #[test]
    fn bolak_balik_menangkap_kasus_yang_meleset_di_desimal() {
        // Nilai inilah yang membuat serde_json meleset 1 ULP.
        let v = 0.9 * 0.2 + 0.3 * 0.8;
        assert_ne!(v, 0.42, "prasyarat uji: nilai ini bukan 0.42 persis");
        assert!(bit_equal(from_hex(&to_hex(v)).unwrap(), v));
    }

    #[test]
    fn nan_dan_takhingga_bolak_balik() {
        assert!(from_hex(&to_hex(f64::NAN)).unwrap().is_nan());
        assert_eq!(from_hex(&to_hex(f64::INFINITY)).unwrap(), f64::INFINITY);
        assert_eq!(
            from_hex(&to_hex(f64::NEG_INFINITY)).unwrap(),
            f64::NEG_INFINITY
        );
    }

    #[test]
    fn hex_menerima_huruf_besar_dan_spasi_pinggir() {
        assert_eq!(from_hex("3FF0000000000000").unwrap(), 1.0);
        assert_eq!(from_hex("  3ff0000000000000  ").unwrap(), 1.0);
    }

    #[test]
    fn hex_menolak_bentuk_salah() {
        assert_eq!(from_hex("3ff"), Err(FxError::BadLength(3)));
        assert_eq!(from_hex(""), Err(FxError::BadLength(0)));
        assert_eq!(from_hex("3ff00000000000000"), Err(FxError::BadLength(17)));
        assert_eq!(from_hex("3ff000000000000z"), Err(FxError::BadDigit('z')));
        assert_eq!(from_hex("0x30000000000000"), Err(FxError::BadDigit('x')));
    }

    #[test]
    fn baris_bolak_balik() {
        let v = vec![1.0, 0.42, -3.5, f64::MIN_POSITIVE];
        let s = row_to_hex(&v);
        assert_eq!(s.split(' ').count(), 4);
        let back = row_from_hex(&s).unwrap();
        assert_eq!(back.len(), v.len());
        for (a, b) in v.iter().zip(&back) {
            assert!(bit_equal(*a, *b));
        }
    }

    #[test]
    fn baris_kosong_menghasilkan_larik_kosong() {
        assert_eq!(row_from_hex("").unwrap(), Vec::<f64>::new());
        assert_eq!(row_to_hex(&[]), "");
    }

    #[test]
    fn baris_rusak_menghasilkan_error() {
        assert!(row_from_hex("3ff0000000000000 rusak").is_err());
    }

    #[test]
    fn jarak_ulp_nol_untuk_nilai_sama() {
        assert_eq!(ulp_distance(1.0, 1.0), Some(0));
        assert_eq!(ulp_distance(0.0, -0.0), Some(0));
        assert_eq!(ulp_distance(-5.5, -5.5), Some(0));
    }

    #[test]
    fn jarak_ulp_satu_untuk_tetangga() {
        let a = 0.42f64;
        let b = f64::from_bits(a.to_bits() + 1);
        assert_eq!(ulp_distance(a, b), Some(1));
        assert_eq!(ulp_distance(b, a), Some(1));
    }

    #[test]
    fn jarak_ulp_mengukur_kasus_serde_json() {
        // Persis penyimpangan yang ditemukan: 0.42000000000000004 vs 0.42.
        let a = 0.9 * 0.2 + 0.3 * 0.8;
        assert_eq!(ulp_distance(a, 0.42), Some(1));
    }

    #[test]
    fn jarak_ulp_melintasi_nol() {
        let pos = f64::from_bits(1); // f64 positif terkecil
        let neg = -pos;
        assert_eq!(ulp_distance(pos, neg), Some(2));
    }

    #[test]
    fn jarak_ulp_tidak_terdefinisi_untuk_nan_dan_takhingga() {
        assert_eq!(ulp_distance(f64::NAN, 1.0), None);
        assert_eq!(ulp_distance(1.0, f64::NAN), None);
        assert_eq!(ulp_distance(f64::INFINITY, 1.0), None);
        // Dua tak hingga yang sama tetap dianggap identik.
        assert_eq!(ulp_distance(f64::INFINITY, f64::INFINITY), Some(0));
    }

    #[test]
    fn kesetaraan_bit() {
        assert!(bit_equal(1.0, 1.0));
        assert!(bit_equal(f64::NAN, f64::NAN));
        assert!(!bit_equal(0.0, -0.0), "nol positif dan negatif beda bit");
        assert!(!bit_equal(0.42, 0.9 * 0.2 + 0.3 * 0.8));
    }

    #[test]
    fn pesan_error_terbaca() {
        assert!(FxError::BadLength(3).to_string().contains('3'));
        assert!(FxError::BadDigit('z').to_string().contains('z'));
    }

    #[test]
    fn keterbandingan_bit_eksak() {
        let c = Comparability::BitExact;
        assert!(c.holds(1.0, 1.0));
        assert!(c.holds(f64::NAN, f64::NAN));
        assert!(!c.holds(0.42, 0.9 * 0.2 + 0.3 * 0.8));
        assert!(!c.holds(0.0, -0.0));
        assert!(!c.describe().is_empty());
    }

    #[test]
    fn keterbandingan_beberapa_ulp() {
        let c = Comparability::NearlyEqual(TRANSCENDENTAL_ULP);
        assert!(c.holds(1.0, 1.0));
        // Selisih satu ULP diterima.
        assert!(c.holds(0.42, 0.9 * 0.2 + 0.3 * 0.8));
        // Selisih besar tetap ditolak.
        assert!(!c.holds(1.0, 1.0001));

        // Batasnya benar-benar dijaga: lima ULP ditolak oleh toleransi empat.
        let a = 1.0f64;
        let lima = f64::from_bits(a.to_bits() + 5);
        assert!(!c.holds(a, lima));
        let empat = f64::from_bits(a.to_bits() + 4);
        assert!(c.holds(a, empat));
    }

    #[test]
    fn keterbandingan_menangani_nilai_khusus() {
        let c = Comparability::NearlyEqual(4);
        assert!(c.holds(f64::NAN, f64::NAN));
        assert!(c.holds(f64::INFINITY, f64::INFINITY));
        assert!(!c.holds(f64::INFINITY, f64::NEG_INFINITY));
        assert!(!c.holds(f64::NAN, 1.0));
    }

    #[test]
    fn keterbandingan_sifat_selalu_lolos() {
        let c = Comparability::PropertyOnly;
        assert!(c.holds(1.0, 1000.0));
        assert!(c.holds(f64::NAN, 0.0));
        assert!(!c.describe().is_empty());
    }

    #[test]
    fn operasi_dasar_memang_bit_eksak() {
        // Keenam operasi ini diwajibkan IEEE-754 dibulatkan dengan benar,
        // sehingga aman dituntut identik antarbahasa.
        let a = 0.1f64;
        let b = 0.3f64;
        for hasil in [a + b, a - b, a * b, a / b, (a * b).sqrt()] {
            assert!(Comparability::BitExact.holds(hasil, hasil));
        }
    }
}
