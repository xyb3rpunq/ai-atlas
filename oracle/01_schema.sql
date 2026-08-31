-- =============================================================================
-- AI ATLAS — skema basis pengetahuan Oracle
--
-- Basis pengetahuan sistem pakar disimpan sebagai tabel relasional, bukan
-- sebagai berkas. Alasannya bukan gaya-gayaan: aturan yang tersimpan di basis
-- data bisa diubah pakar tanpa menyentuh kode, diberi versi, diaudit siapa
-- yang mengubah apa, dan ditelusuri dengan SQL biasa.
--
-- CATATAN TIPE DATA YANG MENENTUKAN SEGALANYA
--
-- Seluruh nilai pecahan memakai BINARY_DOUBLE, bukan NUMBER. Keduanya terlihat
-- sama di layar tetapi sama sekali berbeda di dalam:
--
--   NUMBER         desimal presisi arbitrer, sampai 38 digit signifikan.
--                  Tepat untuk uang. Tidak pernah sepadan bit demi bit dengan
--                  float64 di bahasa lain.
--   BINARY_DOUBLE  IEEE-754 binary64, persis sama dengan f64 di Rust dan
--                  float64 di Go.
--
-- Memakai NUMBER akan membuat seluruh perbandingan lintas bahasa mustahil,
-- dan yang lebih buruk: hasilnya akan terlihat "hampir sama" sehingga
-- ketidakcocokannya diabaikan orang.
--
-- .Deckyx
-- =============================================================================

-- Dijalankan ulang dengan aman: objek lama dibuang lebih dulu.
BEGIN
  FOR t IN (
    SELECT table_name
    FROM   user_tables
    WHERE  table_name IN (
             'AI_CONFORMANCE_RESULT', 'AI_CONFORMANCE_VECTOR',
             'AI_RULE_PREMISE', 'AI_RULE', 'AI_FACT', 'AI_KNOWLEDGE_BASE',
             'AI_FUZZY_SET', 'AI_FUZZY_VARIABLE', 'AI_DATASET_ROW', 'AI_DATASET'
           )
  ) LOOP
    EXECUTE IMMEDIATE 'DROP TABLE ' || t.table_name || ' CASCADE CONSTRAINTS PURGE';
  END LOOP;
  FOR s IN (
    SELECT sequence_name
    FROM   user_sequences
    WHERE  sequence_name = 'AI_CONFORMANCE_RUN_SEQ'
  ) LOOP
    EXECUTE IMMEDIATE 'DROP SEQUENCE ' || s.sequence_name;
  END LOOP;
END;
/

-- -----------------------------------------------------------------------------
-- Basis pengetahuan sistem pakar
-- -----------------------------------------------------------------------------

CREATE TABLE ai_knowledge_base (
  kb_id        NUMBER(6)      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  kb_code      VARCHAR2(40)   NOT NULL,
  kb_name      VARCHAR2(120)  NOT NULL,
  description  VARCHAR2(500),
  created_at   TIMESTAMP      DEFAULT SYSTIMESTAMP NOT NULL,
  CONSTRAINT uq_kb_code UNIQUE (kb_code)
);

COMMENT ON TABLE ai_knowledge_base IS
  'Satu basis pengetahuan sistem pakar. Memisahkan pengetahuan dari mesin inferensinya adalah inti gagasan sistem pakar.';

-- Fakta yang dikenal sebuah basis pengetahuan.
CREATE TABLE ai_fact (
  fact_id      NUMBER(8)      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  kb_id        NUMBER(6)      NOT NULL,
  fact_code    VARCHAR2(60)   NOT NULL,
  fact_label   VARCHAR2(200)  NOT NULL,
  -- 'A' berarti fakta ini boleh ditanyakan kepada pengguna, 'D' berarti hanya
  -- bisa disimpulkan dari aturan. Fakta yang bukan keduanya adalah lubang di
  -- basis pengetahuan: ia diam-diam dianggap tidak berlaku.
  fact_kind    CHAR(1)        DEFAULT 'A' NOT NULL,
  CONSTRAINT fk_fact_kb FOREIGN KEY (kb_id) REFERENCES ai_knowledge_base (kb_id),
  CONSTRAINT uq_fact_code UNIQUE (kb_id, fact_code),
  CONSTRAINT ck_fact_kind CHECK (fact_kind IN ('A', 'D'))
);

COMMENT ON COLUMN ai_fact.fact_kind IS
  'A = dapat ditanyakan kepada pengguna, D = hanya dapat disimpulkan aturan.';

CREATE TABLE ai_rule (
  rule_id       NUMBER(8)       GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  kb_id         NUMBER(6)       NOT NULL,
  rule_code     VARCHAR2(20)    NOT NULL,
  -- Certainty factor aturan, dalam rentang -1 sampai 1.
  certainty     BINARY_DOUBLE   DEFAULT 1 NOT NULL,
  connective    VARCHAR2(3)     DEFAULT 'AND' NOT NULL,
  conclusion_id NUMBER(8)       NOT NULL,
  rationale     VARCHAR2(500),
  CONSTRAINT fk_rule_kb FOREIGN KEY (kb_id) REFERENCES ai_knowledge_base (kb_id),
  CONSTRAINT fk_rule_conclusion FOREIGN KEY (conclusion_id) REFERENCES ai_fact (fact_id),
  CONSTRAINT uq_rule_code UNIQUE (kb_id, rule_code),
  CONSTRAINT ck_rule_connective CHECK (connective IN ('AND', 'OR')),
  -- Batas rentang dijaga di sini, bukan hanya di kode. Data yang salah masuk
  -- lewat jalur mana pun akan tertahan.
  CONSTRAINT ck_rule_certainty CHECK (certainty BETWEEN -1 AND 1)
);

CREATE TABLE ai_rule_premise (
  premise_id  NUMBER(10)  GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  rule_id     NUMBER(8)   NOT NULL,
  fact_id     NUMBER(8)   NOT NULL,
  -- 'Y' berarti premis terpenuhi bila faktanya berlaku, 'N' bila justru tidak.
  expected    CHAR(1)     DEFAULT 'Y' NOT NULL,
  premise_seq NUMBER(3)   NOT NULL,
  CONSTRAINT fk_premise_rule FOREIGN KEY (rule_id) REFERENCES ai_rule (rule_id) ON DELETE CASCADE,
  CONSTRAINT fk_premise_fact FOREIGN KEY (fact_id) REFERENCES ai_fact (fact_id),
  CONSTRAINT uq_premise_seq UNIQUE (rule_id, premise_seq),
  CONSTRAINT ck_premise_expected CHECK (expected IN ('Y', 'N'))
);

CREATE INDEX ix_rule_conclusion ON ai_rule (conclusion_id);
CREATE INDEX ix_premise_fact ON ai_rule_premise (fact_id);

-- -----------------------------------------------------------------------------
-- Variabel dan himpunan kabur
-- -----------------------------------------------------------------------------

CREATE TABLE ai_fuzzy_variable (
  var_id     NUMBER(6)      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  var_code   VARCHAR2(40)   NOT NULL,
  var_name   VARCHAR2(120)  NOT NULL,
  min_value  BINARY_DOUBLE  NOT NULL,
  max_value  BINARY_DOUBLE  NOT NULL,
  CONSTRAINT uq_var_code UNIQUE (var_code),
  CONSTRAINT ck_var_range CHECK (min_value < max_value)
);

CREATE TABLE ai_fuzzy_set (
  set_id     NUMBER(8)      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  var_id     NUMBER(6)      NOT NULL,
  set_name   VARCHAR2(60)   NOT NULL,
  shape      VARCHAR2(20)   NOT NULL,
  -- Empat titik bentuknya. Segitiga memakai p1..p3, trapesium p1..p4,
  -- Gauss memakai p1 sebagai rerata dan p2 sebagai simpangan.
  p1         BINARY_DOUBLE  NOT NULL,
  p2         BINARY_DOUBLE  NOT NULL,
  p3         BINARY_DOUBLE,
  p4         BINARY_DOUBLE,
  CONSTRAINT fk_set_var FOREIGN KEY (var_id) REFERENCES ai_fuzzy_variable (var_id),
  CONSTRAINT uq_set_name UNIQUE (var_id, set_name),
  CONSTRAINT ck_set_shape CHECK (shape IN ('TRIANGULAR', 'TRAPEZOIDAL', 'GAUSSIAN', 'SIGMOID'))
);

-- -----------------------------------------------------------------------------
-- Kumpulan data untuk pohon keputusan
-- -----------------------------------------------------------------------------

CREATE TABLE ai_dataset (
  ds_id      NUMBER(6)      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  ds_code    VARCHAR2(40)   NOT NULL,
  ds_name    VARCHAR2(120)  NOT NULL,
  attr1_name VARCHAR2(40),
  attr2_name VARCHAR2(40),
  attr3_name VARCHAR2(40),
  attr4_name VARCHAR2(40),
  CONSTRAINT uq_ds_code UNIQUE (ds_code)
);

CREATE TABLE ai_dataset_row (
  row_id  NUMBER(10)   GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  ds_id   NUMBER(6)    NOT NULL,
  attr1   VARCHAR2(40),
  attr2   VARCHAR2(40),
  attr3   VARCHAR2(40),
  attr4   VARCHAR2(40),
  label   VARCHAR2(40) NOT NULL,
  CONSTRAINT fk_dsrow_ds FOREIGN KEY (ds_id) REFERENCES ai_dataset (ds_id) ON DELETE CASCADE
);

CREATE INDEX ix_dsrow_ds ON ai_dataset_row (ds_id);

-- -----------------------------------------------------------------------------
-- Vektor konformansi
-- -----------------------------------------------------------------------------

-- Satu baris = satu pernyataan yang bisa benar atau salah. Berkas TSV yang
-- dihasilkan Rust memuat beberapa keluaran per baris; pemuatnya memecah baris
-- itu menjadi satu baris per keluaran, supaya laporan ketidakcocokan menunjuk
-- ke satu perhitungan tertentu dan bukan ke sekumpulan perhitungan.
CREATE TABLE ai_conformance_vector (
  -- Nomornya diberikan pemuat, bukan dibangkitkan basis data. Dengan begitu
  -- satu pernyataan uji selalu bernomor sama di setiap jalan, sehingga hasil
  -- dua jalan bisa dibandingkan langsung. `INSERT ALL` juga tidak bisa dipakai
  -- dengan kolom IDENTITY: seluruh baris dalam satu pernyataan akan menerima
  -- nilai yang sama.
  vec_id         NUMBER(10)     PRIMARY KEY,
  source_file    VARCHAR2(60)   NOT NULL,
  line_no        NUMBER(8)      NOT NULL,
  comparability  VARCHAR2(30)   NOT NULL,
  operation      VARCHAR2(40)   NOT NULL,
  -- Masukan dan jawaban Rust sebagai pola bit heksadesimal. Disimpan sebagai
  -- teks, bukan BINARY_DOUBLE, supaya tidak ada satu pun konversi yang terjadi
  -- tanpa disengaja sebelum perbandingannya -- termasuk konversi yang
  -- menghapus tanda nol negatif.
  arg1_hex       VARCHAR2(16),
  arg2_hex       VARCHAR2(16),
  arg3_hex       VARCHAR2(16),
  arg4_hex       VARCHAR2(16),
  arg5_hex       VARCHAR2(16),
  -- Masukan yang bukan pecahan: daftar label, benih dan indeks pembangkit acak.
  arg_text1      VARCHAR2(400),
  arg_text2      VARCHAR2(400),
  -- Skala tempat aritmetikanya sesungguhnya terjadi, untuk tingkat
  -- CancellingDifference. Hasil yang berupa selisih dua besaran yang hampir
  -- sama memperbesar galat: dua ULP pada 0,94 sama dengan 64 ULP pada 0,029,
  -- padahal tidak ada perhitungan yang lebih buruk di antaranya.
  scale_hex      VARCHAR2(16),
  expected_hex   VARCHAR2(16)   NOT NULL,
  CONSTRAINT ck_vec_comparability
    CHECK (comparability IN ('BitExact', 'NearlyEqual(4)',
                             'CancellingDifference(4)', 'PropertyOnly')),
  -- Tingkat berskala tanpa skalanya adalah salah tulis, bukan alasan untuk
  -- diam-diam melonggarkan pemeriksaan.
  CONSTRAINT ck_vec_skala
    CHECK (comparability != 'CancellingDifference(4)' OR scale_hex IS NOT NULL),
  CONSTRAINT uq_vec_line UNIQUE (source_file, line_no, operation)
);

COMMENT ON COLUMN ai_conformance_vector.expected_hex IS
  'Jawaban implementasi Rust sebagai 16 digit heksadesimal pola bit IEEE-754.';

COMMENT ON COLUMN ai_conformance_vector.comparability IS
  'Tingkat keterbandingan yang ditetapkan Rust. Oracle memakai tingkat ini apa adanya, kecuali pada kasus tanda nol yang dicatat terpisah di kolom hasil.';

CREATE TABLE ai_conformance_result (
  res_id        NUMBER(10)     GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  run_id        NUMBER(10)     NOT NULL,
  vec_id        NUMBER(10)     NOT NULL,
  -- Kosong bila perhitungannya melempar galat; pesannya masuk ke error_text.
  -- Perhitungan yang gagal berbeda dari perhitungan yang menjawab salah, dan
  -- laporan yang menyamakan keduanya menyembunyikan penyebabnya.
  actual_hex    VARCHAR2(16),
  error_text    VARCHAR2(300),
  ulp_distance  NUMBER(20),
  -- 'Y' cocok, 'N' tidak cocok, 'Z' cocok kecuali tanda nol.
  --
  -- 'Z' bukan kelonggaran umum. Oracle BINARY_DOUBLE memang tidak punya nol
  -- negatif: nilai -0 diubah menjadi +0 bahkan pada konversi langsung dari
  -- pola bitnya. Status ini hanya diberikan bila kedua nilai benar-benar nol,
  -- sehingga selisih apa pun selain tanda nol tetap dihitung gagal.
  verdict       CHAR(1)        NOT NULL,
  checked_at    TIMESTAMP      DEFAULT SYSTIMESTAMP NOT NULL,
  CONSTRAINT fk_res_vec FOREIGN KEY (vec_id) REFERENCES ai_conformance_vector (vec_id) ON DELETE CASCADE,
  CONSTRAINT ck_res_verdict CHECK (verdict IN ('Y', 'N', 'Z')),
  CONSTRAINT ck_res_ada_hasil CHECK (actual_hex IS NOT NULL OR error_text IS NOT NULL)
);

CREATE INDEX ix_res_verdict ON ai_conformance_result (verdict);
CREATE INDEX ix_res_run ON ai_conformance_result (run_id);

CREATE SEQUENCE ai_conformance_run_seq START WITH 1 INCREMENT BY 1 NOCACHE;

PROMPT Skema AI ATLAS selesai dibuat.
