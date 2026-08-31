//! Sesi 1 — Pengantar Kecerdasan Buatan.
//!
//! ELIZA (Weizenbaum, 1966), program yang paling banyak disalahpahami dalam
//! sejarah kecerdasan buatan, ditulis ulang di sini beserta alat untuk
//! membongkarnya.
//!
//! ELIZA tidak memahami apa pun. Ia mencocokkan pola, menukar kata ganti, dan
//! memantulkan kalimat pengguna kembali sebagai pertanyaan. Namun orang yang
//! memakainya — termasuk sekretaris Weizenbaum sendiri — meminta ditinggal
//! berdua dengannya. Kesenjangan antara betapa sederhana mesinnya dan betapa
//! kuat kesan yang ditimbulkannya itulah pelajaran sebenarnya dari sesi ini,
//! dan alasan uji Turing lebih banyak berbicara tentang manusia yang menilai
//! daripada tentang mesin yang dinilai.
//!
//! Karena itu setiap balasan di sini menyertakan aturan mana yang dipakai dan
//! berapa nilai keutamaannya. Ilusinya jauh lebih tipis kalau mesinnya
//! kelihatan.

use crate::rng::SplitMix64;
use serde::{Deserialize, Serialize};

/// Kesalahan pada mesin ELIZA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElizaError {
    /// Naskah tidak punya aturan sama sekali.
    EmptyScript,
    /// Sebuah aturan tidak punya balasan.
    RuleWithoutResponses(String),
    /// Masukan melebihi panjang yang wajar.
    InputTooLong(usize),
}

impl core::fmt::Display for ElizaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ElizaError::EmptyScript => write!(f, "naskah tidak punya aturan"),
            ElizaError::RuleWithoutResponses(k) => {
                write!(f, "aturan {k} tidak punya balasan")
            }
            ElizaError::InputTooLong(n) => {
                write!(f, "masukan {n} karakter melebihi batas {MAX_INPUT}")
            }
        }
    }
}

/// Batas panjang masukan.
pub const MAX_INPUT: usize = 1_000;

/// Satu aturan pencocokan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    /// Kata kunci yang dicari di dalam masukan.
    pub keyword: String,
    /// Nilai keutamaan; aturan bernilai lebih tinggi menang.
    pub priority: i32,
    /// Balasan yang mungkin. `{}` diganti sisa kalimat setelah kata kunci.
    pub responses: Vec<String>,
}

/// Naskah ELIZA: kumpulan aturan beserta balasan cadangan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Script {
    /// Nama naskah.
    pub name: String,
    /// Aturan-aturannya.
    pub rules: Vec<Rule>,
    /// Balasan saat tidak ada kata kunci yang cocok.
    pub fallbacks: Vec<String>,
    /// Pasangan penukaran kata ganti, dari sudut pandang pengguna ke mesin.
    pub reflections: Vec<(String, String)>,
}

impl Script {
    /// Memeriksa kesahihan naskah.
    pub fn validate(&self) -> Result<(), ElizaError> {
        if self.rules.is_empty() {
            return Err(ElizaError::EmptyScript);
        }
        for rule in &self.rules {
            if rule.responses.is_empty() {
                return Err(ElizaError::RuleWithoutResponses(rule.keyword.clone()));
            }
        }
        if self.fallbacks.is_empty() {
            return Err(ElizaError::RuleWithoutResponses("cadangan".into()));
        }
        Ok(())
    }
}

/// Balasan ELIZA beserta penjelasan bagaimana ia dihasilkan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reply {
    /// Teks balasan.
    pub text: String,
    /// Kata kunci yang cocok, kosong bila memakai balasan cadangan.
    pub matched_keyword: String,
    /// Nilai keutamaan aturan yang menang.
    pub priority: i32,
    /// Bagian kalimat pengguna yang dipantulkan kembali, setelah penukaran.
    pub reflected_fragment: String,
    /// Benar bila tidak ada aturan yang cocok.
    pub used_fallback: bool,
}

/// Menukar kata ganti agar kalimat terdengar dipantulkan.
///
/// Inilah trik yang membuat ELIZA terasa hidup: "saya sedih" berubah menjadi
/// "Anda sedih". Tidak ada pemahaman di dalamnya, hanya tabel penukaran.
pub fn reflect(text: &str, reflections: &[(String, String)]) -> String {
    text.split_whitespace()
        .map(|word| {
            let bersih: String = word
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-')
                .collect::<String>()
                .to_lowercase();
            reflections
                .iter()
                .find(|(from, _)| *from == bersih)
                .map(|(_, to)| to.clone())
                .unwrap_or(bersih)
        })
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Menghasilkan balasan untuk sebuah masukan.
///
/// `seed` menentukan balasan mana yang dipilih bila sebuah aturan punya
/// beberapa. Benih yang eksplisit membuat percakapan bisa diulang persis —
/// tanpa itu, memperagakan cacat tertentu menjadi mustahil.
pub fn respond(script: &Script, input: &str, seed: u64) -> Result<Reply, ElizaError> {
    script.validate()?;
    if input.chars().count() > MAX_INPUT {
        return Err(ElizaError::InputTooLong(input.chars().count()));
    }

    let lower = input.to_lowercase();
    let mut rng = SplitMix64::new(seed);

    // Aturan dengan keutamaan tertinggi menang; seri diputus urutan penulisan,
    // sehingga naskah bisa disusun dengan sengaja.
    let mut best: Option<(&Rule, usize)> = None;
    for rule in &script.rules {
        if let Some(at) = find_keyword(&lower, &rule.keyword) {
            let menang = match best {
                Some((current, _)) => rule.priority > current.priority,
                None => true,
            };
            if menang {
                best = Some((rule, at + rule.keyword.chars().count()));
            }
        }
    }

    match best {
        Some((rule, after)) => {
            let fragment: String = lower.chars().skip(after).collect();
            let reflected = reflect(fragment.trim(), &script.reflections);
            let template = &rule.responses[rng.below(rule.responses.len() as u64) as usize];
            let text = if template.contains("{}") {
                if reflected.is_empty() {
                    // Templat menuntut potongan kalimat, tetapi tidak ada yang
                    // tersisa. Memasang teks kosong menghasilkan kalimat
                    // menggantung, jadi dipakai balasan cadangan.
                    let fb = &script.fallbacks[rng.below(script.fallbacks.len() as u64) as usize];
                    return Ok(Reply {
                        text: fb.clone(),
                        matched_keyword: rule.keyword.clone(),
                        priority: rule.priority,
                        reflected_fragment: String::new(),
                        used_fallback: true,
                    });
                }
                template.replace("{}", &reflected)
            } else {
                template.clone()
            };
            Ok(Reply {
                text,
                matched_keyword: rule.keyword.clone(),
                priority: rule.priority,
                reflected_fragment: reflected,
                used_fallback: false,
            })
        }
        None => {
            let fb = &script.fallbacks[rng.below(script.fallbacks.len() as u64) as usize];
            Ok(Reply {
                text: fb.clone(),
                matched_keyword: String::new(),
                priority: 0,
                reflected_fragment: String::new(),
                used_fallback: true,
            })
        }
    }
}

/// Mencari kata kunci sebagai kata utuh, mengembalikan posisi karakternya.
///
/// Pencocokan sebagai potongan teks biasa akan menemukan "aku" di dalam
/// "akuntansi", dan balasannya menjadi janggal tanpa sebab yang jelas.
fn find_keyword(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }
    let words: Vec<&str> = needle.split_whitespace().collect();
    let hay: Vec<char> = haystack.chars().collect();
    let target: Vec<char> = words.join(" ").chars().collect();

    // Kata kunci yang lebih panjang daripada masukannya jelas tidak mungkin
    // cocok. Tanpa penjagaan ini, `saturating_sub` menghasilkan nol dan
    // pengirisan di bawah mengindeks di luar batas — masukan kosong saja sudah
    // cukup untuk memicunya.
    if target.is_empty() || target.len() > hay.len() {
        return None;
    }

    let is_boundary = |c: Option<&char>| match c {
        None => true,
        Some(c) => !c.is_alphanumeric(),
    };

    for start in 0..=(hay.len() - target.len()) {
        if hay[start..start + target.len()] != target[..] {
            continue;
        }
        let before = if start == 0 { None } else { hay.get(start - 1) };
        let after = hay.get(start + target.len());
        if is_boundary(before) && is_boundary(after) {
            return Some(start);
        }
    }
    None
}

/// Naskah ELIZA berbahasa Indonesia, meniru gaya psikoterapis Rogerian asli.
pub fn indonesian_script() -> Script {
    let rule = |keyword: &str, priority: i32, responses: &[&str]| Rule {
        keyword: keyword.to_string(),
        priority,
        responses: responses.iter().map(|r| r.to_string()).collect(),
    };

    Script {
        name: "ELIZA Bahasa Indonesia".to_string(),
        rules: vec![
            rule(
                "saya merasa",
                10,
                &[
                    "Sudah berapa lama Anda merasa {}?",
                    "Menurut Anda, apa yang membuat Anda merasa {}?",
                ],
            ),
            rule(
                "saya ingin",
                9,
                &[
                    "Apa yang akan berubah kalau Anda memperoleh {}?",
                    "Mengapa Anda menginginkan {}?",
                ],
            ),
            rule(
                "saya tidak bisa",
                9,
                &[
                    "Apa yang menghalangi Anda {}?",
                    "Menurut Anda, apa yang harus berubah agar Anda bisa {}?",
                ],
            ),
            rule(
                "saya",
                5,
                &[
                    "Ceritakan lebih banyak tentang {}.",
                    "Mengapa Anda berkata {}?",
                ],
            ),
            rule(
                "ibu",
                8,
                &[
                    "Ceritakan lebih banyak tentang keluarga Anda.",
                    "Bagaimana hubungan Anda dengan ibu Anda?",
                ],
            ),
            rule(
                "ayah",
                8,
                &[
                    "Bagaimana perasaan Anda terhadap ayah Anda?",
                    "Ceritakan lebih banyak tentang keluarga Anda.",
                ],
            ),
            rule(
                "kenapa",
                6,
                &[
                    "Menurut Anda sendiri, kenapa?",
                    "Apakah pertanyaan itu sering muncul di pikiran Anda?",
                ],
            ),
            rule(
                "tidak",
                3,
                &["Mengapa tidak?", "Anda terdengar cukup yakin."],
            ),
            rule(
                "ya",
                3,
                &["Anda terdengar yakin.", "Ceritakan lebih lanjut."],
            ),
            rule(
                "teman",
                7,
                &[
                    "Apa arti teman-teman itu bagi Anda?",
                    "Ceritakan lebih banyak tentang mereka.",
                ],
            ),
            rule(
                "halo",
                4,
                &[
                    "Halo. Apa yang ingin Anda ceritakan?",
                    "Halo. Apa kabar Anda hari ini?",
                ],
            ),
        ],
        fallbacks: vec![
            "Coba ceritakan lebih lanjut.".to_string(),
            "Menarik. Lanjutkan.".to_string(),
            "Apa yang membuat Anda memikirkan hal itu?".to_string(),
            "Bisakah Anda menjelaskannya dengan cara lain?".to_string(),
        ],
        reflections: vec![
            ("saya".into(), "Anda".into()),
            ("aku".into(), "Anda".into()),
            ("saya".into(), "Anda".into()),
            ("kamu".into(), "saya".into()),
            ("anda".into(), "saya".into()),
            ("milikku".into(), "milik Anda".into()),
            ("punyaku".into(), "punya Anda".into()),
            ("diriku".into(), "diri Anda".into()),
            ("dirimu".into(), "diri saya".into()),
            ("kami".into(), "kalian".into()),
        ],
    }
}

/// Ringkasan sebuah naskah, dipakai untuk membongkar cara kerjanya.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptSummary {
    /// Nama naskah.
    pub name: String,
    /// Jumlah aturan.
    pub rules: usize,
    /// Total kalimat balasan yang tersedia.
    pub total_responses: usize,
    /// Jumlah balasan cadangan.
    pub fallbacks: usize,
    /// Jumlah pasangan penukaran kata ganti.
    pub reflections: usize,
    /// Kata kunci beserta keutamaannya, terurut dari yang tertinggi.
    pub keywords: Vec<(String, i32)>,
}

/// Meringkas sebuah naskah.
///
/// Angka-angka ini yang membongkar ilusinya: seluruh "kecerdasan" ELIZA
/// muat dalam beberapa lusin kalimat yang ditulis manusia.
pub fn summarise(script: &Script) -> ScriptSummary {
    let mut keywords: Vec<(String, i32)> = script
        .rules
        .iter()
        .map(|r| (r.keyword.clone(), r.priority))
        .collect();
    keywords.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    ScriptSummary {
        name: script.name.clone(),
        rules: script.rules.len(),
        total_responses: script.rules.iter().map(|r| r.responses.len()).sum(),
        fallbacks: script.fallbacks.len(),
        reflections: script.reflections.len(),
        keywords,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s() -> Script {
        indonesian_script()
    }

    #[test]
    fn naskah_bawaan_sah() {
        assert!(s().validate().is_ok());
    }

    #[test]
    fn validasi_menolak_naskah_rusak() {
        let kosong = Script {
            name: "x".into(),
            rules: vec![],
            fallbacks: vec!["a".into()],
            reflections: vec![],
        };
        assert_eq!(kosong.validate(), Err(ElizaError::EmptyScript));

        let mut tanpa_balasan = s();
        tanpa_balasan.rules[0].responses.clear();
        assert!(matches!(
            tanpa_balasan.validate(),
            Err(ElizaError::RuleWithoutResponses(_))
        ));

        let mut tanpa_cadangan = s();
        tanpa_cadangan.fallbacks.clear();
        assert!(matches!(
            tanpa_cadangan.validate(),
            Err(ElizaError::RuleWithoutResponses(_))
        ));
    }

    #[test]
    fn penukaran_kata_ganti() {
        let r = s().reflections;
        assert_eq!(reflect("saya sedih", &r), "Anda sedih");
        assert_eq!(reflect("aku lelah", &r), "Anda lelah");
        assert_eq!(reflect("kamu tahu", &r), "saya tahu");
        // Kata yang tidak ada di tabel dibiarkan apa adanya.
        assert_eq!(reflect("hujan deras", &r), "hujan deras");
    }

    #[test]
    fn penukaran_membuang_tanda_baca() {
        let r = s().reflections;
        assert_eq!(reflect("saya sedih.", &r), "Anda sedih");
        assert_eq!(reflect("  aku  ", &r), "Anda");
        assert_eq!(reflect("", &r), "");
    }

    #[test]
    fn kata_kunci_dicocokkan_sebagai_kata_utuh() {
        // Pencocokan potongan teks biasa akan menemukan "ya" di dalam
        // "budaya", dan balasannya menjadi janggal tanpa sebab yang jelas.
        assert!(find_keyword("saya suka budaya jawa", "ya").is_none());
        assert!(find_keyword("ya benar", "ya").is_some());
        assert!(find_keyword("apakah ya?", "ya").is_some());
        assert!(find_keyword("", "ya").is_none());
        assert!(find_keyword("apa saja", "").is_none());
        // Regresi: kata kunci yang lebih panjang daripada masukan pernah
        // memicu pengindeksan di luar batas, dan masukan kosong sudah cukup
        // untuk memicunya.
        assert!(find_keyword("hi", "kata kunci yang panjang").is_none());
        assert!(find_keyword("", "").is_none());
    }

    #[test]
    fn kata_kunci_beberapa_kata() {
        assert!(find_keyword("saya merasa sedih", "saya merasa").is_some());
        assert!(find_keyword("saya sangat merasa sedih", "saya merasa").is_none());
    }

    #[test]
    fn balasan_memakai_aturan_berkeutamaan_tertinggi() {
        // "saya merasa" (10) harus mengalahkan "saya" (5) walau keduanya cocok.
        let hasil = respond(&s(), "saya merasa sedih", 1).unwrap();
        assert_eq!(hasil.matched_keyword, "saya merasa");
        assert_eq!(hasil.priority, 10);
        assert!(!hasil.used_fallback);
    }

    #[test]
    fn balasan_memantulkan_kalimat_pengguna() {
        let hasil = respond(&s(), "saya merasa sedih hari ini", 1).unwrap();
        assert_eq!(hasil.reflected_fragment, "sedih hari ini");
        assert!(
            hasil.text.contains("sedih hari ini"),
            "balasan: {}",
            hasil.text
        );
    }

    #[test]
    fn templat_tanpa_potongan_jatuh_ke_cadangan() {
        // "saya merasa" tanpa lanjutan apa pun akan menghasilkan kalimat
        // menggantung bila potongan kosong dipasang begitu saja.
        let hasil = respond(&s(), "saya merasa", 1).unwrap();
        assert!(hasil.used_fallback, "balasan: {}", hasil.text);
        assert!(!hasil.text.contains("{}"));
        assert!(!hasil.text.ends_with(" ?"));
    }

    #[test]
    fn balasan_tidak_pernah_memuat_penanda_templat() {
        let script = s();
        for masukan in [
            "saya merasa sedih",
            "saya ingin pulang",
            "saya tidak bisa tidur",
            "ibu saya",
            "halo",
            "entah apa ini",
            "saya",
            "ya",
        ] {
            for seed in 0..8u64 {
                let hasil = respond(&script, masukan, seed).unwrap();
                assert!(
                    !hasil.text.contains("{}"),
                    "penanda templat bocor pada {masukan:?}: {}",
                    hasil.text
                );
                assert!(!hasil.text.trim().is_empty());
            }
        }
    }

    #[test]
    fn masukan_tanpa_kata_kunci_memakai_cadangan() {
        let hasil = respond(&s(), "cuaca hari ini cerah sekali", 1).unwrap();
        assert!(hasil.used_fallback);
        assert!(hasil.matched_keyword.is_empty());
        assert!(s().fallbacks.contains(&hasil.text));
    }

    #[test]
    fn balasan_deterministik_untuk_benih_sama() {
        let script = s();
        let a = respond(&script, "saya merasa sedih", 42).unwrap();
        let b = respond(&script, "saya merasa sedih", 42).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn benih_berbeda_bisa_menghasilkan_balasan_berbeda() {
        let script = s();
        let semua: std::collections::BTreeSet<String> = (0..20u64)
            .map(|seed| respond(&script, "saya merasa sedih", seed).unwrap().text)
            .collect();
        assert!(semua.len() > 1, "balasan tidak pernah bervariasi");
    }

    #[test]
    fn masukan_terlalu_panjang_ditolak() {
        let panjang = "a".repeat(MAX_INPUT + 1);
        assert!(matches!(
            respond(&s(), &panjang, 1),
            Err(ElizaError::InputTooLong(_))
        ));
    }

    #[test]
    fn masukan_kosong_tetap_dijawab() {
        let hasil = respond(&s(), "", 1).unwrap();
        assert!(hasil.used_fallback);
        assert!(!hasil.text.is_empty());
    }

    #[test]
    fn huruf_besar_tidak_mempengaruhi_pencocokan() {
        let a = respond(&s(), "SAYA MERASA SEDIH", 1).unwrap();
        let b = respond(&s(), "saya merasa sedih", 1).unwrap();
        assert_eq!(a.matched_keyword, b.matched_keyword);
        assert_eq!(a.text, b.text);
    }

    #[test]
    fn ringkasan_membongkar_ukuran_sebenarnya() {
        // Angka inilah pelajarannya: seluruh "kecerdasan" ELIZA muat dalam
        // beberapa lusin kalimat yang ditulis manusia.
        let r = summarise(&s());
        assert_eq!(r.rules, 11);
        assert!(
            r.total_responses < 40,
            "hanya {} kalimat",
            r.total_responses
        );
        assert!(r.fallbacks >= 3);
        assert!(r.reflections >= 8);
        // Kata kunci terurut dari keutamaan tertinggi.
        for w in r.keywords.windows(2) {
            assert!(w[0].1 >= w[1].1);
        }
        assert_eq!(r.keywords[0].0, "saya merasa");
    }

    #[test]
    fn hasil_bisa_di_serialisasi() {
        let hasil = respond(&s(), "saya merasa sedih", 1).unwrap();
        let json = serde_json::to_string(&hasil).unwrap();
        assert_eq!(serde_json::from_str::<Reply>(&json).unwrap(), hasil);

        let script = s();
        let sj = serde_json::to_string(&script).unwrap();
        assert_eq!(serde_json::from_str::<Script>(&sj).unwrap(), script);
    }

    #[test]
    fn pesan_error_terbaca() {
        for e in [
            ElizaError::EmptyScript,
            ElizaError::RuleWithoutResponses("x".into()),
            ElizaError::InputTooLong(2000),
        ] {
            assert!(!e.to_string().is_empty());
        }
    }
}
