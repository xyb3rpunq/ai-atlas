-- =============================================================================
-- AI ATLAS — spesifikasi paket PKG_AI_CORE
--
-- Implementasi ketiga dari algoritma yang sama, setelah Rust dan Go. Tujuannya
-- bukan dipakai di produksi melainkan membuktikan kedua implementasi lain
-- benar: rumus yang salah tetap konsisten dengan dirinya sendiri, sehingga uji
-- terhadap satu implementasi tidak bisa menangkapnya.
--
-- SELURUH PECAHAN MEMAKAI BINARY_DOUBLE
--
-- Bukan NUMBER. NUMBER adalah desimal presisi arbitrer yang tepat untuk uang
-- tetapi tidak pernah sepadan bit demi bit dengan f64 di Rust. BINARY_DOUBLE
-- adalah IEEE-754 binary64 yang sama persis. Memakai NUMBER akan membuat
-- hasilnya terlihat "hampir sama" — dan ketidakcocokan yang hampir sama
-- adalah yang paling sering diabaikan orang.
--
-- .Deckyx
-- =============================================================================

CREATE OR REPLACE PACKAGE pkg_ai_core AUTHID DEFINER AS

  -- Versi paket, dicocokkan dengan versi crate Rust.
  c_version CONSTANT VARCHAR2(10) := '1.0.0';

  -- Batas toleransi perbandingan pecahan.
  c_eps CONSTANT BINARY_DOUBLE := 1E-9d;

  -- ---------------------------------------------------------------------------
  -- Pertukaran pecahan bit-eksak
  -- ---------------------------------------------------------------------------

  -- Mengubah BINARY_DOUBLE menjadi 16 digit heksadesimal pola bit IEEE-754.
  --
  -- Urutan bitanya dipaksa big-endian supaya sepadan dengan keluaran
  -- `format!("{:016x}", f64::to_bits(v))` di Rust dan `math.Float64bits` di Go.
  -- Membiarkannya mengikuti urutan mesin akan membuat hasilnya benar di satu
  -- arsitektur dan terbalik di arsitektur lain.
  FUNCTION to_hex (p_value BINARY_DOUBLE) RETURN VARCHAR2 DETERMINISTIC;

  -- Membaca kembali BINARY_DOUBLE dari 16 digit heksadesimal.
  FUNCTION from_hex (p_hex VARCHAR2) RETURN BINARY_DOUBLE DETERMINISTIC;

  -- Jarak dua nilai dalam satuan ULP; NULL bila tidak terdefinisi.
  FUNCTION ulp_distance (p_a BINARY_DOUBLE, p_b BINARY_DOUBLE) RETURN NUMBER DETERMINISTIC;

  -- Apakah dua nilai identik pada tingkat pola bit, dengan NaN dianggap sama.
  FUNCTION bit_equal (p_a BINARY_DOUBLE, p_b BINARY_DOUBLE) RETURN BOOLEAN;

  -- Pemeriksa nilai istimewa, dibaca dari pola bitnya.
  --
  -- Perbandingan biasa tidak bisa dipakai untuk ini: Oracle menyatakan
  -- `NaN = NaN` bernilai benar, kebalikan dari IEEE-754, sehingga menguji
  -- `p != p` justru selalu salah.
  FUNCTION is_nan (p_value BINARY_DOUBLE) RETURN BOOLEAN;
  FUNCTION is_infinite (p_value BINARY_DOUBLE) RETURN BOOLEAN;

  -- Jarak antara `p_x` dan BINARY_DOUBLE terdekat berikutnya yang lebih besar
  -- nilai mutlaknya.
  --
  -- Dipakai untuk menyatakan toleransi pada skala tempat aritmetikanya
  -- terjadi, bukan pada hasil akhirnya. Satu ULP pada 1024 seribu kali lebih
  -- besar daripada satu ULP pada 1, jadi toleransi ULP tanpa skala hanya
  -- bermakna kalau hasilnya sendiri yang jadi skalanya -- dan itu tidak
  -- berlaku untuk besaran yang berupa selisih.
  FUNCTION ulp_step (p_x BINARY_DOUBLE) RETURN BINARY_DOUBLE DETERMINISTIC;

  -- ---------------------------------------------------------------------------
  -- SplitMix64
  -- ---------------------------------------------------------------------------

  -- Nilai ke-`p_index` dari deret SplitMix64 berbenih `p_seed`, sebagai hex.
  --
  -- Dikembalikan sebagai teks heksadesimal karena PL/SQL tidak punya tipe
  -- bilangan bulat 64 bit tak bertanda; NUMBER dipakai di dalamnya dengan
  -- aritmetika modulo yang eksplisit.
  FUNCTION splitmix64_hex (p_seed VARCHAR2, p_index PLS_INTEGER) RETURN VARCHAR2;

  -- Pecahan [0,1) ke-`p_index` dari deret yang sama.
  FUNCTION splitmix64_f64 (p_seed VARCHAR2, p_index PLS_INTEGER) RETURN BINARY_DOUBLE;

  -- ---------------------------------------------------------------------------
  -- Certainty factor
  -- ---------------------------------------------------------------------------

  FUNCTION cf_from_mb_md (p_mb BINARY_DOUBLE, p_md BINARY_DOUBLE)
    RETURN BINARY_DOUBLE DETERMINISTIC;

  FUNCTION cf_combine_parallel (p_a BINARY_DOUBLE, p_b BINARY_DOUBLE)
    RETURN BINARY_DOUBLE DETERMINISTIC;

  FUNCTION cf_combine_sequential (p_rule BINARY_DOUBLE, p_evidence BINARY_DOUBLE)
    RETURN BINARY_DOUBLE DETERMINISTIC;

  FUNCTION cf_and (p_a BINARY_DOUBLE, p_b BINARY_DOUBLE)
    RETURN BINARY_DOUBLE DETERMINISTIC;

  FUNCTION cf_or (p_a BINARY_DOUBLE, p_b BINARY_DOUBLE)
    RETURN BINARY_DOUBLE DETERMINISTIC;

  -- ---------------------------------------------------------------------------
  -- Bayesian
  -- ---------------------------------------------------------------------------

  FUNCTION bayes_evidence (
    p_prior            BINARY_DOUBLE,
    p_likelihood_h     BINARY_DOUBLE,
    p_likelihood_not_h BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC;

  FUNCTION bayes_posterior (
    p_prior            BINARY_DOUBLE,
    p_likelihood_h     BINARY_DOUBLE,
    p_likelihood_not_h BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC;

  FUNCTION bayes_likelihood_ratio (
    p_likelihood_h     BINARY_DOUBLE,
    p_likelihood_not_h BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC;

  -- ---------------------------------------------------------------------------
  -- Keanggotaan fuzzy
  -- ---------------------------------------------------------------------------

  FUNCTION fuzzy_triangular (
    p_a BINARY_DOUBLE, p_b BINARY_DOUBLE, p_c BINARY_DOUBLE, p_x BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC;

  FUNCTION fuzzy_trapezoidal (
    p_a BINARY_DOUBLE, p_b BINARY_DOUBLE, p_c BINARY_DOUBLE,
    p_d BINARY_DOUBLE, p_x BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC;

  FUNCTION fuzzy_gaussian (
    p_mean BINARY_DOUBLE, p_sigma BINARY_DOUBLE, p_x BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC;

  FUNCTION fuzzy_sigmoid (
    p_a BINARY_DOUBLE, p_c BINARY_DOUBLE, p_x BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC;

  -- Derajat keanggotaan sebuah himpunan yang tersimpan di tabel.
  FUNCTION fuzzy_degree (p_set_id NUMBER, p_x BINARY_DOUBLE) RETURN BINARY_DOUBLE;

  -- ---------------------------------------------------------------------------
  -- Jarak dan ketakmurnian
  -- ---------------------------------------------------------------------------

  FUNCTION distance_euclidean (
    p_ax BINARY_DOUBLE, p_ay BINARY_DOUBLE, p_bx BINARY_DOUBLE, p_by BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC;

  FUNCTION distance_manhattan (
    p_ax BINARY_DOUBLE, p_ay BINARY_DOUBLE, p_bx BINARY_DOUBLE, p_by BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC;

  FUNCTION distance_chebyshev (
    p_ax BINARY_DOUBLE, p_ay BINARY_DOUBLE, p_bx BINARY_DOUBLE, p_by BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC;

  -- Entropi Shannon sebuah daftar label berpisah koma, dalam bit.
  --
  -- Label dikelompokkan lalu dijumlah dalam urutan menaik, sama dengan
  -- `BTreeMap` pada implementasi Rust. Urutan penjumlahan yang berbeda
  -- menghasilkan bit terakhir yang berbeda, jadi urutan itu bagian dari
  -- spesifikasinya, bukan detail penyusunan.
  FUNCTION entropy_of_list (p_labels VARCHAR2) RETURN BINARY_DOUBLE;

  -- Ketakmurnian Gini sebuah daftar label berpisah koma.
  --
  -- Berbeda dengan entropi, Gini hanya memakai perkalian dan pengurangan,
  -- sehingga hasilnya wajib identik bit demi bit dengan Rust dan Go.
  FUNCTION gini_of_list (p_labels VARCHAR2) RETURN BINARY_DOUBLE;

  -- Perolehan informasi dari sepasang daftar nilai atribut dan label.
  FUNCTION information_gain_of_lists (p_values VARCHAR2, p_labels VARCHAR2)
    RETURN BINARY_DOUBLE;

  -- Entropi Shannon sebuah kumpulan data tersimpan, dalam bit.
  FUNCTION dataset_entropy (p_ds_code VARCHAR2) RETURN BINARY_DOUBLE;

  -- Perolehan informasi bila kumpulan data dipecah menurut sebuah atribut.
  FUNCTION dataset_information_gain (p_ds_code VARCHAR2, p_attr PLS_INTEGER)
    RETURN BINARY_DOUBLE;

  -- ---------------------------------------------------------------------------
  -- Inferensi sistem pakar
  -- ---------------------------------------------------------------------------

  -- Satu fakta beserta keyakinannya di memori kerja.
  TYPE t_fact IS RECORD (
    fact_code VARCHAR2(60),
    certainty BINARY_DOUBLE
  );
  TYPE t_facts IS TABLE OF t_fact INDEX BY VARCHAR2(60);

  -- Satu langkah penalaran, untuk fasilitas penjelasan.
  TYPE t_step IS RECORD (
    step_no    PLS_INTEGER,
    rule_code  VARCHAR2(20),
    rule_text  VARCHAR2(1000),
    conclusion VARCHAR2(60),
    certainty  BINARY_DOUBLE
  );
  TYPE t_steps IS TABLE OF t_step;

  -- Penalaran runut maju: menyapu basis aturan sampai keadaan tetap.
  PROCEDURE forward_chain (
    p_kb_code IN     VARCHAR2,
    p_facts   IN OUT NOCOPY t_facts,
    p_steps      OUT NOCOPY t_steps,
    p_passes     OUT PLS_INTEGER
  );

  -- Fakta yang dipakai sebagai premis tetapi tidak bisa disimpulkan maupun
  -- ditanyakan. Fakta seperti ini diam-diam dianggap tidak berlaku, sehingga
  -- sebagian aturan tidak akan pernah menyala tanpa pesan galat apa pun.
  FUNCTION unreachable_facts (p_kb_code VARCHAR2) RETURN SYS_REFCURSOR;

  -- Pohon pewarisan sebuah fakta, ditelusuri dengan CONNECT BY.
  FUNCTION rule_tree (p_kb_code VARCHAR2, p_goal VARCHAR2) RETURN SYS_REFCURSOR;

END pkg_ai_core;
/
