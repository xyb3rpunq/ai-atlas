//! Sesi 8 — Teknik Pencarian dan Pelacakan.
//!
//! Pencarian buta (BFS, DFS, DLS, IDDFS, UCS) dan pencarian terbimbing
//! (*greedy best-first*, A\*), ditambah pencarian lokal (*hill climbing* dan
//! *simulated annealing*) di atas ruang keadaan yang sama.
//!
//! Semua algoritma mengembalikan jejak urutan simpul yang dibuka, bukan hanya
//! jalurnya. Perbedaan antaralgoritma justru terlihat di sana: dua algoritma
//! bisa menemukan jalur yang sama panjang sambil membuka jumlah simpul yang
//! berbeda jauh, dan itulah yang membuat satu lebih baik daripada yang lain.

use crate::rng::SplitMix64;
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

/// Kesalahan pada pencarian.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchError {
    /// Ukuran kisi nol atau melampaui batas wajar.
    BadGrid {
        /// Lebar yang diminta.
        width: usize,
        /// Tinggi yang diminta.
        height: usize,
    },
    /// Titik berada di luar kisi.
    OutOfBounds {
        /// Koordinat mendatar.
        x: usize,
        /// Koordinat tegak.
        y: usize,
    },
    /// Titik awal atau tujuan berdiri di atas dinding.
    BlockedEndpoint(&'static str),
    /// Panjang data dinding tidak sepadan dengan ukuran kisi.
    WallLengthMismatch {
        /// Panjang yang diharapkan.
        expected: usize,
        /// Panjang yang diterima.
        got: usize,
    },
}

impl core::fmt::Display for SearchError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SearchError::BadGrid { width, height } => {
                write!(f, "ukuran kisi tidak sah: {width} x {height}")
            }
            SearchError::OutOfBounds { x, y } => write!(f, "titik ({x}, {y}) di luar kisi"),
            SearchError::BlockedEndpoint(which) => {
                write!(f, "{which} berdiri di atas dinding")
            }
            SearchError::WallLengthMismatch { expected, got } => {
                write!(f, "data dinding harus {expected} sel, diberi {got}")
            }
        }
    }
}

/// Batas ukuran kisi, cukup besar untuk peragaan tetapi tidak untuk
/// menghabiskan memori peramban.
pub const MAX_DIMENSION: usize = 200;

/// Sebuah titik pada kisi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Point {
    /// Koordinat mendatar, dari kiri.
    pub x: usize,
    /// Koordinat tegak, dari atas.
    pub y: usize,
}

impl Point {
    /// Membuat titik baru.
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

/// Fungsi heuristik yang tersedia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Heuristic {
    /// Jumlah selisih mendatar dan tegak. Admissible bila gerak hanya empat arah.
    Manhattan,
    /// Jarak lurus. Admissible, tetapi lebih longgar untuk gerak empat arah.
    Euclidean,
    /// Selisih terbesar di antara kedua sumbu. Admissible untuk gerak delapan arah.
    Chebyshev,
    /// Selalu nol. Membuat A\* merosot menjadi pencarian biaya seragam.
    Zero,
}

impl Heuristic {
    /// Menaksir biaya dari `a` ke `b`.
    pub fn estimate(self, a: Point, b: Point) -> f64 {
        let dx = a.x.abs_diff(b.x) as f64;
        let dy = a.y.abs_diff(b.y) as f64;
        match self {
            Heuristic::Manhattan => dx + dy,
            Heuristic::Euclidean => (dx * dx + dy * dy).sqrt(),
            Heuristic::Chebyshev => dx.max(dy),
            Heuristic::Zero => 0.0,
        }
    }

    /// Nama pendek untuk ditampilkan.
    pub fn short_name(self) -> &'static str {
        match self {
            Heuristic::Manhattan => "Manhattan",
            Heuristic::Euclidean => "Euclidean",
            Heuristic::Chebyshev => "Chebyshev",
            Heuristic::Zero => "Nol",
        }
    }
}

/// Kisi tempat pencarian berlangsung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grid {
    /// Lebar kisi dalam sel.
    pub width: usize,
    /// Tinggi kisi dalam sel.
    pub height: usize,
    /// `true` berarti sel itu dinding. Panjangnya `width * height`, baris demi baris.
    pub walls: Vec<bool>,
    /// Apakah gerak diagonal diizinkan.
    #[serde(default)]
    pub diagonal: bool,
}

impl Grid {
    /// Kisi kosong tanpa dinding.
    pub fn new(width: usize, height: usize) -> Result<Self, SearchError> {
        if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
            return Err(SearchError::BadGrid { width, height });
        }
        Ok(Self {
            width,
            height,
            walls: vec![false; width * height],
            diagonal: false,
        })
    }

    /// Memeriksa kesahihan bentuk kisi.
    pub fn validate(&self) -> Result<(), SearchError> {
        if self.width == 0
            || self.height == 0
            || self.width > MAX_DIMENSION
            || self.height > MAX_DIMENSION
        {
            return Err(SearchError::BadGrid {
                width: self.width,
                height: self.height,
            });
        }
        let expected = self.width * self.height;
        if self.walls.len() != expected {
            return Err(SearchError::WallLengthMismatch {
                expected,
                got: self.walls.len(),
            });
        }
        Ok(())
    }

    /// Indeks datar sebuah titik.
    pub fn index(&self, p: Point) -> Option<usize> {
        if p.x < self.width && p.y < self.height {
            Some(p.y * self.width + p.x)
        } else {
            None
        }
    }

    /// Apakah sebuah titik bisa dilewati.
    pub fn passable(&self, p: Point) -> bool {
        match self.index(p) {
            Some(i) => !self.walls[i],
            None => false,
        }
    }

    /// Menandai atau membersihkan sebuah dinding.
    pub fn set_wall(&mut self, p: Point, wall: bool) -> Result<(), SearchError> {
        let i = self
            .index(p)
            .ok_or(SearchError::OutOfBounds { x: p.x, y: p.y })?;
        self.walls[i] = wall;
        Ok(())
    }

    /// Tetangga yang bisa dilewati, beserta biaya langkahnya.
    ///
    /// Urutannya tetap — atas, kanan, bawah, kiri, lalu diagonal — supaya
    /// hasil pencarian dapat direproduksi persis di implementasi lain.
    /// Gerak diagonal dilarang memotong sudut di antara dua dinding.
    pub fn neighbours(&self, p: Point) -> Vec<(Point, f64)> {
        let mut out = Vec::with_capacity(8);
        let straight: [(i64, i64); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];
        let diagonal: [(i64, i64); 4] = [(1, -1), (1, 1), (-1, 1), (-1, -1)];

        let at = |dx: i64, dy: i64| -> Option<Point> {
            let nx = p.x as i64 + dx;
            let ny = p.y as i64 + dy;
            if nx < 0 || ny < 0 {
                return None;
            }
            let q = Point::new(nx as usize, ny as usize);
            if self.passable(q) {
                Some(q)
            } else {
                None
            }
        };

        for (dx, dy) in straight {
            if let Some(q) = at(dx, dy) {
                out.push((q, 1.0));
            }
        }
        if self.diagonal {
            for (dx, dy) in diagonal {
                // Memotong sudut hanya diizinkan bila kedua sel bersebelahan
                // juga bisa dilewati; tanpa aturan ini jalurnya menembus tembok.
                if at(dx, 0).is_none() || at(0, dy).is_none() {
                    continue;
                }
                if let Some(q) = at(dx, dy) {
                    out.push((q, core::f64::consts::SQRT_2));
                }
            }
        }
        out
    }
}

/// Algoritma pencarian yang tersedia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Algorithm {
    /// Melebar lebih dulu. Menjamin jalur terpendek bila semua langkah berbiaya sama.
    BreadthFirst,
    /// Mendalam lebih dulu. Cepat, hemat memori, tetapi jalurnya bisa jauh dari optimal.
    DepthFirst,
    /// Mendalam dengan batas kedalaman.
    DepthLimited,
    /// Mendalam dengan batas yang dinaikkan bertahap. Optimal seperti BFS, hemat seperti DFS.
    IterativeDeepening,
    /// Biaya seragam. Menjamin jalur termurah walau biaya langkah berbeda-beda.
    UniformCost,
    /// Serakah. Hanya melihat taksiran sisa jarak, jadi cepat tetapi tidak optimal.
    GreedyBestFirst,
    /// A\*. Menggabungkan biaya yang sudah ditempuh dan taksiran sisa.
    AStar,
    /// Mendaki bukit. Pencarian lokal yang mudah tersangkut di puncak semu.
    HillClimbing,
    /// Pendinginan simulasi. Sesekali menerima langkah memburuk agar lolos dari puncak semu.
    SimulatedAnnealing,
}

impl Algorithm {
    /// Nama pendek untuk ditampilkan.
    pub fn short_name(self) -> &'static str {
        match self {
            Algorithm::BreadthFirst => "BFS",
            Algorithm::DepthFirst => "DFS",
            Algorithm::DepthLimited => "DLS",
            Algorithm::IterativeDeepening => "IDDFS",
            Algorithm::UniformCost => "UCS",
            Algorithm::GreedyBestFirst => "Greedy",
            Algorithm::AStar => "A*",
            Algorithm::HillClimbing => "Hill Climbing",
            Algorithm::SimulatedAnnealing => "Simulated Annealing",
        }
    }

    /// Apakah algoritma ini menjamin jalur termurah pada kisi berbiaya seragam.
    pub fn is_optimal(self) -> bool {
        matches!(
            self,
            Algorithm::BreadthFirst
                | Algorithm::IterativeDeepening
                | Algorithm::UniformCost
                | Algorithm::AStar
        )
    }

    /// Apakah algoritma ini memakai heuristik.
    pub fn uses_heuristic(self) -> bool {
        matches!(
            self,
            Algorithm::GreedyBestFirst
                | Algorithm::AStar
                | Algorithm::HillClimbing
                | Algorithm::SimulatedAnnealing
        )
    }
}

/// Pengaturan sebuah pencarian.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Algoritma yang dipakai.
    pub algorithm: Algorithm,
    /// Heuristik, diabaikan algoritma buta.
    pub heuristic: Heuristic,
    /// Batas kedalaman untuk DLS dan batas awal untuk IDDFS.
    pub depth_limit: usize,
    /// Benih keacakan untuk simulated annealing.
    pub seed: u64,
    /// Batas simpul yang boleh dibuka, sebagai jaring pengaman.
    pub max_expansions: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            algorithm: Algorithm::AStar,
            heuristic: Heuristic::Manhattan,
            depth_limit: 64,
            seed: 0x5EED,
            max_expansions: 100_000,
        }
    }
}

/// Hasil sebuah pencarian.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    /// Jalur dari awal ke tujuan, kosong bila tidak ditemukan.
    pub path: Vec<Point>,
    /// Urutan simpul yang dibuka, dipakai untuk menganimasikan pencarian.
    pub expanded: Vec<Point>,
    /// Total biaya jalur.
    pub cost: f64,
    /// Apakah tujuan tercapai.
    pub found: bool,
    /// Jumlah simpul yang dibuka.
    pub expansions: usize,
    /// Ukuran terbesar yang pernah dicapai daftar tunggu.
    pub peak_frontier: usize,
}

impl SearchResult {
    /// Hasil kosong untuk pencarian yang gagal.
    fn failed(expanded: Vec<Point>, peak_frontier: usize) -> Self {
        let expansions = expanded.len();
        Self {
            path: Vec::new(),
            expanded,
            cost: f64::INFINITY,
            found: false,
            expansions,
            peak_frontier,
        }
    }
}

/// Menyusun ulang jalur dari peta pendahulu.
fn reconstruct(came_from: &HashMap<Point, Point>, goal: Point) -> Vec<Point> {
    let mut path = vec![goal];
    let mut current = goal;
    // Batas iterasi mencegah putaran tak berhingga bila peta pendahulu rusak.
    for _ in 0..(MAX_DIMENSION * MAX_DIMENSION + 1) {
        match came_from.get(&current) {
            Some(prev) => {
                path.push(*prev);
                current = *prev;
            }
            None => break,
        }
    }
    path.reverse();
    path
}

/// Biaya sebuah jalur pada kisi tertentu.
pub fn path_cost(grid: &Grid, path: &[Point]) -> f64 {
    path.windows(2)
        .map(|w| {
            grid.neighbours(w[0])
                .into_iter()
                .find(|(q, _)| *q == w[1])
                .map(|(_, c)| c)
                .unwrap_or(f64::INFINITY)
        })
        .sum()
}

/// Simpul di dalam antrean prioritas.
///
/// `BinaryHeap` di Rust adalah tumpukan maksimum, sedangkan yang dibutuhkan
/// adalah minimum, jadi urutannya dibalik. Prioritas disimpan sebagai bit
/// bilangan bulat agar perbandingannya total dan hasilnya sama persis di
/// implementasi lain — `f64` tidak punya urutan total karena `NaN`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Node {
    priority_bits: u64,
    /// Pemutus seri: taksiran sisa jarak.
    ///
    /// Tanpa ini, A\* pada ruang terbuka melebar percuma. Ribuan simpul di sana
    /// punya `f` yang persis sama, sehingga urutan masuk yang menentukan
    /// siapa dibuka lebih dulu — dan hasilnya A\* membuka seluruh kisi, sama
    /// banyaknya dengan pencarian tanpa heuristik. Mendahulukan simpul yang
    /// taksiran sisanya terkecil membuat pencarian menempel pada arah tujuan.
    tie_bits: u64,
    order: usize,
    point: Point,
}

/// Mengubah prioritas menjadi bilangan bulat yang urutannya setara.
///
/// Percabangannya memakai **bit tanda**, bukan perbandingan `v >= 0.0`.
/// Perbandingan itu menyesatkan: pada IEEE-754 `-0.0 >= 0.0` bernilai benar,
/// sehingga nol negatif akan masuk cabang bilangan positif dan seluruh
/// urutannya rusak untuk nilai negatif.
fn priority_key(v: f64) -> u64 {
    // Nilai bukan bilangan diletakkan paling belakang.
    if v.is_nan() {
        return u64::MAX;
    }
    let bits = v.to_bits();
    const SIGN: u64 = 0x8000_0000_0000_0000;
    if bits & SIGN != 0 {
        !bits
    } else {
        bits ^ SIGN
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // Dibalik supaya prioritas terkecil keluar lebih dulu. Seri diputus
        // oleh urutan masuk, sehingga hasilnya deterministik.
        other
            .priority_bits
            .cmp(&self.priority_bits)
            .then_with(|| other.tie_bits.cmp(&self.tie_bits))
            .then_with(|| other.order.cmp(&self.order))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Menjalankan pencarian dari `start` ke `goal`.
pub fn search(
    grid: &Grid,
    start: Point,
    goal: Point,
    options: SearchOptions,
) -> Result<SearchResult, SearchError> {
    grid.validate()?;
    if grid.index(start).is_none() {
        return Err(SearchError::OutOfBounds {
            x: start.x,
            y: start.y,
        });
    }
    if grid.index(goal).is_none() {
        return Err(SearchError::OutOfBounds {
            x: goal.x,
            y: goal.y,
        });
    }
    if !grid.passable(start) {
        return Err(SearchError::BlockedEndpoint("titik awal"));
    }
    if !grid.passable(goal) {
        return Err(SearchError::BlockedEndpoint("tujuan"));
    }

    Ok(match options.algorithm {
        Algorithm::BreadthFirst => breadth_first(grid, start, goal, options),
        Algorithm::DepthFirst => depth_limited(grid, start, goal, options, usize::MAX),
        Algorithm::DepthLimited => depth_limited(grid, start, goal, options, options.depth_limit),
        Algorithm::IterativeDeepening => iterative_deepening(grid, start, goal, options),
        Algorithm::UniformCost | Algorithm::GreedyBestFirst | Algorithm::AStar => {
            best_first(grid, start, goal, options)
        }
        Algorithm::HillClimbing => hill_climbing(grid, start, goal, options),
        Algorithm::SimulatedAnnealing => simulated_annealing(grid, start, goal, options),
    })
}

fn breadth_first(grid: &Grid, start: Point, goal: Point, o: SearchOptions) -> SearchResult {
    let mut queue = VecDeque::from([start]);
    let mut seen = HashSet::from([start]);
    let mut came_from: HashMap<Point, Point> = HashMap::new();
    let mut expanded = Vec::new();
    let mut peak = 1usize;

    while let Some(current) = queue.pop_front() {
        expanded.push(current);
        if current == goal {
            let path = reconstruct(&came_from, goal);
            let cost = path_cost(grid, &path);
            return SearchResult {
                cost,
                found: true,
                expansions: expanded.len(),
                path,
                expanded,
                peak_frontier: peak,
            };
        }
        if expanded.len() >= o.max_expansions {
            break;
        }
        for (next, _) in grid.neighbours(current) {
            if seen.insert(next) {
                came_from.insert(next, current);
                queue.push_back(next);
            }
        }
        peak = peak.max(queue.len());
    }
    SearchResult::failed(expanded, peak)
}

fn depth_limited(
    grid: &Grid,
    start: Point,
    goal: Point,
    o: SearchOptions,
    limit: usize,
) -> SearchResult {
    // Ditulis iteratif, bukan rekursif, agar kedalaman kisi besar tidak
    // meluapkan tumpukan pemanggilan di WebAssembly.
    let mut stack = vec![(start, 0usize)];
    let mut seen = HashSet::from([start]);
    let mut came_from: HashMap<Point, Point> = HashMap::new();
    let mut expanded = Vec::new();
    let mut peak = 1usize;

    while let Some((current, depth)) = stack.pop() {
        expanded.push(current);
        if current == goal {
            let path = reconstruct(&came_from, goal);
            let cost = path_cost(grid, &path);
            return SearchResult {
                cost,
                found: true,
                expansions: expanded.len(),
                path,
                expanded,
                peak_frontier: peak,
            };
        }
        if expanded.len() >= o.max_expansions || depth >= limit {
            continue;
        }
        // Dibalik supaya tetangga pertama diperiksa lebih dulu, sesuai urutan
        // yang sama dengan BFS.
        for (next, _) in grid.neighbours(current).into_iter().rev() {
            if seen.insert(next) {
                came_from.insert(next, current);
                stack.push((next, depth + 1));
            }
        }
        peak = peak.max(stack.len());
    }
    SearchResult::failed(expanded, peak)
}

fn iterative_deepening(grid: &Grid, start: Point, goal: Point, o: SearchOptions) -> SearchResult {
    let ceiling = (grid.width * grid.height).min(o.max_expansions);
    let mut all_expanded = Vec::new();
    let mut peak = 0usize;
    for limit in 0..=ceiling {
        let attempt = depth_limited(grid, start, goal, o, limit);
        all_expanded.extend(attempt.expanded.iter().copied());
        peak = peak.max(attempt.peak_frontier);
        if attempt.found {
            return SearchResult {
                expansions: all_expanded.len(),
                expanded: all_expanded,
                peak_frontier: peak,
                ..attempt
            };
        }
        if all_expanded.len() >= o.max_expansions {
            break;
        }
    }
    SearchResult::failed(all_expanded, peak)
}

fn best_first(grid: &Grid, start: Point, goal: Point, o: SearchOptions) -> SearchResult {
    let mut heap = BinaryHeap::new();
    let mut g_score: HashMap<Point, f64> = HashMap::from([(start, 0.0)]);
    let mut came_from: HashMap<Point, Point> = HashMap::new();
    let mut closed: HashSet<Point> = HashSet::new();
    let mut expanded = Vec::new();
    let mut order = 0usize;
    let mut peak = 1usize;

    let f = |point: Point, g: f64| -> f64 {
        match o.algorithm {
            Algorithm::UniformCost => g,
            Algorithm::GreedyBestFirst => o.heuristic.estimate(point, goal),
            _ => g + o.heuristic.estimate(point, goal),
        }
    };

    // Pemutus seri hanya berlaku untuk algoritma yang memang berheuristik.
    // Pencarian biaya seragam menurut definisinya tidak mengenal heuristik;
    // memakainya untuk memutus seri di sana akan diam-diam mengubah UCS
    // menjadi algoritma lain yang kebetulan bernama sama.
    let tie = |point: Point| -> f64 {
        if o.algorithm == Algorithm::UniformCost {
            0.0
        } else {
            o.heuristic.estimate(point, goal)
        }
    };

    heap.push(Node {
        priority_bits: priority_key(f(start, 0.0)),
        tie_bits: priority_key(tie(start)),
        order,
        point: start,
    });

    while let Some(node) = heap.pop() {
        let current = node.point;
        if !closed.insert(current) {
            continue;
        }
        expanded.push(current);
        if current == goal {
            let path = reconstruct(&came_from, goal);
            let cost = path_cost(grid, &path);
            return SearchResult {
                cost,
                found: true,
                expansions: expanded.len(),
                path,
                expanded,
                peak_frontier: peak,
            };
        }
        if expanded.len() >= o.max_expansions {
            break;
        }
        let current_g = *g_score.get(&current).unwrap_or(&f64::INFINITY);
        for (next, step) in grid.neighbours(current) {
            if closed.contains(&next) {
                continue;
            }
            let tentative = current_g + step;
            if tentative < *g_score.get(&next).unwrap_or(&f64::INFINITY) {
                g_score.insert(next, tentative);
                came_from.insert(next, current);
                order += 1;
                heap.push(Node {
                    priority_bits: priority_key(f(next, tentative)),
                    tie_bits: priority_key(tie(next)),
                    order,
                    point: next,
                });
            }
        }
        peak = peak.max(heap.len());
    }
    SearchResult::failed(expanded, peak)
}

fn hill_climbing(grid: &Grid, start: Point, goal: Point, o: SearchOptions) -> SearchResult {
    let mut current = start;
    let mut path = vec![start];
    let mut expanded = vec![start];
    let mut visited = HashSet::from([start]);

    while current != goal && expanded.len() < o.max_expansions {
        let here = o.heuristic.estimate(current, goal);
        // Hanya tetangga yang benar-benar lebih dekat yang diterima. Begitu
        // tidak ada, pencarian berhenti — inilah puncak semu yang membuat
        // metode ini gagal pada peta berbentuk huruf U.
        let best = grid
            .neighbours(current)
            .into_iter()
            .filter(|(q, _)| !visited.contains(q))
            .map(|(q, _)| (q, o.heuristic.estimate(q, goal)))
            .filter(|(_, h)| *h < here)
            .min_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

        match best {
            Some((next, _)) => {
                visited.insert(next);
                expanded.push(next);
                path.push(next);
                current = next;
            }
            None => break,
        }
    }

    let found = current == goal;
    let cost = if found {
        path_cost(grid, &path)
    } else {
        f64::INFINITY
    };
    let expansions = expanded.len();
    SearchResult {
        path: if found { path } else { Vec::new() },
        expanded,
        cost,
        found,
        expansions,
        peak_frontier: 1,
    }
}

fn simulated_annealing(grid: &Grid, start: Point, goal: Point, o: SearchOptions) -> SearchResult {
    let mut rng = SplitMix64::new(o.seed);
    let mut current = start;
    let mut path = vec![start];
    let mut expanded = vec![start];

    let steps = o.max_expansions.min(20_000);
    for i in 0..steps {
        if current == goal {
            break;
        }
        let options = grid.neighbours(current);
        if options.is_empty() {
            break;
        }
        // Suhu menurun dari 1 ke mendekati 0. Selagi panas, langkah yang
        // memperburuk jarak masih mungkin diterima; setelah dingin, perilakunya
        // menyerupai hill climbing.
        let temperature = 1.0 - (i as f64 / steps as f64);
        let (candidate, _) = options[rng.below(options.len() as u64) as usize];
        let delta = o.heuristic.estimate(candidate, goal) - o.heuristic.estimate(current, goal);

        let accept = if delta < 0.0 {
            true
        } else if temperature <= f64::MIN_POSITIVE {
            false
        } else {
            rng.next_f64() < (-delta / temperature).exp()
        };

        if accept {
            current = candidate;
            path.push(candidate);
            expanded.push(candidate);
        }
    }

    let found = current == goal;
    let expansions = expanded.len();
    SearchResult {
        path: if found { path.clone() } else { Vec::new() },
        expanded,
        cost: if found {
            path_cost(grid, &path)
        } else {
            f64::INFINITY
        },
        found,
        expansions,
        peak_frontier: 1,
    }
}

/// Membangun labirin acak yang dijamin punya jalur dari awal ke tujuan.
///
/// Memakai pertumbuhan pohon rentang acak pada sel bernomor ganjil, sehingga
/// hasilnya selalu terhubung — labirin yang mustahil diselesaikan tidak
/// mengajarkan apa pun tentang perbedaan antaralgoritma.
pub fn generate_maze(width: usize, height: usize, seed: u64) -> Result<Grid, SearchError> {
    let mut grid = Grid::new(width, height)?;
    grid.walls.iter_mut().for_each(|w| *w = true);
    let mut rng = SplitMix64::new(seed);

    let mut stack = vec![Point::new(0, 0)];
    grid.walls[0] = false;

    while let Some(current) = stack.last().copied() {
        let mut candidates: Vec<(Point, Point)> = Vec::with_capacity(4);
        for (dx, dy) in [(0i64, -2i64), (2, 0), (0, 2), (-2, 0)] {
            let nx = current.x as i64 + dx;
            let ny = current.y as i64 + dy;
            if nx < 0 || ny < 0 || nx as usize >= width || ny as usize >= height {
                continue;
            }
            let next = Point::new(nx as usize, ny as usize);
            if let Some(i) = grid.index(next) {
                if grid.walls[i] {
                    let between = Point::new(
                        (current.x as i64 + dx / 2) as usize,
                        (current.y as i64 + dy / 2) as usize,
                    );
                    candidates.push((next, between));
                }
            }
        }

        if candidates.is_empty() {
            stack.pop();
            continue;
        }
        let (next, between) = candidates[rng.below(candidates.len() as u64) as usize];
        if let Some(i) = grid.index(between) {
            grid.walls[i] = false;
        }
        if let Some(i) = grid.index(next) {
            grid.walls[i] = false;
        }
        stack.push(next);
    }

    // Sudut kanan bawah dijamin terbuka agar selalu bisa dipakai sebagai tujuan.
    let goal = Point::new(width - 1, height - 1);
    if let Some(i) = grid.index(goal) {
        if grid.walls[i] {
            grid.walls[i] = false;
            // Membuka satu sel bersebelahan supaya tujuan tidak terkurung.
            for (dx, dy) in [(-1i64, 0i64), (0, -1)] {
                let nx = goal.x as i64 + dx;
                let ny = goal.y as i64 + dy;
                if nx >= 0 && ny >= 0 {
                    if let Some(j) = grid.index(Point::new(nx as usize, ny as usize)) {
                        grid.walls[j] = false;
                    }
                }
            }
        }
    }
    Ok(grid)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Kisi 7x7 dengan dinding tegak berlubang di tengah.
    fn kisi_berdinding() -> Grid {
        let mut g = Grid::new(7, 7).unwrap();
        for y in 0..7 {
            if y != 3 {
                g.set_wall(Point::new(3, y), true).unwrap();
            }
        }
        g
    }

    fn semua_algoritma() -> [Algorithm; 9] {
        [
            Algorithm::BreadthFirst,
            Algorithm::DepthFirst,
            Algorithm::DepthLimited,
            Algorithm::IterativeDeepening,
            Algorithm::UniformCost,
            Algorithm::GreedyBestFirst,
            Algorithm::AStar,
            Algorithm::HillClimbing,
            Algorithm::SimulatedAnnealing,
        ]
    }

    #[test]
    fn kisi_baru_kosong() {
        let g = Grid::new(4, 3).unwrap();
        assert_eq!(g.walls.len(), 12);
        assert!(g.walls.iter().all(|w| !w));
        assert!(g.validate().is_ok());
    }

    #[test]
    fn kisi_menolak_ukuran_tak_sah() {
        assert!(matches!(Grid::new(0, 5), Err(SearchError::BadGrid { .. })));
        assert!(matches!(Grid::new(5, 0), Err(SearchError::BadGrid { .. })));
        assert!(matches!(
            Grid::new(MAX_DIMENSION + 1, 5),
            Err(SearchError::BadGrid { .. })
        ));
    }

    #[test]
    fn kisi_menolak_dinding_yang_tidak_sepadan() {
        let mut g = Grid::new(3, 3).unwrap();
        g.walls.push(false);
        assert!(matches!(
            g.validate(),
            Err(SearchError::WallLengthMismatch {
                expected: 9,
                got: 10
            })
        ));
    }

    #[test]
    fn indeks_dan_keterlewatan() {
        let mut g = Grid::new(4, 3).unwrap();
        assert_eq!(g.index(Point::new(0, 0)), Some(0));
        assert_eq!(g.index(Point::new(3, 2)), Some(11));
        assert_eq!(g.index(Point::new(4, 0)), None);
        assert_eq!(g.index(Point::new(0, 3)), None);
        assert!(g.passable(Point::new(1, 1)));
        g.set_wall(Point::new(1, 1), true).unwrap();
        assert!(!g.passable(Point::new(1, 1)));
        assert!(!g.passable(Point::new(9, 9)));
    }

    #[test]
    fn menandai_dinding_di_luar_kisi_gagal() {
        let mut g = Grid::new(3, 3).unwrap();
        assert_eq!(
            g.set_wall(Point::new(5, 5), true),
            Err(SearchError::OutOfBounds { x: 5, y: 5 })
        );
    }

    #[test]
    fn tetangga_di_sudut_hanya_dua() {
        let g = Grid::new(5, 5).unwrap();
        assert_eq!(g.neighbours(Point::new(0, 0)).len(), 2);
        assert_eq!(g.neighbours(Point::new(4, 4)).len(), 2);
        assert_eq!(g.neighbours(Point::new(2, 2)).len(), 4);
    }

    #[test]
    fn tetangga_melewati_dinding() {
        let mut g = Grid::new(5, 5).unwrap();
        g.set_wall(Point::new(2, 1), true).unwrap();
        let n = g.neighbours(Point::new(2, 2));
        assert_eq!(n.len(), 3);
        assert!(!n.iter().any(|(p, _)| *p == Point::new(2, 1)));
    }

    #[test]
    fn diagonal_tidak_memotong_sudut() {
        let mut g = Grid::new(5, 5).unwrap();
        g.diagonal = true;
        g.set_wall(Point::new(3, 2), true).unwrap();
        g.set_wall(Point::new(2, 1), true).unwrap();
        // Kedua sel lurus menuju (3,1) tertutup, jadi diagonalnya harus ditolak.
        let n = g.neighbours(Point::new(2, 2));
        assert!(!n.iter().any(|(p, _)| *p == Point::new(3, 1)));
    }

    #[test]
    fn biaya_langkah_diagonal_akar_dua() {
        let mut g = Grid::new(5, 5).unwrap();
        g.diagonal = true;
        let n = g.neighbours(Point::new(2, 2));
        let diag = n
            .iter()
            .find(|(p, _)| *p == Point::new(3, 1))
            .expect("diagonal tersedia");
        assert!((diag.1 - core::f64::consts::SQRT_2).abs() < 1e-12);
    }

    #[test]
    fn heuristik_nol_pada_titik_sama() {
        let p = Point::new(3, 4);
        for h in [
            Heuristic::Manhattan,
            Heuristic::Euclidean,
            Heuristic::Chebyshev,
            Heuristic::Zero,
        ] {
            assert_eq!(h.estimate(p, p), 0.0);
            assert!(!h.short_name().is_empty());
        }
    }

    #[test]
    fn heuristik_nilai_yang_dikenal() {
        let a = Point::new(0, 0);
        let b = Point::new(3, 4);
        assert_eq!(Heuristic::Manhattan.estimate(a, b), 7.0);
        assert_eq!(Heuristic::Euclidean.estimate(a, b), 5.0);
        assert_eq!(Heuristic::Chebyshev.estimate(a, b), 4.0);
        assert_eq!(Heuristic::Zero.estimate(a, b), 0.0);
    }

    #[test]
    fn heuristik_setangkup() {
        let a = Point::new(1, 6);
        let b = Point::new(5, 2);
        for h in [
            Heuristic::Manhattan,
            Heuristic::Euclidean,
            Heuristic::Chebyshev,
        ] {
            assert_eq!(h.estimate(a, b), h.estimate(b, a));
        }
    }

    #[test]
    fn heuristik_manhattan_admissible_pada_gerak_empat_arah() {
        // Taksiran tidak boleh melebihi biaya sebenarnya, kalau tidak A* bisa
        // mengembalikan jalur yang bukan terpendek.
        let g = Grid::new(9, 9).unwrap();
        let goal = Point::new(8, 8);
        for y in 0..9 {
            for x in 0..9 {
                let p = Point::new(x, y);
                let sebenarnya = search(
                    &g,
                    p,
                    goal,
                    SearchOptions {
                        algorithm: Algorithm::BreadthFirst,
                        ..Default::default()
                    },
                )
                .unwrap();
                assert!(
                    Heuristic::Manhattan.estimate(p, goal) <= sebenarnya.cost + 1e-9,
                    "heuristik melebihi biaya sebenarnya di {p:?}"
                );
            }
        }
    }

    #[test]
    fn semua_algoritma_menemukan_jalan_di_kisi_kosong() {
        let g = Grid::new(9, 9).unwrap();
        let start = Point::new(0, 0);
        let goal = Point::new(8, 8);
        for algorithm in semua_algoritma() {
            let r = search(
                &g,
                start,
                goal,
                SearchOptions {
                    algorithm,
                    depth_limit: 64,
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(r.found, "{} gagal di kisi kosong", algorithm.short_name());
            assert_eq!(r.path.first(), Some(&start));
            assert_eq!(r.path.last(), Some(&goal));
        }
    }

    #[test]
    fn jalur_selalu_bersambung_dan_bisa_dilewati() {
        let g = kisi_berdinding();
        let start = Point::new(0, 0);
        let goal = Point::new(6, 6);
        for algorithm in semua_algoritma() {
            let r = search(
                &g,
                start,
                goal,
                SearchOptions {
                    algorithm,
                    depth_limit: 128,
                    max_expansions: 50_000,
                    ..Default::default()
                },
            )
            .unwrap();
            if !r.found {
                continue;
            }
            for w in r.path.windows(2) {
                assert!(
                    g.neighbours(w[0]).iter().any(|(q, _)| *q == w[1]),
                    "{}: {:?} dan {:?} tidak bersebelahan",
                    algorithm.short_name(),
                    w[0],
                    w[1]
                );
            }
            assert!(
                r.path.iter().all(|p| g.passable(*p)),
                "{} melewati dinding",
                algorithm.short_name()
            );
        }
    }

    #[test]
    fn algoritma_optimal_sepakat_pada_biaya_terpendek() {
        let g = kisi_berdinding();
        let start = Point::new(0, 0);
        let goal = Point::new(6, 6);
        let mut biaya = Vec::new();
        for algorithm in [
            Algorithm::BreadthFirst,
            Algorithm::IterativeDeepening,
            Algorithm::UniformCost,
            Algorithm::AStar,
        ] {
            let r = search(
                &g,
                start,
                goal,
                SearchOptions {
                    algorithm,
                    max_expansions: 200_000,
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(r.found, "{} gagal", algorithm.short_name());
            assert!(algorithm.is_optimal());
            biaya.push((algorithm.short_name(), r.cost));
        }
        let pertama = biaya[0].1;
        for (nama, c) in &biaya {
            assert!(
                (c - pertama).abs() < 1e-9,
                "{nama} menghasilkan {c}, bukan {pertama}"
            );
        }
    }

    #[test]
    fn astar_membuka_lebih_sedikit_daripada_ucs_di_ruang_terbuka() {
        // Inti dari heuristik: hasilnya sama, kerjanya lebih ringan.
        //
        // Uji ini pernah gagal karena A* membuka seluruh 441 sel, sama seperti
        // UCS. Penyebabnya bukan heuristiknya, melainkan tidak adanya pemutus
        // seri: di ruang terbuka ribuan simpul punya nilai `f` identik. Setelah
        // seri diputus dengan taksiran sisa terkecil, selisihnya jelas.
        let g = Grid::new(21, 21).unwrap();
        let start = Point::new(0, 0);
        let goal = Point::new(20, 20);
        let ucs = search(
            &g,
            start,
            goal,
            SearchOptions {
                algorithm: Algorithm::UniformCost,
                ..Default::default()
            },
        )
        .unwrap();
        let astar = search(
            &g,
            start,
            goal,
            SearchOptions {
                algorithm: Algorithm::AStar,
                heuristic: Heuristic::Manhattan,
                ..Default::default()
            },
        )
        .unwrap();
        assert!((ucs.cost - astar.cost).abs() < 1e-9, "biaya harus sama");
        assert!(
            astar.expansions < ucs.expansions,
            "A* membuka {} simpul, UCS {}",
            astar.expansions,
            ucs.expansions
        );
    }

    #[test]
    fn astar_membuka_lebih_sedikit_daripada_ucs_di_labirin() {
        let g = generate_maze(31, 31, 2026).unwrap();
        let start = Point::new(0, 0);
        let goal = Point::new(30, 30);
        let opts = |algorithm| SearchOptions {
            algorithm,
            heuristic: Heuristic::Manhattan,
            max_expansions: 200_000,
            ..Default::default()
        };
        let ucs = search(&g, start, goal, opts(Algorithm::UniformCost)).unwrap();
        let astar = search(&g, start, goal, opts(Algorithm::AStar)).unwrap();
        assert!(ucs.found && astar.found);
        assert!((ucs.cost - astar.cost).abs() < 1e-9, "biaya harus sama");
        assert!(
            astar.expansions <= ucs.expansions,
            "A* membuka {} simpul, UCS {}",
            astar.expansions,
            ucs.expansions
        );
    }

    #[test]
    fn pemutus_seri_tidak_merusak_optimalitas() {
        // Pemutus seri mempercepat, tetapi tidak boleh mengubah biaya jalur.
        for seed in 0..6u64 {
            let g = generate_maze(21, 21, seed).unwrap();
            let start = Point::new(0, 0);
            let goal = Point::new(20, 20);
            let bfs = search(
                &g,
                start,
                goal,
                SearchOptions {
                    algorithm: Algorithm::BreadthFirst,
                    max_expansions: 200_000,
                    ..Default::default()
                },
            )
            .unwrap();
            let astar = search(
                &g,
                start,
                goal,
                SearchOptions {
                    algorithm: Algorithm::AStar,
                    max_expansions: 200_000,
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(bfs.found && astar.found, "labirin benih {seed}");
            assert!(
                (bfs.cost - astar.cost).abs() < 1e-9,
                "benih {seed}: BFS {} vs A* {}",
                bfs.cost,
                astar.cost
            );
        }
    }

    #[test]
    fn kunci_prioritas_menangani_nol_negatif() {
        // Regresi: percabangan pernah memakai `v >= 0.0`, padahal
        // `-0.0 >= 0.0` bernilai benar pada IEEE-754 sehingga nol negatif
        // masuk cabang yang salah dan merusak urutan seluruh nilai negatif.
        assert!(priority_key(-1.5) < priority_key(-0.0));
        assert!(priority_key(-0.0) < priority_key(0.0));
        assert!(priority_key(0.0) < priority_key(1.5));
        assert!(priority_key(-1.0) < priority_key(1.0));
    }

    #[test]
    fn ucs_tidak_terpengaruh_pilihan_heuristik() {
        // Kalau UCS berubah perilaku saat heuristiknya diganti, berarti ia
        // bukan UCS lagi.
        let g = generate_maze(21, 21, 3).unwrap();
        let start = Point::new(0, 0);
        let goal = Point::new(20, 20);
        let jalan = |heuristic| {
            search(
                &g,
                start,
                goal,
                SearchOptions {
                    algorithm: Algorithm::UniformCost,
                    heuristic,
                    max_expansions: 200_000,
                    ..Default::default()
                },
            )
            .unwrap()
        };
        let a = jalan(Heuristic::Manhattan);
        let b = jalan(Heuristic::Zero);
        let c = jalan(Heuristic::Euclidean);
        assert_eq!(a.expanded, b.expanded);
        assert_eq!(a.expanded, c.expanded);
        assert_eq!(a.expansions, b.expansions);
    }

    #[test]
    fn astar_dengan_heuristik_nol_setara_ucs() {
        let g = kisi_berdinding();
        let start = Point::new(0, 0);
        let goal = Point::new(6, 6);
        let ucs = search(
            &g,
            start,
            goal,
            SearchOptions {
                algorithm: Algorithm::UniformCost,
                ..Default::default()
            },
        )
        .unwrap();
        let astar = search(
            &g,
            start,
            goal,
            SearchOptions {
                algorithm: Algorithm::AStar,
                heuristic: Heuristic::Zero,
                ..Default::default()
            },
        )
        .unwrap();
        assert!((ucs.cost - astar.cost).abs() < 1e-9);
        assert_eq!(ucs.expansions, astar.expansions);
    }

    #[test]
    fn greedy_lebih_cepat_tetapi_tidak_dijamin_optimal() {
        let g = kisi_berdinding();
        let start = Point::new(0, 0);
        let goal = Point::new(6, 6);
        let greedy = search(
            &g,
            start,
            goal,
            SearchOptions {
                algorithm: Algorithm::GreedyBestFirst,
                ..Default::default()
            },
        )
        .unwrap();
        let astar = search(
            &g,
            start,
            goal,
            SearchOptions {
                algorithm: Algorithm::AStar,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(greedy.found && astar.found);
        // Serakah tidak pernah lebih murah daripada A*, karena A* optimal.
        assert!(greedy.cost >= astar.cost - 1e-9);
        assert!(!Algorithm::GreedyBestFirst.is_optimal());
    }

    #[test]
    fn hill_climbing_tersangkut_di_puncak_semu() {
        // Hill climbing hanya menerima langkah yang memperkecil taksiran jarak,
        // jadi ia hanya bisa bergerak ke kanan atau ke bawah. Dinding di bawah
        // ini menuntunnya ke sel (2,2), tempat kedua arah itu tertutup —
        // padahal tujuannya tetap terjangkau lewat sisi atas.
        //
        //   . . . . . . .      S = (0,0), G = (6,6)
        //   # . . . . . .      # = dinding
        //   . . . # . . .
        //   . # # . . . .
        //   . . . . . . .
        let mut g = Grid::new(7, 7).unwrap();
        for p in [
            Point::new(0, 1),
            Point::new(1, 3),
            Point::new(2, 3),
            Point::new(3, 2),
        ] {
            g.set_wall(p, true).unwrap();
        }
        let r = search(
            &g,
            Point::new(0, 0),
            Point::new(6, 6),
            SearchOptions {
                algorithm: Algorithm::HillClimbing,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!r.found, "hill climbing seharusnya gagal di peta ini");
        assert!(r.cost.is_infinite());
        assert!(r.path.is_empty());

        // A* pada peta yang sama tetap menemukan jalan.
        let a = search(
            &g,
            Point::new(0, 0),
            Point::new(6, 6),
            SearchOptions {
                algorithm: Algorithm::AStar,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(a.found);
    }

    #[test]
    fn simulated_annealing_deterministik_untuk_benih_sama() {
        let g = Grid::new(11, 11).unwrap();
        let opts = SearchOptions {
            algorithm: Algorithm::SimulatedAnnealing,
            seed: 12345,
            ..Default::default()
        };
        let a = search(&g, Point::new(0, 0), Point::new(10, 10), opts).unwrap();
        let b = search(&g, Point::new(0, 0), Point::new(10, 10), opts).unwrap();
        assert_eq!(a.expanded, b.expanded);
        assert_eq!(a.found, b.found);
    }

    #[test]
    fn simulated_annealing_berbeda_untuk_benih_berbeda() {
        let g = Grid::new(11, 11).unwrap();
        let jalan = |seed: u64| {
            search(
                &g,
                Point::new(0, 0),
                Point::new(10, 10),
                SearchOptions {
                    algorithm: Algorithm::SimulatedAnnealing,
                    seed,
                    ..Default::default()
                },
            )
            .unwrap()
            .expanded
        };
        assert_ne!(jalan(1), jalan(2));
    }

    #[test]
    fn batas_kedalaman_menghentikan_dls() {
        let g = Grid::new(9, 9).unwrap();
        let r = search(
            &g,
            Point::new(0, 0),
            Point::new(8, 8),
            SearchOptions {
                algorithm: Algorithm::DepthLimited,
                depth_limit: 3,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!r.found, "tujuan berjarak 16 langkah, batasnya 3");
    }

    #[test]
    fn iddfs_menemukan_yang_gagal_ditemukan_dls_dangkal() {
        let g = Grid::new(9, 9).unwrap();
        let dls = search(
            &g,
            Point::new(0, 0),
            Point::new(8, 8),
            SearchOptions {
                algorithm: Algorithm::DepthLimited,
                depth_limit: 3,
                ..Default::default()
            },
        )
        .unwrap();
        let iddfs = search(
            &g,
            Point::new(0, 0),
            Point::new(8, 8),
            SearchOptions {
                algorithm: Algorithm::IterativeDeepening,
                depth_limit: 3,
                max_expansions: 200_000,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!dls.found);
        assert!(iddfs.found);
    }

    #[test]
    fn tujuan_terkurung_tidak_ditemukan() {
        let mut g = Grid::new(7, 7).unwrap();
        for y in 0..7 {
            g.set_wall(Point::new(3, y), true).unwrap();
        }
        for algorithm in [
            Algorithm::BreadthFirst,
            Algorithm::UniformCost,
            Algorithm::AStar,
            Algorithm::DepthFirst,
        ] {
            let r = search(
                &g,
                Point::new(0, 0),
                Point::new(6, 6),
                SearchOptions {
                    algorithm,
                    max_expansions: 50_000,
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(
                !r.found,
                "{} mengaku menemukan jalan",
                algorithm.short_name()
            );
            assert!(r.path.is_empty());
            assert!(r.cost.is_infinite());
        }
    }

    #[test]
    fn awal_sama_dengan_tujuan() {
        let g = Grid::new(5, 5).unwrap();
        let p = Point::new(2, 2);
        for algorithm in semua_algoritma() {
            let r = search(
                &g,
                p,
                p,
                SearchOptions {
                    algorithm,
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(r.found, "{}", algorithm.short_name());
            assert_eq!(r.cost, 0.0);
        }
    }

    #[test]
    fn titik_di_luar_kisi_ditolak() {
        let g = Grid::new(5, 5).unwrap();
        let o = SearchOptions::default();
        assert_eq!(
            search(&g, Point::new(9, 0), Point::new(1, 1), o),
            Err(SearchError::OutOfBounds { x: 9, y: 0 })
        );
        assert_eq!(
            search(&g, Point::new(0, 0), Point::new(0, 9), o),
            Err(SearchError::OutOfBounds { x: 0, y: 9 })
        );
    }

    #[test]
    fn ujung_di_atas_dinding_ditolak() {
        let mut g = Grid::new(5, 5).unwrap();
        g.set_wall(Point::new(0, 0), true).unwrap();
        let o = SearchOptions::default();
        assert_eq!(
            search(&g, Point::new(0, 0), Point::new(4, 4), o),
            Err(SearchError::BlockedEndpoint("titik awal"))
        );
        let mut g2 = Grid::new(5, 5).unwrap();
        g2.set_wall(Point::new(4, 4), true).unwrap();
        assert_eq!(
            search(&g2, Point::new(0, 0), Point::new(4, 4), o),
            Err(SearchError::BlockedEndpoint("tujuan"))
        );
    }

    #[test]
    fn batas_pembukaan_dihormati() {
        let g = Grid::new(50, 50).unwrap();
        let r = search(
            &g,
            Point::new(0, 0),
            Point::new(49, 49),
            SearchOptions {
                algorithm: Algorithm::BreadthFirst,
                max_expansions: 20,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(r.expansions <= 21, "membuka {} simpul", r.expansions);
        assert!(!r.found);
    }

    #[test]
    fn biaya_jalur_dihitung_benar() {
        let g = Grid::new(5, 5).unwrap();
        let jalur = vec![Point::new(0, 0), Point::new(1, 0), Point::new(2, 0)];
        assert_eq!(path_cost(&g, &jalur), 2.0);
        assert_eq!(path_cost(&g, &[Point::new(0, 0)]), 0.0);
        // Jalur yang melompat menghasilkan biaya tak berhingga, bukan angka palsu.
        assert!(path_cost(&g, &[Point::new(0, 0), Point::new(4, 4)]).is_infinite());
    }

    #[test]
    fn kunci_prioritas_mempertahankan_urutan() {
        let nilai = [
            -10.0,
            -1.5,
            -0.0,
            0.0,
            1e-300,
            0.5,
            1.0,
            1e300,
            f64::INFINITY,
        ];
        for w in nilai.windows(2) {
            assert!(
                priority_key(w[0]) <= priority_key(w[1]),
                "{} dan {} tidak terurut",
                w[0],
                w[1]
            );
        }
        assert_eq!(priority_key(f64::NAN), u64::MAX);
    }

    #[test]
    fn labirin_selalu_bisa_diselesaikan() {
        for seed in 0..12u64 {
            let g = generate_maze(21, 21, seed).unwrap();
            let r = search(
                &g,
                Point::new(0, 0),
                Point::new(20, 20),
                SearchOptions {
                    algorithm: Algorithm::AStar,
                    max_expansions: 100_000,
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(r.found, "labirin benih {seed} tidak bisa diselesaikan");
        }
    }

    #[test]
    fn labirin_deterministik() {
        let a = generate_maze(15, 15, 7).unwrap();
        let b = generate_maze(15, 15, 7).unwrap();
        assert_eq!(a.walls, b.walls);
        let c = generate_maze(15, 15, 8).unwrap();
        assert_ne!(a.walls, c.walls);
    }

    #[test]
    fn labirin_menolak_ukuran_tak_sah() {
        assert!(generate_maze(0, 5, 1).is_err());
        assert!(generate_maze(MAX_DIMENSION + 1, 5, 1).is_err());
    }

    #[test]
    fn sifat_algoritma_terlaporkan_benar() {
        assert!(Algorithm::AStar.is_optimal());
        assert!(Algorithm::BreadthFirst.is_optimal());
        assert!(!Algorithm::DepthFirst.is_optimal());
        assert!(!Algorithm::HillClimbing.is_optimal());
        assert!(Algorithm::AStar.uses_heuristic());
        assert!(!Algorithm::BreadthFirst.uses_heuristic());
        for a in semua_algoritma() {
            assert!(!a.short_name().is_empty());
        }
    }

    #[test]
    fn hasil_bisa_di_serialisasi() {
        let g = Grid::new(7, 7).unwrap();
        let r = search(
            &g,
            Point::new(0, 0),
            Point::new(6, 6),
            SearchOptions::default(),
        )
        .unwrap();
        let json = serde_json::to_string(&r).unwrap();
        let balik: SearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(balik.path, r.path);
        assert_eq!(balik.found, r.found);

        let gj = serde_json::to_string(&g).unwrap();
        assert_eq!(serde_json::from_str::<Grid>(&gj).unwrap(), g);
    }

    #[test]
    fn pesan_error_terbaca() {
        for e in [
            SearchError::BadGrid {
                width: 0,
                height: 0,
            },
            SearchError::OutOfBounds { x: 1, y: 2 },
            SearchError::BlockedEndpoint("tujuan"),
            SearchError::WallLengthMismatch {
                expected: 9,
                got: 8,
            },
        ] {
            assert!(!e.to_string().is_empty());
        }
    }
}
