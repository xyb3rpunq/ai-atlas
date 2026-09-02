//! Sesi 5 & 6 — Logika Fuzzy.
//!
//! Fungsi keanggotaan, operasi himpunan kabur, dan tiga mesin inferensi:
//! Mamdani, Sugeno, dan Tsukamoto. Defuzzifikasi tersedia dalam lima metode
//! yang diajarkan di modul: centroid, bisector, *mean of maximum*, *smallest
//! of maximum*, dan *largest of maximum*.
//!
//! Rujukan konseptual: Zadeh, L. A. (1965) dan Zadeh, L. A. (2008),
//! *Is there a need for fuzzy logic?*, Information Sciences 178(13).

use serde::{Deserialize, Serialize};

/// Batas toleransi perbandingan bilangan pecahan di modul ini.
pub const EPS: f64 = 1e-9;

/// Kesalahan pada perhitungan fuzzy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FuzzyError {
    /// Titik-titik fungsi keanggotaan tidak menaik.
    UnorderedPoints(String),
    /// Rentang semesta pembicaraan tidak sah.
    BadUniverse {
        /// Batas bawah.
        min: f64,
        /// Batas atas.
        max: f64,
    },
    /// Jumlah cuplikan terlalu sedikit untuk menghitung.
    TooFewSamples(usize),
    /// Tidak ada aturan yang menyala, sehingga keluaran tidak terdefinisi.
    NoRuleFired,
    /// Basis aturan kosong.
    EmptyRuleBase,
    /// Nama himpunan tidak ditemukan pada variabel.
    UnknownSet(String),
    /// Nama variabel tidak ditemukan.
    UnknownVariable(String),
    /// Derajat keanggotaan di luar rentang `[0, 1]`.
    DegreeOutOfRange(f64),
}

impl core::fmt::Display for FuzzyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FuzzyError::UnorderedPoints(s) => write!(f, "titik tidak terurut menaik: {s}"),
            FuzzyError::BadUniverse { min, max } => {
                write!(
                    f,
                    "semesta tidak sah: min {min} tidak kurang dari max {max}"
                )
            }
            FuzzyError::TooFewSamples(n) => write!(f, "butuh minimal 2 cuplikan, diberi {n}"),
            FuzzyError::NoRuleFired => write!(f, "tidak ada aturan yang menyala"),
            FuzzyError::EmptyRuleBase => write!(f, "basis aturan kosong"),
            FuzzyError::UnknownSet(s) => write!(f, "himpunan tidak dikenal: {s}"),
            FuzzyError::UnknownVariable(s) => write!(f, "variabel tidak dikenal: {s}"),
            FuzzyError::DegreeOutOfRange(v) => {
                write!(f, "derajat keanggotaan harus di [0,1], diberi {v}")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Fungsi keanggotaan
// ---------------------------------------------------------------------------

/// Bentuk fungsi keanggotaan.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Membership {
    /// Segitiga dengan kaki kiri `a`, puncak `b`, kaki kanan `c`.
    Triangular {
        /// Kaki kiri.
        a: f64,
        /// Puncak.
        b: f64,
        /// Kaki kanan.
        c: f64,
    },
    /// Trapesium dengan kaki `a`..`d` dan bahu datar `b`..`c`.
    Trapezoidal {
        /// Kaki kiri.
        a: f64,
        /// Awal bahu.
        b: f64,
        /// Akhir bahu.
        c: f64,
        /// Kaki kanan.
        d: f64,
    },
    /// Kurva Gauss dengan pusat `mean` dan lebar `sigma`.
    Gaussian {
        /// Pusat kurva.
        mean: f64,
        /// Lebar kurva; nilai nol diperlakukan sebagai sangat kecil.
        sigma: f64,
    },
    /// Kurva sigmoid; `a` mengatur kecuraman, `c` titik tengah.
    Sigmoid {
        /// Kecuraman. Negatif membalik arah kurva.
        a: f64,
        /// Titik tengah.
        c: f64,
    },
    /// Kurva-S menaik dari `a` ke `b`.
    SCurve {
        /// Awal kenaikan.
        a: f64,
        /// Akhir kenaikan.
        b: f64,
    },
    /// Kurva-Z menurun dari `a` ke `b`.
    ZCurve {
        /// Awal penurunan.
        a: f64,
        /// Akhir penurunan.
        b: f64,
    },
}

impl Membership {
    /// Memastikan titik-titiknya terurut menaik.
    pub fn validate(&self) -> Result<(), FuzzyError> {
        let ordered = |vals: &[f64], name: &str| -> Result<(), FuzzyError> {
            if vals.windows(2).all(|w| w[0] <= w[1] + EPS) {
                Ok(())
            } else {
                Err(FuzzyError::UnorderedPoints(format!("{name} {vals:?}")))
            }
        };
        match *self {
            Membership::Triangular { a, b, c } => ordered(&[a, b, c], "segitiga"),
            Membership::Trapezoidal { a, b, c, d } => ordered(&[a, b, c, d], "trapesium"),
            Membership::SCurve { a, b } => ordered(&[a, b], "kurva-S"),
            Membership::ZCurve { a, b } => ordered(&[a, b], "kurva-Z"),
            Membership::Gaussian { .. } | Membership::Sigmoid { .. } => Ok(()),
        }
    }

    /// Derajat keanggotaan `x` pada himpunan ini, selalu di rentang `[0, 1]`.
    pub fn degree(&self, x: f64) -> f64 {
        let v = match *self {
            Membership::Triangular { a, b, c } => {
                // Puncak diperiksa lebih dulu. Kalau tidak, segitiga berkaki
                // berimpit (a == b, atau b == c) akan menghasilkan nol tepat di
                // puncaknya — bentuk yang justru lazim dipakai di tepi semesta.
                if (x - b).abs() < EPS {
                    1.0
                } else if x <= a || x >= c {
                    0.0
                } else if x < b {
                    if (b - a).abs() < EPS {
                        1.0
                    } else {
                        (x - a) / (b - a)
                    }
                } else if (c - b).abs() < EPS {
                    1.0
                } else {
                    (c - x) / (c - b)
                }
            }
            Membership::Trapezoidal { a, b, c, d } => {
                // Bahu datar diperiksa lebih dulu, dengan alasan yang sama:
                // trapesium bahu seperti (5, 8, 10, 10) harus bernilai satu di
                // x = 10, bukan nol.
                if x >= b && x <= c {
                    1.0
                } else if x <= a || x >= d {
                    0.0
                } else if x < b {
                    if (b - a).abs() < EPS {
                        1.0
                    } else {
                        (x - a) / (b - a)
                    }
                } else if (d - c).abs() < EPS {
                    1.0
                } else {
                    (d - x) / (d - c)
                }
            }
            Membership::Gaussian { mean, sigma } => {
                let s = if sigma.abs() < EPS { EPS } else { sigma.abs() };
                let z = (x - mean) / s;
                (-0.5 * z * z).exp()
            }
            Membership::Sigmoid { a, c } => 1.0 / (1.0 + (-a * (x - c)).exp()),
            Membership::SCurve { a, b } => {
                if x <= a {
                    0.0
                } else if x >= b {
                    1.0
                } else {
                    let mid = (a + b) / 2.0;
                    if x <= mid {
                        2.0 * ((x - a) / (b - a)).powi(2)
                    } else {
                        1.0 - 2.0 * ((b - x) / (b - a)).powi(2)
                    }
                }
            }
            Membership::ZCurve { a, b } => {
                if x <= a {
                    1.0
                } else if x >= b {
                    0.0
                } else {
                    let mid = (a + b) / 2.0;
                    if x <= mid {
                        1.0 - 2.0 * ((x - a) / (b - a)).powi(2)
                    } else {
                        2.0 * ((b - x) / (b - a)).powi(2)
                    }
                }
            }
        };
        if v.is_nan() {
            0.0
        } else {
            v.clamp(0.0, 1.0)
        }
    }

    /// Titik dengan derajat keanggotaan tertinggi, dipakai Tsukamoto dan MOM.
    ///
    /// Untuk bentuk berpuncak datar, yang dikembalikan adalah tengah bahunya.
    pub fn peak(&self) -> f64 {
        match *self {
            Membership::Triangular { b, .. } => b,
            Membership::Trapezoidal { b, c, .. } => (b + c) / 2.0,
            Membership::Gaussian { mean, .. } => mean,
            Membership::Sigmoid { c, .. } => c,
            Membership::SCurve { b, .. } => b,
            Membership::ZCurve { a, .. } => a,
        }
    }

    /// Kebalikan fungsi keanggotaan: nilai `x` yang derajatnya `alpha`.
    ///
    /// Dipakai mesin Tsukamoto, yang mensyaratkan himpunan keluaran monoton.
    /// Untuk bentuk tak monoton, dikembalikan sisi naiknya.
    pub fn inverse(&self, alpha: f64) -> f64 {
        let a1 = alpha.clamp(0.0, 1.0);
        match *self {
            Membership::Triangular { a, b, .. } => a + a1 * (b - a),
            Membership::Trapezoidal { a, b, .. } => a + a1 * (b - a),
            Membership::SCurve { a, b } => {
                // Membalik dua potong parabola kurva-S.
                if a1 <= 0.5 {
                    a + (b - a) * (a1 / 2.0).sqrt()
                } else {
                    b - (b - a) * ((1.0 - a1) / 2.0).sqrt()
                }
            }
            Membership::ZCurve { a, b } => {
                if a1 <= 0.5 {
                    b - (b - a) * (a1 / 2.0).sqrt()
                } else {
                    a + (b - a) * ((1.0 - a1) / 2.0).sqrt()
                }
            }
            Membership::Gaussian { mean, sigma } => {
                let s = if sigma.abs() < EPS { EPS } else { sigma.abs() };
                if a1 <= EPS {
                    mean - 4.0 * s
                } else {
                    mean - s * (-2.0 * a1.ln()).sqrt()
                }
            }
            Membership::Sigmoid { a, c } => {
                if a.abs() < EPS {
                    c
                } else if a1 <= EPS {
                    c - 10.0 / a.abs()
                } else if a1 >= 1.0 - EPS {
                    c + 10.0 / a.abs()
                } else {
                    c + (a1 / (1.0 - a1)).ln() / a
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Operasi himpunan kabur
// ---------------------------------------------------------------------------

/// Irisan dua derajat keanggotaan (t-norm minimum, operator `AND` Zadeh).
pub fn and(a: f64, b: f64) -> f64 {
    a.min(b)
}

/// Gabungan dua derajat keanggotaan (s-norm maksimum, operator `OR` Zadeh).
pub fn or(a: f64, b: f64) -> f64 {
    a.max(b)
}

/// Komplemen sebuah derajat keanggotaan.
pub fn not(a: f64) -> f64 {
    (1.0 - a).clamp(0.0, 1.0)
}

/// Irisan hasil kali (t-norm produk), alternatif yang lebih halus dari minimum.
pub fn and_product(a: f64, b: f64) -> f64 {
    (a * b).clamp(0.0, 1.0)
}

/// Gabungan jumlah probabilistik (s-norm), pasangan dari [`and_product`].
pub fn or_probabilistic(a: f64, b: f64) -> f64 {
    (a + b - a * b).clamp(0.0, 1.0)
}

/// Potongan alfa: himpunan tegas berisi titik yang derajatnya minimal `alpha`.
///
/// Semesta dicuplik seragam sebanyak `samples` titik.
pub fn alpha_cut(
    set: &Membership,
    alpha: f64,
    min: f64,
    max: f64,
    samples: usize,
) -> Result<Vec<f64>, FuzzyError> {
    let grid = sample_universe(min, max, samples)?;
    Ok(grid
        .into_iter()
        .filter(|x| set.degree(*x) >= alpha - EPS)
        .collect())
}

/// Membuat titik cuplikan seragam pada semesta `[min, max]`.
pub fn sample_universe(min: f64, max: f64, samples: usize) -> Result<Vec<f64>, FuzzyError> {
    // Keberhinggaan diperiksa lebih dulu supaya NaN tersaring di sini; setelah
    // itu `>=` sudah cukup dan niatnya terbaca langsung.
    if !min.is_finite() || !max.is_finite() || min >= max {
        return Err(FuzzyError::BadUniverse { min, max });
    }
    if samples < 2 {
        return Err(FuzzyError::TooFewSamples(samples));
    }
    let step = (max - min) / (samples - 1) as f64;
    Ok((0..samples).map(|i| min + step * i as f64).collect())
}

// ---------------------------------------------------------------------------
// Defuzzifikasi
// ---------------------------------------------------------------------------

/// Metode defuzzifikasi yang diajarkan di modul.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Defuzzifier {
    /// Titik berat daerah, metode paling umum.
    Centroid,
    /// Titik yang membagi luas menjadi dua bagian sama besar.
    Bisector,
    /// Rerata dari seluruh titik berderajat maksimum.
    MeanOfMaximum,
    /// Titik terkecil yang berderajat maksimum.
    SmallestOfMaximum,
    /// Titik terbesar yang berderajat maksimum.
    LargestOfMaximum,
}

impl Defuzzifier {
    /// Nama pendek untuk ditampilkan.
    pub fn short_name(self) -> &'static str {
        match self {
            Defuzzifier::Centroid => "Centroid",
            Defuzzifier::Bisector => "Bisector",
            Defuzzifier::MeanOfMaximum => "MOM",
            Defuzzifier::SmallestOfMaximum => "SOM",
            Defuzzifier::LargestOfMaximum => "LOM",
        }
    }
}

/// Menerapkan defuzzifikasi pada kurva keluaran yang sudah tercuplik.
///
/// `xs` dan `ys` harus sepanjang dan `xs` menaik. Bila seluruh derajat nol,
/// dikembalikan [`FuzzyError::NoRuleFired`] alih-alih diam-diam menghasilkan
/// titik tengah semesta — nilai palsu seperti itu justru yang berbahaya.
pub fn defuzzify(method: Defuzzifier, xs: &[f64], ys: &[f64]) -> Result<f64, FuzzyError> {
    if xs.len() < 2 || xs.len() != ys.len() {
        return Err(FuzzyError::TooFewSamples(xs.len().min(ys.len())));
    }
    let total: f64 = ys.iter().sum();
    if total < EPS {
        return Err(FuzzyError::NoRuleFired);
    }

    match method {
        Defuzzifier::Centroid => {
            let num: f64 = xs.iter().zip(ys).map(|(x, y)| x * y).sum();
            Ok(num / total)
        }
        Defuzzifier::Bisector => {
            let half = total / 2.0;
            let mut acc = 0.0;
            for (i, y) in ys.iter().enumerate() {
                acc += y;
                if acc >= half - EPS {
                    return Ok(xs[i]);
                }
            }
            Ok(xs[xs.len() - 1])
        }
        Defuzzifier::MeanOfMaximum
        | Defuzzifier::SmallestOfMaximum
        | Defuzzifier::LargestOfMaximum => {
            let peak = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let at_peak: Vec<f64> = xs
                .iter()
                .zip(ys)
                .filter(|(_, y)| (**y - peak).abs() < EPS)
                .map(|(x, _)| *x)
                .collect();
            if at_peak.is_empty() {
                return Err(FuzzyError::NoRuleFired);
            }
            Ok(match method {
                Defuzzifier::MeanOfMaximum => at_peak.iter().sum::<f64>() / at_peak.len() as f64,
                Defuzzifier::SmallestOfMaximum => at_peak[0],
                _ => at_peak[at_peak.len() - 1],
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Variabel linguistik dan basis aturan
// ---------------------------------------------------------------------------

/// Satu himpunan kabur bernama pada sebuah variabel linguistik.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedSet {
    /// Nama himpunan, mis. `"Dingin"`.
    pub name: String,
    /// Bentuk fungsi keanggotaannya.
    pub membership: Membership,
}

/// Variabel linguistik: sebuah semesta beserta himpunan-himpunan di atasnya.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable {
    /// Nama variabel, mis. `"Suhu"`.
    pub name: String,
    /// Batas bawah semesta pembicaraan.
    pub min: f64,
    /// Batas atas semesta pembicaraan.
    pub max: f64,
    /// Himpunan-himpunan kabur yang didefinisikan pada variabel ini.
    pub sets: Vec<NamedSet>,
}

impl Variable {
    /// Mencari himpunan berdasarkan namanya.
    pub fn set(&self, name: &str) -> Result<&Membership, FuzzyError> {
        self.sets
            .iter()
            .find(|s| s.name == name)
            .map(|s| &s.membership)
            .ok_or_else(|| FuzzyError::UnknownSet(format!("{}.{name}", self.name)))
    }

    /// Derajat keanggotaan sebuah nilai pada seluruh himpunan variabel ini.
    pub fn fuzzify(&self, x: f64) -> Vec<(String, f64)> {
        self.sets
            .iter()
            .map(|s| (s.name.clone(), s.membership.degree(x)))
            .collect()
    }

    /// Memeriksa kesahihan semesta dan seluruh himpunannya.
    pub fn validate(&self) -> Result<(), FuzzyError> {
        if !self.min.is_finite() || !self.max.is_finite() || self.min >= self.max {
            return Err(FuzzyError::BadUniverse {
                min: self.min,
                max: self.max,
            });
        }
        for s in &self.sets {
            s.membership.validate()?;
        }
        Ok(())
    }
}

/// Penghubung antarpremis dalam sebuah aturan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Connective {
    /// Seluruh premis harus terpenuhi; derajatnya diambil minimum.
    And,
    /// Cukup satu premis terpenuhi; derajatnya diambil maksimum.
    Or,
}

/// Satu premis: variabel tertentu bernilai himpunan tertentu.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Antecedent {
    /// Nama variabel masukan.
    pub variable: String,
    /// Nama himpunan pada variabel itu.
    pub set: String,
}

/// Satu aturan `JIKA ... MAKA ...`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    /// Daftar premis.
    pub antecedents: Vec<Antecedent>,
    /// Penghubung antarpremis.
    pub connective: Connective,
    /// Nama himpunan keluaran (Mamdani dan Tsukamoto).
    pub consequent_set: String,
    /// Nilai tetap keluaran (Sugeno orde nol). Diabaikan mesin lain.
    #[serde(default)]
    pub consequent_value: f64,
    /// Bobot aturan, biasanya `1.0`.
    #[serde(default = "one")]
    pub weight: f64,
}

fn one() -> f64 {
    1.0
}

/// Jejak satu aturan setelah dievaluasi.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleTrace {
    /// Nomor urut aturan, mulai dari satu.
    pub index: usize,
    /// Derajat tiap premis.
    pub degrees: Vec<f64>,
    /// Derajat penyalaan aturan setelah penghubung dan bobot.
    pub firing_strength: f64,
    /// Premis aturan, sejajar dengan `degrees`.
    pub antecedents: Vec<Antecedent>,
    /// Penghubung antarpremis.
    pub connective: Connective,
    /// Nama variabel keluaran.
    pub output: String,
    /// Himpunan keluaran yang disimpulkan aturan ini.
    pub consequent_set: String,
}

/// Hasil inferensi lengkap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Inference {
    /// Nilai tegas keluaran.
    pub crisp: f64,
    /// Jejak tiap aturan.
    pub rules: Vec<RuleTrace>,
    /// Titik cuplikan semesta keluaran (kosong pada Sugeno dan Tsukamoto).
    pub xs: Vec<f64>,
    /// Derajat keanggotaan keluaran gabungan pada tiap titik cuplikan.
    pub ys: Vec<f64>,
}

/// Sistem inferensi kabur lengkap.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FuzzySystem {
    /// Variabel-variabel masukan.
    pub inputs: Vec<Variable>,
    /// Variabel keluaran.
    pub output: Variable,
    /// Basis aturan.
    pub rules: Vec<Rule>,
}

/// Banyaknya titik cuplikan bawaan pada semesta keluaran.
pub const DEFAULT_SAMPLES: usize = 201;

impl FuzzySystem {
    /// Memeriksa kesahihan seluruh variabel dan rujukan pada aturan.
    pub fn validate(&self) -> Result<(), FuzzyError> {
        for v in &self.inputs {
            v.validate()?;
        }
        self.output.validate()?;
        if self.rules.is_empty() {
            return Err(FuzzyError::EmptyRuleBase);
        }
        for r in &self.rules {
            for a in &r.antecedents {
                let var = self
                    .inputs
                    .iter()
                    .find(|v| v.name == a.variable)
                    .ok_or_else(|| FuzzyError::UnknownVariable(a.variable.clone()))?;
                var.set(&a.set)?;
            }
        }
        Ok(())
    }

    /// Derajat penyalaan tiap aturan untuk sekumpulan masukan tegas.
    fn firing_strengths(&self, values: &[(String, f64)]) -> Result<Vec<RuleTrace>, FuzzyError> {
        let lookup = |name: &str| -> Result<f64, FuzzyError> {
            values
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, v)| *v)
                .ok_or_else(|| FuzzyError::UnknownVariable(name.to_string()))
        };

        let mut traces = Vec::with_capacity(self.rules.len());
        for (i, rule) in self.rules.iter().enumerate() {
            let mut degrees = Vec::with_capacity(rule.antecedents.len());
            for a in &rule.antecedents {
                let var = self
                    .inputs
                    .iter()
                    .find(|v| v.name == a.variable)
                    .ok_or_else(|| FuzzyError::UnknownVariable(a.variable.clone()))?;
                let set = var.set(&a.set)?;
                let d = set.degree(lookup(&a.variable)?);
                degrees.push(d);
            }
            let combined = match rule.connective {
                Connective::And => degrees.iter().copied().fold(1.0_f64, f64::min),
                Connective::Or => degrees.iter().copied().fold(0.0_f64, f64::max),
            };
            traces.push(RuleTrace {
                index: i + 1,
                degrees,
                firing_strength: (combined * rule.weight).clamp(0.0, 1.0),
                antecedents: rule.antecedents.clone(),
                connective: rule.connective,
                output: self.output.name.clone(),
                consequent_set: rule.consequent_set.clone(),
            });
        }
        Ok(traces)
    }

    /// Inferensi Mamdani: implikasi minimum, agregasi maksimum, lalu defuzzifikasi.
    pub fn infer_mamdani(
        &self,
        values: &[(String, f64)],
        method: Defuzzifier,
        samples: usize,
    ) -> Result<Inference, FuzzyError> {
        self.validate()?;
        let traces = self.firing_strengths(values)?;
        let xs = sample_universe(self.output.min, self.output.max, samples)?;
        let mut ys: Vec<f64> = vec![0.0; xs.len()];

        for (rule, trace) in self.rules.iter().zip(&traces) {
            if trace.firing_strength < EPS {
                continue;
            }
            let set = self.output.set(&rule.consequent_set)?;
            for (j, x) in xs.iter().enumerate() {
                // Implikasi Mamdani memotong himpunan keluaran pada derajat penyalaan.
                let clipped = set.degree(*x).min(trace.firing_strength);
                ys[j] = ys[j].max(clipped);
            }
        }

        let crisp = defuzzify(method, &xs, &ys)?;
        Ok(Inference {
            crisp,
            rules: traces,
            xs,
            ys,
        })
    }

    /// Inferensi Sugeno orde nol: rerata berbobot nilai tetap tiap aturan.
    pub fn infer_sugeno(&self, values: &[(String, f64)]) -> Result<Inference, FuzzyError> {
        self.validate()?;
        let traces = self.firing_strengths(values)?;
        let total: f64 = traces.iter().map(|t| t.firing_strength).sum();
        if total < EPS {
            return Err(FuzzyError::NoRuleFired);
        }
        let num: f64 = self
            .rules
            .iter()
            .zip(&traces)
            .map(|(r, t)| t.firing_strength * r.consequent_value)
            .sum();
        Ok(Inference {
            crisp: num / total,
            rules: traces,
            xs: Vec::new(),
            ys: Vec::new(),
        })
    }

    /// Inferensi Tsukamoto: tiap aturan menghasilkan nilai lewat fungsi
    /// keanggotaan keluaran yang monoton, lalu dirata-ratakan berbobot.
    pub fn infer_tsukamoto(&self, values: &[(String, f64)]) -> Result<Inference, FuzzyError> {
        self.validate()?;
        let traces = self.firing_strengths(values)?;
        let total: f64 = traces.iter().map(|t| t.firing_strength).sum();
        if total < EPS {
            return Err(FuzzyError::NoRuleFired);
        }
        let mut num = 0.0;
        for (rule, trace) in self.rules.iter().zip(&traces) {
            let set = self.output.set(&rule.consequent_set)?;
            num += trace.firing_strength * set.inverse(trace.firing_strength);
        }
        Ok(Inference {
            crisp: num / total,
            rules: traces,
            xs: Vec::new(),
            ys: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    fn near(a: f64, b: f64, tol: f64) {
        assert!((a - b).abs() < tol, "{a} != {b} (toleransi {tol})");
    }

    // ---------------------------------------------------- fungsi keanggotaan

    #[test]
    fn segitiga_pada_titik_penting() {
        let m = Membership::Triangular {
            a: 0.0,
            b: 5.0,
            c: 10.0,
        };
        close(m.degree(0.0), 0.0);
        close(m.degree(2.5), 0.5);
        close(m.degree(5.0), 1.0);
        close(m.degree(7.5), 0.5);
        close(m.degree(10.0), 0.0);
        close(m.degree(-1.0), 0.0);
        close(m.degree(11.0), 0.0);
    }

    #[test]
    fn segitiga_dengan_kaki_berimpit_tidak_membagi_nol() {
        let kiri = Membership::Triangular {
            a: 5.0,
            b: 5.0,
            c: 10.0,
        };
        // Kaki kiri tegak: derajatnya penuh tepat di titik puncak, bukan nol.
        close(kiri.degree(5.0), 1.0);
        close(kiri.degree(7.5), 0.5);
        close(kiri.degree(4.9), 0.0);
        let kanan = Membership::Triangular {
            a: 0.0,
            b: 5.0,
            c: 5.0,
        };
        close(kanan.degree(5.0), 1.0);
        close(kanan.degree(2.5), 0.5);
        close(kanan.degree(5.1), 0.0);
    }

    #[test]
    fn trapesium_punya_bahu_datar() {
        let m = Membership::Trapezoidal {
            a: 0.0,
            b: 2.0,
            c: 8.0,
            d: 10.0,
        };
        close(m.degree(0.0), 0.0);
        close(m.degree(1.0), 0.5);
        close(m.degree(2.0), 1.0);
        close(m.degree(5.0), 1.0);
        close(m.degree(8.0), 1.0);
        close(m.degree(9.0), 0.5);
        close(m.degree(10.0), 0.0);
    }

    #[test]
    fn trapesium_dengan_kaki_tegak() {
        let m = Membership::Trapezoidal {
            a: 0.0,
            b: 0.0,
            c: 5.0,
            d: 5.0,
        };
        close(m.degree(2.5), 1.0);
        // Kedua kaki tegak: seluruh rentang [0, 5] berderajat penuh, termasuk
        // kedua ujungnya. Ini bentuk yang dipakai untuk himpunan paling kiri
        // dan paling kanan pada sebuah semesta.
        close(m.degree(0.0), 1.0);
        close(m.degree(5.0), 1.0);
        close(m.degree(-0.1), 0.0);
        close(m.degree(5.1), 0.0);
    }

    #[test]
    fn himpunan_bahu_bernilai_penuh_di_tepi_semesta() {
        // Regresi: pemeriksaan tepi pernah jalan sebelum pemeriksaan bahu datar,
        // sehingga trapesium (5, 8, 10, 10) menghasilkan nol tepat di x = 10 —
        // membuat seluruh basis aturan mati di ujung atas semesta.
        let kanan = Membership::Trapezoidal {
            a: 5.0,
            b: 8.0,
            c: 10.0,
            d: 10.0,
        };
        close(kanan.degree(10.0), 1.0);
        close(kanan.degree(9.0), 1.0);
        close(kanan.degree(8.0), 1.0);
        close(kanan.degree(5.0), 0.0);

        let kiri = Membership::Trapezoidal {
            a: 0.0,
            b: 0.0,
            c: 2.0,
            d: 5.0,
        };
        close(kiri.degree(0.0), 1.0);
        close(kiri.degree(2.0), 1.0);
        close(kiri.degree(5.0), 0.0);
    }

    #[test]
    fn gauss_memuncak_di_reratanya() {
        let m = Membership::Gaussian {
            mean: 5.0,
            sigma: 2.0,
        };
        close(m.degree(5.0), 1.0);
        close(m.degree(3.0), m.degree(7.0));
        assert!(m.degree(3.0) < 1.0);
        assert!(m.degree(50.0) < 1e-9);
    }

    #[test]
    fn gauss_dengan_sigma_nol_tidak_menghasilkan_nan() {
        let m = Membership::Gaussian {
            mean: 5.0,
            sigma: 0.0,
        };
        assert!(m.degree(5.0).is_finite());
        assert!(m.degree(6.0).is_finite());
    }

    #[test]
    fn sigmoid_setengah_di_titik_tengah() {
        let m = Membership::Sigmoid { a: 2.0, c: 5.0 };
        close(m.degree(5.0), 0.5);
        assert!(m.degree(10.0) > 0.99);
        assert!(m.degree(0.0) < 0.01);
        // Kecuraman negatif membalik arah kurva.
        let n = Membership::Sigmoid { a: -2.0, c: 5.0 };
        assert!(n.degree(10.0) < 0.01);
    }

    #[test]
    fn kurva_s_dan_z_saling_melengkapi() {
        let s = Membership::SCurve { a: 0.0, b: 10.0 };
        let z = Membership::ZCurve { a: 0.0, b: 10.0 };
        for x in [0.0, 2.5, 5.0, 7.5, 10.0] {
            close(s.degree(x) + z.degree(x), 1.0);
        }
        close(s.degree(0.0), 0.0);
        close(s.degree(10.0), 1.0);
        close(s.degree(5.0), 0.5);
    }

    #[test]
    fn semua_bentuk_selalu_di_rentang_nol_satu() {
        let bentuk = [
            Membership::Triangular {
                a: 0.0,
                b: 5.0,
                c: 10.0,
            },
            Membership::Trapezoidal {
                a: 0.0,
                b: 2.0,
                c: 8.0,
                d: 10.0,
            },
            Membership::Gaussian {
                mean: 5.0,
                sigma: 2.0,
            },
            Membership::Sigmoid { a: 1.5, c: 5.0 },
            Membership::SCurve { a: 0.0, b: 10.0 },
            Membership::ZCurve { a: 0.0, b: 10.0 },
        ];
        for m in bentuk {
            for i in -50..150 {
                let x = i as f64 / 10.0;
                let d = m.degree(x);
                assert!((0.0..=1.0).contains(&d), "{m:?} pada {x} -> {d}");
            }
        }
    }

    #[test]
    fn validasi_menolak_titik_tidak_terurut() {
        assert!(Membership::Triangular {
            a: 5.0,
            b: 1.0,
            c: 10.0
        }
        .validate()
        .is_err());
        assert!(Membership::Trapezoidal {
            a: 0.0,
            b: 8.0,
            c: 2.0,
            d: 10.0
        }
        .validate()
        .is_err());
        assert!(Membership::SCurve { a: 9.0, b: 1.0 }.validate().is_err());
        assert!(Membership::ZCurve { a: 9.0, b: 1.0 }.validate().is_err());
        assert!(Membership::Gaussian {
            mean: 0.0,
            sigma: 1.0
        }
        .validate()
        .is_ok());
        assert!(Membership::Sigmoid { a: 1.0, c: 0.0 }.validate().is_ok());
    }

    #[test]
    fn puncak_tiap_bentuk() {
        close(
            Membership::Triangular {
                a: 0.0,
                b: 4.0,
                c: 10.0,
            }
            .peak(),
            4.0,
        );
        close(
            Membership::Trapezoidal {
                a: 0.0,
                b: 2.0,
                c: 8.0,
                d: 10.0,
            }
            .peak(),
            5.0,
        );
        close(
            Membership::Gaussian {
                mean: 3.0,
                sigma: 1.0,
            }
            .peak(),
            3.0,
        );
        close(Membership::Sigmoid { a: 1.0, c: 7.0 }.peak(), 7.0);
        close(Membership::SCurve { a: 0.0, b: 6.0 }.peak(), 6.0);
        close(Membership::ZCurve { a: 2.0, b: 6.0 }.peak(), 2.0);
    }

    #[test]
    fn kebalikan_konsisten_dengan_derajat() {
        let bentuk = [
            Membership::Triangular {
                a: 0.0,
                b: 10.0,
                c: 20.0,
            },
            Membership::SCurve { a: 0.0, b: 10.0 },
            Membership::Gaussian {
                mean: 10.0,
                sigma: 3.0,
            },
            Membership::Sigmoid { a: 1.0, c: 5.0 },
        ];
        for m in bentuk {
            for alpha in [0.1, 0.25, 0.5, 0.75, 0.9] {
                let x = m.inverse(alpha);
                near(m.degree(x), alpha, 1e-6);
            }
        }
    }

    #[test]
    fn kebalikan_pada_batas_tetap_berhingga() {
        let bentuk = [
            Membership::Triangular {
                a: 0.0,
                b: 10.0,
                c: 20.0,
            },
            Membership::Trapezoidal {
                a: 0.0,
                b: 5.0,
                c: 15.0,
                d: 20.0,
            },
            Membership::SCurve { a: 0.0, b: 10.0 },
            Membership::ZCurve { a: 0.0, b: 10.0 },
            Membership::Gaussian {
                mean: 5.0,
                sigma: 1.0,
            },
            Membership::Sigmoid { a: 2.0, c: 5.0 },
            Membership::Sigmoid { a: 0.0, c: 5.0 },
        ];
        for m in bentuk {
            for alpha in [-1.0, 0.0, 1.0, 2.0] {
                assert!(m.inverse(alpha).is_finite(), "{m:?} pada alfa {alpha}");
            }
        }
    }

    // -------------------------------------------------------------- operasi

    #[test]
    fn operasi_zadeh() {
        close(and(0.7, 0.3), 0.3);
        close(or(0.7, 0.3), 0.7);
        close(not(0.3), 0.7);
        close(not(0.0), 1.0);
        close(not(1.0), 0.0);
    }

    #[test]
    fn operasi_hasil_kali() {
        close(and_product(0.5, 0.4), 0.2);
        close(or_probabilistic(0.5, 0.4), 0.7);
        // Hukum De Morgan berlaku untuk pasangan produk dan jumlah probabilistik.
        for (a, b) in [(0.2, 0.8), (0.5, 0.5), (0.1, 0.9)] {
            close(not(and_product(a, b)), or_probabilistic(not(a), not(b)));
        }
    }

    #[test]
    fn operasi_zadeh_memenuhi_de_morgan() {
        for (a, b) in [(0.2, 0.8), (0.5, 0.5), (0.0, 1.0)] {
            close(not(and(a, b)), or(not(a), not(b)));
            close(not(or(a, b)), and(not(a), not(b)));
        }
    }

    #[test]
    fn cuplikan_semesta() {
        let xs = sample_universe(0.0, 10.0, 11).unwrap();
        assert_eq!(xs.len(), 11);
        close(xs[0], 0.0);
        close(xs[10], 10.0);
        close(xs[5], 5.0);
    }

    #[test]
    fn cuplikan_semesta_menolak_masukan_tak_sah() {
        assert!(matches!(
            sample_universe(10.0, 0.0, 5),
            Err(FuzzyError::BadUniverse { .. })
        ));
        assert!(matches!(
            sample_universe(0.0, 0.0, 5),
            Err(FuzzyError::BadUniverse { .. })
        ));
        assert_eq!(
            sample_universe(0.0, 10.0, 1),
            Err(FuzzyError::TooFewSamples(1))
        );
        assert!(matches!(
            sample_universe(f64::NAN, 10.0, 5),
            Err(FuzzyError::BadUniverse { .. })
        ));
    }

    #[test]
    fn potongan_alfa() {
        let m = Membership::Triangular {
            a: 0.0,
            b: 5.0,
            c: 10.0,
        };
        let cut = alpha_cut(&m, 0.5, 0.0, 10.0, 101).unwrap();
        assert!(!cut.is_empty());
        // Seluruh titik di dalam potongan memang berderajat minimal 0.5.
        assert!(cut.iter().all(|x| m.degree(*x) >= 0.5 - EPS));
        // Potongan pada alfa 1 hanya memuat puncaknya.
        let puncak = alpha_cut(&m, 1.0, 0.0, 10.0, 101).unwrap();
        assert_eq!(puncak.len(), 1);
        close(puncak[0], 5.0);
    }

    #[test]
    fn potongan_alfa_nol_mencakup_seluruh_semesta() {
        let m = Membership::Triangular {
            a: 0.0,
            b: 5.0,
            c: 10.0,
        };
        assert_eq!(alpha_cut(&m, 0.0, 0.0, 10.0, 51).unwrap().len(), 51);
    }

    // -------------------------------------------------------- defuzzifikasi

    #[test]
    fn centroid_bentuk_simetris_jatuh_di_tengah() {
        let xs = sample_universe(0.0, 10.0, 101).unwrap();
        let m = Membership::Triangular {
            a: 0.0,
            b: 5.0,
            c: 10.0,
        };
        let ys: Vec<f64> = xs.iter().map(|x| m.degree(*x)).collect();
        near(
            defuzzify(Defuzzifier::Centroid, &xs, &ys).unwrap(),
            5.0,
            1e-9,
        );
    }

    #[test]
    fn bisector_membagi_luas_jadi_dua() {
        let xs = sample_universe(0.0, 10.0, 101).unwrap();
        let m = Membership::Triangular {
            a: 0.0,
            b: 5.0,
            c: 10.0,
        };
        let ys: Vec<f64> = xs.iter().map(|x| m.degree(*x)).collect();
        near(
            defuzzify(Defuzzifier::Bisector, &xs, &ys).unwrap(),
            5.0,
            0.2,
        );
    }

    #[test]
    fn mom_som_lom_pada_bahu_datar() {
        let xs = sample_universe(0.0, 10.0, 101).unwrap();
        let m = Membership::Trapezoidal {
            a: 0.0,
            b: 2.0,
            c: 8.0,
            d: 10.0,
        };
        let ys: Vec<f64> = xs.iter().map(|x| m.degree(*x)).collect();
        near(
            defuzzify(Defuzzifier::MeanOfMaximum, &xs, &ys).unwrap(),
            5.0,
            1e-9,
        );
        near(
            defuzzify(Defuzzifier::SmallestOfMaximum, &xs, &ys).unwrap(),
            2.0,
            1e-9,
        );
        near(
            defuzzify(Defuzzifier::LargestOfMaximum, &xs, &ys).unwrap(),
            8.0,
            1e-9,
        );
    }

    #[test]
    fn defuzzifikasi_menolak_kurva_kosong() {
        let xs = sample_universe(0.0, 10.0, 11).unwrap();
        let ys = vec![0.0; 11];
        for m in [
            Defuzzifier::Centroid,
            Defuzzifier::Bisector,
            Defuzzifier::MeanOfMaximum,
            Defuzzifier::SmallestOfMaximum,
            Defuzzifier::LargestOfMaximum,
        ] {
            assert_eq!(defuzzify(m, &xs, &ys), Err(FuzzyError::NoRuleFired));
        }
    }

    #[test]
    fn defuzzifikasi_menolak_panjang_tak_sepadan() {
        assert!(matches!(
            defuzzify(Defuzzifier::Centroid, &[0.0, 1.0], &[1.0]),
            Err(FuzzyError::TooFewSamples(_))
        ));
        assert!(matches!(
            defuzzify(Defuzzifier::Centroid, &[0.0], &[1.0]),
            Err(FuzzyError::TooFewSamples(_))
        ));
    }

    #[test]
    fn nama_pendek_metode() {
        assert_eq!(Defuzzifier::Centroid.short_name(), "Centroid");
        assert_eq!(Defuzzifier::MeanOfMaximum.short_name(), "MOM");
        assert_eq!(Defuzzifier::SmallestOfMaximum.short_name(), "SOM");
        assert_eq!(Defuzzifier::LargestOfMaximum.short_name(), "LOM");
        assert_eq!(Defuzzifier::Bisector.short_name(), "Bisector");
    }

    // ------------------------------------------------------ sistem lengkap

    /// Sistem tip restoran klasik: pelayanan dan makanan menentukan persenan.
    fn sistem_tip() -> FuzzySystem {
        let pelayanan = Variable {
            name: "Pelayanan".into(),
            min: 0.0,
            max: 10.0,
            sets: vec![
                NamedSet {
                    name: "Buruk".into(),
                    membership: Membership::Trapezoidal {
                        a: 0.0,
                        b: 0.0,
                        c: 2.0,
                        d: 5.0,
                    },
                },
                NamedSet {
                    name: "Baik".into(),
                    membership: Membership::Triangular {
                        a: 0.0,
                        b: 5.0,
                        c: 10.0,
                    },
                },
                NamedSet {
                    name: "Istimewa".into(),
                    membership: Membership::Trapezoidal {
                        a: 5.0,
                        b: 8.0,
                        c: 10.0,
                        d: 10.0,
                    },
                },
            ],
        };
        let makanan = Variable {
            name: "Makanan".into(),
            min: 0.0,
            max: 10.0,
            sets: vec![
                NamedSet {
                    name: "Hambar".into(),
                    membership: Membership::Trapezoidal {
                        a: 0.0,
                        b: 0.0,
                        c: 2.0,
                        d: 5.0,
                    },
                },
                NamedSet {
                    name: "Lezat".into(),
                    membership: Membership::Trapezoidal {
                        a: 5.0,
                        b: 8.0,
                        c: 10.0,
                        d: 10.0,
                    },
                },
            ],
        };
        let tip = Variable {
            name: "Tip".into(),
            min: 0.0,
            max: 25.0,
            sets: vec![
                NamedSet {
                    name: "Sedikit".into(),
                    membership: Membership::Triangular {
                        a: 0.0,
                        b: 5.0,
                        c: 10.0,
                    },
                },
                NamedSet {
                    name: "Sedang".into(),
                    membership: Membership::Triangular {
                        a: 7.5,
                        b: 12.5,
                        c: 17.5,
                    },
                },
                NamedSet {
                    name: "Banyak".into(),
                    membership: Membership::Triangular {
                        a: 15.0,
                        b: 20.0,
                        c: 25.0,
                    },
                },
            ],
        };
        FuzzySystem {
            inputs: vec![pelayanan, makanan],
            output: tip,
            rules: vec![
                Rule {
                    antecedents: vec![
                        Antecedent {
                            variable: "Pelayanan".into(),
                            set: "Buruk".into(),
                        },
                        Antecedent {
                            variable: "Makanan".into(),
                            set: "Hambar".into(),
                        },
                    ],
                    connective: Connective::Or,
                    consequent_set: "Sedikit".into(),
                    consequent_value: 5.0,
                    weight: 1.0,
                },
                Rule {
                    antecedents: vec![Antecedent {
                        variable: "Pelayanan".into(),
                        set: "Baik".into(),
                    }],
                    connective: Connective::And,
                    consequent_set: "Sedang".into(),
                    consequent_value: 12.5,
                    weight: 1.0,
                },
                Rule {
                    antecedents: vec![
                        Antecedent {
                            variable: "Pelayanan".into(),
                            set: "Istimewa".into(),
                        },
                        Antecedent {
                            variable: "Makanan".into(),
                            set: "Lezat".into(),
                        },
                    ],
                    connective: Connective::Or,
                    consequent_set: "Banyak".into(),
                    consequent_value: 20.0,
                    weight: 1.0,
                },
            ],
        }
    }

    #[test]
    fn variabel_fuzzifikasi() {
        let s = sistem_tip();
        let hasil = s.inputs[0].fuzzify(5.0);
        assert_eq!(hasil.len(), 3);
        assert_eq!(hasil[1].0, "Baik");
        close(hasil[1].1, 1.0);
    }

    #[test]
    fn variabel_mencari_himpunan() {
        let s = sistem_tip();
        assert!(s.inputs[0].set("Baik").is_ok());
        assert!(matches!(
            s.inputs[0].set("TidakAda"),
            Err(FuzzyError::UnknownSet(_))
        ));
    }

    #[test]
    fn validasi_sistem_lolos() {
        assert!(sistem_tip().validate().is_ok());
    }

    #[test]
    fn validasi_menolak_basis_aturan_kosong() {
        let mut s = sistem_tip();
        s.rules.clear();
        assert_eq!(s.validate(), Err(FuzzyError::EmptyRuleBase));
    }

    #[test]
    fn validasi_menolak_rujukan_yang_salah() {
        let mut s = sistem_tip();
        s.rules[0].antecedents[0].variable = "Cuaca".into();
        assert!(matches!(s.validate(), Err(FuzzyError::UnknownVariable(_))));

        let mut s2 = sistem_tip();
        s2.rules[0].antecedents[0].set = "Aneh".into();
        assert!(matches!(s2.validate(), Err(FuzzyError::UnknownSet(_))));
    }

    #[test]
    fn validasi_menolak_semesta_terbalik() {
        let mut s = sistem_tip();
        s.output.min = 25.0;
        s.output.max = 0.0;
        assert!(matches!(s.validate(), Err(FuzzyError::BadUniverse { .. })));
    }

    #[test]
    fn validasi_menolak_semesta_bukan_angka() {
        // NaN harus ditolak, bukan lolos diam-diam. Perbandingan `min >= max`
        // saja akan meloloskannya, karena setiap perbandingan dengan NaN
        // bernilai salah.
        for (min, max) in [
            (f64::NAN, 10.0),
            (0.0, f64::NAN),
            (f64::NEG_INFINITY, 10.0),
            (0.0, f64::INFINITY),
        ] {
            let mut s = sistem_tip();
            s.output.min = min;
            s.output.max = max;
            assert!(
                matches!(s.validate(), Err(FuzzyError::BadUniverse { .. })),
                "semesta ({min}, {max}) seharusnya ditolak"
            );
            assert!(
                matches!(
                    sample_universe(min, max, 11),
                    Err(FuzzyError::BadUniverse { .. })
                ),
                "cuplikan ({min}, {max}) seharusnya ditolak"
            );
        }
    }

    #[test]
    fn validasi_menolak_semesta_sama_panjang_nol() {
        let mut s = sistem_tip();
        s.output.min = 5.0;
        s.output.max = 5.0;
        assert!(matches!(s.validate(), Err(FuzzyError::BadUniverse { .. })));
    }

    #[test]
    fn mamdani_pelayanan_buruk_menghasilkan_tip_kecil() {
        let s = sistem_tip();
        let hasil = s
            .infer_mamdani(
                &[("Pelayanan".into(), 1.0), ("Makanan".into(), 1.0)],
                Defuzzifier::Centroid,
                DEFAULT_SAMPLES,
            )
            .unwrap();
        assert!(hasil.crisp < 8.0, "tip {} terlalu besar", hasil.crisp);
        assert_eq!(hasil.rules.len(), 3);
        assert_eq!(hasil.xs.len(), DEFAULT_SAMPLES);
        assert_eq!(hasil.ys.len(), DEFAULT_SAMPLES);
    }

    #[test]
    fn mamdani_pelayanan_istimewa_menghasilkan_tip_besar() {
        let s = sistem_tip();
        let hasil = s
            .infer_mamdani(
                &[("Pelayanan".into(), 10.0), ("Makanan".into(), 10.0)],
                Defuzzifier::Centroid,
                DEFAULT_SAMPLES,
            )
            .unwrap();
        assert!(hasil.crisp > 15.0, "tip {} terlalu kecil", hasil.crisp);
    }

    #[test]
    fn mamdani_monoton_terhadap_kualitas_pelayanan() {
        let s = sistem_tip();
        let mut sebelumnya = f64::NEG_INFINITY;
        for pelayanan in [0.0, 2.5, 5.0, 7.5, 10.0] {
            let hasil = s
                .infer_mamdani(
                    &[
                        ("Pelayanan".into(), pelayanan),
                        ("Makanan".into(), pelayanan),
                    ],
                    Defuzzifier::Centroid,
                    DEFAULT_SAMPLES,
                )
                .unwrap();
            assert!(
                hasil.crisp >= sebelumnya - 1e-6,
                "tip turun di pelayanan {pelayanan}: {} < {sebelumnya}",
                hasil.crisp
            );
            sebelumnya = hasil.crisp;
        }
    }

    #[test]
    fn mamdani_hasilnya_selalu_di_dalam_semesta_keluaran() {
        let s = sistem_tip();
        for p in [0.0, 3.0, 5.0, 7.0, 10.0] {
            for m in [0.0, 5.0, 10.0] {
                let hasil = s.infer_mamdani(
                    &[("Pelayanan".into(), p), ("Makanan".into(), m)],
                    Defuzzifier::Centroid,
                    DEFAULT_SAMPLES,
                );
                if let Ok(h) = hasil {
                    assert!(
                        h.crisp >= s.output.min - EPS && h.crisp <= s.output.max + EPS,
                        "tip {} di luar semesta pada ({p}, {m})",
                        h.crisp
                    );
                }
            }
        }
    }

    #[test]
    fn mamdani_lima_metode_defuzzifikasi_semuanya_berjalan() {
        let s = sistem_tip();
        for metode in [
            Defuzzifier::Centroid,
            Defuzzifier::Bisector,
            Defuzzifier::MeanOfMaximum,
            Defuzzifier::SmallestOfMaximum,
            Defuzzifier::LargestOfMaximum,
        ] {
            let hasil = s
                .infer_mamdani(
                    &[("Pelayanan".into(), 7.0), ("Makanan".into(), 8.0)],
                    metode,
                    DEFAULT_SAMPLES,
                )
                .unwrap();
            assert!(
                hasil.crisp.is_finite(),
                "{metode:?} menghasilkan bukan angka"
            );
        }
    }

    #[test]
    fn mamdani_menolak_variabel_yang_tidak_diberi_nilai() {
        let s = sistem_tip();
        assert!(matches!(
            s.infer_mamdani(
                &[("Pelayanan".into(), 5.0)],
                Defuzzifier::Centroid,
                DEFAULT_SAMPLES
            ),
            Err(FuzzyError::UnknownVariable(_))
        ));
    }

    #[test]
    fn sugeno_rerata_berbobot() {
        let s = sistem_tip();
        let hasil = s
            .infer_sugeno(&[("Pelayanan".into(), 5.0), ("Makanan".into(), 5.0)])
            .unwrap();
        assert!(hasil.crisp.is_finite());
        assert!(hasil.xs.is_empty());
        // Pada pelayanan 5, aturan "Baik" menyala penuh sehingga hasilnya
        // condong ke nilai tetapnya, 12.5.
        assert!(hasil.crisp > 5.0 && hasil.crisp < 20.0);
    }

    #[test]
    fn sugeno_dengan_satu_aturan_menyala_mengembalikan_nilai_tetapnya() {
        let mut s = sistem_tip();
        s.rules = vec![s.rules[1].clone()];
        let hasil = s
            .infer_sugeno(&[("Pelayanan".into(), 5.0), ("Makanan".into(), 5.0)])
            .unwrap();
        close(hasil.crisp, 12.5);
    }

    #[test]
    fn tsukamoto_menghasilkan_nilai_di_dalam_semesta() {
        let s = sistem_tip();
        let hasil = s
            .infer_tsukamoto(&[("Pelayanan".into(), 7.0), ("Makanan".into(), 8.0)])
            .unwrap();
        assert!(hasil.crisp >= s.output.min && hasil.crisp <= s.output.max);
        assert!(hasil.xs.is_empty());
    }

    #[test]
    fn tiga_mesin_sepakat_pada_arah_yang_sama() {
        let s = sistem_tip();
        let buruk = [("Pelayanan".into(), 1.0), ("Makanan".into(), 1.0)];
        let bagus = [("Pelayanan".into(), 9.0), ("Makanan".into(), 9.0)];
        let mam_b = s
            .infer_mamdani(&buruk, Defuzzifier::Centroid, DEFAULT_SAMPLES)
            .unwrap()
            .crisp;
        let mam_g = s
            .infer_mamdani(&bagus, Defuzzifier::Centroid, DEFAULT_SAMPLES)
            .unwrap()
            .crisp;
        let sug_b = s.infer_sugeno(&buruk).unwrap().crisp;
        let sug_g = s.infer_sugeno(&bagus).unwrap().crisp;
        let tsu_b = s.infer_tsukamoto(&buruk).unwrap().crisp;
        let tsu_g = s.infer_tsukamoto(&bagus).unwrap().crisp;
        assert!(mam_g > mam_b, "Mamdani tidak naik");
        assert!(sug_g > sug_b, "Sugeno tidak naik");
        assert!(tsu_g > tsu_b, "Tsukamoto tidak naik");
    }

    #[test]
    fn tidak_ada_aturan_menyala_menghasilkan_error() {
        // Sistem dengan satu aturan yang tidak mungkin menyala pada masukan ini.
        let s = FuzzySystem {
            inputs: vec![Variable {
                name: "X".into(),
                min: 0.0,
                max: 10.0,
                sets: vec![NamedSet {
                    name: "Tinggi".into(),
                    membership: Membership::Triangular {
                        a: 8.0,
                        b: 9.0,
                        c: 10.0,
                    },
                }],
            }],
            output: Variable {
                name: "Y".into(),
                min: 0.0,
                max: 10.0,
                sets: vec![NamedSet {
                    name: "Besar".into(),
                    membership: Membership::Triangular {
                        a: 5.0,
                        b: 8.0,
                        c: 10.0,
                    },
                }],
            },
            rules: vec![Rule {
                antecedents: vec![Antecedent {
                    variable: "X".into(),
                    set: "Tinggi".into(),
                }],
                connective: Connective::And,
                consequent_set: "Besar".into(),
                consequent_value: 9.0,
                weight: 1.0,
            }],
        };
        let masukan = [("X".into(), 0.0)];
        assert_eq!(
            s.infer_mamdani(&masukan, Defuzzifier::Centroid, DEFAULT_SAMPLES),
            Err(FuzzyError::NoRuleFired)
        );
        assert_eq!(s.infer_sugeno(&masukan), Err(FuzzyError::NoRuleFired));
        assert_eq!(s.infer_tsukamoto(&masukan), Err(FuzzyError::NoRuleFired));
    }

    #[test]
    fn bobot_aturan_mempengaruhi_hasil() {
        let mut s = sistem_tip();
        let masukan = [("Pelayanan".into(), 9.0), ("Makanan".into(), 9.0)];
        let penuh = s.infer_sugeno(&masukan).unwrap().crisp;
        s.rules[2].weight = 0.1;
        let dilemahkan = s.infer_sugeno(&masukan).unwrap().crisp;
        assert!(
            dilemahkan < penuh,
            "melemahkan aturan tip besar seharusnya menurunkan hasil"
        );
    }

    #[test]
    fn jejak_aturan_terisi_lengkap() {
        let s = sistem_tip();
        let hasil = s
            .infer_mamdani(
                &[("Pelayanan".into(), 9.0), ("Makanan".into(), 9.0)],
                Defuzzifier::Centroid,
                DEFAULT_SAMPLES,
            )
            .unwrap();
        for (i, t) in hasil.rules.iter().enumerate() {
            assert_eq!(t.index, i + 1);
            assert!((0.0..=1.0).contains(&t.firing_strength));
            assert!(!t.degrees.is_empty());
            // Jejaknya membawa bentuk aturannya, bukan kalimatnya. Kalimat
            // yang dirakit di sini akan selalu berbahasa Indonesia, sedangkan
            // yang membacanya belum tentu.
            assert_eq!(t.degrees.len(), t.antecedents.len());
            assert_eq!(t.output, s.output.name);
            assert!(!t.consequent_set.is_empty());
            assert_eq!(t.antecedents, s.rules[i].antecedents);
            assert_eq!(t.connective, s.rules[i].connective);
        }
        // Aturan pertama memang berpenghubung OR.
        assert_eq!(hasil.rules[0].connective, Connective::Or);
    }

    #[test]
    fn sistem_bisa_di_serialisasi() {
        let s = sistem_tip();
        let json = serde_json::to_string(&s).unwrap();
        let balik: FuzzySystem = serde_json::from_str(&json).unwrap();
        assert_eq!(balik.inputs.len(), s.inputs.len());
        assert_eq!(balik.rules.len(), s.rules.len());
        assert_eq!(balik.output.name, s.output.name);
    }

    #[test]
    fn pesan_error_terbaca() {
        for e in [
            FuzzyError::UnorderedPoints("x".into()),
            FuzzyError::BadUniverse { min: 1.0, max: 0.0 },
            FuzzyError::TooFewSamples(1),
            FuzzyError::NoRuleFired,
            FuzzyError::EmptyRuleBase,
            FuzzyError::UnknownSet("a".into()),
            FuzzyError::UnknownVariable("b".into()),
            FuzzyError::DegreeOutOfRange(2.0),
        ] {
            assert!(!e.to_string().is_empty());
        }
    }
}
