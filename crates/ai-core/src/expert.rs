//! Sesi 11 — Sistem Pakar.
//!
//! Mesin inferensi berbasis aturan dengan dua arah penalaran:
//!
//! - **Runut maju** (*forward chaining*) berangkat dari fakta yang diketahui
//!   dan menyalakan aturan sampai tidak ada lagi yang baru. Cocok saat
//!   datanya lengkap dan pertanyaannya "apa yang bisa disimpulkan".
//! - **Runut mundur** (*backward chaining*) berangkat dari hipotesis dan
//!   menelusuri mundur mencari dukungannya. Cocok saat pertanyaannya "benarkah
//!   yang ini", karena hanya fakta yang relevan yang perlu ditanyakan.
//!
//! Keduanya menyertakan **fasilitas penjelasan**: sistem pakar yang tidak bisa
//! menjawab "kenapa" hanyalah tebakan bercangkang komputer.
//!
//! Ketidakpastian ditangani dengan certainty factor dari [`crate::certainty`],
//! sehingga aturan boleh berkata "kemungkinan besar", bukan hanya "ya".

use crate::certainty;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Kesalahan pada sistem pakar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpertError {
    /// Basis aturan kosong.
    EmptyRuleBase,
    /// Sebuah aturan tidak punya premis sama sekali.
    RuleWithoutPremises(String),
    /// Nilai certainty factor di luar rentang `[-1, 1]`.
    BadCertainty {
        /// Nama aturan atau fakta yang bermasalah.
        source: String,
        /// Nilai yang diberikan.
        value: f64,
    },
    /// Penalaran melingkar terdeteksi.
    CircularReasoning(Vec<String>),
    /// Batas langkah penalaran terlampaui.
    StepLimitExceeded(usize),
}

impl core::fmt::Display for ExpertError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ExpertError::EmptyRuleBase => write!(f, "basis aturan kosong"),
            ExpertError::RuleWithoutPremises(id) => {
                write!(f, "aturan {id} tidak punya premis")
            }
            ExpertError::BadCertainty { source, value } => {
                write!(f, "certainty factor {source} di luar [-1,1]: {value}")
            }
            ExpertError::CircularReasoning(path) => {
                write!(f, "penalaran melingkar: {}", path.join(" -> "))
            }
            ExpertError::StepLimitExceeded(n) => {
                write!(f, "melampaui {n} langkah penalaran")
            }
        }
    }
}

/// Batas langkah penalaran, sebagai jaring pengaman terhadap basis aturan
/// yang tidak pernah mencapai keadaan tetap.
pub const MAX_STEPS: usize = 10_000;

/// Penghubung antarpremis dalam satu aturan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Connective {
    /// Semua premis harus benar; certainty factor-nya diambil minimum.
    And,
    /// Cukup satu premis benar; certainty factor-nya diambil maksimum.
    Or,
}

/// Satu premis: sebuah fakta yang diharapkan bernilai benar atau salah.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Premise {
    /// Nama fakta.
    pub fact: String,
    /// `false` berarti premis terpenuhi bila faktanya justru tidak berlaku.
    #[serde(default = "yes")]
    pub expected: bool,
}

fn yes() -> bool {
    true
}

impl Premise {
    /// Premis biasa: fakta harus berlaku.
    pub fn new(fact: impl Into<String>) -> Self {
        Self {
            fact: fact.into(),
            expected: true,
        }
    }

    /// Premis ingkar: fakta justru harus tidak berlaku.
    pub fn negated(fact: impl Into<String>) -> Self {
        Self {
            fact: fact.into(),
            expected: false,
        }
    }
}

/// Satu aturan `JIKA ... MAKA ...`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rule {
    /// Pengenal aturan, mis. `"R1"`.
    pub id: String,
    /// Daftar premis.
    pub premises: Vec<Premise>,
    /// Penghubung antarpremis.
    pub connective: Connective,
    /// Fakta yang disimpulkan bila aturan menyala.
    pub conclusion: String,
    /// Keyakinan pakar terhadap aturan ini.
    #[serde(default = "one")]
    pub certainty: f64,
    /// Penjelasan mengapa aturan ini ada, ditampilkan pada jawaban "kenapa".
    #[serde(default)]
    pub rationale: String,
}

fn one() -> f64 {
    1.0
}

/// Basis pengetahuan: kumpulan aturan beserta pertanyaan yang boleh diajukan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeBase {
    /// Nama basis pengetahuan.
    pub name: String,
    /// Aturan-aturannya.
    pub rules: Vec<Rule>,
    /// Fakta yang hanya bisa diketahui dengan bertanya kepada pengguna.
    ///
    /// Runut mundur memakai ini untuk memutuskan kapan harus bertanya alih-alih
    /// menyerah. Fakta yang tidak ada di sini dan tidak bisa disimpulkan
    /// dianggap tidak berlaku.
    #[serde(default)]
    pub askable: Vec<String>,
}

impl KnowledgeBase {
    /// Memeriksa kesahihan basis pengetahuan.
    pub fn validate(&self) -> Result<(), ExpertError> {
        if self.rules.is_empty() {
            return Err(ExpertError::EmptyRuleBase);
        }
        for rule in &self.rules {
            if rule.premises.is_empty() {
                return Err(ExpertError::RuleWithoutPremises(rule.id.clone()));
            }
            if !rule.certainty.is_finite() || !(-1.0..=1.0).contains(&rule.certainty) {
                return Err(ExpertError::BadCertainty {
                    source: rule.id.clone(),
                    value: rule.certainty,
                });
            }
        }
        Ok(())
    }

    /// Seluruh fakta yang muncul sebagai kesimpulan sebuah aturan.
    pub fn derivable(&self) -> BTreeSet<String> {
        self.rules.iter().map(|r| r.conclusion.clone()).collect()
    }

    /// Seluruh fakta yang muncul sebagai premis tetapi tidak bisa disimpulkan.
    ///
    /// Inilah yang harus ditanyakan kepada pengguna; kalau ada yang tidak
    /// terdaftar di [`KnowledgeBase::askable`], sistemnya punya lubang.
    pub fn leaf_facts(&self) -> BTreeSet<String> {
        let derivable = self.derivable();
        self.rules
            .iter()
            .flat_map(|r| r.premises.iter().map(|p| p.fact.clone()))
            .filter(|f| !derivable.contains(f))
            .collect()
    }

    /// Fakta daun yang belum terdaftar sebagai bisa ditanyakan.
    pub fn unreachable_facts(&self) -> BTreeSet<String> {
        let askable: BTreeSet<&String> = self.askable.iter().collect();
        self.leaf_facts()
            .into_iter()
            .filter(|f| !askable.contains(f))
            .collect()
    }
}

/// Memori kerja: fakta yang diketahui beserta tingkat keyakinannya.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WorkingMemory {
    facts: BTreeMap<String, f64>,
}

impl WorkingMemory {
    /// Memori kosong.
    pub fn new() -> Self {
        Self::default()
    }

    /// Menyatakan sebuah fakta dengan tingkat keyakinan tertentu.
    ///
    /// Bila faktanya sudah ada, kedua keyakinan digabungkan memakai kombinasi
    /// paralel MYCIN — dua bukti yang sama-sama mendukung memperkuat, bukan
    /// saling menimpa.
    pub fn assert(&mut self, fact: impl Into<String>, cf: f64) -> Result<f64, ExpertError> {
        let name = fact.into();
        if !cf.is_finite() || !(-1.0..=1.0).contains(&cf) {
            return Err(ExpertError::BadCertainty {
                source: name,
                value: cf,
            });
        }
        let combined = match self.facts.get(&name) {
            Some(existing) => certainty::combine_parallel(*existing, cf).map_err(|_| {
                ExpertError::BadCertainty {
                    source: name.clone(),
                    value: cf,
                }
            })?,
            None => cf,
        };
        self.facts.insert(name, combined);
        Ok(combined)
    }

    /// Keyakinan terhadap sebuah fakta. Fakta yang belum diketahui bernilai nol.
    pub fn certainty_of(&self, fact: &str) -> f64 {
        self.facts.get(fact).copied().unwrap_or(0.0)
    }

    /// Apakah sebuah fakta sudah pernah dinyatakan.
    pub fn knows(&self, fact: &str) -> bool {
        self.facts.contains_key(fact)
    }

    /// Seluruh fakta yang diketahui, terurut menurut nama.
    pub fn all(&self) -> Vec<(String, f64)> {
        self.facts.iter().map(|(k, v)| (k.clone(), *v)).collect()
    }

    /// Fakta yang keyakinannya melampaui ambang, terurut dari yang terkuat.
    pub fn conclusions(&self, threshold: f64) -> Vec<(String, f64)> {
        let mut out: Vec<(String, f64)> = self
            .facts
            .iter()
            .filter(|(_, v)| **v >= threshold)
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        // Diurutkan menurun; seri diputus nama agar hasilnya deterministik.
        out.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        out
    }

    /// Jumlah fakta yang diketahui.
    pub fn len(&self) -> usize {
        self.facts.len()
    }

    /// Apakah memori kosong.
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }
}

/// Satu langkah penalaran, dipakai fasilitas penjelasan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    /// Nomor urut langkah, mulai dari satu.
    pub order: usize,
    /// Aturan yang menyala.
    pub rule_id: String,
    /// Bentuk teks aturan, siap ditampilkan.
    pub text: String,
    /// Fakta yang dihasilkan.
    pub conclusion: String,
    /// Keyakinan gabungan premis.
    pub premise_certainty: f64,
    /// Keyakinan kesimpulan setelah dikalikan keyakinan aturan.
    pub conclusion_certainty: f64,
    /// Premis yang mendukung, beserta keyakinannya masing-masing.
    pub support: Vec<(String, f64)>,
}

/// Hasil penalaran runut maju.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForwardResult {
    /// Isi memori kerja setelah penalaran selesai.
    pub memory: WorkingMemory,
    /// Urutan langkah penalaran.
    pub steps: Vec<Step>,
    /// Berapa kali seluruh basis aturan disapu.
    pub passes: usize,
}

/// Menyusun bentuk teks sebuah aturan.
fn rule_text(rule: &Rule) -> String {
    let joiner = match rule.connective {
        Connective::And => " DAN ",
        Connective::Or => " ATAU ",
    };
    let premises: Vec<String> = rule
        .premises
        .iter()
        .map(|p| {
            if p.expected {
                p.fact.clone()
            } else {
                format!("BUKAN {}", p.fact)
            }
        })
        .collect();
    format!("JIKA {} MAKA {}", premises.join(joiner), rule.conclusion)
}

/// Keyakinan sebuah premis terhadap memori kerja saat ini.
fn premise_certainty(memory: &WorkingMemory, premise: &Premise) -> f64 {
    let cf = memory.certainty_of(&premise.fact);
    if premise.expected {
        cf
    } else {
        // Premis ingkar terpenuhi justru saat faktanya lemah atau menyangkal.
        -cf
    }
}

/// Keyakinan gabungan seluruh premis sebuah aturan.
fn combined_premises(memory: &WorkingMemory, rule: &Rule) -> f64 {
    let values: Vec<f64> = rule
        .premises
        .iter()
        .map(|p| premise_certainty(memory, p))
        .collect();
    match rule.connective {
        Connective::And => values.iter().copied().fold(f64::INFINITY, f64::min),
        Connective::Or => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    }
}

/// Ambang minimum agar sebuah aturan dianggap menyala.
pub const FIRING_THRESHOLD: f64 = 0.2;

/// Penalaran runut maju dari fakta awal.
///
/// Basis aturan disapu berulang kali sampai satu sapuan penuh tidak lagi
/// mengubah memori kerja. Aturan yang sudah menyala tidak dijalankan lagi
/// dengan dukungan yang sama, sehingga penalarannya berhenti alih-alih
/// menguatkan kesimpulan sendiri tanpa batas.
pub fn forward_chain(
    kb: &KnowledgeBase,
    initial: &WorkingMemory,
) -> Result<ForwardResult, ExpertError> {
    kb.validate()?;
    let mut memory = initial.clone();
    let mut steps: Vec<Step> = Vec::new();
    let mut fired: BTreeSet<(String, u64)> = BTreeSet::new();
    let mut passes = 0usize;
    let mut budget = MAX_STEPS;

    loop {
        passes += 1;
        let mut changed = false;

        for rule in &kb.rules {
            budget = budget
                .checked_sub(1)
                .ok_or(ExpertError::StepLimitExceeded(MAX_STEPS))?;

            let premise_cf = combined_premises(&memory, rule);
            if premise_cf < FIRING_THRESHOLD {
                continue;
            }

            // Aturan yang sama dengan dukungan yang sama tidak dijalankan dua
            // kali. Pola bit dipakai sebagai kunci supaya perbandingannya
            // eksak, bukan bergantung pembulatan desimal.
            let key = (rule.id.clone(), premise_cf.to_bits());
            if !fired.insert(key) {
                continue;
            }

            let conclusion_cf =
                certainty::combine_sequential(rule.certainty, premise_cf).map_err(|_| {
                    ExpertError::BadCertainty {
                        source: rule.id.clone(),
                        value: rule.certainty,
                    }
                })?;

            let before = memory.certainty_of(&rule.conclusion);
            let after = memory.assert(&rule.conclusion, conclusion_cf)?;
            if (after - before).abs() > 1e-12 || !memory.knows(&rule.conclusion) {
                changed = true;
            }

            steps.push(Step {
                order: steps.len() + 1,
                rule_id: rule.id.clone(),
                text: rule_text(rule),
                conclusion: rule.conclusion.clone(),
                premise_certainty: premise_cf,
                conclusion_certainty: after,
                support: rule
                    .premises
                    .iter()
                    .map(|p| (p.fact.clone(), memory.certainty_of(&p.fact)))
                    .collect(),
            });
        }

        if !changed {
            break;
        }
    }

    Ok(ForwardResult {
        memory,
        steps,
        passes,
    })
}

/// Simpul pada pohon penelusuran runut mundur.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProofNode {
    /// Fakta yang sedang dibuktikan.
    pub goal: String,
    /// Kedalaman pada pohon, akar bernilai nol.
    pub depth: usize,
    /// Keyakinan yang diperoleh.
    pub certainty: f64,
    /// Bagaimana keyakinan itu diperoleh.
    pub outcome: ProofOutcome,
    /// Anak-anak simpul, yaitu premis yang ditelusuri.
    pub children: Vec<ProofNode>,
}

/// Bagaimana sebuah tujuan diselesaikan.
///
/// Bentuk kawatnya sengaja bertanda seragam — `{"kind": "...", ...}` — bukan
/// bentuk bawaan serde yang membungkus varian berdata di dalam objek
/// bernama. Bentuk bawaan itu membuat sisi JavaScript harus membedakan dua
/// susunan yang berbeda untuk satu tipe yang sama, dan cacatnya muncul di
/// layar sebagai `undefined`, bukan sebagai galat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProofOutcome {
    /// Sudah ada di memori kerja sejak awal.
    Known,
    /// Disimpulkan dari sebuah aturan.
    Derived {
        /// Pengenal aturan yang dipakai.
        rule_id: String,
    },
    /// Harus ditanyakan kepada pengguna.
    NeedsAsking,
    /// Tidak ada aturan maupun pertanyaan yang bisa membuktikannya.
    Unprovable,
}

/// Hasil penalaran runut mundur.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackwardResult {
    /// Tujuan yang ditanyakan.
    pub goal: String,
    /// Keyakinan akhir.
    pub certainty: f64,
    /// Pohon penelusuran lengkap, dipakai menjawab "bagaimana".
    pub proof: ProofNode,
    /// Fakta yang perlu ditanyakan agar penelusuran bisa dilanjutkan.
    pub questions: Vec<String>,
}

/// Penalaran runut mundur terhadap sebuah tujuan.
pub fn backward_chain(
    kb: &KnowledgeBase,
    memory: &WorkingMemory,
    goal: &str,
) -> Result<BackwardResult, ExpertError> {
    kb.validate()?;
    let mut questions: BTreeSet<String> = BTreeSet::new();
    let mut visiting: Vec<String> = Vec::new();
    let proof = prove(kb, memory, goal, 0, &mut visiting, &mut questions)?;
    Ok(BackwardResult {
        goal: goal.to_string(),
        certainty: proof.certainty,
        proof,
        questions: questions.into_iter().collect(),
    })
}

/// Membuktikan satu tujuan secara rekursif.
fn prove(
    kb: &KnowledgeBase,
    memory: &WorkingMemory,
    goal: &str,
    depth: usize,
    visiting: &mut Vec<String>,
    questions: &mut BTreeSet<String>,
) -> Result<ProofNode, ExpertError> {
    // Fakta yang sudah diketahui tidak perlu ditelusuri lebih jauh.
    if memory.knows(goal) {
        return Ok(ProofNode {
            goal: goal.to_string(),
            depth,
            certainty: memory.certainty_of(goal),
            outcome: ProofOutcome::Known,
            children: Vec::new(),
        });
    }

    // Penelusuran yang kembali ke tujuan yang sedang dikerjakan berarti basis
    // aturannya melingkar. Ini dilaporkan, bukan dibiarkan menjadi rekursi tak
    // berujung yang menghabiskan tumpukan.
    if visiting.iter().any(|g| g == goal) {
        let mut path = visiting.clone();
        path.push(goal.to_string());
        return Err(ExpertError::CircularReasoning(path));
    }
    if depth > 64 {
        return Err(ExpertError::StepLimitExceeded(depth));
    }

    visiting.push(goal.to_string());

    let candidates: Vec<&Rule> = kb.rules.iter().filter(|r| r.conclusion == goal).collect();
    let mut best: Option<ProofNode> = None;
    for (i, rule) in candidates.iter().enumerate() {
        // Pemangkasan yang tidak mengubah jawaban.
        //
        // Keyakinan yang bisa dihasilkan sebuah aturan tidak pernah melebihi
        // keyakinan aturan itu sendiri, karena kesimpulannya adalah keyakinan
        // aturan dikali keyakinan premis, dan yang terakhir paling besar satu.
        // Jadi bila hasil terbaik sejauh ini sudah melampaui keyakinan setiap
        // aturan yang tersisa, tidak ada yang bisa mengunggulinya dan sisanya
        // tidak perlu ditelusuri.
        //
        // Ini bukan sekadar penghematan waktu. Menelusuri aturan yang mustahil
        // menang berarti menanyakan gejala yang tidak akan mengubah jawaban —
        // padahal mengurangi pertanyaan itulah alasan runut mundur dipakai.
        if let Some(current) = &best {
            let batas_tersisa = candidates[i..]
                .iter()
                .map(|r| r.certainty)
                .fold(f64::NEG_INFINITY, f64::max);
            if current.certainty >= batas_tersisa {
                break;
            }
        }
        let mut children = Vec::with_capacity(rule.premises.len());
        let mut values = Vec::with_capacity(rule.premises.len());
        for premise in &rule.premises {
            let child = prove(kb, memory, &premise.fact, depth + 1, visiting, questions)?;
            values.push(if premise.expected {
                child.certainty
            } else {
                -child.certainty
            });
            children.push(child);
        }
        let premise_cf = match rule.connective {
            Connective::And => values.iter().copied().fold(f64::INFINITY, f64::min),
            Connective::Or => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        };
        let cf = certainty::combine_sequential(rule.certainty, premise_cf).map_err(|_| {
            ExpertError::BadCertainty {
                source: rule.id.clone(),
                value: rule.certainty,
            }
        })?;

        let candidate = ProofNode {
            goal: goal.to_string(),
            depth,
            certainty: cf,
            outcome: ProofOutcome::Derived {
                rule_id: rule.id.clone(),
            },
            children,
        };
        // Beberapa aturan bisa menyimpulkan tujuan yang sama; yang dipilih
        // adalah yang keyakinannya tertinggi.
        best = match best {
            Some(current) if current.certainty >= candidate.certainty => Some(current),
            _ => Some(candidate),
        };
    }

    visiting.pop();

    Ok(match best {
        Some(node) => node,
        None => {
            let askable = kb.askable.iter().any(|f| f == goal);
            if askable {
                questions.insert(goal.to_string());
            }
            ProofNode {
                goal: goal.to_string(),
                depth,
                certainty: 0.0,
                outcome: if askable {
                    ProofOutcome::NeedsAsking
                } else {
                    ProofOutcome::Unprovable
                },
                children: Vec::new(),
            }
        }
    })
}

/// Menjawab "kenapa aturan ini ditanyakan" untuk sebuah aturan.
pub fn explain_why(kb: &KnowledgeBase, rule_id: &str) -> Option<String> {
    let rule = kb.rules.iter().find(|r| r.id == rule_id)?;
    let alasan = if rule.rationale.is_empty() {
        String::new()
    } else {
        format!(" {}", rule.rationale)
    };
    Some(format!(
        "{} dipakai untuk menyimpulkan {}.{alasan}",
        rule_text(rule),
        rule.conclusion
    ))
}

/// Menjawab "bagaimana sampai pada kesimpulan ini" dalam bentuk teks berurut.
pub fn explain_how(result: &ForwardResult, fact: &str) -> Vec<String> {
    result
        .steps
        .iter()
        .filter(|s| s.conclusion == fact)
        .map(|s| {
            let dukungan: Vec<String> = s
                .support
                .iter()
                .map(|(f, cf)| format!("{f} ({cf:.2})"))
                .collect();
            format!(
                "Langkah {}: {} [{}] menghasilkan {} dengan keyakinan {:.2}",
                s.order,
                s.rule_id,
                dukungan.join(", "),
                s.conclusion,
                s.conclusion_certainty
            )
        })
        .collect()
}

/// Basis pengetahuan contoh: diagnosis flu, dari studi kasus modul Sesi 11.
pub fn flu_knowledge_base() -> KnowledgeBase {
    let rule = |id: &str,
                premises: Vec<Premise>,
                connective: Connective,
                conclusion: &str,
                certainty: f64,
                rationale: &str| Rule {
        id: id.to_string(),
        premises,
        connective,
        conclusion: conclusion.to_string(),
        certainty,
        rationale: rationale.to_string(),
    };

    KnowledgeBase {
        name: "Dokter Virtual".to_string(),
        rules: vec![
            rule(
                "R1",
                vec![
                    Premise::new("demam"),
                    Premise::new("pilek"),
                    Premise::new("batuk"),
                ],
                Connective::And,
                "flu",
                0.9,
                "Ketiganya bersamaan adalah pola khas influenza.",
            ),
            rule(
                "R2",
                vec![Premise::new("demam"), Premise::new("nyeri otot")],
                Connective::And,
                "flu",
                0.7,
                "Demam disertai nyeri otot menguatkan dugaan influenza.",
            ),
            rule(
                "R3",
                vec![Premise::new("pilek"), Premise::negated("demam")],
                Connective::And,
                "alergi",
                0.8,
                "Pilek tanpa demam lebih menyerupai reaksi alergi.",
            ),
            rule(
                "R4",
                vec![Premise::new("bersin berulang"), Premise::negated("demam")],
                Connective::And,
                "alergi",
                0.7,
                "Bersin berulang tanpa demam adalah gejala alergi yang umum.",
            ),
            rule(
                "R5",
                vec![Premise::new("flu"), Premise::new("sesak napas")],
                Connective::And,
                "rujuk ke dokter",
                0.95,
                "Sesak napas pada influenza memerlukan pemeriksaan langsung.",
            ),
            rule(
                "R6",
                vec![Premise::new("demam tinggi")],
                Connective::Or,
                "rujuk ke dokter",
                0.9,
                "Demam tinggi selalu perlu diperiksa, apa pun penyebabnya.",
            ),
        ],
        askable: vec![
            "demam".into(),
            "demam tinggi".into(),
            "pilek".into(),
            "batuk".into(),
            "nyeri otot".into(),
            "bersin berulang".into(),
            "sesak napas".into(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    fn memori(pairs: &[(&str, f64)]) -> WorkingMemory {
        let mut m = WorkingMemory::new();
        for (f, cf) in pairs {
            m.assert(*f, *cf).unwrap();
        }
        m
    }

    #[test]
    fn memori_kerja_dasar() {
        let mut m = WorkingMemory::new();
        assert!(m.is_empty());
        assert_eq!(m.certainty_of("apa pun"), 0.0);
        assert!(!m.knows("apa pun"));

        m.assert("demam", 0.8).unwrap();
        assert!(m.knows("demam"));
        close(m.certainty_of("demam"), 0.8);
        assert_eq!(m.len(), 1);
        assert!(!m.is_empty());
    }

    #[test]
    fn memori_menggabungkan_bukti_yang_sama_arah() {
        let mut m = WorkingMemory::new();
        m.assert("flu", 0.6).unwrap();
        let hasil = m.assert("flu", 0.5).unwrap();
        // Kombinasi paralel MYCIN: 0.6 + 0.5*(1-0.6) = 0.8
        close(hasil, 0.8);
        close(m.certainty_of("flu"), 0.8);
    }

    #[test]
    fn memori_menolak_keyakinan_di_luar_rentang() {
        let mut m = WorkingMemory::new();
        assert!(matches!(
            m.assert("x", 1.5),
            Err(ExpertError::BadCertainty { .. })
        ));
        assert!(matches!(
            m.assert("x", f64::NAN),
            Err(ExpertError::BadCertainty { .. })
        ));
        assert!(m.is_empty(), "nilai tak sah tidak boleh ikut tersimpan");
    }

    #[test]
    fn kesimpulan_terurut_menurun() {
        let m = memori(&[("a", 0.3), ("b", 0.9), ("c", 0.6)]);
        let hasil = m.conclusions(0.5);
        assert_eq!(hasil.len(), 2);
        assert_eq!(hasil[0].0, "b");
        assert_eq!(hasil[1].0, "c");
        assert!(m.conclusions(0.95).is_empty());
        assert_eq!(m.all().len(), 3);
    }

    #[test]
    fn validasi_basis_pengetahuan() {
        assert!(flu_knowledge_base().validate().is_ok());

        let kosong = KnowledgeBase {
            name: "x".into(),
            rules: vec![],
            askable: vec![],
        };
        assert_eq!(kosong.validate(), Err(ExpertError::EmptyRuleBase));

        let mut tanpa_premis = flu_knowledge_base();
        tanpa_premis.rules[0].premises.clear();
        assert!(matches!(
            tanpa_premis.validate(),
            Err(ExpertError::RuleWithoutPremises(_))
        ));

        let mut cf_salah = flu_knowledge_base();
        cf_salah.rules[0].certainty = 2.0;
        assert!(matches!(
            cf_salah.validate(),
            Err(ExpertError::BadCertainty { .. })
        ));
    }

    #[test]
    fn basis_pengetahuan_flu_tidak_punya_lubang() {
        // Setiap fakta yang dipakai sebagai premis harus bisa disimpulkan atau
        // ditanyakan. Kalau tidak, sistem akan diam-diam menganggapnya salah.
        let kb = flu_knowledge_base();
        assert!(
            kb.unreachable_facts().is_empty(),
            "fakta tanpa sumber: {:?}",
            kb.unreachable_facts()
        );
        assert!(kb.derivable().contains("flu"));
        assert!(kb.leaf_facts().contains("demam"));
        assert!(!kb.leaf_facts().contains("flu"));
    }

    #[test]
    fn runut_maju_mendiagnosis_flu() {
        let kb = flu_knowledge_base();
        let awal = memori(&[("demam", 1.0), ("pilek", 1.0), ("batuk", 1.0)]);
        let hasil = forward_chain(&kb, &awal).unwrap();
        assert!(hasil.memory.knows("flu"));
        close(hasil.memory.certainty_of("flu"), 0.9);
        assert!(!hasil.steps.is_empty());
        assert!(hasil.passes >= 2, "harus ada sapuan penutup");
    }

    #[test]
    fn runut_maju_membedakan_alergi_dari_flu() {
        let kb = flu_knowledge_base();
        // Pilek tanpa demam: alergi, bukan flu.
        let awal = memori(&[("pilek", 1.0), ("demam", -1.0)]);
        let hasil = forward_chain(&kb, &awal).unwrap();
        assert!(hasil.memory.certainty_of("alergi") > 0.5);
        assert!(
            hasil.memory.certainty_of("flu") < 0.2,
            "flu seharusnya tidak menyala: {}",
            hasil.memory.certainty_of("flu")
        );
    }

    #[test]
    fn runut_maju_merantai_kesimpulan() {
        // R1 menyimpulkan flu, lalu R5 memakai flu untuk merujuk ke dokter.
        // Perantaian inilah yang membedakan runut maju dari sekadar tabel.
        let kb = flu_knowledge_base();
        let awal = memori(&[
            ("demam", 1.0),
            ("pilek", 1.0),
            ("batuk", 1.0),
            ("sesak napas", 1.0),
        ]);
        let hasil = forward_chain(&kb, &awal).unwrap();
        assert!(hasil.memory.knows("rujuk ke dokter"));
        assert!(hasil.memory.certainty_of("rujuk ke dokter") > 0.8);
        let urutan: Vec<&str> = hasil.steps.iter().map(|s| s.rule_id.as_str()).collect();
        let posisi_r1 = urutan.iter().position(|r| *r == "R1").unwrap();
        let posisi_r5 = urutan.iter().position(|r| *r == "R5").unwrap();
        assert!(posisi_r1 < posisi_r5, "R5 tidak boleh menyala sebelum R1");
    }

    #[test]
    fn runut_maju_berhenti_dan_tidak_menguatkan_diri_sendiri() {
        let kb = flu_knowledge_base();
        let awal = memori(&[("demam", 1.0), ("pilek", 1.0), ("batuk", 1.0)]);
        let hasil = forward_chain(&kb, &awal).unwrap();
        // Aturan yang sama tidak boleh menyala dua kali dengan dukungan sama;
        // tanpa penjagaan itu, keyakinannya akan merangkak naik ke satu.
        let r1 = hasil.steps.iter().filter(|s| s.rule_id == "R1").count();
        assert_eq!(r1, 1, "R1 menyala {r1} kali");
        assert!(hasil.memory.certainty_of("flu") <= 1.0);
    }

    #[test]
    fn runut_maju_tanpa_fakta_tidak_menyimpulkan_apa_pun() {
        let kb = flu_knowledge_base();
        let hasil = forward_chain(&kb, &WorkingMemory::new()).unwrap();
        assert!(hasil.steps.is_empty());
        assert!(hasil.memory.is_empty());
    }

    #[test]
    fn runut_maju_menolak_basis_aturan_tak_sah() {
        let kb = KnowledgeBase {
            name: "x".into(),
            rules: vec![],
            askable: vec![],
        };
        assert_eq!(
            forward_chain(&kb, &WorkingMemory::new()),
            Err(ExpertError::EmptyRuleBase)
        );
    }

    #[test]
    fn premis_ingkar_bekerja() {
        let kb = flu_knowledge_base();
        // R3 memerlukan pilek DAN BUKAN demam.
        let dengan_demam = memori(&[("pilek", 1.0), ("demam", 1.0)]);
        let tanpa_demam = memori(&[("pilek", 1.0), ("demam", -1.0)]);
        let a = forward_chain(&kb, &dengan_demam).unwrap();
        let b = forward_chain(&kb, &tanpa_demam).unwrap();
        assert!(a.memory.certainty_of("alergi") < 0.2);
        assert!(b.memory.certainty_of("alergi") > 0.5);
    }

    #[test]
    fn runut_mundur_membuktikan_dari_fakta_yang_ada() {
        let kb = flu_knowledge_base();
        let m = memori(&[("demam", 1.0), ("pilek", 1.0), ("batuk", 1.0)]);
        let hasil = backward_chain(&kb, &m, "flu").unwrap();
        close(hasil.certainty, 0.9);
        assert!(matches!(hasil.proof.outcome, ProofOutcome::Derived { .. }));
        assert_eq!(hasil.proof.children.len(), 3);
        assert!(hasil.questions.is_empty(), "semua fakta sudah diketahui");
    }

    #[test]
    fn runut_mundur_hanya_menanyakan_yang_relevan() {
        // Inilah keunggulan runut mundur: dari tujuh fakta yang bisa
        // ditanyakan, hanya yang menuju tujuan yang perlu diajukan.
        let kb = flu_knowledge_base();
        let hasil = backward_chain(&kb, &WorkingMemory::new(), "alergi").unwrap();
        assert!(!hasil.questions.is_empty());
        assert!(hasil.questions.iter().all(|q| kb.askable.contains(q)));
        assert!(
            !hasil.questions.contains(&"sesak napas".to_string()),
            "sesak napas tidak ada hubungannya dengan alergi"
        );
        assert!(hasil.questions.len() < kb.askable.len());
    }

    #[test]
    fn pemangkasan_tidak_mengubah_jawaban() {
        // Pemangkasan hanya sah bila jawabannya tetap sama. Diuji dengan
        // membandingkan hasilnya terhadap keyakinan tertinggi yang bisa
        // dihasilkan aturan mana pun untuk tujuan itu.
        let kb = flu_knowledge_base();
        for fakta in [
            vec![("demam", 1.0), ("pilek", 1.0), ("batuk", 1.0)],
            vec![("demam", 1.0), ("nyeri otot", 1.0)],
            vec![
                ("demam", 0.5),
                ("pilek", 0.9),
                ("batuk", 0.9),
                ("nyeri otot", 1.0),
            ],
            vec![
                ("demam", 1.0),
                ("pilek", 1.0),
                ("batuk", 1.0),
                ("nyeri otot", 1.0),
            ],
        ] {
            let m = memori(&fakta);

            // Hitung tanpa pemangkasan: semua aturan untuk "flu" dicoba.
            let terbaik_lengkap = kb
                .rules
                .iter()
                .filter(|r| r.conclusion == "flu")
                .map(|r| {
                    let nilai: Vec<f64> = r
                        .premises
                        .iter()
                        .map(|p| {
                            let cf = m.certainty_of(&p.fact);
                            if p.expected {
                                cf
                            } else {
                                -cf
                            }
                        })
                        .collect();
                    let premis = match r.connective {
                        Connective::And => nilai.iter().copied().fold(f64::INFINITY, f64::min),
                        Connective::Or => nilai.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                    };
                    certainty::combine_sequential(r.certainty, premis).unwrap()
                })
                .fold(f64::NEG_INFINITY, f64::max);

            let dengan_pemangkasan = backward_chain(&kb, &m, "flu").unwrap().certainty;
            assert!(
                (dengan_pemangkasan - terbaik_lengkap).abs() < 1e-9,
                "pemangkasan mengubah jawaban untuk {fakta:?}: {dengan_pemangkasan} vs {terbaik_lengkap}"
            );
        }
    }

    #[test]
    fn pemangkasan_mengurangi_pertanyaan() {
        // R1 membuktikan flu dengan keyakinan 0,9 dari gejala yang sudah
        // diketahui. R2 paling banter menghasilkan 0,7, jadi menanyakan nyeri
        // otot tidak akan mengubah apa pun.
        let kb = flu_knowledge_base();
        let m = memori(&[("demam", 1.0), ("pilek", 1.0), ("batuk", 1.0)]);
        let hasil = backward_chain(&kb, &m, "flu").unwrap();
        assert!(
            !hasil.questions.contains(&"nyeri otot".to_string()),
            "pertanyaan yang tidak bisa mengubah jawaban tetap diajukan: {:?}",
            hasil.questions
        );
    }

    #[test]
    fn runut_mundur_memilih_aturan_terkuat() {
        // Flu bisa disimpulkan R1 (0.9) atau R2 (0.7); yang dipilih R1.
        let kb = flu_knowledge_base();
        let m = memori(&[
            ("demam", 1.0),
            ("pilek", 1.0),
            ("batuk", 1.0),
            ("nyeri otot", 1.0),
        ]);
        let hasil = backward_chain(&kb, &m, "flu").unwrap();
        close(hasil.certainty, 0.9);
        match &hasil.proof.outcome {
            ProofOutcome::Derived { rule_id } => assert_eq!(rule_id, "R1"),
            other => panic!("seharusnya diturunkan dari aturan, bukan {other:?}"),
        }
    }

    #[test]
    fn runut_mundur_menandai_yang_tidak_bisa_dibuktikan() {
        let kb = flu_knowledge_base();
        let hasil = backward_chain(&kb, &WorkingMemory::new(), "patah tulang").unwrap();
        assert_eq!(hasil.proof.outcome, ProofOutcome::Unprovable);
        close(hasil.certainty, 0.0);
        assert!(hasil.questions.is_empty());
    }

    #[test]
    fn runut_mundur_mengenali_fakta_yang_sudah_diketahui() {
        let kb = flu_knowledge_base();
        let m = memori(&[("flu", 0.75)]);
        let hasil = backward_chain(&kb, &m, "flu").unwrap();
        assert_eq!(hasil.proof.outcome, ProofOutcome::Known);
        close(hasil.certainty, 0.75);
    }

    #[test]
    fn runut_mundur_mendeteksi_penalaran_melingkar() {
        // Basis aturan yang saling merujuk akan membuat penelusuran berputar.
        // Ini harus dilaporkan, bukan menghabiskan tumpukan pemanggilan.
        let kb = KnowledgeBase {
            name: "melingkar".into(),
            rules: vec![
                Rule {
                    id: "A".into(),
                    premises: vec![Premise::new("q")],
                    connective: Connective::And,
                    conclusion: "p".into(),
                    certainty: 1.0,
                    rationale: String::new(),
                },
                Rule {
                    id: "B".into(),
                    premises: vec![Premise::new("p")],
                    connective: Connective::And,
                    conclusion: "q".into(),
                    certainty: 1.0,
                    rationale: String::new(),
                },
            ],
            askable: vec![],
        };
        let hasil = backward_chain(&kb, &WorkingMemory::new(), "p");
        assert!(matches!(hasil, Err(ExpertError::CircularReasoning(_))));
        if let Err(ExpertError::CircularReasoning(path)) = hasil {
            assert!(path.len() >= 3, "jalur melingkar terlalu pendek: {path:?}");
        }
    }

    #[test]
    fn penjelasan_kenapa() {
        let kb = flu_knowledge_base();
        let teks = explain_why(&kb, "R1").unwrap();
        assert!(teks.contains("JIKA"));
        assert!(teks.contains("flu"));
        assert!(teks.contains("influenza"), "alasan pakar harus ikut muncul");
        assert!(explain_why(&kb, "R99").is_none());
    }

    #[test]
    fn penjelasan_bagaimana() {
        let kb = flu_knowledge_base();
        let awal = memori(&[("demam", 1.0), ("pilek", 1.0), ("batuk", 1.0)]);
        let hasil = forward_chain(&kb, &awal).unwrap();
        let langkah = explain_how(&hasil, "flu");
        assert!(!langkah.is_empty());
        assert!(langkah[0].contains("R1"));
        assert!(langkah[0].contains("demam"));
        assert!(explain_how(&hasil, "tidak ada").is_empty());
    }

    #[test]
    fn bentuk_teks_aturan() {
        let kb = flu_knowledge_base();
        let r1 = rule_text(&kb.rules[0]);
        assert!(r1.starts_with("JIKA "));
        assert!(r1.contains(" DAN "));
        assert!(r1.ends_with("MAKA flu"));

        let r3 = rule_text(&kb.rules[2]);
        assert!(r3.contains("BUKAN demam"));

        let r6 = rule_text(&kb.rules[5]);
        assert!(r6.contains("demam tinggi"));
    }

    #[test]
    fn dua_arah_penalaran_sepakat() {
        // Runut maju dan runut mundur adalah dua jalan menuju jawaban yang
        // sama. Kalau hasilnya berbeda, salah satunya cacat.
        let kb = flu_knowledge_base();
        for fakta in [
            vec![("demam", 1.0), ("pilek", 1.0), ("batuk", 1.0)],
            vec![("demam", 1.0), ("nyeri otot", 1.0)],
            vec![("demam", 0.6), ("pilek", 0.8), ("batuk", 0.7)],
        ] {
            let m = memori(&fakta);
            let maju = forward_chain(&kb, &m).unwrap();
            let mundur = backward_chain(&kb, &m, "flu").unwrap();
            assert!(
                (maju.memory.certainty_of("flu") - mundur.certainty).abs() < 1e-9,
                "maju {} vs mundur {} untuk {fakta:?}",
                maju.memory.certainty_of("flu"),
                mundur.certainty
            );
        }
    }

    #[test]
    fn keyakinan_sebagian_menurunkan_kesimpulan() {
        let kb = flu_knowledge_base();
        let penuh = memori(&[("demam", 1.0), ("pilek", 1.0), ("batuk", 1.0)]);
        let ragu = memori(&[("demam", 0.6), ("pilek", 0.6), ("batuk", 0.6)]);
        let a = forward_chain(&kb, &penuh).unwrap();
        let b = forward_chain(&kb, &ragu).unwrap();
        assert!(
            b.memory.certainty_of("flu") < a.memory.certainty_of("flu"),
            "gejala yang meragukan seharusnya menghasilkan kesimpulan yang lebih lemah"
        );
        // 0.9 x 0.6 = 0.54
        close(b.memory.certainty_of("flu"), 0.54);
    }

    #[test]
    fn hasil_bisa_di_serialisasi() {
        let kb = flu_knowledge_base();
        let json = serde_json::to_string(&kb).unwrap();
        assert_eq!(serde_json::from_str::<KnowledgeBase>(&json).unwrap(), kb);

        let awal = memori(&[("demam", 1.0), ("pilek", 1.0), ("batuk", 1.0)]);
        let hasil = forward_chain(&kb, &awal).unwrap();
        let hj = serde_json::to_string(&hasil).unwrap();
        assert_eq!(
            serde_json::from_str::<ForwardResult>(&hj)
                .unwrap()
                .steps
                .len(),
            hasil.steps.len()
        );

        let mundur = backward_chain(&kb, &awal, "flu").unwrap();
        let mj = serde_json::to_string(&mundur).unwrap();
        assert_eq!(
            serde_json::from_str::<BackwardResult>(&mj).unwrap().goal,
            "flu"
        );
    }

    #[test]
    fn bentuk_kawat_hasil_penelusuran_seragam() {
        // Regresi: bentuk bawaan serde membungkus varian berdata sebagai
        // {"derived":{"rule_id":"R1"}}, sementara varian tanpa data menjadi
        // string biasa. Sisi JavaScript lalu harus membedakan dua susunan
        // untuk satu tipe, dan kekeliruannya muncul di layar sebagai
        // "undefined" alih-alih sebagai galat.
        let kb = flu_knowledge_base();
        let m = memori(&[("demam", 1.0), ("pilek", 1.0), ("batuk", 1.0)]);
        let hasil = backward_chain(&kb, &m, "flu").unwrap();
        let json = serde_json::to_string(&hasil.proof.outcome).unwrap();
        assert_eq!(json, r#"{"kind":"derived","rule_id":"R1"}"#);

        let diketahui = serde_json::to_string(&ProofOutcome::Known).unwrap();
        assert_eq!(diketahui, r#"{"kind":"known"}"#);
        let bertanya = serde_json::to_string(&ProofOutcome::NeedsAsking).unwrap();
        assert_eq!(bertanya, r#"{"kind":"needs_asking"}"#);
        let buntu = serde_json::to_string(&ProofOutcome::Unprovable).unwrap();
        assert_eq!(buntu, r#"{"kind":"unprovable"}"#);

        // Bolak-balik tetap utuh.
        for o in [
            ProofOutcome::Known,
            ProofOutcome::NeedsAsking,
            ProofOutcome::Unprovable,
            ProofOutcome::Derived {
                rule_id: "R9".into(),
            },
        ] {
            let j = serde_json::to_string(&o).unwrap();
            assert_eq!(serde_json::from_str::<ProofOutcome>(&j).unwrap(), o);
        }
    }

    #[test]
    fn pesan_error_terbaca() {
        for e in [
            ExpertError::EmptyRuleBase,
            ExpertError::RuleWithoutPremises("R1".into()),
            ExpertError::BadCertainty {
                source: "R1".into(),
                value: 2.0,
            },
            ExpertError::CircularReasoning(vec!["p".into(), "q".into()]),
            ExpertError::StepLimitExceeded(10),
        ] {
            assert!(!e.to_string().is_empty());
        }
    }
}
