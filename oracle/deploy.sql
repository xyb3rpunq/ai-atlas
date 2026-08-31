-- Memasang seluruh objek AI ATLAS di skema saat ini, lalu melaporkan galat
-- kompilasi kalau ada. Dijalankan ulang dengan aman.
--
-- .Deckyx
WHENEVER SQLERROR EXIT SQL.SQLCODE
SET DEFINE OFF
SET LINESIZE 200
SET PAGESIZE 100
@01_schema.sql
@02_pkg_ai_core.pks
@03_pkg_ai_core.pkb
@04_seed_knowledge.sql
@05_conformance.sql
COLUMN object_name FORMAT A24
COLUMN status FORMAT A10
SELECT object_name, object_type, status FROM user_objects WHERE object_type LIKE 'PACKAGE%' ORDER BY 1,2;
COLUMN text FORMAT A90
SELECT name, line, position, text FROM user_errors ORDER BY name, sequence;
EXIT
