// Perintah conform mengadu implementasi Go terhadap vektor uji yang dihasilkan
// implementasi Rust.
//
// Cara kerjanya sederhana dan itulah gunanya: Rust menulis masukan beserta
// jawabannya sebagai pola bit, Go membaca masukan itu, menghitung ulang dengan
// kodenya sendiri, lalu membandingkan. Rumus yang salah tetap konsisten dengan
// dirinya sendiri, sehingga uji terhadap satu implementasi tidak bisa
// menangkapnya — tetapi hampir mustahil dua implementasi yang ditulis terpisah
// salah dengan cara yang persis sama.
//
// Tiap berkas vektor menyatakan tingkat keterbandingannya:
//
//	BitExact                  wajib identik bit demi bit; hanya untuk + - x / sqrt
//	NearlyEqual(n)            boleh berbeda n ULP; untuk exp, ln, log2, pow
//	CancellingDifference(n)   boleh berbeda n ULP diukur pada skala masukannya
//	PropertyOnly              hanya sifatnya yang diuji, bukan angkanya
//
// Pembagian itu bukan kelonggaran melainkan keharusan: IEEE-754 hanya
// mewajibkan enam operasi dibulatkan dengan benar, dan fungsi transendental
// tidak termasuk. Menuntut kesamaan bit pada `exp` akan menghasilkan kegagalan
// yang berselang-seling tanpa ada yang benar-benar salah.
//
// .Deckyx
package main

import (
	"bufio"
	"fmt"
	"math"
	"os"
	"path/filepath"
	"regexp"
	"runtime"
	"strconv"
	"strings"
	"time"

	"github.com/xyb3rpunq/ai-atlas/tools/conform/aicore"
)

// Keterbandingan menyatakan seberapa jauh dua implementasi boleh berbeda.
type Keterbandingan struct {
	Nama    string
	MaksUlp int64
	// SifatSaja berarti angkanya tidak dibandingkan sama sekali.
	SifatSaja bool
	// PakaiSkala berarti toleransinya diukur pada skala yang disertakan
	// berkas vektor, bukan pada hasilnya. Dipakai untuk besaran yang berupa
	// selisih dua nilai yang hampir sama, karena di sana galat kecil pada
	// masukannya membesar pada hasilnya tanpa ada perhitungan yang keliru.
	PakaiSkala bool
}

var polaNearly = regexp.MustCompile(`^NearlyEqual\((\d+)\)$`)
var polaSelisih = regexp.MustCompile(`^CancellingDifference\((\d+)\)$`)

// BacaKeterbandingan menguraikan penanda keterbandingan dari kepala berkas.
func BacaKeterbandingan(s string) (Keterbandingan, error) {
	t := strings.TrimSpace(s)
	switch t {
	case "BitExact":
		return Keterbandingan{Nama: t, MaksUlp: 0}, nil
	case "PropertyOnly":
		return Keterbandingan{Nama: t, SifatSaja: true}, nil
	}
	if m := polaNearly.FindStringSubmatch(t); m != nil {
		n, err := strconv.ParseInt(m[1], 10, 64)
		if err != nil {
			return Keterbandingan{}, fmt.Errorf("toleransi tidak terbaca: %q", t)
		}
		return Keterbandingan{Nama: t, MaksUlp: n}, nil
	}
	if m := polaSelisih.FindStringSubmatch(t); m != nil {
		n, err := strconv.ParseInt(m[1], 10, 64)
		if err != nil {
			return Keterbandingan{}, fmt.Errorf("toleransi tidak terbaca: %q", t)
		}
		return Keterbandingan{Nama: t, MaksUlp: n, PakaiSkala: true}, nil
	}
	return Keterbandingan{}, fmt.Errorf("tingkat keterbandingan tidak dikenal: %q", t)
}

// Terpenuhi memeriksa apakah dua nilai memenuhi tingkat keterbandingan ini.
//
// Pada tingkat BitExact yang dituntut adalah kesamaan pola bit, bukan jarak
// ULP nol. Keduanya terlihat sama tetapi tidak sama: IEEE-754 menyatakan
// 0.0 == -0.0 bernilai benar, sehingga jarak ULP di antara keduanya nol,
// padahal pola bitnya berbeda dan menyebar berbeda pula — 1/+0 menghasilkan
// tak hingga positif sedangkan 1/-0 menghasilkan tak hingga negatif. Dua
// implementasi yang berbeda tanda nolnya memang berbeda, dan pada tingkat
// yang bernama "bit exact" perbedaan itu harus dilaporkan.
func (k Keterbandingan) Terpenuhi(a, b float64) bool {
	if k.SifatSaja {
		return true
	}
	if aicore.SamaBit(a, b) {
		return true
	}
	// Tingkat berskala menuntut skala, dan skalanya tidak ada di sini. Yang
	// dikembalikan adalah pemeriksaan paling ketat, bukan paling longgar:
	// pemanggil yang lupa memberi skala melihat kegagalan, bukan kelolosan
	// palsu.
	if k.PakaiSkala {
		return false
	}
	if k.MaksUlp == 0 {
		return false
	}
	d := aicore.JarakUlp(a, b)
	if d < 0 {
		return false
	}
	return d <= k.MaksUlp
}

// TerpenuhiSkala seperti Terpenuhi, tetapi menyertakan skala tempat
// aritmetikanya terjadi. Hanya tingkat berskala yang memakainya.
func (k Keterbandingan) TerpenuhiSkala(a, b, skala float64) bool {
	if !k.PakaiSkala {
		return k.Terpenuhi(a, b)
	}
	if aicore.SamaBit(a, b) {
		return true
	}
	if math.IsNaN(a) || math.IsInf(a, 0) ||
		math.IsNaN(b) || math.IsInf(b, 0) ||
		math.IsNaN(skala) || math.IsInf(skala, 0) {
		return false
	}
	return math.Abs(a-b) <= float64(k.MaksUlp)*aicore.LangkahUlp(skala)
}

// Berkas adalah satu berkas vektor yang sudah diuraikan.
type Berkas struct {
	Nama           string
	Keterbandingan Keterbandingan
	Kolom          []string
	Baris          [][]string
}

// MuatBerkas membaca sebuah berkas vektor.
func MuatBerkas(path string) (*Berkas, error) {
	f, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer f.Close()

	b := &Berkas{Nama: filepath.Base(path)}
	pemindai := bufio.NewScanner(f)
	// Baris vektor bisa panjang; penyangga bawaan 64 KB dinaikkan agar tidak
	// memotong baris di tengah dan menghasilkan kegagalan palsu.
	pemindai.Buffer(make([]byte, 0, 1<<20), 1<<20)

	for pemindai.Scan() {
		baris := pemindai.Text()
		if strings.HasPrefix(baris, "#") {
			isi := strings.TrimSpace(strings.TrimPrefix(baris, "#"))
			if strings.HasPrefix(isi, "keterbandingan:") {
				k, err := BacaKeterbandingan(strings.TrimPrefix(isi, "keterbandingan:"))
				if err != nil {
					return nil, fmt.Errorf("%s: %w", b.Nama, err)
				}
				b.Keterbandingan = k
			}
			if strings.HasPrefix(isi, "kolom:") {
				b.Kolom = strings.Split(strings.TrimSpace(strings.TrimPrefix(isi, "kolom:")), "\t")
			}
			continue
		}
		if strings.TrimSpace(baris) == "" {
			continue
		}
		b.Baris = append(b.Baris, strings.Split(baris, "\t"))
	}
	if err := pemindai.Err(); err != nil {
		return nil, err
	}
	if b.Keterbandingan.Nama == "" {
		return nil, fmt.Errorf("%s: kepala berkas tidak menyatakan keterbandingan", b.Nama)
	}
	return b, nil
}

// Ketidakcocokan mencatat satu baris yang gagal.
type Ketidakcocokan struct {
	Berkas    string
	Baris     int
	Konteks   string
	Harapan   float64
	Diperoleh float64
	JarakUlp  int64
}

func (m Ketidakcocokan) String() string {
	jarak := "tidak terdefinisi"
	if m.JarakUlp >= 0 {
		jarak = fmt.Sprintf("%d ULP", m.JarakUlp)
	}
	return fmt.Sprintf(
		"  %s baris %d [%s]\n    Rust : %s (%v)\n    Go   : %s (%v)\n    jarak: %s",
		m.Berkas, m.Baris, m.Konteks,
		aicore.KeHex(m.Harapan), m.Harapan,
		aicore.KeHex(m.Diperoleh), m.Diperoleh,
		jarak,
	)
}

// Hasil merangkum satu berkas yang sudah diperiksa.
type Hasil struct {
	Berkas         string
	Keterbandingan string
	Diperiksa      int
	Gagal          []Ketidakcocokan
	// Dilewati mencatat baris yang tidak punya pemeriksa di sisi Go.
	Dilewati int
}

// Nilai adalah satu besaran yang dihitung ulang dari sebuah baris vektor.
//
// Satu baris bisa menghasilkan lebih dari satu: `bayes.tsv` menuliskan peluang
// bukti, posterior, dan rasio kemungkinan sekaligus, dan `rng.tsv` menuliskan
// bilangan bulat beserta pecahan yang diturunkan darinya.
//
// Mengembalikan seluruhnya membuat pemeriksa dan pemancar berbagi satu sumber
// perhitungan. Dua daftar terpisah pasti menyimpang cepat atau lambat — dan
// yang menyimpang adalah alat yang justru dibuat untuk membuktikan tidak ada
// yang menyimpang.
type Nilai struct {
	// Kolom menyebut kolom hasil di berkas vektornya, apa adanya. Nama itu
	// yang dipakai memasangkan hasil Go dengan hasil bahasa lain.
	Kolom     string
	Harapan   float64
	Diperoleh float64
	Konteks   string
}

// pemeriksa menghitung ulang satu baris dan mengembalikan seluruh besaran yang
// dihasilkannya. Mengembalikan false bila baris ini tidak ditangani.
type pemeriksa func(baris []string) (nilai []Nilai, ditangani bool, err error)

func hexAtau(s string) (float64, error) {
	return aicore.DariHex(s)
}

// periksaRng mencocokkan deret SplitMix64.
//
// Bilangan bulatnya dibandingkan sebagai pola bit, sama seperti pecahannya.
// Kegagalan pada bilangan bulat jauh lebih serius karena tidak ada pembulatan
// yang terlibat sama sekali; jaraknya dalam ULP tidak punya arti di sana, dan
// berkas ini memang bertingkat BitExact.
func periksaRng(baris []string) ([]Nilai, bool, error) {
	if len(baris) < 4 {
		return nil, false, fmt.Errorf("baris rng tidak lengkap")
	}
	seed, err := strconv.ParseUint(baris[0], 10, 64)
	if err != nil {
		return nil, false, err
	}
	index, err := strconv.Atoi(baris[1])
	if err != nil {
		return nil, false, err
	}
	harapanU, err := strconv.ParseUint(baris[2], 16, 64)
	if err != nil {
		return nil, false, err
	}
	harapanF, err := hexAtau(baris[3])
	if err != nil {
		return nil, false, err
	}

	r := aicore.BaruSplitMix64(seed)
	var u uint64
	for i := 0; i <= index; i++ {
		u = r.NextU64()
	}
	rf := aicore.BaruSplitMix64(seed)
	var f float64
	for i := 0; i <= index; i++ {
		f = rf.NextF64()
	}

	konteks := fmt.Sprintf("benih %d indeks %d", seed, index)
	hu := math.Float64frombits(harapanU)
	du := math.Float64frombits(u)
	return []Nilai{
		{Kolom: "next_u64_hex", Harapan: hu, Diperoleh: du, Konteks: "next_u64 " + konteks},
		{Kolom: "next_f64_hex", Harapan: harapanF, Diperoleh: f, Konteks: "next_f64 " + konteks},
	}, true, nil
}

// periksaCertainty mencocokkan perhitungan certainty factor.
func periksaCertainty(baris []string) ([]Nilai, bool, error) {
	if len(baris) < 4 {
		return nil, false, fmt.Errorf("baris certainty tidak lengkap")
	}
	a, err := hexAtau(baris[1])
	if err != nil {
		return nil, false, err
	}
	b, err := hexAtau(baris[2])
	if err != nil {
		return nil, false, err
	}
	harapan, err := hexAtau(baris[3])
	if err != nil {
		return nil, false, err
	}

	var diperoleh float64
	switch baris[0] {
	case "parallel":
		diperoleh = aicore.GabungParalel(a, b)
	case "sequential":
		diperoleh = aicore.GabungBerantai(a, b)
	case "and":
		diperoleh = aicore.GabungDan([]float64{a, b})
	case "or":
		diperoleh = aicore.GabungAtau([]float64{a, b})
	case "mb_md":
		diperoleh = aicore.CfDariMbMd(a, b)
	default:
		return nil, false, nil
	}
	return []Nilai{{
		Kolom:     "result_hex",
		Harapan:   harapan,
		Diperoleh: diperoleh,
		Konteks:   fmt.Sprintf("%s(%v, %v)", baris[0], a, b),
	}}, true, nil
}

// periksaBayes mencocokkan perhitungan Bayesian.
func periksaBayes(baris []string) ([]Nilai, bool, error) {
	if len(baris) < 6 {
		return nil, false, fmt.Errorf("baris bayes tidak lengkap")
	}
	nilai := make([]float64, 6)
	for i := 0; i < 6; i++ {
		v, err := hexAtau(baris[i])
		if err != nil {
			return nil, false, err
		}
		nilai[i] = v
	}
	hasil := aicore.BayesBiner(nilai[0], nilai[1], nilai[2])
	konteks := fmt.Sprintf("prior %v", nilai[0])

	lr := hasil.RasioKemungkinan
	return []Nilai{
		{Kolom: "evidence_hex", Harapan: nilai[3], Diperoleh: hasil.Bukti, Konteks: "P(E) pada " + konteks},
		{Kolom: "posterior_hex", Harapan: nilai[4], Diperoleh: hasil.Posterior, Konteks: "P(H|E) pada " + konteks},
		{Kolom: "likelihood_ratio_hex", Harapan: nilai[5], Diperoleh: lr, Konteks: "LR+ pada " + konteks},
	}, true, nil
}

// periksaFuzzyLinear mencocokkan keanggotaan segitiga dan trapesium.
func periksaFuzzyLinear(baris []string) ([]Nilai, bool, error) {
	if len(baris) < 7 {
		return nil, false, fmt.Errorf("baris fuzzy tidak lengkap")
	}
	p1, _ := hexAtau(baris[1])
	p2, _ := hexAtau(baris[2])
	p3, _ := hexAtau(baris[3])
	p4, _ := hexAtau(baris[4])
	x, err := hexAtau(baris[5])
	if err != nil {
		return nil, false, err
	}
	harapan, err := hexAtau(baris[6])
	if err != nil {
		return nil, false, err
	}

	var diperoleh float64
	switch baris[0] {
	case "triangular":
		diperoleh = aicore.KeanggotaanSegitiga(p1, p2, p3, x)
	case "trapezoidal":
		diperoleh = aicore.KeanggotaanTrapesium(p1, p2, p3, p4, x)
	default:
		return nil, false, nil
	}
	return []Nilai{{
		Kolom:     "degree_hex",
		Harapan:   harapan,
		Diperoleh: diperoleh,
		Konteks:   fmt.Sprintf("%s pada x=%v", baris[0], x),
	}}, true, nil
}

// periksaFuzzyTranscendental mencocokkan keanggotaan Gauss dan sigmoid.
func periksaFuzzyTranscendental(baris []string) ([]Nilai, bool, error) {
	if len(baris) < 5 {
		return nil, false, fmt.Errorf("baris fuzzy tidak lengkap")
	}
	p1, _ := hexAtau(baris[1])
	p2, _ := hexAtau(baris[2])
	x, err := hexAtau(baris[3])
	if err != nil {
		return nil, false, err
	}
	harapan, err := hexAtau(baris[4])
	if err != nil {
		return nil, false, err
	}

	var diperoleh float64
	switch baris[0] {
	case "gaussian":
		diperoleh = aicore.KeanggotaanGauss(p1, p2, x)
	case "sigmoid":
		diperoleh = aicore.KeanggotaanSigmoid(p1, p2, x)
	default:
		return nil, false, nil
	}
	return []Nilai{{
		Kolom:     "degree_hex",
		Harapan:   harapan,
		Diperoleh: diperoleh,
		Konteks:   fmt.Sprintf("%s pada x=%v", baris[0], x),
	}}, true, nil
}

// periksaMlExact mencocokkan jarak dan ketakmurnian Gini.
func periksaMlExact(baris []string) ([]Nilai, bool, error) {
	if len(baris) < 6 {
		return nil, false, fmt.Errorf("baris ml tidak lengkap")
	}
	harapan, err := hexAtau(baris[5])
	if err != nil {
		return nil, false, err
	}

	if baris[0] == "gini" {
		labels := strings.Split(baris[1], ",")
		return []Nilai{{
			Kolom:     "result_hex",
			Harapan:   harapan,
			Diperoleh: aicore.Gini(labels),
			Konteks:   "gini " + baris[1],
		}}, true, nil
	}

	ax, _ := hexAtau(baris[1])
	ay, _ := hexAtau(baris[2])
	bx, _ := hexAtau(baris[3])
	by, _ := hexAtau(baris[4])
	a := []float64{ax, ay}
	b := []float64{bx, by}

	var diperoleh float64
	switch baris[0] {
	case "euclidean":
		diperoleh = aicore.JarakEuclidean(a, b)
	case "manhattan":
		diperoleh = aicore.JarakManhattan(a, b)
	case "chebyshev":
		diperoleh = aicore.JarakChebyshev(a, b)
	default:
		return nil, false, nil
	}
	return []Nilai{{
		Kolom:     "result_hex",
		Harapan:   harapan,
		Diperoleh: diperoleh,
		Konteks:   fmt.Sprintf("%s(%v, %v)", baris[0], a, b),
	}}, true, nil
}

// periksaMlEntropy mencocokkan entropi dan perolehan informasi.
func periksaMlEntropy(baris []string) ([]Nilai, bool, error) {
	if len(baris) < 4 {
		return nil, false, fmt.Errorf("baris entropi tidak lengkap")
	}
	harapan, err := hexAtau(baris[3])
	if err != nil {
		return nil, false, err
	}
	labels := strings.Split(baris[1], ",")

	switch baris[0] {
	case "entropy":
		return []Nilai{{
			Kolom:     "result_hex",
			Harapan:   harapan,
			Diperoleh: aicore.Entropi(labels),
			Konteks:   "entropi " + baris[1],
		}}, true, nil
	case "information_gain":
		bagian := strings.SplitN(baris[2], "=", 2)
		if len(bagian) != 2 {
			return nil, false, fmt.Errorf("atribut tidak terbaca: %q", baris[2])
		}
		values := strings.Split(bagian[1], ",")
		return []Nilai{{
			Kolom:     "result_hex",
			Harapan:   harapan,
			Diperoleh: aicore.PerolehanInformasi(values, labels),
			Konteks:   "perolehan " + bagian[0],
		}}, true, nil
	}
	return nil, false, nil
}

// periksaMlGain mencocokkan perolehan informasi.
//
// Terpisah dari entropi karena tingkat keterbandingannya berbeda: hasilnya
// adalah selisih dua entropi yang hampir sama besar, sehingga toleransinya
// diukur pada kolom skala, bukan pada hasilnya.
func periksaMlGain(baris []string) ([]Nilai, bool, error) {
	if len(baris) < 5 {
		return nil, false, fmt.Errorf("baris perolehan tidak lengkap")
	}
	harapan, err := hexAtau(baris[4])
	if err != nil {
		return nil, false, err
	}
	labels := strings.Split(baris[1], ",")
	bagian := strings.SplitN(baris[2], "=", 2)
	if len(bagian) != 2 {
		return nil, false, fmt.Errorf("atribut tidak terbaca: %q", baris[2])
	}
	values := strings.Split(bagian[1], ",")
	return []Nilai{{
		Kolom:     "result_hex",
		Harapan:   harapan,
		Diperoleh: aicore.PerolehanInformasi(values, labels),
		Konteks:   "perolehan " + bagian[0],
	}}, true, nil
}

// periksaFx mencocokkan bolak-balik pola bit.
func periksaFx(baris []string) ([]Nilai, bool, error) {
	if len(baris) < 2 {
		return nil, false, fmt.Errorf("baris fx tidak lengkap")
	}
	v, err := hexAtau(baris[1])
	if err != nil {
		return nil, false, err
	}
	// Membaca hex lalu menuliskannya kembali harus menghasilkan teks yang sama.
	ulang, err := hexAtau(aicore.KeHex(v))
	if err != nil {
		return nil, false, err
	}
	return []Nilai{{
		Kolom:     "hex",
		Harapan:   v,
		Diperoleh: ulang,
		Konteks:   "bolak-balik " + baris[0],
	}}, true, nil
}

var pemeriksaBerkas = map[string]pemeriksa{
	"rng.tsv":                  periksaRng,
	"certainty.tsv":            periksaCertainty,
	"bayes.tsv":                periksaBayes,
	"fuzzy_linear.tsv":         periksaFuzzyLinear,
	"fuzzy_transcendental.tsv": periksaFuzzyTranscendental,
	"ml_exact.tsv":             periksaMlExact,
	"ml_entropy.tsv":           periksaMlEntropy,
	"ml_gain.tsv":              periksaMlGain,
	"fx.tsv":                   periksaFx,
}

// PeriksaBerkas menjalankan seluruh baris sebuah berkas.
func PeriksaBerkas(b *Berkas) (Hasil, error) {
	fn, ada := pemeriksaBerkas[b.Nama]
	if !ada {
		return Hasil{}, fmt.Errorf("tidak ada pemeriksa untuk %s", b.Nama)
	}

	// Tingkat berskala memerlukan kolom skala; berkasnya harus menyebutkannya
	// di baris kepala. Berkas yang menyatakan tingkat itu tanpa kolomnya
	// adalah salah tulis, bukan alasan untuk diam-diam melonggarkan periksa.
	kolomSkala := -1
	if b.Keterbandingan.PakaiSkala {
		for i, nama := range b.Kolom {
			if nama == "scale_hex" {
				kolomSkala = i
			}
		}
		if kolomSkala < 0 {
			return Hasil{}, fmt.Errorf("%s: tingkat %s menuntut kolom scale_hex",
				b.Nama, b.Keterbandingan.Nama)
		}
	}

	hasil := Hasil{Berkas: b.Nama, Keterbandingan: b.Keterbandingan.Nama}
	for i, baris := range b.Baris {
		nilai, ditangani, err := fn(baris)
		if err != nil {
			return hasil, fmt.Errorf("%s baris %d: %w", b.Nama, i+1, err)
		}
		if !ditangani {
			hasil.Dilewati++
			continue
		}
		skala := math.NaN()
		if kolomSkala >= 0 {
			if kolomSkala >= len(baris) {
				return hasil, fmt.Errorf("%s baris %d: kolom scale_hex kosong", b.Nama, i+1)
			}
			skala, err = aicore.DariHex(baris[kolomSkala])
			if err != nil {
				return hasil, fmt.Errorf("%s baris %d: skala tidak terbaca: %w", b.Nama, i+1, err)
			}
		}

		// Dihitung per besaran, bukan per baris. Satu baris `bayes.tsv`
		// menghasilkan tiga pernyataan dan satu baris `rng.tsv` dua, jadi
		// menghitung baris membuat Go melaporkan 2.266 sementara Lua dan
		// Oracle melaporkan 3.796 untuk vektor yang sama persis.
		for _, v := range nilai {
			hasil.Diperiksa++
			if b.Keterbandingan.TerpenuhiSkala(v.Harapan, v.Diperoleh, skala) {
				continue
			}
			hasil.Gagal = append(hasil.Gagal, Ketidakcocokan{
				Berkas:    b.Nama,
				Baris:     i + 1,
				Konteks:   v.Konteks,
				Harapan:   v.Harapan,
				Diperoleh: v.Diperoleh,
				JarakUlp:  aicore.JarakUlp(v.Harapan, v.Diperoleh),
			})
		}
	}
	return hasil, nil
}

// PancarkanBerkas menuliskan pola bit yang dihitung Go untuk seluruh baris.
//
// Perhitungannya sama persis dengan yang dipakai memeriksa — daftar `Nilai`
// yang sama, dari fungsi yang sama. Yang berbeda hanya apa yang dilakukan
// terhadapnya: dibandingkan, atau dituliskan.
func PancarkanBerkas(b *Berkas, tulis func(baris int, kolom, hex, konteks string)) error {
	fn, ada := pemeriksaBerkas[b.Nama]
	if !ada {
		return fmt.Errorf("tidak ada pemeriksa untuk %s", b.Nama)
	}
	for i, baris := range b.Baris {
		nilai, ditangani, err := fn(baris)
		if err != nil {
			return fmt.Errorf("%s baris %d: %w", b.Nama, i+1, err)
		}
		if !ditangani {
			continue
		}
		for _, v := range nilai {
			tulis(i+1, v.Kolom, aicore.KeHex(v.Diperoleh), v.Konteks)
		}
	}
	return nil
}

// pancar menuliskan pola bit Go ke keluaran baku dalam bentuk TSV.
//
// Berkasnya dibaca halaman "Enam bahasa, satu angka": ia memasangkan hasil tiap
// bahasa lewat kunci (berkas, baris, kolom). Kuncinya sengaja sama dengan yang
// dipakai tabel `ai_conformance_vector` di Oracle, supaya keenam bahasa bisa
// disandingkan tanpa satu pun penyesuaian.
func pancar(dir string, entri []os.DirEntry) error {
	out := bufio.NewWriter(os.Stdout)
	defer out.Flush()

	fmt.Fprintln(out, "# ai-atlas — pola bit yang dihitung Go")
	fmt.Fprintln(out, "# bahasa: go")
	fmt.Fprintf(out, "# versi: %s\n", runtime.Version())
	fmt.Fprintf(out, "# dihasilkan: %s\n", time.Now().UTC().Format(time.RFC3339))
	fmt.Fprintln(out, "# perintah: go run ./tools/conform --pancar")
	fmt.Fprintln(out, "# kolom: berkas\tbaris\tkolom\thasil_hex\tkonteks")

	for _, e := range entri {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".tsv") {
			continue
		}
		berkas, err := MuatBerkas(filepath.Join(dir, e.Name()))
		if err != nil {
			return err
		}
		nama := e.Name()
		if err := PancarkanBerkas(berkas, func(baris int, kolom, hex, konteks string) {
			fmt.Fprintf(out, "%s\t%d\t%s\t%s\t%s\n", nama, baris, kolom, hex, konteks)
		}); err != nil {
			return err
		}
	}
	return nil
}

func main() {
	dir := "vectors"
	memancar := false
	for _, a := range os.Args[1:] {
		if a == "--pancar" {
			memancar = true
			continue
		}
		dir = a
	}

	entri, err := os.ReadDir(dir)
	if err != nil {
		fmt.Fprintf(os.Stderr, "gagal membaca %s: %v\n", dir, err)
		fmt.Fprintln(os.Stderr, "jalankan `cargo run -p ai-core --bin export_vectors` lebih dulu")
		os.Exit(1)
	}

	if memancar {
		if err := pancar(dir, entri); err != nil {
			fmt.Fprintf(os.Stderr, "GAGAL: %v\n", err)
			os.Exit(1)
		}
		return
	}

	fmt.Println("Konformansi Rust terhadap Go — AI ATLAS .Deckyx")
	fmt.Println(strings.Repeat("=", 72))

	totalDiperiksa := 0
	totalGagal := 0
	var semuaGagal []Ketidakcocokan

	for _, e := range entri {
		if e.IsDir() || !strings.HasSuffix(e.Name(), ".tsv") {
			continue
		}
		berkas, err := MuatBerkas(filepath.Join(dir, e.Name()))
		if err != nil {
			fmt.Fprintf(os.Stderr, "GAGAL: %v\n", err)
			os.Exit(1)
		}
		hasil, err := PeriksaBerkas(berkas)
		if err != nil {
			fmt.Fprintf(os.Stderr, "GAGAL: %v\n", err)
			os.Exit(1)
		}

		tanda := "ok"
		if len(hasil.Gagal) > 0 {
			tanda = "GAGAL"
		}
		fmt.Printf("%-28s %6d diperiksa  %-16s %s\n",
			hasil.Berkas, hasil.Diperiksa, hasil.Keterbandingan, tanda)

		totalDiperiksa += hasil.Diperiksa
		totalGagal += len(hasil.Gagal)
		semuaGagal = append(semuaGagal, hasil.Gagal...)
	}

	fmt.Println(strings.Repeat("=", 72))
	if totalGagal == 0 {
		fmt.Printf("Seluruh %d vektor cocok antara Rust dan Go.\n", totalDiperiksa)
		return
	}

	fmt.Printf("%d dari %d vektor tidak cocok:\n\n", totalGagal, totalDiperiksa)
	// Hanya dua puluh pertama yang ditampilkan; sisanya hanya mengulang pola.
	batas := len(semuaGagal)
	if batas > 20 {
		batas = 20
	}
	for _, m := range semuaGagal[:batas] {
		fmt.Println(m)
	}
	if len(semuaGagal) > batas {
		fmt.Printf("\n… dan %d ketidakcocokan lainnya.\n", len(semuaGagal)-batas)
	}
	os.Exit(1)
}
