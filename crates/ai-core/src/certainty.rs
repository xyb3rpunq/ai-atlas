//! Sesi 3 — Ketidakpastian pada Kecerdasan Buatan.
//!
//! Implementasi *Certainty Factor* seperti yang dipakai sistem pakar MYCIN
//! (Shortliffe & Buchanan, 1975). CF menyatakan seberapa kuat sebuah bukti
//! mendukung (MB, *measure of belief*) atau menentang (MD, *measure of
//! disbelief*) sebuah hipotesis.

use serde::{Deserialize, Serialize};

/// Batas toleransi perbandingan bilangan pecahan di modul ini.
pub const EPS: f64 = 1e-9;

/// Kesalahan yang bisa terjadi saat menghitung certainty factor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CfError {
    /// Nilai MB atau MD di luar rentang `[0, 1]`.
    BeliefOutOfRange(String),
    /// Nilai CF di luar rentang `[-1, 1]`.
    CfOutOfRange(String),
    /// Daftar CF yang diberikan kosong.
    EmptyInput,
}

impl crate::galat::Dijelaskan for CfError {
    fn kode(&self) -> &'static str {
        match self {
            CfError::BeliefOutOfRange(_) => "cf.mb_md_di_luar_rentang",
            CfError::CfOutOfRange(_) => "cf.cf_di_luar_rentang",
            CfError::EmptyInput => "cf.daftar_kosong",
        }
    }

    fn argumen(&self) -> Vec<String> {
        match self {
            CfError::BeliefOutOfRange(v) | CfError::CfOutOfRange(v) => vec![v.clone()],
            CfError::EmptyInput => Vec::new(),
        }
    }
}

impl core::fmt::Display for CfError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CfError::BeliefOutOfRange(v) => write!(f, "MB/MD harus di rentang [0,1], diberi {v}"),
            CfError::CfOutOfRange(v) => write!(f, "CF harus di rentang [-1,1], diberi {v}"),
            CfError::EmptyInput => write!(f, "daftar CF kosong"),
        }
    }
}

/// Interpretasi linguistik dari sebuah nilai CF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    /// `CF <= -0.8`
    DefinitelyNot,
    /// `-0.8 < CF <= -0.4`
    ProbablyNot,
    /// `-0.4 < CF <= -0.2`
    MaybeNot,
    /// `-0.2 < CF < 0.2`
    Unknown,
    /// `0.2 <= CF < 0.4`
    Maybe,
    /// `0.4 <= CF < 0.8`
    Probably,
    /// `CF >= 0.8`
    Definitely,
}

impl Confidence {
    /// Label Bahasa Indonesia.
    pub fn label_id(self) -> &'static str {
        match self {
            Confidence::DefinitelyNot => "pasti tidak",
            Confidence::ProbablyNot => "hampir pasti tidak",
            Confidence::MaybeNot => "mungkin tidak",
            Confidence::Unknown => "tidak diketahui",
            Confidence::Maybe => "mungkin",
            Confidence::Probably => "hampir pasti",
            Confidence::Definitely => "pasti",
        }
    }

    /// Label Bahasa Inggris.
    pub fn label_en(self) -> &'static str {
        match self {
            Confidence::DefinitelyNot => "definitely not",
            Confidence::ProbablyNot => "almost certainly not",
            Confidence::MaybeNot => "probably not",
            Confidence::Unknown => "unknown",
            Confidence::Maybe => "maybe",
            Confidence::Probably => "almost certainly",
            Confidence::Definitely => "definitely",
        }
    }
}

fn check_belief(v: f64, name: &str) -> Result<f64, CfError> {
    if !v.is_finite() || !(-EPS..=1.0 + EPS).contains(&v) {
        return Err(CfError::BeliefOutOfRange(format!("{name}={v}")));
    }
    Ok(v.clamp(0.0, 1.0))
}

fn check_cf(v: f64, name: &str) -> Result<f64, CfError> {
    if !v.is_finite() || !(-1.0 - EPS..=1.0 + EPS).contains(&v) {
        return Err(CfError::CfOutOfRange(format!("{name}={v}")));
    }
    Ok(v.clamp(-1.0, 1.0))
}

/// `CF = MB - MD`. Rumus dasar sesi 3.
///
/// ```
/// use ai_core::certainty::cf_from_mb_md;
/// // Tugas Sesi 3: MB[Cacar, Bintik] = 0.8, MD[Cacar, Bintik] = 0.01
/// assert!((cf_from_mb_md(0.8, 0.01).unwrap() - 0.79).abs() < 1e-9);
/// ```
pub fn cf_from_mb_md(mb: f64, md: f64) -> Result<f64, CfError> {
    let mb = check_belief(mb, "MB")?;
    let md = check_belief(md, "MD")?;
    Ok(mb - md)
}

/// Menggabungkan dua CF dari bukti berbeda untuk hipotesis yang sama
/// (kombinasi paralel, *incrementally acquired evidence* pada MYCIN).
pub fn combine_parallel(cf1: f64, cf2: f64) -> Result<f64, CfError> {
    let a = check_cf(cf1, "cf1")?;
    let b = check_cf(cf2, "cf2")?;
    let out = if a >= 0.0 && b >= 0.0 {
        a + b * (1.0 - a)
    } else if a <= 0.0 && b <= 0.0 {
        a + b * (1.0 + a)
    } else {
        let denom = 1.0 - a.abs().min(b.abs());
        if denom.abs() < EPS {
            // Bukti berlawanan penuh (+1 lawan -1) saling meniadakan.
            0.0
        } else {
            (a + b) / denom
        }
    };
    Ok(out.clamp(-1.0, 1.0))
}

/// Menggabungkan seluruh CF dalam daftar secara paralel, kiri ke kanan.
pub fn combine_many(cfs: &[f64]) -> Result<f64, CfError> {
    let (first, rest) = cfs.split_first().ok_or(CfError::EmptyInput)?;
    let mut acc = check_cf(*first, "cf[0]")?;
    for c in rest {
        acc = combine_parallel(acc, *c)?;
    }
    Ok(acc)
}

/// CF sebuah kesimpulan = CF aturan x CF bukti (kombinasi berantai).
///
/// Bukti dengan CF negatif tidak menyalakan aturan, jadi hasilnya nol.
pub fn combine_sequential(cf_rule: f64, cf_evidence: f64) -> Result<f64, CfError> {
    let r = check_cf(cf_rule, "cf_rule")?;
    let e = check_cf(cf_evidence, "cf_evidence")?;
    Ok((r * e.max(0.0)).clamp(-1.0, 1.0))
}

/// CF gabungan untuk premis yang dihubungkan `AND` — diambil nilai minimum.
pub fn combine_and(cfs: &[f64]) -> Result<f64, CfError> {
    if cfs.is_empty() {
        return Err(CfError::EmptyInput);
    }
    let mut min = f64::INFINITY;
    for (i, c) in cfs.iter().enumerate() {
        let v = check_cf(*c, &format!("cf[{i}]"))?;
        if v < min {
            min = v;
        }
    }
    Ok(min)
}

/// CF gabungan untuk premis yang dihubungkan `OR` — diambil nilai maksimum.
pub fn combine_or(cfs: &[f64]) -> Result<f64, CfError> {
    if cfs.is_empty() {
        return Err(CfError::EmptyInput);
    }
    let mut max = f64::NEG_INFINITY;
    for (i, c) in cfs.iter().enumerate() {
        let v = check_cf(*c, &format!("cf[{i}]"))?;
        if v > max {
            max = v;
        }
    }
    Ok(max)
}

/// Menerjemahkan nilai CF menjadi label linguistik.
pub fn interpret(cf: f64) -> Confidence {
    if cf <= -0.8 {
        Confidence::DefinitelyNot
    } else if cf <= -0.4 {
        Confidence::ProbablyNot
    } else if cf <= -0.2 {
        Confidence::MaybeNot
    } else if cf < 0.2 {
        Confidence::Unknown
    } else if cf < 0.4 {
        Confidence::Maybe
    } else if cf < 0.8 {
        Confidence::Probably
    } else {
        Confidence::Definitely
    }
}

/// Satu langkah perhitungan, dipakai untuk menampilkan jejak ke pengguna.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CfStep {
    /// Operasi yang dilakukan, mis. `"combine_parallel"`.
    pub op: String,
    /// Rumus dalam bentuk teks siap tampil.
    pub formula: String,
    /// Hasil setelah langkah ini.
    pub value: f64,
}

/// Menggabungkan banyak CF sambil merekam tiap langkahnya.
pub fn combine_many_traced(cfs: &[f64]) -> Result<(f64, Vec<CfStep>), CfError> {
    let (first, rest) = cfs.split_first().ok_or(CfError::EmptyInput)?;
    let mut acc = check_cf(*first, "cf[0]")?;
    let mut steps = vec![CfStep {
        op: "init".into(),
        formula: format!("CF = {acc:.4}"),
        value: acc,
    }];
    for (i, c) in rest.iter().enumerate() {
        let prev = acc;
        acc = combine_parallel(acc, *c)?;
        let rule = if prev >= 0.0 && *c >= 0.0 {
            format!("{prev:.4} + {c:.4} x (1 - {prev:.4})")
        } else if prev <= 0.0 && *c <= 0.0 {
            format!("{prev:.4} + {c:.4} x (1 + {prev:.4})")
        } else {
            format!("({prev:.4} + {c:.4}) / (1 - min(|{prev:.4}|, |{c:.4}|))")
        };
        steps.push(CfStep {
            op: "combine_parallel".into(),
            formula: format!("CF{} = {rule} = {acc:.4}", i + 1),
            value: acc,
        });
    }
    Ok((acc, steps))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    #[test]
    fn cf_dasar_kasus_cacar() {
        // Tugas Sesi 3 IND323: MB=0.8, MD=0.01 -> CF=0.79
        close(cf_from_mb_md(0.8, 0.01).unwrap(), 0.79);
    }

    #[test]
    fn cf_dasar_tanpa_ketidakpercayaan() {
        // Kasus kedua tugas: MB=0.3, MD=0 -> CF=0.3
        close(cf_from_mb_md(0.3, 0.0).unwrap(), 0.3);
    }

    #[test]
    fn cf_dasar_bisa_negatif() {
        close(cf_from_mb_md(0.2, 0.7).unwrap(), -0.5);
    }

    #[test]
    fn cf_dasar_menolak_di_luar_rentang() {
        assert!(matches!(
            cf_from_mb_md(1.2, 0.0),
            Err(CfError::BeliefOutOfRange(_))
        ));
        assert!(matches!(
            cf_from_mb_md(0.5, -0.1),
            Err(CfError::BeliefOutOfRange(_))
        ));
        assert!(matches!(
            cf_from_mb_md(f64::NAN, 0.0),
            Err(CfError::BeliefOutOfRange(_))
        ));
        assert!(matches!(
            cf_from_mb_md(0.5, f64::INFINITY),
            Err(CfError::BeliefOutOfRange(_))
        ));
    }

    #[test]
    fn paralel_dua_positif() {
        // 0.8 + 0.6*(1-0.8) = 0.92
        close(combine_parallel(0.8, 0.6).unwrap(), 0.92);
    }

    #[test]
    fn paralel_dua_negatif() {
        // -0.8 + (-0.6)*(1-0.8) = -0.92
        close(combine_parallel(-0.8, -0.6).unwrap(), -0.92);
    }

    #[test]
    fn paralel_tanda_berbeda() {
        // (0.8 + (-0.6)) / (1 - 0.6) = 0.2 / 0.4 = 0.5
        close(combine_parallel(0.8, -0.6).unwrap(), 0.5);
    }

    #[test]
    fn paralel_berlawanan_penuh_jadi_nol() {
        close(combine_parallel(1.0, -1.0).unwrap(), 0.0);
        close(combine_parallel(-1.0, 1.0).unwrap(), 0.0);
    }

    #[test]
    fn paralel_bersifat_komutatif() {
        for (a, b) in [(0.3, 0.7), (-0.4, -0.2), (0.9, -0.3), (0.0, 0.5)] {
            close(
                combine_parallel(a, b).unwrap(),
                combine_parallel(b, a).unwrap(),
            );
        }
    }

    #[test]
    fn paralel_elemen_identitas_nol() {
        for v in [-0.9, -0.3, 0.0, 0.4, 1.0] {
            close(combine_parallel(v, 0.0).unwrap(), v);
        }
    }

    #[test]
    fn paralel_tidak_pernah_keluar_rentang() {
        let vals = [-1.0, -0.75, -0.5, 0.0, 0.5, 0.75, 1.0];
        for a in vals {
            for b in vals {
                let r = combine_parallel(a, b).unwrap();
                assert!((-1.0..=1.0).contains(&r), "cf({a},{b}) = {r}");
            }
        }
    }

    #[test]
    fn paralel_monoton_naik_untuk_bukti_positif() {
        // Menambah bukti pendukung tidak boleh menurunkan keyakinan.
        let base = 0.5;
        let mut prev = base;
        for extra in [0.1, 0.2, 0.3, 0.4] {
            let next = combine_parallel(base, extra).unwrap();
            assert!(next >= prev - 1e-12, "{next} < {prev}");
            prev = next;
        }
    }

    #[test]
    fn paralel_menolak_input_tak_valid() {
        assert!(matches!(
            combine_parallel(1.5, 0.0),
            Err(CfError::CfOutOfRange(_))
        ));
        assert!(matches!(
            combine_parallel(0.0, f64::NAN),
            Err(CfError::CfOutOfRange(_))
        ));
    }

    #[test]
    fn gabung_banyak() {
        // 0.5 -> 0.5+0.3*0.5 = 0.65 -> 0.65+0.2*0.35 = 0.72
        close(combine_many(&[0.5, 0.3, 0.2]).unwrap(), 0.72);
    }

    #[test]
    fn gabung_banyak_satu_elemen() {
        close(combine_many(&[0.42]).unwrap(), 0.42);
    }

    #[test]
    fn gabung_banyak_kosong_error() {
        assert_eq!(combine_many(&[]), Err(CfError::EmptyInput));
    }

    #[test]
    fn gabung_banyak_menolak_input_tak_valid() {
        assert!(matches!(
            combine_many(&[0.5, 9.0]),
            Err(CfError::CfOutOfRange(_))
        ));
        assert!(matches!(
            combine_many(&[9.0]),
            Err(CfError::CfOutOfRange(_))
        ));
    }

    #[test]
    fn berantai() {
        close(combine_sequential(0.8, 0.5).unwrap(), 0.4);
    }

    #[test]
    fn berantai_bukti_negatif_tidak_menyalakan_aturan() {
        close(combine_sequential(0.8, -0.5).unwrap(), 0.0);
    }

    #[test]
    fn berantai_menolak_input_tak_valid() {
        assert!(matches!(
            combine_sequential(2.0, 0.5),
            Err(CfError::CfOutOfRange(_))
        ));
        assert!(matches!(
            combine_sequential(0.5, 2.0),
            Err(CfError::CfOutOfRange(_))
        ));
    }

    #[test]
    fn and_ambil_minimum() {
        close(combine_and(&[0.8, 0.4, 0.9]).unwrap(), 0.4);
        close(combine_and(&[-0.2, 0.4]).unwrap(), -0.2);
    }

    #[test]
    fn or_ambil_maksimum() {
        close(combine_or(&[0.8, 0.4, 0.9]).unwrap(), 0.9);
        close(combine_or(&[-0.8, -0.4]).unwrap(), -0.4);
    }

    #[test]
    fn and_or_satu_elemen_mengembalikan_elemen_itu() {
        close(combine_and(&[0.33]).unwrap(), 0.33);
        close(combine_or(&[0.33]).unwrap(), 0.33);
    }

    #[test]
    fn and_or_kosong_error() {
        assert_eq!(combine_and(&[]), Err(CfError::EmptyInput));
        assert_eq!(combine_or(&[]), Err(CfError::EmptyInput));
    }

    #[test]
    fn and_or_menolak_input_tak_valid() {
        assert!(matches!(
            combine_and(&[0.5, 2.0]),
            Err(CfError::CfOutOfRange(_))
        ));
        assert!(matches!(
            combine_or(&[0.5, -2.0]),
            Err(CfError::CfOutOfRange(_))
        ));
    }

    #[test]
    fn interpretasi_seluruh_pita() {
        assert_eq!(interpret(-1.0), Confidence::DefinitelyNot);
        assert_eq!(interpret(-0.8), Confidence::DefinitelyNot);
        assert_eq!(interpret(-0.5), Confidence::ProbablyNot);
        assert_eq!(interpret(-0.3), Confidence::MaybeNot);
        assert_eq!(interpret(-0.1), Confidence::Unknown);
        assert_eq!(interpret(0.0), Confidence::Unknown);
        assert_eq!(interpret(0.3), Confidence::Maybe);
        assert_eq!(interpret(0.79), Confidence::Probably);
        assert_eq!(interpret(0.95), Confidence::Definitely);
        assert_eq!(interpret(1.0), Confidence::Definitely);
    }

    #[test]
    fn interpretasi_monoton() {
        // Naiknya CF tidak boleh menurunkan tingkat keyakinan.
        let order = |c: Confidence| match c {
            Confidence::DefinitelyNot => 0,
            Confidence::ProbablyNot => 1,
            Confidence::MaybeNot => 2,
            Confidence::Unknown => 3,
            Confidence::Maybe => 4,
            Confidence::Probably => 5,
            Confidence::Definitely => 6,
        };
        let mut prev = 0;
        let mut cf = -1.0;
        while cf <= 1.0 {
            let cur = order(interpret(cf));
            assert!(cur >= prev, "turun di cf={cf}");
            prev = cur;
            cf += 0.01;
        }
    }

    #[test]
    fn label_tersedia_dua_bahasa() {
        let semua = [
            Confidence::DefinitelyNot,
            Confidence::ProbablyNot,
            Confidence::MaybeNot,
            Confidence::Unknown,
            Confidence::Maybe,
            Confidence::Probably,
            Confidence::Definitely,
        ];
        for c in semua {
            assert!(!c.label_id().is_empty());
            assert!(!c.label_en().is_empty());
        }
        assert_eq!(Confidence::Definitely.label_id(), "pasti");
        assert_eq!(Confidence::Definitely.label_en(), "definitely");
    }

    #[test]
    fn jejak_langkah_konsisten_dengan_hasil() {
        let (v, steps) = combine_many_traced(&[0.5, 0.3, 0.2]).unwrap();
        close(v, combine_many(&[0.5, 0.3, 0.2]).unwrap());
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].op, "init");
        assert!(steps[1].formula.contains("CF1"));
        close(steps.last().unwrap().value, v);
    }

    #[test]
    fn jejak_mencatat_ketiga_bentuk_rumus() {
        let (_, positif) = combine_many_traced(&[0.5, 0.3]).unwrap();
        assert!(positif[1].formula.contains("(1 - 0.5000)"));
        let (_, negatif) = combine_many_traced(&[-0.5, -0.3]).unwrap();
        assert!(negatif[1].formula.contains("(1 + -0.5000)"));
        let (_, campur) = combine_many_traced(&[0.5, -0.3]).unwrap();
        assert!(campur[1].formula.contains("min"));
    }

    #[test]
    fn jejak_kosong_error() {
        assert_eq!(combine_many_traced(&[]).unwrap_err(), CfError::EmptyInput);
    }

    #[test]
    fn pesan_error_terbaca() {
        assert!(!CfError::EmptyInput.to_string().is_empty());
        assert!(CfError::BeliefOutOfRange("MB=2".into())
            .to_string()
            .contains("MB=2"));
        assert!(CfError::CfOutOfRange("cf1=2".into())
            .to_string()
            .contains("cf1=2"));
    }

    #[test]
    fn error_dan_confidence_bisa_di_serialisasi() {
        let e = CfError::BeliefOutOfRange("MB=2".into());
        let json = serde_json::to_string(&e).unwrap();
        assert_eq!(serde_json::from_str::<CfError>(&json).unwrap(), e);
        let c = Confidence::Probably;
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(serde_json::from_str::<Confidence>(&json).unwrap(), c);
    }

    #[test]
    fn langkah_bisa_di_serialisasi() {
        let (_, steps) = combine_many_traced(&[0.5, 0.3]).unwrap();
        let json = serde_json::to_string(&steps).unwrap();
        let balik: Vec<CfStep> = serde_json::from_str(&json).unwrap();
        assert_eq!(balik, steps);
    }
}
