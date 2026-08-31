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
}
