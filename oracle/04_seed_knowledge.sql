-- =============================================================================
-- AI ATLAS — data benih
--
-- Basis pengetahuan, variabel kabur, dan kumpulan data untuk pohon keputusan.
-- Isinya sengaja sama dengan contoh yang dipakai laboratorium di situs, supaya
-- angka yang dilihat mahasiswa di peramban bisa ditelusuri sampai ke baris
-- tabel di sini.
--
-- SATU LUBANG DITINGGALKAN DENGAN SENGAJA
--
-- Fakta P05 dipakai sebagai premis aturan R07 tetapi tidak disimpulkan aturan
-- mana pun dan tidak boleh ditanyakan kepada pengguna. Basis pengetahuan
-- seperti ini tidak melanggar satu pun batasan basis data: ia tetap lolos
-- seluruh FOREIGN KEY dan CHECK, tetap bisa dijalankan, dan tetap memberi
-- jawaban. Yang terjadi hanyalah R07 tidak akan pernah menyala. Lubang seperti
-- inilah yang membuat sistem pakar diam-diam salah, dan itulah gunanya
-- `pkg_ai_core.unreachable_facts`.
--
-- .Deckyx
-- =============================================================================

SET DEFINE OFF

DELETE FROM ai_rule_premise;
DELETE FROM ai_rule;
DELETE FROM ai_fact;
DELETE FROM ai_knowledge_base;
DELETE FROM ai_fuzzy_set;
DELETE FROM ai_fuzzy_variable;
DELETE FROM ai_dataset_row;
DELETE FROM ai_dataset;

-- -----------------------------------------------------------------------------
-- Basis pengetahuan diagnosis demam
-- -----------------------------------------------------------------------------

INSERT INTO ai_knowledge_base (kb_code, kb_name, description) VALUES (
  'DIAG_DEMAM',
  'Diagnosis penyakit bergejala demam',
  'Contoh sistem pakar berbasis aturan dengan certainty factor, mengikuti bentuk MYCIN. Bukan alat medis.'
);

DECLARE
  l_kb NUMBER;

  PROCEDURE tambah_fakta (p_code VARCHAR2, p_label VARCHAR2, p_kind CHAR) IS
  BEGIN
    INSERT INTO ai_fact (kb_id, fact_code, fact_label, fact_kind)
    VALUES (l_kb, p_code, p_label, p_kind);
  END;

  PROCEDURE tambah_aturan (
    p_code       VARCHAR2,
    p_cf         BINARY_DOUBLE,
    p_conn       VARCHAR2,
    p_conclusion VARCHAR2,
    p_rationale  VARCHAR2,
    p_premis     VARCHAR2
  ) IS
    l_rule    NUMBER;
    l_concl   NUMBER;
    l_fact    NUMBER;
    l_kode    VARCHAR2(60);
    l_expect  CHAR(1);
    l_jumlah  PLS_INTEGER := REGEXP_COUNT(p_premis, ',') + 1;
  BEGIN
    SELECT fact_id INTO l_concl
    FROM   ai_fact WHERE kb_id = l_kb AND fact_code = p_conclusion;

    INSERT INTO ai_rule (kb_id, rule_code, certainty, connective, conclusion_id, rationale)
    VALUES (l_kb, p_code, p_cf, p_conn, l_concl, p_rationale)
    RETURNING rule_id INTO l_rule;

    FOR i IN 1 .. l_jumlah LOOP
      l_kode := TRIM(REGEXP_SUBSTR(p_premis, '[^,]+', 1, i));
      -- Awalan tanda seru menandai premis ingkar: terpenuhi justru saat
      -- faktanya lemah atau menyangkal.
      IF SUBSTR(l_kode, 1, 1) = '!' THEN
        l_expect := 'N';
        l_kode := SUBSTR(l_kode, 2);
      ELSE
        l_expect := 'Y';
      END IF;
      SELECT fact_id INTO l_fact
      FROM   ai_fact WHERE kb_id = l_kb AND fact_code = l_kode;
      INSERT INTO ai_rule_premise (rule_id, fact_id, expected, premise_seq)
      VALUES (l_rule, l_fact, l_expect, i);
    END LOOP;
  END;
BEGIN
  SELECT kb_id INTO l_kb FROM ai_knowledge_base WHERE kb_code = 'DIAG_DEMAM';

  tambah_fakta('G01', 'Demam di atas 38 derajat', 'A');
  tambah_fakta('G02', 'Nyeri sendi dan otot', 'A');
  tambah_fakta('G03', 'Bintik merah di kulit', 'A');
  tambah_fakta('G04', 'Mual atau muntah', 'A');
  tambah_fakta('G05', 'Batuk', 'A');
  tambah_fakta('G06', 'Pilek', 'A');
  tambah_fakta('G07', 'Sakit kepala', 'A');
  tambah_fakta('G08', 'Trombosit menurun', 'A');
  tambah_fakta('G09', 'Lidah berselaput putih', 'A');

  tambah_fakta('P01', 'Demam berdarah', 'D');
  tambah_fakta('P02', 'Influenza', 'D');
  tambah_fakta('P03', 'Demam tifoid', 'D');
  tambah_fakta('P04', 'Perlu rawat inap', 'D');
  -- Lubang yang disengaja; lihat catatan di kepala berkas.
  tambah_fakta('P05', 'Demam berdarah tahap lanjut', 'D');

  tambah_aturan('R01', 0.9d,  'AND', 'P01',
    'Demam disertai bintik merah dan trombosit turun sangat khas demam berdarah',
    'G01,G03,G08');
  tambah_aturan('R02', 0.7d,  'AND', 'P01',
    'Demam disertai nyeri sendi hebat mengarah ke demam berdarah',
    'G01,G02');
  tambah_aturan('R03', 0.85d, 'AND', 'P02',
    'Demam dengan batuk dan pilek mengarah ke influenza',
    'G01,G05,G06');
  tambah_aturan('R04', 0.5d,  'AND', 'P02',
    'Sakit kepala dengan batuk saja hanya dukungan lemah untuk influenza',
    'G07,G05');
  tambah_aturan('R05', 0.88d, 'AND', 'P03',
    'Demam dengan mual dan lidah berselaput khas demam tifoid',
    'G01,G04,G09');
  tambah_aturan('R06', 0.95d, 'AND', 'P04',
    'Demam berdarah dengan trombosit menurun perlu pemantauan menginap',
    'P01,G08');
  tambah_aturan('R07', 0.99d, 'AND', 'P04',
    'Tahap lanjut selalu perlu rawat inap -- aturan ini tidak akan pernah menyala',
    'P05,G01');
  -- Aturan berpremis ingkar: influenza justru menguat kalau trombositnya
  -- tidak menurun.
  tambah_aturan('R08', 0.4d,  'AND', 'P02',
    'Demam tanpa penurunan trombosit lebih mungkin influenza daripada demam berdarah',
    'G01,!G08');
END;
/

-- -----------------------------------------------------------------------------
-- Variabel kabur
-- -----------------------------------------------------------------------------

INSERT INTO ai_fuzzy_variable (var_code, var_name, min_value, max_value)
VALUES ('SUHU', 'Suhu ruangan dalam derajat Celsius', 0d, 50d);

INSERT INTO ai_fuzzy_variable (var_code, var_name, min_value, max_value)
VALUES ('PERMINTAAN', 'Permintaan barang per hari', 0d, 5000d);

DECLARE
  l_suhu  NUMBER;
  l_minta NUMBER;
BEGIN
  SELECT var_id INTO l_suhu  FROM ai_fuzzy_variable WHERE var_code = 'SUHU';
  SELECT var_id INTO l_minta FROM ai_fuzzy_variable WHERE var_code = 'PERMINTAAN';

  -- Himpunan tepi memakai trapesium dengan bahu berimpit di ujung semesta.
  -- Bentuk (0, 0, 15, 20) bernilai satu di x = 0, bukan nol; inilah kasus
  -- yang membuat urutan pemeriksaan di `fuzzy_trapezoidal` penting.
  INSERT INTO ai_fuzzy_set (var_id, set_name, shape, p1, p2, p3, p4)
  VALUES (l_suhu, 'DINGIN', 'TRAPEZOIDAL', 0d, 0d, 15d, 20d);
  INSERT INTO ai_fuzzy_set (var_id, set_name, shape, p1, p2, p3, p4)
  VALUES (l_suhu, 'SEJUK', 'TRIANGULAR', 15d, 22d, 28d, NULL);
  INSERT INTO ai_fuzzy_set (var_id, set_name, shape, p1, p2, p3, p4)
  VALUES (l_suhu, 'HANGAT', 'TRIANGULAR', 25d, 30d, 35d, NULL);
  INSERT INTO ai_fuzzy_set (var_id, set_name, shape, p1, p2, p3, p4)
  VALUES (l_suhu, 'PANAS', 'TRAPEZOIDAL', 30d, 38d, 50d, 50d);
  INSERT INTO ai_fuzzy_set (var_id, set_name, shape, p1, p2, p3, p4)
  VALUES (l_suhu, 'NYAMAN', 'GAUSSIAN', 24d, 3d, NULL, NULL);

  INSERT INTO ai_fuzzy_set (var_id, set_name, shape, p1, p2, p3, p4)
  VALUES (l_minta, 'TURUN', 'TRAPEZOIDAL', 0d, 0d, 1000d, 5000d);
  INSERT INTO ai_fuzzy_set (var_id, set_name, shape, p1, p2, p3, p4)
  VALUES (l_minta, 'NAIK', 'TRAPEZOIDAL', 1000d, 5000d, 5000d, 5000d);
END;
/

-- -----------------------------------------------------------------------------
-- Kumpulan data pohon keputusan
--
-- Dataset tenis klasik Quinlan, sama persis dengan yang dipakai vektor uji
-- `ml_gain.tsv`. Kesamaan itu disengaja: hasil `dataset_information_gain` di
-- sini bisa dibandingkan langsung dengan jawaban Rust.
-- -----------------------------------------------------------------------------

INSERT INTO ai_dataset (ds_code, ds_name, attr1_name, attr2_name, attr3_name, attr4_name)
VALUES ('TENIS', 'Bermain tenis atau tidak', 'cuaca', 'suhu', 'kelembapan', 'angin');

DECLARE
  l_ds NUMBER;
  PROCEDURE baris (a1 VARCHAR2, a2 VARCHAR2, a3 VARCHAR2, a4 VARCHAR2, l VARCHAR2) IS
  BEGIN
    INSERT INTO ai_dataset_row (ds_id, attr1, attr2, attr3, attr4, label)
    VALUES (l_ds, a1, a2, a3, a4, l);
  END;
BEGIN
  SELECT ds_id INTO l_ds FROM ai_dataset WHERE ds_code = 'TENIS';
  baris('Cerah',   'Panas',  'Tinggi', 'Lemah', 'Tidak');
  baris('Cerah',   'Panas',  'Tinggi', 'Kuat',  'Tidak');
  baris('Mendung', 'Panas',  'Tinggi', 'Lemah', 'Ya');
  baris('Hujan',   'Sejuk',  'Tinggi', 'Lemah', 'Ya');
  baris('Hujan',   'Dingin', 'Normal', 'Lemah', 'Ya');
  baris('Hujan',   'Dingin', 'Normal', 'Kuat',  'Tidak');
  baris('Mendung', 'Dingin', 'Normal', 'Kuat',  'Ya');
  baris('Cerah',   'Sejuk',  'Tinggi', 'Lemah', 'Tidak');
  baris('Cerah',   'Dingin', 'Normal', 'Lemah', 'Ya');
  baris('Hujan',   'Sejuk',  'Normal', 'Lemah', 'Ya');
  baris('Cerah',   'Sejuk',  'Normal', 'Kuat',  'Ya');
  baris('Mendung', 'Sejuk',  'Tinggi', 'Kuat',  'Ya');
  baris('Mendung', 'Panas',  'Normal', 'Lemah', 'Ya');
  baris('Hujan',   'Sejuk',  'Tinggi', 'Kuat',  'Tidak');
END;
/

COMMIT;

PROMPT Data benih AI ATLAS selesai dimuat.
