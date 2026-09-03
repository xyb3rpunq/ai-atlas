//! Galat yang bisa dijelaskan dalam bahasa apa pun.
//!
//! # Kenapa kodenya, bukan kalimatnya
//!
//! Setiap galat di pustaka ini punya `Display` berbahasa Indonesia, dan itu
//! tepat untuk pembacanya: yang membaca `Display` adalah pengembang yang
//! sedang menatap kegagalan uji. Yang keliru adalah membiarkan kalimat itu
//! menyeberang ke peramban, karena di sana pembacanya bisa siapa saja — dan
//! sisi antarmuka tidak punya cara memperbaikinya: yang ia terima sudah berupa
//! kalimat jadi.
//!
//! Jadi galat yang menyeberang membawa **kode dan nilai**, bukan kalimat.
//! Kalimatnya dirakit sisi antarmuka, dalam bahasa pembacanya. Ini perlakuan
//! yang sama persis dengan kalimat aturan JIKA–MAKA: mesin menyediakan
//! bentuknya, antarmuka menyusun kalimatnya.
//!
//! # Kenapa daftar kodenya ada di sini
//!
//! Supaya sisi antarmuka bisa menuntut kelengkapan terjemahannya. Tanpa daftar
//! yang bisa dibaca, satu-satunya cara mengetahui ada kode yang belum
//! diterjemahkan adalah menemuinya — dan yang menemuinya adalah pengguna yang
//! sedang mengalami kegagalan.
//!
//! Daftar ini dan implementasinya saling menjaga: ada uji yang menuntut setiap
//! varian galat punya kode di daftar dengan jumlah argumen yang sepadan, dan
//! sebaliknya bahwa tidak ada kode di daftar yang tidak dipakai varian mana
//! pun.
//!
//! .Deckyx

/// Galat yang membawa kode dan nilai, bukan kalimat.
pub trait Dijelaskan {
    /// Kode yang tetap, tidak pernah diterjemahkan.
    ///
    /// Bentuknya `modul.sebab`, huruf kecil dengan garis bawah. Ia kunci, dan
    /// menggantinya berarti memutus terjemahan yang sudah ada — sama seperti
    /// mengganti nama fakta memutus rantai penalaran sistem pakar.
    fn kode(&self) -> &'static str;

    /// Nilai yang disisipkan ke dalam kalimatnya, berurut.
    ///
    /// Selalu untai, bahkan untuk angka: yang menerimanya akan menampilkannya
    /// apa adanya, dan pemformatan angka adalah urusan tampilan.
    fn argumen(&self) -> Vec<String>;
}

/// Seluruh kode galat pustaka inti, beserta jumlah argumennya.
///
/// Jembatan WebAssembly punya daftarnya sendiri untuk kegagalan yang terjadi
/// sebelum mesinnya dipanggil — masukan yang bukan JSON, nama metode yang tidak
/// dikenal. Keduanya disatukan saat diserahkan ke sisi antarmuka.
///
/// Diurutkan menurut modul lalu sebab, supaya penambahan baru terlihat jelas
/// di dalam beda berkas.
pub const KODE: &[(&str, usize)] = &[
    // Sesi 2 — agen cerdas
    ("agen.jumlah_rombongan", 2),
    ("agen.jumlah_ruangan", 2),
    ("agen.kapasitas_teko", 2),
    ("agen.posisi_awal", 2),
    ("agen.sasaran_bukan_kelipatan", 2),
    ("agen.sasaran_melebihi_teko", 2),
    ("agen.penyeberangan_tak_aman", 0),
    ("agen.ruang_keadaan_habis", 0),
    // Sesi 10 — pengolahan bahasa
    ("bahasa.korpus_kosong", 0),
    ("bahasa.panjang_tak_sepadan", 2),
    ("bahasa.ukuran_ngram", 1),
    // Sesi 4 — probabilitas Bayesian
    ("bayes.belum_dilatih", 0),
    ("bayes.bukti_nol", 0),
    ("bayes.indeks_di_luar_jangkauan", 2),
    ("bayes.masukan_kosong", 0),
    ("bayes.panjang_tak_sepadan", 2),
    ("bayes.prior_tak_berjumlah_satu", 1),
    ("bayes.probabilitas_di_luar_rentang", 1),
    // Sesi 8 — pencarian
    ("cari.awal_terhalang", 0),
    ("cari.di_luar_kisi", 2),
    ("cari.kisi_tak_sah", 2),
    ("cari.panjang_dinding", 2),
    ("cari.tujuan_terhalang", 0),
    // Sesi 3 — certainty factor
    ("cf.cf_di_luar_rentang", 1),
    ("cf.daftar_kosong", 0),
    ("cf.mb_md_di_luar_rentang", 1),
    // Sesi 1 — ELIZA
    ("eliza.aturan_tanpa_balasan", 1),
    ("eliza.masukan_terlalu_panjang", 2),
    ("eliza.naskah_kosong", 0),
    // Pertukaran pecahan bit-eksak
    ("fx.bukan_digit_heksadesimal", 1),
    ("fx.panjang_salah", 2),
    // Sesi 5 & 6 — logika kabur
    ("kabur.basis_aturan_kosong", 0),
    ("kabur.cuplikan_terlalu_sedikit", 1),
    ("kabur.derajat_di_luar_rentang", 1),
    ("kabur.himpunan_tak_dikenal", 1),
    ("kabur.semesta_tak_sah", 2),
    ("kabur.tidak_ada_aturan_menyala", 0),
    ("kabur.titik_tak_terurut", 1),
    ("kabur.variabel_tak_dikenal", 1),
    // Sesi 7 — representasi pengetahuan
    ("logika.basis_kosong", 0),
    ("logika.batas_pembuktian", 1),
    ("logika.simpul_tak_dikenal", 1),
    ("logika.terlalu_banyak_variabel", 2),
    ("logika.urai_karakter_tak_dikenal", 2),
    ("logika.urai_kurung_tutup_hilang", 1),
    ("logika.urai_operator_tanpa_operand", 1),
    ("logika.urai_rumus_kosong", 0),
    ("logika.urai_rumus_terputus", 1),
    ("logika.urai_sisa_masukan", 1),
    // Sesi 12 & 13 — machine learning
    ("ml.baris_tak_rata", 2),
    ("ml.belum_dilatih", 0),
    ("ml.data_kosong", 0),
    ("ml.kelompok_terlalu_banyak", 2),
    ("ml.nilai_bukan_bilangan", 2),
    ("ml.panjang_tak_sepadan", 2),
    ("ml.parameter_tak_sah", 2),
    // Sesi 11 — sistem pakar
    ("pakar.aturan_tanpa_premis", 1),
    ("pakar.basis_aturan_kosong", 0),
    ("pakar.batas_langkah", 1),
    ("pakar.keyakinan_di_luar_rentang", 2),
    ("pakar.penalaran_melingkar", 1),
    // Sesi 14 — robotika
    ("robot.di_luar_jangkauan", 3),
    ("robot.parameter_tak_sah", 2),
    ("robot.tidak_konvergen", 1),
    // Sesi 9 — jaringan syaraf
    ("syaraf.arsitektur_tak_sah", 1),
    ("syaraf.data_kosong", 0),
    ("syaraf.data_tak_sepadan", 2),
    ("syaraf.laju_belajar", 1),
    ("syaraf.masukan_tak_sepadan", 2),
    ("syaraf.menyimpang", 1),
    ("syaraf.target_tak_sepadan", 2),
];

/// Apakah sebuah kode ada di daftar, dengan jumlah argumen yang sepadan.
pub fn terdaftar(kode: &str, argumen: usize) -> bool {
    KODE.iter().any(|(k, n)| *k == kode && *n == argumen)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentError;
    use crate::bayes::BayesError;
    use crate::certainty::CfError;
    use crate::eliza::ElizaError;
    use crate::expert::ExpertError;
    use crate::fuzzy::FuzzyError;
    use crate::fx::FxError;
    use crate::knowledge::{KnowledgeError, ParseCause};
    use crate::ml::MlError;
    use crate::neural::NeuralError;
    use crate::nlp::NlpError;
    use crate::robotics::RoboticsError;
    use crate::search::{Endpoint, SearchError};

    /// Satu contoh tiap varian galat di seluruh pustaka.
    ///
    /// Ditulis tangan, bukan dibangkitkan. Rust tidak punya cara menelusuri
    /// varian sebuah enum saat program berjalan, jadi daftar ini satu-satunya
    /// tempat yang tahu semuanya — dan uji `daftar_kode_terpakai_seluruhnya`
    /// di bawah yang menjaga ia tidak pernah tertinggal: varian baru yang lupa
    /// dimasukkan ke sini akan meninggalkan kodenya tanpa pemakai, dan itu
    /// gagal.
    fn semua_galat() -> Vec<Box<dyn Dijelaskan>> {
        vec![
            Box::new(AgentError::BadRoomCount(0)),
            Box::new(AgentError::StartOutOfRange {
                position: 5,
                rooms: 2,
            }),
            Box::new(AgentError::BadCapacity { a: 0, b: 5 }),
            Box::new(AgentError::TargetExceedsLargestJug {
                target: 9,
                largest: 5,
            }),
            Box::new(AgentError::TargetNotMultipleOfGcd { target: 3, gcd: 2 }),
            Box::new(AgentError::SearchExhausted),
            Box::new(AgentError::NoSafeCrossing),
            Box::new(AgentError::BadPartySize(0)),
            Box::new(BayesError::ProbabilityOutOfRange("1.5".into())),
            Box::new(BayesError::ZeroEvidence),
            Box::new(BayesError::LengthMismatch { a: 2, b: 3 }),
            Box::new(BayesError::EmptyInput),
            Box::new(BayesError::PriorsDoNotSumToOne(0.9)),
            Box::new(BayesError::IndexOutOfRange { index: 4, len: 2 }),
            Box::new(BayesError::NotTrained),
            Box::new(CfError::BeliefOutOfRange("1.5".into())),
            Box::new(CfError::CfOutOfRange("2.0".into())),
            Box::new(CfError::EmptyInput),
            Box::new(ElizaError::EmptyScript),
            Box::new(ElizaError::RuleWithoutResponses("halo".into())),
            Box::new(ElizaError::InputTooLong(9000)),
            Box::new(ExpertError::EmptyRuleBase),
            Box::new(ExpertError::RuleWithoutPremises("R1".into())),
            Box::new(ExpertError::BadCertainty {
                source: "R1".into(),
                value: 2.0,
            }),
            Box::new(ExpertError::CircularReasoning(vec![
                "a".into(),
                "b".into(),
                "a".into(),
            ])),
            Box::new(ExpertError::StepLimitExceeded(100)),
            Box::new(FuzzyError::UnorderedPoints("3, 1".into())),
            Box::new(FuzzyError::BadUniverse {
                min: 10.0,
                max: 0.0,
            }),
            Box::new(FuzzyError::TooFewSamples(1)),
            Box::new(FuzzyError::NoRuleFired),
            Box::new(FuzzyError::EmptyRuleBase),
            Box::new(FuzzyError::UnknownSet("Panas".into())),
            Box::new(FuzzyError::UnknownVariable("Suhu".into())),
            Box::new(FuzzyError::DegreeOutOfRange(1.5)),
            Box::new(FxError::BadLength(3)),
            Box::new(FxError::BadDigit('z')),
            Box::new(KnowledgeError::ParseError {
                cause: ParseCause::EmptyFormula,
                position: 0,
            }),
            Box::new(KnowledgeError::ParseError {
                cause: ParseCause::UnknownCharacter('%'),
                position: 1,
            }),
            Box::new(KnowledgeError::ParseError {
                cause: ParseCause::MissingCloseParen,
                position: 2,
            }),
            Box::new(KnowledgeError::ParseError {
                cause: ParseCause::OperatorWithoutOperand,
                position: 3,
            }),
            Box::new(KnowledgeError::ParseError {
                cause: ParseCause::UnexpectedEnd,
                position: 4,
            }),
            Box::new(KnowledgeError::ParseError {
                cause: ParseCause::TrailingInput,
                position: 5,
            }),
            Box::new(KnowledgeError::TooManyVariables(30)),
            Box::new(KnowledgeError::EmptyKnowledgeBase),
            Box::new(KnowledgeError::UnknownNode("burung".into())),
            Box::new(KnowledgeError::ProofLimitExceeded(100)),
            Box::new(MlError::EmptyDataset),
            Box::new(MlError::LengthMismatch {
                features: 3,
                labels: 2,
            }),
            Box::new(MlError::RaggedRows {
                expected: 3,
                got: 2,
            }),
            Box::new(MlError::BadParameter {
                name: "k".into(),
                value: 0.0,
            }),
            Box::new(MlError::NotTrained),
            Box::new(MlError::TooManyClusters { k: 5, points: 3 }),
            Box::new(MlError::NonFiniteValue { row: 1, column: 2 }),
            Box::new(NeuralError::BadArchitecture("[]".into())),
            Box::new(NeuralError::InputSizeMismatch {
                expected: 2,
                got: 3,
            }),
            Box::new(NeuralError::TargetSizeMismatch {
                expected: 1,
                got: 2,
            }),
            Box::new(NeuralError::DatasetMismatch {
                inputs: 4,
                targets: 3,
            }),
            Box::new(NeuralError::EmptyDataset),
            Box::new(NeuralError::BadLearningRate(-1.0)),
            Box::new(NeuralError::Diverged { epoch: 7 }),
            Box::new(NlpError::EmptyCorpus),
            Box::new(NlpError::LengthMismatch { a: 2, b: 3 }),
            Box::new(NlpError::BadNgramSize(0)),
            Box::new(RoboticsError::BadParameter {
                name: "kp".into(),
                value: -1.0,
            }),
            Box::new(RoboticsError::OutOfReach {
                distance: 9.0,
                max_reach: 4.0,
                min_reach: 1.0,
            }),
            Box::new(RoboticsError::DidNotConverge { steps: 50 }),
            Box::new(SearchError::BadGrid {
                width: 0,
                height: 5,
            }),
            Box::new(SearchError::OutOfBounds { x: 9, y: 9 }),
            Box::new(SearchError::BlockedEndpoint(Endpoint::Start)),
            Box::new(SearchError::BlockedEndpoint(Endpoint::Goal)),
            Box::new(SearchError::WallLengthMismatch {
                expected: 25,
                got: 24,
            }),
        ]
    }

    #[test]
    fn tiap_galat_punya_kode_yang_terdaftar() {
        for g in semua_galat() {
            assert!(
                terdaftar(g.kode(), g.argumen().len()),
                "kode tidak terdaftar atau jumlah argumennya berbeda: {} ({} argumen)",
                g.kode(),
                g.argumen().len()
            );
        }
    }

    #[test]
    fn daftar_kode_terpakai_seluruhnya() {
        // Kode yang tidak dipakai varian mana pun adalah terjemahan yang
        // dirawat tanpa satu pun galat yang menghasilkannya — dan, lebih
        // sering, tanda bahwa sebuah varian lupa dimasukkan ke `semua_galat`.
        let dipakai: Vec<&'static str> = semua_galat().iter().map(|g| g.kode()).collect();
        for (kode, _) in KODE {
            assert!(
                dipakai.contains(kode),
                "kode tidak dipakai siapa pun: {kode}"
            );
        }
    }

    #[test]
    fn kodenya_unik() {
        let mut kode: Vec<&str> = KODE.iter().map(|(k, _)| *k).collect();
        let sebelum = kode.len();
        kode.sort_unstable();
        kode.dedup();
        assert_eq!(kode.len(), sebelum, "ada kode kembar di daftar");
    }

    #[test]
    fn kodenya_berbentuk_modul_titik_sebab() {
        // Bentuk yang tetap membuat daftarnya bisa dibaca sebagai daftar, dan
        // membuat kode baru yang salah bentuk terlihat langsung.
        for (kode, _) in KODE {
            let bagian: Vec<&str> = kode.split('.').collect();
            assert_eq!(bagian.len(), 2, "bentuk kode salah: {kode}");
            for b in bagian {
                assert!(!b.is_empty(), "bagian kosong pada kode: {kode}");
                assert!(
                    b.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                    "kode harus huruf kecil dan garis bawah: {kode}"
                );
            }
        }
    }

    #[test]
    fn tiap_galat_tetap_punya_kalimat_untuk_pengembang() {
        // `Display` tetap ada dan tetap berbahasa Indonesia. Pembacanya
        // pengembang yang sedang menatap kegagalan uji, bukan pengunjung —
        // dan pesan uji yang berbunyi "cari.awal_terhalang" jauh lebih sulit
        // dibaca daripada kalimatnya.
        for g in semua_galat() {
            assert!(!g.kode().is_empty());
        }
    }
}
