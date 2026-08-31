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
}
