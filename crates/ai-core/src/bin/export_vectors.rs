//! Mengekspor vektor uji lintas bahasa.
//!
//! Keluarannya dibaca harness Go dan skrip PL/SQL untuk memeriksa bahwa
//! implementasi mereka menghasilkan angka yang sama dengan Rust.
//!
//! Seluruh bilangan pecahan ditulis sebagai **pola bit heksadesimal**, bukan
//! desimal. Alasannya terukur: `serde_json::from_str::<f64>` salah membulat
//! sebesar 1 ULP pada 27.548 dari 200.000 nilai uji, sementara pola bit tidak
//! punya ruang tafsir sama sekali. Lihat [`ai_core::fx`].
//!
//! Tiap berkas juga menyatakan tingkat keterbandingannya, karena tidak semua
//! perhitungan boleh dituntut identik bit demi bit — fungsi transendental
//! seperti `exp` dan `ln` tidak diwajibkan IEEE-754 dibulatkan dengan benar,
//! sehingga pustaka matematika yang berbeda boleh meleset satu ULP.
//!
//! .Deckyx

use ai_core::fx;
use ai_core::rng::SplitMix64;
use std::fmt::Write as _;

/// Menyusun satu berkas vektor.
struct VectorFile {
    name: &'static str,
    comparability: &'static str,
    description: &'static str,
    header: Vec<&'static str>,
    rows: Vec<Vec<String>>,
}

impl VectorFile {
    fn new(
        name: &'static str,
        comparability: &'static str,
        description: &'static str,
        header: Vec<&'static str>,
    ) -> Self {
        Self {
            name,
            comparability,
            description,
            header,
            rows: Vec::new(),
        }
    }

    fn push(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    /// Bentuk teks berkas: baris komentar diawali `#`, sisanya dipisah tab.
    fn render(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "# ai-atlas vektor uji lintas bahasa — .Deckyx");
        let _ = writeln!(out, "# berkas: {}", self.name);
        let _ = writeln!(out, "# keterbandingan: {}", self.comparability);
        let _ = writeln!(out, "# {}", self.description);
        let _ = writeln!(
            out,
            "# seluruh pecahan ditulis sebagai 16 digit heksadesimal pola bit IEEE-754"
        );
        let _ = writeln!(out, "# kolom: {}", self.header.join("\t"));
        for row in &self.rows {
            let _ = writeln!(out, "{}", row.join("\t"));
        }
        out
    }
}

fn hex(v: f64) -> String {
    fx::to_hex(v)
}

/// Vektor pembangkit bilangan acak. Operasi bilangan bulat, wajib identik.
fn rng_vectors() -> VectorFile {
    let mut file = VectorFile::new(
        "rng.tsv",
        "BitExact",
        "SplitMix64: benih, indeks, keluaran u64, dan pecahan [0,1)",
        vec!["seed", "index", "next_u64_hex", "next_f64_hex"],
    );
    for seed in [0u64, 1, 42, 2026, 0x5EED, u64::MAX] {
        let mut a = SplitMix64::new(seed);
        let mut b = SplitMix64::new(seed);
        for index in 0..12u32 {
            let u = a.next_u64();
            let f = b.next_f64();
            file.push(vec![
                seed.to_string(),
                index.to_string(),
                format!("{u:016x}"),
                hex(f),
            ]);
        }
    }
    file
}

/// Vektor certainty factor. Murni penjumlahan, pengurangan, perkalian, pembagian.
fn certainty_vectors() -> VectorFile {
    use ai_core::certainty::*;
    let mut file = VectorFile::new(
        "certainty.tsv",
        "BitExact",
        "Certainty factor MYCIN: kombinasi paralel, berantai, premis AND dan OR",
        vec!["op", "a_hex", "b_hex", "result_hex"],
    );

    let nilai = [
        -1.0, -0.92, -0.75, -0.5, -0.3, -0.01, 0.0, 0.01, 0.3, 0.5, 0.79, 0.92, 1.0,
    ];
    for a in nilai {
        for b in nilai {
            if let Ok(r) = combine_parallel(a, b) {
                file.push(vec!["parallel".into(), hex(a), hex(b), hex(r)]);
            }
            if let Ok(r) = combine_sequential(a, b) {
                file.push(vec!["sequential".into(), hex(a), hex(b), hex(r)]);
            }
            if let Ok(r) = combine_and(&[a, b]) {
                file.push(vec!["and".into(), hex(a), hex(b), hex(r)]);
            }
            if let Ok(r) = combine_or(&[a, b]) {
                file.push(vec!["or".into(), hex(a), hex(b), hex(r)]);
            }
        }
    }

    // Kasus dari lembar tugas mata kuliah.
    for (mb, md) in [(0.8, 0.01), (0.3, 0.0), (0.6, 0.1), (0.0, 1.0)] {
        if let Ok(r) = cf_from_mb_md(mb, md) {
            file.push(vec!["mb_md".into(), hex(mb), hex(md), hex(r)]);
        }
    }
    file
}

/// Vektor Bayesian. Murni aritmetika dasar.
fn bayes_vectors() -> VectorFile {
    use ai_core::bayes::*;
    let mut file = VectorFile::new(
        "bayes.tsv",
        "BitExact",
        "Teorema Bayes dua hipotesis: probabilitas bukti, posterior, rasio kemungkinan",
        vec![
            "prior_hex",
            "likelihood_h_hex",
            "likelihood_not_h_hex",
            "evidence_hex",
            "posterior_hex",
            "likelihood_ratio_hex",
        ],
    );
    let peluang = [0.001, 0.02, 0.1, 0.2, 0.4, 0.5, 0.75, 0.9, 0.99];
    for prior in peluang {
        for lh in peluang {
            for lnh in peluang {
                if let Ok(r) = binary_traced(prior, lh, lnh) {
                    file.push(vec![
                        hex(prior),
                        hex(lh),
                        hex(lnh),
                        hex(r.evidence),
                        hex(r.posterior),
                        hex(r.likelihood_ratio),
                    ]);
                }
            }
        }
    }
    file
}

/// Vektor keanggotaan fuzzy berbentuk garis lurus. Bit-eksak.
fn fuzzy_linear_vectors() -> VectorFile {
    use ai_core::fuzzy::Membership;
    let mut file = VectorFile::new(
        "fuzzy_linear.tsv",
        "BitExact",
        "Fungsi keanggotaan segitiga dan trapesium, seluruhnya aritmetika dasar",
        vec![
            "shape",
            "p1_hex",
            "p2_hex",
            "p3_hex",
            "p4_hex",
            "x_hex",
            "degree_hex",
        ],
    );

    let segitiga = [
        (0.0, 5.0, 10.0),
        (5.0, 5.0, 10.0),
        (0.0, 5.0, 5.0),
        (-3.0, 1.5, 7.0),
    ];
    let trapesium = [
        (0.0, 2.0, 8.0, 10.0),
        (0.0, 0.0, 2.0, 5.0),
        (5.0, 8.0, 10.0, 10.0),
        (0.0, 0.0, 5.0, 5.0),
    ];

    let mut x = -4.0f64;
    while x <= 12.0 {
        for (a, b, c) in segitiga {
            let m = Membership::Triangular { a, b, c };
            file.push(vec![
                "triangular".into(),
                hex(a),
                hex(b),
                hex(c),
                hex(f64::NAN),
                hex(x),
                hex(m.degree(x)),
            ]);
        }
        for (a, b, c, d) in trapesium {
            let m = Membership::Trapezoidal { a, b, c, d };
            file.push(vec![
                "trapezoidal".into(),
                hex(a),
                hex(b),
                hex(c),
                hex(d),
                hex(x),
                hex(m.degree(x)),
            ]);
        }
        x += 0.25;
    }
    file
}

/// Vektor keanggotaan fuzzy yang menyentuh `exp`. Toleransi beberapa ULP.
fn fuzzy_transcendental_vectors() -> VectorFile {
    use ai_core::fuzzy::Membership;
    let mut file = VectorFile::new(
        "fuzzy_transcendental.tsv",
        "NearlyEqual(4)",
        "Keanggotaan Gauss dan sigmoid; memakai exp, yang tidak diwajibkan IEEE-754 tepat",
        vec!["shape", "p1_hex", "p2_hex", "x_hex", "degree_hex"],
    );
    let gauss = [(5.0, 2.0), (0.0, 1.0), (-2.0, 0.5)];
    let sigmoid = [(2.0, 5.0), (-1.5, 0.0), (0.5, 3.0)];

    let mut x = -6.0f64;
    while x <= 12.0 {
        for (mean, sigma) in gauss {
            let m = Membership::Gaussian { mean, sigma };
            file.push(vec![
                "gaussian".into(),
                hex(mean),
                hex(sigma),
                hex(x),
                hex(m.degree(x)),
            ]);
        }
        for (a, c) in sigmoid {
            let m = Membership::Sigmoid { a, c };
            file.push(vec![
                "sigmoid".into(),
                hex(a),
                hex(c),
                hex(x),
                hex(m.degree(x)),
            ]);
        }
        x += 0.5;
    }
    file
}

/// Vektor jarak dan ketakmurnian Gini. Bit-eksak.
fn ml_exact_vectors() -> VectorFile {
    use ai_core::ml::{gini, Distance};
    let mut file = VectorFile::new(
        "ml_exact.tsv",
        "BitExact",
        "Ukuran jarak dan ketakmurnian Gini; hanya memakai aritmetika dasar dan akar kuadrat",
        vec!["op", "arg1", "arg2", "arg3", "arg4", "result_hex"],
    );

    let titik = [
        ([0.0, 0.0], [3.0, 4.0]),
        ([1.5, -2.25], [-0.5, 3.75]),
        ([1e8, 1.0], [1.0, 1e8]),
        ([0.1, 0.2], [0.3, 0.4]),
    ];
    for (a, b) in titik {
        for (nama, d) in [
            ("euclidean", Distance::Euclidean),
            ("manhattan", Distance::Manhattan),
            ("chebyshev", Distance::Chebyshev),
        ] {
            file.push(vec![
                nama.into(),
                hex(a[0]),
                hex(a[1]),
                hex(b[0]),
                hex(b[1]),
                hex(d.between(&a, &b)),
            ]);
        }
    }

    // Gini hanya memakai perkalian dan pengurangan, jadi wajib identik.
    let komposisi: [&[&str]; 6] = [
        &["A"],
        &["A", "B"],
        &["A", "A", "B"],
        &["A", "A", "A", "B"],
        &["A", "B", "C"],
        &["A", "A", "B", "B", "C", "C"],
    ];
    for labels in komposisi {
        let owned: Vec<String> = labels.iter().map(|v| v.to_string()).collect();
        file.push(vec![
            "gini".into(),
            labels.join(",").to_string(),
            String::new(),
            String::new(),
            String::new(),
            hex(gini(&owned)),
        ]);
    }
    file
}

/// Vektor entropi. Memakai logaritma, jadi toleransi beberapa ULP.
fn ml_entropy_vectors() -> VectorFile {
    use ai_core::ml::entropy;
    let mut file = VectorFile::new(
        "ml_entropy.tsv",
        "NearlyEqual(4)",
        "Entropi Shannon; memakai log2, yang tidak diwajibkan IEEE-754 tepat",
        vec!["op", "labels", "values", "result_hex"],
    );

    let komposisi: [&[&str]; 7] = [
        &["A"],
        &["A", "B"],
        &["A", "A", "B", "B"],
        &["A", "A", "A", "B"],
        &["A", "B", "C", "D"],
        &["Ya", "Tidak", "Ya", "Ya", "Tidak"],
        &[
            "Tidak", "Tidak", "Ya", "Ya", "Ya", "Tidak", "Ya", "Tidak", "Ya", "Ya", "Ya", "Ya",
            "Ya", "Tidak",
        ],
    ];
    for labels in komposisi {
        let owned: Vec<String> = labels.iter().map(|v| v.to_string()).collect();
        file.push(vec![
            "entropy".into(),
            labels.join(","),
            String::new(),
            hex(entropy(&owned)),
        ]);
    }

    file
}

/// Vektor perolehan informasi, terpisah dari entropi karena tingkat
/// keterbandingannya berbeda.
///
/// `gain = H(sebelum) - H(sesudah)` adalah selisih dua besaran yang hampir
/// sama, sehingga galat pada `H` membesar pada hasilnya. Kolom `scale_hex`
/// memuat `H(sebelum)`, yaitu skala tempat aritmetikanya sesungguhnya
/// terjadi; toleransinya diukur di sana, bukan pada hasilnya.
fn ml_gain_vectors() -> VectorFile {
    use ai_core::ml::{entropy, information_gain};
    let mut file = VectorFile::new(
        "ml_gain.tsv",
        "CancellingDifference(4)",
        "Perolehan informasi; selisih dua entropi yang hampir sama besar",
        vec!["op", "labels", "values", "scale_hex", "result_hex"],
    );

    // Perolehan informasi pada dataset tenis klasik.
    let y: Vec<String> = [
        "Tidak", "Tidak", "Ya", "Ya", "Ya", "Tidak", "Ya", "Tidak", "Ya", "Ya", "Ya", "Ya", "Ya",
        "Tidak",
    ]
    .iter()
    .map(|v| v.to_string())
    .collect();
    let atribut: [(&str, [&str; 14]); 4] = [
        (
            "cuaca",
            [
                "Cerah", "Cerah", "Mendung", "Hujan", "Hujan", "Hujan", "Mendung", "Cerah",
                "Cerah", "Hujan", "Cerah", "Mendung", "Mendung", "Hujan",
            ],
        ),
        (
            "suhu",
            [
                "Panas", "Panas", "Panas", "Sejuk", "Dingin", "Dingin", "Dingin", "Sejuk",
                "Dingin", "Sejuk", "Sejuk", "Sejuk", "Panas", "Sejuk",
            ],
        ),
        (
            "kelembapan",
            [
                "Tinggi", "Tinggi", "Tinggi", "Tinggi", "Normal", "Normal", "Normal", "Tinggi",
                "Normal", "Normal", "Normal", "Tinggi", "Normal", "Tinggi",
            ],
        ),
        (
            "angin",
            [
                "Lemah", "Kuat", "Lemah", "Lemah", "Lemah", "Kuat", "Kuat", "Lemah", "Lemah",
                "Lemah", "Kuat", "Kuat", "Lemah", "Kuat",
            ],
        ),
    ];
    let skala = entropy(&y);
    for (nama, nilai) in atribut {
        let values: Vec<String> = nilai.iter().map(|v| v.to_string()).collect();
        file.push(vec![
            "information_gain".into(),
            y.join(","),
            format!("{nama}={}", values.join(",")),
            hex(skala),
            hex(information_gain(&values, &y)),
        ]);
    }
    file
}

/// Vektor pertukaran pecahan. Memeriksa bahwa pembacaan hex sepadan.
fn fx_vectors() -> VectorFile {
    let mut file = VectorFile::new(
        "fx.tsv",
        "BitExact",
        "Pola bit pecahan yang menuntut ketelitian penuh, termasuk nilai batas",
        vec!["label", "hex", "decimal_text"],
    );
    let nilai: [(&str, f64); 14] = [
        ("nol", 0.0),
        ("nol_negatif", -0.0),
        ("satu", 1.0),
        ("sepersepuluh", 0.1),
        ("nol_koma_empat_dua", 0.42),
        ("hasil_bayes_hoaks", 0.9 * 0.2 + 0.3 * 0.8),
        ("pi", core::f64::consts::PI),
        ("terkecil_positif", f64::MIN_POSITIVE),
        ("terbesar", f64::MAX),
        ("terkecil", f64::MIN),
        ("subnormal", f64::from_bits(1)),
        ("takhingga", f64::INFINITY),
        ("takhingga_negatif", f64::NEG_INFINITY),
        ("sepertiga", 1.0 / 3.0),
    ];
    for (label, v) in nilai {
        file.push(vec![label.into(), hex(v), format!("{v:?}")]);
    }
    file
}

fn main() -> std::io::Result<()> {
    let dir = std::path::Path::new("tools/conform/vectors");
    std::fs::create_dir_all(dir)?;

    let files = [
        rng_vectors(),
        certainty_vectors(),
        bayes_vectors(),
        fuzzy_linear_vectors(),
        fuzzy_transcendental_vectors(),
        ml_exact_vectors(),
        ml_entropy_vectors(),
        ml_gain_vectors(),
        fx_vectors(),
    ];

    let mut total = 0usize;
    for file in &files {
        let path = dir.join(file.name);
        std::fs::write(&path, file.render())?;
        total += file.rows.len();
        println!(
            "{:<28} {:>6} baris  [{}]",
            file.name,
            file.rows.len(),
            file.comparability
        );
    }
    println!("\nTotal {total} vektor uji ditulis ke {}", dir.display());
    Ok(())
}
