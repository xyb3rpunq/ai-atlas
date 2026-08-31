//! Sesi 9 — Jaringan Syaraf Tiruan.
//!
//! Perceptron satu lapis, Adaline, dan perceptron banyak lapis yang dilatih
//! dengan perambatan balik. Seluruhnya ditulis dari nol: tidak ada pustaka
//! aljabar linear, tidak ada diferensiasi otomatis.
//!
//! Nilai awal bobot digerakkan [`SplitMix64`] dengan benih eksplisit, sehingga
//! pelatihan yang sama menghasilkan bobot yang sama persis setiap kali
//! dijalankan. Tanpa itu, membandingkan dua percobaan tidak berarti apa-apa.

use crate::rng::SplitMix64;
use serde::{Deserialize, Serialize};

/// Kesalahan pada jaringan syaraf.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NeuralError {
    /// Arsitektur jaringan tidak sah.
    BadArchitecture(String),
    /// Panjang masukan tidak sesuai dengan lapisan pertama.
    InputSizeMismatch {
        /// Jumlah yang diharapkan.
        expected: usize,
        /// Jumlah yang diterima.
        got: usize,
    },
    /// Panjang target tidak sesuai dengan lapisan keluaran.
    TargetSizeMismatch {
        /// Jumlah yang diharapkan.
        expected: usize,
        /// Jumlah yang diterima.
        got: usize,
    },
    /// Jumlah baris masukan dan target berbeda.
    DatasetMismatch {
        /// Jumlah baris masukan.
        inputs: usize,
        /// Jumlah baris target.
        targets: usize,
    },
    /// Kumpulan data kosong.
    EmptyDataset,
    /// Laju belajar bukan bilangan positif.
    BadLearningRate(f64),
    /// Nilai bukan bilangan muncul saat pelatihan.
    Diverged {
        /// Epoch saat kemunculannya terdeteksi.
        epoch: usize,
    },
}

impl core::fmt::Display for NeuralError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            NeuralError::BadArchitecture(s) => write!(f, "arsitektur tidak sah: {s}"),
            NeuralError::InputSizeMismatch { expected, got } => {
                write!(f, "masukan harus {expected} nilai, diberi {got}")
            }
            NeuralError::TargetSizeMismatch { expected, got } => {
                write!(f, "target harus {expected} nilai, diberi {got}")
            }
            NeuralError::DatasetMismatch { inputs, targets } => {
                write!(f, "{inputs} baris masukan tetapi {targets} baris target")
            }
            NeuralError::EmptyDataset => write!(f, "kumpulan data kosong"),
            NeuralError::BadLearningRate(v) => {
                write!(f, "laju belajar harus positif dan berhingga, diberi {v}")
            }
            NeuralError::Diverged { epoch } => {
                write!(f, "pelatihan menyimpang pada epoch {epoch}")
            }
        }
    }
}

/// Fungsi aktivasi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Activation {
    /// Ambang keras. Hanya untuk perceptron; tidak bisa dilatih perambatan balik
    /// karena turunannya nol di mana-mana.
    Step,
    /// Sigmoid logistik, keluaran di rentang `(0, 1)`.
    Sigmoid,
    /// Tangen hiperbolik, keluaran di rentang `(-1, 1)`.
    Tanh,
    /// Penyearah linear. Murah dan tidak menjenuh untuk masukan positif.
    Relu,
    /// Penyearah linear bocor, mencegah neuron mati.
    LeakyRelu,
    /// Identitas, dipakai pada lapisan keluaran regresi.
    Linear,
}

/// Kemiringan bagian negatif pada [`Activation::LeakyRelu`].
pub const LEAKY_SLOPE: f64 = 0.01;

/// Batas laju belajar efektif yang mulai berisiko menyimpang.
pub const RISKY_EFFECTIVE_RATE: f64 = 1.0;

impl Activation {
    /// Menerapkan fungsi aktivasi.
    pub fn apply(self, x: f64) -> f64 {
        match self {
            Activation::Step => {
                if x >= 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
            // Bentuk yang stabil secara numerik: memakai `exp` pada nilai
            // bertanda sama agar tidak meluap untuk masukan besar.
            Activation::Sigmoid => {
                if x >= 0.0 {
                    1.0 / (1.0 + (-x).exp())
                } else {
                    let e = x.exp();
                    e / (1.0 + e)
                }
            }
            Activation::Tanh => x.tanh(),
            Activation::Relu => x.max(0.0),
            Activation::LeakyRelu => {
                if x >= 0.0 {
                    x
                } else {
                    LEAKY_SLOPE * x
                }
            }
            Activation::Linear => x,
        }
    }

    /// Turunan fungsi aktivasi, dinyatakan dari **keluarannya**.
    ///
    /// Bentuk ini dipilih karena perambatan balik sudah memegang keluaran tiap
    /// neuron, sehingga tidak perlu menyimpan masukannya juga.
    pub fn derivative_from_output(self, y: f64) -> f64 {
        match self {
            // Turunan ambang keras nol di mana-mana; nilai satu dipakai supaya
            // aturan delta perceptron tetap berjalan.
            Activation::Step => 1.0,
            Activation::Sigmoid => y * (1.0 - y),
            Activation::Tanh => 1.0 - y * y,
            Activation::Relu => {
                if y > 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
            Activation::LeakyRelu => {
                if y > 0.0 {
                    1.0
                } else {
                    LEAKY_SLOPE
                }
            }
            Activation::Linear => 1.0,
        }
    }

    /// Nama pendek untuk ditampilkan.
    pub fn short_name(self) -> &'static str {
        match self {
            Activation::Step => "Step",
            Activation::Sigmoid => "Sigmoid",
            Activation::Tanh => "Tanh",
            Activation::Relu => "ReLU",
            Activation::LeakyRelu => "Leaky ReLU",
            Activation::Linear => "Linear",
        }
    }

    /// Apakah aktivasi ini bisa dilatih dengan perambatan balik.
    pub fn is_differentiable(self) -> bool {
        !matches!(self, Activation::Step)
    }
}

// ---------------------------------------------------------------------------
// Perceptron satu lapis
// ---------------------------------------------------------------------------

/// Perceptron satu lapis (Rosenblatt, 1958).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Perceptron {
    /// Bobot tiap masukan.
    pub weights: Vec<f64>,
    /// Bias.
    pub bias: f64,
    /// Laju belajar.
    pub learning_rate: f64,
    /// Fungsi aktivasi.
    pub activation: Activation,
}

/// Catatan satu epoch pelatihan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpochRecord {
    /// Nomor epoch, mulai dari satu.
    pub epoch: usize,
    /// Galat kuadrat rata-rata pada seluruh data latih.
    pub loss: f64,
    /// Ketepatan klasifikasi, `0.0` sampai `1.0`.
    pub accuracy: f64,
}

impl Perceptron {
    /// Perceptron baru dengan bobot acak kecil.
    pub fn new(
        inputs: usize,
        learning_rate: f64,
        activation: Activation,
        seed: u64,
    ) -> Result<Self, NeuralError> {
        if inputs == 0 {
            return Err(NeuralError::BadArchitecture(
                "perceptron butuh minimal satu masukan".into(),
            ));
        }
        if !learning_rate.is_finite() || learning_rate <= 0.0 {
            return Err(NeuralError::BadLearningRate(learning_rate));
        }
        let mut rng = SplitMix64::new(seed);
        Ok(Self {
            weights: (0..inputs).map(|_| rng.range_f64(-0.5, 0.5)).collect(),
            bias: rng.range_f64(-0.5, 0.5),
            learning_rate,
            activation,
        })
    }

    /// Jumlah masukan yang diterima.
    pub fn input_size(&self) -> usize {
        self.weights.len()
    }

    /// Jumlah berbobot sebelum aktivasi.
    pub fn net_input(&self, x: &[f64]) -> Result<f64, NeuralError> {
        if x.len() != self.weights.len() {
            return Err(NeuralError::InputSizeMismatch {
                expected: self.weights.len(),
                got: x.len(),
            });
        }
        Ok(self.weights.iter().zip(x).map(|(w, v)| w * v).sum::<f64>() + self.bias)
    }

    /// Keluaran setelah aktivasi.
    pub fn predict(&self, x: &[f64]) -> Result<f64, NeuralError> {
        Ok(self.activation.apply(self.net_input(x)?))
    }

    /// Melatih satu epoch dengan aturan delta, mengembalikan galat rata-rata.
    pub fn train_epoch(&mut self, x: &[Vec<f64>], y: &[f64]) -> Result<f64, NeuralError> {
        if x.is_empty() {
            return Err(NeuralError::EmptyDataset);
        }
        if x.len() != y.len() {
            return Err(NeuralError::DatasetMismatch {
                inputs: x.len(),
                targets: y.len(),
            });
        }
        let mut total = 0.0;
        for (row, target) in x.iter().zip(y) {
            let output = self.predict(row)?;
            let error = target - output;
            total += error * error;
            let gradient = self.activation.derivative_from_output(output);
            let delta = self.learning_rate * error * gradient;
            for (w, v) in self.weights.iter_mut().zip(row) {
                *w += delta * v;
            }
            self.bias += delta;
        }
        Ok(total / x.len() as f64)
    }

    /// Melatih beberapa epoch, berhenti lebih awal bila galatnya sudah cukup kecil.
    pub fn train(
        &mut self,
        x: &[Vec<f64>],
        y: &[f64],
        epochs: usize,
        tolerance: f64,
    ) -> Result<Vec<EpochRecord>, NeuralError> {
        let mut history = Vec::with_capacity(epochs);
        for epoch in 1..=epochs {
            let loss = self.train_epoch(x, y)?;
            if !loss.is_finite() {
                return Err(NeuralError::Diverged { epoch });
            }
            let accuracy = self.accuracy(x, y)?;
            history.push(EpochRecord {
                epoch,
                loss,
                accuracy,
            });
            if loss <= tolerance {
                break;
            }
        }
        Ok(history)
    }

    /// Ketepatan klasifikasi biner dengan ambang di titik tengah.
    pub fn accuracy(&self, x: &[Vec<f64>], y: &[f64]) -> Result<f64, NeuralError> {
        if x.is_empty() {
            return Err(NeuralError::EmptyDataset);
        }
        if x.len() != y.len() {
            return Err(NeuralError::DatasetMismatch {
                inputs: x.len(),
                targets: y.len(),
            });
        }
        // Ambang mengikuti rentang aktivasi: nol untuk tanh, setengah untuk sisanya.
        let threshold = if self.activation == Activation::Tanh {
            0.0
        } else {
            0.5
        };
        let mut correct = 0usize;
        for (row, target) in x.iter().zip(y) {
            let predicted = self.predict(row)? >= threshold;
            if predicted == (*target >= threshold) {
                correct += 1;
            }
        }
        Ok(correct as f64 / x.len() as f64)
    }
}

// ---------------------------------------------------------------------------
// Perceptron banyak lapis
// ---------------------------------------------------------------------------

/// Satu lapisan tersembunyi atau keluaran.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    /// Bobot, `[neuron][masukan]`.
    pub weights: Vec<Vec<f64>>,
    /// Bias tiap neuron.
    pub biases: Vec<f64>,
    /// Fungsi aktivasi lapisan ini.
    pub activation: Activation,
}

impl Layer {
    /// Keluaran lapisan untuk sebuah masukan.
    pub fn forward(&self, x: &[f64]) -> Vec<f64> {
        self.weights
            .iter()
            .zip(&self.biases)
            .map(|(w, b)| {
                let net = w.iter().zip(x).map(|(wi, xi)| wi * xi).sum::<f64>() + b;
                self.activation.apply(net)
            })
            .collect()
    }
}

/// Jaringan berlapis yang dilatih dengan perambatan balik.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Network {
    /// Lapisan-lapisan, dari yang terdekat dengan masukan.
    pub layers: Vec<Layer>,
    /// Laju belajar.
    pub learning_rate: f64,
    /// Momentum, `0.0` berarti tidak dipakai.
    pub momentum: f64,
    /// Kecepatan perubahan bobot dari langkah sebelumnya.
    #[serde(skip)]
    velocity_w: Vec<Vec<Vec<f64>>>,
    /// Kecepatan perubahan bias dari langkah sebelumnya.
    #[serde(skip)]
    velocity_b: Vec<Vec<f64>>,
}

impl Network {
    /// Membangun jaringan dari ukuran tiap lapisan.
    ///
    /// `sizes` memuat jumlah masukan di posisi pertama, lalu ukuran tiap
    /// lapisan berikutnya. Bobot awal memakai penskalaan Xavier, yang menjaga
    /// besar sinyal tetap wajar saat melewati banyak lapisan.
    pub fn new(
        sizes: &[usize],
        activation: Activation,
        output_activation: Activation,
        learning_rate: f64,
        momentum: f64,
        seed: u64,
    ) -> Result<Self, NeuralError> {
        if sizes.len() < 2 {
            return Err(NeuralError::BadArchitecture(
                "butuh minimal lapisan masukan dan keluaran".into(),
            ));
        }
        if sizes.contains(&0) {
            return Err(NeuralError::BadArchitecture(
                "tidak boleh ada lapisan berukuran nol".into(),
            ));
        }
        if !learning_rate.is_finite() || learning_rate <= 0.0 {
            return Err(NeuralError::BadLearningRate(learning_rate));
        }
        if !activation.is_differentiable() || !output_activation.is_differentiable() {
            return Err(NeuralError::BadArchitecture(
                "aktivasi ambang keras tidak bisa dilatih perambatan balik".into(),
            ));
        }

        let mut rng = SplitMix64::new(seed);
        let mut layers = Vec::with_capacity(sizes.len() - 1);
        for i in 1..sizes.len() {
            let fan_in = sizes[i - 1];
            let fan_out = sizes[i];
            // Xavier: sebaran seragam pada +/- sqrt(6 / (fan_in + fan_out)).
            let limit = (6.0 / (fan_in + fan_out) as f64).sqrt();
            let act = if i == sizes.len() - 1 {
                output_activation
            } else {
                activation
            };
            layers.push(Layer {
                weights: (0..fan_out)
                    .map(|_| (0..fan_in).map(|_| rng.range_f64(-limit, limit)).collect())
                    .collect(),
                biases: vec![0.0; fan_out],
                activation: act,
            });
        }

        let velocity_w = layers
            .iter()
            .map(|l| l.weights.iter().map(|w| vec![0.0; w.len()]).collect())
            .collect();
        let velocity_b = layers.iter().map(|l| vec![0.0; l.biases.len()]).collect();

        Ok(Self {
            layers,
            learning_rate,
            momentum: momentum.clamp(0.0, 0.99),
            velocity_w,
            velocity_b,
        })
    }

    /// Jumlah masukan yang diterima jaringan.
    pub fn input_size(&self) -> usize {
        self.layers.first().map(|l| l.weights[0].len()).unwrap_or(0)
    }

    /// Jumlah keluaran jaringan.
    pub fn output_size(&self) -> usize {
        self.layers.last().map(|l| l.biases.len()).unwrap_or(0)
    }

    /// Laju belajar efektif setelah memperhitungkan momentum.
    ///
    /// Momentum menumpuk langkah-langkah sebelumnya, sehingga pada keadaan
    /// tunak ukuran langkahnya mendekati `laju / (1 - momentum)`. Pada momentum
    /// 0,9 itu berarti **sepuluh kali** laju yang tertulis.
    ///
    /// Angka ini penting ditampilkan, bukan disembunyikan. Pengukuran pada
    /// kumpulan data spiral menunjukkan jaringan yang sama mencapai ketepatan
    /// penuh dalam 70 epoch pada laju 0,08 dengan momentum 0,9, tetapi macet di
    /// 50 persen pada laju 0,2 dengan momentum yang sama — bukan karena
    /// arsitekturnya kurang, melainkan karena langkah efektifnya menjadi 2,0.
    pub fn effective_learning_rate(&self) -> f64 {
        let denom = 1.0 - self.momentum;
        if denom <= f64::MIN_POSITIVE {
            f64::INFINITY
        } else {
            self.learning_rate / denom
        }
    }

    /// Apakah langkah efektifnya berada di wilayah yang cenderung menyimpang.
    ///
    /// Ambangnya adalah pengalaman, bukan teorema: di atas nilai ini pelatihan
    /// pada kumpulan data di modul ini mulai berayun alih-alih menurun.
    pub fn is_step_risky(&self) -> bool {
        self.effective_learning_rate() > RISKY_EFFECTIVE_RATE
    }

    /// Total bobot dan bias yang dapat dilatih.
    pub fn parameter_count(&self) -> usize {
        self.layers
            .iter()
            .map(|l| l.weights.iter().map(|w| w.len()).sum::<usize>() + l.biases.len())
            .sum()
    }

    /// Keluaran jaringan untuk sebuah masukan.
    pub fn predict(&self, x: &[f64]) -> Result<Vec<f64>, NeuralError> {
        if x.len() != self.input_size() {
            return Err(NeuralError::InputSizeMismatch {
                expected: self.input_size(),
                got: x.len(),
            });
        }
        let mut current = x.to_vec();
        for layer in &self.layers {
            current = layer.forward(&current);
        }
        Ok(current)
    }

    /// Keluaran tiap lapisan, dipakai perambatan balik dan peragaan.
    pub fn forward_all(&self, x: &[f64]) -> Result<Vec<Vec<f64>>, NeuralError> {
        if x.len() != self.input_size() {
            return Err(NeuralError::InputSizeMismatch {
                expected: self.input_size(),
                got: x.len(),
            });
        }
        let mut outputs = Vec::with_capacity(self.layers.len() + 1);
        outputs.push(x.to_vec());
        for layer in &self.layers {
            let last = outputs.last().expect("selalu ada minimal masukan");
            outputs.push(layer.forward(last));
        }
        Ok(outputs)
    }

    /// Menyiapkan penyangga momentum bila bentuknya belum cocok.
    ///
    /// Kecepatan perubahan bobot tidak ikut diserialisasi, karena nilainya
    /// hanya bermakna di tengah satu sesi pelatihan. Akibatnya jaringan yang
    /// dibaca kembali dari JSON datang dengan penyangga kosong, dan langsung
    /// melatihnya akan mengindeks di luar batas.
    ///
    /// Jalur itu bukan kasus pinggiran: antarmuka menyimpan jaringan sebagai
    /// JSON di antara potongan pelatihan supaya tampilan tidak membeku, jadi
    /// setiap potongan setelah yang pertama melewatinya.
    fn ensure_velocity(&mut self) {
        let cocok = self.velocity_w.len() == self.layers.len()
            && self.velocity_w.iter().zip(&self.layers).all(|(v, l)| {
                v.len() == l.weights.len()
                    && v.iter()
                        .zip(&l.weights)
                        .all(|(vr, wr)| vr.len() == wr.len())
            })
            && self.velocity_b.len() == self.layers.len()
            && self
                .velocity_b
                .iter()
                .zip(&self.layers)
                .all(|(v, l)| v.len() == l.biases.len());
        if cocok {
            return;
        }
        self.velocity_w = self
            .layers
            .iter()
            .map(|l| l.weights.iter().map(|w| vec![0.0; w.len()]).collect())
            .collect();
        self.velocity_b = self
            .layers
            .iter()
            .map(|l| vec![0.0; l.biases.len()])
            .collect();
    }

    /// Satu langkah perambatan balik untuk sebuah contoh.
    ///
    /// Mengembalikan galat kuadrat contoh itu.
    pub fn backpropagate(&mut self, x: &[f64], target: &[f64]) -> Result<f64, NeuralError> {
        self.ensure_velocity();
        if target.len() != self.output_size() {
            return Err(NeuralError::TargetSizeMismatch {
                expected: self.output_size(),
                got: target.len(),
            });
        }
        let outputs = self.forward_all(x)?;
        let last = self.layers.len() - 1;

        // Delta lapisan keluaran: (target - keluaran) x turunan aktivasi.
        let mut deltas: Vec<Vec<f64>> = vec![Vec::new(); self.layers.len()];
        let mut loss = 0.0;
        deltas[last] = outputs[last + 1]
            .iter()
            .zip(target)
            .map(|(o, t)| {
                let e = t - o;
                loss += e * e;
                e * self.layers[last].activation.derivative_from_output(*o)
            })
            .collect();

        // Delta lapisan tersembunyi, dirambatkan mundur.
        for i in (0..last).rev() {
            let next = &self.layers[i + 1];
            let next_delta = deltas[i + 1].clone();
            deltas[i] = outputs[i + 1]
                .iter()
                .enumerate()
                .map(|(j, o)| {
                    let sum: f64 = next
                        .weights
                        .iter()
                        .zip(&next_delta)
                        .map(|(w, d)| w[j] * d)
                        .sum();
                    sum * self.layers[i].activation.derivative_from_output(*o)
                })
                .collect();
        }

        // Pembaruan bobot dengan momentum.
        let lr = self.learning_rate;
        let momentum = self.momentum;
        for (i, layer) in self.layers.iter_mut().enumerate() {
            let input = &outputs[i];
            let layer_delta = &deltas[i];
            let vw = &mut self.velocity_w[i];
            let vb = &mut self.velocity_b[i];
            for (j, (row, bias)) in layer
                .weights
                .iter_mut()
                .zip(layer.biases.iter_mut())
                .enumerate()
            {
                let d = layer_delta[j];
                for (k, w) in row.iter_mut().enumerate() {
                    let v = momentum * vw[j][k] + lr * d * input[k];
                    vw[j][k] = v;
                    *w += v;
                }
                let v = momentum * vb[j] + lr * d;
                vb[j] = v;
                *bias += v;
            }
        }

        Ok(loss / target.len() as f64)
    }

    /// Melatih satu epoch pada seluruh data.
    pub fn train_epoch(&mut self, x: &[Vec<f64>], y: &[Vec<f64>]) -> Result<f64, NeuralError> {
        if x.is_empty() {
            return Err(NeuralError::EmptyDataset);
        }
        if x.len() != y.len() {
            return Err(NeuralError::DatasetMismatch {
                inputs: x.len(),
                targets: y.len(),
            });
        }
        let mut total = 0.0;
        for (row, target) in x.iter().zip(y) {
            total += self.backpropagate(row, target)?;
        }
        Ok(total / x.len() as f64)
    }

    /// Melatih beberapa epoch dengan pengacakan urutan tiap epoch.
    pub fn train(
        &mut self,
        x: &[Vec<f64>],
        y: &[Vec<f64>],
        epochs: usize,
        tolerance: f64,
        seed: u64,
    ) -> Result<Vec<EpochRecord>, NeuralError> {
        if x.is_empty() {
            return Err(NeuralError::EmptyDataset);
        }
        if x.len() != y.len() {
            return Err(NeuralError::DatasetMismatch {
                inputs: x.len(),
                targets: y.len(),
            });
        }
        let mut rng = SplitMix64::new(seed);
        let mut order: Vec<usize> = (0..x.len()).collect();
        let mut history = Vec::with_capacity(epochs);

        for epoch in 1..=epochs {
            // Urutan diacak tiap epoch agar jaringan tidak menghafal urutannya,
            // tetapi pengacaknya berbenih tetap supaya hasilnya reproduktif.
            rng.shuffle(&mut order);
            let mut total = 0.0;
            for &i in &order {
                total += self.backpropagate(&x[i], &y[i])?;
            }
            let loss = total / x.len() as f64;
            if !loss.is_finite() {
                return Err(NeuralError::Diverged { epoch });
            }
            let accuracy = self.accuracy(x, y)?;
            history.push(EpochRecord {
                epoch,
                loss,
                accuracy,
            });
            if loss <= tolerance {
                break;
            }
        }
        Ok(history)
    }

    /// Ketepatan klasifikasi: keluaran terbesar dianggap kelas terpilih.
    ///
    /// Untuk keluaran tunggal, dipakai ambang di titik tengah rentang aktivasi.
    pub fn accuracy(&self, x: &[Vec<f64>], y: &[Vec<f64>]) -> Result<f64, NeuralError> {
        if x.is_empty() {
            return Err(NeuralError::EmptyDataset);
        }
        if x.len() != y.len() {
            return Err(NeuralError::DatasetMismatch {
                inputs: x.len(),
                targets: y.len(),
            });
        }
        let threshold = if self.layers.last().map(|l| l.activation) == Some(Activation::Tanh) {
            0.0
        } else {
            0.5
        };
        let mut correct = 0usize;
        for (row, target) in x.iter().zip(y) {
            let out = self.predict(row)?;
            let hit = if out.len() == 1 {
                (out[0] >= threshold) == (target[0] >= threshold)
            } else {
                argmax(&out) == argmax(target)
            };
            if hit {
                correct += 1;
            }
        }
        Ok(correct as f64 / x.len() as f64)
    }
}

/// Indeks nilai terbesar. Seri diputus oleh indeks terkecil.
pub fn argmax(values: &[f64]) -> usize {
    let mut best = 0usize;
    let mut best_value = f64::NEG_INFINITY;
    for (i, v) in values.iter().enumerate() {
        if *v > best_value {
            best_value = *v;
            best = i;
        }
    }
    best
}

/// Kumpulan data XOR — contoh klasik yang tidak bisa dipisahkan satu garis.
pub fn xor_dataset() -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    (
        vec![
            vec![0.0, 0.0],
            vec![0.0, 1.0],
            vec![1.0, 0.0],
            vec![1.0, 1.0],
        ],
        vec![vec![0.0], vec![1.0], vec![1.0], vec![0.0]],
    )
}

/// Kumpulan data AND — bisa dipisahkan satu garis.
pub fn and_dataset() -> (Vec<Vec<f64>>, Vec<f64>) {
    (
        vec![
            vec![0.0, 0.0],
            vec![0.0, 1.0],
            vec![1.0, 0.0],
            vec![1.0, 1.0],
        ],
        vec![0.0, 0.0, 0.0, 1.0],
    )
}

/// Kumpulan data OR — bisa dipisahkan satu garis.
pub fn or_dataset() -> (Vec<Vec<f64>>, Vec<f64>) {
    (
        vec![
            vec![0.0, 0.0],
            vec![0.0, 1.0],
            vec![1.0, 0.0],
            vec![1.0, 1.0],
        ],
        vec![0.0, 1.0, 1.0, 1.0],
    )
}

/// Jari-jari terdalam spiral.
///
/// Sengaja bukan nol. Kalau kedua lengan spiral dimulai dari titik asal, titik
/// terdalam kedua kelas menumpuk di koordinat yang sama persis dan menjadi
/// mustahil dibedakan — berapa pun besar jaringannya. Galat yang tersisa itu
/// bukan kekurangan model, melainkan cacat kumpulan datanya, dan akan
/// menyesatkan siapa pun yang memakai angka ketepatan untuk menilai jaringan.
pub const SPIRAL_INNER_RADIUS: f64 = 0.15;

/// Dua gugus melingkar yang saling melilit, tidak terpisahkan garis lurus.
pub fn spiral_dataset(
    points_per_class: usize,
    noise: f64,
    seed: u64,
) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let mut rng = SplitMix64::new(seed);
    let mut x = Vec::with_capacity(points_per_class * 2);
    let mut y = Vec::with_capacity(points_per_class * 2);
    for class in 0..2usize {
        for i in 0..points_per_class {
            let t01 = i as f64 / points_per_class.max(1) as f64;
            let r = SPIRAL_INNER_RADIUS + (1.0 - SPIRAL_INNER_RADIUS) * t01;
            let t = 1.75 * t01 * core::f64::consts::TAU / 2.0
                + class as f64 * core::f64::consts::PI
                + rng.next_gaussian() * noise;
            x.push(vec![r * t.sin(), r * t.cos()]);
            y.push(if class == 0 {
                vec![1.0, 0.0]
            } else {
                vec![0.0, 1.0]
            });
        }
    }
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }

    #[test]
    fn aktivasi_nilai_yang_dikenal() {
        close(Activation::Step.apply(0.5), 1.0);
        close(Activation::Step.apply(-0.5), 0.0);
        close(Activation::Step.apply(0.0), 1.0);
        close(Activation::Sigmoid.apply(0.0), 0.5);
        close(Activation::Tanh.apply(0.0), 0.0);
        close(Activation::Relu.apply(-3.0), 0.0);
        close(Activation::Relu.apply(3.0), 3.0);
        close(Activation::LeakyRelu.apply(-1.0), -LEAKY_SLOPE);
        close(Activation::Linear.apply(-7.5), -7.5);
    }

    #[test]
    fn sigmoid_stabil_untuk_masukan_ekstrem() {
        // Bentuk naif meluap dan menghasilkan bukan bilangan di sini.
        for x in [-800.0, -100.0, 100.0, 800.0] {
            let y = Activation::Sigmoid.apply(x);
            assert!(y.is_finite(), "sigmoid({x}) = {y}");
            assert!((0.0..=1.0).contains(&y));
        }
        assert!(Activation::Sigmoid.apply(800.0) > 0.999);
        assert!(Activation::Sigmoid.apply(-800.0) < 0.001);
    }

    #[test]
    fn sigmoid_setangkup_terhadap_titik_tengah() {
        for x in [0.5, 1.0, 3.0, 7.0] {
            close(
                Activation::Sigmoid.apply(x) + Activation::Sigmoid.apply(-x),
                1.0,
            );
        }
    }

    #[test]
    fn turunan_cocok_dengan_selisih_hingga() {
        // Membandingkan turunan analitik dengan selisih hingga adalah cara
        // paling jujur untuk memeriksa gradien: rumus yang salah tetap
        // menghasilkan angka, hanya angka yang keliru.
        let h = 1e-6;
        for act in [
            Activation::Sigmoid,
            Activation::Tanh,
            Activation::Relu,
            Activation::LeakyRelu,
            Activation::Linear,
        ] {
            for x in [-2.0, -0.5, 0.5, 2.0] {
                let numeric = (act.apply(x + h) - act.apply(x - h)) / (2.0 * h);
                let analytic = act.derivative_from_output(act.apply(x));
                assert!(
                    (numeric - analytic).abs() < 1e-4,
                    "{}: pada x={x} numerik {numeric} vs analitik {analytic}",
                    act.short_name()
                );
            }
        }
    }

    #[test]
    fn aktivasi_ambang_ditandai_tak_terdiferensialkan() {
        assert!(!Activation::Step.is_differentiable());
        for act in [
            Activation::Sigmoid,
            Activation::Tanh,
            Activation::Relu,
            Activation::LeakyRelu,
            Activation::Linear,
        ] {
            assert!(act.is_differentiable());
            assert!(!act.short_name().is_empty());
        }
    }

    #[test]
    fn perceptron_menolak_arsitektur_tak_sah() {
        assert!(matches!(
            Perceptron::new(0, 0.1, Activation::Step, 1),
            Err(NeuralError::BadArchitecture(_))
        ));
        assert!(matches!(
            Perceptron::new(2, 0.0, Activation::Step, 1),
            Err(NeuralError::BadLearningRate(_))
        ));
        assert!(matches!(
            Perceptron::new(2, -1.0, Activation::Step, 1),
            Err(NeuralError::BadLearningRate(_))
        ));
        assert!(matches!(
            Perceptron::new(2, f64::NAN, Activation::Step, 1),
            Err(NeuralError::BadLearningRate(_))
        ));
    }

    #[test]
    fn perceptron_deterministik_untuk_benih_sama() {
        let a = Perceptron::new(3, 0.1, Activation::Step, 42).unwrap();
        let b = Perceptron::new(3, 0.1, Activation::Step, 42).unwrap();
        assert_eq!(a.weights, b.weights);
        close(a.bias, b.bias);
        let c = Perceptron::new(3, 0.1, Activation::Step, 43).unwrap();
        assert_ne!(a.weights, c.weights);
    }

    #[test]
    fn perceptron_menolak_masukan_salah_ukuran() {
        let p = Perceptron::new(2, 0.1, Activation::Step, 1).unwrap();
        assert_eq!(p.input_size(), 2);
        assert!(matches!(
            p.predict(&[1.0]),
            Err(NeuralError::InputSizeMismatch {
                expected: 2,
                got: 1
            })
        ));
    }

    #[test]
    fn perceptron_belajar_and() {
        let (x, y) = and_dataset();
        let mut p = Perceptron::new(2, 0.1, Activation::Step, 7).unwrap();
        let history = p.train(&x, &y, 200, 0.0).unwrap();
        assert!(!history.is_empty());
        close(p.accuracy(&x, &y).unwrap(), 1.0);
        for (row, target) in x.iter().zip(&y) {
            close(p.predict(row).unwrap(), *target);
        }
    }

    #[test]
    fn perceptron_belajar_or() {
        let (x, y) = or_dataset();
        let mut p = Perceptron::new(2, 0.1, Activation::Step, 11).unwrap();
        p.train(&x, &y, 200, 0.0).unwrap();
        close(p.accuracy(&x, &y).unwrap(), 1.0);
    }

    #[test]
    fn perceptron_gagal_pada_xor() {
        // Batas Minsky dan Papert (1969): satu lapis tidak bisa memisahkan XOR,
        // berapa pun lamanya dilatih. Ini bukan bug, ini alasan lapisan
        // tersembunyi ditemukan.
        let (x, y_vec) = xor_dataset();
        let y: Vec<f64> = y_vec.iter().map(|v| v[0]).collect();
        let mut p = Perceptron::new(2, 0.1, Activation::Step, 3).unwrap();
        p.train(&x, &y, 2_000, 0.0).unwrap();
        assert!(
            p.accuracy(&x, &y).unwrap() < 1.0,
            "perceptron satu lapis seharusnya tidak bisa memisahkan XOR"
        );
    }

    #[test]
    fn perceptron_menolak_data_tak_sepadan() {
        let mut p = Perceptron::new(2, 0.1, Activation::Step, 1).unwrap();
        assert_eq!(p.train_epoch(&[], &[]), Err(NeuralError::EmptyDataset));
        assert_eq!(
            p.train_epoch(&[vec![0.0, 0.0]], &[]),
            Err(NeuralError::DatasetMismatch {
                inputs: 1,
                targets: 0
            })
        );
        assert_eq!(p.accuracy(&[], &[]), Err(NeuralError::EmptyDataset));
        assert_eq!(
            p.accuracy(&[vec![0.0, 0.0]], &[]),
            Err(NeuralError::DatasetMismatch {
                inputs: 1,
                targets: 0
            })
        );
    }

    #[test]
    fn jaringan_menolak_arsitektur_tak_sah() {
        assert!(matches!(
            Network::new(&[2], Activation::Tanh, Activation::Sigmoid, 0.1, 0.0, 1),
            Err(NeuralError::BadArchitecture(_))
        ));
        assert!(matches!(
            Network::new(
                &[2, 0, 1],
                Activation::Tanh,
                Activation::Sigmoid,
                0.1,
                0.0,
                1
            ),
            Err(NeuralError::BadArchitecture(_))
        ));
        assert!(matches!(
            Network::new(
                &[2, 3, 1],
                Activation::Step,
                Activation::Sigmoid,
                0.1,
                0.0,
                1
            ),
            Err(NeuralError::BadArchitecture(_))
        ));
        assert!(matches!(
            Network::new(&[2, 3, 1], Activation::Tanh, Activation::Step, 0.1, 0.0, 1),
            Err(NeuralError::BadArchitecture(_))
        ));
        assert!(matches!(
            Network::new(
                &[2, 3, 1],
                Activation::Tanh,
                Activation::Sigmoid,
                0.0,
                0.0,
                1
            ),
            Err(NeuralError::BadLearningRate(_))
        ));
    }

    #[test]
    fn bentuk_jaringan_benar() {
        let n = Network::new(
            &[2, 4, 3, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.1,
            0.9,
            1,
        )
        .unwrap();
        assert_eq!(n.input_size(), 2);
        assert_eq!(n.output_size(), 1);
        assert_eq!(n.layers.len(), 3);
        // (2x4 + 4) + (4x3 + 3) + (3x1 + 1) = 12 + 15 + 4 = 31
        assert_eq!(n.parameter_count(), 31);
        assert_eq!(n.predict(&[0.5, 0.5]).unwrap().len(), 1);
    }

    #[test]
    fn keluaran_tiap_lapisan_terekam() {
        let n = Network::new(
            &[2, 3, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.1,
            0.0,
            1,
        )
        .unwrap();
        let all = n.forward_all(&[0.2, 0.8]).unwrap();
        assert_eq!(all.len(), 3);
        assert_eq!(all[0], vec![0.2, 0.8]);
        assert_eq!(all[1].len(), 3);
        assert_eq!(all[2].len(), 1);
    }

    #[test]
    fn jaringan_menolak_ukuran_salah() {
        let mut n = Network::new(
            &[2, 3, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.1,
            0.0,
            1,
        )
        .unwrap();
        assert!(matches!(
            n.predict(&[1.0]),
            Err(NeuralError::InputSizeMismatch {
                expected: 2,
                got: 1
            })
        ));
        assert!(matches!(
            n.forward_all(&[1.0, 2.0, 3.0]),
            Err(NeuralError::InputSizeMismatch {
                expected: 2,
                got: 3
            })
        ));
        assert!(matches!(
            n.backpropagate(&[1.0, 2.0], &[1.0, 2.0]),
            Err(NeuralError::TargetSizeMismatch {
                expected: 1,
                got: 2
            })
        ));
    }

    #[test]
    fn gradien_perambatan_balik_cocok_dengan_selisih_hingga() {
        // Pemeriksaan gradien: satu-satunya uji yang benar-benar membuktikan
        // rumus perambatan balik. Galat yang menurun belum tentu berarti
        // gradiennya benar — jaringan yang gradiennya salah pun sering tetap
        // belajar, hanya lebih lambat dan berhenti di tempat yang keliru.
        //
        // Momentum dimatikan dan laju belajar dibuat sangat kecil supaya
        // perubahan bobot satu langkah bisa dibaca sebagai gradien.
        let lr = 1e-6;
        let mut net = Network::new(
            &[3, 4, 3, 2],
            Activation::Tanh,
            Activation::Sigmoid,
            lr,
            0.0,
            2026,
        )
        .unwrap();

        let x = vec![0.3, -0.7, 0.5];
        let target = vec![0.8, 0.2];

        /// Galat kuadrat rata-rata sebuah jaringan pada satu contoh.
        fn loss_of(net: &Network, x: &[f64], target: &[f64]) -> f64 {
            let out = net.predict(x).unwrap();
            out.iter()
                .zip(target)
                .map(|(o, t)| (t - o) * (t - o))
                .sum::<f64>()
                / target.len() as f64
        }

        let sebelum = net.clone();
        net.backpropagate(&x, &target).unwrap();

        let h = 1e-5;
        let mut diperiksa = 0usize;
        for li in 0..sebelum.layers.len() {
            for ni in 0..sebelum.layers[li].weights.len() {
                for wi in 0..sebelum.layers[li].weights[ni].len() {
                    // Gradien numerik dari galat terhadap bobot ini.
                    let mut naik = sebelum.clone();
                    naik.layers[li].weights[ni][wi] += h;
                    let mut turun = sebelum.clone();
                    turun.layers[li].weights[ni][wi] -= h;
                    let numerik =
                        (loss_of(&naik, &x, &target) - loss_of(&turun, &x, &target)) / (2.0 * h);

                    // Perambatan balik menaikkan bobot sebesar
                    // `lr * (-d(loss)/d(w))`, jadi gradiennya bisa dibaca
                    // kembali dari selisih bobot sebelum dan sesudah.
                    let selisih =
                        net.layers[li].weights[ni][wi] - sebelum.layers[li].weights[ni][wi];
                    let analitik = -selisih / lr;

                    let skala = numerik.abs().max(analitik.abs()).max(1e-8);
                    assert!(
                        (numerik - analitik).abs() / skala < 1e-3,
                        "lapisan {li} neuron {ni} bobot {wi}: numerik {numerik}, analitik {analitik}"
                    );
                    diperiksa += 1;
                }
            }
        }
        assert!(diperiksa >= 20, "hanya {diperiksa} bobot yang diperiksa");
    }

    #[test]
    fn jaringan_menyelesaikan_xor() {
        // Inti sesi ini: lapisan tersembunyi mengubah masalah yang mustahil
        // bagi satu lapis menjadi bisa dipelajari.
        let (x, y) = xor_dataset();
        let mut n = Network::new(
            &[2, 4, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.5,
            0.9,
            2026,
        )
        .unwrap();
        let history = n.train(&x, &y, 5_000, 1e-4, 7).unwrap();
        let last = history.last().expect("ada riwayat");
        assert!(
            last.accuracy >= 1.0,
            "ketepatan hanya {} setelah {} epoch",
            last.accuracy,
            history.len()
        );
        assert!(last.loss < 0.01, "galat masih {}", last.loss);
    }

    #[test]
    fn galat_menurun_selama_pelatihan() {
        let (x, y) = xor_dataset();
        let mut n = Network::new(
            &[2, 4, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.3,
            0.9,
            5,
        )
        .unwrap();
        let history = n.train(&x, &y, 1_500, 0.0, 3).unwrap();
        assert!(history.len() > 10);
        let awal = history[0].loss;
        let akhir = history.last().unwrap().loss;
        assert!(akhir < awal, "galat naik dari {awal} menjadi {akhir}");
    }

    #[test]
    fn pelatihan_deterministik_untuk_benih_sama() {
        let (x, y) = xor_dataset();
        let jalan = || {
            let mut n = Network::new(
                &[2, 4, 1],
                Activation::Tanh,
                Activation::Sigmoid,
                0.3,
                0.9,
                99,
            )
            .unwrap();
            let h = n.train(&x, &y, 300, 0.0, 13).unwrap();
            (n.layers[0].weights.clone(), h.last().unwrap().loss)
        };
        let (wa, la) = jalan();
        let (wb, lb) = jalan();
        assert_eq!(wa, wb);
        close(la, lb);
    }

    #[test]
    fn benih_berbeda_menghasilkan_bobot_berbeda() {
        let a = Network::new(
            &[2, 4, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.3,
            0.0,
            1,
        )
        .unwrap();
        let b = Network::new(
            &[2, 4, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.3,
            0.0,
            2,
        )
        .unwrap();
        assert_ne!(a.layers[0].weights, b.layers[0].weights);
    }

    #[test]
    fn momentum_dijepit_ke_rentang_wajar() {
        let n = Network::new(
            &[2, 2, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.1,
            5.0,
            1,
        )
        .unwrap();
        assert!(n.momentum <= 0.99);
        let m = Network::new(
            &[2, 2, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.1,
            -1.0,
            1,
        )
        .unwrap();
        assert_eq!(m.momentum, 0.0);
    }

    #[test]
    fn bobot_awal_xavier_berada_di_rentang_yang_benar() {
        let n = Network::new(
            &[4, 6, 2],
            Activation::Tanh,
            Activation::Sigmoid,
            0.1,
            0.0,
            1,
        )
        .unwrap();
        let limit = (6.0f64 / (4.0 + 6.0)).sqrt();
        for row in &n.layers[0].weights {
            for w in row {
                assert!(w.abs() <= limit + 1e-12, "bobot {w} melebihi batas {limit}");
            }
        }
        // Bias dimulai dari nol supaya keluaran awal tidak condong ke satu sisi.
        assert!(n.layers[0].biases.iter().all(|b| *b == 0.0));
    }

    #[test]
    fn jaringan_menolak_data_tak_sepadan() {
        let mut n = Network::new(
            &[2, 3, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.1,
            0.0,
            1,
        )
        .unwrap();
        assert_eq!(n.train_epoch(&[], &[]), Err(NeuralError::EmptyDataset));
        assert_eq!(
            n.train_epoch(&[vec![0.0, 0.0]], &[]),
            Err(NeuralError::DatasetMismatch {
                inputs: 1,
                targets: 0
            })
        );
        assert_eq!(
            n.train(&[], &[], 10, 0.0, 1),
            Err(NeuralError::EmptyDataset)
        );
        assert_eq!(n.accuracy(&[], &[]), Err(NeuralError::EmptyDataset));
    }

    #[test]
    fn argmax_memilih_indeks_terbesar() {
        assert_eq!(argmax(&[0.1, 0.9, 0.3]), 1);
        assert_eq!(argmax(&[0.5]), 0);
        assert_eq!(argmax(&[]), 0);
        // Seri diputus oleh indeks terkecil, supaya hasilnya deterministik.
        assert_eq!(argmax(&[0.5, 0.5, 0.2]), 0);
    }

    #[test]
    fn kumpulan_data_logika_berbentuk_benar() {
        let (x, y) = xor_dataset();
        assert_eq!(x.len(), 4);
        assert_eq!(y.len(), 4);
        assert!(x.iter().all(|r| r.len() == 2));
        assert!(y.iter().all(|r| r.len() == 1));

        let (xa, ya) = and_dataset();
        assert_eq!(xa.len(), 4);
        assert_eq!(ya, vec![0.0, 0.0, 0.0, 1.0]);

        let (xo, yo) = or_dataset();
        assert_eq!(xo.len(), 4);
        assert_eq!(yo, vec![0.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn spiral_berbentuk_benar_dan_seimbang() {
        let (x, y) = spiral_dataset(50, 0.05, 1);
        assert_eq!(x.len(), 100);
        assert_eq!(y.len(), 100);
        assert!(x
            .iter()
            .all(|r| r.len() == 2 && r.iter().all(|v| v.is_finite())));
        let kelas_a = y.iter().filter(|t| t[0] > t[1]).count();
        assert_eq!(kelas_a, 50, "kedua kelas harus sama banyak");
    }

    #[test]
    fn spiral_deterministik() {
        let (a, _) = spiral_dataset(20, 0.05, 9);
        let (b, _) = spiral_dataset(20, 0.05, 9);
        assert_eq!(a, b);
        let (c, _) = spiral_dataset(20, 0.05, 10);
        assert_ne!(a, c);
    }

    #[test]
    fn spiral_kedua_lengan_tidak_menumpuk_di_pusat() {
        // Regresi: jari-jari yang dimulai dari nol membuat titik terdalam kedua
        // kelas berimpit, sehingga sebagian data mustahil dibedakan dan angka
        // ketepatan menjadi menyesatkan.
        let (x, y) = spiral_dataset(80, 0.0, 3);
        let mut terdekat = f64::INFINITY;
        for (i, a) in x.iter().enumerate() {
            for (j, b) in x.iter().enumerate() {
                // Hanya pasangan dari kelas berbeda yang diperiksa.
                if (y[i][0] > y[i][1]) == (y[j][0] > y[j][1]) {
                    continue;
                }
                let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt();
                terdekat = terdekat.min(d);
            }
        }
        assert!(
            terdekat > 0.05,
            "titik antarkelas terlalu berdekatan: {terdekat}"
        );
        // Seluruh titik berada di luar lingkaran dalam.
        for p in &x {
            let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
            assert!(
                r >= SPIRAL_INNER_RADIUS - 1e-9,
                "ada titik di jari-jari {r}"
            );
        }
    }

    #[test]
    fn spiral_dengan_nol_titik_tidak_membagi_nol() {
        let (x, y) = spiral_dataset(0, 0.05, 1);
        assert!(x.is_empty());
        assert!(y.is_empty());
    }

    #[test]
    fn jaringan_belajar_memisahkan_spiral() {
        let (x, y) = spiral_dataset(60, 0.03, 4);
        let mut n = Network::new(
            &[2, 16, 16, 2],
            Activation::Tanh,
            Activation::Sigmoid,
            0.08,
            0.9,
            5,
        )
        .unwrap();
        let history = n.train(&x, &y, 2_000, 1e-4, 8).unwrap();
        let akhir = history.last().unwrap();
        assert!(
            akhir.accuracy >= 1.0,
            "ketepatan hanya {} setelah {} epoch",
            akhir.accuracy,
            history.len()
        );
        // Pengukuran menunjukkan susunan ini selesai dalam sekitar 70 epoch.
        // Batas longgar dipakai agar uji tidak rapuh, tetapi tetap menangkap
        // regresi yang membuat pelatihan melambat berkali lipat.
        assert!(
            history.len() < 500,
            "butuh {} epoch, jauh lebih lambat dari yang seharusnya",
            history.len()
        );
    }

    #[test]
    fn langkah_efektif_dilaporkan_dan_ditandai() {
        let aman = Network::new(
            &[2, 4, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.08,
            0.9,
            1,
        )
        .unwrap();
        // 0.08 / (1 - 0.9) = 0.8
        close(aman.effective_learning_rate(), 0.08 / (1.0 - 0.9));
        assert!(!aman.is_step_risky());

        let berisiko = Network::new(
            &[2, 4, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.2,
            0.9,
            1,
        )
        .unwrap();
        // 0.2 / (1 - 0.9) = 2.0, susunan yang terbukti macet di 50 persen.
        close(berisiko.effective_learning_rate(), 2.0);
        assert!(berisiko.is_step_risky());

        let tanpa_momentum = Network::new(
            &[2, 4, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.2,
            0.0,
            1,
        )
        .unwrap();
        close(tanpa_momentum.effective_learning_rate(), 0.2);
        assert!(!tanpa_momentum.is_step_risky());
    }

    #[test]
    fn momentum_maksimum_tidak_menghasilkan_pembagian_nol() {
        // Momentum dijepit di 0,99, jadi laju efektifnya besar tetapi berhingga.
        let n = Network::new(
            &[2, 4, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.1,
            1.0,
            1,
        )
        .unwrap();
        assert!(n.effective_learning_rate().is_finite());
        assert!(n.is_step_risky());
    }

    #[test]
    fn langkah_terlalu_besar_benar_benar_merusak_pelatihan() {
        // Uji ini memagari temuan pengukuran: kegagalan pada spiral bukan
        // karena arsitektur kurang, melainkan karena langkah efektifnya 2,0.
        let (x, y) = spiral_dataset(60, 0.03, 4);
        let mut buruk = Network::new(
            &[2, 16, 16, 2],
            Activation::Tanh,
            Activation::Sigmoid,
            0.2,
            0.9,
            5,
        )
        .unwrap();
        assert!(buruk.is_step_risky());
        let h = buruk.train(&x, &y, 2_000, 1e-4, 8).unwrap();
        assert!(
            h.last().unwrap().accuracy < 0.9,
            "susunan ini seharusnya gagal; kalau kini berhasil, catatan pengukuran perlu diperbarui"
        );
    }

    #[test]
    fn jaringan_hasil_serialisasi_bisa_langsung_dilatih() {
        // Regresi: kecepatan momentum tidak ikut diserialisasi, sehingga
        // jaringan yang dibaca kembali datang dengan penyangga kosong dan
        // melatihnya langsung mengindeks di luar batas. Antarmuka melewati
        // jalur ini pada setiap potongan pelatihan setelah yang pertama.
        let (x, y) = xor_dataset();
        let asli = Network::new(
            &[2, 4, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.1,
            0.9,
            1,
        )
        .unwrap();
        let json = serde_json::to_string(&asli).unwrap();
        let mut balik: Network = serde_json::from_str(&json).unwrap();

        assert!(
            balik.velocity_w.is_empty(),
            "prasyarat uji: penyangga kosong"
        );
        let h = balik.train(&x, &y, 20, 0.0, 3).unwrap();
        assert_eq!(h.len(), 20);
        assert!(h.iter().all(|r| r.loss.is_finite()));

        // Melanjutkan pelatihan berkali-kali lewat JSON harus tetap menurunkan
        // galat, sama seperti melatih tanpa terputus.
        let mut lanjut = balik.clone();
        let mut terakhir = h.last().unwrap().loss;
        for _ in 0..5 {
            let j = serde_json::to_string(&lanjut).unwrap();
            lanjut = serde_json::from_str(&j).unwrap();
            let potongan = lanjut.train(&x, &y, 200, 0.0, 3).unwrap();
            terakhir = potongan.last().unwrap().loss;
            assert!(terakhir.is_finite());
        }
        assert!(terakhir < h[0].loss, "galat tidak menurun antarpotongan");
    }

    #[test]
    fn penyangga_momentum_dibangun_ulang_saat_bentuknya_berubah() {
        let mut n = Network::new(
            &[2, 3, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.1,
            0.9,
            1,
        )
        .unwrap();
        n.velocity_w.clear();
        n.velocity_b.clear();
        n.ensure_velocity();
        assert_eq!(n.velocity_w.len(), n.layers.len());
        assert_eq!(n.velocity_w[0].len(), n.layers[0].weights.len());
        assert_eq!(n.velocity_w[0][0].len(), n.layers[0].weights[0].len());
        assert_eq!(n.velocity_b[0].len(), n.layers[0].biases.len());

        // Penyangga yang sudah benar tidak boleh dinolkan ulang, karena itu
        // akan menghapus momentum di tengah pelatihan.
        n.velocity_w[0][0][0] = 0.5;
        n.ensure_velocity();
        close(n.velocity_w[0][0][0], 0.5);
    }

    #[test]
    fn model_bisa_di_serialisasi() {
        let n = Network::new(
            &[2, 3, 1],
            Activation::Tanh,
            Activation::Sigmoid,
            0.1,
            0.9,
            1,
        )
        .unwrap();
        let json = serde_json::to_string(&n).unwrap();
        let balik: Network = serde_json::from_str(&json).unwrap();
        assert_eq!(balik.layers, n.layers);
        close(balik.learning_rate, n.learning_rate);

        let p = Perceptron::new(2, 0.1, Activation::Step, 1).unwrap();
        let pj = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<Perceptron>(&pj).unwrap(), p);
    }

    #[test]
    fn pesan_error_terbaca() {
        for e in [
            NeuralError::BadArchitecture("x".into()),
            NeuralError::InputSizeMismatch {
                expected: 2,
                got: 1,
            },
            NeuralError::TargetSizeMismatch {
                expected: 2,
                got: 1,
            },
            NeuralError::DatasetMismatch {
                inputs: 1,
                targets: 2,
            },
            NeuralError::EmptyDataset,
            NeuralError::BadLearningRate(-1.0),
            NeuralError::Diverged { epoch: 5 },
        ] {
            assert!(!e.to_string().is_empty());
        }
    }
}
