-- =============================================================================
-- AI ATLAS — badan paket PKG_AI_CORE
--
-- Aturan yang dipegang seluruh berkas ini: urutan operasinya ditulis persis
-- sama dengan implementasi Rust. `a + b * (1 - a)` tidak boleh disederhanakan
-- menjadi `a + b - a * b` sekalipun keduanya setara secara aljabar; pada
-- aritmetika IEEE-754 keduanya menghasilkan bit yang berbeda, dan perbedaan
-- itulah yang sedang diukur.
--
-- .Deckyx
-- =============================================================================

CREATE OR REPLACE PACKAGE BODY pkg_ai_core AS

  -- Pangkat dua yang dipakai sebagai pembagi dan modulus pada aritmetika
  -- 64 bit. Ditulis sebagai tetapan agar tidak ada pemanggilan POWER yang
  -- diam-diam melewati pembulatan.
  c_2p32 CONSTANT NUMBER := 4294967296;
  c_2p52 CONSTANT NUMBER := 4503599627370496;
  c_2p63 CONSTANT NUMBER := 9223372036854775808;
  c_2p64 CONSTANT NUMBER := 18446744073709551616;

  -- Tetapan SplitMix64, sama dengan `crates/ai-core/src/rng.rs`.
  c_gamma  CONSTANT NUMBER := 11400714819323198485;  -- 0x9E3779B97F4A7C15
  c_mix1   CONSTANT NUMBER := 13787848793156543929;  -- 0xBF58476D1CE4E5B9
  c_mix2   CONSTANT NUMBER := 10723151780598845931;  -- 0x94D049BB133111EB

  -- 1 / 2^53, pengali `next_f64`.
  c_inv_2p53 CONSTANT BINARY_DOUBLE := 1d / 9007199254740992d;

  -- ---------------------------------------------------------------------------
  -- Pertukaran pecahan bit-eksak
  -- ---------------------------------------------------------------------------

  FUNCTION to_hex (p_value BINARY_DOUBLE) RETURN VARCHAR2 DETERMINISTIC IS
  BEGIN
    IF p_value IS NULL THEN
      RETURN NULL;
    END IF;
    RETURN LOWER(RAWTOHEX(UTL_RAW.CAST_FROM_BINARY_DOUBLE(p_value, UTL_RAW.BIG_ENDIAN)));
  END to_hex;

  FUNCTION from_hex (p_hex VARCHAR2) RETURN BINARY_DOUBLE DETERMINISTIC IS
    l_t VARCHAR2(32) := TRIM(p_hex);
  BEGIN
    IF l_t IS NULL THEN
      RETURN NULL;
    END IF;
    -- Panjang diperiksa sebelum HEXTORAW. Tanpa pemeriksaan ini, teks 15 digit
    -- ditolak dengan pesan galat yang tidak menyebut penyebabnya, dan teks
    -- 14 digit justru diterima sebagai angka yang sama sekali lain.
    IF LENGTH(l_t) != 16 OR NOT REGEXP_LIKE(l_t, '^[0-9A-Fa-f]{16}$') THEN
      RAISE_APPLICATION_ERROR(-20101, 'bukan 16 digit heksadesimal: ' || l_t);
    END IF;
    RETURN UTL_RAW.CAST_TO_BINARY_DOUBLE(HEXTORAW(UPPER(l_t)), UTL_RAW.BIG_ENDIAN);
  END from_hex;

  -- Pola bit sebuah BINARY_DOUBLE sebagai NUMBER dalam [0, 2^64).
  FUNCTION bits_of (p_value BINARY_DOUBLE) RETURN NUMBER IS
  BEGIN
    RETURN TO_NUMBER(
      RAWTOHEX(UTL_RAW.CAST_FROM_BINARY_DOUBLE(p_value, UTL_RAW.BIG_ENDIAN)),
      'XXXXXXXXXXXXXXXX'
    );
  END bits_of;

  -- Kunci terurut monoton dari pola bit, sama dengan penutup `key` pada
  -- `fx::ulp_distance` di Rust: pola bit dibaca sebagai bilangan bertanda,
  -- lalu yang negatif dicerminkan sehingga urutannya menyambung.
  FUNCTION order_key (p_bits NUMBER) RETURN NUMBER IS
    l_signed NUMBER;
  BEGIN
    l_signed := CASE WHEN p_bits >= c_2p63 THEN p_bits - c_2p64 ELSE p_bits END;
    RETURN CASE WHEN l_signed < 0 THEN (-c_2p63) - l_signed ELSE l_signed END;
  END order_key;

  FUNCTION is_nan (p_value BINARY_DOUBLE) RETURN BOOLEAN IS
    b NUMBER;
  BEGIN
    b := bits_of(p_value);
    RETURN MOD(TRUNC(b / c_2p52), 2048) = 2047 AND MOD(b, c_2p52) != 0;
  END is_nan;

  FUNCTION is_infinite (p_value BINARY_DOUBLE) RETURN BOOLEAN IS
    b NUMBER;
  BEGIN
    b := bits_of(p_value);
    RETURN MOD(TRUNC(b / c_2p52), 2048) = 2047 AND MOD(b, c_2p52) = 0;
  END is_infinite;

  FUNCTION ulp_distance (p_a BINARY_DOUBLE, p_b BINARY_DOUBLE) RETURN NUMBER DETERMINISTIC IS
  BEGIN
    IF p_a IS NULL OR p_b IS NULL THEN
      RETURN NULL;
    END IF;
    IF is_nan(p_a) OR is_nan(p_b) THEN
      RETURN NULL;
    END IF;
    -- Perbandingan biasa lebih dulu, supaya tak hingga yang sama bernilai nol
    -- dan bukan NULL.
    IF p_a = p_b THEN
      RETURN 0;
    END IF;
    IF is_infinite(p_a) OR is_infinite(p_b) THEN
      RETURN NULL;
    END IF;
    RETURN ABS(order_key(bits_of(p_a)) - order_key(bits_of(p_b)));
  END ulp_distance;

  FUNCTION bit_equal (p_a BINARY_DOUBLE, p_b BINARY_DOUBLE) RETURN BOOLEAN IS
  BEGIN
    IF p_a IS NULL OR p_b IS NULL THEN
      RETURN p_a IS NULL AND p_b IS NULL;
    END IF;
    IF is_nan(p_a) AND is_nan(p_b) THEN
      RETURN TRUE;
    END IF;
    RETURN bits_of(p_a) = bits_of(p_b);
  END bit_equal;

  FUNCTION ulp_step (p_x BINARY_DOUBLE) RETURN BINARY_DOUBLE DETERMINISTIC IS
    a BINARY_DOUBLE;
  BEGIN
    IF p_x IS NULL OR is_nan(p_x) OR is_infinite(p_x) THEN
      RETURN BINARY_DOUBLE_NAN;
    END IF;
    a := ABS(p_x);
    IF a = 0d THEN
      -- Nol tidak punya ULP yang bermakna; dipakai bilangan subnormal
      -- terkecil, yaitu langkah sesungguhnya dari nol ke bilangan berikutnya.
      RETURN from_hex('0000000000000001');
    END IF;
    RETURN from_hex(LOWER(LPAD(TRIM(TO_CHAR(bits_of(a) + 1, 'XXXXXXXXXXXXXXXX')), 16, '0'))) - a;
  END ulp_step;

  -- ---------------------------------------------------------------------------
  -- Aritmetika 64 bit tak bertanda
  --
  -- PL/SQL tidak punya bilangan bulat 64 bit tak bertanda. NUMBER dipakai
  -- sebagai gantinya: presisinya 38 digit desimal, sedangkan seluruh nilai
  -- antara di bawah ini paling besar 2^65 (20 digit), jadi tidak ada satu pun
  -- pembulatan yang terjadi.
  -- ---------------------------------------------------------------------------

  FUNCTION u64_hex (p_a NUMBER) RETURN VARCHAR2 IS
  BEGIN
    RETURN LOWER(LPAD(TRIM(TO_CHAR(p_a, 'XXXXXXXXXXXXXXXX')), 16, '0'));
  END u64_hex;

  FUNCTION u64_xor (p_a NUMBER, p_b NUMBER) RETURN NUMBER IS
  BEGIN
    RETURN TO_NUMBER(
      RAWTOHEX(UTL_RAW.BIT_XOR(HEXTORAW(u64_hex(p_a)), HEXTORAW(u64_hex(p_b)))),
      'XXXXXXXXXXXXXXXX'
    );
  END u64_xor;

  -- Perkalian modulo 2^64, dipecah menjadi paruh 32 bit supaya hasil antaranya
  -- tetap jauh di bawah batas presisi NUMBER.
  FUNCTION u64_mul (p_a NUMBER, p_b NUMBER) RETURN NUMBER IS
    ah    NUMBER := TRUNC(p_a / c_2p32);
    al    NUMBER := MOD(p_a, c_2p32);
    bh    NUMBER := TRUNC(p_b / c_2p32);
    bl    NUMBER := MOD(p_b, c_2p32);
    l_mid NUMBER;
  BEGIN
    -- Suku ah*bh seluruhnya jatuh di atas bit ke-64, jadi tidak ikut dihitung.
    l_mid := MOD(al * bh + ah * bl, c_2p32);
    RETURN MOD(al * bl + l_mid * c_2p32, c_2p64);
  END u64_mul;

  FUNCTION u64_shr (p_a NUMBER, p_bits PLS_INTEGER) RETURN NUMBER IS
  BEGIN
    RETURN TRUNC(p_a / POWER(2, p_bits));
  END u64_shr;

  FUNCTION splitmix64_raw (p_seed NUMBER, p_index PLS_INTEGER) RETURN NUMBER IS
    l_state NUMBER := MOD(p_seed, c_2p64);
    z       NUMBER;
  BEGIN
    FOR i IN 0 .. p_index LOOP
      l_state := MOD(l_state + c_gamma, c_2p64);
      z := l_state;
      z := u64_mul(u64_xor(z, u64_shr(z, 30)), c_mix1);
      z := u64_mul(u64_xor(z, u64_shr(z, 27)), c_mix2);
      z := u64_xor(z, u64_shr(z, 31));
    END LOOP;
    RETURN z;
  END splitmix64_raw;

  FUNCTION splitmix64_hex (p_seed VARCHAR2, p_index PLS_INTEGER) RETURN VARCHAR2 IS
  BEGIN
    RETURN u64_hex(splitmix64_raw(TO_NUMBER(p_seed), p_index));
  END splitmix64_hex;

  FUNCTION splitmix64_f64 (p_seed VARCHAR2, p_index PLS_INTEGER) RETURN BINARY_DOUBLE IS
    z NUMBER := splitmix64_raw(TO_NUMBER(p_seed), p_index);
  BEGIN
    -- `(next_u64() >> 11) as f64 * (1.0 / 2^53)` pada Rust. Nilai setelah
    -- pergeseran paling besar 2^53 - 1, sehingga konversinya ke BINARY_DOUBLE
    -- tepat dan pengalinya pun pangkat dua yang tepat.
    RETURN TO_BINARY_DOUBLE(u64_shr(z, 11)) * c_inv_2p53;
  END splitmix64_f64;

  -- ---------------------------------------------------------------------------
  -- Certainty factor
  -- ---------------------------------------------------------------------------

  FUNCTION check_belief (p_v BINARY_DOUBLE, p_name VARCHAR2) RETURN BINARY_DOUBLE IS
  BEGIN
    IF p_v IS NULL OR is_nan(p_v) OR is_infinite(p_v)
       OR p_v < -c_eps OR p_v > 1d + c_eps THEN
      RAISE_APPLICATION_ERROR(-20102, 'MB/MD harus di rentang [0,1], diberi ' || p_name);
    END IF;
    RETURN CASE WHEN p_v < 0d THEN 0d WHEN p_v > 1d THEN 1d ELSE p_v END;
  END check_belief;

  FUNCTION check_cf (p_v BINARY_DOUBLE, p_name VARCHAR2) RETURN BINARY_DOUBLE IS
  BEGIN
    IF p_v IS NULL OR is_nan(p_v) OR is_infinite(p_v)
       OR p_v < -1d - c_eps OR p_v > 1d + c_eps THEN
      RAISE_APPLICATION_ERROR(-20102, 'CF harus di rentang [-1,1], diberi ' || p_name);
    END IF;
    RETURN CASE WHEN p_v < -1d THEN -1d WHEN p_v > 1d THEN 1d ELSE p_v END;
  END check_cf;

  FUNCTION cf_from_mb_md (p_mb BINARY_DOUBLE, p_md BINARY_DOUBLE)
    RETURN BINARY_DOUBLE DETERMINISTIC IS
  BEGIN
    RETURN check_belief(p_mb, 'MB') - check_belief(p_md, 'MD');
  END cf_from_mb_md;

  FUNCTION cf_combine_parallel (p_a BINARY_DOUBLE, p_b BINARY_DOUBLE)
    RETURN BINARY_DOUBLE DETERMINISTIC IS
    a     BINARY_DOUBLE := check_cf(p_a, 'cf1');
    b     BINARY_DOUBLE := check_cf(p_b, 'cf2');
    denom BINARY_DOUBLE;
    out   BINARY_DOUBLE;
  BEGIN
    IF a >= 0d AND b >= 0d THEN
      out := a + b * (1d - a);
    ELSIF a <= 0d AND b <= 0d THEN
      out := a + b * (1d + a);
    ELSE
      denom := 1d - LEAST(ABS(a), ABS(b));
      IF ABS(denom) < c_eps THEN
        -- Bukti berlawanan penuh (+1 lawan -1) saling meniadakan.
        out := 0d;
      ELSE
        out := (a + b) / denom;
      END IF;
    END IF;
    RETURN CASE WHEN out < -1d THEN -1d WHEN out > 1d THEN 1d ELSE out END;
  END cf_combine_parallel;

  FUNCTION cf_combine_sequential (p_rule BINARY_DOUBLE, p_evidence BINARY_DOUBLE)
    RETURN BINARY_DOUBLE DETERMINISTIC IS
    r   BINARY_DOUBLE := check_cf(p_rule, 'cf_rule');
    e   BINARY_DOUBLE := check_cf(p_evidence, 'cf_evidence');
    out BINARY_DOUBLE;
  BEGIN
    out := r * GREATEST(e, 0d);
    RETURN CASE WHEN out < -1d THEN -1d WHEN out > 1d THEN 1d ELSE out END;
  END cf_combine_sequential;

  FUNCTION cf_and (p_a BINARY_DOUBLE, p_b BINARY_DOUBLE)
    RETURN BINARY_DOUBLE DETERMINISTIC IS
  BEGIN
    RETURN LEAST(check_cf(p_a, 'cf[0]'), check_cf(p_b, 'cf[1]'));
  END cf_and;

  FUNCTION cf_or (p_a BINARY_DOUBLE, p_b BINARY_DOUBLE)
    RETURN BINARY_DOUBLE DETERMINISTIC IS
  BEGIN
    RETURN GREATEST(check_cf(p_a, 'cf[0]'), check_cf(p_b, 'cf[1]'));
  END cf_or;

  -- ---------------------------------------------------------------------------
  -- Bayesian
  -- ---------------------------------------------------------------------------

  FUNCTION check_prob (p_v BINARY_DOUBLE, p_name VARCHAR2) RETURN BINARY_DOUBLE IS
  BEGIN
    IF p_v IS NULL OR is_nan(p_v) OR is_infinite(p_v)
       OR p_v < -c_eps OR p_v > 1d + c_eps THEN
      RAISE_APPLICATION_ERROR(-20102, 'peluang harus di rentang [0,1]: ' || p_name);
    END IF;
    RETURN CASE WHEN p_v < 0d THEN 0d WHEN p_v > 1d THEN 1d ELSE p_v END;
  END check_prob;

  FUNCTION bayes_evidence (
    p_prior            BINARY_DOUBLE,
    p_likelihood_h     BINARY_DOUBLE,
    p_likelihood_not_h BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC IS
    p_h   BINARY_DOUBLE := check_prob(p_prior, 'P(H)');
    p_e_h BINARY_DOUBLE := check_prob(p_likelihood_h, 'P(E|H)');
    p_e_n BINARY_DOUBLE := check_prob(p_likelihood_not_h, 'P(E|~H)');
    p_nh  BINARY_DOUBLE;
    ev    BINARY_DOUBLE;
  BEGIN
    p_nh := 1d - p_h;
    ev := p_h * p_e_h + p_nh * p_e_n;
    IF ev < c_eps THEN
      RAISE_APPLICATION_ERROR(-20103, 'P(E) nol: posterior tidak terdefinisi');
    END IF;
    RETURN ev;
  END bayes_evidence;

  FUNCTION bayes_posterior (
    p_prior            BINARY_DOUBLE,
    p_likelihood_h     BINARY_DOUBLE,
    p_likelihood_not_h BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC IS
    p_h   BINARY_DOUBLE := check_prob(p_prior, 'P(H)');
    p_e_h BINARY_DOUBLE := check_prob(p_likelihood_h, 'P(E|H)');
    ev    BINARY_DOUBLE := bayes_evidence(p_prior, p_likelihood_h, p_likelihood_not_h);
    post  BINARY_DOUBLE;
  BEGIN
    post := p_e_h * p_h / ev;
    RETURN CASE WHEN post < 0d THEN 0d WHEN post > 1d THEN 1d ELSE post END;
  END bayes_posterior;

  FUNCTION bayes_likelihood_ratio (
    p_likelihood_h     BINARY_DOUBLE,
    p_likelihood_not_h BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC IS
    a BINARY_DOUBLE := check_prob(p_likelihood_h, 'P(E|H)');
    b BINARY_DOUBLE := check_prob(p_likelihood_not_h, 'P(E|~H)');
  BEGIN
    IF b < c_eps THEN
      RETURN CASE WHEN a < c_eps THEN 0d ELSE BINARY_DOUBLE_INFINITY END;
    END IF;
    RETURN a / b;
  END bayes_likelihood_ratio;

  -- ---------------------------------------------------------------------------
  -- Keanggotaan fuzzy
  -- ---------------------------------------------------------------------------

  FUNCTION fuzzy_triangular (
    p_a BINARY_DOUBLE, p_b BINARY_DOUBLE, p_c BINARY_DOUBLE, p_x BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC IS
    v BINARY_DOUBLE;
  BEGIN
    -- Puncak diperiksa lebih dulu. Kalau tidak, segitiga berkaki berimpit
    -- (a = b, atau b = c) akan bernilai nol tepat di puncaknya -- bentuk yang
    -- justru lazim dipakai di tepi semesta.
    IF ABS(p_x - p_b) < c_eps THEN
      v := 1d;
    ELSIF p_x <= p_a OR p_x >= p_c THEN
      v := 0d;
    ELSIF p_x < p_b THEN
      v := CASE WHEN ABS(p_b - p_a) < c_eps THEN 1d ELSE (p_x - p_a) / (p_b - p_a) END;
    ELSIF ABS(p_c - p_b) < c_eps THEN
      v := 1d;
    ELSE
      v := (p_c - p_x) / (p_c - p_b);
    END IF;
    RETURN CASE WHEN v < 0d THEN 0d WHEN v > 1d THEN 1d ELSE v END;
  END fuzzy_triangular;

  FUNCTION fuzzy_trapezoidal (
    p_a BINARY_DOUBLE, p_b BINARY_DOUBLE, p_c BINARY_DOUBLE,
    p_d BINARY_DOUBLE, p_x BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC IS
    v BINARY_DOUBLE;
  BEGIN
    -- Bahu datar diperiksa lebih dulu, dengan alasan yang sama: trapesium
    -- bahu seperti (5, 8, 10, 10) harus bernilai satu di x = 10, bukan nol.
    IF p_x >= p_b AND p_x <= p_c THEN
      v := 1d;
    ELSIF p_x <= p_a OR p_x >= p_d THEN
      v := 0d;
    ELSIF p_x < p_b THEN
      v := CASE WHEN ABS(p_b - p_a) < c_eps THEN 1d ELSE (p_x - p_a) / (p_b - p_a) END;
    ELSIF ABS(p_d - p_c) < c_eps THEN
      v := 1d;
    ELSE
      v := (p_d - p_x) / (p_d - p_c);
    END IF;
    RETURN CASE WHEN v < 0d THEN 0d WHEN v > 1d THEN 1d ELSE v END;
  END fuzzy_trapezoidal;

  FUNCTION fuzzy_gaussian (
    p_mean BINARY_DOUBLE, p_sigma BINARY_DOUBLE, p_x BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC IS
    s BINARY_DOUBLE;
    z BINARY_DOUBLE;
    v BINARY_DOUBLE;
  BEGIN
    s := CASE WHEN ABS(p_sigma) < c_eps THEN c_eps ELSE ABS(p_sigma) END;
    z := (p_x - p_mean) / s;
    v := EXP(-0.5d * z * z);
    RETURN CASE WHEN v < 0d THEN 0d WHEN v > 1d THEN 1d ELSE v END;
  END fuzzy_gaussian;

  FUNCTION fuzzy_sigmoid (
    p_a BINARY_DOUBLE, p_c BINARY_DOUBLE, p_x BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC IS
    v BINARY_DOUBLE;
  BEGIN
    v := 1d / (1d + EXP(-p_a * (p_x - p_c)));
    RETURN CASE WHEN v < 0d THEN 0d WHEN v > 1d THEN 1d ELSE v END;
  END fuzzy_sigmoid;

  FUNCTION fuzzy_degree (p_set_id NUMBER, p_x BINARY_DOUBLE) RETURN BINARY_DOUBLE IS
    r ai_fuzzy_set%ROWTYPE;
  BEGIN
    SELECT * INTO r FROM ai_fuzzy_set WHERE set_id = p_set_id;
    RETURN CASE r.shape
             WHEN 'TRIANGULAR'  THEN fuzzy_triangular(r.p1, r.p2, r.p3, p_x)
             WHEN 'TRAPEZOIDAL' THEN fuzzy_trapezoidal(r.p1, r.p2, r.p3, r.p4, p_x)
             WHEN 'GAUSSIAN'    THEN fuzzy_gaussian(r.p1, r.p2, p_x)
             WHEN 'SIGMOID'     THEN fuzzy_sigmoid(r.p1, r.p2, p_x)
           END;
  END fuzzy_degree;

  -- ---------------------------------------------------------------------------
  -- Jarak dan ketakmurnian
  -- ---------------------------------------------------------------------------

  FUNCTION distance_euclidean (
    p_ax BINARY_DOUBLE, p_ay BINARY_DOUBLE, p_bx BINARY_DOUBLE, p_by BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC IS
    dx BINARY_DOUBLE := p_ax - p_bx;
    dy BINARY_DOUBLE := p_ay - p_by;
  BEGIN
    -- Penjumlahan dimulai dari nol dan berjalan kiri ke kanan, sama seperti
    -- `Iterator::sum` pada Rust. Urutannya berpengaruh pada bit terakhir.
    RETURN SQRT(0d + dx * dx + dy * dy);
  END distance_euclidean;

  FUNCTION distance_manhattan (
    p_ax BINARY_DOUBLE, p_ay BINARY_DOUBLE, p_bx BINARY_DOUBLE, p_by BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC IS
  BEGIN
    RETURN 0d + ABS(p_ax - p_bx) + ABS(p_ay - p_by);
  END distance_manhattan;

  FUNCTION distance_chebyshev (
    p_ax BINARY_DOUBLE, p_ay BINARY_DOUBLE, p_bx BINARY_DOUBLE, p_by BINARY_DOUBLE
  ) RETURN BINARY_DOUBLE DETERMINISTIC IS
  BEGIN
    RETURN GREATEST(GREATEST(0d, ABS(p_ax - p_bx)), ABS(p_ay - p_by));
  END distance_chebyshev;

  -- Entropi Shannon sebuah daftar label berpisah koma.
  FUNCTION entropy_of_list (p_labels VARCHAR2) RETURN BINARY_DOUBLE IS
    n   BINARY_DOUBLE;
    acc BINARY_DOUBLE := 0d;
    p   BINARY_DOUBLE;
    cnt NUMBER;
  BEGIN
    IF p_labels IS NULL OR LENGTH(TRIM(p_labels)) = 0 THEN
      RETURN 0d;
    END IF;
    cnt := REGEXP_COUNT(p_labels, ',') + 1;
    n := TO_BINARY_DOUBLE(cnt);
    -- Label diurutkan menaik supaya urutan penjumlahannya sama dengan
    -- `BTreeMap` pada Rust. Menjumlah dalam urutan lain menghasilkan bit
    -- terakhir yang berbeda.
    FOR r IN (
      SELECT lbl, COUNT(*) AS c
      FROM   (SELECT REGEXP_SUBSTR(p_labels, '[^,]+', 1, LEVEL) AS lbl
              FROM   DUAL
              CONNECT BY LEVEL <= cnt)
      GROUP  BY lbl
      ORDER  BY lbl
    ) LOOP
      p := TO_BINARY_DOUBLE(r.c) / n;
      acc := acc + p * LOG(2d, p);
    END LOOP;
    RETURN -acc;
  END entropy_of_list;

  FUNCTION gini_of_list (p_labels VARCHAR2) RETURN BINARY_DOUBLE IS
    n   BINARY_DOUBLE;
    acc BINARY_DOUBLE := 0d;
    p   BINARY_DOUBLE;
    cnt NUMBER;
  BEGIN
    IF p_labels IS NULL OR LENGTH(TRIM(p_labels)) = 0 THEN
      RETURN 0d;
    END IF;
    cnt := REGEXP_COUNT(p_labels, ',') + 1;
    n := TO_BINARY_DOUBLE(cnt);
    FOR r IN (
      SELECT lbl, COUNT(*) AS c
      FROM   (SELECT REGEXP_SUBSTR(p_labels, '[^,]+', 1, LEVEL) AS lbl
              FROM   DUAL
              CONNECT BY LEVEL <= cnt)
      GROUP  BY lbl
      ORDER  BY lbl
    ) LOOP
      p := TO_BINARY_DOUBLE(r.c) / n;
      acc := acc + p * p;
    END LOOP;
    RETURN 1d - acc;
  END gini_of_list;

  FUNCTION information_gain_of_lists (p_values VARCHAR2, p_labels VARCHAR2)
    RETURN BINARY_DOUBLE IS
    before BINARY_DOUBLE;
    after  BINARY_DOUBLE := 0d;
    n      BINARY_DOUBLE;
    cnt    NUMBER;
  BEGIN
    before := entropy_of_list(p_labels);
    cnt := REGEXP_COUNT(p_labels, ',') + 1;
    n := TO_BINARY_DOUBLE(cnt);
    FOR g IN (
      SELECT val, LISTAGG(lbl, ',') WITHIN GROUP (ORDER BY pos) AS grup, COUNT(*) AS c
      FROM (
        SELECT LEVEL AS pos,
               REGEXP_SUBSTR(p_values, '[^,]+', 1, LEVEL) AS val,
               REGEXP_SUBSTR(p_labels, '[^,]+', 1, LEVEL) AS lbl
        FROM   DUAL
        CONNECT BY LEVEL <= cnt
      )
      GROUP BY val
      ORDER BY val
    ) LOOP
      after := after + (TO_BINARY_DOUBLE(g.c) / n) * entropy_of_list(g.grup);
    END LOOP;
    RETURN before - after;
  END information_gain_of_lists;

  FUNCTION dataset_entropy (p_ds_code VARCHAR2) RETURN BINARY_DOUBLE IS
    l_labels VARCHAR2(4000);
  BEGIN
    SELECT LISTAGG(r.label, ',') WITHIN GROUP (ORDER BY r.row_id)
    INTO   l_labels
    FROM   ai_dataset_row r
    JOIN   ai_dataset d ON d.ds_id = r.ds_id
    WHERE  d.ds_code = p_ds_code;
    RETURN entropy_of_list(l_labels);
  END dataset_entropy;

  FUNCTION dataset_information_gain (p_ds_code VARCHAR2, p_attr PLS_INTEGER)
    RETURN BINARY_DOUBLE IS
    l_labels VARCHAR2(4000);
    l_values VARCHAR2(4000);
  BEGIN
    SELECT LISTAGG(r.label, ',') WITHIN GROUP (ORDER BY r.row_id),
           LISTAGG(CASE p_attr WHEN 1 THEN r.attr1 WHEN 2 THEN r.attr2
                               WHEN 3 THEN r.attr3 WHEN 4 THEN r.attr4 END, ',')
             WITHIN GROUP (ORDER BY r.row_id)
    INTO   l_labels, l_values
    FROM   ai_dataset_row r
    JOIN   ai_dataset d ON d.ds_id = r.ds_id
    WHERE  d.ds_code = p_ds_code;
    RETURN information_gain_of_lists(l_values, l_labels);
  END dataset_information_gain;

  -- ---------------------------------------------------------------------------
  -- Inferensi sistem pakar
  -- ---------------------------------------------------------------------------

  FUNCTION certainty_of (p_facts t_facts, p_code VARCHAR2) RETURN BINARY_DOUBLE IS
  BEGIN
    IF p_facts.EXISTS(p_code) THEN
      RETURN p_facts(p_code).certainty;
    END IF;
    -- Fakta yang tidak diketahui bernilai nol, bukan galat. Sistem pakar
    -- memang harus bisa menalar dengan pengetahuan yang belum lengkap.
    RETURN 0d;
  END certainty_of;

  PROCEDURE forward_chain (
    p_kb_code IN     VARCHAR2,
    p_facts   IN OUT NOCOPY t_facts,
    p_steps      OUT NOCOPY t_steps,
    p_passes     OUT PLS_INTEGER
  ) IS
    c_firing_threshold CONSTANT BINARY_DOUBLE := 0.2d;
    c_max_steps        CONSTANT PLS_INTEGER := 10000;
    l_changed  BOOLEAN;
    l_budget   PLS_INTEGER := c_max_steps;
    l_premise  BINARY_DOUBLE;
    l_concl    BINARY_DOUBLE;
    l_before   BINARY_DOUBLE;
    l_after    BINARY_DOUBLE;
    l_nilai    BINARY_DOUBLE;
    l_fired    t_facts;
    l_key      VARCHAR2(60);
    l_rec      t_fact;
    l_step     t_step;
  BEGIN
    p_steps := t_steps();
    p_passes := 0;
    LOOP
      p_passes := p_passes + 1;
      l_changed := FALSE;

      FOR r IN (
        SELECT ru.rule_id, ru.rule_code, ru.certainty, ru.connective,
               f.fact_code AS conclusion, ru.rationale
        FROM   ai_rule ru
        JOIN   ai_knowledge_base kb ON kb.kb_id = ru.kb_id
        JOIN   ai_fact f ON f.fact_id = ru.conclusion_id
        WHERE  kb.kb_code = p_kb_code
        ORDER  BY ru.rule_code
      ) LOOP
        l_budget := l_budget - 1;
        IF l_budget < 0 THEN
          RAISE_APPLICATION_ERROR(-20104,
            'penalaran melewati ' || c_max_steps || ' langkah; basis aturan mungkin berputar');
        END IF;

        -- Premis AND diambil minimumnya, OR maksimumnya. Premis ingkar
        -- terpenuhi justru saat faktanya lemah atau menyangkal, jadi nilainya
        -- dibalik tandanya.
        l_premise := CASE WHEN r.connective = 'AND'
                          THEN BINARY_DOUBLE_INFINITY
                          ELSE -BINARY_DOUBLE_INFINITY END;
        FOR pr IN (
          SELECT p.expected, f.fact_code
          FROM   ai_rule_premise p
          JOIN   ai_fact f ON f.fact_id = p.fact_id
          WHERE  p.rule_id = r.rule_id
          ORDER  BY p.premise_seq
        ) LOOP
          l_nilai := certainty_of(p_facts, pr.fact_code);
          IF pr.expected = 'N' THEN
            l_nilai := -l_nilai;
          END IF;
          l_premise := CASE WHEN r.connective = 'AND'
                            THEN LEAST(l_premise, l_nilai)
                            ELSE GREATEST(l_premise, l_nilai) END;
        END LOOP;

        CONTINUE WHEN l_premise < c_firing_threshold;

        -- Aturan yang sama dengan dukungan yang sama tidak dijalankan dua
        -- kali. Pola bit dipakai sebagai kunci supaya perbandingannya eksak,
        -- bukan bergantung pembulatan desimal.
        l_key := r.rule_code || '#' || to_hex(l_premise);
        CONTINUE WHEN l_fired.EXISTS(l_key);
        l_rec.fact_code := l_key;
        l_rec.certainty := l_premise;
        l_fired(l_key) := l_rec;

        l_concl := cf_combine_sequential(r.certainty, l_premise);
        l_before := certainty_of(p_facts, r.conclusion);
        IF p_facts.EXISTS(r.conclusion) THEN
          l_after := cf_combine_parallel(l_before, l_concl);
        ELSE
          l_after := l_concl;
        END IF;
        l_rec.fact_code := r.conclusion;
        l_rec.certainty := l_after;
        p_facts(r.conclusion) := l_rec;

        IF ABS(l_after - l_before) > 1E-12d THEN
          l_changed := TRUE;
        END IF;

        p_steps.EXTEND;
        l_step.step_no := p_steps.COUNT;
        l_step.rule_code := r.rule_code;
        l_step.rule_text := r.rationale;
        l_step.conclusion := r.conclusion;
        l_step.certainty := l_after;
        p_steps(p_steps.COUNT) := l_step;
      END LOOP;

      EXIT WHEN NOT l_changed;
    END LOOP;
  END forward_chain;

  FUNCTION unreachable_facts (p_kb_code VARCHAR2) RETURN SYS_REFCURSOR IS
    c SYS_REFCURSOR;
  BEGIN
    OPEN c FOR
      SELECT f.fact_code, f.fact_label
      FROM   ai_fact f
      JOIN   ai_knowledge_base kb ON kb.kb_id = f.kb_id
      WHERE  kb.kb_code = p_kb_code
      AND    f.fact_kind = 'D'
      AND    NOT EXISTS (SELECT 1 FROM ai_rule r WHERE r.conclusion_id = f.fact_id)
      AND    EXISTS (SELECT 1 FROM ai_rule_premise p WHERE p.fact_id = f.fact_id)
      ORDER  BY f.fact_code;
    RETURN c;
  END unreachable_facts;

  FUNCTION rule_tree (p_kb_code VARCHAR2, p_goal VARCHAR2) RETURN SYS_REFCURSOR IS
    c SYS_REFCURSOR;
  BEGIN
    -- Menelusuri mundur dari kesimpulan ke premisnya, lalu ke aturan yang
    -- menyimpulkan premis itu, dan seterusnya. CONNECT BY NOCYCLE memastikan
    -- basis aturan yang berputar tidak menggantung, melainkan berhenti dan
    -- menandai baris tempat putarannya terjadi.
    OPEN c FOR
      SELECT LEVEL AS kedalaman,
             rule_code,
             conclusion_code,
             premise_code,
             expected,
             CONNECT_BY_ISCYCLE AS berputar,
             LPAD(' ', (LEVEL - 1) * 2) || rule_code || ': ' ||
               premise_code || ' -> ' || conclusion_code AS jalur
      FROM (
        SELECT r.rule_code,
               fc.fact_code AS conclusion_code,
               fp.fact_code AS premise_code,
               p.expected
        FROM   ai_rule r
        JOIN   ai_knowledge_base kb ON kb.kb_id = r.kb_id
        JOIN   ai_fact fc ON fc.fact_id = r.conclusion_id
        JOIN   ai_rule_premise p ON p.rule_id = r.rule_id
        JOIN   ai_fact fp ON fp.fact_id = p.fact_id
        WHERE  kb.kb_code = p_kb_code
      )
      START WITH conclusion_code = p_goal
      CONNECT BY NOCYCLE PRIOR premise_code = conclusion_code
      ORDER SIBLINGS BY rule_code;
    RETURN c;
  END rule_tree;

END pkg_ai_core;
/
