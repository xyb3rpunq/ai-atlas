//! Sesi 12 & 13 — Sains Data dan Machine Learning.
//!
//! Lima algoritma pembelajaran mesin klasik yang ditulis dari nol, ditambah
//! perkakas evaluasi yang membuat angkanya bisa dipercaya:
//!
//! - **K-Nearest Neighbours** — pengklasifikasi malas; tidak ada model yang
//!   dilatih, semua kerja terjadi saat memprediksi.
//! - **K-Means** — pengelompokan tanpa label, dengan penempatan pusat awal
//!   K-Means++ agar tidak terjebak hasil buruk.
//! - **Pohon Keputusan ID3** — memakai entropi dan perolehan informasi.
//! - **Regresi Linear** — bentuk tertutup kuadrat terkecil.
//! - **Regresi Logistik** — penurunan gradien pada peluang log.
//!
//! Bagian evaluasinya sengaja selengkap algoritmanya. Model yang dinilai hanya
//! dengan ketepatan bisa terlihat hebat sambil gagal total: pada data yang
//! sembilan puluh sembilan persen satu kelas, menebak kelas itu terus-menerus
//! sudah memberi ketepatan sembilan puluh sembilan persen.

use crate::rng::SplitMix64;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Kesalahan pada pembelajaran mesin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MlError {
    /// Kumpulan data kosong.
    EmptyDataset,
    /// Jumlah baris fitur dan label berbeda.
    LengthMismatch {
        /// Jumlah baris fitur.
        features: usize,
        /// Jumlah baris label.
        labels: usize,
    },
    /// Baris data punya jumlah fitur yang berbeda-beda.
    RaggedRows {
        /// Jumlah fitur yang diharapkan.
        expected: usize,
        /// Jumlah fitur yang ditemukan.
        got: usize,
    },
    /// Nilai parameter tidak masuk akal.
    BadParameter {
        /// Nama parameter.
        name: String,
        /// Nilai yang diberikan.
        value: f64,
    },
    /// Model dipakai sebelum dilatih.
    NotTrained,
    /// Jumlah kelompok melebihi jumlah titik data.
    TooManyClusters {
        /// Jumlah kelompok yang diminta.
        k: usize,
        /// Jumlah titik yang tersedia.
        points: usize,
    },
    /// Data memuat nilai yang bukan bilangan.
    NonFiniteValue {
        /// Baris tempat nilai itu ditemukan.
        row: usize,
        /// Kolom tempat nilai itu ditemukan.
        column: usize,
    },
}

impl crate::galat::Dijelaskan for MlError {
    fn kode(&self) -> &'static str {
        match self {
            MlError::EmptyDataset => "ml.data_kosong",
            MlError::LengthMismatch { .. } => "ml.panjang_tak_sepadan",
            MlError::RaggedRows { .. } => "ml.baris_tak_rata",
            MlError::BadParameter { .. } => "ml.parameter_tak_sah",
            MlError::NotTrained => "ml.belum_dilatih",
            MlError::TooManyClusters { .. } => "ml.kelompok_terlalu_banyak",
            MlError::NonFiniteValue { .. } => "ml.nilai_bukan_bilangan",
        }
    }

    fn argumen(&self) -> Vec<String> {
        match self {
            MlError::EmptyDataset | MlError::NotTrained => Vec::new(),
            MlError::LengthMismatch { features, labels } => {
                vec![features.to_string(), labels.to_string()]
            }
            MlError::RaggedRows { expected, got } => {
                vec![expected.to_string(), got.to_string()]
            }
            MlError::BadParameter { name, value } => vec![name.clone(), value.to_string()],
            MlError::TooManyClusters { k, points } => {
                vec![k.to_string(), points.to_string()]
            }
            MlError::NonFiniteValue { row, column } => {
                vec![row.to_string(), column.to_string()]
            }
        }
    }
}

impl core::fmt::Display for MlError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MlError::EmptyDataset => write!(f, "kumpulan data kosong"),
            MlError::LengthMismatch { features, labels } => {
                write!(f, "{features} baris fitur tetapi {labels} label")
            }
            MlError::RaggedRows { expected, got } => {
                write!(f, "baris harus punya {expected} fitur, ditemukan {got}")
            }
            MlError::BadParameter { name, value } => {
                write!(f, "parameter {name} tidak sah: {value}")
            }
            MlError::NotTrained => write!(f, "model belum dilatih"),
            MlError::TooManyClusters { k, points } => {
                write!(f, "{k} kelompok diminta untuk {points} titik")
            }
            MlError::NonFiniteValue { row, column } => {
                write!(f, "nilai bukan bilangan pada baris {row} kolom {column}")
            }
        }
    }
}

/// Memeriksa bahwa matriks fitur berbentuk persegi dan seluruh nilainya sah.
fn validate_matrix(x: &[Vec<f64>]) -> Result<usize, MlError> {
    let first = x.first().ok_or(MlError::EmptyDataset)?;
    let width = first.len();
    if width == 0 {
        return Err(MlError::EmptyDataset);
    }
    for (r, row) in x.iter().enumerate() {
        if row.len() != width {
            return Err(MlError::RaggedRows {
                expected: width,
                got: row.len(),
            });
        }
        for (c, v) in row.iter().enumerate() {
            if !v.is_finite() {
                return Err(MlError::NonFiniteValue { row: r, column: c });
            }
        }
    }
    Ok(width)
}

// ---------------------------------------------------------------------------
// Ukuran jarak
// ---------------------------------------------------------------------------

/// Ukuran jarak antartitik.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Distance {
    /// Jarak lurus.
    Euclidean,
    /// Jumlah selisih tiap sumbu; tahan terhadap pencilan.
    Manhattan,
    /// Selisih terbesar di antara semua sumbu.
    Chebyshev,
}

impl Distance {
    /// Jarak antara dua titik. Panjang keduanya harus sama.
    pub fn between(self, a: &[f64], b: &[f64]) -> f64 {
        match self {
            Distance::Euclidean => a
                .iter()
                .zip(b)
                .map(|(p, q)| (p - q) * (p - q))
                .sum::<f64>()
                .sqrt(),
            Distance::Manhattan => a.iter().zip(b).map(|(p, q)| (p - q).abs()).sum(),
            Distance::Chebyshev => a
                .iter()
                .zip(b)
                .map(|(p, q)| (p - q).abs())
                .fold(0.0, f64::max),
        }
    }

    /// Nama pendek untuk ditampilkan.
    pub fn short_name(self) -> &'static str {
        match self {
            Distance::Euclidean => "Euclidean",
            Distance::Manhattan => "Manhattan",
            Distance::Chebyshev => "Chebyshev",
        }
    }
}

// ---------------------------------------------------------------------------
// Penskalaan
// ---------------------------------------------------------------------------

/// Ringkasan penskalaan satu kolom.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ColumnScale {
    /// Nilai terkecil pada kolom.
    pub min: f64,
    /// Nilai terbesar pada kolom.
    pub max: f64,
    /// Rerata kolom.
    pub mean: f64,
    /// Simpangan baku kolom, memakai pembagi `n-1` bila `n > 1`.
    pub std_dev: f64,
}

/// Menghitung ringkasan penskalaan tiap kolom.
pub fn column_scales(x: &[Vec<f64>]) -> Result<Vec<ColumnScale>, MlError> {
    let width = validate_matrix(x)?;
    let n = x.len() as f64;
    Ok((0..width)
        .map(|c| {
            let col: Vec<f64> = x.iter().map(|r| r[c]).collect();
            let mean = col.iter().sum::<f64>() / n;
            let var = if col.len() > 1 {
                col.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0)
            } else {
                0.0
            };
            ColumnScale {
                min: col.iter().copied().fold(f64::INFINITY, f64::min),
                max: col.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                mean,
                std_dev: var.sqrt(),
            }
        })
        .collect())
}

/// Menormalkan tiap kolom ke rentang `[0, 1]`.
///
/// Kolom yang seluruh nilainya sama dipetakan ke nol, bukan menghasilkan
/// pembagian dengan nol. Kolom seperti itu memang tidak membawa informasi.
pub fn min_max_scale(x: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, MlError> {
    let scales = column_scales(x)?;
    Ok(x.iter()
        .map(|row| {
            row.iter()
                .zip(&scales)
                .map(|(v, s)| {
                    let span = s.max - s.min;
                    if span.abs() < 1e-12 {
                        0.0
                    } else {
                        (v - s.min) / span
                    }
                })
                .collect()
        })
        .collect())
}

/// Membakukan tiap kolom menjadi rerata nol dan simpangan baku satu.
pub fn standardise(x: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, MlError> {
    let scales = column_scales(x)?;
    Ok(x.iter()
        .map(|row| {
            row.iter()
                .zip(&scales)
                .map(|(v, s)| {
                    if s.std_dev.abs() < 1e-12 {
                        0.0
                    } else {
                        (v - s.mean) / s.std_dev
                    }
                })
                .collect()
        })
        .collect())
}

// ---------------------------------------------------------------------------
// K-Nearest Neighbours
// ---------------------------------------------------------------------------

/// Satu tetangga beserta jaraknya.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Neighbour {
    /// Indeks baris pada data latih.
    pub index: usize,
    /// Jarak ke titik yang ditanyakan.
    pub distance: f64,
    /// Label baris itu.
    pub label: String,
}

/// Hasil klasifikasi KNN.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnnResult {
    /// Kelas terpilih.
    pub label: String,
    /// Tetangga terdekat yang dipakai, terurut dari yang paling dekat.
    pub neighbours: Vec<Neighbour>,
    /// Suara tiap kelas.
    pub votes: BTreeMap<String, f64>,
}

/// Pengklasifikasi K-Nearest Neighbours.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Knn {
    x: Vec<Vec<f64>>,
    y: Vec<String>,
    /// Jumlah tetangga yang ikut memilih.
    pub k: usize,
    /// Ukuran jarak yang dipakai.
    pub distance: Distance,
    /// Bila benar, suara tetangga ditimbang kebalikan jaraknya.
    pub weighted: bool,
}

impl Knn {
    /// Membuat pengklasifikasi dari data latih.
    ///
    /// KNN tidak melatih apa pun; data latihnya disimpan apa adanya dan seluruh
    /// kerja terjadi saat memprediksi. Itulah sebabnya ia disebut pembelajar
    /// malas, dan itu pula sebabnya ia lambat pada data besar.
    pub fn new(
        x: Vec<Vec<f64>>,
        y: Vec<String>,
        k: usize,
        distance: Distance,
        weighted: bool,
    ) -> Result<Self, MlError> {
        validate_matrix(&x)?;
        if x.len() != y.len() {
            return Err(MlError::LengthMismatch {
                features: x.len(),
                labels: y.len(),
            });
        }
        if k == 0 || k > x.len() {
            return Err(MlError::BadParameter {
                name: "k".into(),
                value: k as f64,
            });
        }
        Ok(Self {
            x,
            y,
            k,
            distance,
            weighted,
        })
    }

    /// Jumlah baris data latih.
    pub fn len(&self) -> usize {
        self.x.len()
    }

    /// Apakah data latihnya kosong.
    pub fn is_empty(&self) -> bool {
        self.x.is_empty()
    }

    /// Memprediksi kelas sebuah titik.
    pub fn predict(&self, query: &[f64]) -> Result<KnnResult, MlError> {
        let width = self.x[0].len();
        if query.len() != width {
            return Err(MlError::RaggedRows {
                expected: width,
                got: query.len(),
            });
        }

        let mut all: Vec<Neighbour> = self
            .x
            .iter()
            .enumerate()
            .map(|(i, row)| Neighbour {
                index: i,
                distance: self.distance.between(row, query),
                label: self.y[i].clone(),
            })
            .collect();
        // Seri jarak diputus oleh indeks agar hasilnya deterministik.
        all.sort_by(|a, b| {
            a.distance
                .total_cmp(&b.distance)
                .then(a.index.cmp(&b.index))
        });
        all.truncate(self.k);

        let mut votes: BTreeMap<String, f64> = BTreeMap::new();
        for n in &all {
            // Bobot kebalikan jarak; jarak nol berarti titiknya persis sama,
            // dan diberi bobot sangat besar alih-alih membagi dengan nol.
            let weight = if self.weighted {
                if n.distance < 1e-12 {
                    1e12
                } else {
                    1.0 / n.distance
                }
            } else {
                1.0
            };
            *votes.entry(n.label.clone()).or_insert(0.0) += weight;
        }

        let label = votes
            .iter()
            .fold(None::<(&String, f64)>, |best, (k, v)| match best {
                Some((bk, bv)) if bv > *v || (bv == *v && bk <= k) => Some((bk, bv)),
                _ => Some((k, *v)),
            })
            .map(|(k, _)| k.clone())
            .ok_or(MlError::NotTrained)?;

        Ok(KnnResult {
            label,
            neighbours: all,
            votes,
        })
    }
}

// ---------------------------------------------------------------------------
// K-Means
// ---------------------------------------------------------------------------

/// Hasil pengelompokan K-Means.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Clustering {
    /// Titik pusat tiap kelompok.
    pub centroids: Vec<Vec<f64>>,
    /// Kelompok tiap baris data.
    pub assignments: Vec<usize>,
    /// Jumlah kuadrat jarak ke pusat kelompoknya, makin kecil makin rapat.
    pub inertia: f64,
    /// Berapa kali seluruh data disapu sebelum keadaan tetap.
    pub iterations: usize,
    /// Apakah pengelompokan benar-benar mencapai keadaan tetap.
    pub converged: bool,
}

/// Menempatkan pusat awal dengan K-Means++.
///
/// Pusat pertama diambil acak, lalu tiap pusat berikutnya dipilih dengan
/// peluang sebanding kuadrat jaraknya ke pusat terdekat yang sudah ada.
/// Penempatan acak biasa sering menaruh dua pusat pada gugus yang sama, dan
/// hasilnya terkunci di pengelompokan yang jelas keliru.
fn kmeans_plus_plus(
    x: &[Vec<f64>],
    k: usize,
    distance: Distance,
    rng: &mut SplitMix64,
) -> Vec<Vec<f64>> {
    let mut centroids: Vec<Vec<f64>> = Vec::with_capacity(k);
    centroids.push(x[rng.below(x.len() as u64) as usize].clone());

    while centroids.len() < k {
        let squared: Vec<f64> = x
            .iter()
            .map(|p| {
                let d = centroids
                    .iter()
                    .map(|c| distance.between(p, c))
                    .fold(f64::INFINITY, f64::min);
                d * d
            })
            .collect();
        let total: f64 = squared.iter().sum();
        if total < 1e-12 {
            // Seluruh titik berimpit dengan pusat yang ada; sisanya diisi
            // titik mana pun agar jumlah pusatnya tetap terpenuhi.
            centroids.push(x[rng.below(x.len() as u64) as usize].clone());
            continue;
        }
        let target = rng.next_f64() * total;
        let mut acc = 0.0;
        let mut chosen = x.len() - 1;
        for (i, s) in squared.iter().enumerate() {
            acc += s;
            if acc >= target {
                chosen = i;
                break;
            }
        }
        centroids.push(x[chosen].clone());
    }
    centroids
}

/// Mengelompokkan data menjadi `k` kelompok.
pub fn kmeans(
    x: &[Vec<f64>],
    k: usize,
    distance: Distance,
    max_iterations: usize,
    seed: u64,
) -> Result<Clustering, MlError> {
    let width = validate_matrix(x)?;
    if k == 0 {
        return Err(MlError::BadParameter {
            name: "k".into(),
            value: 0.0,
        });
    }
    if k > x.len() {
        return Err(MlError::TooManyClusters { k, points: x.len() });
    }

    let mut rng = SplitMix64::new(seed);
    let mut centroids = kmeans_plus_plus(x, k, distance, &mut rng);
    let mut assignments = vec![0usize; x.len()];
    let mut converged = false;
    let mut iterations = 0usize;

    for _ in 0..max_iterations.max(1) {
        iterations += 1;
        let mut changed = false;

        for (i, point) in x.iter().enumerate() {
            let mut best = 0usize;
            let mut best_distance = f64::INFINITY;
            for (c, centroid) in centroids.iter().enumerate() {
                let d = distance.between(point, centroid);
                if d < best_distance {
                    best_distance = d;
                    best = c;
                }
            }
            if assignments[i] != best {
                assignments[i] = best;
                changed = true;
            }
        }

        // Pusat baru adalah rerata anggotanya. Kelompok yang kehilangan seluruh
        // anggotanya dipindahkan ke titik terjauh, bukan dibiarkan kosong —
        // kelompok kosong membuat jumlah kelompok yang diminta tidak terpenuhi.
        for c in 0..k {
            let members: Vec<&Vec<f64>> = x
                .iter()
                .enumerate()
                .filter(|(i, _)| assignments[*i] == c)
                .map(|(_, p)| p)
                .collect();
            if members.is_empty() {
                let furthest = x
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| {
                        let da = centroids
                            .iter()
                            .map(|c| distance.between(a, c))
                            .fold(f64::INFINITY, f64::min);
                        let db = centroids
                            .iter()
                            .map(|c| distance.between(b, c))
                            .fold(f64::INFINITY, f64::min);
                        da.total_cmp(&db)
                    })
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                centroids[c] = x[furthest].clone();
                changed = true;
                continue;
            }
            let n = members.len() as f64;
            centroids[c] = (0..width)
                .map(|d| members.iter().map(|m| m[d]).sum::<f64>() / n)
                .collect();
        }

        if !changed {
            converged = true;
            break;
        }
    }

    let inertia = x
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let d = distance.between(p, &centroids[assignments[i]]);
            d * d
        })
        .sum();

    Ok(Clustering {
        centroids,
        assignments,
        inertia,
        iterations,
        converged,
    })
}

// ---------------------------------------------------------------------------
// Pohon Keputusan ID3
// ---------------------------------------------------------------------------

/// Entropi Shannon sebuah sebaran label, dalam satuan bit.
///
/// Bernilai nol bila seluruh label sama, dan maksimum bila seluruh kelas
/// muncul sama banyak.
pub fn entropy(labels: &[String]) -> f64 {
    if labels.is_empty() {
        return 0.0;
    }
    let n = labels.len() as f64;
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for l in labels {
        *counts.entry(l.as_str()).or_insert(0) += 1;
    }
    -counts
        .values()
        .map(|c| {
            let p = *c as f64 / n;
            p * p.log2()
        })
        .sum::<f64>()
}

/// Ketakmurnian Gini sebuah sebaran label.
pub fn gini(labels: &[String]) -> f64 {
    if labels.is_empty() {
        return 0.0;
    }
    let n = labels.len() as f64;
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for l in labels {
        *counts.entry(l.as_str()).or_insert(0) += 1;
    }
    1.0 - counts
        .values()
        .map(|c| {
            let p = *c as f64 / n;
            p * p
        })
        .sum::<f64>()
}

/// Perolehan informasi bila data dipecah menurut sebuah atribut.
///
/// Nilainya adalah entropi sebelum pemecahan dikurangi rerata berbobot
/// entropi tiap cabang. Atribut dengan perolehan tertinggi yang dipilih ID3.
pub fn information_gain(values: &[String], labels: &[String]) -> f64 {
    if values.len() != labels.len() || labels.is_empty() {
        return 0.0;
    }
    let before = entropy(labels);
    let n = labels.len() as f64;
    let mut groups: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for (v, l) in values.iter().zip(labels) {
        groups.entry(v.as_str()).or_default().push(l.clone());
    }
    let after: f64 = groups
        .values()
        .map(|g| (g.len() as f64 / n) * entropy(g))
        .sum();
    before - after
}

/// Satu simpul pohon keputusan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TreeNode {
    /// Daun: seluruh data di sini berlabel sama, atau tidak ada lagi atribut.
    Leaf {
        /// Label yang diputuskan.
        label: String,
        /// Berapa baris data yang sampai di daun ini.
        samples: usize,
        /// Seberapa murni daun ini, `1.0` berarti seluruhnya satu label.
        purity: f64,
    },
    /// Cabang: data dipecah menurut sebuah atribut.
    Branch {
        /// Indeks atribut yang dipakai memecah.
        attribute: usize,
        /// Nama atribut, untuk ditampilkan.
        attribute_name: String,
        /// Perolehan informasi dari pemecahan ini.
        gain: f64,
        /// Anak-anak simpul menurut nilai atributnya.
        children: BTreeMap<String, TreeNode>,
        /// Label mayoritas, dipakai bila muncul nilai atribut yang belum pernah dilihat.
        fallback: String,
    },
}

impl TreeNode {
    /// Kedalaman pohon; daun berkedalaman satu.
    pub fn depth(&self) -> usize {
        match self {
            TreeNode::Leaf { .. } => 1,
            TreeNode::Branch { children, .. } => {
                1 + children.values().map(|c| c.depth()).max().unwrap_or(0)
            }
        }
    }

    /// Jumlah daun.
    pub fn leaf_count(&self) -> usize {
        match self {
            TreeNode::Leaf { .. } => 1,
            TreeNode::Branch { children, .. } => children.values().map(|c| c.leaf_count()).sum(),
        }
    }

    /// Memprediksi label sebuah baris atribut kategorikal.
    pub fn predict(&self, row: &[String]) -> String {
        match self {
            TreeNode::Leaf { label, .. } => label.clone(),
            TreeNode::Branch {
                attribute,
                children,
                fallback,
                ..
            } => match row.get(*attribute).and_then(|v| children.get(v)) {
                Some(child) => child.predict(row),
                // Nilai yang belum pernah dilihat saat pelatihan jatuh ke label
                // mayoritas. Tanpa ini, pohon tidak bisa menjawab sama sekali.
                None => fallback.clone(),
            },
        }
    }
}

/// Label yang paling sering muncul. Seri diputus urutan abjad.
fn majority(labels: &[String]) -> String {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for l in labels {
        *counts.entry(l.as_str()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .fold(None::<(&str, usize)>, |best, (k, v)| match best {
            Some((bk, bv)) if bv > v || (bv == v && bk <= k) => Some((bk, bv)),
            _ => Some((k, v)),
        })
        .map(|(k, _)| k.to_string())
        .unwrap_or_default()
}

/// Membangun pohon keputusan ID3 dari data kategorikal.
///
/// `max_depth` dihitung dengan satuan yang sama seperti [`TreeNode::depth`]:
/// pohon berisi satu daun berkedalaman satu. Jadi `max_depth == 1` menghasilkan
/// satu daun berisi label mayoritas, dan `max_depth == 2` menghasilkan satu
/// pemecahan diikuti daun. Menyamakan satuannya penting karena batas kedalaman
/// yang meleset satu tingkat adalah cara diam-diam sebuah pohon menjadi lebih
/// besar daripada yang diminta.
pub fn build_id3(
    x: &[Vec<String>],
    y: &[String],
    attribute_names: &[String],
    max_depth: usize,
) -> Result<TreeNode, MlError> {
    if x.is_empty() {
        return Err(MlError::EmptyDataset);
    }
    if x.len() != y.len() {
        return Err(MlError::LengthMismatch {
            features: x.len(),
            labels: y.len(),
        });
    }
    let width = x[0].len();
    for row in x {
        if row.len() != width {
            return Err(MlError::RaggedRows {
                expected: width,
                got: row.len(),
            });
        }
    }
    let available: Vec<usize> = (0..width).collect();
    Ok(id3(x, y, &available, attribute_names, max_depth))
}

fn id3(
    x: &[Vec<String>],
    y: &[String],
    available: &[usize],
    names: &[String],
    depth_left: usize,
) -> TreeNode {
    let purity = {
        let m = majority(y);
        y.iter().filter(|l| **l == m).count() as f64 / y.len().max(1) as f64
    };

    // Berhenti bila sudah murni, kehabisan atribut, atau mencapai batas dalam.
    // Batasnya diperiksa pada satu, bukan nol, karena daun itu sendiri sudah
    // menghabiskan satu tingkat kedalaman.
    if purity >= 1.0 || available.is_empty() || depth_left <= 1 {
        return TreeNode::Leaf {
            label: majority(y),
            samples: y.len(),
            purity,
        };
    }

    let best = available
        .iter()
        .map(|a| {
            let values: Vec<String> = x.iter().map(|r| r[*a].clone()).collect();
            (*a, information_gain(&values, y))
        })
        // Seri perolehan diputus indeks terkecil agar pohonnya deterministik.
        .fold(None::<(usize, f64)>, |best, (a, g)| match best {
            Some((ba, bg)) if bg >= g => Some((ba, bg)),
            _ => Some((a, g)),
        });

    let (attribute, gain) = match best {
        Some(v) => v,
        None => {
            return TreeNode::Leaf {
                label: majority(y),
                samples: y.len(),
                purity,
            }
        }
    };

    // Perolehan nol berarti tidak ada atribut yang memisahkan apa pun;
    // memecah lebih jauh hanya memperbesar pohon tanpa menambah ketepatan.
    if gain <= 1e-12 {
        return TreeNode::Leaf {
            label: majority(y),
            samples: y.len(),
            purity,
        };
    }

    let mut groups: BTreeMap<String, (Vec<Vec<String>>, Vec<String>)> = BTreeMap::new();
    for (row, label) in x.iter().zip(y) {
        let entry = groups.entry(row[attribute].clone()).or_default();
        entry.0.push(row.clone());
        entry.1.push(label.clone());
    }

    let remaining: Vec<usize> = available
        .iter()
        .copied()
        .filter(|a| *a != attribute)
        .collect();
    let children: BTreeMap<String, TreeNode> = groups
        .into_iter()
        .map(|(value, (gx, gy))| (value, id3(&gx, &gy, &remaining, names, depth_left - 1)))
        .collect();

    TreeNode::Branch {
        attribute,
        attribute_name: names
            .get(attribute)
            .cloned()
            .unwrap_or_else(|| format!("atribut {attribute}")),
        gain,
        children,
        fallback: majority(y),
    }
}

// ---------------------------------------------------------------------------
// Regresi
// ---------------------------------------------------------------------------

/// Model regresi linear satu peubah.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LinearRegression {
    /// Titik potong sumbu tegak.
    pub intercept: f64,
    /// Kemiringan garis.
    pub slope: f64,
    /// Koefisien determinasi, `1.0` berarti garisnya melewati semua titik.
    pub r_squared: f64,
}

/// Mencocokkan garis lurus dengan kuadrat terkecil.
///
/// Memakai bentuk tertutup, bukan penurunan gradien: untuk satu peubah,
/// jawabannya bisa dihitung langsung dan pasti optimal.
pub fn fit_linear(x: &[f64], y: &[f64]) -> Result<LinearRegression, MlError> {
    if x.is_empty() {
        return Err(MlError::EmptyDataset);
    }
    if x.len() != y.len() {
        return Err(MlError::LengthMismatch {
            features: x.len(),
            labels: y.len(),
        });
    }
    for (i, v) in x.iter().chain(y).enumerate() {
        if !v.is_finite() {
            return Err(MlError::NonFiniteValue {
                row: i % x.len(),
                column: 0,
            });
        }
    }

    let n = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;
    let sxx: f64 = x.iter().map(|v| (v - mean_x).powi(2)).sum();
    let sxy: f64 = x
        .iter()
        .zip(y)
        .map(|(a, b)| (a - mean_x) * (b - mean_y))
        .sum();

    // Seluruh x sama berarti tidak ada garis yang bisa dicocokkan; kemiringan
    // nol dilaporkan alih-alih menghasilkan nilai bukan bilangan.
    let slope = if sxx.abs() < 1e-12 { 0.0 } else { sxy / sxx };
    let intercept = mean_y - slope * mean_x;

    let ss_total: f64 = y.iter().map(|v| (v - mean_y).powi(2)).sum();
    let ss_residual: f64 = x
        .iter()
        .zip(y)
        .map(|(a, b)| {
            let pred = intercept + slope * a;
            (b - pred).powi(2)
        })
        .sum();
    let r_squared = if ss_total.abs() < 1e-12 {
        // Seluruh y sama: modelnya sempurna bila galatnya nol.
        if ss_residual.abs() < 1e-12 {
            1.0
        } else {
            0.0
        }
    } else {
        1.0 - ss_residual / ss_total
    };

    Ok(LinearRegression {
        intercept,
        slope,
        r_squared,
    })
}

impl LinearRegression {
    /// Nilai ramalan pada sebuah titik.
    pub fn predict(&self, x: f64) -> f64 {
        self.intercept + self.slope * x
    }
}

/// Model regresi logistik banyak peubah.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogisticRegression {
    /// Bobot tiap peubah.
    pub weights: Vec<f64>,
    /// Bias.
    pub bias: f64,
    /// Galat entropi silang pada tiap epoch.
    pub loss_history: Vec<f64>,
}

/// Sigmoid logistik yang tahan terhadap masukan besar.
fn sigmoid(z: f64) -> f64 {
    if z >= 0.0 {
        1.0 / (1.0 + (-z).exp())
    } else {
        let e = z.exp();
        e / (1.0 + e)
    }
}

/// Melatih regresi logistik dengan penurunan gradien.
pub fn fit_logistic(
    x: &[Vec<f64>],
    y: &[f64],
    learning_rate: f64,
    epochs: usize,
) -> Result<LogisticRegression, MlError> {
    let width = validate_matrix(x)?;
    if x.len() != y.len() {
        return Err(MlError::LengthMismatch {
            features: x.len(),
            labels: y.len(),
        });
    }
    if !learning_rate.is_finite() || learning_rate <= 0.0 {
        return Err(MlError::BadParameter {
            name: "learning_rate".into(),
            value: learning_rate,
        });
    }

    let mut weights = vec![0.0; width];
    let mut bias = 0.0;
    let mut loss_history = Vec::with_capacity(epochs);
    let n = x.len() as f64;

    for _ in 0..epochs {
        let mut grad_w = vec![0.0; width];
        let mut grad_b = 0.0;
        let mut loss = 0.0;

        for (row, target) in x.iter().zip(y) {
            let z = row.iter().zip(&weights).map(|(a, w)| a * w).sum::<f64>() + bias;
            let p = sigmoid(z);
            let error = p - target;
            for (g, v) in grad_w.iter_mut().zip(row) {
                *g += error * v;
            }
            grad_b += error;
            // Peluang dijepit sebelum dilogaritmakan; ln(0) menghasilkan tak
            // hingga dan seluruh riwayat galat menjadi tak terbaca.
            let clamped = p.clamp(1e-12, 1.0 - 1e-12);
            loss -= target * clamped.ln() + (1.0 - target) * (1.0 - clamped).ln();
        }

        for (w, g) in weights.iter_mut().zip(&grad_w) {
            *w -= learning_rate * g / n;
        }
        bias -= learning_rate * grad_b / n;
        loss_history.push(loss / n);
    }

    Ok(LogisticRegression {
        weights,
        bias,
        loss_history,
    })
}

impl LogisticRegression {
    /// Peluang kelas positif untuk sebuah baris.
    pub fn probability(&self, row: &[f64]) -> Result<f64, MlError> {
        if row.len() != self.weights.len() {
            return Err(MlError::RaggedRows {
                expected: self.weights.len(),
                got: row.len(),
            });
        }
        let z = row
            .iter()
            .zip(&self.weights)
            .map(|(a, w)| a * w)
            .sum::<f64>()
            + self.bias;
        Ok(sigmoid(z))
    }

    /// Kelas terpilih dengan ambang tertentu.
    pub fn predict(&self, row: &[f64], threshold: f64) -> Result<bool, MlError> {
        Ok(self.probability(row)? >= threshold)
    }
}

// ---------------------------------------------------------------------------
// Evaluasi
// ---------------------------------------------------------------------------

/// Matriks konfusi beserta ukuran turunannya.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evaluation {
    /// Daftar kelas, terurut.
    pub labels: Vec<String>,
    /// Matriks konfusi, `[sebenarnya][diramalkan]`.
    pub matrix: Vec<Vec<usize>>,
    /// Ketepatan keseluruhan.
    pub accuracy: f64,
    /// Presisi tiap kelas.
    pub precision: BTreeMap<String, f64>,
    /// Kepekaan tiap kelas.
    pub recall: BTreeMap<String, f64>,
    /// Skor F1 tiap kelas.
    pub f1: BTreeMap<String, f64>,
    /// Rerata F1 tanpa pembobotan jumlah anggota kelas.
    pub macro_f1: f64,
    /// Ketepatan yang dicapai dengan selalu menebak kelas terbanyak.
    ///
    /// Angka pembanding yang wajib dilihat berdampingan dengan ketepatan.
    /// Model yang tidak melampaui angka ini tidak mempelajari apa pun.
    pub baseline_accuracy: f64,
}

/// Menghitung matriks konfusi dan ukuran turunannya.
pub fn evaluate(actual: &[String], predicted: &[String]) -> Result<Evaluation, MlError> {
    if actual.is_empty() {
        return Err(MlError::EmptyDataset);
    }
    if actual.len() != predicted.len() {
        return Err(MlError::LengthMismatch {
            features: actual.len(),
            labels: predicted.len(),
        });
    }

    let mut labels: Vec<String> = actual
        .iter()
        .chain(predicted)
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    labels.sort();
    let index: BTreeMap<&str, usize> = labels
        .iter()
        .enumerate()
        .map(|(i, l)| (l.as_str(), i))
        .collect();

    let mut matrix = vec![vec![0usize; labels.len()]; labels.len()];
    for (a, p) in actual.iter().zip(predicted) {
        matrix[index[a.as_str()]][index[p.as_str()]] += 1;
    }

    let total = actual.len() as f64;
    let correct: usize = (0..labels.len()).map(|i| matrix[i][i]).sum();
    let accuracy = correct as f64 / total;

    let mut precision = BTreeMap::new();
    let mut recall = BTreeMap::new();
    let mut f1 = BTreeMap::new();
    for (i, label) in labels.iter().enumerate() {
        let tp = matrix[i][i] as f64;
        let predicted_positive: f64 = (0..labels.len()).map(|r| matrix[r][i] as f64).sum();
        let actual_positive: f64 = matrix[i].iter().map(|v| *v as f64).sum();
        // Kelas yang tidak pernah diramalkan punya presisi nol, bukan tak
        // terdefinisi; melaporkannya sebagai nol jauh lebih jujur daripada
        // menyembunyikannya.
        let p = if predicted_positive > 0.0 {
            tp / predicted_positive
        } else {
            0.0
        };
        let r = if actual_positive > 0.0 {
            tp / actual_positive
        } else {
            0.0
        };
        let f = if p + r > 0.0 {
            2.0 * p * r / (p + r)
        } else {
            0.0
        };
        precision.insert(label.clone(), p);
        recall.insert(label.clone(), r);
        f1.insert(label.clone(), f);
    }

    let macro_f1 = f1.values().sum::<f64>() / labels.len() as f64;

    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for a in actual {
        *counts.entry(a.as_str()).or_insert(0) += 1;
    }
    let baseline_accuracy = counts.values().copied().max().unwrap_or(0) as f64 / total;

    Ok(Evaluation {
        labels,
        matrix,
        accuracy,
        precision,
        recall,
        f1,
        macro_f1,
        baseline_accuracy,
    })
}

/// Membagi data menjadi bagian latih dan uji.
///
/// Urutannya diacak lebih dulu dengan benih eksplisit; membagi data terurut
/// tanpa diacak sering menempatkan seluruh satu kelas di satu sisi saja.
pub fn train_test_split(
    n: usize,
    test_ratio: f64,
    seed: u64,
) -> Result<(Vec<usize>, Vec<usize>), MlError> {
    if n == 0 {
        return Err(MlError::EmptyDataset);
    }
    if !(0.0..1.0).contains(&test_ratio) {
        return Err(MlError::BadParameter {
            name: "test_ratio".into(),
            value: test_ratio,
        });
    }
    let mut order: Vec<usize> = (0..n).collect();
    SplitMix64::new(seed).shuffle(&mut order);
    let test_size = ((n as f64) * test_ratio).round() as usize;
    let test = order[..test_size].to_vec();
    let train = order[test_size..].to_vec();
    Ok((train, test))
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

    fn s(v: &str) -> String {
        v.to_string()
    }

    fn labels(list: &[&str]) -> Vec<String> {
        list.iter().map(|v| s(v)).collect()
    }

    // -------------------------------------------------------------- jarak

    #[test]
    fn jarak_nilai_yang_dikenal() {
        let a = [0.0, 0.0];
        let b = [3.0, 4.0];
        close(Distance::Euclidean.between(&a, &b), 5.0);
        close(Distance::Manhattan.between(&a, &b), 7.0);
        close(Distance::Chebyshev.between(&a, &b), 4.0);
    }

    #[test]
    fn jarak_ke_diri_sendiri_nol_dan_setangkup() {
        let a = [1.0, -2.0, 3.5];
        let b = [4.0, 0.5, -1.0];
        for d in [
            Distance::Euclidean,
            Distance::Manhattan,
            Distance::Chebyshev,
        ] {
            close(d.between(&a, &a), 0.0);
            close(d.between(&a, &b), d.between(&b, &a));
            assert!(!d.short_name().is_empty());
        }
    }

    #[test]
    fn jarak_memenuhi_ketaksamaan_segitiga() {
        let a = [0.0, 0.0];
        let b = [1.0, 2.0];
        let c = [3.0, 1.0];
        for d in [
            Distance::Euclidean,
            Distance::Manhattan,
            Distance::Chebyshev,
        ] {
            assert!(
                d.between(&a, &c) <= d.between(&a, &b) + d.between(&b, &c) + 1e-12,
                "{} melanggar ketaksamaan segitiga",
                d.short_name()
            );
        }
    }

    // ----------------------------------------------------------- validasi

    #[test]
    fn matriks_tak_sah_ditolak() {
        assert_eq!(validate_matrix(&[]), Err(MlError::EmptyDataset));
        assert_eq!(validate_matrix(&[vec![]]), Err(MlError::EmptyDataset));
        assert_eq!(
            validate_matrix(&[vec![1.0, 2.0], vec![3.0]]),
            Err(MlError::RaggedRows {
                expected: 2,
                got: 1
            })
        );
        assert_eq!(
            validate_matrix(&[vec![1.0, f64::NAN]]),
            Err(MlError::NonFiniteValue { row: 0, column: 1 })
        );
    }

    // --------------------------------------------------------- penskalaan

    #[test]
    fn ringkasan_kolom() {
        let x = vec![vec![1.0, 10.0], vec![3.0, 20.0], vec![5.0, 30.0]];
        let s = column_scales(&x).unwrap();
        assert_eq!(s.len(), 2);
        close(s[0].min, 1.0);
        close(s[0].max, 5.0);
        close(s[0].mean, 3.0);
        close(s[0].std_dev, 2.0);
    }

    #[test]
    fn penskalaan_min_maks() {
        let x = vec![vec![1.0], vec![3.0], vec![5.0]];
        let scaled = min_max_scale(&x).unwrap();
        close(scaled[0][0], 0.0);
        close(scaled[1][0], 0.5);
        close(scaled[2][0], 1.0);
    }

    #[test]
    fn penskalaan_kolom_tetap_tidak_membagi_nol() {
        let x = vec![vec![7.0, 1.0], vec![7.0, 2.0], vec![7.0, 3.0]];
        let scaled = min_max_scale(&x).unwrap();
        assert!(scaled.iter().all(|r| r[0] == 0.0));
        assert!(scaled.iter().all(|r| r.iter().all(|v| v.is_finite())));

        let std = standardise(&x).unwrap();
        assert!(std.iter().all(|r| r.iter().all(|v| v.is_finite())));
        assert!(std.iter().all(|r| r[0] == 0.0));
    }

    #[test]
    fn pembakuan_menghasilkan_rerata_nol() {
        let x = vec![vec![1.0], vec![2.0], vec![3.0], vec![4.0]];
        let std = standardise(&x).unwrap();
        let mean: f64 = std.iter().map(|r| r[0]).sum::<f64>() / 4.0;
        near(mean, 0.0, 1e-12);
    }

    // ---------------------------------------------------------------- KNN

    fn data_knn() -> (Vec<Vec<f64>>, Vec<String>) {
        (
            vec![
                vec![1.0, 1.0],
                vec![1.2, 0.9],
                vec![0.8, 1.1],
                vec![8.0, 8.0],
                vec![8.2, 7.9],
                vec![7.8, 8.1],
            ],
            labels(&["A", "A", "A", "B", "B", "B"]),
        )
    }

    #[test]
    fn knn_mengklasifikasi_dua_gugus() {
        let (x, y) = data_knn();
        let knn = Knn::new(x, y, 3, Distance::Euclidean, false).unwrap();
        assert_eq!(knn.len(), 6);
        assert!(!knn.is_empty());
        assert_eq!(knn.predict(&[1.0, 1.0]).unwrap().label, "A");
        assert_eq!(knn.predict(&[8.0, 8.0]).unwrap().label, "B");
    }

    #[test]
    fn knn_melaporkan_tetangganya() {
        let (x, y) = data_knn();
        let knn = Knn::new(x, y, 3, Distance::Euclidean, false).unwrap();
        let hasil = knn.predict(&[1.0, 1.0]).unwrap();
        assert_eq!(hasil.neighbours.len(), 3);
        // Tetangga harus terurut dari yang terdekat.
        for w in hasil.neighbours.windows(2) {
            assert!(w[0].distance <= w[1].distance);
        }
        close(hasil.neighbours[0].distance, 0.0);
        assert_eq!(hasil.votes.len(), 1, "ketiga tetangga sekelas");
    }

    #[test]
    fn knn_berbobot_memihak_yang_lebih_dekat() {
        // Titik yang sangat dekat ke satu A, tetapi dikepung banyak B agak jauh.
        let x = vec![
            vec![0.0, 0.0],
            vec![5.0, 0.0],
            vec![0.0, 5.0],
            vec![5.0, 5.0],
        ];
        let y = labels(&["A", "B", "B", "B"]);
        let biasa = Knn::new(x.clone(), y.clone(), 4, Distance::Euclidean, false).unwrap();
        let berbobot = Knn::new(x, y, 4, Distance::Euclidean, true).unwrap();
        assert_eq!(biasa.predict(&[0.01, 0.01]).unwrap().label, "B");
        assert_eq!(berbobot.predict(&[0.01, 0.01]).unwrap().label, "A");
    }

    #[test]
    fn knn_jarak_nol_tidak_membagi_nol() {
        let (x, y) = data_knn();
        let knn = Knn::new(x.clone(), y, 1, Distance::Euclidean, true).unwrap();
        let hasil = knn.predict(&x[0]).unwrap();
        assert!(hasil.votes.values().all(|v| v.is_finite()));
        assert_eq!(hasil.label, "A");
    }

    #[test]
    fn knn_menolak_parameter_tak_sah() {
        let (x, y) = data_knn();
        assert!(matches!(
            Knn::new(x.clone(), y.clone(), 0, Distance::Euclidean, false),
            Err(MlError::BadParameter { .. })
        ));
        assert!(matches!(
            Knn::new(x.clone(), y.clone(), 99, Distance::Euclidean, false),
            Err(MlError::BadParameter { .. })
        ));
        assert!(matches!(
            Knn::new(x.clone(), labels(&["A"]), 3, Distance::Euclidean, false),
            Err(MlError::LengthMismatch { .. })
        ));
        let knn = Knn::new(x, y, 3, Distance::Euclidean, false).unwrap();
        assert!(matches!(
            knn.predict(&[1.0]),
            Err(MlError::RaggedRows { .. })
        ));
    }

    // ------------------------------------------------------------ K-Means

    fn data_kmeans() -> Vec<Vec<f64>> {
        vec![
            vec![1.0, 1.0],
            vec![1.5, 1.2],
            vec![0.8, 0.9],
            vec![9.0, 9.0],
            vec![9.4, 8.8],
            vec![8.7, 9.2],
        ]
    }

    #[test]
    fn kmeans_menemukan_dua_gugus() {
        let x = data_kmeans();
        let hasil = kmeans(&x, 2, Distance::Euclidean, 100, 42).unwrap();
        assert_eq!(hasil.centroids.len(), 2);
        assert_eq!(hasil.assignments.len(), 6);
        assert!(hasil.converged, "seharusnya mencapai keadaan tetap");
        // Tiga titik pertama harus sekelompok, tiga terakhir juga.
        assert_eq!(hasil.assignments[0], hasil.assignments[1]);
        assert_eq!(hasil.assignments[0], hasil.assignments[2]);
        assert_eq!(hasil.assignments[3], hasil.assignments[4]);
        assert_ne!(hasil.assignments[0], hasil.assignments[3]);
    }

    #[test]
    fn kmeans_deterministik_untuk_benih_sama() {
        let x = data_kmeans();
        let a = kmeans(&x, 2, Distance::Euclidean, 100, 7).unwrap();
        let b = kmeans(&x, 2, Distance::Euclidean, 100, 7).unwrap();
        assert_eq!(a.assignments, b.assignments);
        close(a.inertia, b.inertia);
    }

    #[test]
    fn kmeans_plus_plus_lebih_baik_daripada_pusat_berdekatan() {
        // Dengan berbagai benih, K-Means++ harus konsisten menemukan
        // pengelompokan yang benar pada data yang jelas terpisah.
        let x = data_kmeans();
        for seed in 0..20u64 {
            let hasil = kmeans(&x, 2, Distance::Euclidean, 100, seed).unwrap();
            assert_ne!(
                hasil.assignments[0], hasil.assignments[3],
                "benih {seed} menyatukan dua gugus yang jelas terpisah"
            );
        }
    }

    #[test]
    fn kmeans_inertia_menurun_saat_kelompok_bertambah() {
        let x = data_kmeans();
        let dua = kmeans(&x, 2, Distance::Euclidean, 100, 1).unwrap().inertia;
        let tiga = kmeans(&x, 3, Distance::Euclidean, 100, 1).unwrap().inertia;
        assert!(tiga <= dua + 1e-9, "inertia naik: {dua} lalu {tiga}");
    }

    #[test]
    fn kmeans_tidak_meninggalkan_kelompok_kosong() {
        let x = data_kmeans();
        for k in 1..=6usize {
            let hasil = kmeans(&x, k, Distance::Euclidean, 100, 3).unwrap();
            assert_eq!(hasil.centroids.len(), k);
            let terpakai: std::collections::BTreeSet<usize> =
                hasil.assignments.iter().copied().collect();
            assert_eq!(terpakai.len(), k, "ada kelompok kosong pada k={k}");
        }
    }

    #[test]
    fn kmeans_menolak_parameter_tak_sah() {
        let x = data_kmeans();
        assert!(matches!(
            kmeans(&x, 0, Distance::Euclidean, 100, 1),
            Err(MlError::BadParameter { .. })
        ));
        assert_eq!(
            kmeans(&x, 10, Distance::Euclidean, 100, 1),
            Err(MlError::TooManyClusters { k: 10, points: 6 })
        );
        assert_eq!(
            kmeans(&[], 2, Distance::Euclidean, 100, 1),
            Err(MlError::EmptyDataset)
        );
    }

    // ---------------------------------------------------------------- ID3

    #[test]
    fn entropi_nilai_yang_dikenal() {
        close(entropy(&labels(&["A", "A", "A"])), 0.0);
        close(entropy(&labels(&["A", "B"])), 1.0);
        close(entropy(&labels(&["A", "A", "B", "B"])), 1.0);
        close(entropy(&[]), 0.0);
        // Empat kelas seimbang = 2 bit.
        close(entropy(&labels(&["A", "B", "C", "D"])), 2.0);
    }

    #[test]
    fn entropi_dataset_tenis_klasik() {
        // Sembilan "Ya" dan lima "Tidak" menghasilkan 0,940 bit.
        let y = labels(&[
            "Tidak", "Tidak", "Ya", "Ya", "Ya", "Tidak", "Ya", "Tidak", "Ya", "Ya", "Ya", "Ya",
            "Ya", "Tidak",
        ]);
        near(entropy(&y), 0.940, 0.001);
    }

    #[test]
    fn gini_nilai_yang_dikenal() {
        close(gini(&labels(&["A", "A"])), 0.0);
        close(gini(&labels(&["A", "B"])), 0.5);
        close(gini(&[]), 0.0);
        // Gini selalu tidak melebihi entropi untuk dua kelas.
        for komposisi in [
            labels(&["A", "B"]),
            labels(&["A", "A", "B"]),
            labels(&["A", "A", "A", "B"]),
        ] {
            assert!(gini(&komposisi) <= entropy(&komposisi) + 1e-12);
        }
    }

    #[test]
    fn perolehan_informasi_dataset_tenis() {
        // Atribut cuaca pada dataset tenis klasik memberi perolehan 0,247 bit.
        let cuaca = labels(&[
            "Cerah", "Cerah", "Mendung", "Hujan", "Hujan", "Hujan", "Mendung", "Cerah", "Cerah",
            "Hujan", "Cerah", "Mendung", "Mendung", "Hujan",
        ]);
        let y = labels(&[
            "Tidak", "Tidak", "Ya", "Ya", "Ya", "Tidak", "Ya", "Tidak", "Ya", "Ya", "Ya", "Ya",
            "Ya", "Tidak",
        ]);
        near(information_gain(&cuaca, &y), 0.247, 0.001);
    }

    #[test]
    fn perolehan_informasi_nol_untuk_atribut_tak_berguna() {
        let sama = labels(&["X", "X", "X", "X"]);
        let y = labels(&["A", "B", "A", "B"]);
        close(information_gain(&sama, &y), 0.0);
        // Panjang tidak sepadan menghasilkan nol, bukan panik.
        close(information_gain(&labels(&["X"]), &y), 0.0);
    }

    #[test]
    fn perolehan_informasi_maksimum_untuk_atribut_sempurna() {
        let sempurna = labels(&["p", "q", "p", "q"]);
        let y = labels(&["A", "B", "A", "B"]);
        close(information_gain(&sempurna, &y), 1.0);
    }

    fn data_tenis() -> (Vec<Vec<String>>, Vec<String>, Vec<String>) {
        let baris = [
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
        let x = baris
            .iter()
            .map(|(f, _)| f.iter().map(|v| s(v)).collect())
            .collect();
        let y = baris.iter().map(|(_, l)| s(l)).collect();
        let nama = labels(&["Cuaca", "Suhu", "Kelembapan", "Angin"]);
        (x, y, nama)
    }

    #[test]
    fn id3_memilih_atribut_dengan_perolehan_tertinggi() {
        let (x, y, nama) = data_tenis();
        let pohon = build_id3(&x, &y, &nama, 10).unwrap();
        match &pohon {
            TreeNode::Branch {
                attribute_name,
                gain,
                ..
            } => {
                // Cuaca memberi perolehan tertinggi pada dataset ini.
                assert_eq!(attribute_name, "Cuaca");
                near(*gain, 0.247, 0.001);
            }
            other => panic!("akar seharusnya cabang, bukan {other:?}"),
        }
    }

    #[test]
    fn id3_mengklasifikasi_data_latihnya_dengan_sempurna() {
        let (x, y, nama) = data_tenis();
        let pohon = build_id3(&x, &y, &nama, 10).unwrap();
        for (row, target) in x.iter().zip(&y) {
            assert_eq!(&pohon.predict(row), target, "salah pada baris {row:?}");
        }
    }

    #[test]
    fn id3_membentuk_daun_murni() {
        let (x, y, nama) = data_tenis();
        let pohon = build_id3(&x, &y, &nama, 10).unwrap();
        assert!(pohon.depth() >= 2);
        assert!(pohon.leaf_count() >= 3);

        // Cabang "Mendung" seluruhnya berlabel Ya, jadi harus langsung daun.
        if let TreeNode::Branch { children, .. } = &pohon {
            match children.get("Mendung") {
                Some(TreeNode::Leaf { label, purity, .. }) => {
                    assert_eq!(label, "Ya");
                    close(*purity, 1.0);
                }
                other => panic!("Mendung seharusnya daun murni, bukan {other:?}"),
            }
        }
    }

    #[test]
    fn id3_menghormati_batas_kedalaman() {
        // Batas kedalaman memakai satuan yang sama dengan TreeNode::depth,
        // sehingga angka yang diminta dan angka yang dihasilkan bisa langsung
        // dibandingkan tanpa terjemahan diam-diam.
        let (x, y, nama) = data_tenis();
        for batas in 1..=6usize {
            let pohon = build_id3(&x, &y, &nama, batas).unwrap();
            assert!(
                pohon.depth() <= batas,
                "batas {batas} menghasilkan kedalaman {}",
                pohon.depth()
            );
        }
        assert_eq!(
            build_id3(&x, &y, &nama, 1).unwrap().depth(),
            1,
            "batas 1 seharusnya menghasilkan satu daun"
        );
        assert!(matches!(
            build_id3(&x, &y, &nama, 1).unwrap(),
            TreeNode::Leaf { .. }
        ));
        // Batas nol tidak boleh membuat pohon lebih besar daripada batas satu.
        assert_eq!(build_id3(&x, &y, &nama, 0).unwrap().depth(), 1);
    }

    #[test]
    fn id3_menjawab_nilai_yang_belum_pernah_dilihat() {
        let (x, y, nama) = data_tenis();
        let pohon = build_id3(&x, &y, &nama, 10).unwrap();
        // "Bersalju" tidak ada di data latih; pohon harus tetap menjawab.
        let jawaban = pohon.predict(&labels(&["Bersalju", "Panas", "Tinggi", "Lemah"]));
        assert!(jawaban == "Ya" || jawaban == "Tidak");
    }

    #[test]
    fn id3_menolak_data_tak_sah() {
        let (x, y, nama) = data_tenis();
        assert_eq!(build_id3(&[], &y, &nama, 5), Err(MlError::EmptyDataset));
        assert!(matches!(
            build_id3(&x, &labels(&["Ya"]), &nama, 5),
            Err(MlError::LengthMismatch { .. })
        ));
        let mut rusak = x.clone();
        rusak[0].pop();
        assert!(matches!(
            build_id3(&rusak, &y, &nama, 5),
            Err(MlError::RaggedRows { .. })
        ));
    }

    #[test]
    fn id3_deterministik() {
        let (x, y, nama) = data_tenis();
        let a = build_id3(&x, &y, &nama, 10).unwrap();
        let b = build_id3(&x, &y, &nama, 10).unwrap();
        assert_eq!(a, b);
    }

    // ------------------------------------------------------------ regresi

    #[test]
    fn regresi_linear_garis_sempurna() {
        let x = vec![1.0, 2.0, 3.0, 4.0];
        let y = vec![3.0, 5.0, 7.0, 9.0];
        let model = fit_linear(&x, &y).unwrap();
        close(model.slope, 2.0);
        close(model.intercept, 1.0);
        close(model.r_squared, 1.0);
        close(model.predict(5.0), 11.0);
    }

    #[test]
    fn regresi_linear_dengan_derau() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.1, 3.9, 6.2, 7.8, 10.1];
        let model = fit_linear(&x, &y).unwrap();
        near(model.slope, 2.0, 0.1);
        assert!(model.r_squared > 0.99);
    }

    #[test]
    fn regresi_linear_x_tetap_tidak_menghasilkan_nan() {
        let x = vec![3.0, 3.0, 3.0];
        let y = vec![1.0, 2.0, 3.0];
        let model = fit_linear(&x, &y).unwrap();
        assert!(model.slope.is_finite());
        assert!(model.intercept.is_finite());
        assert!(model.r_squared.is_finite());
        close(model.slope, 0.0);
    }

    #[test]
    fn regresi_linear_y_tetap() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![5.0, 5.0, 5.0];
        let model = fit_linear(&x, &y).unwrap();
        close(model.slope, 0.0);
        close(model.intercept, 5.0);
        close(model.r_squared, 1.0);
    }

    #[test]
    fn regresi_linear_menolak_data_tak_sah() {
        assert_eq!(fit_linear(&[], &[]), Err(MlError::EmptyDataset));
        assert!(matches!(
            fit_linear(&[1.0, 2.0], &[1.0]),
            Err(MlError::LengthMismatch { .. })
        ));
        assert!(matches!(
            fit_linear(&[1.0, f64::NAN], &[1.0, 2.0]),
            Err(MlError::NonFiniteValue { .. })
        ));
    }

    #[test]
    fn regresi_logistik_memisahkan_dua_kelas() {
        let x: Vec<Vec<f64>> = (0..40).map(|i| vec![i as f64 / 10.0 - 2.0]).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|r| if r[0] > 0.0 { 1.0 } else { 0.0 })
            .collect();
        let model = fit_logistic(&x, &y, 0.5, 2_000).unwrap();

        assert!(model.weights[0] > 0.0, "bobot harus positif");
        assert!(model.probability(&[2.0]).unwrap() > 0.9);
        assert!(model.probability(&[-2.0]).unwrap() < 0.1);
        assert!(model.predict(&[1.0], 0.5).unwrap());
        assert!(!model.predict(&[-1.0], 0.5).unwrap());
    }

    #[test]
    fn regresi_logistik_galat_menurun() {
        let x: Vec<Vec<f64>> = (0..30).map(|i| vec![i as f64 / 10.0 - 1.5]).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|r| if r[0] > 0.0 { 1.0 } else { 0.0 })
            .collect();
        let model = fit_logistic(&x, &y, 0.5, 500).unwrap();
        assert_eq!(model.loss_history.len(), 500);
        assert!(model.loss_history.iter().all(|v| v.is_finite()));
        assert!(
            model.loss_history.last().unwrap() < &model.loss_history[0],
            "galat tidak menurun"
        );
    }

    #[test]
    fn regresi_logistik_menolak_parameter_tak_sah() {
        let x = vec![vec![1.0], vec![2.0]];
        let y = vec![0.0, 1.0];
        assert!(matches!(
            fit_logistic(&x, &y, 0.0, 10),
            Err(MlError::BadParameter { .. })
        ));
        assert!(matches!(
            fit_logistic(&x, &y, -1.0, 10),
            Err(MlError::BadParameter { .. })
        ));
        assert!(matches!(
            fit_logistic(&x, &[0.0], 0.1, 10),
            Err(MlError::LengthMismatch { .. })
        ));
        let model = fit_logistic(&x, &y, 0.1, 10).unwrap();
        assert!(matches!(
            model.probability(&[1.0, 2.0]),
            Err(MlError::RaggedRows { .. })
        ));
    }

    // ----------------------------------------------------------- evaluasi

    #[test]
    fn matriks_konfusi_sempurna() {
        let a = labels(&["A", "A", "B", "B"]);
        let e = evaluate(&a, &a).unwrap();
        close(e.accuracy, 1.0);
        close(e.macro_f1, 1.0);
        assert_eq!(e.labels, labels(&["A", "B"]));
        assert_eq!(e.matrix, vec![vec![2, 0], vec![0, 2]]);
    }

    #[test]
    fn matriks_konfusi_dengan_kesalahan() {
        let sebenarnya = labels(&["A", "A", "A", "B"]);
        let diramalkan = labels(&["A", "A", "B", "B"]);
        let e = evaluate(&sebenarnya, &diramalkan).unwrap();
        close(e.accuracy, 0.75);
        // A: benar 2 dari 3 sebenarnya; presisi 2/2, kepekaan 2/3.
        close(e.precision["A"], 1.0);
        near(e.recall["A"], 2.0 / 3.0, 1e-12);
        // B: diramalkan 2 kali, benar 1.
        close(e.precision["B"], 0.5);
        close(e.recall["B"], 1.0);
    }

    #[test]
    fn ketepatan_dasar_membongkar_model_yang_tidak_belajar() {
        // Ini alasan ketepatan saja tidak boleh dipercaya: pada data yang
        // sangat timpang, menebak kelas terbanyak terus-menerus sudah
        // menghasilkan ketepatan tinggi tanpa mempelajari apa pun.
        let mut sebenarnya = vec![s("A"); 99];
        sebenarnya.push(s("B"));
        let selalu_a = vec![s("A"); 100];
        let e = evaluate(&sebenarnya, &selalu_a).unwrap();
        close(e.accuracy, 0.99);
        close(e.baseline_accuracy, 0.99);
        assert!(
            e.accuracy <= e.baseline_accuracy + 1e-12,
            "model ini tidak melampaui tebakan terbanyak"
        );
        // F1 makro membongkarnya: kelas B tidak pernah tertangkap.
        close(e.recall["B"], 0.0);
        assert!(e.macro_f1 < 0.55, "F1 makro {} terlalu tinggi", e.macro_f1);
    }

    #[test]
    fn kelas_yang_tak_pernah_diramalkan_tidak_menghasilkan_nan() {
        let sebenarnya = labels(&["A", "B", "C"]);
        let diramalkan = labels(&["A", "A", "A"]);
        let e = evaluate(&sebenarnya, &diramalkan).unwrap();
        for label in &e.labels {
            assert!(e.precision[label].is_finite());
            assert!(e.recall[label].is_finite());
            assert!(e.f1[label].is_finite());
        }
        close(e.precision["B"], 0.0);
        assert!(e.macro_f1.is_finite());
    }

    #[test]
    fn evaluasi_menolak_masukan_tak_sah() {
        assert_eq!(evaluate(&[], &[]), Err(MlError::EmptyDataset));
        assert!(matches!(
            evaluate(&labels(&["A"]), &[]),
            Err(MlError::LengthMismatch { .. })
        ));
    }

    #[test]
    fn pembagian_latih_uji() {
        let (train, test) = train_test_split(100, 0.2, 7).unwrap();
        assert_eq!(test.len(), 20);
        assert_eq!(train.len(), 80);
        // Tidak boleh ada indeks yang muncul di kedua sisi.
        let mut semua: Vec<usize> = train.iter().chain(&test).copied().collect();
        semua.sort_unstable();
        assert_eq!(semua, (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn pembagian_latih_uji_deterministik_dan_teracak() {
        let (a, _) = train_test_split(50, 0.3, 5).unwrap();
        let (b, _) = train_test_split(50, 0.3, 5).unwrap();
        assert_eq!(a, b);
        let (c, _) = train_test_split(50, 0.3, 6).unwrap();
        assert_ne!(a, c);
        // Urutannya benar-benar teracak, bukan potongan berurut.
        assert_ne!(a, (0..a.len()).collect::<Vec<_>>());
    }

    #[test]
    fn pembagian_latih_uji_menolak_masukan_tak_sah() {
        assert_eq!(train_test_split(0, 0.2, 1), Err(MlError::EmptyDataset));
        assert!(matches!(
            train_test_split(10, 1.0, 1),
            Err(MlError::BadParameter { .. })
        ));
        assert!(matches!(
            train_test_split(10, -0.1, 1),
            Err(MlError::BadParameter { .. })
        ));
    }

    #[test]
    fn model_bisa_di_serialisasi() {
        let (x, y, nama) = data_tenis();
        let pohon = build_id3(&x, &y, &nama, 10).unwrap();
        let json = serde_json::to_string(&pohon).unwrap();
        assert_eq!(serde_json::from_str::<TreeNode>(&json).unwrap(), pohon);
        // Bentuk kawatnya bertanda seragam.
        assert!(json.contains(r#""kind":"branch""#), "{json}");

        let hasil = kmeans(&data_kmeans(), 2, Distance::Euclidean, 50, 1).unwrap();
        let kj = serde_json::to_string(&hasil).unwrap();
        assert_eq!(
            serde_json::from_str::<Clustering>(&kj).unwrap().assignments,
            hasil.assignments
        );
    }

    #[test]
    fn pesan_error_terbaca() {
        for e in [
            MlError::EmptyDataset,
            MlError::LengthMismatch {
                features: 1,
                labels: 2,
            },
            MlError::RaggedRows {
                expected: 2,
                got: 1,
            },
            MlError::BadParameter {
                name: "k".into(),
                value: 0.0,
            },
            MlError::NotTrained,
            MlError::TooManyClusters { k: 5, points: 2 },
            MlError::NonFiniteValue { row: 1, column: 2 },
        ] {
            assert!(!e.to_string().is_empty());
        }
    }
}
