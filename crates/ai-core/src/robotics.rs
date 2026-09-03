//! Sesi 14 — Pengenalan Robotika.
//!
//! Kinematika penggerak diferensial, kendali PID, kinematika maju dan balik
//! lengan dua sendi, serta perencanaan lintasan dengan medan potensial.
//!
//! Bagian yang paling banyak mengajarkan sesuatu justru kegagalannya. Kendali
//! PID yang penguatannya salah tidak sekadar lambat — ia berayun makin lebar
//! sampai lepas kendali. Medan potensial punya cacat bawaan berupa minimum
//! lokal, tempat robot berhenti di depan rintangan berbentuk cekung padahal
//! tujuannya terlihat jelas. Keduanya diperagakan di sini, bukan disembunyikan.

use serde::{Deserialize, Serialize};

/// Kesalahan pada perhitungan robotika.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RoboticsError {
    /// Nilai parameter tidak masuk akal.
    BadParameter {
        /// Nama parameter.
        name: String,
        /// Nilai yang diberikan.
        value: f64,
    },
    /// Sasaran berada di luar jangkauan lengan.
    OutOfReach {
        /// Jarak sasaran dari pangkal.
        distance: f64,
        /// Jangkauan terjauh.
        max_reach: f64,
        /// Jangkauan terdekat.
        min_reach: f64,
    },
    /// Simulasi tidak mencapai tujuan dalam batas langkah.
    DidNotConverge {
        /// Jumlah langkah yang sudah dijalankan.
        steps: usize,
    },
}

impl crate::galat::Dijelaskan for RoboticsError {
    fn kode(&self) -> &'static str {
        match self {
            RoboticsError::BadParameter { .. } => "robot.parameter_tak_sah",
            RoboticsError::OutOfReach { .. } => "robot.di_luar_jangkauan",
            RoboticsError::DidNotConverge { .. } => "robot.tidak_konvergen",
        }
    }

    fn argumen(&self) -> Vec<String> {
        match self {
            RoboticsError::BadParameter { name, value } => {
                vec![name.clone(), value.to_string()]
            }
            RoboticsError::OutOfReach {
                distance,
                max_reach,
                min_reach,
            } => vec![
                format!("{distance:.3}"),
                format!("{min_reach:.3}"),
                format!("{max_reach:.3}"),
            ],
            RoboticsError::DidNotConverge { steps } => vec![steps.to_string()],
        }
    }
}

impl core::fmt::Display for RoboticsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RoboticsError::BadParameter { name, value } => {
                write!(f, "parameter {name} tidak sah: {value}")
            }
            RoboticsError::OutOfReach {
                distance,
                max_reach,
                min_reach,
            } => write!(
                f,
                "jarak {distance:.3} di luar jangkauan {min_reach:.3} sampai {max_reach:.3}"
            ),
            RoboticsError::DidNotConverge { steps } => {
                write!(f, "tidak mencapai tujuan dalam {steps} langkah")
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Kinematika penggerak diferensial
// ---------------------------------------------------------------------------

/// Kedudukan robot: posisi dan arah hadap.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pose {
    /// Koordinat mendatar.
    pub x: f64,
    /// Koordinat tegak.
    pub y: f64,
    /// Arah hadap dalam radian, diukur dari sumbu mendatar.
    pub theta: f64,
}

impl Pose {
    /// Kedudukan baru.
    pub fn new(x: f64, y: f64, theta: f64) -> Self {
        Self { x, y, theta }
    }

    /// Jarak lurus ke sebuah titik.
    pub fn distance_to(&self, x: f64, y: f64) -> f64 {
        ((self.x - x).powi(2) + (self.y - y).powi(2)).sqrt()
    }
}

/// Membawa sudut ke rentang `(-π, π]`.
///
/// Tanpa penormalan ini, selisih sudut antara `179°` dan `-179°` terbaca
/// sebagai `358°` alih-alih `2°`, dan robot akan berputar hampir satu putaran
/// penuh untuk koreksi yang sebenarnya sangat kecil.
pub fn normalise_angle(angle: f64) -> f64 {
    let tau = core::f64::consts::TAU;
    let mut a = angle % tau;
    if a > core::f64::consts::PI {
        a -= tau;
    } else if a <= -core::f64::consts::PI {
        a += tau;
    }
    a
}

/// Memperbarui kedudukan robot penggerak diferensial.
///
/// `wheel_base` adalah jarak antarroda. Kecepatan tiap roda dalam satuan
/// panjang per satuan waktu.
pub fn differential_step(
    pose: Pose,
    left_speed: f64,
    right_speed: f64,
    wheel_base: f64,
    dt: f64,
) -> Result<Pose, RoboticsError> {
    if !wheel_base.is_finite() || wheel_base <= 0.0 {
        return Err(RoboticsError::BadParameter {
            name: "wheel_base".into(),
            value: wheel_base,
        });
    }
    if !dt.is_finite() || dt <= 0.0 {
        return Err(RoboticsError::BadParameter {
            name: "dt".into(),
            value: dt,
        });
    }

    let linear = (right_speed + left_speed) / 2.0;
    let angular = (right_speed - left_speed) / wheel_base;

    // Kedua roda berkecepatan sama berarti lintasannya lurus. Rumus busur
    // membagi dengan kecepatan sudut, jadi kasus ini dipisahkan.
    if angular.abs() < 1e-12 {
        return Ok(Pose {
            x: pose.x + linear * pose.theta.cos() * dt,
            y: pose.y + linear * pose.theta.sin() * dt,
            theta: pose.theta,
        });
    }

    let radius = linear / angular;
    let theta_next = normalise_angle(pose.theta + angular * dt);
    Ok(Pose {
        x: pose.x + radius * (theta_next.sin() - pose.theta.sin()),
        y: pose.y - radius * (theta_next.cos() - pose.theta.cos()),
        theta: theta_next,
    })
}

// ---------------------------------------------------------------------------
// Kendali PID
// ---------------------------------------------------------------------------

/// Kendali proporsional-integral-turunan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pid {
    /// Penguatan proporsional.
    pub kp: f64,
    /// Penguatan integral.
    pub ki: f64,
    /// Penguatan turunan.
    pub kd: f64,
    /// Batas keluaran, dipakai dua arah.
    pub output_limit: f64,
    integral: f64,
    previous_error: f64,
    started: bool,
}

impl Pid {
    /// Kendali baru.
    pub fn new(kp: f64, ki: f64, kd: f64, output_limit: f64) -> Result<Self, RoboticsError> {
        for (name, value) in [("kp", kp), ("ki", ki), ("kd", kd)] {
            if !value.is_finite() {
                return Err(RoboticsError::BadParameter {
                    name: name.into(),
                    value,
                });
            }
        }
        if !output_limit.is_finite() || output_limit <= 0.0 {
            return Err(RoboticsError::BadParameter {
                name: "output_limit".into(),
                value: output_limit,
            });
        }
        Ok(Self {
            kp,
            ki,
            kd,
            output_limit,
            integral: 0.0,
            previous_error: 0.0,
            started: false,
        })
    }

    /// Mengembalikan kendali ke keadaan awal.
    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.previous_error = 0.0;
        self.started = false;
    }

    /// Menghitung keluaran kendali untuk sebuah galat.
    pub fn update(&mut self, error: f64, dt: f64) -> Result<f64, RoboticsError> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err(RoboticsError::BadParameter {
                name: "dt".into(),
                value: dt,
            });
        }

        // Turunan pada langkah pertama tidak punya makna: tidak ada galat
        // sebelumnya untuk dibandingkan. Memakai nol sebagai galat sebelumnya
        // menghasilkan lonjakan turunan yang besar dan menyesatkan.
        let derivative = if self.started {
            (error - self.previous_error) / dt
        } else {
            0.0
        };

        let raw_integral = self.integral + error * dt;
        let raw = self.kp * error + self.ki * raw_integral + self.kd * derivative;
        let output = raw.clamp(-self.output_limit, self.output_limit);

        // Penumpukan integral dicegah: bila keluarannya sudah mentok, menambah
        // integral hanya membuat kendali lambat pulih saat galatnya berbalik.
        // Tanpa ini, robot yang sempat jauh dari sasaran akan melampauinya
        // jauh setelah akhirnya sampai.
        if (raw - output).abs() < 1e-12 {
            self.integral = raw_integral;
        }

        self.previous_error = error;
        self.started = true;
        Ok(output)
    }

    /// Nilai integral yang tertumpuk saat ini.
    pub fn accumulated_integral(&self) -> f64 {
        self.integral
    }
}

/// Satu langkah simulasi kendali.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ControlStep {
    /// Waktu simulasi.
    pub time: f64,
    /// Nilai proses saat ini.
    pub value: f64,
    /// Galat terhadap sasaran.
    pub error: f64,
    /// Keluaran kendali.
    pub output: f64,
}

/// Hasil simulasi kendali PID pada sistem orde pertama.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControlRun {
    /// Jejak tiap langkah.
    pub steps: Vec<ControlStep>,
    /// Apakah sistemnya mencapai sasaran dan bertahan di sana.
    pub settled: bool,
    /// Waktu saat sistem pertama kali menetap, atau kosong bila tidak pernah.
    ///
    /// Sengaja bukan `f64` bertak hingga. JSON tidak punya lambang untuk tak
    /// hingga: `serde_json` menuliskannya sebagai `null`, lalu gagal
    /// membacanya kembali sebagai bilangan. Nilai yang tidak ada memang lebih
    /// jujur dinyatakan sebagai tidak ada.
    pub settling_time: Option<f64>,
    /// Lonjakan terbesar melampaui sasaran, dalam persen.
    pub overshoot_percent: f64,
    /// Galat pada langkah terakhir.
    pub final_error: f64,
}

/// Mensimulasikan kendali PID pada sistem orde pertama.
///
/// Sistemnya sederhana: nilai proses bergerak menuju keluaran kendali dengan
/// tetapan waktu tertentu. Cukup untuk memperlihatkan perbedaan antara
/// penguatan yang wajar dan yang membuat sistem berayun.
pub fn simulate_pid(
    pid: &mut Pid,
    setpoint: f64,
    initial: f64,
    time_constant: f64,
    dt: f64,
    steps: usize,
) -> Result<ControlRun, RoboticsError> {
    if !time_constant.is_finite() || time_constant <= 0.0 {
        return Err(RoboticsError::BadParameter {
            name: "time_constant".into(),
            value: time_constant,
        });
    }
    if steps == 0 {
        return Err(RoboticsError::BadParameter {
            name: "steps".into(),
            value: 0.0,
        });
    }

    pid.reset();
    let mut value = initial;
    let mut trace = Vec::with_capacity(steps);
    let span = (setpoint - initial).abs().max(1e-9);
    let mut peak_beyond = 0.0f64;
    let mut settling_time = f64::NAN;

    for i in 0..steps {
        let time = i as f64 * dt;
        let error = setpoint - value;
        let output = pid.update(error, dt)?;
        trace.push(ControlStep {
            time,
            value,
            error,
            output,
        });

        // Lonjakan diukur hanya pada sisi seberang sasaran.
        let beyond = if setpoint >= initial {
            value - setpoint
        } else {
            setpoint - value
        };
        peak_beyond = peak_beyond.max(beyond);

        if error.abs() <= 0.02 * span && settling_time.is_nan() {
            settling_time = time;
        } else if error.abs() > 0.02 * span {
            settling_time = f64::NAN;
        }

        // Sistem orde pertama: nilainya mengejar keluaran kendali.
        value += (output - value) / time_constant * dt;
        if !value.is_finite() {
            return Err(RoboticsError::DidNotConverge { steps: i + 1 });
        }
    }

    let final_error = setpoint - value;
    Ok(ControlRun {
        settled: !settling_time.is_nan(),
        settling_time: if settling_time.is_nan() {
            None
        } else {
            Some(settling_time)
        },
        overshoot_percent: (peak_beyond / span * 100.0).max(0.0),
        final_error,
        steps: trace,
    })
}

// ---------------------------------------------------------------------------
// Lengan dua sendi
// ---------------------------------------------------------------------------

/// Sudut kedua sendi lengan.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ArmAngles {
    /// Sudut sendi pangkal, radian.
    pub theta1: f64,
    /// Sudut sendi siku relatif terhadap lengan pertama, radian.
    pub theta2: f64,
}

/// Kinematika maju: posisi ujung lengan dari sudut sendinya.
pub fn forward_kinematics(
    angles: ArmAngles,
    length1: f64,
    length2: f64,
) -> Result<(f64, f64), RoboticsError> {
    for (name, value) in [("length1", length1), ("length2", length2)] {
        if !value.is_finite() || value <= 0.0 {
            return Err(RoboticsError::BadParameter {
                name: name.into(),
                value,
            });
        }
    }
    let x = length1 * angles.theta1.cos() + length2 * (angles.theta1 + angles.theta2).cos();
    let y = length1 * angles.theta1.sin() + length2 * (angles.theta1 + angles.theta2).sin();
    Ok((x, y))
}

/// Kinematika balik: sudut sendi agar ujung lengan mencapai sebuah titik.
///
/// Mengembalikan dua penyelesaian bila ada: siku ke atas dan siku ke bawah.
/// Keduanya sah, dan memilih salah satunya adalah keputusan perancang, bukan
/// keputusan matematika — biasanya yang paling sedikit menggerakkan sendi.
pub fn inverse_kinematics(
    x: f64,
    y: f64,
    length1: f64,
    length2: f64,
) -> Result<[ArmAngles; 2], RoboticsError> {
    for (name, value) in [("length1", length1), ("length2", length2)] {
        if !value.is_finite() || value <= 0.0 {
            return Err(RoboticsError::BadParameter {
                name: name.into(),
                value,
            });
        }
    }
    if !x.is_finite() || !y.is_finite() {
        return Err(RoboticsError::BadParameter {
            name: "target".into(),
            value: if x.is_finite() { y } else { x },
        });
    }

    let distance = (x * x + y * y).sqrt();
    let max_reach = length1 + length2;
    let min_reach = (length1 - length2).abs();
    // Toleransi kecil supaya titik tepat di batas jangkauan tidak ditolak
    // hanya karena pembulatan.
    if distance > max_reach + 1e-9 || distance < min_reach - 1e-9 {
        return Err(RoboticsError::OutOfReach {
            distance,
            max_reach,
            min_reach,
        });
    }

    let cos_theta2 = ((distance * distance - length1 * length1 - length2 * length2)
        / (2.0 * length1 * length2))
        .clamp(-1.0, 1.0);
    let theta2_down = cos_theta2.acos();
    let theta2_up = -theta2_down;

    let solve = |theta2: f64| -> ArmAngles {
        let k1 = length1 + length2 * theta2.cos();
        let k2 = length2 * theta2.sin();
        ArmAngles {
            theta1: normalise_angle(y.atan2(x) - k2.atan2(k1)),
            theta2: normalise_angle(theta2),
        }
    };

    Ok([solve(theta2_down), solve(theta2_up)])
}

// ---------------------------------------------------------------------------
// Medan potensial
// ---------------------------------------------------------------------------

/// Sebuah rintangan berbentuk lingkaran.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Obstacle {
    /// Pusat rintangan.
    pub x: f64,
    /// Pusat rintangan.
    pub y: f64,
    /// Jari-jari pengaruh.
    pub radius: f64,
}

/// Hasil perencanaan lintasan dengan medan potensial.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PotentialPath {
    /// Titik-titik lintasan.
    pub points: Vec<(f64, f64)>,
    /// Apakah tujuan tercapai.
    pub reached: bool,
    /// Apakah robot berhenti di minimum lokal.
    pub stuck_in_local_minimum: bool,
    /// Panjang lintasan.
    pub length: f64,
}

/// Merencanakan lintasan dengan medan potensial.
///
/// Tujuan menarik, rintangan menolak, dan robot mengikuti gaya total. Cara ini
/// sederhana dan cepat, tetapi punya cacat bawaan: bila gaya tarik dan gaya
/// tolak saling meniadakan, robot berhenti di tempat meski tujuannya terlihat
/// jelas. Titik itu disebut minimum lokal, dan fungsi ini melaporkannya
/// sebagai temuan, bukan menyembunyikannya sebagai kegagalan biasa.
pub fn plan_potential_field(
    start: (f64, f64),
    goal: (f64, f64),
    obstacles: &[Obstacle],
    attractive_gain: f64,
    repulsive_gain: f64,
    step_size: f64,
    max_steps: usize,
) -> Result<PotentialPath, RoboticsError> {
    for (name, value) in [
        ("attractive_gain", attractive_gain),
        ("repulsive_gain", repulsive_gain),
        ("step_size", step_size),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(RoboticsError::BadParameter {
                name: name.into(),
                value,
            });
        }
    }

    let mut position = start;
    let mut points = vec![start];
    let mut length = 0.0;

    for _ in 0..max_steps.max(1) {
        let to_goal = (goal.0 - position.0, goal.1 - position.1);
        let distance = (to_goal.0 * to_goal.0 + to_goal.1 * to_goal.1).sqrt();
        if distance < step_size {
            points.push(goal);
            length += distance;
            return Ok(PotentialPath {
                points,
                reached: true,
                stuck_in_local_minimum: false,
                length,
            });
        }

        // Gaya tarik sebanding jarak ke tujuan.
        let mut fx = attractive_gain * to_goal.0 / distance;
        let mut fy = attractive_gain * to_goal.1 / distance;

        // Gaya tolak hanya berlaku di dalam jari-jari pengaruh, dan menguat
        // tajam saat mendekat.
        for obstacle in obstacles {
            let dx = position.0 - obstacle.x;
            let dy = position.1 - obstacle.y;
            let d = (dx * dx + dy * dy).sqrt();
            if d >= obstacle.radius || d < 1e-9 {
                continue;
            }
            let strength = repulsive_gain * (1.0 / d - 1.0 / obstacle.radius) / (d * d);
            fx += strength * dx / d;
            fy += strength * dy / d;
        }

        let magnitude = (fx * fx + fy * fy).sqrt();
        if magnitude < 1e-9 {
            // Gaya total nol: robot terjebak di minimum lokal.
            return Ok(PotentialPath {
                points,
                reached: false,
                stuck_in_local_minimum: true,
                length,
            });
        }

        let next = (
            position.0 + step_size * fx / magnitude,
            position.1 + step_size * fy / magnitude,
        );
        // Berhenti bergerak berarti juga terjebak, walau gayanya tidak nol.
        let moved = ((next.0 - position.0).powi(2) + (next.1 - position.1).powi(2)).sqrt();
        if moved < step_size * 1e-6 {
            return Ok(PotentialPath {
                points,
                reached: false,
                stuck_in_local_minimum: true,
                length,
            });
        }

        length += moved;
        position = next;
        points.push(position);
    }

    Ok(PotentialPath {
        points,
        reached: false,
        stuck_in_local_minimum: false,
        length,
    })
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

    // ------------------------------------------------------------ sudut

    #[test]
    fn penormalan_sudut() {
        close(normalise_angle(0.0), 0.0);
        close(
            normalise_angle(core::f64::consts::PI),
            core::f64::consts::PI,
        );
        near(normalise_angle(core::f64::consts::TAU), 0.0, 1e-12);
        near(
            normalise_angle(3.0 * core::f64::consts::PI),
            core::f64::consts::PI,
            1e-12,
        );
        assert!(normalise_angle(-3.0 * core::f64::consts::PI).abs() <= core::f64::consts::PI);
    }

    #[test]
    fn penormalan_membuat_selisih_sudut_masuk_akal() {
        // Selisih antara 179 dan -179 derajat seharusnya 2 derajat, bukan 358.
        let a = 179.0f64.to_radians();
        let b = (-179.0f64).to_radians();
        let selisih = normalise_angle(a - b).abs().to_degrees();
        near(selisih, 2.0, 1e-9);
    }

    #[test]
    fn sudut_selalu_dalam_rentang() {
        let mut x = -20.0;
        while x < 20.0 {
            let a = normalise_angle(x);
            assert!(
                a > -core::f64::consts::PI - 1e-12 && a <= core::f64::consts::PI + 1e-12,
                "{x} menjadi {a}"
            );
            x += 0.37;
        }
    }

    // ------------------------------------------------- penggerak diferensial

    #[test]
    fn roda_sama_cepat_bergerak_lurus() {
        let awal = Pose::new(0.0, 0.0, 0.0);
        let hasil = differential_step(awal, 1.0, 1.0, 0.5, 1.0).unwrap();
        close(hasil.x, 1.0);
        close(hasil.y, 0.0);
        close(hasil.theta, 0.0);
    }

    #[test]
    fn roda_berlawanan_berputar_di_tempat() {
        let awal = Pose::new(0.0, 0.0, 0.0);
        let hasil = differential_step(awal, -1.0, 1.0, 1.0, 1.0).unwrap();
        near(hasil.x, 0.0, 1e-9);
        near(hasil.y, 0.0, 1e-9);
        assert!(hasil.theta.abs() > 0.5, "seharusnya berputar");
    }

    #[test]
    fn roda_berbeda_membentuk_busur() {
        let awal = Pose::new(0.0, 0.0, 0.0);
        let hasil = differential_step(awal, 1.0, 2.0, 1.0, 1.0).unwrap();
        assert!(hasil.theta > 0.0, "berbelok ke kiri");
        assert!(hasil.x > 0.0);
        assert!(hasil.y > 0.0);
    }

    #[test]
    fn kedudukan_selalu_bernilai_wajar() {
        let mut pose = Pose::new(0.0, 0.0, 0.0);
        for i in 0..200 {
            let kiri = (i as f64 * 0.1).sin();
            let kanan = (i as f64 * 0.13).cos();
            pose = differential_step(pose, kiri, kanan, 0.4, 0.05).unwrap();
            assert!(pose.x.is_finite() && pose.y.is_finite() && pose.theta.is_finite());
            assert!(pose.theta.abs() <= core::f64::consts::PI + 1e-9);
        }
    }

    #[test]
    fn penggerak_menolak_parameter_tak_sah() {
        let p = Pose::new(0.0, 0.0, 0.0);
        assert!(matches!(
            differential_step(p, 1.0, 1.0, 0.0, 1.0),
            Err(RoboticsError::BadParameter { .. })
        ));
        assert!(matches!(
            differential_step(p, 1.0, 1.0, 0.5, 0.0),
            Err(RoboticsError::BadParameter { .. })
        ));
        assert!(matches!(
            differential_step(p, 1.0, 1.0, f64::NAN, 1.0),
            Err(RoboticsError::BadParameter { .. })
        ));
    }

    #[test]
    fn jarak_ke_titik() {
        let p = Pose::new(0.0, 0.0, 0.0);
        close(p.distance_to(3.0, 4.0), 5.0);
        close(p.distance_to(0.0, 0.0), 0.0);
    }

    // -------------------------------------------------------------- PID

    #[test]
    fn pid_menolak_penguatan_tak_sah() {
        assert!(matches!(
            Pid::new(f64::NAN, 0.0, 0.0, 1.0),
            Err(RoboticsError::BadParameter { .. })
        ));
        assert!(matches!(
            Pid::new(1.0, 0.0, 0.0, 0.0),
            Err(RoboticsError::BadParameter { .. })
        ));
        assert!(matches!(
            Pid::new(1.0, 0.0, 0.0, -1.0),
            Err(RoboticsError::BadParameter { .. })
        ));
    }

    #[test]
    fn pid_proporsional_sebanding_galat() {
        let mut pid = Pid::new(2.0, 0.0, 0.0, 100.0).unwrap();
        close(pid.update(3.0, 0.1).unwrap(), 6.0);
        close(pid.update(-1.5, 0.1).unwrap(), -3.0);
    }

    #[test]
    fn pid_keluaran_dibatasi() {
        let mut pid = Pid::new(100.0, 0.0, 0.0, 5.0).unwrap();
        close(pid.update(10.0, 0.1).unwrap(), 5.0);
        close(pid.update(-10.0, 0.1).unwrap(), -5.0);
    }

    #[test]
    fn pid_turunan_tidak_melonjak_di_langkah_pertama() {
        // Memakai nol sebagai galat sebelumnya menghasilkan lonjakan turunan
        // yang besar dan menyesatkan pada langkah pertama.
        let mut pid = Pid::new(0.0, 0.0, 10.0, 1000.0).unwrap();
        close(pid.update(5.0, 0.1).unwrap(), 0.0);
        // Langkah kedua barulah turunannya bermakna.
        assert!(pid.update(5.0, 0.1).unwrap().abs() < 1e-9);
        assert!(pid.update(6.0, 0.1).unwrap() > 0.0);
    }

    #[test]
    fn pid_mencegah_penumpukan_integral() {
        // Saat keluaran mentok, integral tidak boleh terus menumpuk. Tanpa
        // penjagaan ini, kendali sangat lambat pulih setelah galatnya berbalik.
        let mut pid = Pid::new(1.0, 5.0, 0.0, 1.0).unwrap();
        for _ in 0..50 {
            pid.update(10.0, 0.1).unwrap();
        }
        let tertumpuk = pid.accumulated_integral();
        assert!(
            tertumpuk < 5.0,
            "integral menumpuk sampai {tertumpuk} walau keluaran mentok"
        );
    }

    #[test]
    fn pid_reset_mengembalikan_keadaan_awal() {
        let mut pid = Pid::new(1.0, 1.0, 1.0, 100.0).unwrap();
        pid.update(5.0, 0.1).unwrap();
        pid.update(5.0, 0.1).unwrap();
        pid.reset();
        close(pid.accumulated_integral(), 0.0);
        // Setelah reset, langkah pertama kembali tanpa lonjakan turunan.
        close(pid.update(5.0, 0.1).unwrap(), 5.0 * 1.0 + 1.0 * 0.5);
    }

    #[test]
    fn pid_menolak_dt_tak_sah() {
        let mut pid = Pid::new(1.0, 0.0, 0.0, 10.0).unwrap();
        assert!(matches!(
            pid.update(1.0, 0.0),
            Err(RoboticsError::BadParameter { .. })
        ));
        assert!(matches!(
            pid.update(1.0, -0.1),
            Err(RoboticsError::BadParameter { .. })
        ));
    }

    #[test]
    fn simulasi_pid_yang_disetel_baik_menetap() {
        let mut pid = Pid::new(1.2, 0.4, 0.2, 20.0).unwrap();
        let hasil = simulate_pid(&mut pid, 10.0, 0.0, 2.0, 0.05, 600).unwrap();
        assert!(hasil.settled, "galat akhir {}", hasil.final_error);
        assert!(hasil.final_error.abs() < 0.5);
        assert!(hasil.steps.len() == 600);
    }

    #[test]
    fn penguatan_terlalu_besar_membuat_sistem_berayun() {
        // Inti pelajaran sesi ini: penguatan yang salah tidak sekadar lambat,
        // ia membuat sistem berayun.
        //
        // Batas keluaran dibuat longgar di sini dengan sengaja. Batas yang
        // ketat justru menyembunyikan cacatnya: keluaran yang selalu mentok
        // berperilaku seperti sakelar hidup-mati, sehingga penguatan sebesar
        // apa pun menghasilkan grafik yang mirip.
        let mut baik = Pid::new(1.2, 0.4, 0.2, 1_000.0).unwrap();
        let mut buruk = Pid::new(60.0, 0.0, 0.0, 1_000.0).unwrap();
        let hasil_baik = simulate_pid(&mut baik, 10.0, 0.0, 2.0, 0.05, 400).unwrap();
        let hasil_buruk = simulate_pid(&mut buruk, 10.0, 0.0, 2.0, 0.05, 400).unwrap();
        assert!(
            hasil_buruk.overshoot_percent > hasil_baik.overshoot_percent + 5.0,
            "buruk {:.1}% vs baik {:.1}%",
            hasil_buruk.overshoot_percent,
            hasil_baik.overshoot_percent
        );
        assert!(
            hasil_baik.settling_time.is_some(),
            "penyetelan yang wajar seharusnya menetap"
        );
    }

    #[test]
    fn waktu_menetap_kosong_bila_tidak_pernah_menetap() {
        // Regresi: nilai tak hingga tidak punya lambang di JSON, sehingga
        // serde_json menuliskannya sebagai null lalu gagal membacanya kembali
        // sebagai bilangan. Nilai yang tidak ada dinyatakan sebagai tidak ada.
        let mut liar = Pid::new(200.0, 0.0, 0.0, 1_000.0).unwrap();
        let hasil = simulate_pid(&mut liar, 10.0, 0.0, 2.0, 0.05, 200).unwrap();
        if !hasil.settled {
            assert!(hasil.settling_time.is_none());
        }
        // Bagaimanapun hasilnya, ia harus bisa bolak-balik lewat JSON.
        let json = serde_json::to_string(&hasil).unwrap();
        let balik: ControlRun = serde_json::from_str(&json).unwrap();
        assert_eq!(balik.settled, hasil.settled);
        assert_eq!(balik.settling_time.is_some(), hasil.settling_time.is_some());
    }

    #[test]
    fn simulasi_pid_menolak_parameter_tak_sah() {
        let mut pid = Pid::new(1.0, 0.0, 0.0, 10.0).unwrap();
        assert!(matches!(
            simulate_pid(&mut pid, 1.0, 0.0, 0.0, 0.1, 10),
            Err(RoboticsError::BadParameter { .. })
        ));
        assert!(matches!(
            simulate_pid(&mut pid, 1.0, 0.0, 1.0, 0.1, 0),
            Err(RoboticsError::BadParameter { .. })
        ));
    }

    // ------------------------------------------------------ lengan dua sendi

    #[test]
    fn kinematika_maju_lurus_ke_depan() {
        let (x, y) = forward_kinematics(
            ArmAngles {
                theta1: 0.0,
                theta2: 0.0,
            },
            2.0,
            1.0,
        )
        .unwrap();
        close(x, 3.0);
        close(y, 0.0);
    }

    #[test]
    fn kinematika_maju_siku_tertekuk() {
        let (x, y) = forward_kinematics(
            ArmAngles {
                theta1: 0.0,
                theta2: core::f64::consts::FRAC_PI_2,
            },
            2.0,
            1.0,
        )
        .unwrap();
        near(x, 2.0, 1e-9);
        near(y, 1.0, 1e-9);
    }

    #[test]
    fn kinematika_balik_membalikkan_kinematika_maju() {
        // Uji yang paling berarti: apa pun sudutnya, mencari sudut dari posisi
        // lalu menghitung posisinya kembali harus menghasilkan titik semula.
        let l1 = 2.0;
        let l2 = 1.5;
        let mut diperiksa = 0;
        for t1 in [-2.0, -0.7, 0.0, 0.5, 1.3, 2.5] {
            for t2 in [-2.0, -0.6, 0.4, 1.1, 2.2] {
                let sudut = ArmAngles {
                    theta1: t1,
                    theta2: t2,
                };
                let (x, y) = forward_kinematics(sudut, l1, l2).unwrap();
                let solusi = inverse_kinematics(x, y, l1, l2).unwrap();
                let cocok = solusi.iter().any(|s| {
                    let (bx, by) = forward_kinematics(*s, l1, l2).unwrap();
                    (bx - x).abs() < 1e-6 && (by - y).abs() < 1e-6
                });
                assert!(cocok, "tidak ada solusi yang mengembalikan ({x}, {y})");
                diperiksa += 1;
            }
        }
        assert!(diperiksa >= 30);
    }

    #[test]
    fn kinematika_balik_memberi_dua_penyelesaian() {
        // Siku ke atas dan siku ke bawah, keduanya sah.
        let solusi = inverse_kinematics(2.0, 1.0, 2.0, 1.5).unwrap();
        assert_eq!(solusi.len(), 2);
        assert!(
            (solusi[0].theta2 - solusi[1].theta2).abs() > 1e-6,
            "kedua penyelesaian seharusnya berbeda"
        );
    }

    #[test]
    fn kinematika_balik_menolak_di_luar_jangkauan() {
        assert!(matches!(
            inverse_kinematics(100.0, 0.0, 2.0, 1.0),
            Err(RoboticsError::OutOfReach { .. })
        ));
        // Terlalu dekat juga di luar jangkauan bila lengannya tidak sama panjang.
        assert!(matches!(
            inverse_kinematics(0.0, 0.0, 3.0, 1.0),
            Err(RoboticsError::OutOfReach { .. })
        ));
    }

    #[test]
    fn kinematika_balik_menerima_titik_tepat_di_batas() {
        // Titik tepat di jangkauan terjauh tidak boleh ditolak hanya karena
        // pembulatan.
        let solusi = inverse_kinematics(3.0, 0.0, 2.0, 1.0).unwrap();
        let (x, y) = forward_kinematics(solusi[0], 2.0, 1.0).unwrap();
        near(x, 3.0, 1e-6);
        near(y, 0.0, 1e-6);
    }

    #[test]
    fn kinematika_menolak_panjang_tak_sah() {
        let sudut = ArmAngles {
            theta1: 0.0,
            theta2: 0.0,
        };
        assert!(matches!(
            forward_kinematics(sudut, 0.0, 1.0),
            Err(RoboticsError::BadParameter { .. })
        ));
        assert!(matches!(
            inverse_kinematics(1.0, 0.0, -1.0, 1.0),
            Err(RoboticsError::BadParameter { .. })
        ));
        assert!(matches!(
            inverse_kinematics(f64::NAN, 0.0, 2.0, 1.0),
            Err(RoboticsError::BadParameter { .. })
        ));
    }

    // ----------------------------------------------------- medan potensial

    #[test]
    fn medan_potensial_lintasan_lurus_tanpa_rintangan() {
        let hasil = plan_potential_field((0.0, 0.0), (10.0, 0.0), &[], 1.0, 1.0, 0.2, 200).unwrap();
        assert!(hasil.reached);
        assert!(!hasil.stuck_in_local_minimum);
        near(hasil.length, 10.0, 0.3);
    }

    #[test]
    fn medan_potensial_menghindari_rintangan() {
        let rintangan = [Obstacle {
            x: 5.0,
            y: 0.3,
            radius: 2.0,
        }];
        let hasil =
            plan_potential_field((0.0, 0.0), (10.0, 0.0), &rintangan, 1.0, 2.0, 0.2, 400).unwrap();
        assert!(hasil.reached, "seharusnya tetap sampai tujuan");
        // Lintasannya harus memutar, jadi lebih panjang daripada garis lurus.
        assert!(hasil.length > 10.0);
        // Tidak boleh ada titik yang menembus pusat rintangan.
        for (x, y) in &hasil.points {
            let d = ((x - 5.0).powi(2) + (y - 0.3).powi(2)).sqrt();
            assert!(d > 0.1, "menembus rintangan di ({x}, {y})");
        }
    }

    #[test]
    fn medan_potensial_terjebak_di_minimum_lokal() {
        // Cacat bawaan metode ini: rintangan tepat di antara robot dan tujuan
        // membuat gaya tarik dan gaya tolak saling meniadakan.
        let rintangan = [Obstacle {
            x: 5.0,
            y: 0.0,
            radius: 2.0,
        }];
        let hasil =
            plan_potential_field((0.0, 0.0), (10.0, 0.0), &rintangan, 1.0, 5.0, 0.2, 300).unwrap();
        assert!(
            hasil.stuck_in_local_minimum || !hasil.reached,
            "seharusnya gagal; ini cacat bawaan metodenya, bukan bug"
        );
    }

    #[test]
    fn medan_potensial_menolak_parameter_tak_sah() {
        assert!(matches!(
            plan_potential_field((0.0, 0.0), (1.0, 0.0), &[], 0.0, 1.0, 0.1, 10),
            Err(RoboticsError::BadParameter { .. })
        ));
        assert!(matches!(
            plan_potential_field((0.0, 0.0), (1.0, 0.0), &[], 1.0, 1.0, 0.0, 10),
            Err(RoboticsError::BadParameter { .. })
        ));
    }

    #[test]
    fn lintasan_selalu_bernilai_wajar() {
        let rintangan = [
            Obstacle {
                x: 3.0,
                y: 1.0,
                radius: 1.5,
            },
            Obstacle {
                x: 6.0,
                y: -1.0,
                radius: 1.5,
            },
        ];
        let hasil =
            plan_potential_field((0.0, 0.0), (10.0, 0.0), &rintangan, 1.0, 1.5, 0.15, 500).unwrap();
        assert!(hasil
            .points
            .iter()
            .all(|(x, y)| x.is_finite() && y.is_finite()));
        assert!(hasil.length.is_finite() && hasil.length >= 0.0);
    }

    #[test]
    fn hasil_bisa_di_serialisasi() {
        let mut pid = Pid::new(1.0, 0.1, 0.05, 10.0).unwrap();
        let run = simulate_pid(&mut pid, 5.0, 0.0, 1.0, 0.1, 20).unwrap();
        let json = serde_json::to_string(&run).unwrap();
        assert_eq!(
            serde_json::from_str::<ControlRun>(&json)
                .unwrap()
                .steps
                .len(),
            run.steps.len()
        );

        let path = plan_potential_field((0.0, 0.0), (3.0, 0.0), &[], 1.0, 1.0, 0.2, 50).unwrap();
        let pj = serde_json::to_string(&path).unwrap();
        let balik: PotentialPath = serde_json::from_str(&pj).unwrap();

        // Struktur lintasannya harus utuh.
        assert_eq!(balik.points.len(), path.points.len());
        assert_eq!(balik.reached, path.reached);
        assert_eq!(balik.stuck_in_local_minimum, path.stuck_in_local_minimum);

        // Koordinatnya dibandingkan dengan toleransi satu ULP, bukan `==`.
        // Penyebabnya cacat pihak ketiga yang sudah dipagari uji tersendiri di
        // `crate::fx`: `serde_json::from_str::<f64>` salah membulat 1 ULP pada
        // sebagian nilai, sehingga 1.6 bisa kembali sebagai 1.5999999999999999.
        for ((ax, ay), (bx, by)) in balik.points.iter().zip(&path.points) {
            assert!(
                crate::fx::ulp_distance(*ax, *bx).is_some_and(|d| d <= 1),
                "{ax} menyimpang lebih dari 1 ULP dari {bx}"
            );
            assert!(crate::fx::ulp_distance(*ay, *by).is_some_and(|d| d <= 1));
        }
        assert!(crate::fx::ulp_distance(balik.length, path.length).is_some_and(|d| d <= 1));
    }

    #[test]
    fn pesan_error_terbaca() {
        for e in [
            RoboticsError::BadParameter {
                name: "x".into(),
                value: 0.0,
            },
            RoboticsError::OutOfReach {
                distance: 5.0,
                max_reach: 3.0,
                min_reach: 1.0,
            },
            RoboticsError::DidNotConverge { steps: 10 },
        ] {
            assert!(!e.to_string().is_empty());
        }
    }
}
