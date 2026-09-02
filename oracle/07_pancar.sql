-- Memancarkan pola bit yang dihitung PL/SQL, satu baris per pernyataan.
--
-- Dipakai halaman "Enam bahasa, satu angka". Kuncinya (berkas, baris, kolom)
-- sama persis dengan yang dipakai harness Go, Lua, Swift, dan Python, sehingga
-- keenam bahasa bisa disandingkan tanpa satu pun penyesuaian.
--
-- # Kenapa dari tabel hasil, bukan dihitung ulang di sini
--
-- Karena `ai_conformance_result.actual_hex` **adalah** jawaban PL/SQL yang
-- barusan diperiksa konformansi. Menghitungnya lagi lewat kueri terpisah
-- berarti ada dua jalur perhitungan, dan yang ditampilkan halaman belum tentu
-- yang diperiksa — tabel yang terlihat seperti bukti padahal tidak pernah
-- dibandingkan dengan apa pun.
--
-- Hanya jalan terakhir yang diambil: satu berkas memang harus menggambarkan
-- satu jalan, bukan gabungan beberapa.
--
-- # Kenapa barisan kepalanya ditulis run.sh
--
-- Karena membaca versi basis data menuntut hak akses ke `v$version`, dan
-- pengguna aplikasi tidak selalu punya. Sebuah SELECT yang gagal di sini akan
-- menghentikan seluruh jalan, karena `WHENEVER SQLERROR EXIT` sedang berlaku.
-- Keterangan versinya diketahui run.sh dari citra yang dijalankannya.
--
-- .Deckyx

SET HEADING OFF
SET FEEDBACK OFF
SET PAGESIZE 0
SET LINESIZE 32767
SET LONG 1000000
SET TRIMSPOOL ON
SET TERMOUT OFF
SET VERIFY OFF

SPOOL generated/plsql-baris.tsv

-- Nama kolom hasilnya disusun dari berkas dan operasinya, karena tabel vektor
-- Oracle menyimpan satu `expected_hex` per pernyataan sementara berkas TSV-nya
-- punya nama kolom sendiri. Pemetaan inilah yang menyambungkan keduanya.
SELECT v.source_file
       || CHR(9) || TO_CHAR(v.line_no)
       || CHR(9) || CASE
                      WHEN v.operation = 'splitmix64_u64'        THEN 'next_u64_hex'
                      WHEN v.operation = 'splitmix64_f64'        THEN 'next_f64_hex'
                      WHEN v.operation = 'bayes_evidence'        THEN 'evidence_hex'
                      WHEN v.operation = 'bayes_posterior'       THEN 'posterior_hex'
                      WHEN v.operation = 'bayes_likelihood_ratio' THEN 'likelihood_ratio_hex'
                      WHEN v.source_file LIKE 'fuzzy%'           THEN 'degree_hex'
                      WHEN v.source_file = 'fx.tsv'              THEN 'hex'
                      ELSE 'result_hex'
                    END
       || CHR(9) || LOWER(r.actual_hex)
       || CHR(9) || v.operation
FROM   ai_conformance_result r
       JOIN ai_conformance_vector v ON v.vec_id = r.vec_id
WHERE  r.run_id = (SELECT MAX(run_id) FROM ai_conformance_result)
       AND r.actual_hex IS NOT NULL
ORDER  BY v.source_file, v.line_no, v.vec_id;

SPOOL OFF
SET TERMOUT ON
SET HEADING ON
SET FEEDBACK ON
