//! Jembatan WebAssembly untuk [`ai_core`].
//!
//! Semua data melintasi batas Rust/JavaScript sebagai string JSON. Pilihan ini
//! disengaja: satu format, satu jalur kesalahan, dan payload di aplikasi ini
//! kecil sehingga biaya serialisasinya tidak terasa. Setiap fungsi
//! mengembalikan JSON dengan bentuk `{"ok": <hasil>}` atau `{"err": "<pesan>"}`
//! agar sisi JavaScript tidak perlu menebak.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::Serialize;
use wasm_bindgen::prelude::*;

/// Membungkus hasil yang berhasil menjadi `{"ok": ...}`.
fn ok<T: Serialize>(value: T) -> String {
    #[derive(Serialize)]
    struct Ok<T> {
        ok: T,
    }
    serde_json::to_string(&Ok { ok: value })
        .unwrap_or_else(|e| format!(r#"{{"err":"gagal serialisasi: {e}"}}"#))
}

/// Membungkus kegagalan menjadi `{"err": "..."}`.
fn err<E: core::fmt::Display>(e: E) -> String {
    let msg = e.to_string().replace('"', "'");
    format!(r#"{{"err":"{msg}"}}"#)
}

/// Versi pustaka inti.
#[wasm_bindgen]
pub fn version() -> String {
    ai_core::VERSION.to_string()
}

/// Daftar sesi kuliah yang sudah terimplementasi, sebagai JSON.
#[wasm_bindgen]
pub fn sessions() -> String {
    #[derive(Serialize)]
    struct S {
        session: u8,
        module: &'static str,
        title_id: &'static str,
        title_en: &'static str,
    }
    let list: Vec<S> = ai_core::SESSIONS
        .iter()
        .map(|s| S {
            session: s.session,
            module: s.module,
            title_id: s.title_id,
            title_en: s.title_en,
        })
        .collect();
    ok(list)
}

// ---------------------------------------------------------------------------
// Sesi 3 — Certainty Factor
// ---------------------------------------------------------------------------

/// `CF = MB - MD`.
#[wasm_bindgen]
pub fn cf_from_mb_md(mb: f64, md: f64) -> String {
    match ai_core::certainty::cf_from_mb_md(mb, md) {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

/// Menggabungkan daftar CF secara paralel sambil merekam tiap langkah.
///
/// `cfs_json` adalah larik JSON berisi bilangan, mis. `"[0.5,0.3,0.2]"`.
#[wasm_bindgen]
pub fn cf_combine(cfs_json: &str) -> String {
    let cfs: Vec<f64> = match serde_json::from_str(cfs_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    match ai_core::certainty::combine_many_traced(&cfs) {
        Ok((value, steps)) => {
            #[derive(Serialize)]
            struct Out {
                value: f64,
                steps: Vec<ai_core::certainty::CfStep>,
                label_id: &'static str,
                label_en: &'static str,
            }
            let c = ai_core::certainty::interpret(value);
            ok(Out {
                value,
                steps,
                label_id: c.label_id(),
                label_en: c.label_en(),
            })
        }
        Err(e) => err(e),
    }
}

/// CF gabungan premis `AND` (minimum) atau `OR` (maksimum).
#[wasm_bindgen]
pub fn cf_premise(cfs_json: &str, operator: &str) -> String {
    let cfs: Vec<f64> = match serde_json::from_str(cfs_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let result = match operator.to_ascii_uppercase().as_str() {
        "AND" => ai_core::certainty::combine_and(&cfs),
        "OR" => ai_core::certainty::combine_or(&cfs),
        other => return err(format!("operator tidak dikenal: {other}")),
    };
    match result {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

/// CF berantai: CF aturan dikali CF bukti.
#[wasm_bindgen]
pub fn cf_sequential(cf_rule: f64, cf_evidence: f64) -> String {
    match ai_core::certainty::combine_sequential(cf_rule, cf_evidence) {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

// ---------------------------------------------------------------------------
// Sesi 4 — Bayesian
// ---------------------------------------------------------------------------

/// Kasus Bayes dua hipotesis lengkap dengan jejak langkahnya.
#[wasm_bindgen]
pub fn bayes_binary(prior: f64, likelihood_h: f64, likelihood_not_h: f64) -> String {
    match ai_core::bayes::binary_traced(prior, likelihood_h, likelihood_not_h) {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

/// Posterior seluruh hipotesis dari larik prior dan likelihood.
#[wasm_bindgen]
pub fn bayes_posterior_all(priors_json: &str, likelihoods_json: &str) -> String {
    let priors: Vec<f64> = match serde_json::from_str(priors_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let likelihoods: Vec<f64> = match serde_json::from_str(likelihoods_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    match ai_core::bayes::posterior_all(&priors, &likelihoods) {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

/// Melatih Naive Bayes kategorikal lalu langsung memprediksi satu baris.
///
/// `samples_json` berbentuk `[{"features":["Cerah","Panas"],"label":"Tidak"}]`,
/// `query_json` berbentuk `["Cerah","Panas"]`.
#[wasm_bindgen]
pub fn naive_bayes_predict(samples_json: &str, query_json: &str, alpha: f64) -> String {
    let samples: Vec<ai_core::bayes::CategoricalSample> = match serde_json::from_str(samples_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let query: Vec<String> = match serde_json::from_str(query_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let mut model = ai_core::bayes::CategoricalNaiveBayes::new(alpha);
    if let Err(e) = model.fit(&samples) {
        return err(e);
    }
    match model.predict(&query) {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

// ---------------------------------------------------------------------------
// Sesi 5 & 6 — Logika Fuzzy
// ---------------------------------------------------------------------------

/// Derajat keanggotaan sebuah nilai pada satu fungsi keanggotaan.
///
/// `set_json` memakai bentuk bertanda, mis.
/// `{"kind":"triangular","a":0,"b":5,"c":10}`.
#[wasm_bindgen]
pub fn fuzzy_degree(set_json: &str, x: f64) -> String {
    let set: ai_core::fuzzy::Membership = match serde_json::from_str(set_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if let Err(e) = set.validate() {
        return err(e);
    }
    ok(set.degree(x))
}

/// Kurva sebuah fungsi keanggotaan, tercuplik seragam pada semesta.
#[wasm_bindgen]
pub fn fuzzy_curve(set_json: &str, min: f64, max: f64, samples: usize) -> String {
    let set: ai_core::fuzzy::Membership = match serde_json::from_str(set_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if let Err(e) = set.validate() {
        return err(e);
    }
    match ai_core::fuzzy::sample_universe(min, max, samples) {
        Ok(xs) => {
            let ys: Vec<f64> = xs.iter().map(|x| set.degree(*x)).collect();
            #[derive(Serialize)]
            struct Curve {
                xs: Vec<f64>,
                ys: Vec<f64>,
            }
            ok(Curve { xs, ys })
        }
        Err(e) => err(e),
    }
}

/// Inferensi kabur lengkap.
///
/// `engine` menerima `"mamdani"`, `"sugeno"`, atau `"tsukamoto"`.
/// `method` hanya dipakai Mamdani: `"centroid"`, `"bisector"`,
/// `"mean_of_maximum"`, `"smallest_of_maximum"`, `"largest_of_maximum"`.
#[wasm_bindgen]
pub fn fuzzy_infer(
    system_json: &str,
    inputs_json: &str,
    engine: &str,
    method: &str,
    samples: usize,
) -> String {
    let system: ai_core::fuzzy::FuzzySystem = match serde_json::from_str(system_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let inputs: Vec<(String, f64)> = match serde_json::from_str(inputs_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let result = match engine.to_ascii_lowercase().as_str() {
        "mamdani" => {
            let m = match method.to_ascii_lowercase().as_str() {
                "centroid" => ai_core::fuzzy::Defuzzifier::Centroid,
                "bisector" => ai_core::fuzzy::Defuzzifier::Bisector,
                "mean_of_maximum" | "mom" => ai_core::fuzzy::Defuzzifier::MeanOfMaximum,
                "smallest_of_maximum" | "som" => ai_core::fuzzy::Defuzzifier::SmallestOfMaximum,
                "largest_of_maximum" | "lom" => ai_core::fuzzy::Defuzzifier::LargestOfMaximum,
                other => return err(format!("metode defuzzifikasi tidak dikenal: {other}")),
            };
            system.infer_mamdani(&inputs, m, samples)
        }
        "sugeno" => system.infer_sugeno(&inputs),
        "tsukamoto" => system.infer_tsukamoto(&inputs),
        other => return err(format!("mesin inferensi tidak dikenal: {other}")),
    };
    match result {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

// ---------------------------------------------------------------------------
// Sesi 8 — Teknik Pencarian
// ---------------------------------------------------------------------------

/// Menjalankan pencarian pada sebuah kisi.
///
/// `grid_json` berbentuk `{"width":21,"height":21,"walls":[...],"diagonal":false}`
/// dan `options_json` berbentuk
/// `{"algorithm":"a_star","heuristic":"manhattan","depth_limit":64,"seed":1,"max_expansions":100000}`.
#[wasm_bindgen]
pub fn search_run(
    grid_json: &str,
    start_x: usize,
    start_y: usize,
    goal_x: usize,
    goal_y: usize,
    options_json: &str,
) -> String {
    let grid: ai_core::search::Grid = match serde_json::from_str(grid_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let options: ai_core::search::SearchOptions = match serde_json::from_str(options_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    match ai_core::search::search(
        &grid,
        ai_core::search::Point::new(start_x, start_y),
        ai_core::search::Point::new(goal_x, goal_y),
        options,
    ) {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

/// Membangun labirin acak yang dijamin punya jalur keluar.
#[wasm_bindgen]
pub fn search_maze(width: usize, height: usize, seed: u64) -> String {
    match ai_core::search::generate_maze(width, height, seed) {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

/// Kisi kosong tanpa dinding.
#[wasm_bindgen]
pub fn search_empty_grid(width: usize, height: usize) -> String {
    match ai_core::search::Grid::new(width, height) {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

/// Menjalankan seluruh algoritma pada kisi yang sama untuk dibandingkan.
///
/// Ini bentuk yang paling mengajarkan sesuatu: jalur yang sama panjang bisa
/// menuntut jumlah simpul yang dibuka jauh berbeda.
#[wasm_bindgen]
pub fn search_compare(
    grid_json: &str,
    start_x: usize,
    start_y: usize,
    goal_x: usize,
    goal_y: usize,
    options_json: &str,
) -> String {
    use ai_core::search::Algorithm;

    let grid: ai_core::search::Grid = match serde_json::from_str(grid_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let base: ai_core::search::SearchOptions = match serde_json::from_str(options_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    #[derive(Serialize)]
    struct Row {
        algorithm: &'static str,
        name: &'static str,
        optimal: bool,
        uses_heuristic: bool,
        found: bool,
        cost: f64,
        steps: usize,
        expansions: usize,
        peak_frontier: usize,
    }

    let all = [
        (Algorithm::BreadthFirst, "breadth_first"),
        (Algorithm::DepthFirst, "depth_first"),
        (Algorithm::DepthLimited, "depth_limited"),
        (Algorithm::IterativeDeepening, "iterative_deepening"),
        (Algorithm::UniformCost, "uniform_cost"),
        (Algorithm::GreedyBestFirst, "greedy_best_first"),
        (Algorithm::AStar, "a_star"),
        (Algorithm::HillClimbing, "hill_climbing"),
        (Algorithm::SimulatedAnnealing, "simulated_annealing"),
    ];

    let start = ai_core::search::Point::new(start_x, start_y);
    let goal = ai_core::search::Point::new(goal_x, goal_y);
    let mut rows = Vec::with_capacity(all.len());
    for (algorithm, slug) in all {
        let options = ai_core::search::SearchOptions { algorithm, ..base };
        match ai_core::search::search(&grid, start, goal, options) {
            Ok(r) => rows.push(Row {
                algorithm: slug,
                name: algorithm.short_name(),
                optimal: algorithm.is_optimal(),
                uses_heuristic: algorithm.uses_heuristic(),
                found: r.found,
                // Biaya tak berhingga tidak punya padanan di JSON, jadi
                // kegagalan dilaporkan lewat medan `found`, bukan angka aneh.
                cost: if r.found { r.cost } else { -1.0 },
                steps: r.path.len().saturating_sub(1),
                expansions: r.expansions,
                peak_frontier: r.peak_frontier,
            }),
            Err(e) => return err(e),
        }
    }
    ok(rows)
}

// ---------------------------------------------------------------------------
// Sesi 9 — Jaringan Syaraf Tiruan
// ---------------------------------------------------------------------------

/// Membaca nama aktivasi.
fn parse_activation(name: &str) -> Result<ai_core::neural::Activation, String> {
    use ai_core::neural::Activation;
    Ok(match name.to_ascii_lowercase().as_str() {
        "step" => Activation::Step,
        "sigmoid" => Activation::Sigmoid,
        "tanh" => Activation::Tanh,
        "relu" => Activation::Relu,
        "leaky_relu" => Activation::LeakyRelu,
        "linear" => Activation::Linear,
        other => return Err(format!("aktivasi tidak dikenal: {other}")),
    })
}

/// Ringkasan jaringan beserta bobotnya, siap dikirim ke antarmuka.
#[derive(Serialize)]
struct NetworkSummary {
    network: ai_core::neural::Network,
    input_size: usize,
    output_size: usize,
    parameters: usize,
    effective_learning_rate: f64,
    step_risky: bool,
}

impl NetworkSummary {
    fn of(net: &ai_core::neural::Network) -> Self {
        Self {
            network: net.clone(),
            input_size: net.input_size(),
            output_size: net.output_size(),
            parameters: net.parameter_count(),
            effective_learning_rate: net.effective_learning_rate(),
            step_risky: net.is_step_risky(),
        }
    }
}

/// Membuat jaringan baru dan mengembalikannya sebagai JSON.
///
/// `sizes_json` berbentuk `[2,8,8,2]`: jumlah masukan, lalu ukuran tiap lapisan.
#[wasm_bindgen]
pub fn neural_create(
    sizes_json: &str,
    hidden_activation: &str,
    output_activation: &str,
    learning_rate: f64,
    momentum: f64,
    seed: u64,
) -> String {
    let sizes: Vec<usize> = match serde_json::from_str(sizes_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let hidden = match parse_activation(hidden_activation) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let output = match parse_activation(output_activation) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    match ai_core::neural::Network::new(&sizes, hidden, output, learning_rate, momentum, seed) {
        Ok(net) => ok(NetworkSummary::of(&net)),
        Err(e) => err(e),
    }
}

/// Kumpulan data siap latih.
#[wasm_bindgen]
pub fn neural_dataset(name: &str, points: usize, noise: f64, seed: u64) -> String {
    #[derive(Serialize)]
    struct Data {
        x: Vec<Vec<f64>>,
        y: Vec<Vec<f64>>,
    }
    let (x, y) = match name.to_ascii_lowercase().as_str() {
        "xor" => ai_core::neural::xor_dataset(),
        "and" => {
            let (x, y) = ai_core::neural::and_dataset();
            (x, y.into_iter().map(|v| vec![v]).collect())
        }
        "or" => {
            let (x, y) = ai_core::neural::or_dataset();
            (x, y.into_iter().map(|v| vec![v]).collect())
        }
        "spiral" => ai_core::neural::spiral_dataset(points, noise, seed),
        other => return err(format!("kumpulan data tidak dikenal: {other}")),
    };
    ok(Data { x, y })
}

/// Melatih jaringan sejumlah epoch lalu mengembalikan keadaan barunya.
///
/// Pelatihan dipecah menjadi potongan kecil oleh sisi antarmuka supaya utas
/// tampilan tidak membeku; fungsi ini hanya mengerjakan satu potongan.
#[wasm_bindgen]
pub fn neural_train(
    network_json: &str,
    x_json: &str,
    y_json: &str,
    epochs: usize,
    tolerance: f64,
    seed: u64,
) -> String {
    let mut net: ai_core::neural::Network = match serde_json::from_str(network_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let x: Vec<Vec<f64>> = match serde_json::from_str(x_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let y: Vec<Vec<f64>> = match serde_json::from_str(y_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    match net.train(&x, &y, epochs, tolerance, seed) {
        Ok(history) => {
            #[derive(Serialize)]
            struct Out {
                summary: NetworkSummary,
                history: Vec<ai_core::neural::EpochRecord>,
            }
            ok(Out {
                summary: NetworkSummary::of(&net),
                history,
            })
        }
        Err(e) => err(e),
    }
}

/// Keluaran jaringan pada kisi seragam, untuk menggambar batas keputusan.
///
/// Inilah bagian yang paling berat: sebuah kisi 100x100 berarti sepuluh ribu
/// perambatan maju. Dikerjakan di WebAssembly, ini selesai dalam hitungan
/// milidetik; di JavaScript murni, tampilan akan tersendat.
#[wasm_bindgen]
pub fn neural_decision_grid(network_json: &str, min: f64, max: f64, resolution: usize) -> String {
    let net: ai_core::neural::Network = match serde_json::from_str(network_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if net.input_size() != 2 {
        return err("batas keputusan hanya bisa digambar untuk dua masukan");
    }
    if !(2..=400).contains(&resolution) {
        return err(format!(
            "resolusi harus antara 2 dan 400, diberi {resolution}"
        ));
    }
    if !min.is_finite() || !max.is_finite() || min >= max {
        return err(format!("rentang tidak sah: {min} sampai {max}"));
    }

    let step = (max - min) / (resolution - 1) as f64;
    let mut values = Vec::with_capacity(resolution * resolution);
    for j in 0..resolution {
        for i in 0..resolution {
            let x = min + step * i as f64;
            let y = min + step * j as f64;
            match net.predict(&[x, y]) {
                Ok(out) => {
                    // Satu keluaran dibaca apa adanya; dua keluaran dinyatakan
                    // sebagai selisih, sehingga nilai tengah menandai batas.
                    let v = if out.len() == 1 {
                        out[0]
                    } else {
                        (out[0] - out[1] + 1.0) / 2.0
                    };
                    values.push(v);
                }
                Err(e) => return err(e),
            }
        }
    }
    #[derive(Serialize)]
    struct Out {
        resolution: usize,
        min: f64,
        max: f64,
        values: Vec<f64>,
    }
    ok(Out {
        resolution,
        min,
        max,
        values,
    })
}

// ---------------------------------------------------------------------------
// Sesi 11 — Sistem Pakar
// ---------------------------------------------------------------------------

/// Basis pengetahuan contoh: diagnosis flu dari studi kasus modul.
#[wasm_bindgen]
pub fn expert_sample_kb() -> String {
    ok(ai_core::expert::flu_knowledge_base())
}

/// Membaca basis pengetahuan lalu melaporkan kesehatannya.
///
/// Yang paling berguna di sini adalah daftar fakta yang dipakai sebagai premis
/// tetapi tidak bisa disimpulkan maupun ditanyakan. Fakta seperti itu diam-diam
/// dianggap tidak berlaku, sehingga sebagian aturan tidak akan pernah menyala
/// tanpa ada pesan galat apa pun.
#[wasm_bindgen]
pub fn expert_inspect_kb(kb_json: &str) -> String {
    let kb: ai_core::expert::KnowledgeBase = match serde_json::from_str(kb_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if let Err(e) = kb.validate() {
        return err(e);
    }
    #[derive(Serialize)]
    struct Out {
        name: String,
        rules: usize,
        derivable: Vec<String>,
        leaf_facts: Vec<String>,
        unreachable_facts: Vec<String>,
        askable: Vec<String>,
    }
    ok(Out {
        name: kb.name.clone(),
        rules: kb.rules.len(),
        derivable: kb.derivable().into_iter().collect(),
        leaf_facts: kb.leaf_facts().into_iter().collect(),
        unreachable_facts: kb.unreachable_facts().into_iter().collect(),
        askable: kb.askable.clone(),
    })
}

/// Membangun memori kerja dari daftar pasangan fakta dan keyakinan.
fn build_memory(facts_json: &str) -> Result<ai_core::expert::WorkingMemory, String> {
    let pairs: Vec<(String, f64)> = serde_json::from_str(facts_json).map_err(|e| e.to_string())?;
    let mut memory = ai_core::expert::WorkingMemory::new();
    for (fact, cf) in pairs {
        memory.assert(fact, cf).map_err(|e| e.to_string())?;
    }
    Ok(memory)
}

/// Penalaran runut maju.
///
/// `facts_json` berbentuk `[["demam",1.0],["batuk",0.8]]`.
#[wasm_bindgen]
pub fn expert_forward(kb_json: &str, facts_json: &str, threshold: f64) -> String {
    let kb: ai_core::expert::KnowledgeBase = match serde_json::from_str(kb_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let memory = match build_memory(facts_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    match ai_core::expert::forward_chain(&kb, &memory) {
        Ok(result) => {
            #[derive(Serialize)]
            struct Out {
                /// Hanya fakta yang benar-benar disimpulkan sistem.
                derived: Vec<(String, f64)>,
                /// Fakta yang berasal dari masukan pengguna.
                given: Vec<(String, f64)>,
                all_facts: Vec<(String, f64)>,
                steps: Vec<ai_core::expert::Step>,
                passes: usize,
            }
            // Memori kerja memuat masukan pengguna dan hasil penalaran
            // sekaligus. Menampilkan keduanya sebagai "kesimpulan" membuat
            // sistem terlihat menyimpulkan gejala yang justru diketikkan
            // penggunanya sendiri, jadi keduanya dipisahkan di sini.
            let derivable = kb.derivable();
            let semua = result.memory.conclusions(threshold);
            let (derived, given): (Vec<_>, Vec<_>) = semua
                .into_iter()
                .partition(|(fact, _)| derivable.contains(fact));
            ok(Out {
                derived,
                given,
                all_facts: result.memory.all(),
                steps: result.steps.clone(),
                passes: result.passes,
            })
        }
        Err(e) => err(e),
    }
}

/// Penalaran runut mundur terhadap sebuah tujuan.
#[wasm_bindgen]
pub fn expert_backward(kb_json: &str, facts_json: &str, goal: &str) -> String {
    let kb: ai_core::expert::KnowledgeBase = match serde_json::from_str(kb_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let memory = match build_memory(facts_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    match ai_core::expert::backward_chain(&kb, &memory, goal) {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

/// Jawaban atas pertanyaan "kenapa aturan ini ada".
#[wasm_bindgen]
pub fn expert_why(kb_json: &str, rule_id: &str) -> String {
    let kb: ai_core::expert::KnowledgeBase = match serde_json::from_str(kb_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    match ai_core::expert::explain_why(&kb, rule_id) {
        Some(v) => ok(v),
        None => err(format!("aturan tidak ditemukan: {rule_id}")),
    }
}

/// Jawaban atas pertanyaan "bagaimana kesimpulan ini diperoleh".
#[wasm_bindgen]
pub fn expert_how(kb_json: &str, facts_json: &str, fact: &str) -> String {
    let kb: ai_core::expert::KnowledgeBase = match serde_json::from_str(kb_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let memory = match build_memory(facts_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    match ai_core::expert::forward_chain(&kb, &memory) {
        Ok(result) => ok(ai_core::expert::explain_how(&result, fact)),
        Err(e) => err(e),
    }
}

// ---------------------------------------------------------------------------
// Sesi 12 & 13 — Sains Data dan Machine Learning
// ---------------------------------------------------------------------------

/// Membaca nama ukuran jarak.
fn parse_distance(name: &str) -> Result<ai_core::ml::Distance, String> {
    use ai_core::ml::Distance;
    Ok(match name.to_ascii_lowercase().as_str() {
        "euclidean" => Distance::Euclidean,
        "manhattan" => Distance::Manhattan,
        "chebyshev" => Distance::Chebyshev,
        other => return Err(format!("ukuran jarak tidak dikenal: {other}")),
    })
}

/// Klasifikasi satu titik dengan K-Nearest Neighbours.
#[wasm_bindgen]
pub fn ml_knn_predict(
    x_json: &str,
    y_json: &str,
    query_json: &str,
    k: usize,
    distance: &str,
    weighted: bool,
) -> String {
    let x: Vec<Vec<f64>> = match serde_json::from_str(x_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let y: Vec<String> = match serde_json::from_str(y_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let query: Vec<f64> = match serde_json::from_str(query_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let d = match parse_distance(distance) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let model = match ai_core::ml::Knn::new(x, y, k, d, weighted) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    match model.predict(&query) {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

/// Wilayah keputusan KNN pada kisi seragam, untuk digambar sebagai latar.
///
/// Jumlah argumennya melewati ambang clippy. Membungkusnya menjadi satu struktur
/// justru memperburuk keadaan di sini: setiap argumen harus melintasi batas
/// WebAssembly, dan `wasm-bindgen` hanya menerima tipe sederhana, sehingga
/// pembungkusnya harus melewati JSON dan menambah satu jalur kesalahan baru
/// untuk masalah yang murni kosmetik.
#[allow(clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn ml_knn_regions(
    x_json: &str,
    y_json: &str,
    k: usize,
    distance: &str,
    weighted: bool,
    min: f64,
    max: f64,
    resolution: usize,
) -> String {
    let x: Vec<Vec<f64>> = match serde_json::from_str(x_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let y: Vec<String> = match serde_json::from_str(y_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let d = match parse_distance(distance) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if !(2..=200).contains(&resolution) {
        return err(format!(
            "resolusi harus antara 2 dan 200, diberi {resolution}"
        ));
    }
    if !min.is_finite() || !max.is_finite() || min >= max {
        return err(format!("rentang tidak sah: {min} sampai {max}"));
    }

    // Daftar kelas dikumpulkan lebih dulu supaya keluarannya berupa indeks
    // ringkas, bukan ribuan salinan nama kelas yang sama.
    let mut classes: Vec<String> = y.clone();
    classes.sort();
    classes.dedup();

    let model = match ai_core::ml::Knn::new(x, y, k, d, weighted) {
        Ok(v) => v,
        Err(e) => return err(e),
    };

    let step = (max - min) / (resolution - 1) as f64;
    let mut cells = Vec::with_capacity(resolution * resolution);
    for j in 0..resolution {
        for i in 0..resolution {
            let point = [min + step * i as f64, min + step * j as f64];
            match model.predict(&point) {
                Ok(r) => {
                    let index = classes.iter().position(|c| *c == r.label).unwrap_or(0);
                    cells.push(index as u32);
                }
                Err(e) => return err(e),
            }
        }
    }
    #[derive(Serialize)]
    struct Out {
        classes: Vec<String>,
        resolution: usize,
        cells: Vec<u32>,
    }
    ok(Out {
        classes,
        resolution,
        cells,
    })
}

/// Pengelompokan K-Means.
#[wasm_bindgen]
pub fn ml_kmeans(
    x_json: &str,
    k: usize,
    distance: &str,
    max_iterations: usize,
    seed: u64,
) -> String {
    let x: Vec<Vec<f64>> = match serde_json::from_str(x_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let d = match parse_distance(distance) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    match ai_core::ml::kmeans(&x, k, d, max_iterations, seed) {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

/// Membangun pohon keputusan ID3 dari data kategorikal.
#[wasm_bindgen]
pub fn ml_build_tree(x_json: &str, y_json: &str, names_json: &str, max_depth: usize) -> String {
    let x: Vec<Vec<String>> = match serde_json::from_str(x_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let y: Vec<String> = match serde_json::from_str(y_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let names: Vec<String> = match serde_json::from_str(names_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    match ai_core::ml::build_id3(&x, &y, &names, max_depth) {
        Ok(tree) => {
            #[derive(Serialize)]
            struct Out {
                tree: ai_core::ml::TreeNode,
                depth: usize,
                leaves: usize,
                root_entropy: f64,
                gains: Vec<(String, f64)>,
            }
            // Perolehan tiap atribut pada akar ditampilkan agar terlihat
            // mengapa atribut tertentu yang dipilih lebih dulu.
            let gains = names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let values: Vec<String> = x
                        .iter()
                        .map(|r| r.get(i).cloned().unwrap_or_default())
                        .collect();
                    (name.clone(), ai_core::ml::information_gain(&values, &y))
                })
                .collect();
            ok(Out {
                depth: tree.depth(),
                leaves: tree.leaf_count(),
                root_entropy: ai_core::ml::entropy(&y),
                gains,
                tree,
            })
        }
        Err(e) => err(e),
    }
}

/// Memprediksi label sebuah baris dengan pohon yang sudah dibangun.
#[wasm_bindgen]
pub fn ml_tree_predict(tree_json: &str, row_json: &str) -> String {
    let tree: ai_core::ml::TreeNode = match serde_json::from_str(tree_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let row: Vec<String> = match serde_json::from_str(row_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    ok(tree.predict(&row))
}

/// Regresi linear satu peubah.
#[wasm_bindgen]
pub fn ml_fit_linear(x_json: &str, y_json: &str) -> String {
    let x: Vec<f64> = match serde_json::from_str(x_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let y: Vec<f64> = match serde_json::from_str(y_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    match ai_core::ml::fit_linear(&x, &y) {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

/// Matriks konfusi beserta ukuran turunannya.
#[wasm_bindgen]
pub fn ml_evaluate(actual_json: &str, predicted_json: &str) -> String {
    let actual: Vec<String> = match serde_json::from_str(actual_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    let predicted: Vec<String> = match serde_json::from_str(predicted_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    match ai_core::ml::evaluate(&actual, &predicted) {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

/// Kumpulan data tenis klasik, dipakai pada laboratorium pohon keputusan.
#[wasm_bindgen]
pub fn ml_tennis_dataset() -> String {
    let rows: [([&str; 4], &str); 14] = [
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
    #[derive(Serialize)]
    struct Out {
        names: Vec<String>,
        values: Vec<Vec<String>>,
        x: Vec<Vec<String>>,
        y: Vec<String>,
    }
    let x: Vec<Vec<String>> = rows
        .iter()
        .map(|(f, _)| f.iter().map(|v| v.to_string()).collect())
        .collect();
    let y: Vec<String> = rows.iter().map(|(_, l)| l.to_string()).collect();
    let names: Vec<String> = ["Cuaca", "Suhu", "Kelembapan", "Angin"]
        .iter()
        .map(|v| v.to_string())
        .collect();
    // Nilai unik tiap atribut, dipakai antarmuka untuk menyusun pilihan.
    let values: Vec<Vec<String>> = (0..names.len())
        .map(|i| {
            let mut v: Vec<String> = x.iter().map(|r| r[i].clone()).collect();
            v.sort();
            v.dedup();
            v
        })
        .collect();
    ok(Out {
        names,
        values,
        x,
        y,
    })
}

// ---------------------------------------------------------------------------
// Sesi 10 — Pemrosesan Bahasa Alami
// ---------------------------------------------------------------------------

/// Memenggal teks, membuang kata henti bila diminta, lalu mencari kata dasarnya.
///
/// Seluruh tahap dikembalikan sekaligus supaya antarmuka bisa memperlihatkan
/// apa yang hilang di tiap langkah — bagian yang paling sering mengejutkan
/// orang adalah berapa banyak kata yang lenyap saat kata henti dibuang.
#[wasm_bindgen]
pub fn nlp_pipeline(text: &str, remove_stop: bool, do_stem: bool) -> String {
    let tokens = ai_core::nlp::tokenize(text);
    let after_stop = if remove_stop {
        ai_core::nlp::remove_stopwords(&tokens, ai_core::nlp::STOPWORDS_ID)
    } else {
        tokens.clone()
    };
    let stems = if do_stem {
        after_stop
            .iter()
            .map(|t| ai_core::nlp::stem_id(t, ai_core::nlp::DICTIONARY_ID))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    #[derive(Serialize)]
    struct Out {
        sentences: Vec<String>,
        tokens: Vec<String>,
        after_stopwords: Vec<String>,
        stems: Vec<ai_core::nlp::StemResult>,
        final_tokens: Vec<String>,
    }
    let final_tokens = if do_stem {
        stems.iter().map(|s| s.stem.clone()).collect()
    } else {
        after_stop.clone()
    };
    ok(Out {
        sentences: ai_core::nlp::sentences(text),
        tokens,
        after_stopwords: after_stop,
        stems,
        final_tokens,
    })
}

/// Pencarian kata dasar satu kata beserta jejak pengupasannya.
#[wasm_bindgen]
pub fn nlp_stem(word: &str) -> String {
    ok(ai_core::nlp::stem_id(word, ai_core::nlp::DICTIONARY_ID))
}

/// Bobot TF-IDF sebuah korpus, ditambah matriks kemiripan kosinus antardokumen.
#[wasm_bindgen]
pub fn nlp_tfidf(documents_json: &str, remove_stop: bool, do_stem: bool) -> String {
    let texts: Vec<String> = match serde_json::from_str(documents_json) {
        Ok(v) => v,
        Err(e) => return err(e),
    };
    if texts.is_empty() {
        return err("korpus kosong");
    }

    let docs: Vec<Vec<String>> = texts
        .iter()
        .map(|text| {
            let tokens = ai_core::nlp::tokenize(text);
            let tokens = if remove_stop {
                ai_core::nlp::remove_stopwords(&tokens, ai_core::nlp::STOPWORDS_ID)
            } else {
                tokens
            };
            if do_stem {
                ai_core::nlp::stem_all(&tokens, ai_core::nlp::DICTIONARY_ID)
            } else {
                tokens
            }
        })
        .collect();

    match ai_core::nlp::tf_idf(&docs) {
        Ok(model) => {
            let n = model.vectors.len();
            let mut similarity = vec![vec![0.0; n]; n];
            for (i, row) in similarity.iter_mut().enumerate() {
                for (j, cell) in row.iter_mut().enumerate() {
                    match ai_core::nlp::cosine_similarity(&model.vectors[i], &model.vectors[j]) {
                        Ok(v) => *cell = v,
                        Err(e) => return err(e),
                    }
                }
            }
            #[derive(Serialize)]
            struct Out {
                vocabulary: Vec<String>,
                idf: Vec<f64>,
                vectors: Vec<Vec<f64>>,
                similarity: Vec<Vec<f64>>,
                documents: Vec<Vec<String>>,
            }
            ok(Out {
                vocabulary: model.vocabulary,
                idf: model.idf,
                vectors: model.vectors,
                similarity,
                documents: docs,
            })
        }
        Err(e) => err(e),
    }
}

/// Jarak sunting dan kemiripannya antara dua kata.
#[wasm_bindgen]
pub fn nlp_levenshtein(a: &str, b: &str) -> String {
    #[derive(Serialize)]
    struct Out {
        distance: usize,
        similarity: f64,
    }
    ok(Out {
        distance: ai_core::nlp::levenshtein(a, b),
        similarity: ai_core::nlp::levenshtein_similarity(a, b),
    })
}

/// N-gram kata dari sebuah teks.
#[wasm_bindgen]
pub fn nlp_ngrams(text: &str, n: usize) -> String {
    let tokens = ai_core::nlp::tokenize(text);
    match ai_core::nlp::ngrams(&tokens, n) {
        Ok(v) => ok(v),
        Err(e) => err(e),
    }
}

/// Analisis sentimen berbasis leksikon yang menghormati pengingkaran.
#[wasm_bindgen]
pub fn nlp_sentiment(text: &str) -> String {
    ok(ai_core::nlp::sentiment_id(&ai_core::nlp::tokenize(text)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pembungkus_ok_dan_err() {
        assert_eq!(ok(1.5), r#"{"ok":1.5}"#);
        assert!(err("gagal").contains("gagal"));
        // Tanda kutip ganda pada pesan galat tidak boleh merusak JSON.
        assert!(!err(r#"pesan "aneh""#).contains(r#""aneh""#));
    }

    #[test]
    fn versi_dan_sesi() {
        assert!(!version().is_empty());
        let s = sessions();
        assert!(s.starts_with(r#"{"ok":["#));
        assert!(s.contains("certainty"));
    }

    #[test]
    fn cf_dasar_lewat_jembatan() {
        assert!(cf_from_mb_md(0.8, 0.01).contains("0.79"));
        assert!(cf_from_mb_md(2.0, 0.0).contains("err"));
    }

    #[test]
    fn cf_gabungan_lewat_jembatan() {
        let out = cf_combine("[0.5,0.3,0.2]");
        assert!(out.contains(r#""value":0.72"#), "{out}");
        assert!(out.contains("steps"));
        assert!(out.contains("label_id"));
        assert!(cf_combine("bukan json").contains("err"));
        assert!(cf_combine("[]").contains("err"));
    }

    #[test]
    fn cf_premis_lewat_jembatan() {
        assert!(cf_premise("[0.8,0.4]", "AND").contains("0.4"));
        assert!(cf_premise("[0.8,0.4]", "or").contains("0.8"));
        assert!(cf_premise("[0.8,0.4]", "XOR").contains("err"));
        assert!(cf_premise("{}", "AND").contains("err"));
    }

    #[test]
    fn cf_berantai_lewat_jembatan() {
        assert!(cf_sequential(0.8, 0.5).contains("0.4"));
        assert!(cf_sequential(9.0, 0.5).contains("err"));
    }

    #[test]
    fn bayes_lewat_jembatan() {
        let out = bayes_binary(0.2, 0.9, 0.3);
        assert!(out.contains(r#""evidence":0.42"#), "{out}");
        assert!(bayes_binary(2.0, 0.9, 0.3).contains("err"));
    }

    #[test]
    fn bayes_posterior_semua_lewat_jembatan() {
        let out = bayes_posterior_all("[0.2,0.8]", "[0.9,0.3]");
        assert!(out.starts_with(r#"{"ok":["#), "{out}");
        assert!(bayes_posterior_all("x", "[0.9]").contains("err"));
        assert!(bayes_posterior_all("[0.2]", "x").contains("err"));
        assert!(bayes_posterior_all("[0.2,0.8]", "[0.9]").contains("err"));
    }

    #[test]
    fn naive_bayes_lewat_jembatan() {
        let samples = r#"[
            {"features":["Cerah","Panas"],"label":"Tidak"},
            {"features":["Mendung","Panas"],"label":"Ya"},
            {"features":["Mendung","Dingin"],"label":"Ya"}
        ]"#;
        let out = naive_bayes_predict(samples, r#"["Mendung","Panas"]"#, 1.0);
        assert!(out.contains(r#""label":"Ya""#), "{out}");
        assert!(naive_bayes_predict("[]", r#"["a"]"#, 1.0).contains("err"));
        assert!(naive_bayes_predict("bukan", r#"["a"]"#, 1.0).contains("err"));
        assert!(naive_bayes_predict(samples, "bukan", 1.0).contains("err"));
        assert!(naive_bayes_predict(samples, r#"["Mendung"]"#, 1.0).contains("err"));
    }

    #[test]
    fn fuzzy_derajat_lewat_jembatan() {
        let seg = r#"{"kind":"triangular","a":0,"b":5,"c":10}"#;
        assert!(fuzzy_degree(seg, 5.0).contains("1"));
        assert!(fuzzy_degree(seg, 2.5).contains("0.5"));
        assert!(fuzzy_degree("bukan json", 1.0).contains("err"));
        // Titik tidak terurut ditolak, bukan diam-diam dihitung.
        assert!(fuzzy_degree(r#"{"kind":"triangular","a":9,"b":1,"c":10}"#, 5.0).contains("err"));
    }

    #[test]
    fn fuzzy_kurva_lewat_jembatan() {
        let seg = r#"{"kind":"triangular","a":0,"b":5,"c":10}"#;
        let out = fuzzy_curve(seg, 0.0, 10.0, 11);
        assert!(out.contains("xs"), "{out}");
        assert!(out.contains("ys"));
        assert!(fuzzy_curve(seg, 10.0, 0.0, 11).contains("err"));
        assert!(fuzzy_curve(seg, 0.0, 10.0, 1).contains("err"));
        assert!(fuzzy_curve("{}", 0.0, 10.0, 11).contains("err"));
    }

    /// Sistem minimal dua aturan untuk menguji jembatan inferensi.
    fn sistem_uji() -> String {
        r#"{
          "inputs":[{"name":"X","min":0,"max":10,"sets":[
            {"name":"Rendah","membership":{"kind":"trapezoidal","a":0,"b":0,"c":2,"d":5}},
            {"name":"Tinggi","membership":{"kind":"trapezoidal","a":5,"b":8,"c":10,"d":10}}
          ]}],
          "output":{"name":"Y","min":0,"max":100,"sets":[
            {"name":"Kecil","membership":{"kind":"triangular","a":0,"b":0,"c":50}},
            {"name":"Besar","membership":{"kind":"triangular","a":50,"b":100,"c":100}}
          ]},
          "rules":[
            {"antecedents":[{"variable":"X","set":"Rendah"}],"connective":"AND",
             "consequent_set":"Kecil","consequent_value":10,"weight":1},
            {"antecedents":[{"variable":"X","set":"Tinggi"}],"connective":"AND",
             "consequent_set":"Besar","consequent_value":90,"weight":1}
          ]
        }"#
        .to_string()
    }

    #[test]
    fn fuzzy_inferensi_tiga_mesin() {
        let s = sistem_uji();
        for mesin in ["mamdani", "sugeno", "tsukamoto"] {
            let out = fuzzy_infer(&s, r#"[["X",9.0]]"#, mesin, "centroid", 101);
            assert!(out.contains(r#""crisp""#), "{mesin}: {out}");
            assert!(!out.contains("err"), "{mesin}: {out}");
        }
    }

    #[test]
    fn fuzzy_inferensi_menolak_masukan_salah() {
        let s = sistem_uji();
        assert!(fuzzy_infer(&s, r#"[["X",9.0]]"#, "entah", "centroid", 101).contains("err"));
        assert!(fuzzy_infer(&s, r#"[["X",9.0]]"#, "mamdani", "entah", 101).contains("err"));
        assert!(fuzzy_infer("{}", r#"[["X",9.0]]"#, "mamdani", "centroid", 101).contains("err"));
        assert!(fuzzy_infer(&s, "bukan", "mamdani", "centroid", 101).contains("err"));
        // Variabel yang tidak diberi nilai harus menghasilkan galat, bukan nol diam.
        assert!(fuzzy_infer(&s, "[]", "mamdani", "centroid", 101).contains("err"));
    }

    #[test]
    fn fuzzy_lima_metode_defuzzifikasi_dikenali() {
        let s = sistem_uji();
        for m in [
            "centroid",
            "bisector",
            "mean_of_maximum",
            "mom",
            "smallest_of_maximum",
            "som",
            "largest_of_maximum",
            "lom",
        ] {
            let out = fuzzy_infer(&s, r#"[["X",9.0]]"#, "mamdani", m, 101);
            assert!(!out.contains("err"), "{m}: {out}");
        }
    }

    #[test]
    fn kisi_kosong_lewat_jembatan() {
        let out = search_empty_grid(5, 4);
        assert!(out.contains(r#""width":5"#), "{out}");
        assert!(search_empty_grid(0, 4).contains("err"));
    }

    #[test]
    fn labirin_lewat_jembatan() {
        let out = search_maze(11, 11, 7);
        assert!(out.contains("walls"), "{out}");
        assert!(search_maze(0, 11, 7).contains("err"));
    }

    fn kisi_uji() -> String {
        let g = ai_core::search::Grid::new(9, 9).unwrap();
        serde_json::to_string(&g).unwrap()
    }

    const OPSI: &str = r#"{"algorithm":"a_star","heuristic":"manhattan","depth_limit":64,"seed":1,"max_expansions":50000}"#;

    #[test]
    fn pencarian_lewat_jembatan() {
        let out = search_run(&kisi_uji(), 0, 0, 8, 8, OPSI);
        assert!(out.contains(r#""found":true"#), "{out}");
        assert!(out.contains("expanded"));
    }

    #[test]
    fn pencarian_menolak_masukan_salah() {
        assert!(search_run("bukan", 0, 0, 8, 8, OPSI).contains("err"));
        assert!(search_run(&kisi_uji(), 0, 0, 8, 8, "bukan").contains("err"));
        // Titik di luar kisi harus jadi galat, bukan hasil kosong yang membingungkan.
        assert!(search_run(&kisi_uji(), 0, 0, 99, 99, OPSI).contains("err"));
        assert!(search_run(&kisi_uji(), 99, 0, 8, 8, OPSI).contains("err"));
    }

    #[test]
    fn perbandingan_memuat_sembilan_algoritma() {
        let out = search_compare(&kisi_uji(), 0, 0, 8, 8, OPSI);
        assert!(!out.contains(r#""err""#), "{out}");
        for slug in [
            "breadth_first",
            "depth_first",
            "depth_limited",
            "iterative_deepening",
            "uniform_cost",
            "greedy_best_first",
            "a_star",
            "hill_climbing",
            "simulated_annealing",
        ] {
            assert!(out.contains(slug), "{slug} hilang dari perbandingan");
        }
        // Biaya tak berhingga tidak boleh bocor ke JSON sebagai null.
        assert!(!out.contains("null"), "{out}");
    }

    #[test]
    fn perbandingan_menolak_masukan_salah() {
        assert!(search_compare("bukan", 0, 0, 8, 8, OPSI).contains("err"));
        assert!(search_compare(&kisi_uji(), 0, 0, 8, 8, "{").contains("err"));
        assert!(search_compare(&kisi_uji(), 0, 0, 99, 99, OPSI).contains("err"));
    }

    fn jaringan_uji() -> String {
        let out = neural_create("[2,4,1]", "tanh", "sigmoid", 0.1, 0.9, 1);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        serde_json::to_string(&v["ok"]["network"]).unwrap()
    }

    #[test]
    fn membuat_jaringan_lewat_jembatan() {
        // Laju 0,05 dengan momentum 0,9 memberi langkah efektif 0,5 — jelas di
        // bawah ambang. Nilai 0,1 sengaja dihindari di sini: 0.1 / (1 - 0.9)
        // menghasilkan 1.0000000000000002 pada aritmetika biner, tepat
        // menyerempet ambang, sehingga ujinya akan rapuh tanpa alasan.
        let out = neural_create("[2,4,1]", "tanh", "sigmoid", 0.05, 0.9, 1);
        assert!(out.contains(r#""parameters":17"#), "{out}");
        assert!(out.contains("effective_learning_rate"));
        assert!(out.contains(r#""step_risky":false"#), "{out}");
    }

    #[test]
    fn membuat_jaringan_menolak_masukan_salah() {
        assert!(neural_create("bukan", "tanh", "sigmoid", 0.1, 0.9, 1).contains("err"));
        assert!(neural_create("[2,4,1]", "entah", "sigmoid", 0.1, 0.9, 1).contains("err"));
        assert!(neural_create("[2,4,1]", "tanh", "entah", 0.1, 0.9, 1).contains("err"));
        // Ambang keras tidak bisa dilatih perambatan balik.
        assert!(neural_create("[2,4,1]", "step", "sigmoid", 0.1, 0.9, 1).contains("err"));
        assert!(neural_create("[2]", "tanh", "sigmoid", 0.1, 0.9, 1).contains("err"));
        assert!(neural_create("[2,4,1]", "tanh", "sigmoid", 0.0, 0.9, 1).contains("err"));
    }

    #[test]
    fn langkah_berisiko_ditandai() {
        // 0.2 / (1 - 0.9) = 2.0, susunan yang terbukti gagal pada spiral.
        let out = neural_create("[2,4,1]", "tanh", "sigmoid", 0.2, 0.9, 1);
        assert!(out.contains(r#""step_risky":true"#), "{out}");
    }

    #[test]
    fn kumpulan_data_lewat_jembatan() {
        for nama in ["xor", "and", "or", "spiral"] {
            let out = neural_dataset(nama, 20, 0.03, 1);
            assert!(!out.contains(r#""err""#), "{nama}: {out}");
            assert!(out.contains(r#""x":[["#), "{nama}: {out}");
        }
        assert!(neural_dataset("entah", 20, 0.03, 1).contains("err"));
    }

    #[test]
    fn pelatihan_lewat_jembatan() {
        let data = neural_dataset("xor", 0, 0.0, 1);
        let v: serde_json::Value = serde_json::from_str(&data).unwrap();
        let x = serde_json::to_string(&v["ok"]["x"]).unwrap();
        let y = serde_json::to_string(&v["ok"]["y"]).unwrap();

        let out = neural_train(&jaringan_uji(), &x, &y, 50, 1e-4, 7);
        assert!(!out.contains(r#""err""#), "{out}");
        assert!(out.contains("history"));
        assert!(out.contains(r#""epoch":1"#));
    }

    #[test]
    fn pelatihan_menolak_masukan_salah() {
        assert!(neural_train("bukan", "[]", "[]", 10, 0.0, 1).contains("err"));
        assert!(neural_train(&jaringan_uji(), "bukan", "[]", 10, 0.0, 1).contains("err"));
        assert!(neural_train(&jaringan_uji(), "[[0,0]]", "bukan", 10, 0.0, 1).contains("err"));
        // Data kosong harus jadi galat, bukan riwayat kosong yang membingungkan.
        assert!(neural_train(&jaringan_uji(), "[]", "[]", 10, 0.0, 1).contains("err"));
    }

    #[test]
    fn kisi_keputusan_lewat_jembatan() {
        let out = neural_decision_grid(&jaringan_uji(), -1.0, 1.0, 20);
        assert!(out.contains(r#""resolution":20"#), "{out}");
        assert!(out.contains("values"));
    }

    #[test]
    fn kisi_keputusan_menolak_masukan_salah() {
        let net = jaringan_uji();
        assert!(neural_decision_grid("bukan", -1.0, 1.0, 20).contains("err"));
        assert!(neural_decision_grid(&net, -1.0, 1.0, 1).contains("err"));
        assert!(neural_decision_grid(&net, -1.0, 1.0, 999).contains("err"));
        assert!(neural_decision_grid(&net, 1.0, -1.0, 20).contains("err"));
        assert!(neural_decision_grid(&net, f64::NAN, 1.0, 20).contains("err"));

        // Jaringan dengan jumlah masukan lain harus ditolak, bukan menghasilkan
        // gambar yang terlihat masuk akal tetapi tidak bermakna.
        let tiga = neural_create("[3,4,1]", "tanh", "sigmoid", 0.1, 0.0, 1);
        let v: serde_json::Value = serde_json::from_str(&tiga).unwrap();
        let net3 = serde_json::to_string(&v["ok"]["network"]).unwrap();
        assert!(neural_decision_grid(&net3, -1.0, 1.0, 20).contains("err"));
    }

    fn kb_uji() -> String {
        let out = expert_sample_kb();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        serde_json::to_string(&v["ok"]).unwrap()
    }

    const GEJALA_FLU: &str = r#"[["demam",1.0],["pilek",1.0],["batuk",1.0]]"#;

    #[test]
    fn basis_pengetahuan_contoh_lewat_jembatan() {
        let out = expert_sample_kb();
        assert!(out.contains("Dokter Virtual"), "{out}");
        assert!(out.contains("R1"));
    }

    #[test]
    fn pemeriksaan_basis_pengetahuan() {
        let out = expert_inspect_kb(&kb_uji());
        assert!(out.contains(r#""rules":6"#), "{out}");
        // Basis contoh tidak boleh punya fakta yang tak terjangkau.
        assert!(out.contains(r#""unreachable_facts":[]"#), "{out}");
        assert!(out.contains("leaf_facts"));
        assert!(expert_inspect_kb("bukan").contains("err"));

        let rusak = r#"{"name":"x","rules":[],"askable":[]}"#;
        assert!(expert_inspect_kb(rusak).contains("err"));
    }

    #[test]
    fn runut_maju_lewat_jembatan() {
        let out = expert_forward(&kb_uji(), GEJALA_FLU, 0.2);
        assert!(!out.contains(r#""err""#), "{out}");
        assert!(out.contains("flu"), "{out}");
        assert!(out.contains("steps"));
        assert!(out.contains("passes"));
    }

    #[test]
    fn runut_maju_memisahkan_kesimpulan_dari_masukan() {
        // Gejala yang diketikkan pengguna tidak boleh muncul sebagai
        // "kesimpulan"; sistem yang menyimpulkan masukannya sendiri terlihat
        // pintar padahal tidak melakukan apa pun.
        let out = expert_forward(&kb_uji(), GEJALA_FLU, 0.2);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let derived = v["ok"]["derived"].as_array().unwrap();
        let given = v["ok"]["given"].as_array().unwrap();

        let nama = |arr: &Vec<serde_json::Value>| -> Vec<String> {
            arr.iter()
                .map(|p| p[0].as_str().unwrap().to_string())
                .collect()
        };
        assert!(nama(derived).contains(&"flu".to_string()));
        assert!(!nama(derived).contains(&"demam".to_string()));
        assert!(nama(given).contains(&"demam".to_string()));
        assert!(nama(given).contains(&"batuk".to_string()));
    }

    #[test]
    fn runut_maju_menolak_masukan_salah() {
        assert!(expert_forward("bukan", GEJALA_FLU, 0.2).contains("err"));
        assert!(expert_forward(&kb_uji(), "bukan", 0.2).contains("err"));
        // Keyakinan di luar rentang harus ditolak, bukan dijepit diam-diam.
        assert!(expert_forward(&kb_uji(), r#"[["demam",9.0]]"#, 0.2).contains("err"));
    }

    #[test]
    fn runut_mundur_lewat_jembatan() {
        let out = expert_backward(&kb_uji(), GEJALA_FLU, "flu");
        assert!(!out.contains(r#""err""#), "{out}");
        assert!(out.contains(r#""goal":"flu""#), "{out}");
        assert!(out.contains("proof"));

        // Dari memori kosong, sistem harus tahu apa yang perlu ditanyakan.
        let kosong = expert_backward(&kb_uji(), "[]", "flu");
        assert!(kosong.contains("questions"));
        assert!(kosong.contains("demam"), "{kosong}");
    }

    #[test]
    fn runut_mundur_menolak_masukan_salah() {
        assert!(expert_backward("bukan", "[]", "flu").contains("err"));
        assert!(expert_backward(&kb_uji(), "bukan", "flu").contains("err"));
    }

    #[test]
    fn penjelasan_lewat_jembatan() {
        let why = expert_why(&kb_uji(), "R1");
        assert!(why.contains("JIKA"), "{why}");
        assert!(expert_why(&kb_uji(), "R99").contains("err"));
        assert!(expert_why("bukan", "R1").contains("err"));

        let how = expert_how(&kb_uji(), GEJALA_FLU, "flu");
        assert!(how.contains("R1"), "{how}");
        assert!(expert_how("bukan", GEJALA_FLU, "flu").contains("err"));
        assert!(expert_how(&kb_uji(), "bukan", "flu").contains("err"));
    }

    const X_KNN: &str = r#"[[1,1],[1.2,0.9],[0.8,1.1],[8,8],[8.2,7.9],[7.8,8.1]]"#;
    const Y_KNN: &str = r#"["A","A","A","B","B","B"]"#;

    #[test]
    fn knn_lewat_jembatan() {
        let out = ml_knn_predict(X_KNN, Y_KNN, "[1,1]", 3, "euclidean", false);
        assert!(out.contains(r#""label":"A""#), "{out}");
        assert!(out.contains("neighbours"));
        let jauh = ml_knn_predict(X_KNN, Y_KNN, "[8,8]", 3, "manhattan", true);
        assert!(jauh.contains(r#""label":"B""#), "{jauh}");
    }

    #[test]
    fn knn_menolak_masukan_salah() {
        assert!(ml_knn_predict("bukan", Y_KNN, "[1,1]", 3, "euclidean", false).contains("err"));
        assert!(ml_knn_predict(X_KNN, "bukan", "[1,1]", 3, "euclidean", false).contains("err"));
        assert!(ml_knn_predict(X_KNN, Y_KNN, "bukan", 3, "euclidean", false).contains("err"));
        assert!(ml_knn_predict(X_KNN, Y_KNN, "[1,1]", 3, "entah", false).contains("err"));
        assert!(ml_knn_predict(X_KNN, Y_KNN, "[1,1]", 0, "euclidean", false).contains("err"));
        // Titik dengan jumlah fitur berbeda harus ditolak.
        assert!(ml_knn_predict(X_KNN, Y_KNN, "[1]", 3, "euclidean", false).contains("err"));
    }

    #[test]
    fn wilayah_keputusan_knn() {
        let out = ml_knn_regions(X_KNN, Y_KNN, 3, "euclidean", false, 0.0, 10.0, 20);
        assert!(out.contains(r#""resolution":20"#), "{out}");
        assert!(out.contains(r#""classes":["A","B"]"#), "{out}");
        assert!(ml_knn_regions(X_KNN, Y_KNN, 3, "euclidean", false, 0.0, 10.0, 1).contains("err"));
        assert!(ml_knn_regions(X_KNN, Y_KNN, 3, "euclidean", false, 10.0, 0.0, 20).contains("err"));
        assert!(ml_knn_regions(X_KNN, Y_KNN, 3, "entah", false, 0.0, 10.0, 20).contains("err"));
    }

    #[test]
    fn kmeans_lewat_jembatan() {
        let out = ml_kmeans(X_KNN, 2, "euclidean", 100, 42);
        assert!(out.contains("centroids"), "{out}");
        assert!(out.contains(r#""converged":true"#), "{out}");
        assert!(ml_kmeans("bukan", 2, "euclidean", 100, 1).contains("err"));
        assert!(ml_kmeans(X_KNN, 99, "euclidean", 100, 1).contains("err"));
        assert!(ml_kmeans(X_KNN, 2, "entah", 100, 1).contains("err"));
    }

    fn data_tenis_json() -> (String, String, String) {
        let out = ml_tennis_dataset();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        (
            serde_json::to_string(&v["ok"]["x"]).unwrap(),
            serde_json::to_string(&v["ok"]["y"]).unwrap(),
            serde_json::to_string(&v["ok"]["names"]).unwrap(),
        )
    }

    #[test]
    fn kumpulan_data_tenis_lewat_jembatan() {
        let out = ml_tennis_dataset();
        assert!(out.contains("Cuaca"), "{out}");
        assert!(out.contains("Mendung"));
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["ok"]["x"].as_array().unwrap().len(), 14);
        assert_eq!(v["ok"]["values"].as_array().unwrap().len(), 4);
    }

    #[test]
    fn pohon_keputusan_lewat_jembatan() {
        let (x, y, names) = data_tenis_json();
        let out = ml_build_tree(&x, &y, &names, 10);
        assert!(!out.contains(r#""err""#), "{out}");
        assert!(out.contains(r#""kind":"branch""#), "{out}");
        assert!(out.contains("root_entropy"));
        assert!(out.contains("gains"));

        // Perolehan Cuaca harus yang tertinggi.
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let gains = v["ok"]["gains"].as_array().unwrap();
        let cuaca = gains.iter().find(|g| g[0] == "Cuaca").unwrap()[1]
            .as_f64()
            .unwrap();
        for g in gains {
            assert!(cuaca >= g[1].as_f64().unwrap() - 1e-12);
        }
    }

    #[test]
    fn pohon_keputusan_menolak_masukan_salah() {
        let (x, y, names) = data_tenis_json();
        assert!(ml_build_tree("bukan", &y, &names, 5).contains("err"));
        assert!(ml_build_tree(&x, "bukan", &names, 5).contains("err"));
        assert!(ml_build_tree(&x, &y, "bukan", 5).contains("err"));
        assert!(ml_build_tree("[]", "[]", &names, 5).contains("err"));
    }

    #[test]
    fn ramalan_pohon_lewat_jembatan() {
        let (x, y, names) = data_tenis_json();
        let built = ml_build_tree(&x, &y, &names, 10);
        let v: serde_json::Value = serde_json::from_str(&built).unwrap();
        let tree = serde_json::to_string(&v["ok"]["tree"]).unwrap();

        let out = ml_tree_predict(&tree, r#"["Mendung","Panas","Tinggi","Lemah"]"#);
        assert!(out.contains("Ya"), "{out}");
        assert!(ml_tree_predict("bukan", "[]").contains("err"));
        assert!(ml_tree_predict(&tree, "bukan").contains("err"));
    }

    #[test]
    fn regresi_linear_lewat_jembatan() {
        let out = ml_fit_linear("[1,2,3,4]", "[3,5,7,9]");
        assert!(out.contains(r#""slope":2"#), "{out}");
        assert!(out.contains("r_squared"));
        assert!(ml_fit_linear("bukan", "[1]").contains("err"));
        assert!(ml_fit_linear("[1]", "bukan").contains("err"));
        assert!(ml_fit_linear("[1,2]", "[1]").contains("err"));
    }

    #[test]
    fn evaluasi_lewat_jembatan() {
        let out = ml_evaluate(r#"["A","A","B"]"#, r#"["A","B","B"]"#);
        assert!(out.contains("accuracy"), "{out}");
        assert!(out.contains("baseline_accuracy"), "{out}");
        assert!(out.contains("macro_f1"));
        assert!(ml_evaluate("bukan", "[]").contains("err"));
        assert!(ml_evaluate(r#"["A"]"#, "[]").contains("err"));
    }

    #[test]
    fn pipeline_nlp_lewat_jembatan() {
        let out = nlp_pipeline("Saya suka membaca buku di kampus.", true, true);
        assert!(!out.contains(r#""err""#), "{out}");
        assert!(out.contains("tokens"));
        assert!(out.contains("after_stopwords"));
        assert!(out.contains("final_tokens"));
        // Kata henti "saya" dan "di" harus hilang setelah tahap kedua.
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let setelah: Vec<String> =
            serde_json::from_value(v["ok"]["after_stopwords"].clone()).unwrap();
        assert!(!setelah.contains(&"saya".to_string()));
        assert!(!setelah.contains(&"di".to_string()));
        assert!(setelah.contains(&"kampus".to_string()));
    }

    #[test]
    fn pipeline_nlp_bisa_dimatikan_tahapnya() {
        let out = nlp_pipeline("Saya suka membaca", false, false);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let tokens: Vec<String> = serde_json::from_value(v["ok"]["tokens"].clone()).unwrap();
        let akhir: Vec<String> = serde_json::from_value(v["ok"]["final_tokens"].clone()).unwrap();
        assert_eq!(tokens, akhir, "tanpa tahap apa pun, hasilnya harus sama");
    }

    #[test]
    fn stemming_lewat_jembatan() {
        let out = nlp_stem("menyapu");
        assert!(out.contains(r#""stem":"sapu""#), "{out}");
        assert!(out.contains(r#""in_dictionary":true"#));
        assert!(out.contains("steps"));
        // Kata kamus tidak dikupas sama sekali.
        assert!(nlp_stem("beruang").contains(r#""stem":"beruang""#));
    }

    #[test]
    fn tfidf_lewat_jembatan() {
        let docs = r#"["kucing suka ikan","kucing gemar ikan","mobil melaju cepat"]"#;
        let out = nlp_tfidf(docs, true, false);
        assert!(!out.contains(r#""err""#), "{out}");
        assert!(out.contains("vocabulary"));
        assert!(out.contains("similarity"));

        // Dokumen pertama dan kedua harus lebih mirip daripada dengan ketiga.
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let sim: Vec<Vec<f64>> = serde_json::from_value(v["ok"]["similarity"].clone()).unwrap();
        assert!(sim[0][1] > sim[0][2], "{:?}", sim);
        // Diagonalnya harus satu.
        for (i, row) in sim.iter().enumerate() {
            assert!((row[i] - 1.0).abs() < 1e-9, "diagonal {i} = {}", row[i]);
        }
    }

    #[test]
    fn tfidf_menolak_masukan_salah() {
        assert!(nlp_tfidf("bukan", true, false).contains("err"));
        assert!(nlp_tfidf("[]", true, false).contains("err"));
    }

    #[test]
    fn levenshtein_lewat_jembatan() {
        let out = nlp_levenshtein("kitten", "sitting");
        assert!(out.contains(r#""distance":3"#), "{out}");
        assert!(out.contains("similarity"));
        assert!(nlp_levenshtein("sama", "sama").contains(r#""distance":0"#));
    }

    #[test]
    fn ngram_lewat_jembatan() {
        let out = nlp_ngrams("a b c d", 2);
        assert!(out.contains("a b"), "{out}");
        assert!(nlp_ngrams("a b", 0).contains("err"));
    }

    #[test]
    fn sentimen_lewat_jembatan() {
        assert!(nlp_sentiment("pelayanannya bagus").contains(r#""label":"positif""#));
        assert!(nlp_sentiment("pelayanannya buruk").contains(r#""label":"negatif""#));
        // Pengingkaran harus membalik hasilnya.
        let out = nlp_sentiment("makanannya tidak bagus");
        assert!(out.contains(r#""label":"negatif""#), "{out}");
    }
}
