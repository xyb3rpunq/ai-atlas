// Uji untuk harness konformansi itu sendiri.
//
// Harness yang selalu lolos tidak berguna. Uji di berkas ini membuktikan bahwa
// pembandingnya benar-benar bisa gagal: ketidakcocokan sekecil satu ULP pada
// tingkat BitExact harus tertangkap, dan sebaliknya toleransi tidak boleh
// begitu longgar sampai meloloskan cacat sungguhan.
//
// .Deckyx
package main

import (
	"math"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/xyb3rpunq/ai-atlas/tools/conform/aicore"
)

func TestBacaKeterbandingan(t *testing.T) {
	kasus := []struct {
		masukan string
		nama    string
		ulp     int64
		sifat   bool
	}{
		{"BitExact", "BitExact", 0, false},
		{"  BitExact  ", "BitExact", 0, false},
		{"NearlyEqual(4)", "NearlyEqual(4)", 4, false},
		{"NearlyEqual(0)", "NearlyEqual(0)", 0, false},
		{"PropertyOnly", "PropertyOnly", 0, true},
	}
	for _, k := range kasus {
		got, err := BacaKeterbandingan(k.masukan)
		if err != nil {
			t.Fatalf("%q: %v", k.masukan, err)
		}
		if got.Nama != k.nama || got.MaksUlp != k.ulp || got.SifatSaja != k.sifat {
			t.Errorf("%q menghasilkan %+v", k.masukan, got)
		}
	}

	for _, buruk := range []string{"", "Entah", "NearlyEqual", "NearlyEqual(x)", "bitexact"} {
		if _, err := BacaKeterbandingan(buruk); err == nil {
			t.Errorf("%q seharusnya ditolak", buruk)
		}
	}
}

func TestBitExactMenangkapSatuUlp(t *testing.T) {
	// Inti dari seluruh harness ini. Selisih satu ULP adalah cacat terkecil
	// yang mungkin ada, dan justru itulah yang paling sulit ditemukan tanpa
	// perbandingan lintas implementasi.
	k, _ := BacaKeterbandingan("BitExact")
	a := 0.42
	b := math.Float64frombits(math.Float64bits(a) + 1)

	if a == b {
		t.Fatal("prasyarat uji: kedua nilai harus berbeda")
	}
	if k.Terpenuhi(a, b) {
		t.Error("BitExact meloloskan selisih satu ULP")
	}
	if !k.Terpenuhi(a, a) {
		t.Error("BitExact menolak nilai yang identik")
	}
}

func TestNearlyEqualMenghormatiBatasnya(t *testing.T) {
	k, _ := BacaKeterbandingan("NearlyEqual(4)")
	a := 1.0
	empat := math.Float64frombits(math.Float64bits(a) + 4)
	lima := math.Float64frombits(math.Float64bits(a) + 5)

	if !k.Terpenuhi(a, empat) {
		t.Error("empat ULP seharusnya lolos toleransi empat")
	}
	if k.Terpenuhi(a, lima) {
		t.Error("lima ULP seharusnya ditolak toleransi empat")
	}
	// Toleransi tidak boleh begitu longgar sampai meloloskan cacat nyata.
	if k.Terpenuhi(1.0, 1.001) {
		t.Error("selisih besar meloloskan pemeriksaan")
	}
}

func TestNilaiKhusus(t *testing.T) {
	k, _ := BacaKeterbandingan("BitExact")
	nan := math.NaN()
	inf := math.Inf(1)

	if !k.Terpenuhi(nan, nan) {
		t.Error("dua nilai bukan bilangan seharusnya dinilai sepadan")
	}
	if !k.Terpenuhi(inf, inf) {
		t.Error("dua tak hingga sejenis seharusnya sepadan")
	}
	if k.Terpenuhi(inf, math.Inf(-1)) {
		t.Error("tak hingga berlawanan tanda seharusnya ditolak")
	}
	if k.Terpenuhi(nan, 1.0) {
		t.Error("bukan bilangan dibanding bilangan seharusnya ditolak")
	}
	// Nol positif dan nol negatif adalah pola bit berbeda, dan pada tingkat
	// BitExact perbedaan itu harus dilaporkan. Jarak ULP tidak cukup di sini:
	// IEEE-754 menyatakan 0.0 == -0.0, sehingga jaraknya nol padahal kedua
	// nilai itu menyebar berbeda ketika dibagi.
	if k.Terpenuhi(0.0, math.Copysign(0, -1)) {
		t.Error("nol positif dan negatif seharusnya dibedakan pada tingkat bit")
	}
	// Sebaliknya, toleransi yang longgar boleh meloloskannya.
	longgar, _ := BacaKeterbandingan("NearlyEqual(4)")
	if !longgar.Terpenuhi(0.0, math.Copysign(0, -1)) {
		t.Error("toleransi ULP seharusnya meloloskan kedua bentuk nol")
	}
}

func TestPropertyOnlyTidakMembandingkanAngka(t *testing.T) {
	k, _ := BacaKeterbandingan("PropertyOnly")
	if !k.Terpenuhi(1.0, 1000.0) {
		t.Error("PropertyOnly seharusnya tidak membandingkan angka sama sekali")
	}
	if !k.Terpenuhi(math.NaN(), 0.0) {
		t.Error("PropertyOnly seharusnya meloloskan apa pun")
	}
}

func TestBolakBalikHex(t *testing.T) {
	nilai := []float64{
		0, math.Copysign(0, -1), 1, -1, 0.1, 0.42,
		0.9*0.2 + 0.3*0.8,
		math.Pi, math.SmallestNonzeroFloat64, math.MaxFloat64,
		math.Inf(1), math.Inf(-1),
	}
	for _, v := range nilai {
		balik, err := aicore.DariHex(aicore.KeHex(v))
		if err != nil {
			t.Fatalf("%v: %v", v, err)
		}
		if !aicore.SamaBit(v, balik) {
			t.Errorf("%v tidak selamat bolak-balik: %v", v, balik)
		}
	}

	// Bentuk yang salah harus ditolak, bukan diam-diam menghasilkan angka lain.
	for _, buruk := range []string{"", "3ff", "3ff00000000000000", "3ff000000000000z"} {
		if _, err := aicore.DariHex(buruk); err == nil {
			t.Errorf("%q seharusnya ditolak", buruk)
		}
	}
}

func TestJarakUlp(t *testing.T) {
	a := 1.0
	b := math.Float64frombits(math.Float64bits(a) + 3)
	if d := aicore.JarakUlp(a, b); d != 3 {
		t.Errorf("jarak seharusnya 3, diperoleh %d", d)
	}
	if d := aicore.JarakUlp(a, a); d != 0 {
		t.Errorf("jarak ke diri sendiri seharusnya 0, diperoleh %d", d)
	}
	if d := aicore.JarakUlp(math.NaN(), 1.0); d != -1 {
		t.Errorf("jarak ke bukan bilangan seharusnya tidak terdefinisi, diperoleh %d", d)
	}
	// Jaraknya setangkup.
	if aicore.JarakUlp(a, b) != aicore.JarakUlp(b, a) {
		t.Error("jarak ULP seharusnya setangkup")
	}
}

// TestHarnessMenangkapVektorYangDirusak adalah uji yang paling penting di
// berkas ini: ia membuktikan bahwa harness benar-benar akan berteriak kalau
// salah satu implementasi berubah perilaku.
func TestHarnessMenangkapVektorYangDirusak(t *testing.T) {
	dir := t.TempDir()

	// Satu baris sengaja dirusak sebesar satu ULP.
	benar := aicore.GabungParalel(0.8, 0.6)
	rusak := math.Float64frombits(math.Float64bits(benar) + 1)

	isi := strings.Join([]string{
		"# ai-atlas vektor uji",
		"# keterbandingan: BitExact",
		"# kolom: op\ta_hex\tb_hex\tresult_hex",
		strings.Join([]string{"parallel", aicore.KeHex(0.8), aicore.KeHex(0.6), aicore.KeHex(benar)}, "\t"),
		strings.Join([]string{"parallel", aicore.KeHex(0.8), aicore.KeHex(0.6), aicore.KeHex(rusak)}, "\t"),
		"",
	}, "\n")

	path := filepath.Join(dir, "certainty.tsv")
	if err := os.WriteFile(path, []byte(isi), 0o600); err != nil {
		t.Fatal(err)
	}

	berkas, err := MuatBerkas(path)
	if err != nil {
		t.Fatal(err)
	}
	hasil, err := PeriksaBerkas(berkas)
	if err != nil {
		t.Fatal(err)
	}

	if hasil.Diperiksa != 2 {
		t.Fatalf("seharusnya memeriksa 2 baris, memeriksa %d", hasil.Diperiksa)
	}
	if len(hasil.Gagal) != 1 {
		t.Fatalf("seharusnya menangkap tepat 1 ketidakcocokan, menangkap %d", len(hasil.Gagal))
	}
	if hasil.Gagal[0].Baris != 2 {
		t.Errorf("ketidakcocokan seharusnya di baris 2, dilaporkan di baris %d", hasil.Gagal[0].Baris)
	}
	if hasil.Gagal[0].JarakUlp != 1 {
		t.Errorf("jarak seharusnya 1 ULP, dilaporkan %d", hasil.Gagal[0].JarakUlp)
	}
	// Pesannya harus memuat kedua pola bit supaya bisa ditelusuri.
	pesan := hasil.Gagal[0].String()
	if !strings.Contains(pesan, aicore.KeHex(benar)) || !strings.Contains(pesan, aicore.KeHex(rusak)) {
		t.Errorf("pesan tidak memuat kedua pola bit:\n%s", pesan)
	}
}

func TestBerkasTanpaKeterbandinganDitolak(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "certainty.tsv")
	isi := "# tanpa penanda apa pun\n" + strings.Join(
		[]string{"parallel", aicore.KeHex(0.5), aicore.KeHex(0.5), aicore.KeHex(0.75)}, "\t") + "\n"
	if err := os.WriteFile(path, []byte(isi), 0o600); err != nil {
		t.Fatal(err)
	}
	if _, err := MuatBerkas(path); err == nil {
		t.Error("berkas tanpa penanda keterbandingan seharusnya ditolak")
	}
}

func TestSeluruhVektorSungguhanCocok(t *testing.T) {
	// Menjalankan berkas vektor yang sebenarnya, bila tersedia. Dilewati bila
	// belum dihasilkan, supaya `go test` tetap berguna tanpa toolchain Rust.
	entri, err := os.ReadDir("vectors")
	if err != nil {
		t.Skip("vektor belum dihasilkan; jalankan export_vectors lebih dulu")
	}

	totalDiperiksa := 0
	for _, e := range entri {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".tsv") {
			continue
		}
		berkas, err := MuatBerkas(filepath.Join("vectors", e.Name()))
		if err != nil {
			t.Fatalf("%s: %v", e.Name(), err)
		}
		hasil, err := PeriksaBerkas(berkas)
		if err != nil {
			t.Fatalf("%s: %v", e.Name(), err)
		}
		if hasil.Diperiksa == 0 {
			t.Errorf("%s tidak memeriksa satu baris pun", e.Name())
		}
		for _, m := range hasil.Gagal {
			t.Errorf("ketidakcocokan:\n%s", m)
		}
		totalDiperiksa += hasil.Diperiksa
	}

	if totalDiperiksa < 1000 {
		t.Errorf("hanya %d vektor diperiksa; kumpulannya terlalu kecil untuk berarti", totalDiperiksa)
	}
	t.Logf("%d vektor cocok antara Rust dan Go", totalDiperiksa)
}
