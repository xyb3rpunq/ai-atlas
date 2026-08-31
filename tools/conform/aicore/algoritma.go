package aicore

import (
	"math"
	"sort"
)

// ---------------------------------------------------------------------------
// SplitMix64
// ---------------------------------------------------------------------------

// GoldenGamma adalah konstanta penambah SplitMix64.
const GoldenGamma uint64 = 0x9E3779B97F4A7C15

// SplitMix64 adalah pembangkit bilangan acak berstate 64 bit.
//
// Bagian ini yang paling penting dicocokkan: seluruh reproduktifitas di proyek
// ini bergantung padanya. Kalau Go dan Rust menghasilkan deret berbeda, maka
// setiap perbandingan yang melibatkan keacakan — bobot awal jaringan, labirin,
// pembagian data latih — kehilangan maknanya.
type SplitMix64 struct {
	state uint64
}

// BaruSplitMix64 membuat pembangkit dari sebuah benih.
func BaruSplitMix64(seed uint64) *SplitMix64 {
	return &SplitMix64{state: seed}
}

// NextU64 mengambil bilangan bulat berikutnya dan memajukan state.
//
// Seluruh perhitungannya memakai aritmetika bilangan bulat 64 bit yang
// melimpah, yang di Go memang melimpah secara diam-diam — perilaku yang sama
// dengan `wrapping_add` dan `wrapping_mul` di Rust.
func (r *SplitMix64) NextU64() uint64 {
	r.state += GoldenGamma
	z := r.state
	z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9
	z = (z ^ (z >> 27)) * 0x94D049BB133111EB
	return z ^ (z >> 31)
}

// NextF64 mengambil pecahan seragam pada rentang [0, 1).
//
// Memakai 53 bit teratas agar setiap nilai float64 yang mungkin punya peluang
// sama, sama seperti sisi Rust.
func (r *SplitMix64) NextF64() float64 {
	return float64(r.NextU64()>>11) * (1.0 / 9007199254740992.0)
}

// ---------------------------------------------------------------------------
// Certainty factor
// ---------------------------------------------------------------------------

// CfDariMbMd menghitung certainty factor dari ukuran kepercayaan dan
// ketidakpercayaan.
func CfDariMbMd(mb, md float64) float64 {
	return mb - md
}

// GabungParalel menggabungkan dua certainty factor dari bukti berbeda untuk
// hipotesis yang sama.
func GabungParalel(a, b float64) float64 {
	var hasil float64
	switch {
	case a >= 0 && b >= 0:
		hasil = a + b*(1-a)
	case a <= 0 && b <= 0:
		hasil = a + b*(1+a)
	default:
		penyebut := 1 - math.Min(math.Abs(a), math.Abs(b))
		if math.Abs(penyebut) < 1e-9 {
			// Bukti berlawanan penuh saling meniadakan.
			hasil = 0
		} else {
			hasil = (a + b) / penyebut
		}
	}
	return jepit(hasil, -1, 1)
}

// GabungBerantai mengalikan keyakinan aturan dengan keyakinan buktinya.
//
// Bukti berkeyakinan negatif tidak menyalakan aturan, sehingga hasilnya nol.
func GabungBerantai(cfAturan, cfBukti float64) float64 {
	return jepit(cfAturan*math.Max(cfBukti, 0), -1, 1)
}

// GabungDan mengambil keyakinan terkecil di antara premis.
func GabungDan(nilai []float64) float64 {
	hasil := math.Inf(1)
	for _, v := range nilai {
		hasil = math.Min(hasil, v)
	}
	return hasil
}

// GabungAtau mengambil keyakinan terbesar di antara premis.
func GabungAtau(nilai []float64) float64 {
	hasil := math.Inf(-1)
	for _, v := range nilai {
		hasil = math.Max(hasil, v)
	}
	return hasil
}

func jepit(v, lo, hi float64) float64 {
	return math.Min(hi, math.Max(lo, v))
}

// ---------------------------------------------------------------------------
// Bayesian
// ---------------------------------------------------------------------------

// HasilBayes memuat besaran yang dihitung untuk kasus dua hipotesis.
type HasilBayes struct {
	Bukti            float64
	Posterior        float64
	RasioKemungkinan float64
}

// BayesBiner menghitung posterior untuk kasus dua hipotesis.
func BayesBiner(prior, likelihoodH, likelihoodBukanH float64) HasilBayes {
	priorBukanH := 1 - prior
	bukti := prior*likelihoodH + priorBukanH*likelihoodBukanH

	posterior := 0.0
	if bukti >= 1e-12 {
		posterior = jepit(likelihoodH*prior/bukti, 0, 1)
	}

	rasio := 0.0
	switch {
	case likelihoodBukanH < 1e-12 && likelihoodH < 1e-12:
		rasio = 0
	case likelihoodBukanH < 1e-12:
		rasio = math.Inf(1)
	default:
		rasio = likelihoodH / likelihoodBukanH
	}

	return HasilBayes{Bukti: bukti, Posterior: posterior, RasioKemungkinan: rasio}
}

// ---------------------------------------------------------------------------
// Keanggotaan fuzzy
// ---------------------------------------------------------------------------

const epsFuzzy = 1e-9

// KeanggotaanSegitiga menghitung derajat keanggotaan pada fungsi segitiga.
//
// Puncak diperiksa lebih dulu. Kalau tidak, segitiga berkaki berimpit akan
// menghasilkan nol tepat di puncaknya — bentuk yang justru lazim dipakai di
// tepi semesta pembicaraan.
func KeanggotaanSegitiga(a, b, c, x float64) float64 {
	var v float64
	switch {
	case math.Abs(x-b) < epsFuzzy:
		v = 1
	case x <= a || x >= c:
		v = 0
	case x < b:
		if math.Abs(b-a) < epsFuzzy {
			v = 1
		} else {
			v = (x - a) / (b - a)
		}
	default:
		if math.Abs(c-b) < epsFuzzy {
			v = 1
		} else {
			v = (c - x) / (c - b)
		}
	}
	if math.IsNaN(v) {
		return 0
	}
	return jepit(v, 0, 1)
}

// KeanggotaanTrapesium menghitung derajat keanggotaan pada fungsi trapesium.
func KeanggotaanTrapesium(a, b, c, d, x float64) float64 {
	var v float64
	switch {
	case x >= b && x <= c:
		v = 1
	case x <= a || x >= d:
		v = 0
	case x < b:
		if math.Abs(b-a) < epsFuzzy {
			v = 1
		} else {
			v = (x - a) / (b - a)
		}
	default:
		if math.Abs(d-c) < epsFuzzy {
			v = 1
		} else {
			v = (d - x) / (d - c)
		}
	}
	if math.IsNaN(v) {
		return 0
	}
	return jepit(v, 0, 1)
}

// KeanggotaanGauss menghitung derajat keanggotaan pada kurva Gauss.
func KeanggotaanGauss(mean, sigma, x float64) float64 {
	s := math.Abs(sigma)
	if s < epsFuzzy {
		s = epsFuzzy
	}
	z := (x - mean) / s
	v := math.Exp(-0.5 * z * z)
	if math.IsNaN(v) {
		return 0
	}
	return jepit(v, 0, 1)
}

// KeanggotaanSigmoid menghitung derajat keanggotaan pada kurva sigmoid.
//
// Bentuk yang dipakai stabil untuk masukan besar: memakai eksponen pada nilai
// bertanda sama agar tidak meluap.
func KeanggotaanSigmoid(a, c, x float64) float64 {
	z := a * (x - c)
	var v float64
	if z >= 0 {
		v = 1 / (1 + math.Exp(-z))
	} else {
		e := math.Exp(z)
		v = e / (1 + e)
	}
	if math.IsNaN(v) {
		return 0
	}
	return jepit(v, 0, 1)
}

// ---------------------------------------------------------------------------
// Jarak, entropi, ketakmurnian
// ---------------------------------------------------------------------------

// JarakEuclidean menghitung jarak lurus antara dua titik.
func JarakEuclidean(a, b []float64) float64 {
	total := 0.0
	for i := range a {
		d := a[i] - b[i]
		total += d * d
	}
	return math.Sqrt(total)
}

// JarakManhattan menjumlahkan selisih tiap sumbu.
func JarakManhattan(a, b []float64) float64 {
	total := 0.0
	for i := range a {
		total += math.Abs(a[i] - b[i])
	}
	return total
}

// JarakChebyshev mengambil selisih terbesar di antara sumbu.
func JarakChebyshev(a, b []float64) float64 {
	terbesar := 0.0
	for i := range a {
		terbesar = math.Max(terbesar, math.Abs(a[i]-b[i]))
	}
	return terbesar
}

// hitungLabel menghitung kemunculan tiap label, dikembalikan terurut menurut
// namanya supaya penjumlahannya berurutan sama dengan sisi Rust.
//
// Urutan penjumlahan penting: penjumlahan pecahan tidak asosiatif, sehingga
// menjumlahkan nilai yang sama dalam urutan berbeda bisa menghasilkan angka
// yang berbeda pada digit terakhir.
func hitungLabel(labels []string) ([]string, map[string]int) {
	jumlah := make(map[string]int)
	for _, l := range labels {
		jumlah[l]++
	}
	nama := make([]string, 0, len(jumlah))
	for k := range jumlah {
		nama = append(nama, k)
	}
	sort.Strings(nama)
	return nama, jumlah
}

// Entropi menghitung entropi Shannon sebuah sebaran label, dalam bit.
func Entropi(labels []string) float64 {
	if len(labels) == 0 {
		return 0
	}
	n := float64(len(labels))
	nama, jumlah := hitungLabel(labels)
	total := 0.0
	for _, k := range nama {
		p := float64(jumlah[k]) / n
		total += p * math.Log2(p)
	}
	return -total
}

// Gini menghitung ketakmurnian Gini sebuah sebaran label.
func Gini(labels []string) float64 {
	if len(labels) == 0 {
		return 0
	}
	n := float64(len(labels))
	nama, jumlah := hitungLabel(labels)
	total := 0.0
	for _, k := range nama {
		p := float64(jumlah[k]) / n
		total += p * p
	}
	return 1 - total
}

// PerolehanInformasi menghitung berkurangnya entropi bila data dipecah menurut
// sebuah atribut.
func PerolehanInformasi(values, labels []string) float64 {
	if len(values) != len(labels) || len(labels) == 0 {
		return 0
	}
	sebelum := Entropi(labels)
	n := float64(len(labels))

	kelompok := make(map[string][]string)
	for i, v := range values {
		kelompok[v] = append(kelompok[v], labels[i])
	}
	nama := make([]string, 0, len(kelompok))
	for k := range kelompok {
		nama = append(nama, k)
	}
	sort.Strings(nama)

	sesudah := 0.0
	for _, k := range nama {
		g := kelompok[k]
		sesudah += (float64(len(g)) / n) * Entropi(g)
	}
	return sebelum - sesudah
}
