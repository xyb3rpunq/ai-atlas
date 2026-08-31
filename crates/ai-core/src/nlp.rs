//! Sesi 10 — Pemrosesan Bahasa Alami.
//!
//! Pemenggalan kata, penghapusan kata henti, pencarian kata dasar Bahasa
//! Indonesia dengan algoritma Nazief-Adriani, pembobotan TF-IDF, kemiripan
//! kosinus, n-gram, dan jarak sunting Levenshtein.
//!
//! Bagian yang paling banyak menuntut kehati-hatian adalah pencarian kata
//! dasarnya. Bahasa Indonesia punya imbuhan yang berlapis dan sebagian
//! mengubah huruf pertama kata dasarnya — `menyapu` berasal dari `sapu`, bukan
//! `nyapu`. Aturan peluluhan seperti itulah yang membuat algoritma untuk
//! Bahasa Inggris tidak bisa dipakai begitu saja di sini.
//!
//! Rujukan: Nazief, B. & Adriani, M. (1996), *Confix-Stripping: Approach to
//! Stemming Algorithm for Bahasa Indonesia*, Universitas Indonesia.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Kesalahan pada pemrosesan bahasa.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NlpError {
    /// Korpus kosong.
    EmptyCorpus,
    /// Dua vektor yang dibandingkan berbeda panjang.
    LengthMismatch {
        /// Panjang vektor pertama.
        a: usize,
        /// Panjang vektor kedua.
        b: usize,
    },
    /// Ukuran n-gram tidak masuk akal.
    BadNgramSize(usize),
}

impl core::fmt::Display for NlpError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NlpError::EmptyCorpus => write!(f, "korpus kosong"),
            NlpError::LengthMismatch { a, b } => {
                write!(f, "panjang vektor berbeda: {a} dan {b}")
            }
            NlpError::BadNgramSize(n) => write!(f, "ukuran n-gram harus minimal 1, diberi {n}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Pemenggalan
// ---------------------------------------------------------------------------

/// Memenggal teks menjadi kata, membuang tanda baca dan menyeragamkan huruf.
///
/// Tanda hubung dipertahankan di tengah kata karena Bahasa Indonesia memakainya
/// untuk pengulangan: `anak-anak` adalah satu kata, bukan dua.
pub fn tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            current.extend(ch.to_lowercase());
        } else if ch == '-' && !current.is_empty() {
            current.push('-');
        } else if !current.is_empty() {
            out.push(current.trim_matches('-').to_string());
            current = String::new();
        }
    }
    if !current.is_empty() {
        out.push(current.trim_matches('-').to_string());
    }
    out.retain(|t| !t.is_empty());
    out
}

/// Memenggal teks menjadi kalimat berdasarkan tanda baca akhir.
pub fn sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        current.push(ch);
        if matches!(ch, '.' | '!' | '?') {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                out.push(trimmed.to_string());
            }
            current = String::new();
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        out.push(trimmed.to_string());
    }
    out
}

/// Kata henti Bahasa Indonesia yang paling sering muncul.
///
/// Daftar ringkas yang sengaja dijaga kecil: daftar yang terlalu panjang
/// membuang kata yang justru membedakan makna, misalnya `tidak` pada analisis
/// sentimen — membuangnya mengubah "tidak bagus" menjadi "bagus".
pub const STOPWORDS_ID: &[&str] = &[
    "yang", "dan", "di", "ke", "dari", "ini", "itu", "dengan", "untuk", "pada", "adalah", "dalam",
    "akan", "atau", "juga", "sudah", "saya", "kami", "kita", "mereka", "dia", "ada", "oleh",
    "karena", "bahwa", "sebagai", "para", "telah", "dapat", "lebih", "agar", "bila", "jika",
    "maka", "namun", "tetapi", "serta", "hanya", "saat", "ketika",
];

/// Kata henti Bahasa Inggris yang paling sering muncul.
pub const STOPWORDS_EN: &[&str] = &[
    "the", "a", "an", "and", "or", "but", "of", "to", "in", "on", "at", "for", "with", "is", "are",
    "was", "were", "be", "been", "this", "that", "these", "those", "it", "its", "as", "by", "from",
    "has", "have", "had", "will", "would", "can", "could",
];

/// Membuang kata henti dari daftar token.
pub fn remove_stopwords(tokens: &[String], stopwords: &[&str]) -> Vec<String> {
    let set: BTreeSet<&str> = stopwords.iter().copied().collect();
    tokens
        .iter()
        .filter(|t| !set.contains(t.as_str()))
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// Pencarian kata dasar Bahasa Indonesia
// ---------------------------------------------------------------------------

/// Kamus kata dasar ringkas, cukup untuk memperagakan algoritmanya.
///
/// Algoritma Nazief-Adriani memerlukan kamus untuk memutuskan kapan berhenti
/// mengupas. Tanpa kamus, `beruang` akan dikupas menjadi `uang` — kesalahan
/// yang tidak bisa dihindari aturan mana pun.
pub const DICTIONARY_ID: &[&str] = &[
    "ajar",
    "ambil",
    "anak",
    "asing",
    "atur",
    "baca",
    "bagus",
    "bantu",
    "bawa",
    "beli",
    "benar",
    "beri",
    "besar",
    "buat",
    "buruk",
    "cari",
    "cepat",
    "cinta",
    "coba",
    "cuci",
    "dapat",
    "datang",
    "dengar",
    "duduk",
    "gagal",
    "ganti",
    "guna",
    "hasil",
    "hitung",
    "jalan",
    "jawab",
    "jual",
    "kata",
    "kembang",
    "kenal",
    "kerja",
    "kirim",
    "lapor",
    "lihat",
    "main",
    "makan",
    "masak",
    "minum",
    "nilai",
    "pakai",
    "pikir",
    "pukul",
    "sapu",
    "simpan",
    "suka",
    "tanya",
    "tulis",
    "tunggu",
    "ubah",
    "uji",
    "ukur",
    "beruang",
    "uang",
    "belajar",
    "pelajar",
    "pelajaran",
    "pengajar",
    "pengajaran",
    "hasilkan",
    "kebun",
    "pandang",
    "tinggal",
    "tumbuh",
    "sedih",
    "senang",
    "marah",
    "kecewa",
    "puas",
    "mahal",
    "murah",
    "lambat",
    "rusak",
    "bersih",
    "kotor",
    "ramah",
    "buku",
    "lengkap",
    "petugas",
    "layan",
    "pustaka",
    "perpustakaan",
    "kampus",
    "ikan",
    "kucing",
    "mobil",
    "lintas",
];

/// Awalan yang tidak mengubah huruf pertama kata dasar.
const SIMPLE_PREFIXES: &[&str] = &["di", "ke", "se", "ter", "be", "per"];

/// Akhiran partikel, dikupas paling awal.
const PARTICLES: &[&str] = &["lah", "kah", "tah", "pun"];

/// Akhiran kata ganti kepemilikan.
const POSSESSIVES: &[&str] = &["ku", "mu", "nya"];

/// Akhiran turunan.
const SUFFIXES: &[&str] = &["kan", "an", "i"];

/// Satu langkah pengupasan, dipakai untuk menampilkan jejak.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StemStep {
    /// Jenis imbuhan yang dikupas.
    pub kind: String,
    /// Imbuhan yang dibuang.
    pub affix: String,
    /// Bentuk kata setelah langkah ini.
    pub result: String,
}

/// Hasil pencarian kata dasar beserta jejaknya.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StemResult {
    /// Kata masukan.
    pub original: String,
    /// Kata dasar yang ditemukan.
    pub stem: String,
    /// Langkah-langkah pengupasan.
    pub steps: Vec<StemStep>,
    /// Apakah hasilnya ada di dalam kamus.
    pub in_dictionary: bool,
}

/// Membuang satu akhiran bila cocok, mengembalikan sisa kata dan akhirannya.
///
/// Syarat panjang minimum mencegah kata pendek habis dikupas menjadi kosong;
/// tanpa itu, `kaki` akan kehilangan `i` dan menjadi `kak`.
fn strip_suffix<'a>(word: &'a str, list: &[&str]) -> Option<(&'a str, String)> {
    for candidate in list {
        if word.len() > candidate.len() + 2 && word.ends_with(candidate) {
            // Pengindeksan bita aman karena seluruh imbuhan di modul ini ASCII.
            let cut = word.len() - candidate.len();
            return Some((&word[..cut], (*candidate).to_string()));
        }
    }
    None
}

/// Mencari kata dasar sebuah kata dengan pendekatan Nazief-Adriani.
///
/// Urutan pengupasannya penting dan bukan sembarang: partikel, lalu kata ganti
/// kepemilikan, lalu akhiran turunan, baru awalan. Membalik urutannya
/// menghasilkan kata dasar yang salah pada kata berimbuhan rangkap.
pub fn stem_id(word: &str, dictionary: &[&str]) -> StemResult {
    let dict: BTreeSet<&str> = dictionary.iter().copied().collect();
    let lower = word.to_lowercase();
    let mut steps = Vec::new();

    // Kata yang sudah ada di kamus tidak dikupas sama sekali. Inilah yang
    // menyelamatkan `beruang` dari menjadi `uang`.
    if dict.contains(lower.as_str()) {
        return StemResult {
            original: word.to_string(),
            stem: lower,
            steps,
            in_dictionary: true,
        };
    }

    // Kata ulang seperti `anak-anak` diperlakukan sebagai satu bentuknya saja.
    let mut current = match lower.split_once('-') {
        Some((first, second)) if first == second => {
            steps.push(StemStep {
                kind: "kata ulang".into(),
                affix: format!("-{second}"),
                result: first.to_string(),
            });
            first.to_string()
        }
        _ => lower.clone(),
    };
    if dict.contains(current.as_str()) {
        return StemResult {
            original: word.to_string(),
            stem: current,
            steps,
            in_dictionary: true,
        };
    }

    // 1. Partikel.
    if let Some((rest, affix)) = strip_suffix(&current, PARTICLES) {
        current = rest.to_string();
        steps.push(StemStep {
            kind: "partikel".into(),
            affix,
            result: current.clone(),
        });
    }
    // 2. Kata ganti kepemilikan.
    if let Some((rest, affix)) = strip_suffix(&current, POSSESSIVES) {
        current = rest.to_string();
        steps.push(StemStep {
            kind: "kepemilikan".into(),
            affix: affix.to_string(),
            result: current.clone(),
        });
    }
    if dict.contains(current.as_str()) {
        return StemResult {
            original: word.to_string(),
            stem: current,
            steps,
            in_dictionary: true,
        };
    }

    // 3. Akhiran turunan. Bentuk sebelum pengupasan disimpan supaya bisa
    //    dipulihkan bila ternyata pengupasannya tidak menghasilkan kata dasar.
    let before_suffix = current.clone();
    let suffix_steps = steps.len();
    if let Some((rest, affix)) = strip_suffix(&current, SUFFIXES) {
        current = rest.to_string();
        steps.push(StemStep {
            kind: "akhiran".into(),
            affix: affix.to_string(),
            result: current.clone(),
        });
    }
    if dict.contains(current.as_str()) {
        return StemResult {
            original: word.to_string(),
            stem: current,
            steps,
            in_dictionary: true,
        };
    }

    // 4. Awalan, termasuk yang meluluhkan huruf pertama kata dasarnya.
    if let Some((affix, candidate, found)) = strip_prefix(&current, &dict) {
        steps.push(StemStep {
            kind: "awalan".into(),
            affix,
            result: candidate.clone(),
        });
        if found {
            return StemResult {
                original: word.to_string(),
                stem: candidate,
                steps,
                in_dictionary: true,
            };
        }
        current = candidate;
    }

    if dict.contains(current.as_str()) {
        return StemResult {
            original: word.to_string(),
            stem: current,
            steps,
            in_dictionary: true,
        };
    }

    // Pengupasan akhiran yang tidak menghasilkan kata kamus dibatalkan.
    // Tanpa langkah ini, kata yang bukan bentukan berakhiran akan tercacah.
    if steps.len() > suffix_steps && !dict.contains(current.as_str()) {
        let awalan_saja = strip_prefix(&before_suffix, &dict).map(|(_, candidate, _)| candidate);
        if let Some(alt) = awalan_saja {
            if dict.contains(alt.as_str()) {
                steps.truncate(suffix_steps);
                steps.push(StemStep {
                    kind: "awalan".into(),
                    affix: "(akhiran dipulihkan)".into(),
                    result: alt.clone(),
                });
                return StemResult {
                    original: word.to_string(),
                    stem: alt,
                    steps,
                    in_dictionary: true,
                };
            }
        }
    }

    StemResult {
        original: word.to_string(),
        stem: current,
        steps,
        in_dictionary: false,
    }
}

/// Seluruh kata dasar yang mungkin setelah sebuah awalan dibuang.
///
/// Mengembalikan pasangan `(awalan, kandidat)`, terurut dari yang paling
/// mungkin. Pemanggil memilih kandidat pertama yang ada di kamus.
///
/// Kandidat jamak bukan kemewahan, melainkan syarat kebenaran. Awalan `mem-`
/// meluluhkan huruf `p` pada `memukul`, yang berasal dari `pukul`. Tetapi
/// `membaca` juga berawalan `mem-` dan berasal dari `baca`, tanpa peluluhan
/// apa pun. Aturan tunggal yang selalu memulihkan `p` akan mengubah
/// `membacakan` menjadi `pbaca` — bentuk yang bukan kata dan tidak akan pernah
/// ada di kamus mana pun. Satu-satunya cara memutuskan adalah mencoba keduanya
/// dan bertanya kepada kamus.
fn prefix_candidates(word: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();

    // Awalan bernasal, beserta huruf yang mungkin luluh di baliknya.
    // Daftar huruf kosong berarti awalannya bisa dipakai tanpa peluluhan.
    let nasal: &[(&str, &[&str])] = &[
        ("meny", &["s"]),
        ("peny", &["s"]),
        ("meng", &["k", "", "g", "h"]),
        ("peng", &["k", "", "g", "h"]),
        ("mem", &["p", "", "b", "f"]),
        ("pem", &["p", "", "b", "f"]),
        ("men", &["t", "", "d"]),
        ("pen", &["t", "", "d"]),
        ("mer", &[""]),
        ("per", &[""]),
    ];

    for (prefix, restorations) in nasal {
        if word.len() > prefix.len() + 2 && word.starts_with(prefix) {
            let rest = &word[prefix.len()..];
            for restore in *restorations {
                let candidate = if restore.is_empty() {
                    rest.to_string()
                } else {
                    format!("{restore}{rest}")
                };
                // Kandidat sependek satu atau dua huruf hampir pasti keliru.
                if candidate.len() >= 3 {
                    out.push(((*prefix).to_string(), candidate));
                }
            }
        }
    }

    for prefix in ["me", "pe"] {
        if word.len() > prefix.len() + 2 && word.starts_with(prefix) {
            out.push((prefix.to_string(), word[prefix.len()..].to_string()));
        }
    }

    for prefix in SIMPLE_PREFIXES {
        if word.len() > prefix.len() + 2 && word.starts_with(prefix) {
            out.push(((*prefix).to_string(), word[prefix.len()..].to_string()));
        }
    }

    out
}

/// Memilih kandidat kata dasar terbaik setelah awalan dibuang.
///
/// Kandidat yang ada di kamus selalu menang. Bila tidak ada satu pun yang
/// dikenali, dikembalikan kandidat pertama supaya kata di luar kamus tetap
/// terkupas dengan cara yang wajar, bukan dibiarkan utuh.
fn strip_prefix(word: &str, dict: &BTreeSet<&str>) -> Option<(String, String, bool)> {
    let candidates = prefix_candidates(word);
    if candidates.is_empty() {
        return None;
    }
    for (affix, candidate) in &candidates {
        if dict.contains(candidate.as_str()) {
            return Some((affix.clone(), candidate.clone(), true));
        }
    }
    let (affix, candidate) = candidates.into_iter().next()?;
    Some((affix, candidate, false))
}

/// Mencari kata dasar seluruh token.
pub fn stem_all(tokens: &[String], dictionary: &[&str]) -> Vec<String> {
    tokens.iter().map(|t| stem_id(t, dictionary).stem).collect()
}

// ---------------------------------------------------------------------------
// Pembobotan dan kemiripan
// ---------------------------------------------------------------------------

/// Frekuensi tiap kata dalam sebuah dokumen.
pub fn term_frequency(tokens: &[String]) -> BTreeMap<String, f64> {
    let mut counts: BTreeMap<String, f64> = BTreeMap::new();
    for t in tokens {
        *counts.entry(t.clone()).or_insert(0.0) += 1.0;
    }
    let total = tokens.len() as f64;
    if total > 0.0 {
        for v in counts.values_mut() {
            *v /= total;
        }
    }
    counts
}

/// Bobot TF-IDF sebuah korpus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TfIdf {
    /// Kosakata, terurut. Menjadi urutan kolom pada tiap vektor.
    pub vocabulary: Vec<String>,
    /// Nilai IDF tiap kata.
    pub idf: Vec<f64>,
    /// Vektor TF-IDF tiap dokumen.
    pub vectors: Vec<Vec<f64>>,
}

/// Menghitung bobot TF-IDF sebuah korpus.
///
/// IDF memakai bentuk yang dihaluskan, `ln((1 + N) / (1 + df)) + 1`. Bentuk
/// mentah `ln(N / df)` bernilai nol untuk kata yang muncul di semua dokumen,
/// sehingga kata itu hilang sama sekali dari perhitungan — padahal pada korpus
/// kecil hal itu sering terjadi pada kata yang justru penting.
pub fn tf_idf(documents: &[Vec<String>]) -> Result<TfIdf, NlpError> {
    if documents.is_empty() {
        return Err(NlpError::EmptyCorpus);
    }
    let vocabulary: Vec<String> = documents
        .iter()
        .flatten()
        .cloned()
        .collect::<BTreeSet<String>>()
        .into_iter()
        .collect();

    let n = documents.len() as f64;
    let idf: Vec<f64> = vocabulary
        .iter()
        .map(|term| {
            let df = documents.iter().filter(|d| d.contains(term)).count() as f64;
            ((1.0 + n) / (1.0 + df)).ln() + 1.0
        })
        .collect();

    let vectors = documents
        .iter()
        .map(|doc| {
            let tf = term_frequency(doc);
            vocabulary
                .iter()
                .enumerate()
                .map(|(i, term)| tf.get(term).copied().unwrap_or(0.0) * idf[i])
                .collect()
        })
        .collect();

    Ok(TfIdf {
        vocabulary,
        idf,
        vectors,
    })
}

/// Kemiripan kosinus dua vektor.
///
/// Vektor nol tidak punya arah, jadi kemiripannya dilaporkan nol alih-alih
/// menghasilkan pembagian dengan nol.
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> Result<f64, NlpError> {
    if a.len() != b.len() {
        return Err(NlpError::LengthMismatch {
            a: a.len(),
            b: b.len(),
        });
    }
    let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a < 1e-12 || norm_b < 1e-12 {
        return Ok(0.0);
    }
    Ok((dot / (norm_a * norm_b)).clamp(-1.0, 1.0))
}

/// Kemiripan Jaccard dua himpunan kata.
pub fn jaccard_similarity(a: &[String], b: &[String]) -> f64 {
    let set_a: BTreeSet<&String> = a.iter().collect();
    let set_b: BTreeSet<&String> = b.iter().collect();
    let intersection = set_a.intersection(&set_b).count() as f64;
    let union = set_a.union(&set_b).count() as f64;
    if union < 1.0 {
        return 0.0;
    }
    intersection / union
}

/// N-gram dari daftar token.
pub fn ngrams(tokens: &[String], n: usize) -> Result<Vec<String>, NlpError> {
    if n == 0 {
        return Err(NlpError::BadNgramSize(n));
    }
    if tokens.len() < n {
        return Ok(Vec::new());
    }
    Ok(tokens.windows(n).map(|w| w.join(" ")).collect())
}

/// N-gram karakter dari sebuah kata.
pub fn char_ngrams(word: &str, n: usize) -> Result<Vec<String>, NlpError> {
    if n == 0 {
        return Err(NlpError::BadNgramSize(n));
    }
    let chars: Vec<char> = word.chars().collect();
    if chars.len() < n {
        return Ok(Vec::new());
    }
    Ok(chars
        .windows(n)
        .map(|w| w.iter().collect::<String>())
        .collect())
}

/// Jarak sunting Levenshtein antara dua kata.
///
/// Dihitung pada karakter Unicode, bukan bita. Menghitungnya per bita akan
/// memberi jarak yang salah untuk kata beraksen, dan bisa memotong karakter
/// di tengah.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }

    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0usize; b.len() + 1];

    for (i, ca) in a.iter().enumerate() {
        current[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            current[j + 1] = (previous[j] + cost)
                .min(previous[j + 1] + 1)
                .min(current[j] + 1);
        }
        core::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()]
}

/// Kemiripan berdasarkan jarak sunting, dinormalkan ke rentang `[0, 1]`.
pub fn levenshtein_similarity(a: &str, b: &str) -> f64 {
    let longest = a.chars().count().max(b.chars().count());
    if longest == 0 {
        return 1.0;
    }
    1.0 - levenshtein(a, b) as f64 / longest as f64
}

// ---------------------------------------------------------------------------
// Analisis sentimen sederhana
// ---------------------------------------------------------------------------

/// Kata bermuatan positif dalam Bahasa Indonesia.
pub const POSITIVE_ID: &[&str] = &[
    "bagus", "baik", "senang", "puas", "cepat", "ramah", "bersih", "murah", "suka", "hebat",
    "mantap", "nyaman", "indah", "lezat", "berhasil", "untung",
];

/// Kata bermuatan negatif dalam Bahasa Indonesia.
pub const NEGATIVE_ID: &[&str] = &[
    "buruk", "jelek", "sedih", "kecewa", "lambat", "kasar", "kotor", "mahal", "benci", "gagal",
    "rusak", "parah", "marah", "susah", "rugi", "payah",
];

/// Kata pengingkar yang membalik muatan kata sesudahnya.
pub const NEGATORS_ID: &[&str] = &["tidak", "bukan", "tak", "kurang", "jangan", "belum"];

/// Hasil analisis sentimen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sentiment {
    /// Skor gabungan, negatif berarti condong negatif.
    pub score: f64,
    /// Label ringkas: `"positif"`, `"negatif"`, atau `"netral"`.
    pub label: String,
    /// Kata bermuatan yang ditemukan, beserta bobotnya setelah pengingkaran.
    pub matches: Vec<(String, f64)>,
}

/// Analisis sentimen berbasis leksikon yang menghormati pengingkaran.
///
/// Pengingkaran ditangani karena tanpanya, "tidak bagus" dinilai positif —
/// kesalahan yang membuat seluruh analisis tidak berguna pada ulasan
/// berbahasa Indonesia, yang sangat sering memakai bentuk itu.
pub fn sentiment_id(tokens: &[String]) -> Sentiment {
    let positive: BTreeSet<&str> = POSITIVE_ID.iter().copied().collect();
    let negative: BTreeSet<&str> = NEGATIVE_ID.iter().copied().collect();
    let negators: BTreeSet<&str> = NEGATORS_ID.iter().copied().collect();

    let mut score = 0.0;
    let mut matches = Vec::new();

    for (i, token) in tokens.iter().enumerate() {
        let base = if positive.contains(token.as_str()) {
            1.0
        } else if negative.contains(token.as_str()) {
            -1.0
        } else {
            continue;
        };

        // Pengingkar dicari sampai dua kata ke belakang: "tidak terlalu bagus"
        // masih terhitung ingkar.
        let negated = tokens[i.saturating_sub(2)..i]
            .iter()
            .any(|t| negators.contains(t.as_str()));
        let weight = if negated { -base } else { base };
        score += weight;
        matches.push((token.clone(), weight));
    }

    let label = if score > 0.0 {
        "positif"
    } else if score < 0.0 {
        "negatif"
    } else {
        "netral"
    };

    Sentiment {
        score,
        label: label.to_string(),
        matches,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    fn t(list: &[&str]) -> Vec<String> {
        list.iter().map(|v| v.to_string()).collect()
    }

    // --------------------------------------------------------- pemenggalan

    #[test]
    fn pemenggalan_dasar() {
        assert_eq!(
            tokenize("Saya suka belajar."),
            t(&["saya", "suka", "belajar"])
        );
        assert_eq!(tokenize("HALO Dunia"), t(&["halo", "dunia"]));
        assert!(tokenize("").is_empty());
        assert!(tokenize("   ...   ").is_empty());
    }

    #[test]
    fn pemenggalan_mempertahankan_kata_ulang() {
        // Bahasa Indonesia memakai tanda hubung untuk pengulangan; memecahnya
        // menjadi dua kata akan menghitung "anak" dua kali.
        assert_eq!(tokenize("anak-anak bermain"), t(&["anak-anak", "bermain"]));
        // Tanda hubung di pinggir dibuang.
        assert_eq!(tokenize("-kata-"), t(&["kata"]));
    }

    #[test]
    fn pemenggalan_menangani_angka_dan_tanda_baca() {
        assert_eq!(
            tokenize("Harga: Rp10.000, murah!"),
            t(&["harga", "rp10", "000", "murah"])
        );
    }

    #[test]
    fn pemenggalan_kalimat() {
        let s = sentences("Halo. Apa kabar? Baik!");
        assert_eq!(s.len(), 3);
        assert_eq!(s[0], "Halo.");
        assert_eq!(s[1], "Apa kabar?");
        assert!(sentences("").is_empty());
        // Kalimat tanpa tanda akhir tetap terhitung.
        assert_eq!(sentences("tanpa titik").len(), 1);
    }

    #[test]
    fn penghapusan_kata_henti() {
        let tokens = tokenize("saya suka belajar di kampus ini");
        let hasil = remove_stopwords(&tokens, STOPWORDS_ID);
        assert!(!hasil.contains(&"di".to_string()));
        assert!(!hasil.contains(&"ini".to_string()));
        assert!(hasil.contains(&"belajar".to_string()));
        assert!(hasil.contains(&"kampus".to_string()));
    }

    #[test]
    fn daftar_kata_henti_tidak_memuat_pengingkar() {
        // Membuang "tidak" akan mengubah "tidak bagus" menjadi "bagus" dan
        // membalik hasil analisis sentimen.
        for negator in NEGATORS_ID {
            assert!(
                !STOPWORDS_ID.contains(negator),
                "{negator} tidak boleh jadi kata henti"
            );
        }
    }

    // ------------------------------------------------------------- stemming

    #[test]
    fn kata_kamus_tidak_dikupas() {
        let hasil = stem_id("makan", DICTIONARY_ID);
        assert_eq!(hasil.stem, "makan");
        assert!(hasil.in_dictionary);
        assert!(hasil.steps.is_empty());
    }

    #[test]
    fn peluluhan_huruf_pertama() {
        // Inti kesulitan Bahasa Indonesia: menyapu berasal dari sapu.
        let hasil = stem_id("menyapu", DICTIONARY_ID);
        assert_eq!(hasil.stem, "sapu", "jejak: {:?}", hasil.steps);
        assert!(hasil.in_dictionary);

        assert_eq!(stem_id("memukul", DICTIONARY_ID).stem, "pukul");
        assert_eq!(stem_id("menulis", DICTIONARY_ID).stem, "tulis");
        assert_eq!(stem_id("penyapu", DICTIONARY_ID).stem, "sapu");
    }

    #[test]
    fn peluluhan_dicoba_bukan_dipaksakan() {
        // Regresi: aturan tunggal yang selalu memulihkan huruf luluh mengubah
        // `membacakan` menjadi `pbaca` dan `pembelajaran` menjadi `pbelajar`.
        // Awalan `mem-` memang meluluhkan `p` pada `memukul`, tetapi `membaca`
        // berasal dari `baca` tanpa peluluhan apa pun. Satu-satunya cara
        // memutuskan adalah mencoba keduanya dan bertanya kepada kamus.
        assert_eq!(stem_id("membacakan", DICTIONARY_ID).stem, "baca");
        assert_eq!(stem_id("membaca", DICTIONARY_ID).stem, "baca");
        assert_eq!(stem_id("pembelajaran", DICTIONARY_ID).stem, "belajar");
        // Yang memang meluluhkan tetap harus benar.
        assert_eq!(stem_id("memukul", DICTIONARY_ID).stem, "pukul");
        assert_eq!(stem_id("menyapu", DICTIONARY_ID).stem, "sapu");
        assert_eq!(stem_id("menulis", DICTIONARY_ID).stem, "tulis");
    }

    #[test]
    fn kandidat_awalan_memuat_kedua_kemungkinan() {
        let kandidat = prefix_candidates("membaca");
        let bentuk: Vec<&str> = kandidat.iter().map(|(_, c)| c.as_str()).collect();
        assert!(bentuk.contains(&"pbaca"), "kandidat peluluhan hilang");
        assert!(bentuk.contains(&"baca"), "kandidat tanpa peluluhan hilang");
    }

    #[test]
    fn kandidat_terlalu_pendek_dibuang() {
        // Kandidat satu atau dua huruf hampir pasti keliru dan hanya menambah
        // peluang salah cocok dengan kamus.
        for (_, candidate) in prefix_candidates("mengap") {
            assert!(candidate.len() >= 3, "kandidat terlalu pendek: {candidate}");
        }
    }

    #[test]
    fn kata_kamus_menyelamatkan_beruang() {
        // Tanpa pemeriksaan kamus lebih dulu, `beruang` akan dikupas menjadi
        // `uang` — kesalahan yang tidak bisa dihindari aturan mana pun.
        let hasil = stem_id("beruang", DICTIONARY_ID);
        assert_eq!(hasil.stem, "beruang");
        assert!(hasil.in_dictionary);
        assert!(hasil.steps.is_empty());
    }

    #[test]
    fn pengupasan_akhiran() {
        assert_eq!(stem_id("bacalah", DICTIONARY_ID).stem, "baca");
        assert_eq!(stem_id("bukumu", DICTIONARY_ID).stem, "buku");
        assert_eq!(stem_id("hitungan", DICTIONARY_ID).stem, "hitung");
        assert_eq!(stem_id("kirimkan", DICTIONARY_ID).stem, "kirim");
    }

    #[test]
    fn pengupasan_awalan_sederhana() {
        assert_eq!(stem_id("dibaca", DICTIONARY_ID).stem, "baca");
        assert_eq!(stem_id("terbaca", DICTIONARY_ID).stem, "baca");
        assert_eq!(stem_id("sebesar", DICTIONARY_ID).stem, "besar");
    }

    #[test]
    fn kata_ulang_dikupas_sekali() {
        let hasil = stem_id("anak-anak", DICTIONARY_ID);
        assert_eq!(hasil.stem, "anak");
        assert!(hasil.steps.iter().any(|s| s.kind == "kata ulang"));
    }

    #[test]
    fn jejak_pengupasan_terekam() {
        let hasil = stem_id("menyapukan", DICTIONARY_ID);
        assert!(!hasil.steps.is_empty());
        // Tiap langkah harus menjelaskan apa yang dibuang.
        for step in &hasil.steps {
            assert!(!step.kind.is_empty());
            assert!(!step.affix.is_empty());
        }
    }

    #[test]
    fn kata_di_luar_kamus_dilaporkan_apa_adanya() {
        let hasil = stem_id("zxqvw", DICTIONARY_ID);
        assert!(!hasil.in_dictionary);
        // Tidak boleh panik maupun menghasilkan teks kosong.
        assert!(!hasil.stem.is_empty());
    }

    #[test]
    fn kata_pendek_tidak_dicacah() {
        // Aturan panjang minimum mencegah kata pendek habis dikupas.
        for kata in ["di", "ke", "me", "an", "i", "a"] {
            let hasil = stem_id(kata, DICTIONARY_ID);
            assert!(
                !hasil.stem.is_empty(),
                "{kata} habis dikupas menjadi kosong"
            );
        }
    }

    #[test]
    fn stemming_seluruh_token() {
        let tokens = tokenize("dibaca menulis makan");
        let hasil = stem_all(&tokens, DICTIONARY_ID);
        assert_eq!(hasil, t(&["baca", "tulis", "makan"]));
    }

    #[test]
    fn stemming_tidak_bergantung_huruf_besar() {
        assert_eq!(
            stem_id("MENYAPU", DICTIONARY_ID).stem,
            stem_id("menyapu", DICTIONARY_ID).stem
        );
    }

    // ------------------------------------------------------------- TF-IDF

    #[test]
    fn frekuensi_kata() {
        let tf = term_frequency(&t(&["a", "b", "a", "a"]));
        close(tf["a"], 0.75);
        close(tf["b"], 0.25);
        assert!(term_frequency(&[]).is_empty());
    }

    #[test]
    fn tfidf_bentuk_benar() {
        let docs = vec![
            tokenize("saya suka kucing"),
            tokenize("saya suka anjing"),
            tokenize("kucing dan anjing"),
        ];
        let hasil = tf_idf(&docs).unwrap();
        assert_eq!(hasil.vectors.len(), 3);
        assert!(hasil.vocabulary.contains(&"kucing".to_string()));
        assert!(hasil
            .vectors
            .iter()
            .all(|v| v.len() == hasil.vocabulary.len()));
        assert!(hasil.idf.iter().all(|v| v.is_finite() && *v > 0.0));
    }

    #[test]
    fn tfidf_memberi_bobot_lebih_pada_kata_jarang() {
        let docs = vec![
            tokenize("umum umum jarang"),
            tokenize("umum umum lain"),
            tokenize("umum umum beda"),
        ];
        let hasil = tf_idf(&docs).unwrap();
        let i_umum = hasil.vocabulary.iter().position(|v| v == "umum").unwrap();
        let i_jarang = hasil.vocabulary.iter().position(|v| v == "jarang").unwrap();
        assert!(
            hasil.idf[i_jarang] > hasil.idf[i_umum],
            "kata jarang harus berbobot lebih besar"
        );
    }

    #[test]
    fn tfidf_kata_yang_ada_di_semua_dokumen_tidak_hilang() {
        // Bentuk IDF mentah ln(N/df) memberi nol untuk kata yang muncul di
        // semua dokumen, sehingga kata itu lenyap dari perhitungan.
        let docs = vec![tokenize("sama sama"), tokenize("sama lain")];
        let hasil = tf_idf(&docs).unwrap();
        let i = hasil.vocabulary.iter().position(|v| v == "sama").unwrap();
        assert!(hasil.idf[i] > 0.0, "IDF kata universal tidak boleh nol");
        assert!(hasil.vectors[0][i] > 0.0);
    }

    #[test]
    fn tfidf_menolak_korpus_kosong() {
        assert_eq!(tf_idf(&[]), Err(NlpError::EmptyCorpus));
    }

    // -------------------------------------------------------- kemiripan

    #[test]
    fn kosinus_nilai_yang_dikenal() {
        close(cosine_similarity(&[1.0, 0.0], &[1.0, 0.0]).unwrap(), 1.0);
        close(cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]).unwrap(), 0.0);
        close(cosine_similarity(&[1.0, 0.0], &[-1.0, 0.0]).unwrap(), -1.0);
        // Skala tidak mempengaruhi kemiripan kosinus.
        close(cosine_similarity(&[1.0, 2.0], &[2.0, 4.0]).unwrap(), 1.0);
    }

    #[test]
    fn kosinus_vektor_nol_tidak_membagi_nol() {
        close(cosine_similarity(&[0.0, 0.0], &[1.0, 1.0]).unwrap(), 0.0);
        close(cosine_similarity(&[0.0], &[0.0]).unwrap(), 0.0);
    }

    #[test]
    fn kosinus_menolak_panjang_berbeda() {
        assert_eq!(
            cosine_similarity(&[1.0], &[1.0, 2.0]),
            Err(NlpError::LengthMismatch { a: 1, b: 2 })
        );
    }

    #[test]
    fn kosinus_mengenali_dokumen_serupa() {
        let docs = vec![
            tokenize("kucing suka ikan"),
            tokenize("kucing gemar ikan"),
            tokenize("mobil melaju cepat di jalan raya"),
        ];
        let hasil = tf_idf(&docs).unwrap();
        let mirip = cosine_similarity(&hasil.vectors[0], &hasil.vectors[1]).unwrap();
        let beda = cosine_similarity(&hasil.vectors[0], &hasil.vectors[2]).unwrap();
        assert!(mirip > beda, "{mirip} seharusnya lebih besar dari {beda}");
    }

    #[test]
    fn jaccard_nilai_yang_dikenal() {
        close(jaccard_similarity(&t(&["a", "b"]), &t(&["a", "b"])), 1.0);
        close(jaccard_similarity(&t(&["a"]), &t(&["b"])), 0.0);
        close(
            jaccard_similarity(&t(&["a", "b"]), &t(&["b", "c"])),
            1.0 / 3.0,
        );
        close(jaccard_similarity(&[], &[]), 0.0);
    }

    // ---------------------------------------------------------------- n-gram

    #[test]
    fn ngram_kata() {
        let tokens = t(&["a", "b", "c", "d"]);
        assert_eq!(ngrams(&tokens, 1).unwrap().len(), 4);
        assert_eq!(ngrams(&tokens, 2).unwrap(), t(&["a b", "b c", "c d"]));
        assert_eq!(ngrams(&tokens, 4).unwrap(), t(&["a b c d"]));
        // Lebih panjang daripada tokennya menghasilkan daftar kosong.
        assert!(ngrams(&tokens, 5).unwrap().is_empty());
        assert_eq!(ngrams(&tokens, 0), Err(NlpError::BadNgramSize(0)));
    }

    #[test]
    fn ngram_karakter() {
        assert_eq!(char_ngrams("kata", 2).unwrap(), t(&["ka", "at", "ta"]));
        assert!(char_ngrams("ab", 5).unwrap().is_empty());
        assert_eq!(char_ngrams("ab", 0), Err(NlpError::BadNgramSize(0)));
    }

    // ---------------------------------------------------------- Levenshtein

    #[test]
    fn levenshtein_nilai_yang_dikenal() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("sama", "sama"), 0);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", ""), 0);
    }

    #[test]
    fn levenshtein_setangkup() {
        for (a, b) in [("kucing", "kucin"), ("makan", "minum"), ("a", "bcdef")] {
            assert_eq!(levenshtein(a, b), levenshtein(b, a));
        }
    }

    #[test]
    fn levenshtein_menghitung_karakter_bukan_bita() {
        // "é" memakai dua bita dalam UTF-8; menghitung per bita akan memberi
        // jarak dua untuk perubahan satu huruf.
        assert_eq!(levenshtein("café", "cafe"), 1);
        assert_eq!(levenshtein("naïve", "naive"), 1);
    }

    #[test]
    fn kemiripan_levenshtein_ternormalkan() {
        close(levenshtein_similarity("sama", "sama"), 1.0);
        close(levenshtein_similarity("", ""), 1.0);
        close(levenshtein_similarity("abc", "xyz"), 0.0);
        let v = levenshtein_similarity("kucing", "kucin");
        assert!(v > 0.8 && v < 1.0);
    }

    // -------------------------------------------------------------- sentimen

    #[test]
    fn sentimen_positif_dan_negatif() {
        assert_eq!(
            sentiment_id(&tokenize("pelayanannya bagus")).label,
            "positif"
        );
        assert_eq!(
            sentiment_id(&tokenize("pelayanannya buruk")).label,
            "negatif"
        );
        assert_eq!(sentiment_id(&tokenize("biasa saja")).label, "netral");
    }

    #[test]
    fn sentimen_menghormati_pengingkaran() {
        // Tanpa penanganan pengingkaran, kalimat ini akan dinilai positif —
        // dan bentuk seperti ini sangat lazim pada ulasan berbahasa Indonesia.
        let hasil = sentiment_id(&tokenize("makanannya tidak bagus"));
        assert_eq!(hasil.label, "negatif", "skor {}", hasil.score);

        let hasil2 = sentiment_id(&tokenize("tidak buruk"));
        assert_eq!(hasil2.label, "positif");

        // Pengingkar berjarak dua kata masih terhitung.
        let hasil3 = sentiment_id(&tokenize("tidak terlalu bagus"));
        assert_eq!(hasil3.label, "negatif");
    }

    #[test]
    fn sentimen_menjumlahkan_beberapa_kata() {
        let hasil = sentiment_id(&tokenize("tempatnya bersih dan ramah tapi mahal"));
        assert!(hasil.matches.len() >= 3);
        close(hasil.score, 1.0);
        assert_eq!(hasil.label, "positif");
    }

    #[test]
    fn sentimen_melaporkan_kata_yang_terdeteksi() {
        let hasil = sentiment_id(&tokenize("bagus sekali"));
        assert_eq!(hasil.matches.len(), 1);
        assert_eq!(hasil.matches[0].0, "bagus");
        close(hasil.matches[0].1, 1.0);
    }

    #[test]
    fn leksikon_tidak_tumpang_tindih() {
        // Kata yang muncul di kedua daftar akan membuat skornya bergantung
        // urutan pemeriksaan, bukan pada maknanya.
        for kata in POSITIVE_ID {
            assert!(
                !NEGATIVE_ID.contains(kata),
                "{kata} ada di kedua daftar leksikon"
            );
        }
    }

    #[test]
    fn hasil_bisa_di_serialisasi() {
        let hasil = stem_id("menyapu", DICTIONARY_ID);
        let json = serde_json::to_string(&hasil).unwrap();
        assert_eq!(serde_json::from_str::<StemResult>(&json).unwrap(), hasil);

        let s = sentiment_id(&tokenize("bagus"));
        let sj = serde_json::to_string(&s).unwrap();
        assert_eq!(serde_json::from_str::<Sentiment>(&sj).unwrap(), s);
    }

    #[test]
    fn pesan_error_terbaca() {
        for e in [
            NlpError::EmptyCorpus,
            NlpError::LengthMismatch { a: 1, b: 2 },
            NlpError::BadNgramSize(0),
        ] {
            assert!(!e.to_string().is_empty());
        }
    }
}
