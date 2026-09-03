//! Sesi 7 — Representasi Pengetahuan.
//!
//! Logika proposisi lengkap: penguraian rumus, tabel kebenaran, pemeriksaan
//! tautologi dan kepuasan, bentuk normal konjungtif, dan pembuktian dengan
//! resolusi. Ditambah jaringan semantik dan bingkai, dua bentuk representasi
//! yang diajarkan di modul.
//!
//! Resolusi dipilih karena ia satu-satunya di sini yang benar-benar
//! *membuktikan*, bukan sekadar mengecek. Tabel kebenaran menjawab "apakah
//! benar" dengan mencoba semua kemungkinan — pada dua puluh proposisi itu
//! berarti sejuta baris. Resolusi menjawab pertanyaan yang sama tanpa
//! menyentuh sebagian besarnya.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Sebab sebuah rumus gagal diuraikan.
///
/// Sebuah nilai, bukan untai penjelasan. Untai itu dulu berisi kalimat Bahasa
/// Indonesia — "kurung tutup hilang", "rumus kosong" — dan kalimat yang
/// disimpan di dalam galat hanya bisa punya satu bahasa. Sebabnya dipilah di
/// sini; kalimatnya dirakit sisi antarmuka.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParseCause {
    /// Rumusnya kosong.
    EmptyFormula,
    /// Ada karakter yang bukan bagian tata bahasanya.
    UnknownCharacter(char),
    /// Kurung buka tidak punya penutup.
    MissingCloseParen,
    /// Operator muncul tanpa operand.
    OperatorWithoutOperand,
    /// Rumus berakhir lebih cepat daripada yang dituntut tata bahasanya.
    UnexpectedEnd,
    /// Rumusnya sudah lengkap tetapi masih ada sisa masukan.
    TrailingInput,
}

/// Kesalahan pada representasi pengetahuan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeError {
    /// Rumus tidak bisa diuraikan.
    ParseError {
        /// Sebab kegagalannya.
        cause: ParseCause,
        /// Posisi karakter tempat kesalahan terdeteksi.
        position: usize,
    },
    /// Rumus memuat lebih banyak proposisi daripada yang bisa ditabelkan.
    TooManyVariables(usize),
    /// Basis pengetahuan kosong.
    EmptyKnowledgeBase,
    /// Nama simpul tidak ditemukan pada jaringan semantik.
    UnknownNode(String),
    /// Batas langkah pembuktian terlampaui.
    ProofLimitExceeded(usize),
}

impl crate::galat::Dijelaskan for KnowledgeError {
    fn kode(&self) -> &'static str {
        match self {
            KnowledgeError::ParseError { cause, .. } => match cause {
                ParseCause::EmptyFormula => "logika.urai_rumus_kosong",
                ParseCause::UnknownCharacter(_) => "logika.urai_karakter_tak_dikenal",
                ParseCause::MissingCloseParen => "logika.urai_kurung_tutup_hilang",
                ParseCause::OperatorWithoutOperand => "logika.urai_operator_tanpa_operand",
                ParseCause::UnexpectedEnd => "logika.urai_rumus_terputus",
                ParseCause::TrailingInput => "logika.urai_sisa_masukan",
            },
            KnowledgeError::TooManyVariables(_) => "logika.terlalu_banyak_variabel",
            KnowledgeError::EmptyKnowledgeBase => "logika.basis_kosong",
            KnowledgeError::UnknownNode(_) => "logika.simpul_tak_dikenal",
            KnowledgeError::ProofLimitExceeded(_) => "logika.batas_pembuktian",
        }
    }

    fn argumen(&self) -> Vec<String> {
        match self {
            // Posisi selalu argumen pertama, supaya seluruh galat penguraian
            // punya bentuk yang sama; karakternya menyusul bila ada.
            KnowledgeError::ParseError { cause, position } => match cause {
                ParseCause::UnknownCharacter(c) => vec![position.to_string(), c.to_string()],
                // Rumus kosong selalu gagal di posisi nol, jadi menyebut
                // posisinya hanya menambah angka yang tidak menjelaskan apa
                // pun. Jumlah argumen dan jumlah penanda dituntut sepadan,
                // sehingga argumen yang tidak dipakai bukan sekadar mubazir —
                // ia membuat ujinya gagal.
                ParseCause::EmptyFormula => Vec::new(),
                _ => vec![position.to_string()],
            },
            KnowledgeError::TooManyVariables(n) => {
                vec![n.to_string(), MAX_VARIABLES.to_string()]
            }
            KnowledgeError::EmptyKnowledgeBase => Vec::new(),
            KnowledgeError::UnknownNode(n) => vec![n.clone()],
            KnowledgeError::ProofLimitExceeded(n) => vec![n.to_string()],
        }
    }
}

impl core::fmt::Display for KnowledgeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            KnowledgeError::ParseError { cause, position } => {
                let sebab = match cause {
                    ParseCause::EmptyFormula => "rumus kosong".to_string(),
                    ParseCause::UnknownCharacter(c) => format!("karakter tak dikenal: {c:?}"),
                    ParseCause::MissingCloseParen => "kurung tutup hilang".to_string(),
                    ParseCause::OperatorWithoutOperand => {
                        "operator muncul tanpa operand".to_string()
                    }
                    ParseCause::UnexpectedEnd => {
                        "rumus berakhir lebih cepat dari yang diharapkan".to_string()
                    }
                    ParseCause::TrailingInput => "ada sisa masukan yang tidak terpakai".to_string(),
                };
                write!(f, "gagal menguraikan pada posisi {position}: {sebab}")
            }
            KnowledgeError::TooManyVariables(n) => write!(
                f,
                "{n} proposisi menghasilkan tabel yang terlalu besar (batas {MAX_VARIABLES})"
            ),
            KnowledgeError::EmptyKnowledgeBase => write!(f, "basis pengetahuan kosong"),
            KnowledgeError::UnknownNode(n) => write!(f, "simpul tidak dikenal: {n}"),
            KnowledgeError::ProofLimitExceeded(n) => {
                write!(f, "melampaui {n} langkah pembuktian")
            }
        }
    }
}

/// Batas jumlah proposisi pada tabel kebenaran.
///
/// Tabelnya tumbuh sebagai `2^n`; enam belas proposisi sudah berarti 65.536
/// baris, dan di atas itu peramban akan tersendat sebelum jawabannya berguna.
pub const MAX_VARIABLES: usize = 16;

/// Batas jumlah klausa yang boleh dihasilkan resolusi.
pub const MAX_CLAUSES: usize = 5_000;

// ---------------------------------------------------------------------------
// Rumus proposisi
// ---------------------------------------------------------------------------

/// Sebuah rumus logika proposisi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Formula {
    /// Proposisi dasar, mis. `P`.
    Atom {
        /// Nama proposisi.
        name: String,
    },
    /// Ingkaran.
    Not {
        /// Rumus yang diingkari.
        operand: Box<Formula>,
    },
    /// Konjungsi.
    And {
        /// Ruas kiri.
        left: Box<Formula>,
        /// Ruas kanan.
        right: Box<Formula>,
    },
    /// Disjungsi.
    Or {
        /// Ruas kiri.
        left: Box<Formula>,
        /// Ruas kanan.
        right: Box<Formula>,
    },
    /// Implikasi.
    Implies {
        /// Anteseden.
        left: Box<Formula>,
        /// Konsekuen.
        right: Box<Formula>,
    },
    /// Bi-implikasi.
    Iff {
        /// Ruas kiri.
        left: Box<Formula>,
        /// Ruas kanan.
        right: Box<Formula>,
    },
}

impl Formula {
    /// Proposisi dasar.
    pub fn atom(name: impl Into<String>) -> Self {
        Formula::Atom { name: name.into() }
    }

    /// Ingkaran sebuah rumus.
    ///
    /// Sengaja bernama sama dengan `std::ops::Not::not` walau clippy
    /// memperingatkannya: ini fungsi bersekutu yang menerima rumus dan
    /// mengembalikan rumus, bukan operator negasi bit. Nama lain seperti
    /// `negate` akan membuat pembacaan `Formula::not(a)` kehilangan
    /// kemiripannya dengan notasi logika yang sedang diperagakan.
    #[allow(clippy::should_implement_trait)]
    pub fn not(f: Formula) -> Self {
        Formula::Not {
            operand: Box::new(f),
        }
    }

    /// Konjungsi dua rumus.
    pub fn and(a: Formula, b: Formula) -> Self {
        Formula::And {
            left: Box::new(a),
            right: Box::new(b),
        }
    }

    /// Disjungsi dua rumus.
    pub fn or(a: Formula, b: Formula) -> Self {
        Formula::Or {
            left: Box::new(a),
            right: Box::new(b),
        }
    }

    /// Implikasi dua rumus.
    pub fn implies(a: Formula, b: Formula) -> Self {
        Formula::Implies {
            left: Box::new(a),
            right: Box::new(b),
        }
    }

    /// Bi-implikasi dua rumus.
    pub fn iff(a: Formula, b: Formula) -> Self {
        Formula::Iff {
            left: Box::new(a),
            right: Box::new(b),
        }
    }

    /// Seluruh proposisi yang muncul, terurut.
    pub fn variables(&self) -> Vec<String> {
        let mut set = BTreeSet::new();
        self.collect_variables(&mut set);
        set.into_iter().collect()
    }

    fn collect_variables(&self, out: &mut BTreeSet<String>) {
        match self {
            Formula::Atom { name } => {
                out.insert(name.clone());
            }
            Formula::Not { operand } => operand.collect_variables(out),
            Formula::And { left, right }
            | Formula::Or { left, right }
            | Formula::Implies { left, right }
            | Formula::Iff { left, right } => {
                left.collect_variables(out);
                right.collect_variables(out);
            }
        }
    }

    /// Nilai kebenaran rumus pada sebuah penugasan.
    ///
    /// Proposisi yang tidak ada di penugasan dianggap salah.
    pub fn evaluate(&self, assignment: &BTreeMap<String, bool>) -> bool {
        match self {
            Formula::Atom { name } => assignment.get(name).copied().unwrap_or(false),
            Formula::Not { operand } => !operand.evaluate(assignment),
            Formula::And { left, right } => left.evaluate(assignment) && right.evaluate(assignment),
            Formula::Or { left, right } => left.evaluate(assignment) || right.evaluate(assignment),
            Formula::Implies { left, right } => {
                !left.evaluate(assignment) || right.evaluate(assignment)
            }
            Formula::Iff { left, right } => left.evaluate(assignment) == right.evaluate(assignment),
        }
    }

    /// Bentuk teks rumus, dengan tanda kurung seperlunya.
    pub fn to_text(&self) -> String {
        match self {
            Formula::Atom { name } => name.clone(),
            Formula::Not { operand } => format!("¬{}", operand.wrapped()),
            Formula::And { left, right } => {
                format!("{} ∧ {}", left.wrapped(), right.wrapped())
            }
            Formula::Or { left, right } => format!("{} ∨ {}", left.wrapped(), right.wrapped()),
            Formula::Implies { left, right } => {
                format!("{} → {}", left.wrapped(), right.wrapped())
            }
            Formula::Iff { left, right } => format!("{} ↔ {}", left.wrapped(), right.wrapped()),
        }
    }

    /// Bentuk teks dengan kurung bila rumusnya majemuk.
    fn wrapped(&self) -> String {
        match self {
            Formula::Atom { .. } | Formula::Not { .. } => self.to_text(),
            _ => format!("({})", self.to_text()),
        }
    }
}

// ---------------------------------------------------------------------------
// Penguraian
// ---------------------------------------------------------------------------

/// Menguraikan rumus dari teks.
///
/// Menerima beberapa gaya penulisan sekaligus karena buku, papan tulis, dan
/// papan ketik memakai lambang yang berbeda untuk operator yang sama:
///
/// | Operator | Diterima |
/// |----------|----------|
/// | ingkaran | `¬` `~` `!` `not` |
/// | konjungsi | `∧` `&` `&&` `and` |
/// | disjungsi | `∨` `\|` `\|\|` `or` |
/// | implikasi | `→` `->` `=>` `implies` |
/// | bi-implikasi | `↔` `<->` `<=>` `iff` |
pub fn parse(input: &str) -> Result<Formula, KnowledgeError> {
    let tokens = lex(input)?;
    let mut parser = Parser {
        tokens,
        position: 0,
    };
    let formula = parser.parse_iff()?;
    if parser.position < parser.tokens.len() {
        return Err(KnowledgeError::ParseError {
            cause: ParseCause::TrailingInput,
            position: parser.position,
        });
    }
    Ok(formula)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Atom(String),
    Not,
    And,
    Or,
    Implies,
    Iff,
    Open,
    Close,
}

fn lex(input: &str) -> Result<Vec<Token>, KnowledgeError> {
    let chars: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        match c {
            '(' => {
                out.push(Token::Open);
                i += 1;
            }
            ')' => {
                out.push(Token::Close);
                i += 1;
            }
            '¬' | '~' | '!' => {
                out.push(Token::Not);
                i += 1;
            }
            '∧' => {
                out.push(Token::And);
                i += 1;
            }
            '∨' => {
                out.push(Token::Or);
                i += 1;
            }
            '→' => {
                out.push(Token::Implies);
                i += 1;
            }
            '↔' => {
                out.push(Token::Iff);
                i += 1;
            }
            '&' => {
                i += if chars.get(i + 1) == Some(&'&') { 2 } else { 1 };
                out.push(Token::And);
            }
            '|' => {
                i += if chars.get(i + 1) == Some(&'|') { 2 } else { 1 };
                out.push(Token::Or);
            }
            '-' if chars.get(i + 1) == Some(&'>') => {
                out.push(Token::Implies);
                i += 2;
            }
            '=' if chars.get(i + 1) == Some(&'>') => {
                out.push(Token::Implies);
                i += 2;
            }
            '<' if chars.get(i + 1) == Some(&'-') && chars.get(i + 2) == Some(&'>') => {
                out.push(Token::Iff);
                i += 3;
            }
            '<' if chars.get(i + 1) == Some(&'=') && chars.get(i + 2) == Some(&'>') => {
                out.push(Token::Iff);
                i += 3;
            }
            c if c.is_alphanumeric() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                // Kata kunci diperiksa tanpa membedakan huruf besar-kecil.
                match word.to_ascii_lowercase().as_str() {
                    "not" => out.push(Token::Not),
                    "and" => out.push(Token::And),
                    "or" => out.push(Token::Or),
                    "implies" => out.push(Token::Implies),
                    "iff" => out.push(Token::Iff),
                    _ => out.push(Token::Atom(word)),
                }
            }
            other => {
                return Err(KnowledgeError::ParseError {
                    cause: ParseCause::UnknownCharacter(other),
                    position: i,
                })
            }
        }
    }
    if out.is_empty() {
        return Err(KnowledgeError::ParseError {
            cause: ParseCause::EmptyFormula,
            position: 0,
        });
    }
    Ok(out)
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn error(&self, cause: ParseCause) -> KnowledgeError {
        KnowledgeError::ParseError {
            cause,
            position: self.position,
        }
    }

    // Urutan keutamaan, dari yang paling longgar: ↔, →, ∨, ∧, ¬.
    // Implikasi berasosiasi ke kanan; sisanya ke kiri.
    fn parse_iff(&mut self) -> Result<Formula, KnowledgeError> {
        let mut left = self.parse_implies()?;
        while self.peek() == Some(&Token::Iff) {
            self.position += 1;
            let right = self.parse_implies()?;
            left = Formula::iff(left, right);
        }
        Ok(left)
    }

    fn parse_implies(&mut self) -> Result<Formula, KnowledgeError> {
        let left = self.parse_or()?;
        if self.peek() == Some(&Token::Implies) {
            self.position += 1;
            // Rekursi ke kanan: `a -> b -> c` berarti `a -> (b -> c)`.
            let right = self.parse_implies()?;
            return Ok(Formula::implies(left, right));
        }
        Ok(left)
    }

    fn parse_or(&mut self) -> Result<Formula, KnowledgeError> {
        let mut left = self.parse_and()?;
        while self.peek() == Some(&Token::Or) {
            self.position += 1;
            let right = self.parse_and()?;
            left = Formula::or(left, right);
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Formula, KnowledgeError> {
        let mut left = self.parse_unary()?;
        while self.peek() == Some(&Token::And) {
            self.position += 1;
            let right = self.parse_unary()?;
            left = Formula::and(left, right);
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Formula, KnowledgeError> {
        match self.peek().cloned() {
            Some(Token::Not) => {
                self.position += 1;
                Ok(Formula::not(self.parse_unary()?))
            }
            Some(Token::Open) => {
                self.position += 1;
                let inner = self.parse_iff()?;
                if self.peek() != Some(&Token::Close) {
                    return Err(self.error(ParseCause::MissingCloseParen));
                }
                self.position += 1;
                Ok(inner)
            }
            Some(Token::Atom(name)) => {
                self.position += 1;
                Ok(Formula::atom(name))
            }
            Some(_) => Err(self.error(ParseCause::OperatorWithoutOperand)),
            None => Err(self.error(ParseCause::UnexpectedEnd)),
        }
    }
}

// ---------------------------------------------------------------------------
// Tabel kebenaran
// ---------------------------------------------------------------------------

/// Satu baris tabel kebenaran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TruthRow {
    /// Nilai tiap proposisi, urutannya sesuai daftar proposisi.
    pub values: Vec<bool>,
    /// Nilai rumus pada baris ini.
    pub result: bool,
}

/// Tabel kebenaran lengkap beserta kesimpulannya.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TruthTable {
    /// Proposisi yang muncul, terurut.
    pub variables: Vec<String>,
    /// Seluruh baris, dari semua nilai salah sampai semua benar.
    pub rows: Vec<TruthRow>,
    /// Benar bila rumusnya benar pada seluruh baris.
    pub tautology: bool,
    /// Benar bila ada minimal satu baris yang membuatnya benar.
    pub satisfiable: bool,
    /// Benar bila tidak ada baris yang membuatnya benar.
    pub contradiction: bool,
}

/// Menyusun tabel kebenaran sebuah rumus.
pub fn truth_table(formula: &Formula) -> Result<TruthTable, KnowledgeError> {
    let variables = formula.variables();
    if variables.len() > MAX_VARIABLES {
        return Err(KnowledgeError::TooManyVariables(variables.len()));
    }

    let n = variables.len();
    let total = 1usize << n;
    let mut rows = Vec::with_capacity(total);

    for mask in 0..total {
        let mut assignment = BTreeMap::new();
        let mut values = Vec::with_capacity(n);
        for (i, name) in variables.iter().enumerate() {
            // Bit tertinggi mewakili proposisi pertama, sehingga urutan
            // barisnya sama dengan tabel yang ditulis tangan.
            let value = mask & (1 << (n - 1 - i)) != 0;
            assignment.insert(name.clone(), value);
            values.push(value);
        }
        rows.push(TruthRow {
            result: formula.evaluate(&assignment),
            values,
        });
    }

    let tautology = rows.iter().all(|r| r.result);
    let satisfiable = rows.iter().any(|r| r.result);
    Ok(TruthTable {
        variables,
        rows,
        tautology,
        satisfiable,
        contradiction: !satisfiable,
    })
}

/// Apakah dua rumus setara secara logika.
pub fn equivalent(a: &Formula, b: &Formula) -> Result<bool, KnowledgeError> {
    truth_table(&Formula::iff(a.clone(), b.clone())).map(|t| t.tautology)
}

// ---------------------------------------------------------------------------
// Bentuk normal konjungtif dan resolusi
// ---------------------------------------------------------------------------

/// Sebuah literal: proposisi, mungkin diingkari.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Literal {
    /// Nama proposisi.
    pub name: String,
    /// `true` berarti proposisi itu diingkari.
    pub negated: bool,
}

impl Literal {
    /// Literal positif.
    pub fn positive(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            negated: false,
        }
    }

    /// Literal negatif.
    pub fn negative(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            negated: true,
        }
    }

    /// Kebalikan literal ini.
    pub fn negate(&self) -> Self {
        Self {
            name: self.name.clone(),
            negated: !self.negated,
        }
    }

    /// Bentuk teks literal.
    pub fn to_text(&self) -> String {
        if self.negated {
            format!("¬{}", self.name)
        } else {
            self.name.clone()
        }
    }
}

/// Sebuah klausa: disjungsi literal.
pub type Clause = BTreeSet<Literal>;

/// Bentuk teks sebuah klausa.
pub fn clause_text(clause: &Clause) -> String {
    if clause.is_empty() {
        return "□".to_string();
    }
    clause
        .iter()
        .map(|l| l.to_text())
        .collect::<Vec<_>>()
        .join(" ∨ ")
}

/// Membuang implikasi dan bi-implikasi.
fn eliminate_implications(f: &Formula) -> Formula {
    match f {
        Formula::Atom { .. } => f.clone(),
        Formula::Not { operand } => Formula::not(eliminate_implications(operand)),
        Formula::And { left, right } => {
            Formula::and(eliminate_implications(left), eliminate_implications(right))
        }
        Formula::Or { left, right } => {
            Formula::or(eliminate_implications(left), eliminate_implications(right))
        }
        Formula::Implies { left, right } => Formula::or(
            Formula::not(eliminate_implications(left)),
            eliminate_implications(right),
        ),
        Formula::Iff { left, right } => {
            let a = eliminate_implications(left);
            let b = eliminate_implications(right);
            Formula::and(
                Formula::or(Formula::not(a.clone()), b.clone()),
                Formula::or(Formula::not(b), a),
            )
        }
    }
}

/// Mendorong ingkaran sampai ke proposisi dasar, memakai hukum De Morgan.
fn push_negations(f: &Formula) -> Formula {
    match f {
        Formula::Not { operand } => match operand.as_ref() {
            Formula::Atom { .. } => f.clone(),
            Formula::Not { operand: inner } => push_negations(inner),
            Formula::And { left, right } => Formula::or(
                push_negations(&Formula::not(left.as_ref().clone())),
                push_negations(&Formula::not(right.as_ref().clone())),
            ),
            Formula::Or { left, right } => Formula::and(
                push_negations(&Formula::not(left.as_ref().clone())),
                push_negations(&Formula::not(right.as_ref().clone())),
            ),
            // Implikasi sudah dibuang pada tahap sebelumnya.
            other => push_negations(&Formula::not(other.clone())),
        },
        Formula::And { left, right } => Formula::and(push_negations(left), push_negations(right)),
        Formula::Or { left, right } => Formula::or(push_negations(left), push_negations(right)),
        Formula::Atom { .. } => f.clone(),
        Formula::Implies { .. } | Formula::Iff { .. } => f.clone(),
    }
}

/// Mengubah rumus menjadi himpunan klausa (bentuk normal konjungtif).
pub fn to_cnf(formula: &Formula) -> Vec<Clause> {
    let without_implications = eliminate_implications(formula);
    let pushed = push_negations(&without_implications);
    distribute(&pushed)
}

/// Menyebarkan disjungsi ke dalam konjungsi sampai berbentuk klausa.
fn distribute(f: &Formula) -> Vec<Clause> {
    match f {
        Formula::Atom { name } => vec![BTreeSet::from([Literal::positive(name)])],
        Formula::Not { operand } => match operand.as_ref() {
            Formula::Atom { name } => vec![BTreeSet::from([Literal::negative(name)])],
            // Ingkaran majemuk sudah didorong ke bawah sebelum tahap ini.
            other => distribute(&push_negations(&Formula::not(other.clone()))),
        },
        Formula::And { left, right } => {
            let mut out = distribute(left);
            out.extend(distribute(right));
            out
        }
        Formula::Or { left, right } => {
            let a = distribute(left);
            let b = distribute(right);
            let mut out = Vec::with_capacity(a.len() * b.len());
            for x in &a {
                for y in &b {
                    let mut merged = x.clone();
                    merged.extend(y.iter().cloned());
                    out.push(merged);
                }
            }
            out
        }
        Formula::Implies { .. } | Formula::Iff { .. } => {
            distribute(&push_negations(&eliminate_implications(f)))
        }
    }
}

/// Apakah sebuah klausa selalu benar karena memuat literal dan ingkarannya.
pub fn is_tautological_clause(clause: &Clause) -> bool {
    clause.iter().any(|l| clause.contains(&l.negate()))
}

/// Satu langkah resolusi, dipakai untuk menampilkan jejak pembuktian.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionStep {
    /// Nomor urut langkah.
    pub order: usize,
    /// Klausa pertama.
    pub left: String,
    /// Klausa kedua.
    pub right: String,
    /// Literal yang dihapuskan.
    pub pivot: String,
    /// Klausa hasil.
    pub result: String,
}

/// Hasil pembuktian dengan resolusi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionProof {
    /// Apakah kesimpulannya terbukti.
    pub proved: bool,
    /// Klausa awal, termasuk ingkaran kesimpulan.
    pub initial_clauses: Vec<String>,
    /// Langkah-langkah resolusi sampai klausa kosong ditemukan.
    pub steps: Vec<ResolutionStep>,
    /// Jumlah klausa yang sempat dihasilkan.
    pub generated: usize,
}

/// Membuktikan sebuah kesimpulan dari basis pengetahuan dengan resolusi.
///
/// Bekerja dengan menyangkal kesimpulan lalu mencari kontradiksi. Bila klausa
/// kosong muncul, ingkaran kesimpulan mustahil benar, sehingga kesimpulannya
/// pasti benar. Cara ini menjawab pertanyaan yang sama dengan tabel kebenaran
/// tanpa harus mencoba seluruh `2^n` kemungkinan.
pub fn resolve(
    knowledge: &[Formula],
    conclusion: &Formula,
) -> Result<ResolutionProof, KnowledgeError> {
    if knowledge.is_empty() {
        return Err(KnowledgeError::EmptyKnowledgeBase);
    }

    let mut clauses: Vec<Clause> = Vec::new();
    for f in knowledge {
        clauses.extend(to_cnf(f));
    }
    clauses.extend(to_cnf(&Formula::not(conclusion.clone())));
    clauses.retain(|c| !is_tautological_clause(c));

    let initial_clauses: Vec<String> = clauses.iter().map(clause_text).collect();
    let mut seen: BTreeSet<Clause> = clauses.iter().cloned().collect();
    let mut steps = Vec::new();
    let mut generated = clauses.len();

    // Klausa kosong di antara klausa awal berarti basisnya sendiri kontradiktif.
    if clauses.iter().any(|c| c.is_empty()) {
        return Ok(ResolutionProof {
            proved: true,
            initial_clauses,
            steps,
            generated,
        });
    }

    loop {
        let mut new_clauses: Vec<(Clause, ResolutionStep)> = Vec::new();

        for i in 0..clauses.len() {
            for j in (i + 1)..clauses.len() {
                for literal in clauses[i].iter() {
                    let opposite = literal.negate();
                    if !clauses[j].contains(&opposite) {
                        continue;
                    }
                    let mut merged: Clause = clauses[i]
                        .iter()
                        .filter(|l| **l != *literal)
                        .cloned()
                        .collect();
                    merged.extend(clauses[j].iter().filter(|l| **l != opposite).cloned());

                    // Klausa yang selalu benar tidak membawa informasi baru.
                    if is_tautological_clause(&merged) {
                        continue;
                    }
                    if seen.contains(&merged) {
                        continue;
                    }

                    let step = ResolutionStep {
                        order: steps.len() + new_clauses.len() + 1,
                        left: clause_text(&clauses[i]),
                        right: clause_text(&clauses[j]),
                        pivot: literal.name.clone(),
                        result: clause_text(&merged),
                    };

                    if merged.is_empty() {
                        steps.push(step);
                        return Ok(ResolutionProof {
                            proved: true,
                            initial_clauses,
                            steps,
                            generated: generated + 1,
                        });
                    }
                    new_clauses.push((merged, step));
                }
            }
        }

        if new_clauses.is_empty() {
            // Tidak ada lagi klausa baru: kontradiksi tidak bisa diturunkan,
            // jadi kesimpulannya tidak mengikuti dari basis pengetahuan.
            return Ok(ResolutionProof {
                proved: false,
                initial_clauses,
                steps,
                generated,
            });
        }

        for (clause, step) in new_clauses {
            if seen.insert(clause.clone()) {
                clauses.push(clause);
                steps.push(step);
                generated += 1;
                if generated > MAX_CLAUSES {
                    return Err(KnowledgeError::ProofLimitExceeded(MAX_CLAUSES));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Jaringan semantik
// ---------------------------------------------------------------------------

/// Sebuah relasi berarah antara dua simpul.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relation {
    /// Simpul asal.
    pub from: String,
    /// Nama relasi, mis. `"adalah"` atau `"punya"`.
    pub label: String,
    /// Simpul tujuan.
    pub to: String,
}

/// Jaringan semantik: kumpulan simpul dan relasi berarah.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticNetwork {
    /// Relasi-relasi yang menyusun jaringan.
    pub relations: Vec<Relation>,
    /// Nama relasi yang mewarisi sifat, biasanya `"adalah"`.
    pub inheritance_label: String,
}

impl SemanticNetwork {
    /// Jaringan baru dengan relasi pewarisan bernama `"adalah"`.
    pub fn new() -> Self {
        Self {
            relations: Vec::new(),
            inheritance_label: "adalah".to_string(),
        }
    }

    /// Menambahkan sebuah relasi.
    pub fn add(&mut self, from: &str, label: &str, to: &str) {
        self.relations.push(Relation {
            from: from.to_string(),
            label: label.to_string(),
            to: to.to_string(),
        });
    }

    /// Seluruh simpul yang muncul, terurut.
    pub fn nodes(&self) -> Vec<String> {
        let mut set = BTreeSet::new();
        for r in &self.relations {
            set.insert(r.from.clone());
            set.insert(r.to.clone());
        }
        set.into_iter().collect()
    }

    /// Seluruh sifat sebuah simpul, termasuk yang diwarisi dari induknya.
    ///
    /// Pewarisan inilah alasan jaringan semantik dipakai: menuliskan bahwa
    /// burung bisa terbang satu kali sudah cukup untuk seluruh jenis burung.
    /// Batas kedalaman menjaga jaringan yang melingkar tidak berputar selamanya.
    pub fn properties_of(&self, node: &str) -> Result<Vec<Relation>, KnowledgeError> {
        if !self.nodes().iter().any(|n| n == node) {
            return Err(KnowledgeError::UnknownNode(node.to_string()));
        }
        let mut out: Vec<Relation> = Vec::new();
        let mut visited: BTreeSet<String> = BTreeSet::new();
        let mut queue = vec![node.to_string()];

        while let Some(current) = queue.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            for r in self.relations.iter().filter(|r| r.from == current) {
                if r.label == self.inheritance_label {
                    queue.push(r.to.clone());
                }
                out.push(r.clone());
            }
        }
        Ok(out)
    }

    /// Apakah `child` merupakan turunan dari `ancestor` lewat relasi pewarisan.
    pub fn is_a(&self, child: &str, ancestor: &str) -> bool {
        let mut visited: BTreeSet<String> = BTreeSet::new();
        let mut queue = vec![child.to_string()];
        while let Some(current) = queue.pop() {
            if current == ancestor && !visited.is_empty() {
                return true;
            }
            if !visited.insert(current.clone()) {
                continue;
            }
            for r in self
                .relations
                .iter()
                .filter(|r| r.from == current && r.label == self.inheritance_label)
            {
                if r.to == ancestor {
                    return true;
                }
                queue.push(r.to.clone());
            }
        }
        false
    }
}

/// Sebuah bingkai: kumpulan slot bernama, dengan induk untuk pewarisan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    /// Nama bingkai.
    pub name: String,
    /// Nama bingkai induk, bila ada.
    pub parent: Option<String>,
    /// Slot beserta isinya.
    pub slots: BTreeMap<String, String>,
}

/// Kumpulan bingkai yang saling mewarisi.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameSystem {
    /// Bingkai-bingkai yang tersedia.
    pub frames: Vec<Frame>,
}

impl FrameSystem {
    /// Mencari sebuah bingkai berdasarkan namanya.
    pub fn get(&self, name: &str) -> Option<&Frame> {
        self.frames.iter().find(|f| f.name == name)
    }

    /// Seluruh slot sebuah bingkai, termasuk yang diwarisi.
    ///
    /// Slot yang ditulis di bingkai anak menimpa milik induknya. Inilah yang
    /// membuat bingkai berguna: pengecualian bisa dinyatakan tanpa membatalkan
    /// aturan umumnya, misalnya pinguin yang tidak bisa terbang.
    pub fn resolve_slots(&self, name: &str) -> Result<BTreeMap<String, String>, KnowledgeError> {
        let mut chain: Vec<&Frame> = Vec::new();
        let mut current = self
            .get(name)
            .ok_or_else(|| KnowledgeError::UnknownNode(name.to_string()))?;
        let mut visited: BTreeSet<&str> = BTreeSet::new();

        loop {
            if !visited.insert(current.name.as_str()) {
                // Rantai pewarisan yang melingkar dihentikan di sini.
                break;
            }
            chain.push(current);
            match current.parent.as_deref().and_then(|p| self.get(p)) {
                Some(parent) => current = parent,
                None => break,
            }
        }

        // Diisi dari induk paling jauh supaya anak menimpa induknya.
        let mut out = BTreeMap::new();
        for frame in chain.iter().rev() {
            for (k, v) in &frame.slots {
                out.insert(k.clone(), v.clone());
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Formula {
        parse(s).unwrap_or_else(|e| panic!("gagal menguraikan {s:?}: {e}"))
    }

    // ------------------------------------------------------------ penguraian

    #[test]
    fn menguraikan_proposisi_tunggal() {
        assert_eq!(p("P"), Formula::atom("P"));
        assert_eq!(p("  P  "), Formula::atom("P"));
    }

    #[test]
    fn menguraikan_seluruh_gaya_penulisan() {
        // Papan tulis, papan ketik, dan buku memakai lambang berbeda untuk
        // operator yang sama; ketiganya harus diterima.
        assert_eq!(p("¬P"), p("~P"));
        assert_eq!(p("¬P"), p("!P"));
        assert_eq!(p("¬P"), p("not P"));
        assert_eq!(p("P ∧ Q"), p("P & Q"));
        assert_eq!(p("P ∧ Q"), p("P && Q"));
        assert_eq!(p("P ∧ Q"), p("P and Q"));
        assert_eq!(p("P ∨ Q"), p("P | Q"));
        assert_eq!(p("P ∨ Q"), p("P or Q"));
        assert_eq!(p("P → Q"), p("P -> Q"));
        assert_eq!(p("P → Q"), p("P => Q"));
        assert_eq!(p("P ↔ Q"), p("P <-> Q"));
        assert_eq!(p("P ↔ Q"), p("P iff Q"));
    }

    #[test]
    fn keutamaan_operator_benar() {
        // ¬ mengikat paling erat, lalu ∧, ∨, →, dan ↔ paling longgar.
        assert_eq!(p("¬P ∧ Q"), Formula::and(p("¬P"), p("Q")));
        assert_eq!(p("P ∧ Q ∨ R"), Formula::or(p("P ∧ Q"), p("R")));
        assert_eq!(p("P ∨ Q → R"), Formula::implies(p("P ∨ Q"), p("R")));
        assert_eq!(p("P → Q ↔ R"), Formula::iff(p("P → Q"), p("R")));
    }

    #[test]
    fn implikasi_berasosiasi_ke_kanan() {
        // Konvensi baku: a → b → c berarti a → (b → c).
        assert_eq!(p("P -> Q -> R"), Formula::implies(p("P"), p("Q -> R")));
    }

    #[test]
    fn konjungsi_berasosiasi_ke_kiri() {
        assert_eq!(p("P & Q & R"), Formula::and(p("P & Q"), p("R")));
    }

    #[test]
    fn kurung_mengalahkan_keutamaan() {
        assert_eq!(p("¬(P ∧ Q)"), Formula::not(p("P ∧ Q")));
        assert_ne!(p("¬(P ∧ Q)"), p("¬P ∧ Q"));
    }

    #[test]
    fn penguraian_menolak_masukan_rusak() {
        for buruk in ["", "   ", "P ∧", "∧ P", "(P", "P)", "P Q", "P @ Q"] {
            assert!(
                parse(buruk).is_err(),
                "{buruk:?} seharusnya gagal diuraikan"
            );
        }
    }

    #[test]
    fn galat_penguraian_menyebut_posisi() {
        match parse("P @ Q") {
            Err(KnowledgeError::ParseError { position, .. }) => assert_eq!(position, 2),
            other => panic!("seharusnya galat berposisi, bukan {other:?}"),
        }
    }

    #[test]
    fn bentuk_teks_bisa_diuraikan_kembali() {
        for teks in [
            "P",
            "¬P",
            "P ∧ Q",
            "P ∨ (Q ∧ R)",
            "(P → Q) ∧ (Q → R)",
            "P ↔ ¬Q",
        ] {
            let f = p(teks);
            let ulang = parse(&f.to_text()).unwrap();
            assert_eq!(f, ulang, "bentuk teks {:?} tidak setara", f.to_text());
        }
    }

    // ------------------------------------------------------- tabel kebenaran

    #[test]
    fn daftar_proposisi_terurut_dan_unik() {
        assert_eq!(p("Q ∧ P ∧ Q").variables(), vec!["P", "Q"]);
        assert_eq!(p("P").variables(), vec!["P"]);
    }

    #[test]
    fn tabel_kebenaran_konjungsi() {
        let t = truth_table(&p("P ∧ Q")).unwrap();
        assert_eq!(t.rows.len(), 4);
        assert_eq!(
            t.rows.iter().map(|r| r.result).collect::<Vec<_>>(),
            vec![false, false, false, true]
        );
        assert!(!t.tautology);
        assert!(t.satisfiable);
        assert!(!t.contradiction);
    }

    #[test]
    fn tabel_kebenaran_implikasi() {
        // Implikasi hanya salah bila anteseden benar dan konsekuen salah.
        let t = truth_table(&p("P → Q")).unwrap();
        assert_eq!(
            t.rows.iter().map(|r| r.result).collect::<Vec<_>>(),
            vec![true, true, false, true]
        );
    }

    #[test]
    fn mengenali_tautologi_dan_kontradiksi() {
        assert!(truth_table(&p("P ∨ ¬P")).unwrap().tautology);
        assert!(truth_table(&p("P ∧ ¬P")).unwrap().contradiction);
        assert!(!truth_table(&p("P ∧ ¬P")).unwrap().satisfiable);
        // Modus ponens adalah tautologi.
        assert!(truth_table(&p("((P → Q) ∧ P) → Q")).unwrap().tautology);
        // Begitu pula modus tollens.
        assert!(truth_table(&p("((P → Q) ∧ ¬Q) → ¬P")).unwrap().tautology);
    }

    #[test]
    fn hukum_de_morgan_terbukti() {
        assert!(equivalent(&p("¬(P ∧ Q)"), &p("¬P ∨ ¬Q")).unwrap());
        assert!(equivalent(&p("¬(P ∨ Q)"), &p("¬P ∧ ¬Q")).unwrap());
    }

    #[test]
    fn implikasi_setara_dengan_disjungsi() {
        assert!(equivalent(&p("P → Q"), &p("¬P ∨ Q")).unwrap());
        assert!(!equivalent(&p("P → Q"), &p("Q → P")).unwrap());
    }

    #[test]
    fn tabel_menolak_proposisi_terlalu_banyak() {
        let banyak: Vec<String> = (0..MAX_VARIABLES + 1).map(|i| format!("P{i}")).collect();
        let rumus = p(&banyak.join(" ∧ "));
        assert!(matches!(
            truth_table(&rumus),
            Err(KnowledgeError::TooManyVariables(_))
        ));
    }

    #[test]
    fn urutan_baris_seperti_tabel_tulis_tangan() {
        // Baris pertama semua salah, baris terakhir semua benar.
        let t = truth_table(&p("P ∧ Q")).unwrap();
        assert_eq!(t.rows[0].values, vec![false, false]);
        assert_eq!(t.rows[3].values, vec![true, true]);
        assert_eq!(t.rows[1].values, vec![false, true]);
    }

    // ------------------------------------------------------------------ CNF

    #[test]
    fn cnf_proposisi_tunggal() {
        let klausa = to_cnf(&p("P"));
        assert_eq!(klausa.len(), 1);
        assert_eq!(clause_text(&klausa[0]), "P");
    }

    #[test]
    fn cnf_membuang_implikasi() {
        let klausa = to_cnf(&p("P → Q"));
        assert_eq!(klausa.len(), 1);
        let teks = clause_text(&klausa[0]);
        assert!(teks.contains("¬P"), "{teks}");
        assert!(teks.contains('Q'), "{teks}");
    }

    #[test]
    fn cnf_menerapkan_de_morgan() {
        let klausa = to_cnf(&p("¬(P ∧ Q)"));
        assert_eq!(klausa.len(), 1);
        let teks = clause_text(&klausa[0]);
        assert!(teks.contains("¬P") && teks.contains("¬Q"), "{teks}");
    }

    #[test]
    fn cnf_setara_dengan_rumus_asalnya() {
        // Uji yang paling berarti: bentuk normalnya harus punya tabel kebenaran
        // yang sama persis dengan rumus asalnya.
        for teks in [
            "P → Q",
            "¬(P ∧ Q)",
            "P ↔ Q",
            "(P ∨ Q) ∧ (¬P ∨ R)",
            "¬(P → (Q ∧ R))",
        ] {
            let asli = p(teks);
            let klausa = to_cnf(&asli);
            let variables = asli.variables();
            let n = variables.len();
            for mask in 0..(1usize << n) {
                let mut assignment = BTreeMap::new();
                for (i, name) in variables.iter().enumerate() {
                    assignment.insert(name.clone(), mask & (1 << i) != 0);
                }
                let asli_benar = asli.evaluate(&assignment);
                let cnf_benar = klausa.iter().all(|c| {
                    c.iter().any(|l| {
                        let v = assignment.get(&l.name).copied().unwrap_or(false);
                        if l.negated {
                            !v
                        } else {
                            v
                        }
                    })
                });
                assert_eq!(asli_benar, cnf_benar, "{teks} pada penugasan {mask:b}");
            }
        }
    }

    #[test]
    fn klausa_tautologis_dikenali() {
        let klausa: Clause = BTreeSet::from([Literal::positive("P"), Literal::negative("P")]);
        assert!(is_tautological_clause(&klausa));
        let biasa: Clause = BTreeSet::from([Literal::positive("P"), Literal::positive("Q")]);
        assert!(!is_tautological_clause(&biasa));
    }

    #[test]
    fn klausa_kosong_ditulis_sebagai_kotak() {
        assert_eq!(clause_text(&BTreeSet::new()), "□");
    }

    // ------------------------------------------------------------- resolusi

    #[test]
    fn resolusi_membuktikan_modus_ponens() {
        let kb = vec![p("P → Q"), p("P")];
        let hasil = resolve(&kb, &p("Q")).unwrap();
        assert!(hasil.proved);
        assert!(!hasil.steps.is_empty());
        assert!(hasil.steps.last().unwrap().result.contains('□'));
    }

    #[test]
    fn resolusi_membuktikan_rantai_implikasi() {
        let kb = vec![p("P → Q"), p("Q → R"), p("P")];
        assert!(resolve(&kb, &p("R")).unwrap().proved);
    }

    #[test]
    fn resolusi_membuktikan_modus_tollens() {
        let kb = vec![p("P → Q"), p("¬Q")];
        assert!(resolve(&kb, &p("¬P")).unwrap().proved);
    }

    #[test]
    fn resolusi_menolak_kesimpulan_yang_tidak_mengikuti() {
        let kb = vec![p("P → Q"), p("Q")];
        // Menegaskan konsekuen adalah kekeliruan; P tidak mengikuti.
        let hasil = resolve(&kb, &p("P")).unwrap();
        assert!(!hasil.proved);
    }

    #[test]
    fn resolusi_sepakat_dengan_tabel_kebenaran() {
        // Dua cara menjawab pertanyaan yang sama harus memberi jawaban sama.
        let kasus: [(&[&str], &str); 5] = [
            (&["P -> Q", "P"], "Q"),
            (&["P -> Q", "Q -> R", "P"], "R"),
            (&["P -> Q", "~Q"], "~P"),
            (&["P -> Q", "Q"], "P"),
            (&["P | Q", "~P"], "Q"),
        ];
        for (kb_text, kesimpulan) in kasus {
            let kb: Vec<Formula> = kb_text.iter().map(|t| p(t)).collect();
            let target = p(kesimpulan);

            let lewat_resolusi = resolve(&kb, &target).unwrap().proved;

            // Lewat tabel: kb → kesimpulan harus tautologi.
            let gabungan = kb
                .iter()
                .cloned()
                .reduce(Formula::and)
                .expect("basis tidak kosong");
            let lewat_tabel = truth_table(&Formula::implies(gabungan, target))
                .unwrap()
                .tautology;

            assert_eq!(
                lewat_resolusi, lewat_tabel,
                "{kb_text:?} ⊢ {kesimpulan} berbeda antara resolusi dan tabel"
            );
        }
    }

    #[test]
    fn resolusi_mendeteksi_basis_yang_kontradiktif() {
        let kb = vec![p("P"), p("¬P")];
        // Dari kontradiksi, apa pun bisa dibuktikan.
        assert!(resolve(&kb, &p("Q")).unwrap().proved);
    }

    #[test]
    fn resolusi_menolak_basis_kosong() {
        assert_eq!(
            resolve(&[], &p("P")),
            Err(KnowledgeError::EmptyKnowledgeBase)
        );
    }

    #[test]
    fn resolusi_merekam_klausa_awal_dan_jejaknya() {
        let kb = vec![p("P → Q"), p("P")];
        let hasil = resolve(&kb, &p("Q")).unwrap();
        assert!(
            hasil.initial_clauses.len() >= 3,
            "termasuk ingkaran kesimpulan"
        );
        assert!(hasil.generated >= hasil.initial_clauses.len());
        for step in &hasil.steps {
            assert!(!step.left.is_empty());
            assert!(!step.right.is_empty());
            assert!(!step.pivot.is_empty());
        }
    }

    // ------------------------------------------------------ jaringan semantik

    fn jaringan_hewan() -> SemanticNetwork {
        let mut n = SemanticNetwork::new();
        n.add("burung", "adalah", "hewan");
        n.add("burung", "punya", "sayap");
        n.add("burung", "bisa", "terbang");
        n.add("pinguin", "adalah", "burung");
        n.add("pinguin", "bisa", "berenang");
        n.add("hewan", "punya", "sel");
        n
    }

    #[test]
    fn jaringan_mendaftar_simpulnya() {
        let n = jaringan_hewan();
        let simpul = n.nodes();
        assert!(simpul.contains(&"burung".to_string()));
        assert!(simpul.contains(&"sayap".to_string()));
        assert_eq!(simpul, {
            let mut v = simpul.clone();
            v.sort();
            v
        });
    }

    #[test]
    fn sifat_diwarisi_dari_induk() {
        // Inilah gunanya jaringan semantik: menuliskan sekali, berlaku untuk
        // seluruh turunannya.
        let n = jaringan_hewan();
        let sifat = n.properties_of("pinguin").unwrap();
        let teks: Vec<String> = sifat
            .iter()
            .map(|r| format!("{} {} {}", r.from, r.label, r.to))
            .collect();
        assert!(teks.iter().any(|t| t.contains("berenang")), "{teks:?}");
        assert!(
            teks.iter().any(|t| t.contains("sayap")),
            "sifat burung tidak diwarisi"
        );
        assert!(
            teks.iter().any(|t| t.contains("sel")),
            "sifat hewan tidak diwarisi"
        );
    }

    #[test]
    fn pewarisan_menelusuri_beberapa_tingkat() {
        let n = jaringan_hewan();
        assert!(n.is_a("pinguin", "burung"));
        assert!(n.is_a("pinguin", "hewan"));
        assert!(n.is_a("burung", "hewan"));
        assert!(!n.is_a("hewan", "pinguin"));
        assert!(!n.is_a("pinguin", "ikan"));
    }

    #[test]
    fn jaringan_melingkar_tidak_berputar_selamanya() {
        let mut n = SemanticNetwork::new();
        n.add("a", "adalah", "b");
        n.add("b", "adalah", "a");
        // Harus selesai, bukan menggantung.
        let sifat = n.properties_of("a").unwrap();
        assert!(!sifat.is_empty());
        assert!(n.is_a("a", "b"));
    }

    #[test]
    fn simpul_tak_dikenal_ditolak() {
        let n = jaringan_hewan();
        assert!(matches!(
            n.properties_of("naga"),
            Err(KnowledgeError::UnknownNode(_))
        ));
    }

    // --------------------------------------------------------------- bingkai

    fn sistem_bingkai() -> FrameSystem {
        FrameSystem {
            frames: vec![
                Frame {
                    name: "burung".into(),
                    parent: None,
                    slots: BTreeMap::from([
                        ("bergerak".into(), "terbang".into()),
                        ("kaki".into(), "2".into()),
                        ("darah".into(), "panas".into()),
                    ]),
                },
                Frame {
                    name: "pinguin".into(),
                    parent: Some("burung".into()),
                    slots: BTreeMap::from([
                        ("bergerak".into(), "berenang".into()),
                        ("habitat".into(), "kutub".into()),
                    ]),
                },
            ],
        }
    }

    #[test]
    fn bingkai_anak_menimpa_induknya() {
        // Pengecualian bisa dinyatakan tanpa membatalkan aturan umumnya.
        let s = sistem_bingkai();
        let slots = s.resolve_slots("pinguin").unwrap();
        assert_eq!(slots["bergerak"], "berenang", "pengecualian tidak menimpa");
        assert_eq!(slots["kaki"], "2", "slot induk tidak diwarisi");
        assert_eq!(slots["darah"], "panas");
        assert_eq!(slots["habitat"], "kutub");
    }

    #[test]
    fn bingkai_tanpa_induk_hanya_slotnya_sendiri() {
        let s = sistem_bingkai();
        let slots = s.resolve_slots("burung").unwrap();
        assert_eq!(slots.len(), 3);
        assert!(!slots.contains_key("habitat"));
    }

    #[test]
    fn bingkai_tak_dikenal_ditolak() {
        let s = sistem_bingkai();
        assert!(matches!(
            s.resolve_slots("naga"),
            Err(KnowledgeError::UnknownNode(_))
        ));
        assert!(s.get("naga").is_none());
    }

    #[test]
    fn rantai_bingkai_melingkar_tidak_berputar_selamanya() {
        let s = FrameSystem {
            frames: vec![
                Frame {
                    name: "a".into(),
                    parent: Some("b".into()),
                    slots: BTreeMap::from([("x".into(), "1".into())]),
                },
                Frame {
                    name: "b".into(),
                    parent: Some("a".into()),
                    slots: BTreeMap::from([("y".into(), "2".into())]),
                },
            ],
        };
        let slots = s.resolve_slots("a").unwrap();
        assert_eq!(slots["x"], "1");
        assert_eq!(slots["y"], "2");
    }

    // ---------------------------------------------------------- serialisasi

    #[test]
    fn hasil_bisa_di_serialisasi() {
        let f = p("(P → Q) ∧ ¬R");
        let json = serde_json::to_string(&f).unwrap();
        assert_eq!(serde_json::from_str::<Formula>(&json).unwrap(), f);
        assert!(json.contains(r#""kind":"and""#), "{json}");

        let t = truth_table(&f).unwrap();
        let tj = serde_json::to_string(&t).unwrap();
        assert_eq!(serde_json::from_str::<TruthTable>(&tj).unwrap(), t);

        let n = jaringan_hewan();
        let nj = serde_json::to_string(&n).unwrap();
        assert_eq!(serde_json::from_str::<SemanticNetwork>(&nj).unwrap(), n);
    }

    #[test]
    fn pesan_error_terbaca() {
        for e in [
            KnowledgeError::ParseError {
                cause: ParseCause::EmptyFormula,
                position: 1,
            },
            KnowledgeError::ParseError {
                cause: ParseCause::UnknownCharacter('%'),
                position: 2,
            },
            KnowledgeError::ParseError {
                cause: ParseCause::MissingCloseParen,
                position: 3,
            },
            KnowledgeError::ParseError {
                cause: ParseCause::OperatorWithoutOperand,
                position: 4,
            },
            KnowledgeError::ParseError {
                cause: ParseCause::UnexpectedEnd,
                position: 5,
            },
            KnowledgeError::ParseError {
                cause: ParseCause::TrailingInput,
                position: 6,
            },
            KnowledgeError::TooManyVariables(30),
            KnowledgeError::EmptyKnowledgeBase,
            KnowledgeError::UnknownNode("a".into()),
            KnowledgeError::ProofLimitExceeded(10),
        ] {
            assert!(!e.to_string().is_empty());
        }
    }
}
