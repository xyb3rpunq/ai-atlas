// Package aicore adalah implementasi pembanding algoritma AI ATLAS dalam Go.
//
// Tujuannya bukan dipakai di produksi, melainkan membuktikan bahwa implementasi
// Rust benar. Dua implementasi yang ditulis terpisah dan menghasilkan angka
// yang sama memberi keyakinan yang tidak bisa diberikan uji mana pun terhadap
// satu implementasi: sebuah rumus yang salah tetap konsisten dengan dirinya
// sendiri, tetapi hampir mustahil salah dengan cara yang sama dua kali.
//
// .Deckyx
package aicore

import (
	"fmt"
	"math"
	"strconv"
	"strings"
)

// PanjangHex adalah jumlah digit heksadesimal sebuah float64.
const PanjangHex = 16

// KeHex mengubah float64 menjadi 16 digit heksadesimal huruf kecil.
func KeHex(v float64) string {
	return fmt.Sprintf("%016x", math.Float64bits(v))
}

// DariHex membaca float64 dari 16 digit heksadesimal.
//
// Bentuk ini dipakai, bukan desimal, karena desimal tidak selamat melintasi
// batas bahasa: pengukuran pada sisi Rust menemukan parser JSON-nya salah
// membulat 1 ULP pada 13,8 persen nilai uji. Pola bit tidak punya ruang tafsir.
func DariHex(s string) (float64, error) {
	t := strings.TrimSpace(s)
	if len(t) != PanjangHex {
		return 0, fmt.Errorf("panjang harus %d digit, diberi %d", PanjangHex, len(t))
	}
	bits, err := strconv.ParseUint(t, 16, 64)
	if err != nil {
		return 0, fmt.Errorf("bukan heksadesimal yang sah: %q", t)
	}
	return math.Float64frombits(bits), nil
}

// JarakUlp mengembalikan jarak dua float64 dalam satuan ULP.
//
// Mengembalikan -1 bila jaraknya tidak terdefinisi, yaitu ketika salah satu
// nilai bukan bilangan, atau ketika keduanya tak hingga dengan tanda berbeda.
func JarakUlp(a, b float64) int64 {
	if math.IsNaN(a) || math.IsNaN(b) {
		// Dua nilai bukan bilangan dianggap sepadan; pembandingnya memutuskan.
		if math.IsNaN(a) && math.IsNaN(b) {
			return 0
		}
		return -1
	}
	if a == b {
		return 0
	}
	if math.IsInf(a, 0) || math.IsInf(b, 0) {
		return -1
	}

	// Pola bit dipetakan ke bilangan bulat bertanda yang terurut monoton,
	// sehingga selisihnya langsung menyatakan berapa banyak float64 yang ada
	// di antara keduanya.
	kunci := func(v float64) int64 {
		bits := int64(math.Float64bits(v))
		if bits < 0 {
			return math.MinInt64 - bits
		}
		return bits
	}
	d := kunci(a) - kunci(b)
	if d < 0 {
		d = -d
	}
	return d
}

// LangkahUlp adalah jarak antara x dan float64 terdekat berikutnya yang lebih
// besar nilai mutlaknya.
//
// Dipakai untuk menyatakan toleransi pada skala tempat aritmetikanya terjadi,
// bukan pada hasil akhirnya. Sebuah selisih dua besaran yang hampir sama
// memperbesar galat: dua ULP pada besaran 0,94 sama dengan 64 ULP pada hasil
// 0,029, padahal tidak ada perhitungan yang lebih buruk di antaranya.
//
// Mengembalikan NaN untuk masukan yang tidak berhingga. Untuk nol dipakai
// bilangan subnormal terkecil, yaitu langkah sesungguhnya dari nol.
func LangkahUlp(x float64) float64 {
	if math.IsNaN(x) || math.IsInf(x, 0) {
		return math.NaN()
	}
	a := math.Abs(x)
	if a == 0 {
		return math.Float64frombits(1)
	}
	return math.Float64frombits(math.Float64bits(a)+1) - a
}

// SamaBit memeriksa kesetaraan pada tingkat pola bit, dengan NaN dianggap sama.
//
// Perbandingan == biasa menyatakan NaN != NaN, padahal untuk mengadu dua
// implementasi kita justru ingin "sama-sama menghasilkan NaN" dinilai lolos.
func SamaBit(a, b float64) bool {
	if math.IsNaN(a) && math.IsNaN(b) {
		return true
	}
	return math.Float64bits(a) == math.Float64bits(b)
}
