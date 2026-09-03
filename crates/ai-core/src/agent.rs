//! Sesi 2 — Agen Kecerdasan, Masalah, dan Ruang Keadaan.
//!
//! Empat jenis agen dijalankan pada lingkungan yang sama supaya perbedaannya
//! terlihat sebagai angka, bukan sebagai definisi:
//!
//! | Agen | Yang dipakai | Kelemahan yang terlihat |
//! |------|--------------|-------------------------|
//! | Refleks sederhana | Hanya persepsi saat ini | Tidak tahu ruangan lain, jadi bergerak sia-sia |
//! | Refleks bermodel | Persepsi + ingatan | Berhenti setelah semua ruangan diketahui bersih |
//! | Berbasis tujuan | Ingatan + rencana | Menempuh jalur terpendek menuju keadaan tujuan |
//! | Berbasis utilitas | Rencana + nilai guna | Menimbang biaya gerak terhadap hasil |
//!
//! Ditambah dua masalah ruang keadaan klasik: teko air dan misionaris-kanibal.
//! Keduanya kecil, tetapi cukup untuk memperlihatkan bahwa merumuskan masalah
//! sebagai ruang keadaan adalah separuh dari pemecahannya.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

/// Kesalahan pada simulasi agen dan ruang keadaan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentError {
    /// Jumlah ruangan di luar batas yang wajar.
    BadRoomCount(usize),
    /// Posisi awal agen di luar jangkauan.
    StartOutOfRange {
        /// Posisi yang diminta.
        position: usize,
        /// Jumlah ruangan yang tersedia.
        rooms: usize,
    },
    /// Kapasitas teko tidak masuk akal.
    BadCapacity {
        /// Kapasitas teko pertama.
        a: usize,
        /// Kapasitas teko kedua.
        b: usize,
    },
    /// Sasaran mustahil dicapai dengan kapasitas yang ada.
    TargetExceedsLargestJug {
        /// Sasaran yang diminta.
        target: usize,
        /// Kapasitas teko terbesar.
        largest: usize,
    },
    /// Sasaran bukan kelipatan pembagi bersama terbesar kedua teko.
    ///
    /// Dipisah dari {@link AgentError::TargetExceedsLargestJug} alih-alih
    /// dibedakan lewat untai `reason`. Untai itu dulu berisi kalimat Bahasa
    /// Indonesia, dan kalimat yang disimpan di dalam galat hanya bisa punya
    /// satu bahasa.
    TargetNotMultipleOfGcd {
        /// Sasaran yang diminta.
        target: usize,
        /// Pembagi bersama terbesar kedua kapasitas.
        gcd: usize,
    },
    /// Seluruh ruang keadaan sudah ditelusuri tanpa menemukan sasaran.
    ///
    /// Tidak seharusnya tercapai: keterjangkauan sudah diperiksa di muka.
    SearchExhausted,
    /// Tidak ada urutan penyeberangan yang aman untuk rombongan itu.
    NoSafeCrossing,
    /// Jumlah misionaris atau kanibal di luar batas.
    BadPartySize(usize),
}

impl crate::galat::Dijelaskan for AgentError {
    fn kode(&self) -> &'static str {
        match self {
            AgentError::BadRoomCount(_) => "agen.jumlah_ruangan",
            AgentError::StartOutOfRange { .. } => "agen.posisi_awal",
            AgentError::BadCapacity { .. } => "agen.kapasitas_teko",
            AgentError::TargetExceedsLargestJug { .. } => "agen.sasaran_melebihi_teko",
            AgentError::TargetNotMultipleOfGcd { .. } => "agen.sasaran_bukan_kelipatan",
            AgentError::SearchExhausted => "agen.ruang_keadaan_habis",
            AgentError::NoSafeCrossing => "agen.penyeberangan_tak_aman",
            AgentError::BadPartySize(_) => "agen.jumlah_rombongan",
        }
    }

    fn argumen(&self) -> Vec<String> {
        match self {
            AgentError::BadRoomCount(n) => vec![MAX_ROOMS.to_string(), n.to_string()],
            AgentError::StartOutOfRange { position, rooms } => {
                vec![position.to_string(), rooms.to_string()]
            }
            AgentError::BadCapacity { a, b } => vec![a.to_string(), b.to_string()],
            AgentError::TargetExceedsLargestJug { target, largest } => {
                vec![target.to_string(), largest.to_string()]
            }
            AgentError::TargetNotMultipleOfGcd { target, gcd } => {
                vec![target.to_string(), gcd.to_string()]
            }
            AgentError::SearchExhausted | AgentError::NoSafeCrossing => Vec::new(),
            AgentError::BadPartySize(n) => vec![MAX_PARTY.to_string(), n.to_string()],
        }
    }
}

impl core::fmt::Display for AgentError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AgentError::BadRoomCount(n) => {
                write!(f, "jumlah ruangan harus 1 sampai {MAX_ROOMS}, diberi {n}")
            }
            AgentError::StartOutOfRange { position, rooms } => {
                write!(f, "posisi awal {position} di luar {rooms} ruangan")
            }
            AgentError::BadCapacity { a, b } => {
                write!(f, "kapasitas teko tidak sah: {a} dan {b}")
            }
            AgentError::TargetExceedsLargestJug { target, largest } => {
                write!(f, "sasaran {target} melebihi teko terbesar ({largest})")
            }
            AgentError::TargetNotMultipleOfGcd { target, gcd } => {
                write!(
                    f,
                    "sasaran {target} bukan kelipatan pembagi bersama terbesar ({gcd})"
                )
            }
            AgentError::SearchExhausted => write!(f, "ruang keadaan habis ditelusuri"),
            AgentError::NoSafeCrossing => {
                write!(f, "tidak ada urutan penyeberangan yang aman")
            }
            AgentError::BadPartySize(n) => {
                write!(f, "jumlah rombongan harus 1 sampai {MAX_PARTY}, diberi {n}")
            }
        }
    }
}

/// Batas jumlah ruangan pada dunia penyedot debu.
pub const MAX_ROOMS: usize = 20;

/// Batas jumlah misionaris atau kanibal.
pub const MAX_PARTY: usize = 8;

// ---------------------------------------------------------------------------
// Dunia penyedot debu
// ---------------------------------------------------------------------------

/// Tindakan yang bisa diambil agen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Membersihkan ruangan tempat agen berada.
    Suck,
    /// Bergerak ke ruangan di sebelah kiri.
    MoveLeft,
    /// Bergerak ke ruangan di sebelah kanan.
    MoveRight,
    /// Tidak melakukan apa pun.
    Idle,
}

impl Action {
    /// Nama pendek untuk ditampilkan.
    pub fn short_name(self) -> &'static str {
        match self {
            Action::Suck => "sedot",
            Action::MoveLeft => "kiri",
            Action::MoveRight => "kanan",
            Action::Idle => "diam",
        }
    }

    /// Biaya tindakan. Menyedot lebih mahal daripada bergerak, dan diam gratis.
    pub fn cost(self) -> f64 {
        match self {
            Action::Suck => 2.0,
            Action::MoveLeft | Action::MoveRight => 1.0,
            Action::Idle => 0.0,
        }
    }
}

/// Jenis agen yang tersedia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentKind {
    /// Hanya melihat persepsi saat ini, tanpa ingatan apa pun.
    SimpleReflex,
    /// Menyimpan peta ruangan yang sudah diketahui bersih.
    ModelBased,
    /// Menyusun rencana menuju keadaan seluruh ruangan bersih.
    GoalBased,
    /// Menimbang biaya tindakan terhadap manfaatnya.
    UtilityBased,
}

impl AgentKind {
    /// Nama pendek untuk ditampilkan.
    pub fn short_name(self) -> &'static str {
        match self {
            AgentKind::SimpleReflex => "Refleks sederhana",
            AgentKind::ModelBased => "Refleks bermodel",
            AgentKind::GoalBased => "Berbasis tujuan",
            AgentKind::UtilityBased => "Berbasis utilitas",
        }
    }

    /// Apakah agen ini menyimpan ingatan tentang lingkungannya.
    pub fn has_memory(self) -> bool {
        !matches!(self, AgentKind::SimpleReflex)
    }
}

/// Keadaan lingkungan penyedot debu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VacuumWorld {
    /// `true` berarti ruangan itu kotor.
    pub dirty: Vec<bool>,
    /// Ruangan tempat agen berada.
    pub position: usize,
}

impl VacuumWorld {
    /// Dunia baru dengan pola kekotoran tertentu.
    pub fn new(dirty: Vec<bool>, position: usize) -> Result<Self, AgentError> {
        if dirty.is_empty() || dirty.len() > MAX_ROOMS {
            return Err(AgentError::BadRoomCount(dirty.len()));
        }
        if position >= dirty.len() {
            return Err(AgentError::StartOutOfRange {
                position,
                rooms: dirty.len(),
            });
        }
        Ok(Self { dirty, position })
    }

    /// Jumlah ruangan.
    pub fn rooms(&self) -> usize {
        self.dirty.len()
    }

    /// Apakah seluruh ruangan sudah bersih.
    pub fn is_clean(&self) -> bool {
        self.dirty.iter().all(|d| !d)
    }

    /// Jumlah ruangan yang masih kotor.
    pub fn dirty_count(&self) -> usize {
        self.dirty.iter().filter(|d| **d).count()
    }

    /// Menerapkan sebuah tindakan.
    pub fn apply(&mut self, action: Action) {
        match action {
            Action::Suck => self.dirty[self.position] = false,
            Action::MoveLeft => self.position = self.position.saturating_sub(1),
            Action::MoveRight => {
                if self.position + 1 < self.dirty.len() {
                    self.position += 1;
                }
            }
            Action::Idle => {}
        }
    }
}

/// Satu langkah simulasi, dipakai untuk menampilkan jejak.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentStep {
    /// Nomor langkah, mulai dari satu.
    pub step: usize,
    /// Posisi agen sebelum bertindak.
    pub position: usize,
    /// Apakah ruangan itu kotor saat dipersepsi.
    pub perceived_dirty: bool,
    /// Tindakan yang diambil.
    pub action: Action,
    /// Jumlah ruangan kotor setelah tindakan.
    pub dirty_after: usize,
}

/// Hasil menjalankan seorang agen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentRun {
    /// Jenis agen yang dijalankan.
    pub kind: AgentKind,
    /// Jejak langkah demi langkah.
    pub steps: Vec<AgentStep>,
    /// Apakah seluruh ruangan berhasil dibersihkan.
    pub finished: bool,
    /// Total biaya tindakan.
    pub cost: f64,
    /// Jumlah langkah yang benar-benar diambil.
    pub actions_taken: usize,
    /// Berapa kali agen bergerak tanpa menghasilkan apa pun.
    pub wasted_moves: usize,
}

/// Menjalankan seorang agen pada dunia yang diberikan.
///
/// Agen tanpa ingatan bisa berjalan selamanya tanpa tahu kapan berhenti, jadi
/// jumlah langkah selalu dibatasi. Batas itu bukan detail teknis: ia justru
/// yang memperlihatkan kelemahan agen refleks sederhana.
pub fn run_agent(world: &VacuumWorld, kind: AgentKind, max_steps: usize) -> AgentRun {
    let mut state = world.clone();
    let mut steps = Vec::new();
    let mut cost = 0.0;
    let mut wasted = 0usize;

    // Ingatan agen bermodel: ruangan yang sudah diketahui bersih.
    let mut known_clean: BTreeSet<usize> = BTreeSet::new();
    // Arah gerak terakhir, dipakai agar agen tidak bolak-balik di tempat.
    let mut heading_right = true;

    for step in 1..=max_steps {
        // Tidak ada pemeriksaan "dunia sudah bersih" di sini, dan itu
        // disengaja. Memberitahu agen bahwa seluruh ruangan bersih berarti
        // memberinya pengetahuan yang justru tidak dimilikinya — persis
        // kemampuan yang membedakan agen bermodel dari agen refleks. Setiap
        // agen harus menyimpulkannya sendiri dari apa yang bisa ia persepsi.
        let perceived_dirty = state.dirty[state.position];
        if !perceived_dirty {
            known_clean.insert(state.position);
        } else {
            known_clean.remove(&state.position);
        }

        let action = match kind {
            AgentKind::SimpleReflex => {
                // Hanya tahu ruangan tempatnya berdiri. Kalau bersih, ia
                // bergerak — tanpa cara apa pun untuk tahu bahwa seluruh
                // ruangan sebenarnya sudah bersih.
                if perceived_dirty {
                    Action::Suck
                } else if state.position + 1 < state.rooms() {
                    Action::MoveRight
                } else {
                    Action::MoveLeft
                }
            }
            AgentKind::ModelBased => {
                if perceived_dirty {
                    Action::Suck
                } else if known_clean.len() >= state.rooms() {
                    // Seluruh ruangan sudah pernah dilihat bersih.
                    Action::Idle
                } else {
                    // Menyapu ke satu arah sampai ujung, lalu berbalik.
                    if heading_right && state.position + 1 < state.rooms() {
                        Action::MoveRight
                    } else if !heading_right && state.position > 0 {
                        Action::MoveLeft
                    } else {
                        heading_right = !heading_right;
                        if heading_right {
                            Action::MoveRight
                        } else {
                            Action::MoveLeft
                        }
                    }
                }
            }
            AgentKind::GoalBased | AgentKind::UtilityBased => {
                if perceived_dirty {
                    Action::Suck
                } else {
                    // Bergerak menuju ruangan kotor terdekat. Agen ini
                    // dianggap punya peta lingkungannya, bukan hanya persepsi.
                    match nearest_dirty(&state) {
                        Some(target) if target > state.position => Action::MoveRight,
                        Some(target) if target < state.position => Action::MoveLeft,
                        Some(_) => Action::Suck,
                        None => Action::Idle,
                    }
                }
            }
        };

        // Agen berbasis utilitas berhenti bila biaya menuju sisa kotoran
        // melebihi manfaatnya. Inilah satu-satunya agen di sini yang boleh
        // memutuskan bahwa membersihkan tidak sepadan.
        let action = if kind == AgentKind::UtilityBased && !perceived_dirty {
            match nearest_dirty(&state) {
                Some(target) => {
                    let travel = target.abs_diff(state.position) as f64;
                    if travel * Action::MoveRight.cost() + Action::Suck.cost() > UTILITY_BUDGET {
                        Action::Idle
                    } else {
                        action
                    }
                }
                None => Action::Idle,
            }
        } else {
            action
        };

        if action == Action::Idle {
            steps.push(AgentStep {
                step,
                position: state.position,
                perceived_dirty,
                action,
                dirty_after: state.dirty_count(),
            });
            break;
        }

        // Posisi direkam sebelum tindakan diterapkan, karena itulah tempat
        // agen berdiri ketika ia mengambil keputusan.
        let position_before = state.position;
        // Jarak ke kotoran terdekat sebelum bergerak. Inilah ukuran yang benar
        // untuk "gerak sia-sia": gerak menuju kotoran tidak mengurangi jumlah
        // kotoran, tetapi jelas bukan pemborosan. Yang sia-sia adalah gerak
        // yang tidak mendekatkan agen kepada apa pun.
        let distance_before = nearest_dirty(&state).map(|t| t.abs_diff(state.position));

        state.apply(action);
        cost += action.cost();

        if action != Action::Suck {
            let distance_after = nearest_dirty(&state).map(|t| t.abs_diff(state.position));
            let mendekat = match (distance_before, distance_after) {
                (Some(before), Some(after)) => after < before,
                // Tidak ada kotoran tersisa: gerak apa pun tidak berguna.
                (None, _) => false,
                (Some(_), None) => true,
            };
            if !mendekat {
                wasted += 1;
            }
        }

        steps.push(AgentStep {
            step,
            position: position_before,
            perceived_dirty,
            action,
            dirty_after: state.dirty_count(),
        });
    }

    let actions_taken = steps.iter().filter(|s| s.action != Action::Idle).count();
    AgentRun {
        kind,
        finished: state.is_clean(),
        cost,
        actions_taken,
        wasted_moves: wasted,
        steps,
    }
}

/// Anggaran biaya yang dianggap sepadan oleh agen berbasis utilitas.
pub const UTILITY_BUDGET: f64 = 12.0;

/// Ruangan kotor terdekat dari posisi agen.
fn nearest_dirty(world: &VacuumWorld) -> Option<usize> {
    world
        .dirty
        .iter()
        .enumerate()
        .filter(|(_, d)| **d)
        .min_by_key(|(i, _)| (i.abs_diff(world.position), *i))
        .map(|(i, _)| i)
}

/// Menjalankan seluruh jenis agen pada dunia yang sama untuk dibandingkan.
pub fn compare_agents(world: &VacuumWorld, max_steps: usize) -> Vec<AgentRun> {
    [
        AgentKind::SimpleReflex,
        AgentKind::ModelBased,
        AgentKind::GoalBased,
        AgentKind::UtilityBased,
    ]
    .into_iter()
    .map(|kind| run_agent(world, kind, max_steps))
    .collect()
}

// ---------------------------------------------------------------------------
// Masalah teko air
// ---------------------------------------------------------------------------

/// Keadaan dua teko air: isi masing-masing.
pub type JugState = (usize, usize);

/// Satu langkah penyelesaian teko air.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JugStep {
    /// Keterangan tindakan.
    pub action: String,
    /// Isi teko pertama setelah tindakan.
    pub a: usize,
    /// Isi teko kedua setelah tindakan.
    pub b: usize,
}

/// Pembagi bersama terbesar, dipakai memeriksa keterjangkauan sasaran.
fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Menyelesaikan masalah teko air dengan pencarian melebar.
///
/// Keterjangkauan diperiksa lebih dulu memakai teorema Bézout: sasaran hanya
/// bisa dicapai bila ia kelipatan pembagi bersama terbesar kedua kapasitas,
/// dan tidak melebihi teko terbesar. Memeriksanya di muka jauh lebih jujur
/// daripada membiarkan pencarian berjalan sampai kehabisan keadaan lalu
/// melaporkan "tidak ditemukan" — dua hal itu terlihat sama di layar tetapi
/// berbeda maknanya.
pub fn solve_water_jug(
    capacity_a: usize,
    capacity_b: usize,
    target: usize,
) -> Result<Vec<JugStep>, AgentError> {
    if capacity_a == 0 || capacity_b == 0 || capacity_a > 100 || capacity_b > 100 {
        return Err(AgentError::BadCapacity {
            a: capacity_a,
            b: capacity_b,
        });
    }
    if target > capacity_a.max(capacity_b) {
        return Err(AgentError::TargetExceedsLargestJug {
            target,
            largest: capacity_a.max(capacity_b),
        });
    }
    let d = gcd(capacity_a, capacity_b);
    if target % d != 0 {
        return Err(AgentError::TargetNotMultipleOfGcd { target, gcd: d });
    }

    let start: JugState = (0, 0);
    let mut queue = VecDeque::from([start]);
    let mut came_from: HashMap<JugState, (JugState, String)> = HashMap::new();
    let mut seen: HashSet<JugState> = HashSet::from([start]);

    while let Some(current) = queue.pop_front() {
        if current.0 == target || current.1 == target {
            // Menyusun ulang jalur dari peta pendahulu.
            let mut steps = Vec::new();
            let mut node = current;
            while let Some((prev, action)) = came_from.get(&node) {
                steps.push(JugStep {
                    action: action.clone(),
                    a: node.0,
                    b: node.1,
                });
                node = *prev;
            }
            steps.reverse();
            return Ok(steps);
        }

        let (a, b) = current;
        let pour_ab = a.min(capacity_b - b);
        let pour_ba = b.min(capacity_a - a);
        let moves: [(JugState, String); 6] = [
            (
                (capacity_a, b),
                format!("isi penuh teko A ({capacity_a} liter)"),
            ),
            (
                (a, capacity_b),
                format!("isi penuh teko B ({capacity_b} liter)"),
            ),
            ((0, b), "kosongkan teko A".to_string()),
            ((a, 0), "kosongkan teko B".to_string()),
            (
                (a - pour_ab, b + pour_ab),
                format!("tuang {pour_ab} liter dari A ke B"),
            ),
            (
                (a + pour_ba, b - pour_ba),
                format!("tuang {pour_ba} liter dari B ke A"),
            ),
        ];

        for (next, action) in moves {
            if next != current && seen.insert(next) {
                came_from.insert(next, (current, action));
                queue.push_back(next);
            }
        }
    }

    // Tidak seharusnya tercapai karena keterjangkauan sudah diperiksa di muka.
    Err(AgentError::SearchExhausted)
}

// ---------------------------------------------------------------------------
// Misionaris dan kanibal
// ---------------------------------------------------------------------------

/// Keadaan: misionaris di tepi kiri, kanibal di tepi kiri, perahu di kiri.
pub type MissionaryState = (usize, usize, bool);

/// Satu langkah penyeberangan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossingStep {
    /// Keterangan tindakan.
    pub action: String,
    /// Misionaris di tepi kiri setelah tindakan.
    pub missionaries_left: usize,
    /// Kanibal di tepi kiri setelah tindakan.
    pub cannibals_left: usize,
    /// Apakah perahu berada di tepi kiri.
    pub boat_left: bool,
}

/// Apakah sebuah keadaan aman, yaitu kanibal tidak melebihi misionaris di
/// tepi mana pun yang masih ada misionarisnya.
pub fn is_safe(state: MissionaryState, total_missionaries: usize, total_cannibals: usize) -> bool {
    let (ml, cl, _) = state;
    if ml > total_missionaries || cl > total_cannibals {
        return false;
    }
    let mr = total_missionaries - ml;
    let cr = total_cannibals - cl;
    // Aturannya bukan "kanibal selalu lebih sedikit", melainkan "tidak boleh
    // lebih banyak dari misionaris yang hadir". Tepi tanpa misionaris selalu
    // aman berapa pun kanibalnya.
    (ml == 0 || ml >= cl) && (mr == 0 || mr >= cr)
}

/// Menyelesaikan masalah misionaris dan kanibal dengan pencarian melebar.
pub fn solve_missionaries(
    missionaries: usize,
    cannibals: usize,
    boat_capacity: usize,
) -> Result<Vec<CrossingStep>, AgentError> {
    if missionaries == 0 || missionaries > MAX_PARTY {
        return Err(AgentError::BadPartySize(missionaries));
    }
    if cannibals == 0 || cannibals > MAX_PARTY {
        return Err(AgentError::BadPartySize(cannibals));
    }
    if !(2..=MAX_PARTY).contains(&boat_capacity) {
        return Err(AgentError::BadPartySize(boat_capacity));
    }

    let start: MissionaryState = (missionaries, cannibals, true);
    let goal: MissionaryState = (0, 0, false);
    let mut queue = VecDeque::from([start]);
    let mut came_from: HashMap<MissionaryState, (MissionaryState, String)> = HashMap::new();
    let mut seen: HashSet<MissionaryState> = HashSet::from([start]);

    while let Some(current) = queue.pop_front() {
        if current == goal {
            let mut steps = Vec::new();
            let mut node = current;
            while let Some((prev, action)) = came_from.get(&node) {
                steps.push(CrossingStep {
                    action: action.clone(),
                    missionaries_left: node.0,
                    cannibals_left: node.1,
                    boat_left: node.2,
                });
                node = *prev;
            }
            steps.reverse();
            return Ok(steps);
        }

        let (ml, cl, boat) = current;
        for m in 0..=boat_capacity {
            for c in 0..=(boat_capacity - m) {
                if m + c == 0 || m + c > boat_capacity {
                    continue;
                }
                // Perahu tidak boleh membawa kanibal lebih banyak daripada
                // misionaris di dalamnya, kecuali tanpa misionaris sama sekali.
                if m > 0 && c > m {
                    continue;
                }
                let next = if boat {
                    if m > ml || c > cl {
                        continue;
                    }
                    (ml - m, cl - c, false)
                } else {
                    let mr = missionaries - ml;
                    let cr = cannibals - cl;
                    if m > mr || c > cr {
                        continue;
                    }
                    (ml + m, cl + c, true)
                };

                if !is_safe(next, missionaries, cannibals) {
                    continue;
                }
                if seen.insert(next) {
                    let arah = if boat { "ke seberang" } else { "kembali" };
                    came_from.insert(
                        next,
                        (current, format!("{arah}: {m} misionaris, {c} kanibal")),
                    );
                    queue.push_back(next);
                }
            }
        }
    }

    Err(AgentError::NoSafeCrossing)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    // ------------------------------------------------------- dunia penyedot

    #[test]
    fn dunia_baru_menolak_bentuk_tak_sah() {
        assert!(matches!(
            VacuumWorld::new(vec![], 0),
            Err(AgentError::BadRoomCount(0))
        ));
        assert!(matches!(
            VacuumWorld::new(vec![true; MAX_ROOMS + 1], 0),
            Err(AgentError::BadRoomCount(_))
        ));
        assert!(matches!(
            VacuumWorld::new(vec![true, false], 5),
            Err(AgentError::StartOutOfRange { .. })
        ));
    }

    #[test]
    fn tindakan_mengubah_keadaan() {
        let mut w = VacuumWorld::new(vec![true, true, true], 1).unwrap();
        assert_eq!(w.dirty_count(), 3);
        w.apply(Action::Suck);
        assert_eq!(w.dirty_count(), 2);
        w.apply(Action::MoveRight);
        assert_eq!(w.position, 2);
        w.apply(Action::MoveRight);
        assert_eq!(w.position, 2, "tidak boleh keluar dari ruangan terakhir");
        w.apply(Action::MoveLeft);
        assert_eq!(w.position, 1);
    }

    #[test]
    fn gerak_di_tepi_tidak_meluap() {
        let mut w = VacuumWorld::new(vec![false, false], 0).unwrap();
        w.apply(Action::MoveLeft);
        assert_eq!(w.position, 0, "posisi tidak boleh menjadi negatif");
    }

    #[test]
    fn biaya_tindakan() {
        close(Action::Suck.cost(), 2.0);
        close(Action::MoveLeft.cost(), 1.0);
        close(Action::Idle.cost(), 0.0);
        for a in [
            Action::Suck,
            Action::MoveLeft,
            Action::MoveRight,
            Action::Idle,
        ] {
            assert!(!a.short_name().is_empty());
        }
    }

    #[test]
    fn semua_agen_membersihkan_dunia_kecil() {
        let w = VacuumWorld::new(vec![true, true, true], 0).unwrap();
        for run in compare_agents(&w, 200) {
            assert!(run.finished, "{} gagal membersihkan", run.kind.short_name());
        }
    }

    #[test]
    fn agen_refleks_sederhana_tidak_tahu_kapan_berhenti() {
        // Inti perbedaan antaragen: tanpa ingatan, agen tidak punya cara
        // mengetahui bahwa seluruh ruangan sudah bersih, sehingga ia terus
        // bergerak sampai batas langkah tercapai.
        let w = VacuumWorld::new(vec![true, false, false, false], 0).unwrap();
        let sederhana = run_agent(&w, AgentKind::SimpleReflex, 50);
        let bermodel = run_agent(&w, AgentKind::ModelBased, 50);

        assert!(sederhana.finished);
        assert!(bermodel.finished);
        assert!(
            sederhana.steps.len() > bermodel.steps.len(),
            "sederhana {} langkah, bermodel {}",
            sederhana.steps.len(),
            bermodel.steps.len()
        );
        assert!(sederhana.wasted_moves > 0, "seharusnya ada gerak sia-sia");
    }

    #[test]
    fn agen_bermodel_berhenti_setelah_semua_diketahui_bersih() {
        // Agen bermodel tidak diberi tahu bahwa dunianya bersih; ia harus
        // menyusuri seluruh ruangan lebih dulu, lalu menyimpulkannya sendiri.
        let w = VacuumWorld::new(vec![false, false, false], 0).unwrap();
        let run = run_agent(&w, AgentKind::ModelBased, 100);
        assert!(run.finished);
        assert!(
            run.steps.len() <= 8,
            "berhenti terlalu lama: {} langkah",
            run.steps.len()
        );
        assert!(
            run.steps.iter().any(|s| s.action == Action::Idle),
            "agen bermodel harus berhenti sendiri, jejak: {:?}",
            run.steps
        );
    }

    #[test]
    fn agen_refleks_tidak_pernah_berhenti_sendiri() {
        // Sisi lain dari uji di atas: tanpa ingatan, agen tidak punya cara
        // menyimpulkan bahwa pekerjaannya selesai, jadi ia berjalan sampai
        // batas langkah. Inilah alasan agen bermodel ada.
        let w = VacuumWorld::new(vec![false, false, false], 0).unwrap();
        let run = run_agent(&w, AgentKind::SimpleReflex, 30);
        assert_eq!(run.steps.len(), 30, "seharusnya berjalan sampai batas");
        assert!(!run.steps.iter().any(|s| s.action == Action::Idle));
    }

    #[test]
    fn agen_berbasis_tujuan_langsung_menuju_kotoran() {
        // Kotoran hanya di ujung kanan; agen bertujuan tidak boleh menyusuri
        // ruangan bersih dua kali.
        let w = VacuumWorld::new(vec![false, false, false, false, true], 0).unwrap();
        let tujuan = run_agent(&w, AgentKind::GoalBased, 100);
        assert!(tujuan.finished);
        // Empat gerak ke kanan, satu sedot, lalu satu langkah diam.
        assert_eq!(tujuan.actions_taken, 5);
        close(tujuan.cost, 4.0 * 1.0 + 2.0);
        assert_eq!(
            tujuan.wasted_moves, 0,
            "setiap gerak mendekatkan agen ke kotoran, jadi tidak ada yang sia-sia"
        );
    }

    #[test]
    fn gerak_sia_sia_diukur_dari_jaraknya_ke_kotoran() {
        // Gerak menuju kotoran tidak mengurangi jumlah kotoran, tetapi jelas
        // bukan pemborosan. Yang sia-sia adalah gerak yang tidak mendekatkan
        // agen kepada apa pun — dan itulah yang dilakukan agen refleks pada
        // dunia yang sudah bersih.
        let bersih = VacuumWorld::new(vec![false, false, false], 0).unwrap();
        let refleks = run_agent(&bersih, AgentKind::SimpleReflex, 10);
        assert_eq!(
            refleks.wasted_moves, 10,
            "seluruh geraknya sia-sia karena tidak ada kotoran"
        );

        let ada_kotoran = VacuumWorld::new(vec![false, false, true], 0).unwrap();
        let tujuan = run_agent(&ada_kotoran, AgentKind::GoalBased, 20);
        assert_eq!(tujuan.wasted_moves, 0);
    }

    #[test]
    fn agen_berbasis_tujuan_lebih_murah_daripada_refleks() {
        let w = VacuumWorld::new(vec![false, false, false, false, true], 0).unwrap();
        let refleks = run_agent(&w, AgentKind::SimpleReflex, 200);
        let tujuan = run_agent(&w, AgentKind::GoalBased, 200);
        assert!(
            tujuan.cost <= refleks.cost,
            "tujuan {} vs refleks {}",
            tujuan.cost,
            refleks.cost
        );
    }

    #[test]
    fn agen_utilitas_menolak_perjalanan_yang_tidak_sepadan() {
        // Kotoran terlalu jauh: biaya menempuhnya melebihi anggaran, sehingga
        // agen berbasis utilitas memilih berhenti. Ini bukan kegagalan,
        // melainkan keputusan — dan hanya agen jenis ini yang boleh mengambilnya.
        let mut dirty = vec![false; 15];
        dirty[14] = true;
        let w = VacuumWorld::new(dirty, 0).unwrap();
        let utilitas = run_agent(&w, AgentKind::UtilityBased, 200);
        let tujuan = run_agent(&w, AgentKind::GoalBased, 200);

        assert!(!utilitas.finished, "seharusnya memilih tidak membersihkan");
        assert!(tujuan.finished, "agen bertujuan tetap menempuhnya");
        assert!(utilitas.cost < tujuan.cost);
        assert!(utilitas.steps.iter().any(|s| s.action == Action::Idle));
    }

    #[test]
    fn jejak_agen_terekam_lengkap() {
        let w = VacuumWorld::new(vec![true, true], 0).unwrap();
        let run = run_agent(&w, AgentKind::GoalBased, 50);
        assert!(!run.steps.is_empty());
        for (i, s) in run.steps.iter().enumerate() {
            assert_eq!(s.step, i + 1);
            assert!(s.dirty_after <= 2);
        }
        // Jumlah kotoran tidak pernah bertambah.
        for w in run.steps.windows(2) {
            assert!(w[1].dirty_after <= w[0].dirty_after);
        }
    }

    #[test]
    fn perbandingan_memuat_empat_agen() {
        let w = VacuumWorld::new(vec![true, false, true], 1).unwrap();
        let hasil = compare_agents(&w, 100);
        assert_eq!(hasil.len(), 4);
        assert!(!hasil[0].kind.has_memory());
        assert!(hasil[1].kind.has_memory());
        for run in &hasil {
            assert!(!run.kind.short_name().is_empty());
        }
    }

    #[test]
    fn dunia_sudah_bersih_tidak_menghasilkan_kerja() {
        // Agen bertujuan punya peta, jadi ia langsung tahu tidak ada yang
        // perlu dikerjakan dan berhenti pada langkah pertama.
        let w = VacuumWorld::new(vec![false, false], 0).unwrap();
        let run = run_agent(&w, AgentKind::GoalBased, 50);
        assert!(run.finished);
        assert_eq!(run.actions_taken, 0);
        assert_eq!(run.steps.len(), 1);
        assert_eq!(run.steps[0].action, Action::Idle);
        close(run.cost, 0.0);
    }

    // ---------------------------------------------------------- teko air

    #[test]
    fn teko_air_kasus_klasik() {
        // Empat liter dari teko tiga dan lima liter.
        let langkah = solve_water_jug(3, 5, 4).unwrap();
        assert!(!langkah.is_empty());
        let akhir = langkah.last().unwrap();
        assert!(akhir.a == 4 || akhir.b == 4);
        // Solusi terpendek untuk kasus ini enam langkah.
        assert_eq!(langkah.len(), 6, "jejak: {langkah:?}");
    }

    #[test]
    fn teko_air_setiap_langkah_sah() {
        let langkah = solve_water_jug(3, 5, 4).unwrap();
        for s in &langkah {
            assert!(s.a <= 3, "teko A meluap: {}", s.a);
            assert!(s.b <= 5, "teko B meluap: {}", s.b);
            assert!(!s.action.is_empty());
        }
    }

    #[test]
    fn teko_air_menolak_sasaran_mustahil() {
        // Dengan teko genap, sasaran ganjil mustahil dicapai. Sebabnya
        // dibedakan, bukan disatukan: keduanya mustahil karena alasan yang
        // berbeda, dan yang membacanya berhak tahu yang mana.
        assert!(matches!(
            solve_water_jug(2, 4, 3),
            Err(AgentError::TargetNotMultipleOfGcd { target: 3, gcd: 2 })
        ));
        // Melebihi teko terbesar juga mustahil.
        assert!(matches!(
            solve_water_jug(3, 5, 9),
            Err(AgentError::TargetExceedsLargestJug {
                target: 9,
                largest: 5
            })
        ));
    }

    #[test]
    fn teko_air_menolak_kapasitas_tak_sah() {
        assert!(matches!(
            solve_water_jug(0, 5, 3),
            Err(AgentError::BadCapacity { .. })
        ));
        assert!(matches!(
            solve_water_jug(3, 0, 3),
            Err(AgentError::BadCapacity { .. })
        ));
        assert!(matches!(
            solve_water_jug(300, 5, 3),
            Err(AgentError::BadCapacity { .. })
        ));
    }

    #[test]
    fn teko_air_sasaran_nol_langsung_tercapai() {
        assert!(solve_water_jug(3, 5, 0).unwrap().is_empty());
    }

    #[test]
    fn pembagi_bersama_terbesar() {
        assert_eq!(gcd(3, 5), 1);
        assert_eq!(gcd(4, 6), 2);
        assert_eq!(gcd(12, 0), 12);
        assert_eq!(gcd(0, 7), 7);
    }

    // ------------------------------------------------ misionaris dan kanibal

    #[test]
    fn keamanan_keadaan() {
        // Tiga misionaris dan tiga kanibal.
        assert!(is_safe((3, 3, true), 3, 3));
        assert!(is_safe((0, 3, false), 3, 3), "tepi tanpa misionaris aman");
        assert!(is_safe((3, 0, true), 3, 3));
        assert!(!is_safe((1, 2, true), 3, 3), "kanibal lebih banyak di kiri");
        assert!(
            !is_safe((2, 1, true), 3, 3),
            "kanibal lebih banyak di kanan"
        );
    }

    #[test]
    fn misionaris_kasus_klasik() {
        // Tiga banding tiga dengan perahu berkapasitas dua: sebelas penyeberangan.
        let langkah = solve_missionaries(3, 3, 2).unwrap();
        assert_eq!(langkah.len(), 11, "jejak: {langkah:?}");
        let akhir = langkah.last().unwrap();
        assert_eq!(akhir.missionaries_left, 0);
        assert_eq!(akhir.cannibals_left, 0);
        assert!(!akhir.boat_left);
    }

    #[test]
    fn misionaris_setiap_keadaan_aman() {
        let langkah = solve_missionaries(3, 3, 2).unwrap();
        for s in &langkah {
            assert!(
                is_safe((s.missionaries_left, s.cannibals_left, s.boat_left), 3, 3),
                "keadaan tidak aman: {s:?}"
            );
        }
    }

    #[test]
    fn misionaris_perahu_lebih_besar_mempersingkat() {
        let dua = solve_missionaries(3, 3, 2).unwrap().len();
        let tiga = solve_missionaries(3, 3, 3).unwrap().len();
        assert!(tiga < dua, "perahu {tiga} vs {dua}");
    }

    #[test]
    fn misionaris_kasus_yang_mustahil() {
        // Empat banding empat dengan perahu dua tidak punya solusi aman.
        assert!(matches!(
            solve_missionaries(4, 4, 2),
            Err(AgentError::NoSafeCrossing)
        ));
    }

    #[test]
    fn misionaris_menolak_ukuran_tak_sah() {
        assert!(matches!(
            solve_missionaries(0, 3, 2),
            Err(AgentError::BadPartySize(0))
        ));
        assert!(matches!(
            solve_missionaries(3, 0, 2),
            Err(AgentError::BadPartySize(0))
        ));
        assert!(matches!(
            solve_missionaries(3, 3, 1),
            Err(AgentError::BadPartySize(1))
        ));
        assert!(matches!(
            solve_missionaries(MAX_PARTY + 1, 3, 2),
            Err(AgentError::BadPartySize(_))
        ));
    }

    #[test]
    fn hasil_bisa_di_serialisasi() {
        let w = VacuumWorld::new(vec![true, false], 0).unwrap();
        let run = run_agent(&w, AgentKind::GoalBased, 20);
        let json = serde_json::to_string(&run).unwrap();
        assert_eq!(serde_json::from_str::<AgentRun>(&json).unwrap(), run);

        let jug = solve_water_jug(3, 5, 4).unwrap();
        let jj = serde_json::to_string(&jug).unwrap();
        assert_eq!(serde_json::from_str::<Vec<JugStep>>(&jj).unwrap(), jug);
    }

    #[test]
    fn pesan_error_terbaca() {
        for e in [
            AgentError::BadRoomCount(0),
            AgentError::StartOutOfRange {
                position: 5,
                rooms: 2,
            },
            AgentError::BadCapacity { a: 0, b: 5 },
            AgentError::TargetExceedsLargestJug {
                target: 3,
                largest: 2,
            },
            AgentError::TargetNotMultipleOfGcd { target: 3, gcd: 2 },
            AgentError::SearchExhausted,
            AgentError::NoSafeCrossing,
            AgentError::BadPartySize(0),
        ] {
            assert!(!e.to_string().is_empty());
        }
    }
}
