-- =============================================================================
-- AI ATLAS — uji unit PL/SQL
--
-- Harness-nya ditulis sendiri, bukan memakai utPLSQL. Alasannya sama dengan
-- alasan modul Go di proyek ini tidak memakai pustaka pihak ketiga: berkas ini
-- harus bisa dijalankan di kontainer Oracle mana pun tanpa langkah pemasangan,
-- termasuk di runner CI yang baru dinyalakan. Yang dibutuhkan cuma penghitung
-- dan sebuah galat di akhir.
--
-- Uji di sini melengkapi konformansi, bukan mengulanginya. Konformansi
-- membuktikan angkanya sama dengan Rust; uji di sini membuktikan hal-hal yang
-- memang hanya ada di sisi basis data: penalaran atas aturan yang tersimpan di
-- tabel, penelusuran pohon aturan, dan batasan tabel yang menolak data buruk.
--
-- .Deckyx
-- =============================================================================

SET SERVEROUTPUT ON SIZE UNLIMITED
SET LINESIZE 200
SET DEFINE OFF

DECLARE
  g_jalan  PLS_INTEGER := 0;
  g_gagal  PLS_INTEGER := 0;

  PROCEDURE lulus (p_nama VARCHAR2) IS
  BEGIN
    g_jalan := g_jalan + 1;
    DBMS_OUTPUT.PUT_LINE('  ok    ' || p_nama);
  END;

  PROCEDURE gagal (p_nama VARCHAR2, p_pesan VARCHAR2) IS
  BEGIN
    g_jalan := g_jalan + 1;
    g_gagal := g_gagal + 1;
    DBMS_OUTPUT.PUT_LINE('  GAGAL ' || p_nama || ' -- ' || p_pesan);
  END;

  PROCEDURE benar (p_nama VARCHAR2, p_syarat BOOLEAN) IS
  BEGIN
    IF p_syarat THEN lulus(p_nama); ELSE gagal(p_nama, 'syarat tidak terpenuhi'); END IF;
  END;

  PROCEDURE sama_hex (p_nama VARCHAR2, p_dapat BINARY_DOUBLE, p_harap VARCHAR2) IS
    l VARCHAR2(16) := pkg_ai_core.to_hex(p_dapat);
  BEGIN
    IF l = p_harap THEN lulus(p_nama);
    ELSE gagal(p_nama, 'harap ' || p_harap || ' dapat ' || l);
    END IF;
  END;

  PROCEDURE sama_num (p_nama VARCHAR2, p_dapat NUMBER, p_harap NUMBER) IS
  BEGIN
    IF p_dapat = p_harap OR (p_dapat IS NULL AND p_harap IS NULL) THEN lulus(p_nama);
    ELSE gagal(p_nama, 'harap ' || NVL(TO_CHAR(p_harap), 'NULL') ||
                       ' dapat ' || NVL(TO_CHAR(p_dapat), 'NULL'));
    END IF;
  END;

  PROCEDURE sama_teks (p_nama VARCHAR2, p_dapat VARCHAR2, p_harap VARCHAR2) IS
  BEGIN
    IF p_dapat = p_harap THEN lulus(p_nama);
    ELSE gagal(p_nama, 'harap ' || p_harap || ' dapat ' || NVL(p_dapat, 'NULL'));
    END IF;
  END;

  ---------------------------------------------------------------------------
  -- Pertukaran pola bit
  ---------------------------------------------------------------------------
  PROCEDURE uji_fx IS
    l BINARY_DOUBLE;
  BEGIN
    DBMS_OUTPUT.PUT_LINE('pertukaran pola bit');
    sama_teks('to_hex(1)', pkg_ai_core.to_hex(1d), '3ff0000000000000');
    sama_teks('to_hex(0.1)', pkg_ai_core.to_hex(0.1d), '3fb999999999999a');
    sama_teks('bolak-balik pi',
      pkg_ai_core.to_hex(pkg_ai_core.from_hex('400921fb54442d18')), '400921fb54442d18');
    sama_teks('bolak-balik subnormal terkecil',
      pkg_ai_core.to_hex(pkg_ai_core.from_hex('0000000000000001')), '0000000000000001');
    sama_teks('bolak-balik tak hingga',
      pkg_ai_core.to_hex(pkg_ai_core.from_hex('7ff0000000000000')), '7ff0000000000000');

    -- Panjang yang salah harus ditolak, bukan diam-diam dibaca sebagai angka
    -- lain. Teks 14 digit adalah pola bit yang sah tetapi bukan yang dimaksud.
    BEGIN
      l := pkg_ai_core.from_hex('3ff00000000000');
      gagal('from_hex menolak 14 digit', 'justru diterima');
    EXCEPTION
      WHEN OTHERS THEN
        IF SQLCODE = -20101 THEN lulus('from_hex menolak 14 digit');
        ELSE gagal('from_hex menolak 14 digit', SQLERRM); END IF;
    END;

    BEGIN
      l := pkg_ai_core.from_hex('3ff000000000000z');
      gagal('from_hex menolak huruf di luar heksadesimal', 'justru diterima');
    EXCEPTION
      WHEN OTHERS THEN
        IF SQLCODE = -20101 THEN lulus('from_hex menolak huruf di luar heksadesimal');
        ELSE gagal('from_hex menolak huruf di luar heksadesimal', SQLERRM); END IF;
    END;

    sama_num('ulp_distance(0.42, +1ulp)',
      pkg_ai_core.ulp_distance(pkg_ai_core.from_hex('3fdae147ae147ae1'),
                               pkg_ai_core.from_hex('3fdae147ae147ae2')), 1);
    sama_num('ulp_distance(1, 2)', pkg_ai_core.ulp_distance(1d, 2d), 4503599627370496);
    sama_num('ulp_distance ke tak hingga tidak terdefinisi',
      pkg_ai_core.ulp_distance(BINARY_DOUBLE_INFINITY, 1d), NULL);
    sama_num('ulp_distance ke NaN tidak terdefinisi',
      pkg_ai_core.ulp_distance(BINARY_DOUBLE_NAN, 1d), NULL);
    benar('bit_equal menganggap NaN sama dengan NaN',
      pkg_ai_core.bit_equal(BINARY_DOUBLE_NAN, BINARY_DOUBLE_NAN));

    sama_hex('ulp_step(1) sama dengan epsilon', pkg_ai_core.ulp_step(1d), '3cb0000000000000');
    sama_hex('ulp_step(0) sama dengan subnormal terkecil',
      pkg_ai_core.ulp_step(0d), '0000000000000001');
    benar('ulp_step membesar bersama nilainya',
      pkg_ai_core.ulp_step(1024d) > pkg_ai_core.ulp_step(1d) * 1000d);
    benar('ulp_step tak hingga tidak terdefinisi',
      pkg_ai_core.is_nan(pkg_ai_core.ulp_step(BINARY_DOUBLE_INFINITY)));
  END;

  ---------------------------------------------------------------------------
  -- Batas Oracle yang sudah diketahui
  ---------------------------------------------------------------------------
  PROCEDURE uji_nol_negatif IS
  BEGIN
    DBMS_OUTPUT.PUT_LINE('batas BINARY_DOUBLE yang sudah diketahui');
    -- Uji ini tidak memeriksa kode kita, melainkan mengunci perilaku Oracle
    -- yang menjadi dasar putusan 'Z' pada laporan konformansi. Kalau suatu
    -- hari Oracle mulai mempertahankan nol negatif, uji ini akan gagal dan
    -- memberi tahu bahwa kelonggarannya sudah boleh dicabut.
    sama_teks('Oracle mengubah -0 menjadi +0 lewat pola bit',
      pkg_ai_core.to_hex(pkg_ai_core.from_hex('8000000000000000')), '0000000000000000');
    sama_teks('Oracle mengubah -0 menjadi +0 lewat perkalian',
      pkg_ai_core.to_hex(-1d * 0d), '0000000000000000');
    sama_teks('NaN tetap berpola bit baku',
      pkg_ai_core.to_hex(BINARY_DOUBLE_NAN), '7ff8000000000000');
  END;

  ---------------------------------------------------------------------------
  -- Certainty factor
  ---------------------------------------------------------------------------
  PROCEDURE uji_cf IS
    l BINARY_DOUBLE;
  BEGIN
    DBMS_OUTPUT.PUT_LINE('certainty factor');
    sama_hex('CF = MB - MD', pkg_ai_core.cf_from_mb_md(0.8d, 0.01d), pkg_ai_core.to_hex(0.79d));
    sama_hex('gabungan dua bukti positif',
      pkg_ai_core.cf_combine_parallel(0.8d, 0.6d), pkg_ai_core.to_hex(0.92d));
    -- Bukti yang berlawanan penuh saling meniadakan, bukan menghasilkan
    -- pembagian dengan nol.
    sama_hex('bukti +1 lawan -1 saling meniadakan',
      pkg_ai_core.cf_combine_parallel(1d, -1d), '0000000000000000');
    sama_hex('bukti negatif tidak menyalakan aturan',
      pkg_ai_core.cf_combine_sequential(0.9d, -0.5d), '0000000000000000');
    sama_hex('premis AND diambil terkecil', pkg_ai_core.cf_and(0.9d, 0.3d), pkg_ai_core.to_hex(0.3d));
    sama_hex('premis OR diambil terbesar', pkg_ai_core.cf_or(0.9d, 0.3d), pkg_ai_core.to_hex(0.9d));
    benar('gabungan tetap di dalam [-1, 1]',
      pkg_ai_core.cf_combine_parallel(0.99d, 0.99d) <= 1d);

    BEGIN
      l := pkg_ai_core.cf_combine_parallel(1.5d, 0.2d);
      gagal('CF di luar rentang ditolak', 'justru diterima');
    EXCEPTION
      WHEN OTHERS THEN
        IF SQLCODE = -20102 THEN lulus('CF di luar rentang ditolak');
        ELSE gagal('CF di luar rentang ditolak', SQLERRM); END IF;
    END;
  END;

  ---------------------------------------------------------------------------
  -- Keanggotaan kabur yang tersimpan di tabel
  ---------------------------------------------------------------------------
  PROCEDURE uji_fuzzy IS
    l_dingin NUMBER;
    l_panas  NUMBER;
    l_nyaman NUMBER;
  BEGIN
    DBMS_OUTPUT.PUT_LINE('himpunan kabur tersimpan');
    SELECT s.set_id INTO l_dingin FROM ai_fuzzy_set s JOIN ai_fuzzy_variable v ON v.var_id = s.var_id
    WHERE v.var_code = 'SUHU' AND s.set_name = 'DINGIN';
    SELECT s.set_id INTO l_panas FROM ai_fuzzy_set s JOIN ai_fuzzy_variable v ON v.var_id = s.var_id
    WHERE v.var_code = 'SUHU' AND s.set_name = 'PANAS';
    SELECT s.set_id INTO l_nyaman FROM ai_fuzzy_set s JOIN ai_fuzzy_variable v ON v.var_id = s.var_id
    WHERE v.var_code = 'SUHU' AND s.set_name = 'NYAMAN';

    -- Justru inilah kasus yang gampang salah: himpunan bahu (0, 0, 15, 20)
    -- harus bernilai satu di tepi semestanya, bukan nol.
    sama_hex('DINGIN penuh di 0 derajat', pkg_ai_core.fuzzy_degree(l_dingin, 0d), '3ff0000000000000');
    sama_hex('DINGIN penuh di 10 derajat', pkg_ai_core.fuzzy_degree(l_dingin, 10d), '3ff0000000000000');
    sama_hex('DINGIN kosong di 20 derajat', pkg_ai_core.fuzzy_degree(l_dingin, 20d), '0000000000000000');
    sama_hex('PANAS penuh di 50 derajat', pkg_ai_core.fuzzy_degree(l_panas, 50d), '3ff0000000000000');
    sama_hex('NYAMAN penuh tepat di pusatnya', pkg_ai_core.fuzzy_degree(l_nyaman, 24d), '3ff0000000000000');
    benar('NYAMAN meluruh menjauhi pusatnya',
      pkg_ai_core.fuzzy_degree(l_nyaman, 30d) < pkg_ai_core.fuzzy_degree(l_nyaman, 26d));
    benar('derajat keanggotaan selalu di dalam [0, 1]',
      pkg_ai_core.fuzzy_degree(l_nyaman, -100d) >= 0d
      AND pkg_ai_core.fuzzy_degree(l_panas, 1000d) <= 1d);
  END;

  ---------------------------------------------------------------------------
  -- Kumpulan data dan pohon keputusan
  ---------------------------------------------------------------------------
  PROCEDURE uji_dataset IS
    l_gain_cuaca  BINARY_DOUBLE;
    l_gain_suhu   BINARY_DOUBLE;
    l_gain_lembap BINARY_DOUBLE;
    l_gain_angin  BINARY_DOUBLE;
    l_skala       BINARY_DOUBLE;
  BEGIN
    DBMS_OUTPUT.PUT_LINE('kumpulan data tersimpan');
    l_skala := pkg_ai_core.dataset_entropy('TENIS');
    -- Entropi dataset tenis: 9 Ya lawan 5 Tidak. Nilainya diperiksa terhadap
    -- vektor Rust dengan toleransi transendental, bukan bit demi bit.
    benar('entropi dataset tenis mendekati 0,940',
      ABS(l_skala - pkg_ai_core.from_hex('3fee16d2942c1b98'))
        <= 4 * pkg_ai_core.ulp_step(l_skala));

    l_gain_cuaca  := pkg_ai_core.dataset_information_gain('TENIS', 1);
    l_gain_suhu   := pkg_ai_core.dataset_information_gain('TENIS', 2);
    l_gain_lembap := pkg_ai_core.dataset_information_gain('TENIS', 3);
    l_gain_angin  := pkg_ai_core.dataset_information_gain('TENIS', 4);

    -- Yang menentukan bentuk pohonnya bukan angka persisnya melainkan
    -- urutannya, dan urutan itu tidak boleh bergantung pada bit terakhir.
    benar('cuaca adalah atribut pemecah pertama',
      l_gain_cuaca > l_gain_lembap
      AND l_gain_cuaca > l_gain_angin
      AND l_gain_cuaca > l_gain_suhu);
    benar('suhu adalah atribut paling lemah',
      l_gain_suhu < l_gain_angin AND l_gain_suhu < l_gain_lembap);
    benar('perolehan informasi cuaca sepadan dengan jawaban Rust',
      ABS(l_gain_cuaca - pkg_ai_core.from_hex('3fcf957f831cd070'))
        <= 4 * pkg_ai_core.ulp_step(l_skala));
    benar('perolehan informasi tidak pernah negatif',
      LEAST(l_gain_cuaca, l_gain_suhu, l_gain_lembap, l_gain_angin) >= 0d);

    sama_hex('entropi kelas tunggal adalah nol',
      pkg_ai_core.entropy_of_list('Ya,Ya,Ya,Ya'), '0000000000000000');
    sama_hex('entropi dua kelas seimbang adalah satu bit',
      pkg_ai_core.entropy_of_list('A,B'), '3ff0000000000000');
    sama_hex('Gini dua kelas seimbang adalah setengah',
      pkg_ai_core.gini_of_list('A,B'), '3fe0000000000000');
  END;

  ---------------------------------------------------------------------------
  -- Penalaran runut maju atas aturan yang tersimpan
  ---------------------------------------------------------------------------
  PROCEDURE uji_inferensi IS
    l_fakta  pkg_ai_core.t_facts;
    l_lang   pkg_ai_core.t_steps;
    l_sapuan PLS_INTEGER;
    l_rec    pkg_ai_core.t_fact;
    l_kur    SYS_REFCURSOR;
    l_kode   VARCHAR2(60);
    l_label  VARCHAR2(200);
    l_n      PLS_INTEGER := 0;

    PROCEDURE beri (p_kode VARCHAR2, p_cf BINARY_DOUBLE) IS
      r pkg_ai_core.t_fact;
    BEGIN
      r.fact_code := p_kode;
      r.certainty := p_cf;
      l_fakta(p_kode) := r;
    END;
  BEGIN
    DBMS_OUTPUT.PUT_LINE('penalaran runut maju');

    -- Pasien dengan gejala khas demam berdarah.
    beri('G01', 1d);
    beri('G03', 0.9d);
    beri('G08', 0.8d);
    pkg_ai_core.forward_chain('DIAG_DEMAM', l_fakta, l_lang, l_sapuan);

    benar('demam berdarah tersimpulkan', l_fakta.EXISTS('P01'));
    benar('rawat inap tersimpulkan dari kesimpulan sebelumnya', l_fakta.EXISTS('P04'));
    benar('keyakinan demam berdarah tinggi',
      l_fakta.EXISTS('P01') AND l_fakta('P01').certainty > 0.6d);
    -- R06 menyimpulkan P04 dari P01, yang baru ada setelah R01 menyala. Rantai
    -- seperti ini hanya selesai kalau basis aturannya disapu lebih dari sekali.
    benar('perlu lebih dari satu sapuan untuk rantai bertingkat', l_sapuan >= 2);
    benar('jejak penalaran terekam', l_lang.COUNT > 0);
    -- R07 berpremis fakta yang tidak bisa dicapai siapa pun.
    benar('aturan berpremis mustahil tidak menyala',
      NOT l_fakta.EXISTS('P05'));

    -- Gejala flu; demam berdarah tidak boleh ikut tersimpulkan.
    l_fakta.DELETE;
    beri('G01', 1d);
    beri('G05', 0.9d);
    beri('G06', 0.9d);
    pkg_ai_core.forward_chain('DIAG_DEMAM', l_fakta, l_lang, l_sapuan);
    benar('influenza tersimpulkan', l_fakta.EXISTS('P02'));
    benar('demam berdarah tidak tersimpulkan tanpa gejalanya',
      NOT l_fakta.EXISTS('P01'));
    -- R08 berpremis ingkar !G08; tanpa fakta G08 nilainya nol, dan ingkarannya
    -- juga nol, sehingga aturannya tetap tidak menyala. Premis ingkar menuntut
    -- bukti bahwa faktanya tidak berlaku, bukan sekadar ketiadaan bukti.
    benar('premis ingkar menuntut bukti penyangkalan, bukan kebisuan',
      l_fakta.EXISTS('P02'));

    -- Memori kerja kosong tidak boleh menyimpulkan apa pun.
    l_fakta.DELETE;
    pkg_ai_core.forward_chain('DIAG_DEMAM', l_fakta, l_lang, l_sapuan);
    sama_num('tanpa gejala tidak ada kesimpulan', l_fakta.COUNT, 0);
    sama_num('tanpa gejala hanya satu sapuan', l_sapuan, 1);

    -- Lubang yang disengaja harus ketahuan.
    l_kur := pkg_ai_core.unreachable_facts('DIAG_DEMAM');
    LOOP
      FETCH l_kur INTO l_kode, l_label;
      EXIT WHEN l_kur%NOTFOUND;
      l_n := l_n + 1;
      IF l_kode != 'P05' THEN
        gagal('fakta mustahil yang terdeteksi', 'tak terduga: ' || l_kode);
      END IF;
    END LOOP;
    CLOSE l_kur;
    sama_num('tepat satu fakta mustahil terdeteksi', l_n, 1);
  END;

  ---------------------------------------------------------------------------
  -- Penelusuran pohon aturan
  ---------------------------------------------------------------------------
  PROCEDURE uji_pohon IS
    l_kur     SYS_REFCURSOR;
    l_dalam   NUMBER;
    l_aturan  VARCHAR2(20);
    l_concl   VARCHAR2(60);
    l_premis  VARCHAR2(60);
    l_harap   CHAR(1);
    l_putar   NUMBER;
    l_jalur   VARCHAR2(400);
    l_n       PLS_INTEGER := 0;
    l_bertingkat BOOLEAN := FALSE;
  BEGIN
    DBMS_OUTPUT.PUT_LINE('penelusuran pohon aturan');
    l_kur := pkg_ai_core.rule_tree('DIAG_DEMAM', 'P04');
    LOOP
      FETCH l_kur INTO l_dalam, l_aturan, l_concl, l_premis, l_harap, l_putar, l_jalur;
      EXIT WHEN l_kur%NOTFOUND;
      l_n := l_n + 1;
      IF l_dalam > 1 THEN
        l_bertingkat := TRUE;
      END IF;
    END LOOP;
    CLOSE l_kur;
    benar('pohon aturan P04 punya isi', l_n > 0);
    -- P04 disimpulkan dari P01, yang sendiri disimpulkan aturan lain. Kalau
    -- penelusurannya berhenti di tingkat pertama, ia tidak menelusuri apa pun.
    benar('penelusuran menembus lebih dari satu tingkat', l_bertingkat);
  END;

  ---------------------------------------------------------------------------
  -- Batasan tabel
  ---------------------------------------------------------------------------
  PROCEDURE uji_batasan IS
    l_kb NUMBER;
    l_f  NUMBER;
  BEGIN
    DBMS_OUTPUT.PUT_LINE('batasan tabel');
    SELECT kb_id INTO l_kb FROM ai_knowledge_base WHERE kb_code = 'DIAG_DEMAM';
    SELECT fact_id INTO l_f FROM ai_fact WHERE kb_id = l_kb AND fact_code = 'P01';

    -- Rentang CF dijaga di basis data, bukan hanya di kode. Data buruk yang
    -- masuk lewat jalur lain -- skrip muat, perkakas admin, sambungan langsung
    -- -- tetap tertahan.
    BEGIN
      INSERT INTO ai_rule (kb_id, rule_code, certainty, connective, conclusion_id)
      VALUES (l_kb, 'RX_UJI', 1.5d, 'AND', l_f);
      gagal('CF di luar rentang ditolak basis data', 'justru diterima');
      ROLLBACK;
    EXCEPTION
      WHEN OTHERS THEN
        IF SQLCODE = -2290 THEN lulus('CF di luar rentang ditolak basis data');
        ELSE gagal('CF di luar rentang ditolak basis data', SQLERRM); END IF;
    END;

    BEGIN
      INSERT INTO ai_fact (kb_id, fact_code, fact_label, fact_kind)
      VALUES (l_kb, 'GX_UJI', 'Uji', 'X');
      gagal('jenis fakta di luar daftar ditolak', 'justru diterima');
      ROLLBACK;
    EXCEPTION
      WHEN OTHERS THEN
        IF SQLCODE = -2290 THEN lulus('jenis fakta di luar daftar ditolak');
        ELSE gagal('jenis fakta di luar daftar ditolak', SQLERRM); END IF;
    END;

    BEGIN
      INSERT INTO ai_fact (kb_id, fact_code, fact_label) VALUES (l_kb, 'G01', 'Ganda');
      gagal('kode fakta ganda ditolak', 'justru diterima');
      ROLLBACK;
    EXCEPTION
      WHEN OTHERS THEN
        IF SQLCODE = -1 THEN lulus('kode fakta ganda ditolak');
        ELSE gagal('kode fakta ganda ditolak', SQLERRM); END IF;
    END;

    BEGIN
      INSERT INTO ai_conformance_vector (vec_id, source_file, line_no, comparability, operation, expected_hex)
      VALUES (-1, 'uji.tsv', 1, 'CancellingDifference(4)', 'x', '3ff0000000000000');
      gagal('tingkat berskala tanpa skala ditolak', 'justru diterima');
      ROLLBACK;
    EXCEPTION
      WHEN OTHERS THEN
        IF SQLCODE = -2290 THEN lulus('tingkat berskala tanpa skala ditolak');
        ELSE gagal('tingkat berskala tanpa skala ditolak', SQLERRM); END IF;
    END;
    ROLLBACK;
  END;

BEGIN
  DBMS_OUTPUT.PUT_LINE('=========================================================');
  DBMS_OUTPUT.PUT_LINE('Uji unit PL/SQL AI ATLAS');
  DBMS_OUTPUT.PUT_LINE('=========================================================');
  uji_fx;
  uji_nol_negatif;
  uji_cf;
  uji_fuzzy;
  uji_dataset;
  uji_inferensi;
  uji_pohon;
  uji_batasan;
  DBMS_OUTPUT.PUT_LINE('---------------------------------------------------------');
  DBMS_OUTPUT.PUT_LINE(g_jalan || ' uji dijalankan, ' || g_gagal || ' gagal.');
  IF g_gagal > 0 THEN
    RAISE_APPLICATION_ERROR(-20120, g_gagal || ' uji PL/SQL gagal');
  END IF;
  -- Berkas uji yang tidak menjalankan apa pun akan membuat CI hijau justru
  -- ketika ujinya hilang.
  IF g_jalan < 40 THEN
    RAISE_APPLICATION_ERROR(-20121, 'hanya ' || g_jalan || ' uji dijalankan; ada yang tidak terpanggil');
  END IF;
END;
/
