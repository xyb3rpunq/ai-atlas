//! # ai-core
//!
//! Algoritma kecerdasan buatan klasik yang ditulis dari nol, mengikuti silabus
//! **IND323 Artificial Intelligence** (Fakultas Ilmu Komputer, Universitas Esa
//! Unggul). Setiap sesi kuliah dipetakan menjadi satu modul Rust.
//!
//! ## Prinsip
//!
//! - **Murni.** Tidak ada I/O, tidak ada waktu sistem, tidak ada keacakan
//!   tersembunyi. Semua yang acak digerakkan oleh benih yang eksplisit lewat
//!   [`rng::SplitMix64`], jadi hasilnya bisa direproduksi persis.
//! - **Dapat dibandingkan.** Nilai yang keluar harus identik dengan
//!   implementasi pembanding di Go dan PL/SQL. Perbedaan sekecil apa pun
//!   dianggap cacat.
//! - **Terdokumentasi dan teruji.** Setiap item publik punya dokumentasi dan
//!   setiap fungsi punya uji.
//!
//! ## Peta modul terhadap sesi kuliah
//!
//! | Sesi | Topik | Modul |
//! |------|-------|-------|
//! | 3 | Ketidakpastian | [`certainty`] |
//! | 4 | Probabilitas Bayesian | [`bayes`] |
//! | 5-6 | Logika Fuzzy | [`fuzzy`] |
//! | 8 | Teknik Pencarian | [`search`] |
//! | — | Penunjang: keacakan deterministik | [`rng`] |
//! | — | Penunjang: pertukaran pecahan bit-eksak | [`fx`] |

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod bayes;
pub mod certainty;
pub mod fuzzy;
pub mod fx;
pub mod rng;
pub mod search;

/// Versi pustaka, diambil dari `Cargo.toml` saat kompilasi.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Ringkasan satu sesi kuliah dan modul yang mengimplementasikannya.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionInfo {
    /// Nomor sesi pada silabus IND323.
    pub session: u8,
    /// Nama modul Rust yang mengimplementasikan sesi ini.
    pub module: &'static str,
    /// Judul sesi dalam Bahasa Indonesia.
    pub title_id: &'static str,
    /// Judul sesi dalam Bahasa Inggris.
    pub title_en: &'static str,
}

/// Daftar sesi yang sudah terimplementasi, terurut menaik.
pub const SESSIONS: &[SessionInfo] = &[
    SessionInfo {
        session: 3,
        module: "certainty",
        title_id: "Ketidakpastian pada Kecerdasan Buatan",
        title_en: "Uncertainty in Artificial Intelligence",
    },
    SessionInfo {
        session: 4,
        module: "bayes",
        title_id: "Probabilitas Bayesian",
        title_en: "Bayesian Probability",
    },
    SessionInfo {
        session: 5,
        module: "fuzzy",
        title_id: "Logika Fuzzy I",
        title_en: "Fuzzy Logic I",
    },
    SessionInfo {
        session: 6,
        module: "fuzzy",
        title_id: "Logika Fuzzy II",
        title_en: "Fuzzy Logic II",
    },
    SessionInfo {
        session: 8,
        module: "search",
        title_id: "Teknik Pencarian dan Pelacakan",
        title_en: "Search Techniques",
    },
];

/// Mencari informasi sesi berdasarkan nomornya.
pub fn session(number: u8) -> Option<&'static SessionInfo> {
    SESSIONS.iter().find(|s| s.session == number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versi_tidak_kosong() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.contains('.'));
    }

    #[test]
    fn daftar_sesi_terurut_dan_unik() {
        let mut prev = 0u8;
        for s in SESSIONS {
            assert!(s.session > prev, "sesi {} tidak menaik", s.session);
            prev = s.session;
        }
    }

    #[test]
    fn tiap_sesi_punya_judul_dua_bahasa() {
        for s in SESSIONS {
            assert!(!s.title_id.is_empty(), "sesi {} tanpa judul ID", s.session);
            assert!(!s.title_en.is_empty(), "sesi {} tanpa judul EN", s.session);
            assert!(!s.module.is_empty());
        }
    }

    #[test]
    fn pencarian_sesi() {
        assert_eq!(session(3).unwrap().module, "certainty");
        assert_eq!(session(4).unwrap().module, "bayes");
        assert!(session(99).is_none());
    }
}
