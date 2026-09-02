#!/usr/bin/env bash
# Menjalankan seluruh berkas Oracle AI ATLAS pada sebuah kontainer.
#
# Dipakai sama persis di mesin pengembang dan di CI, sehingga kegagalan di CI
# selalu bisa diulang di tempat. Kontainernya dibuat kalau belum ada.
#
# Pemakaian:
#   oracle/run.sh              -- pasang, muat vektor, konformansi, uji
#   oracle/run.sh --keep       -- sama, tetapi kontainernya tidak dihentikan
#
# Di lingkungan CI, kontainernya sudah disediakan sebagai service container dan
# skrip ini hanya menyambung ke sana lewat sqlplus setempat. Ditandai dengan
# menyetel AI_ATLAS_ORACLE_MODE=langsung.
#
# .Deckyx
set -euo pipefail

NAMA_KONTAINER="${AI_ATLAS_ORACLE_CONTAINER:-ai-atlas-oracle}"
CITRA="${AI_ATLAS_ORACLE_IMAGE:-gvenzl/oracle-free:23-slim}"
SANDI="${AI_ATLAS_ORACLE_PASSWORD:-AiAtlasDev1}"
PENGGUNA="${AI_ATLAS_ORACLE_USER:-aiatlas}"
LAYANAN="${AI_ATLAS_ORACLE_SERVICE:-localhost:1521/FREEPDB1}"
MODE="${AI_ATLAS_ORACLE_MODE:-kontainer}"
AKAR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Seluruh perintah dijalankan dari akar repositori dengan jalur nisbi. Pada
# Windows, jalur POSIX seperti /c/Users/... yang diserahkan ke program bukan
# bawaan MSYS -- node, docker -- ditafsirkan sebagai jalur Windows dan gagal.
cd "$AKAR"

BERKAS=(01_schema.sql 02_pkg_ai_core.pks 03_pkg_ai_core.pkb
        04_seed_knowledge.sql 05_conformance.sql 06_tests.sql 07_pancar.sql
        deploy.sql)

echo "==> Menghasilkan pemuat vektor dari keluaran Rust"
node oracle/tools/make-load-sql.mjs

# Skrip penggerak. Urutannya menentukan: konformansi memerlukan vektor yang
# sudah dimuat, dan uji unit memerlukan data benih.
cat > "oracle/generated/jalan.sql" <<'SQL'
WHENEVER SQLERROR EXIT SQL.SQLCODE
SET DEFINE OFF
SET LINESIZE 220
SET PAGESIZE 200
SET SERVEROUTPUT ON SIZE UNLIMITED
SET FEEDBACK OFF
@deploy.sql
SQL

cat > "oracle/generated/jalan2.sql" <<'SQL'
WHENEVER SQLERROR EXIT SQL.SQLCODE
SET DEFINE OFF
SET LINESIZE 220
SET PAGESIZE 200
SET SERVEROUTPUT ON SIZE UNLIMITED
SET FEEDBACK OFF
@generated/load_vectors.sql
EXEC pkg_ai_conform.jalankan_dan_laporkan;
@06_tests.sql
@07_pancar.sql
EXIT
SQL

jalankan_sqlplus_langsung() {
  # Subkulit: tanpa ini `cd oracle` tetap berlaku sesudah fungsinya selesai,
  # dan langkah berikutnya — yang memakai jalur nisbi dari akar repositori —
  # akan mencari berkasnya satu tingkat terlalu dalam.
  (
    cd oracle
    sqlplus -s "$PENGGUNA/$SANDI@$LAYANAN" @generated/jalan.sql
    sqlplus -s "$PENGGUNA/$SANDI@$LAYANAN" @generated/jalan2.sql
  )
}

jalankan_lewat_kontainer() {
  if ! docker ps --format '{{.Names}}' | grep -qx "$NAMA_KONTAINER"; then
    echo "==> Menyalakan kontainer $NAMA_KONTAINER"
    docker rm -f "$NAMA_KONTAINER" >/dev/null 2>&1 || true
    docker run -d --name "$NAMA_KONTAINER" -p 1521:1521 \
      -e ORACLE_PASSWORD="$SANDI" \
      -e APP_USER="$PENGGUNA" \
      -e APP_USER_PASSWORD="$SANDI" \
      "$CITRA" >/dev/null
    echo "==> Menunggu basis data siap"
    for _ in $(seq 1 120); do
      if docker logs "$NAMA_KONTAINER" 2>&1 | grep -q "DATABASE IS READY TO USE!"; then
        break
      fi
      sleep 5
    done
  fi

  docker exec -u root "$NAMA_KONTAINER" bash -c \
    'rm -rf /tmp/aiatlas && mkdir -p /tmp/aiatlas/generated && chown -R oracle:oinstall /tmp/aiatlas'

  # Berkas dikirim lewat stdin, bukan docker cp: pada Windows, docker cp lewat
  # Git Bash menerjemahkan jalur POSIX menjadi jalur Windows dan berkasnya
  # mendarat di tempat yang salah tanpa pesan galat.
  for f in "${BERKAS[@]}"; do
    tr -d '\r' < "oracle/$f" | docker exec -i "$NAMA_KONTAINER" bash -c "cat > /tmp/aiatlas/$f"
  done
  for f in load_vectors.sql jalan.sql jalan2.sql; do
    tr -d '\r' < "oracle/generated/$f" \
      | docker exec -i "$NAMA_KONTAINER" bash -c "cat > /tmp/aiatlas/generated/$f"
  done

  docker exec "$NAMA_KONTAINER" bash -lc \
    "cd /tmp/aiatlas && sqlplus -s $PENGGUNA/$SANDI@$LAYANAN @generated/jalan.sql"
  docker exec "$NAMA_KONTAINER" bash -lc \
    "cd /tmp/aiatlas && sqlplus -s $PENGGUNA/$SANDI@$LAYANAN @generated/jalan2.sql"

  # Hasil spool-nya ada di dalam kontainer; dikeluarkan lewat stdout dengan
  # alasan yang sama seperti pengiriman berkas masuk.
  docker exec "$NAMA_KONTAINER" bash -lc \
    "cat /tmp/aiatlas/generated/plsql-baris.tsv" > "oracle/generated/plsql-baris.tsv"
}

if [ "$MODE" = "langsung" ]; then
  jalankan_sqlplus_langsung
else
  jalankan_lewat_kontainer
fi

# Kepala berkasnya ditulis di sini, bukan di SQL. Membaca versi basis data
# menuntut hak akses ke v$version yang belum tentu dimiliki pengguna aplikasi,
# dan sebuah SELECT yang gagal akan menghentikan seluruh jalan karena
# `WHENEVER SQLERROR EXIT` sedang berlaku. Citra yang dipakai sudah diketahui
# skrip ini.
if [ -s "oracle/generated/plsql-baris.tsv" ]; then
  {
    echo "# ai-atlas — pola bit yang dihitung PL/SQL"
    echo "# bahasa: plsql"
    echo "# versi: $CITRA"
    echo "# dihasilkan: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "# perintah: bash oracle/run.sh"
    printf '# kolom: berkas\tbaris\tkolom\thasil_hex\tkonteks\n'
    # Spasi di ujung dan baris kosong datang dari SQL*Plus, bukan dari datanya.
    sed -e 's/[[:space:]]*$//' -e '/^$/d' "oracle/generated/plsql-baris.tsv"
  } > "oracle/generated/plsql.tsv"
  echo "==> Pola bit PL/SQL: $(( $(wc -l < oracle/generated/plsql.tsv) - 6 )) pernyataan"
else
  echo "==> PERINGATAN: pola bit PL/SQL tidak terbentuk."
fi

echo
echo "==> Oracle AI ATLAS selesai tanpa galat."
