//! Pembangkit bilangan acak deterministik.
//!
//! Sengaja tidak memakai `rand` agar hasilnya **bit-identik** di Rust, Go, dan
//! WebAssembly. Reproduktifitas ini yang membuat *differential testing*
//! antarimplementasi mungkin dilakukan: dua bahasa diberi benih yang sama harus
//! menghasilkan deret yang sama persis.
//!
//! Algoritma: SplitMix64 (Steele, Lea & Flood, 2014).

/// Generator SplitMix64 dengan state 64-bit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitMix64 {
    state: u64,
}

/// Konstanta penambah SplitMix64 (bagian pecahan dari rasio emas, 64-bit).
pub const GOLDEN_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

impl SplitMix64 {
    /// Membuat generator baru dari sebuah benih.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Nilai state saat ini. Berguna untuk menyimpan/memulihkan posisi.
    pub fn state(&self) -> u64 {
        self.state
    }

    /// Mengambil `u64` berikutnya dan memajukan state.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GOLDEN_GAMMA);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Pecahan seragam pada rentang setengah terbuka `[0, 1)`.
    ///
    /// Memakai 53 bit teratas agar setiap nilai `f64` yang mungkin punya
    /// peluang sama — pola yang sama dipakai di sisi Go.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }

    /// Pecahan seragam pada rentang `[lo, hi)`.
    pub fn range_f64(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.next_f64()
    }

    /// Bilangan bulat seragam pada rentang `[0, n)`. Mengembalikan `0` bila `n == 0`.
    ///
    /// Memakai metode Lemire tanpa penolakan (*bias* di bawah 2^-64, cukup
    /// untuk keperluan simulasi di sini) agar mudah dicocokkan di Go.
    pub fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        ((self.next_u64() as u128 * n as u128) >> 64) as u64
    }

    /// Contoh dari distribusi normal baku memakai transformasi Box-Muller.
    pub fn next_gaussian(&mut self) -> f64 {
        // u1 dijaga > 0 supaya ln() tidak menghasilkan -inf.
        let mut u1 = self.next_f64();
        while u1 <= f64::MIN_POSITIVE {
            u1 = self.next_f64();
        }
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (core::f64::consts::TAU * u2).cos()
    }

    /// Mengacak urutan sebuah slice memakai Fisher-Yates ke arah mundur.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        if items.len() < 2 {
            return;
        }
        for i in (1..items.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            items.swap(i, j);
        }
    }
}

impl Default for SplitMix64 {
    fn default() -> Self {
        Self::new(0x2545_F491_4F6C_DD1D)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deret_referensi_splitmix64() {
        // Vektor uji dari acuan asli SplitMix64 dengan benih 0.
        let mut r = SplitMix64::new(0);
        assert_eq!(r.next_u64(), 0xE220_A839_7B1D_CDAF);
        assert_eq!(r.next_u64(), 0x6E78_9E6A_A1B9_65F4);
        assert_eq!(r.next_u64(), 0x06C4_5D18_8009_454F);
    }

    #[test]
    fn benih_sama_menghasilkan_deret_sama() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..64 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn benih_beda_menghasilkan_deret_beda() {
        let mut a = SplitMix64::new(1);
        let mut b = SplitMix64::new(2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn f64_selalu_dalam_nol_sampai_satu() {
        let mut r = SplitMix64::new(7);
        for _ in 0..10_000 {
            let v = r.next_f64();
            assert!((0.0..1.0).contains(&v), "{v} di luar [0,1)");
        }
    }

    #[test]
    fn f64_rata_ratanya_mendekati_setengah() {
        let mut r = SplitMix64::new(99);
        let n = 100_000;
        let mean: f64 = (0..n).map(|_| r.next_f64()).sum::<f64>() / n as f64;
        assert!((mean - 0.5).abs() < 0.01, "rata-rata {mean}");
    }

    #[test]
    fn range_menghormati_batas() {
        let mut r = SplitMix64::new(11);
        for _ in 0..5_000 {
            let v = r.range_f64(-3.0, 7.5);
            assert!((-3.0..7.5).contains(&v), "{v}");
        }
    }

    #[test]
    fn range_terbalik_tetap_terbatas() {
        let mut r = SplitMix64::new(5);
        for _ in 0..1_000 {
            let v = r.range_f64(5.0, 1.0);
            assert!(v > 1.0 - 1e-9 && v <= 5.0 + 1e-9, "{v}");
        }
    }

    #[test]
    fn below_tidak_pernah_mencapai_n() {
        let mut r = SplitMix64::new(3);
        for _ in 0..10_000 {
            assert!(r.below(10) < 10);
            assert!(r.below(1) < 1);
        }
    }

    #[test]
    fn below_nol_aman() {
        let mut r = SplitMix64::new(3);
        assert_eq!(r.below(0), 0);
    }

    #[test]
    fn below_menyentuh_semua_nilai() {
        let mut r = SplitMix64::new(2024);
        let mut seen = [false; 6];
        for _ in 0..2_000 {
            seen[r.below(6) as usize] = true;
        }
        assert!(seen.iter().all(|s| *s), "ada sisi dadu yang tidak muncul");
    }

    #[test]
    fn gaussian_statistiknya_wajar() {
        let mut r = SplitMix64::new(1234);
        let n = 50_000;
        let xs: Vec<f64> = (0..n).map(|_| r.next_gaussian()).collect();
        assert!(xs.iter().all(|v| v.is_finite()));
        let mean = xs.iter().sum::<f64>() / n as f64;
        let var = xs.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
        assert!(mean.abs() < 0.05, "mean {mean}");
        assert!((var - 1.0).abs() < 0.05, "var {var}");
    }

    #[test]
    fn shuffle_mempertahankan_isi() {
        let mut r = SplitMix64::new(8);
        let mut v: Vec<u32> = (0..50).collect();
        r.shuffle(&mut v);
        let mut sorted = v.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..50).collect::<Vec<_>>());
    }

    #[test]
    fn shuffle_benar_benar_mengacak() {
        let mut r = SplitMix64::new(8);
        let original: Vec<u32> = (0..50).collect();
        let mut v = original.clone();
        r.shuffle(&mut v);
        assert_ne!(v, original, "urutan tidak berubah sama sekali");
    }

    #[test]
    fn shuffle_aman_untuk_slice_pendek() {
        let mut r = SplitMix64::new(8);
        let mut kosong: Vec<u32> = vec![];
        r.shuffle(&mut kosong);
        assert!(kosong.is_empty());
        let mut satu = vec![9u32];
        r.shuffle(&mut satu);
        assert_eq!(satu, vec![9]);
    }

    #[test]
    fn state_bisa_dibaca_dan_maju() {
        let mut r = SplitMix64::new(100);
        let s0 = r.state();
        r.next_u64();
        assert_ne!(r.state(), s0);
        assert_eq!(r.state(), 100u64.wrapping_add(GOLDEN_GAMMA));
    }

    #[test]
    fn default_deterministik() {
        let mut a = SplitMix64::default();
        let mut b = SplitMix64::default();
        assert_eq!(a.next_u64(), b.next_u64());
    }
}
