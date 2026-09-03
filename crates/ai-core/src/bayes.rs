//! Sesi 4 — Probabilitas Bayesian.
//!
//! Teorema Bayes, hukum probabilitas total, rasio kemungkinan, serta dua
//! pengklasifikasi Naive Bayes: kategorikal (multinomial dengan penghalusan
//! Laplace) dan Gaussian untuk fitur kontinu.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Batas toleransi perbandingan bilangan pecahan di modul ini.
pub const EPS: f64 = 1e-12;

/// Kesalahan yang bisa terjadi pada perhitungan Bayesian.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BayesError {
    /// Sebuah probabilitas berada di luar rentang `[0, 1]`.
    ProbabilityOutOfRange(String),
    /// Probabilitas bukti `P(E)` bernilai nol sehingga pembagian mustahil.
    ZeroEvidence,
    /// Panjang dua larik yang seharusnya sepadan ternyata berbeda.
    LengthMismatch {
        /// Panjang larik pertama.
        a: usize,
        /// Panjang larik kedua.
        b: usize,
    },
    /// Masukan kosong.
    EmptyInput,
    /// Jumlah probabilitas prior tidak sama dengan satu.
    PriorsDoNotSumToOne(f64),
    /// Indeks hipotesis di luar jangkauan.
    IndexOutOfRange {
        /// Indeks yang diminta.
        index: usize,
        /// Jumlah hipotesis yang tersedia.
        len: usize,
    },
    /// Model dipakai sebelum dilatih.
    NotTrained,
}

impl crate::galat::Dijelaskan for BayesError {
    fn kode(&self) -> &'static str {
        match self {
            BayesError::ProbabilityOutOfRange(_) => "bayes.probabilitas_di_luar_rentang",
            BayesError::ZeroEvidence => "bayes.bukti_nol",
            BayesError::LengthMismatch { .. } => "bayes.panjang_tak_sepadan",
            BayesError::EmptyInput => "bayes.masukan_kosong",
            BayesError::PriorsDoNotSumToOne(_) => "bayes.prior_tak_berjumlah_satu",
            BayesError::IndexOutOfRange { .. } => "bayes.indeks_di_luar_jangkauan",
            BayesError::NotTrained => "bayes.belum_dilatih",
        }
    }

    fn argumen(&self) -> Vec<String> {
        match self {
            BayesError::ProbabilityOutOfRange(v) => vec![v.clone()],
            BayesError::PriorsDoNotSumToOne(v) => vec![v.to_string()],
            BayesError::LengthMismatch { a, b } => vec![a.to_string(), b.to_string()],
            BayesError::IndexOutOfRange { index, len } => {
                vec![index.to_string(), len.to_string()]
            }
            BayesError::ZeroEvidence | BayesError::EmptyInput | BayesError::NotTrained => {
                Vec::new()
            }
        }
    }
}

impl core::fmt::Display for BayesError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BayesError::ProbabilityOutOfRange(v) => {
                write!(f, "probabilitas harus di rentang [0,1], diberi {v}")
            }
            BayesError::ZeroEvidence => write!(f, "P(E) = 0, posterior tidak terdefinisi"),
            BayesError::LengthMismatch { a, b } => {
                write!(f, "panjang larik tidak sepadan: {a} vs {b}")
            }
            BayesError::EmptyInput => write!(f, "masukan kosong"),
            BayesError::PriorsDoNotSumToOne(s) => {
                write!(f, "jumlah prior harus 1, diperoleh {s}")
            }
            BayesError::IndexOutOfRange { index, len } => {
                write!(f, "indeks {index} di luar jangkauan (0..{len})")
            }
            BayesError::NotTrained => write!(f, "model belum dilatih"),
        }
    }
}

fn check_prob(v: f64, name: &str) -> Result<f64, BayesError> {
    if !v.is_finite() || !(-EPS..=1.0 + EPS).contains(&v) {
        return Err(BayesError::ProbabilityOutOfRange(format!("{name}={v}")));
    }
    Ok(v.clamp(0.0, 1.0))
}

/// Teorema Bayes bentuk langsung: `P(H|E) = P(E|H) x P(H) / P(E)`.
pub fn posterior(prior: f64, likelihood: f64, evidence: f64) -> Result<f64, BayesError> {
    let prior = check_prob(prior, "P(H)")?;
    let likelihood = check_prob(likelihood, "P(E|H)")?;
    let evidence = check_prob(evidence, "P(E)")?;
    if evidence < EPS {
        return Err(BayesError::ZeroEvidence);
    }
    Ok((likelihood * prior / evidence).clamp(0.0, 1.0))
}

/// Hukum probabilitas total: `P(E) = sum_i P(E|H_i) x P(H_i)`.
pub fn total_probability(priors: &[f64], likelihoods: &[f64]) -> Result<f64, BayesError> {
    if priors.is_empty() {
        return Err(BayesError::EmptyInput);
    }
    if priors.len() != likelihoods.len() {
        return Err(BayesError::LengthMismatch {
            a: priors.len(),
            b: likelihoods.len(),
        });
    }
    let mut sum = 0.0;
    for (i, (p, l)) in priors.iter().zip(likelihoods).enumerate() {
        let p = check_prob(*p, &format!("P(H{i})"))?;
        let l = check_prob(*l, &format!("P(E|H{i})"))?;
        sum += p * l;
    }
    Ok(sum.clamp(0.0, 1.0))
}

/// Memastikan sekumpulan prior membentuk partisi (jumlahnya satu).
pub fn validate_priors(priors: &[f64]) -> Result<(), BayesError> {
    if priors.is_empty() {
        return Err(BayesError::EmptyInput);
    }
    let mut sum = 0.0;
    for (i, p) in priors.iter().enumerate() {
        sum += check_prob(*p, &format!("P(H{i})"))?;
    }
    if (sum - 1.0).abs() > 1e-6 {
        return Err(BayesError::PriorsDoNotSumToOne(sum));
    }
    Ok(())
}

/// Posterior untuk seluruh hipotesis sekaligus, ternormalisasi.
///
/// Ini bentuk yang dipakai di soal kelas: diberi prior tiap hipotesis dan
/// likelihood gejala pada tiap hipotesis, hitung `P(H_i|E)` untuk semua `i`.
pub fn posterior_all(priors: &[f64], likelihoods: &[f64]) -> Result<Vec<f64>, BayesError> {
    let evidence = total_probability(priors, likelihoods)?;
    if evidence < EPS {
        return Err(BayesError::ZeroEvidence);
    }
    Ok(priors
        .iter()
        .zip(likelihoods)
        .map(|(p, l)| (p * l / evidence).clamp(0.0, 1.0))
        .collect())
}

/// Posterior satu hipotesis tertentu dari daftar prior dan likelihood.
pub fn posterior_at(priors: &[f64], likelihoods: &[f64], index: usize) -> Result<f64, BayesError> {
    if index >= priors.len() {
        return Err(BayesError::IndexOutOfRange {
            index,
            len: priors.len(),
        });
    }
    Ok(posterior_all(priors, likelihoods)?[index])
}

/// Peluang menjadi *odds*: `odds = p / (1 - p)`. Mengembalikan tak hingga bila `p == 1`.
pub fn to_odds(p: f64) -> Result<f64, BayesError> {
    let p = check_prob(p, "p")?;
    if (1.0 - p).abs() < EPS {
        return Ok(f64::INFINITY);
    }
    Ok(p / (1.0 - p))
}

/// *Odds* kembali menjadi peluang: `p = odds / (1 + odds)`.
pub fn from_odds(odds: f64) -> f64 {
    if odds.is_infinite() && odds > 0.0 {
        return 1.0;
    }
    if odds <= 0.0 {
        return 0.0;
    }
    odds / (1.0 + odds)
}

/// Rasio kemungkinan positif: `LR+ = P(E|H) / P(E|~H)`.
pub fn likelihood_ratio(
    likelihood_given_h: f64,
    likelihood_given_not_h: f64,
) -> Result<f64, BayesError> {
    let a = check_prob(likelihood_given_h, "P(E|H)")?;
    let b = check_prob(likelihood_given_not_h, "P(E|~H)")?;
    if b < EPS {
        return Ok(if a < EPS { 0.0 } else { f64::INFINITY });
    }
    Ok(a / b)
}

/// Memformat bilangan untuk **ditampilkan** di dalam teks rumus.
///
/// Hanya menyentuh tampilan. Nilai yang tersimpan di medan `value` tetap
/// presisi penuh, karena itulah yang dibandingkan dengan implementasi Go dan
/// PL/SQL. Tanpa pembulatan ini, `0.9*0.2 + 0.3*0.8` muncul di layar sebagai
/// `0.42000000000000004` dan terbaca seperti cacat, padahal itu justru
/// representasi biner yang benar.
pub fn display_number(v: f64) -> String {
    if !v.is_finite() {
        return if v.is_nan() {
            "—".to_string()
        } else if v > 0.0 {
            "inf".to_string()
        } else {
            "-inf".to_string()
        };
    }
    // Dibulatkan ke 10 desimal, lalu nol di belakang dibuang.
    let mut s = format!("{:.10}", v);
    if s.contains('.') {
        s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

/// Satu langkah perhitungan Bayes, untuk ditampilkan ke pengguna.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BayesStep {
    /// Nama langkah, mis. `"P(E)"`.
    pub label: String,
    /// Rumus dalam bentuk teks siap tampil.
    pub formula: String,
    /// Hasil langkah ini.
    pub value: f64,
}

/// Hasil lengkap perhitungan Bayes dua hipotesis beserta jejak langkahnya.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BayesResult {
    /// `P(H|E)`.
    pub posterior: f64,
    /// `P(~H|E)`.
    pub posterior_complement: f64,
    /// `P(E)`, probabilitas bukti.
    pub evidence: f64,
    /// Rasio kemungkinan positif.
    pub likelihood_ratio: f64,
    /// Jejak perhitungan.
    pub steps: Vec<BayesStep>,
}

/// Kasus dua hipotesis (`H` dan `~H`) lengkap dengan langkah-langkahnya.
///
/// Ini bentuk soal yang paling sering muncul: "diketahui prevalensi, sensitivitas,
/// dan tingkat positif palsu — berapa peluang sebenarnya?".
///
/// ```
/// use ai_core::bayes::binary_traced;
/// // Tugas Pertemuan 5 IND323 (deteksi hoaks):
/// // 20% berita hoaks, 90% hoaks berjudul provokatif, 30% non-hoaks juga provokatif.
/// let r = binary_traced(0.2, 0.9, 0.3).unwrap();
/// assert!((r.evidence - 0.42).abs() < 1e-12);
/// assert!((r.posterior - 3.0 / 7.0).abs() < 1e-12);
/// ```
pub fn binary_traced(
    prior: f64,
    likelihood_given_h: f64,
    likelihood_given_not_h: f64,
) -> Result<BayesResult, BayesError> {
    let p_h = check_prob(prior, "P(H)")?;
    let p_e_h = check_prob(likelihood_given_h, "P(E|H)")?;
    let p_e_nh = check_prob(likelihood_given_not_h, "P(E|~H)")?;
    let p_nh = 1.0 - p_h;

    let evidence = p_h * p_e_h + p_nh * p_e_nh;
    if evidence < EPS {
        return Err(BayesError::ZeroEvidence);
    }
    let post = (p_e_h * p_h / evidence).clamp(0.0, 1.0);
    let post_c = (p_e_nh * p_nh / evidence).clamp(0.0, 1.0);
    let lr = likelihood_ratio(p_e_h, p_e_nh)?;

    let d = display_number;
    let steps = vec![
        BayesStep {
            label: "P(~H)".into(),
            formula: format!("1 - {} = {}", d(p_h), d(p_nh)),
            value: p_nh,
        },
        BayesStep {
            label: "P(E)".into(),
            formula: format!(
                "P(E|H)xP(H) + P(E|~H)xP(~H) = {}x{} + {}x{} = {}",
                d(p_e_h),
                d(p_h),
                d(p_e_nh),
                d(p_nh),
                d(evidence)
            ),
            value: evidence,
        },
        BayesStep {
            label: "P(H|E)".into(),
            formula: format!(
                "P(E|H)xP(H) / P(E) = {}x{} / {} = {}",
                d(p_e_h),
                d(p_h),
                d(evidence),
                d(post)
            ),
            value: post,
        },
        BayesStep {
            label: "P(~H|E)".into(),
            formula: format!(
                "P(E|~H)xP(~H) / P(E) = {}x{} / {} = {}",
                d(p_e_nh),
                d(p_nh),
                d(evidence),
                d(post_c)
            ),
            value: post_c,
        },
        BayesStep {
            label: "LR+".into(),
            formula: format!(
                "P(E|H) / P(E|~H) = {} / {} = {}",
                d(p_e_h),
                d(p_e_nh),
                d(lr)
            ),
            value: lr,
        },
    ];

    Ok(BayesResult {
        posterior: post,
        posterior_complement: post_c,
        evidence,
        likelihood_ratio: lr,
        steps,
    })
}

// ---------------------------------------------------------------------------
// Naive Bayes kategorikal
// ---------------------------------------------------------------------------

/// Satu baris data kategorikal: daftar nilai fitur berbentuk teks dan labelnya.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoricalSample {
    /// Nilai tiap fitur, urutannya harus konsisten antarbaris.
    pub features: Vec<String>,
    /// Label kelas.
    pub label: String,
}

/// Pengklasifikasi Naive Bayes untuk fitur kategorikal, dengan penghalusan Laplace.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CategoricalNaiveBayes {
    /// Jumlah baris latih per kelas.
    class_counts: BTreeMap<String, usize>,
    /// `(kelas, indeks fitur, nilai fitur) -> jumlah kemunculan`.
    feature_counts: BTreeMap<(String, usize, String), usize>,
    /// Banyaknya nilai unik yang pernah dilihat pada tiap indeks fitur.
    feature_domain: BTreeMap<usize, std::collections::BTreeSet<String>>,
    /// Total baris latih.
    total: usize,
    /// Jumlah fitur per baris.
    n_features: usize,
    /// Konstanta penghalusan Laplace (`1.0` untuk add-one).
    alpha: f64,
}

/// Hasil klasifikasi: kelas terpilih beserta skor tiap kelas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prediction {
    /// Kelas dengan probabilitas posterior tertinggi.
    pub label: String,
    /// Probabilitas posterior tiap kelas, sudah dinormalisasi agar berjumlah satu.
    pub probabilities: BTreeMap<String, f64>,
}

impl CategoricalNaiveBayes {
    /// Membuat model kosong dengan konstanta penghalusan tertentu.
    ///
    /// `alpha = 1.0` berarti penghalusan Laplace klasik; `alpha = 0.0` mematikan
    /// penghalusan (dan membuat model rapuh terhadap kombinasi yang belum pernah dilihat).
    pub fn new(alpha: f64) -> Self {
        Self {
            alpha: alpha.max(0.0),
            ..Default::default()
        }
    }

    /// Melatih model dari sekumpulan baris data.
    pub fn fit(&mut self, samples: &[CategoricalSample]) -> Result<(), BayesError> {
        if samples.is_empty() {
            return Err(BayesError::EmptyInput);
        }
        self.n_features = samples[0].features.len();
        for s in samples {
            if s.features.len() != self.n_features {
                return Err(BayesError::LengthMismatch {
                    a: self.n_features,
                    b: s.features.len(),
                });
            }
            *self.class_counts.entry(s.label.clone()).or_insert(0) += 1;
            for (i, v) in s.features.iter().enumerate() {
                *self
                    .feature_counts
                    .entry((s.label.clone(), i, v.clone()))
                    .or_insert(0) += 1;
                self.feature_domain.entry(i).or_default().insert(v.clone());
            }
            self.total += 1;
        }
        Ok(())
    }

    /// Daftar kelas yang dikenali model, terurut.
    pub fn classes(&self) -> Vec<String> {
        self.class_counts.keys().cloned().collect()
    }

    /// Banyaknya baris latih yang sudah diserap.
    pub fn n_samples(&self) -> usize {
        self.total
    }

    /// Prior sebuah kelas: `P(C) = count(C) / total`.
    pub fn prior(&self, class: &str) -> f64 {
        match self.class_counts.get(class) {
            Some(c) if self.total > 0 => *c as f64 / self.total as f64,
            _ => 0.0,
        }
    }

    /// Likelihood `P(fitur_i = nilai | kelas)` dengan penghalusan Laplace.
    pub fn likelihood(&self, class: &str, index: usize, value: &str) -> f64 {
        let class_total = *self.class_counts.get(class).unwrap_or(&0) as f64;
        let domain = self
            .feature_domain
            .get(&index)
            .map(|d| d.len())
            .unwrap_or(0) as f64;
        let count = *self
            .feature_counts
            .get(&(class.to_string(), index, value.to_string()))
            .unwrap_or(&0) as f64;
        let denom = class_total + self.alpha * domain;
        if denom < EPS {
            return 0.0;
        }
        (count + self.alpha) / denom
    }

    /// Memprediksi kelas sebuah baris fitur.
    ///
    /// Perhitungan dilakukan pada ranah logaritma lalu dikembalikan lewat
    /// *log-sum-exp* agar tidak terjadi *underflow* saat fiturnya banyak.
    pub fn predict(&self, features: &[String]) -> Result<Prediction, BayesError> {
        if self.total == 0 {
            return Err(BayesError::NotTrained);
        }
        if features.len() != self.n_features {
            return Err(BayesError::LengthMismatch {
                a: self.n_features,
                b: features.len(),
            });
        }

        let mut log_scores: Vec<(String, f64)> = Vec::new();
        for class in self.class_counts.keys() {
            let mut logp = self.prior(class).max(f64::MIN_POSITIVE).ln();
            for (i, v) in features.iter().enumerate() {
                logp += self.likelihood(class, i, v).max(f64::MIN_POSITIVE).ln();
            }
            log_scores.push((class.clone(), logp));
        }

        let max = log_scores
            .iter()
            .map(|(_, v)| *v)
            .fold(f64::NEG_INFINITY, f64::max);
        let denom: f64 = log_scores.iter().map(|(_, v)| (v - max).exp()).sum();

        let mut probabilities = BTreeMap::new();
        for (c, v) in &log_scores {
            probabilities.insert(c.clone(), (v - max).exp() / denom);
        }

        // Kelas terpilih ditentukan dari skor log; seri diputus oleh nama kelas
        // agar hasilnya deterministik lintas bahasa.
        let label = log_scores
            .iter()
            .fold(None::<(String, f64)>, |best, (c, v)| match best {
                Some((bc, bv)) if bv > *v || (bv == *v && bc.as_str() <= c.as_str()) => {
                    Some((bc, bv))
                }
                _ => Some((c.clone(), *v)),
            })
            .map(|(c, _)| c)
            .ok_or(BayesError::NotTrained)?;

        Ok(Prediction {
            label,
            probabilities,
        })
    }
}

// ---------------------------------------------------------------------------
// Naive Bayes Gaussian
// ---------------------------------------------------------------------------

/// Ringkasan satu fitur kontinu pada satu kelas.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GaussianStat {
    /// Rerata.
    pub mean: f64,
    /// Simpangan baku (memakai pembagi `n-1` bila `n > 1`).
    pub std_dev: f64,
}

/// Naive Bayes untuk fitur kontinu, mengasumsikan tiap fitur berdistribusi normal.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GaussianNaiveBayes {
    stats: BTreeMap<String, Vec<GaussianStat>>,
    class_counts: BTreeMap<String, usize>,
    total: usize,
    n_features: usize,
}

/// Kerapatan peluang distribusi normal pada titik `x`.
///
/// Simpangan baku nol diperlakukan sebagai nilai sangat kecil agar tidak
/// menghasilkan pembagian dengan nol pada fitur konstan.
pub fn gaussian_pdf(x: f64, mean: f64, std_dev: f64) -> f64 {
    let s = if std_dev.abs() < 1e-9 {
        1e-9
    } else {
        std_dev.abs()
    };
    let z = (x - mean) / s;
    (-0.5 * z * z).exp() / (s * (core::f64::consts::TAU).sqrt())
}

impl GaussianNaiveBayes {
    /// Membuat model kosong.
    pub fn new() -> Self {
        Self::default()
    }

    /// Melatih model dari pasangan `(fitur, label)`.
    pub fn fit(&mut self, x: &[Vec<f64>], y: &[String]) -> Result<(), BayesError> {
        if x.is_empty() {
            return Err(BayesError::EmptyInput);
        }
        if x.len() != y.len() {
            return Err(BayesError::LengthMismatch {
                a: x.len(),
                b: y.len(),
            });
        }
        self.n_features = x[0].len();
        let mut grouped: BTreeMap<String, Vec<&Vec<f64>>> = BTreeMap::new();
        for (row, label) in x.iter().zip(y) {
            if row.len() != self.n_features {
                return Err(BayesError::LengthMismatch {
                    a: self.n_features,
                    b: row.len(),
                });
            }
            grouped.entry(label.clone()).or_default().push(row);
            self.total += 1;
        }
        for (label, rows) in grouped {
            self.class_counts.insert(label.clone(), rows.len());
            let mut stats = Vec::with_capacity(self.n_features);
            for i in 0..self.n_features {
                let vals: Vec<f64> = rows.iter().map(|r| r[i]).collect();
                let n = vals.len() as f64;
                let mean = vals.iter().sum::<f64>() / n;
                let var = if vals.len() > 1 {
                    vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0)
                } else {
                    0.0
                };
                stats.push(GaussianStat {
                    mean,
                    std_dev: var.sqrt(),
                });
            }
            self.stats.insert(label, stats);
        }
        Ok(())
    }

    /// Statistik yang dipelajari untuk sebuah kelas.
    pub fn stats_for(&self, class: &str) -> Option<&[GaussianStat]> {
        self.stats.get(class).map(|v| v.as_slice())
    }

    /// Prior sebuah kelas.
    pub fn prior(&self, class: &str) -> f64 {
        match self.class_counts.get(class) {
            Some(c) if self.total > 0 => *c as f64 / self.total as f64,
            _ => 0.0,
        }
    }

    /// Memprediksi kelas sebuah baris fitur kontinu.
    pub fn predict(&self, features: &[f64]) -> Result<Prediction, BayesError> {
        if self.total == 0 {
            return Err(BayesError::NotTrained);
        }
        if features.len() != self.n_features {
            return Err(BayesError::LengthMismatch {
                a: self.n_features,
                b: features.len(),
            });
        }
        let mut log_scores: Vec<(String, f64)> = Vec::new();
        for (class, stats) in &self.stats {
            let mut logp = self.prior(class).max(f64::MIN_POSITIVE).ln();
            for (i, s) in stats.iter().enumerate() {
                logp += gaussian_pdf(features[i], s.mean, s.std_dev)
                    .max(f64::MIN_POSITIVE)
                    .ln();
            }
            log_scores.push((class.clone(), logp));
        }
        let max = log_scores
            .iter()
            .map(|(_, v)| *v)
            .fold(f64::NEG_INFINITY, f64::max);
        let denom: f64 = log_scores.iter().map(|(_, v)| (v - max).exp()).sum();
        let mut probabilities = BTreeMap::new();
        for (c, v) in &log_scores {
            probabilities.insert(c.clone(), (v - max).exp() / denom);
        }
        let label = log_scores
            .iter()
            .fold(None::<(String, f64)>, |best, (c, v)| match best {
                Some((bc, bv)) if bv > *v || (bv == *v && bc.as_str() <= c.as_str()) => {
                    Some((bc, bv))
                }
                _ => Some((c.clone(), *v)),
            })
            .map(|(c, _)| c)
            .ok_or(BayesError::NotTrained)?;
        Ok(Prediction {
            label,
            probabilities,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    fn s(v: &str) -> String {
        v.to_string()
    }

    #[test]
    fn posterior_dasar() {
        // P(H)=0.02, P(E|H)=0.3, P(E)=0.1 -> 0.06
        close(posterior(0.02, 0.3, 0.1).unwrap(), 0.06);
    }

    #[test]
    fn posterior_menolak_bukti_nol() {
        assert_eq!(posterior(0.5, 0.5, 0.0), Err(BayesError::ZeroEvidence));
    }

    #[test]
    fn posterior_menolak_di_luar_rentang() {
        assert!(matches!(
            posterior(1.5, 0.5, 0.5),
            Err(BayesError::ProbabilityOutOfRange(_))
        ));
        assert!(matches!(
            posterior(0.5, -0.1, 0.5),
            Err(BayesError::ProbabilityOutOfRange(_))
        ));
        assert!(matches!(
            posterior(0.5, 0.5, f64::NAN),
            Err(BayesError::ProbabilityOutOfRange(_))
        ));
    }

    #[test]
    fn probabilitas_total_kasus_hoaks() {
        // 0.2*0.9 + 0.8*0.3 = 0.42
        close(total_probability(&[0.2, 0.8], &[0.9, 0.3]).unwrap(), 0.42);
    }

    #[test]
    fn probabilitas_total_panjang_beda_error() {
        assert_eq!(
            total_probability(&[0.5, 0.5], &[0.1]),
            Err(BayesError::LengthMismatch { a: 2, b: 1 })
        );
    }

    #[test]
    fn probabilitas_total_kosong_error() {
        assert_eq!(total_probability(&[], &[]), Err(BayesError::EmptyInput));
    }

    #[test]
    fn validasi_prior() {
        assert!(validate_priors(&[0.2, 0.8]).is_ok());
        assert!(validate_priors(&[0.3, 0.3, 0.4]).is_ok());
        assert!(matches!(
            validate_priors(&[0.2, 0.5]),
            Err(BayesError::PriorsDoNotSumToOne(_))
        ));
        assert_eq!(validate_priors(&[]), Err(BayesError::EmptyInput));
    }

    #[test]
    fn posterior_semua_berjumlah_satu() {
        let post = posterior_all(&[0.2, 0.3, 0.5], &[0.9, 0.4, 0.1]).unwrap();
        close(post.iter().sum::<f64>(), 1.0);
        assert_eq!(post.len(), 3);
    }

    #[test]
    fn posterior_semua_kasus_hoaks() {
        let post = posterior_all(&[0.2, 0.8], &[0.9, 0.3]).unwrap();
        close(post[0], 3.0 / 7.0);
        close(post[1], 4.0 / 7.0);
    }

    #[test]
    fn posterior_pada_indeks() {
        close(
            posterior_at(&[0.2, 0.8], &[0.9, 0.3], 0).unwrap(),
            3.0 / 7.0,
        );
        assert_eq!(
            posterior_at(&[0.2, 0.8], &[0.9, 0.3], 5),
            Err(BayesError::IndexOutOfRange { index: 5, len: 2 })
        );
    }

    #[test]
    fn posterior_semua_menolak_bukti_nol() {
        assert_eq!(
            posterior_all(&[0.5, 0.5], &[0.0, 0.0]),
            Err(BayesError::ZeroEvidence)
        );
    }

    #[test]
    fn odds_bolak_balik() {
        for p in [0.0, 0.1, 0.25, 0.5, 0.75, 0.9] {
            close(from_odds(to_odds(p).unwrap()), p);
        }
    }

    #[test]
    fn odds_pada_batas() {
        assert!(to_odds(1.0).unwrap().is_infinite());
        close(to_odds(0.0).unwrap(), 0.0);
        close(from_odds(f64::INFINITY), 1.0);
        close(from_odds(-1.0), 0.0);
        close(from_odds(0.0), 0.0);
    }

    #[test]
    fn odds_menolak_di_luar_rentang() {
        assert!(matches!(
            to_odds(1.5),
            Err(BayesError::ProbabilityOutOfRange(_))
        ));
    }

    #[test]
    fn rasio_kemungkinan() {
        close(likelihood_ratio(0.9, 0.3).unwrap(), 3.0);
        assert!(likelihood_ratio(0.9, 0.0).unwrap().is_infinite());
        close(likelihood_ratio(0.0, 0.0).unwrap(), 0.0);
    }

    #[test]
    fn kasus_hoaks_lengkap() {
        // Tugas Pertemuan 5 IND323.
        let r = binary_traced(0.2, 0.9, 0.3).unwrap();
        close(r.evidence, 0.42);
        close(r.posterior, 3.0 / 7.0);
        close(r.posterior_complement, 4.0 / 7.0);
        close(r.likelihood_ratio, 3.0);
        close(r.posterior + r.posterior_complement, 1.0);
        assert_eq!(r.steps.len(), 5);
        assert_eq!(r.steps[1].label, "P(E)");
    }

    #[test]
    fn kasus_biner_prior_ekstrem() {
        let r = binary_traced(1.0, 0.9, 0.3).unwrap();
        close(r.posterior, 1.0);
        let r = binary_traced(0.0, 0.9, 0.3).unwrap();
        close(r.posterior, 0.0);
    }

    #[test]
    fn kasus_biner_menolak_bukti_nol() {
        assert_eq!(binary_traced(0.5, 0.0, 0.0), Err(BayesError::ZeroEvidence));
    }

    #[test]
    fn kasus_biner_menolak_di_luar_rentang() {
        assert!(matches!(
            binary_traced(2.0, 0.5, 0.5),
            Err(BayesError::ProbabilityOutOfRange(_))
        ));
    }

    fn data_cuaca() -> Vec<CategoricalSample> {
        // Dataset "bermain tenis" klasik (Mitchell, 1997) — 14 baris.
        let raw = [
            (["Cerah", "Panas", "Tinggi", "Lemah"], "Tidak"),
            (["Cerah", "Panas", "Tinggi", "Kuat"], "Tidak"),
            (["Mendung", "Panas", "Tinggi", "Lemah"], "Ya"),
            (["Hujan", "Sejuk", "Tinggi", "Lemah"], "Ya"),
            (["Hujan", "Dingin", "Normal", "Lemah"], "Ya"),
            (["Hujan", "Dingin", "Normal", "Kuat"], "Tidak"),
            (["Mendung", "Dingin", "Normal", "Kuat"], "Ya"),
            (["Cerah", "Sejuk", "Tinggi", "Lemah"], "Tidak"),
            (["Cerah", "Dingin", "Normal", "Lemah"], "Ya"),
            (["Hujan", "Sejuk", "Normal", "Lemah"], "Ya"),
            (["Cerah", "Sejuk", "Normal", "Kuat"], "Ya"),
            (["Mendung", "Sejuk", "Tinggi", "Kuat"], "Ya"),
            (["Mendung", "Panas", "Normal", "Lemah"], "Ya"),
            (["Hujan", "Sejuk", "Tinggi", "Kuat"], "Tidak"),
        ];
        raw.iter()
            .map(|(f, l)| CategoricalSample {
                features: f.iter().map(|v| s(v)).collect(),
                label: s(l),
            })
            .collect()
    }

    #[test]
    fn naive_bayes_kategorikal_prior() {
        let mut nb = CategoricalNaiveBayes::new(1.0);
        nb.fit(&data_cuaca()).unwrap();
        assert_eq!(nb.n_samples(), 14);
        close(nb.prior("Ya"), 9.0 / 14.0);
        close(nb.prior("Tidak"), 5.0 / 14.0);
        close(nb.prior("TidakAda"), 0.0);
        assert_eq!(nb.classes(), vec![s("Tidak"), s("Ya")]);
    }

    #[test]
    fn naive_bayes_kategorikal_likelihood_dengan_laplace() {
        let mut nb = CategoricalNaiveBayes::new(1.0);
        nb.fit(&data_cuaca()).unwrap();
        // Fitur 0 punya 3 nilai unik. Kelas "Ya" muncul 9 kali, "Mendung" 4 kali.
        close(nb.likelihood("Ya", 0, "Mendung"), (4.0 + 1.0) / (9.0 + 3.0));
        // Nilai yang tidak pernah muncul tetap dapat peluang kecil, bukan nol.
        assert!(nb.likelihood("Tidak", 0, "Mendung") > 0.0);
    }

    #[test]
    fn naive_bayes_kategorikal_prediksi() {
        let mut nb = CategoricalNaiveBayes::new(1.0);
        nb.fit(&data_cuaca()).unwrap();
        let p = nb
            .predict(&[s("Cerah"), s("Dingin"), s("Tinggi"), s("Kuat")])
            .unwrap();
        assert_eq!(p.label, "Tidak");
        close(p.probabilities.values().sum::<f64>(), 1.0);
        assert_eq!(p.probabilities.len(), 2);
    }

    #[test]
    fn naive_bayes_kategorikal_mengenali_data_latihnya_sendiri() {
        let mut nb = CategoricalNaiveBayes::new(1.0);
        let data = data_cuaca();
        nb.fit(&data).unwrap();
        let benar = data
            .iter()
            .filter(|s| nb.predict(&s.features).unwrap().label == s.label)
            .count();
        // Naive Bayes pada dataset ini biasanya benar minimal 12 dari 14.
        assert!(benar >= 12, "hanya {benar}/14 benar");
    }

    #[test]
    fn naive_bayes_kategorikal_tanpa_penghalusan() {
        let mut nb = CategoricalNaiveBayes::new(0.0);
        nb.fit(&data_cuaca()).unwrap();
        close(nb.likelihood("Ya", 0, "Mendung"), 4.0 / 9.0);
        close(nb.likelihood("Tidak", 0, "Mendung"), 0.0);
    }

    #[test]
    fn naive_bayes_kategorikal_error() {
        let mut nb = CategoricalNaiveBayes::new(1.0);
        assert_eq!(nb.predict(&[s("Cerah")]), Err(BayesError::NotTrained));
        assert_eq!(nb.fit(&[]), Err(BayesError::EmptyInput));
        nb.fit(&data_cuaca()).unwrap();
        assert_eq!(
            nb.predict(&[s("Cerah")]),
            Err(BayesError::LengthMismatch { a: 4, b: 1 })
        );
    }

    #[test]
    fn naive_bayes_kategorikal_menolak_baris_tak_seragam() {
        let mut nb = CategoricalNaiveBayes::new(1.0);
        let data = vec![
            CategoricalSample {
                features: vec![s("a"), s("b")],
                label: s("X"),
            },
            CategoricalSample {
                features: vec![s("a")],
                label: s("Y"),
            },
        ];
        assert_eq!(
            nb.fit(&data),
            Err(BayesError::LengthMismatch { a: 2, b: 1 })
        );
    }

    #[test]
    fn naive_bayes_kategorikal_alpha_negatif_dijepit_ke_nol() {
        let nb = CategoricalNaiveBayes::new(-5.0);
        // Model kosong: likelihood apa pun nol, bukan negatif atau NaN.
        assert_eq!(nb.likelihood("Ya", 0, "Cerah"), 0.0);
    }

    #[test]
    fn kerapatan_normal() {
        // Puncak distribusi baku = 1/sqrt(2*pi) ~ 0.3989
        close(
            gaussian_pdf(0.0, 0.0, 1.0),
            1.0 / (core::f64::consts::TAU).sqrt(),
        );
        // Simetris terhadap rerata.
        close(gaussian_pdf(1.0, 0.0, 1.0), gaussian_pdf(-1.0, 0.0, 1.0));
        // Simpangan nol tidak menghasilkan NaN.
        assert!(gaussian_pdf(0.0, 0.0, 0.0).is_finite());
    }

    #[test]
    fn naive_bayes_gaussian_memisahkan_dua_gugus() {
        let x = vec![
            vec![1.0, 1.0],
            vec![1.2, 0.8],
            vec![0.9, 1.1],
            vec![8.0, 8.0],
            vec![8.2, 7.8],
            vec![7.9, 8.1],
        ];
        let y: Vec<String> = ["A", "A", "A", "B", "B", "B"]
            .iter()
            .map(|v| s(v))
            .collect();
        let mut nb = GaussianNaiveBayes::new();
        nb.fit(&x, &y).unwrap();
        assert_eq!(nb.predict(&[1.0, 1.0]).unwrap().label, "A");
        assert_eq!(nb.predict(&[8.0, 8.0]).unwrap().label, "B");
        close(nb.prior("A"), 0.5);
        let st = nb.stats_for("A").unwrap();
        assert_eq!(st.len(), 2);
        close(st[0].mean, (1.0 + 1.2 + 0.9) / 3.0);
    }

    #[test]
    fn naive_bayes_gaussian_probabilitas_berjumlah_satu() {
        let x = vec![vec![1.0], vec![2.0], vec![9.0], vec![10.0]];
        let y: Vec<String> = ["A", "A", "B", "B"].iter().map(|v| s(v)).collect();
        let mut nb = GaussianNaiveBayes::new();
        nb.fit(&x, &y).unwrap();
        let p = nb.predict(&[5.0]).unwrap();
        close(p.probabilities.values().sum::<f64>(), 1.0);
    }

    #[test]
    fn naive_bayes_gaussian_error() {
        let mut nb = GaussianNaiveBayes::new();
        assert_eq!(nb.predict(&[1.0]), Err(BayesError::NotTrained));
        assert_eq!(nb.fit(&[], &[]), Err(BayesError::EmptyInput));
        assert_eq!(
            nb.fit(&[vec![1.0]], &[]),
            Err(BayesError::LengthMismatch { a: 1, b: 0 })
        );
        let mut nb2 = GaussianNaiveBayes::new();
        assert_eq!(
            nb2.fit(&[vec![1.0, 2.0], vec![1.0]], &[s("A"), s("B")]),
            Err(BayesError::LengthMismatch { a: 2, b: 1 })
        );
    }

    #[test]
    fn naive_bayes_gaussian_kelas_satu_baris_tidak_pecah() {
        let x = vec![vec![1.0], vec![5.0]];
        let y = vec![s("A"), s("B")];
        let mut nb = GaussianNaiveBayes::new();
        nb.fit(&x, &y).unwrap();
        let p = nb.predict(&[1.0]).unwrap();
        assert!(p.probabilities.values().all(|v| v.is_finite()));
        assert_eq!(p.label, "A");
    }

    #[test]
    fn pesan_error_terbaca() {
        for e in [
            BayesError::ProbabilityOutOfRange("p=2".into()),
            BayesError::ZeroEvidence,
            BayesError::LengthMismatch { a: 1, b: 2 },
            BayesError::EmptyInput,
            BayesError::PriorsDoNotSumToOne(1.5),
            BayesError::IndexOutOfRange { index: 3, len: 2 },
            BayesError::NotTrained,
        ] {
            assert!(!e.to_string().is_empty());
        }
    }

    #[test]
    fn pemformatan_tampilan_membuang_derau_biner() {
        assert_eq!(display_number(0.9 * 0.2 + 0.3 * 0.8), "0.42");
        assert_eq!(display_number(0.42), "0.42");
        assert_eq!(display_number(3.0), "3");
        assert_eq!(display_number(0.0), "0");
        assert_eq!(display_number(-0.0), "0");
        assert_eq!(display_number(1.0 / 3.0), "0.3333333333");
        assert_eq!(display_number(-2.5), "-2.5");
        assert_eq!(display_number(f64::INFINITY), "inf");
        assert_eq!(display_number(f64::NEG_INFINITY), "-inf");
        assert_eq!(display_number(f64::NAN), "—");
    }

    #[test]
    fn rumus_tidak_memuat_derau_biner() {
        let r = binary_traced(0.2, 0.9, 0.3).unwrap();
        for s in &r.steps {
            assert!(
                !s.formula.contains("0000000000"),
                "rumus masih menampilkan derau: {}",
                s.formula
            );
        }
        // Nilai internal tetap presisi penuh, bukan versi yang dibulatkan.
        assert_ne!(r.evidence, 0.42);
        assert_eq!(crate::fx::ulp_distance(r.evidence, 0.42), Some(1));
    }

    #[test]
    fn hasil_bisa_di_serialisasi() {
        let r = binary_traced(0.2, 0.9, 0.3).unwrap();
        let json = serde_json::to_string(&r).unwrap();
        let balik: BayesResult = serde_json::from_str(&json).unwrap();
        // Struktur harus utuh: jumlah langkah, label, dan teks rumus tidak berubah.
        assert_eq!(balik.steps.len(), r.steps.len());
        for (a, b) in balik.steps.iter().zip(&r.steps) {
            assert_eq!(a.label, b.label);
            assert_eq!(a.formula, b.formula);
        }
        // Angkanya dibandingkan dengan toleransi 1 ULP, bukan `==`. Lihat
        // `serde_json_bisa_meleset_satu_ulp` di bawah untuk alasannya.
        for (a, b) in [
            (balik.posterior, r.posterior),
            (balik.posterior_complement, r.posterior_complement),
            (balik.evidence, r.evidence),
            (balik.likelihood_ratio, r.likelihood_ratio),
        ] {
            assert!(
                crate::fx::ulp_distance(a, b).is_some_and(|d| d <= 1),
                "{a} menyimpang lebih dari 1 ULP dari {b}"
            );
        }
    }

    #[test]
    fn serde_json_bisa_meleset_satu_ulp() {
        // Uji ini memagari cacat pihak ketiga, bukan cacat kita.
        //
        // `serde_json::from_str::<f64>` memakai jalur cepat yang salah membulat
        // pada sebagian nilai: pengukuran 200.000 sampel menemukan 27.548
        // penyimpangan 1 ULP, sedangkan `str::parse::<f64>` bawaan Rust nol.
        //
        // Konsekuensinya untuk proyek ini: vektor uji lintas bahasa TIDAK boleh
        // memakai desimal. Semuanya lewat [`crate::fx`] dalam bentuk pola bit.
        // Kalau suatu saat uji ini gagal, artinya serde_json sudah diperbaiki
        // dan catatan di `fx.rs` perlu diperbarui — bukan berarti ada regresi.
        let v = 0.9 * 0.2 + 0.3 * 0.8;
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "0.42000000000000004");

        let lewat_serde: f64 = serde_json::from_str(&json).unwrap();
        let lewat_std: f64 = json.parse().unwrap();

        assert!(
            crate::fx::bit_equal(lewat_std, v),
            "parser baku Rust seharusnya eksak"
        );
        assert_eq!(
            crate::fx::ulp_distance(lewat_serde, v),
            Some(1),
            "cacat serde_json berubah perilaku; perbarui catatan di fx.rs"
        );

        // Jalur yang kita pakai memang selalu eksak.
        assert!(crate::fx::bit_equal(
            crate::fx::from_hex(&crate::fx::to_hex(v)).unwrap(),
            v
        ));
    }
}
