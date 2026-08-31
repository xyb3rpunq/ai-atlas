-- =============================================================================
-- AI ATLAS — konformansi tiga arah
--
-- Menjalankan setiap pernyataan di `ai_conformance_vector` lewat PL/SQL, lalu
-- membandingkan pola bitnya dengan jawaban yang dihasilkan Rust. Karena
-- jawaban yang sama juga sudah diadu dengan implementasi Go yang ditulis
-- terpisah, lolosnya berkas ini berarti tiga implementasi mandiri sepakat.
--
-- KENAPA PERBANDINGANNYA MEMAKAI TEKS HEKSADESIMAL, BUKAN NILAINYA
--
-- Membandingkan `BINARY_DOUBLE` akan melewatkan dua hal sekaligus. Pertama,
-- Oracle menyatakan `-0 = +0` bernilai benar, sehingga perbedaan tanda nol
-- lolos tanpa terlihat. Kedua, Oracle menyatakan `NaN = NaN` juga benar,
-- padahal dua NaN berpola bit berbeda seharusnya diperiksa. Perbandingan
-- teks tidak punya kelonggaran itu: yang ingin dilonggarkan harus ditulis
-- sebagai aturan tersendiri, dan aturannya bisa dihitung.
--
-- .Deckyx
-- =============================================================================

CREATE OR REPLACE PACKAGE pkg_ai_conform AUTHID DEFINER AS

  -- Pola bit kedua macam nol.
  c_nol_positif CONSTANT VARCHAR2(16) := '0000000000000000';
  c_nol_negatif CONSTANT VARCHAR2(16) := '8000000000000000';

  -- Toleransi untuk perhitungan yang menyentuh fungsi transendental, sama
  -- dengan `fx::TRANSCENDENTAL_ULP` di Rust.
  c_ulp_transendental CONSTANT NUMBER := 4;

  -- Toleransi untuk hasil yang berupa selisih dua besaran yang hampir sama,
  -- diukur pada skalanya. Terukur pada dataset tenis: selisih PL/SQL lawan
  -- Rust paling besar 2 ULP pada skalanya.
  c_ulp_selisih CONSTANT NUMBER := 4;

  -- Menghitung ulang satu pernyataan dan mengembalikan pola bit jawabannya.
  FUNCTION hitung (p_vec_id NUMBER) RETURN VARCHAR2;

  -- Menjalankan seluruh pernyataan, menyimpan hasilnya, dan mengembalikan
  -- nomor jalannya.
  FUNCTION jalankan RETURN NUMBER;

  -- Mencetak ringkasan sebuah jalan konformansi ke DBMS_OUTPUT.
  PROCEDURE laporkan (p_run_id NUMBER);

  -- Menjalankan lalu melaporkan; keluar dengan galat bila ada yang gagal.
  PROCEDURE jalankan_dan_laporkan;

END pkg_ai_conform;
/

CREATE OR REPLACE PACKAGE BODY pkg_ai_conform AS

  FUNCTION d (p_hex VARCHAR2) RETURN BINARY_DOUBLE IS
  BEGIN
    RETURN pkg_ai_core.from_hex(p_hex);
  END d;

  FUNCTION hitung (p_vec_id NUMBER) RETURN VARCHAR2 IS
    v ai_conformance_vector%ROWTYPE;
  BEGIN
    SELECT * INTO v FROM ai_conformance_vector WHERE vec_id = p_vec_id;

    CASE v.operation
      WHEN 'splitmix64_u64' THEN
        RETURN pkg_ai_core.splitmix64_hex(v.arg_text1, TO_NUMBER(v.arg_text2));
      WHEN 'splitmix64_f64' THEN
        RETURN pkg_ai_core.to_hex(pkg_ai_core.splitmix64_f64(v.arg_text1, TO_NUMBER(v.arg_text2)));

      WHEN 'cf_parallel' THEN
        RETURN pkg_ai_core.to_hex(pkg_ai_core.cf_combine_parallel(d(v.arg1_hex), d(v.arg2_hex)));
      WHEN 'cf_sequential' THEN
        RETURN pkg_ai_core.to_hex(pkg_ai_core.cf_combine_sequential(d(v.arg1_hex), d(v.arg2_hex)));
      WHEN 'cf_and' THEN
        RETURN pkg_ai_core.to_hex(pkg_ai_core.cf_and(d(v.arg1_hex), d(v.arg2_hex)));
      WHEN 'cf_or' THEN
        RETURN pkg_ai_core.to_hex(pkg_ai_core.cf_or(d(v.arg1_hex), d(v.arg2_hex)));
      WHEN 'cf_mb_md' THEN
        RETURN pkg_ai_core.to_hex(pkg_ai_core.cf_from_mb_md(d(v.arg1_hex), d(v.arg2_hex)));

      WHEN 'bayes_evidence' THEN
        RETURN pkg_ai_core.to_hex(pkg_ai_core.bayes_evidence(d(v.arg1_hex), d(v.arg2_hex), d(v.arg3_hex)));
      WHEN 'bayes_posterior' THEN
        RETURN pkg_ai_core.to_hex(pkg_ai_core.bayes_posterior(d(v.arg1_hex), d(v.arg2_hex), d(v.arg3_hex)));
      WHEN 'bayes_likelihood_ratio' THEN
        RETURN pkg_ai_core.to_hex(pkg_ai_core.bayes_likelihood_ratio(d(v.arg2_hex), d(v.arg3_hex)));

      WHEN 'fuzzy_triangular' THEN
        RETURN pkg_ai_core.to_hex(
          pkg_ai_core.fuzzy_triangular(d(v.arg1_hex), d(v.arg2_hex), d(v.arg3_hex), d(v.arg5_hex)));
      WHEN 'fuzzy_trapezoidal' THEN
        RETURN pkg_ai_core.to_hex(
          pkg_ai_core.fuzzy_trapezoidal(d(v.arg1_hex), d(v.arg2_hex), d(v.arg3_hex), d(v.arg4_hex), d(v.arg5_hex)));
      WHEN 'fuzzy_gaussian' THEN
        RETURN pkg_ai_core.to_hex(
          pkg_ai_core.fuzzy_gaussian(d(v.arg1_hex), d(v.arg2_hex), d(v.arg3_hex)));
      WHEN 'fuzzy_sigmoid' THEN
        RETURN pkg_ai_core.to_hex(
          pkg_ai_core.fuzzy_sigmoid(d(v.arg1_hex), d(v.arg2_hex), d(v.arg3_hex)));

      WHEN 'distance_euclidean' THEN
        RETURN pkg_ai_core.to_hex(
          pkg_ai_core.distance_euclidean(d(v.arg1_hex), d(v.arg2_hex), d(v.arg3_hex), d(v.arg4_hex)));
      WHEN 'distance_manhattan' THEN
        RETURN pkg_ai_core.to_hex(
          pkg_ai_core.distance_manhattan(d(v.arg1_hex), d(v.arg2_hex), d(v.arg3_hex), d(v.arg4_hex)));
      WHEN 'distance_chebyshev' THEN
        RETURN pkg_ai_core.to_hex(
          pkg_ai_core.distance_chebyshev(d(v.arg1_hex), d(v.arg2_hex), d(v.arg3_hex), d(v.arg4_hex)));

      WHEN 'gini' THEN
        RETURN pkg_ai_core.to_hex(pkg_ai_core.gini_of_list(v.arg_text1));
      WHEN 'entropy' THEN
        RETURN pkg_ai_core.to_hex(pkg_ai_core.entropy_of_list(v.arg_text1));
      WHEN 'information_gain' THEN
        RETURN pkg_ai_core.to_hex(pkg_ai_core.information_gain_of_lists(v.arg_text2, v.arg_text1));

      WHEN 'roundtrip' THEN
        RETURN pkg_ai_core.to_hex(pkg_ai_core.from_hex(v.arg1_hex));

      ELSE
        -- Operasi baru di berkas vektor tanpa pemeriksa di sini akan
        -- menghentikan seluruh jalannya, bukan diam-diam dilewati.
        RAISE_APPLICATION_ERROR(-20110, 'operasi tanpa pemeriksa: ' || v.operation);
    END CASE;
  END hitung;

  FUNCTION jalankan RETURN NUMBER IS
    l_run    NUMBER;
    l_aktual VARCHAR2(16);
    l_galat  VARCHAR2(300);
    l_ulp    NUMBER;
    l_vonis  CHAR(1);
    l_maks   NUMBER;
    l_skala  BINARY_DOUBLE;
  BEGIN
    SELECT ai_conformance_run_seq.NEXTVAL INTO l_run FROM dual;

    FOR v IN (SELECT * FROM ai_conformance_vector ORDER BY vec_id) LOOP
      l_aktual := NULL;
      l_galat  := NULL;
      l_ulp    := NULL;
      BEGIN
        l_aktual := hitung(v.vec_id);
      EXCEPTION
        WHEN OTHERS THEN
          l_galat := SUBSTR(SQLERRM, 1, 300);
      END;

      IF l_galat IS NOT NULL THEN
        l_vonis := 'N';
      ELSIF l_aktual = v.expected_hex THEN
        l_vonis := 'Y';
        l_ulp := 0;
      ELSIF l_aktual IN (c_nol_positif, c_nol_negatif)
            AND v.expected_hex IN (c_nol_positif, c_nol_negatif) THEN
        -- Oracle BINARY_DOUBLE tidak punya nol negatif: -0 menjadi +0 bahkan
        -- pada konversi langsung dari pola bitnya. Kelonggaran ini hanya
        -- berlaku ketika kedua nilainya benar-benar nol, sehingga selisih apa
        -- pun selain tanda nol tetap dihitung gagal.
        l_vonis := 'Z';
        l_ulp := 0;
      ELSIF v.comparability = 'CancellingDifference(4)' THEN
        -- Toleransinya diukur pada skala tempat aritmetikanya terjadi, bukan
        -- pada hasilnya. Menuntut jarak ULP kecil pada hasil yang merupakan
        -- selisih dua besaran yang hampir sama berarti menuntut `log2` yang
        -- lebih teliti daripada yang diwajibkan IEEE-754 -- yaitu menuntut
        -- sesuatu yang tidak dijanjikan bahasa mana pun.
        l_ulp := pkg_ai_core.ulp_distance(d(l_aktual), d(v.expected_hex));
        l_skala := pkg_ai_core.ulp_step(d(v.scale_hex));
        l_vonis := CASE
                     WHEN l_skala IS NOT NULL
                          AND ABS(d(l_aktual) - d(v.expected_hex)) <= c_ulp_selisih * l_skala
                     THEN 'Y'
                     ELSE 'N'
                   END;
      ELSE
        l_ulp := pkg_ai_core.ulp_distance(d(l_aktual), d(v.expected_hex));
        l_maks := CASE WHEN v.comparability = 'NearlyEqual(4)' THEN c_ulp_transendental ELSE 0 END;
        l_vonis := CASE
                     WHEN v.comparability = 'PropertyOnly' THEN 'Y'
                     WHEN l_ulp IS NOT NULL AND l_ulp <= l_maks THEN 'Y'
                     ELSE 'N'
                   END;
      END IF;

      INSERT INTO ai_conformance_result (run_id, vec_id, actual_hex, error_text, ulp_distance, verdict)
      VALUES (l_run, v.vec_id, l_aktual, l_galat, l_ulp, l_vonis);
    END LOOP;

    COMMIT;
    RETURN l_run;
  END jalankan;

  PROCEDURE laporkan (p_run_id NUMBER) IS
    l_total NUMBER;
    l_cocok NUMBER;
    l_nol   NUMBER;
    l_gagal NUMBER;
    l_ulp   NUMBER;
    l_ulp_skala BINARY_DOUBLE;
  BEGIN
    SELECT COUNT(*),
           COUNT(CASE WHEN verdict = 'Y' THEN 1 END),
           COUNT(CASE WHEN verdict = 'Z' THEN 1 END),
           COUNT(CASE WHEN verdict = 'N' THEN 1 END)
    INTO   l_total, l_cocok, l_nol, l_gagal
    FROM   ai_conformance_result
    WHERE  run_id = p_run_id;

    DBMS_OUTPUT.PUT_LINE('=========================================================');
    DBMS_OUTPUT.PUT_LINE('Konformansi PL/SQL lawan Rust — jalan ke-' || p_run_id);
    DBMS_OUTPUT.PUT_LINE('=========================================================');

    FOR r IN (
      SELECT v.source_file, v.comparability,
             COUNT(*) AS total,
             COUNT(CASE WHEN s.verdict = 'Y' THEN 1 END) AS cocok,
             COUNT(CASE WHEN s.verdict = 'Z' THEN 1 END) AS nol,
             COUNT(CASE WHEN s.verdict = 'N' THEN 1 END) AS gagal,
             MAX(s.ulp_distance) AS ulp_maks
      FROM   ai_conformance_result s
      JOIN   ai_conformance_vector v ON v.vec_id = s.vec_id
      WHERE  s.run_id = p_run_id
      GROUP  BY v.source_file, v.comparability
      ORDER  BY v.source_file
    ) LOOP
      DBMS_OUTPUT.PUT_LINE(
        RPAD(r.source_file, 26) || RPAD(r.comparability, 16) ||
        LPAD(r.total, 6) || ' pernyataan, ' ||
        LPAD(r.cocok, 5) || ' cocok, ' ||
        LPAD(r.nol, 3) || ' beda tanda nol, ' ||
        LPAD(r.gagal, 4) || ' gagal, ULP maks ' || NVL(TO_CHAR(r.ulp_maks), '-'));
    END LOOP;

    SELECT MAX(ulp_distance) INTO l_ulp
    FROM   ai_conformance_result s
    JOIN   ai_conformance_vector v ON v.vec_id = s.vec_id
    WHERE  s.run_id = p_run_id AND v.comparability = 'NearlyEqual(4)';

    -- Pada tingkat berskala, jarak ULP diukur pada hasilnya dan karena itu
    -- besar; yang bermakna adalah jarak yang sama diukur pada skalanya.
    SELECT MAX(ABS(pkg_ai_core.from_hex(s.actual_hex) - pkg_ai_core.from_hex(v.expected_hex))
               / pkg_ai_core.ulp_step(pkg_ai_core.from_hex(v.scale_hex)))
    INTO   l_ulp_skala
    FROM   ai_conformance_result s
    JOIN   ai_conformance_vector v ON v.vec_id = s.vec_id
    WHERE  s.run_id = p_run_id
    AND    v.comparability = 'CancellingDifference(4)'
    AND    s.actual_hex IS NOT NULL;

    DBMS_OUTPUT.PUT_LINE('---------------------------------------------------------');
    DBMS_OUTPUT.PUT_LINE('Total          : ' || l_total || ' pernyataan');
    DBMS_OUTPUT.PUT_LINE('Cocok bit-eksak: ' || l_cocok);
    DBMS_OUTPUT.PUT_LINE('Beda tanda nol : ' || l_nol ||
      '   (Oracle BINARY_DOUBLE memang tidak punya -0)');
    DBMS_OUTPUT.PUT_LINE('Gagal          : ' || l_gagal);
    DBMS_OUTPUT.PUT_LINE('ULP terjauh pada tingkat NearlyEqual(4)          : ' || NVL(TO_CHAR(l_ulp), '-'));
    DBMS_OUTPUT.PUT_LINE('ULP terjauh pada tingkat CancellingDifference(4): ' ||
      NVL(TO_CHAR(ROUND(l_ulp_skala, 3)), '-') || '   (diukur pada skalanya, batas ' || c_ulp_selisih || ')');

    IF l_gagal > 0 THEN
      DBMS_OUTPUT.PUT_LINE('---------------------------------------------------------');
      DBMS_OUTPUT.PUT_LINE('Ketidakcocokan (paling banyak 25 baris pertama):');
      FOR r IN (
        SELECT v.source_file, v.line_no, v.operation, v.expected_hex,
               s.actual_hex, s.ulp_distance, s.error_text
        FROM   ai_conformance_result s
        JOIN   ai_conformance_vector v ON v.vec_id = s.vec_id
        WHERE  s.run_id = p_run_id AND s.verdict = 'N'
        ORDER  BY v.source_file, v.line_no, v.operation
        FETCH  FIRST 25 ROWS ONLY
      ) LOOP
        DBMS_OUTPUT.PUT_LINE(
          '  ' || RPAD(r.source_file || ':' || r.line_no, 26) ||
          RPAD(r.operation, 24) ||
          'harap ' || r.expected_hex ||
          '  dapat ' || NVL(r.actual_hex, '(galat)') ||
          CASE WHEN r.ulp_distance IS NOT NULL THEN '  ULP ' || r.ulp_distance ELSE '' END ||
          CASE WHEN r.error_text IS NOT NULL THEN '  ' || r.error_text ELSE '' END);
      END LOOP;
    END IF;
  END laporkan;

  PROCEDURE jalankan_dan_laporkan IS
    l_run   NUMBER;
    l_gagal NUMBER;
    l_total NUMBER;
  BEGIN
    l_run := jalankan;
    laporkan(l_run);

    SELECT COUNT(*), COUNT(CASE WHEN verdict = 'N' THEN 1 END)
    INTO   l_total, l_gagal
    FROM   ai_conformance_result
    WHERE  run_id = l_run;

    -- Jalan tanpa satu pun pernyataan bukan keberhasilan, melainkan tanda
    -- pemuat vektornya tidak berjalan. Tanpa pemeriksaan ini, CI akan hijau
    -- justru ketika ujinya tidak ada.
    IF l_total = 0 THEN
      RAISE_APPLICATION_ERROR(-20111, 'tidak ada pernyataan konformansi yang dimuat');
    END IF;
    IF l_gagal > 0 THEN
      RAISE_APPLICATION_ERROR(-20112, l_gagal || ' pernyataan konformansi gagal');
    END IF;
  END jalankan_dan_laporkan;

END pkg_ai_conform;
/
