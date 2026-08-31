/**
 * Katalog laboratorium.
 *
 * Setiap sesi kuliah IND323 punya satu entri.
 *
 * # Kenapa mesinnya diimpor belakangan
 *
 * Keterangan laboratorium — judul, nomor sesi, penjelasan singkat — dibutuhkan
 * sejak halaman pertama dibuka, karena daftar isinya memuat kedua belasnya.
 * Mesinnya tidak: pengunjung yang membuka satu laboratorium tidak punya alasan
 * mengunduh sebelas yang lain.
 *
 * Karena itu keterangannya tinggal di berkas ini, sedangkan mesinnya dimuat
 * lewat `import()` saat laboratoriumnya benar-benar dibuka. Vite memecah tiap
 * `import()` menjadi berkas tersendiri, sehingga pembagiannya terjadi
 * sungguhan dan bukan sekadar penataan kode.
 *
 * Akibatnya berkas ini menjadi satu-satunya tempat yang tahu ada laboratorium
 * apa saja. Itu justru diinginkan: menambah laboratorium berarti menyunting
 * satu berkas, dan tidak ada jalan untuk menambahkannya setengah-setengah.
 *
 * .Deckyx
 */

import type { Bilingual } from "../i18n.js";
import { bi } from "../i18n.js";

/**
 * Isi sebuah modul laboratorium.
 *
 * `mount` mengembalikan fungsi pembersih yang dipanggil saat pengguna
 * berpindah halaman. Laboratorium yang menjalankan animasi atau pendengar
 * peristiwa wajib menghentikannya di sana; tanpa itu, berpindah bolak-balik
 * meninggalkan gelang animasi yang terus berjalan tanpa terlihat.
 */
export interface LabModule {
  mount: (root: HTMLElement) => () => void;
}

/** Satu laboratorium beserta cara memuat mesinnya. */
export interface Lab {
  /** Bagian URL setelah tanda pagar, mis. `#/certainty-factor`. */
  slug: string;
  /** Nomor sesi pada silabus IND323. */
  session: number;
  /** Judul laboratorium. */
  title: Bilingual;
  /** Penjelasan singkat yang muncul di bawah judul. */
  blurb: Bilingual;
  /** Memuat modul mesinnya. */
  load: () => Promise<LabModule>;
}

/** Sesi yang sudah bisa dijalankan. */
export const LABS: Lab[] = [
  {
    slug: "eliza",
    session: 1,
    title: bi("ELIZA & Uji Turing", "ELIZA & the Turing Test"),
    blurb: bi(
      "Program percakapan pertama yang membuat orang percaya sedang dipahami, dengan mesinnya dibiarkan terbuka. Setiap balasan disertai aturan mana yang menang dan bagian mana dari kalimat Anda yang sekadar dipantulkan kembali.",
      "The first conversational program that made people feel understood, with its machinery left open. Every reply shows which rule won and which part of your sentence was merely bounced back.",
    ),
    load: () => import("./eliza.js"),
  },
  {
    slug: "agents",
    session: 2,
    title: bi("Agen Cerdas & Ruang Keadaan", "Agents & State Space"),
    blurb: bi(
      "Empat jenis agen pada dunia yang sama. Yang membedakannya bukan kecanggihan, melainkan seberapa banyak yang mereka ingat: agen tanpa ingatan tidak punya cara mengetahui bahwa pekerjaannya sudah selesai, jadi ia terus bergerak sampai dihentikan paksa.",
      "Four kinds of agent on one world. What separates them is not sophistication but how much they remember: an agent without memory has no way to know its work is done, so it keeps moving until forced to stop.",
    ),
    load: () => import("./agent.js"),
  },
  {
    slug: "certainty-factor",
    session: 3,
    title: bi("Certainty Factor", "Certainty Factor"),
    blurb: bi(
      "Cara MYCIN menakar keyakinan ketika buktinya tidak pasti. Setiap bukti punya ukuran kepercayaan (MB) dan ketidakpercayaan (MD); CF adalah selisihnya, lalu bukti-bukti digabungkan satu per satu.",
      "How MYCIN weighs belief when the evidence is uncertain. Each piece of evidence carries a measure of belief (MB) and disbelief (MD); CF is their difference, and pieces are then combined one at a time.",
    ),
    load: () => import("./certainty.js"),
  },
  {
    slug: "bayesian",
    session: 4,
    title: bi("Probabilitas Bayesian", "Bayesian Probability"),
    blurb: bi(
      "Teorema Bayes membalik arah pertanyaan: dari “seberapa sering gejala muncul pada yang sakit” menjadi “seberapa mungkin sakit bila gejalanya muncul”. Geser penggesernya dan perhatikan betapa jauh hasilnya dari tebakan intuitif.",
      "Bayes' theorem reverses the question: from “how often does the symptom appear in the ill” to “how likely is illness given the symptom”. Move the sliders and watch how far the answer drifts from intuition.",
    ),
    load: () => import("./bayes.js"),
  },
  {
    slug: "fuzzy-logic",
    session: 5,
    title: bi("Logika Fuzzy", "Fuzzy Logic"),
    blurb: bi(
      "Keanggotaan bertingkat menggantikan benar-salah. Susun himpunan kaburnya sendiri, tulis aturannya, lalu bandingkan tiga cara penegasan yang bisa memberi jawaban berbeda dari aturan yang sama persis.",
      "Graded membership replaces true and false. Build your own fuzzy sets, write the rules, then compare three defuzzification methods that can give different answers from exactly the same rules.",
    ),
    load: () => import("./fuzzy.js"),
  },
  {
    slug: "search",
    session: 8,
    title: bi("Pencarian & Heuristik", "Search & Heuristics"),
    blurb: bi(
      "Sembilan algoritma penelusuran pada peta yang bisa Anda gambar sendiri. Yang layak diperhatikan bukan siapa yang menemukan jalan, melainkan berapa banyak petak yang harus mereka periksa untuk itu.",
      "Nine search algorithms on a map you draw yourself. What matters is not who finds a path but how many cells each had to examine to do it.",
    ),
    load: () => import("./search.js"),
  },
  {
    slug: "neural-network",
    session: 9,
    title: bi("Jaringan Syaraf Tiruan", "Neural Networks"),
    blurb: bi(
      "Perceptron dan perambatan balik, dilatih di depan mata. Naikkan laju belajarnya sedikit demi sedikit dan perhatikan titik ketika pelatihannya berhenti menurun lalu mulai melompat-lompat.",
      "Perceptrons and backpropagation, trained in front of you. Raise the learning rate step by step and watch the point where training stops descending and starts bouncing.",
    ),
    load: () => import("./neural.js"),
  },
  {
    slug: "expert-system",
    session: 11,
    title: bi("Sistem Pakar", "Expert Systems"),
    blurb: bi(
      "Basis pengetahuan “Dokter Virtual” dari studi kasus modul, dijalankan dua arah. Runut maju bertanya “apa yang bisa disimpulkan”; runut mundur bertanya “benarkah dugaan ini, dan gejala mana yang masih perlu saya tanyakan”.",
      "The “Virtual Doctor” knowledge base from the course case study, run in both directions. Forward chaining asks “what follows”; backward chaining asks “is this hypothesis true, and which symptoms do I still need to ask about”.",
    ),
    load: () => import("./expert.js"),
  },
  {
    slug: "machine-learning",
    session: 13,
    title: bi("Pembelajaran Mesin", "Machine Learning"),
    blurb: bi(
      "kNN, k-means, pohon keputusan, dan regresi pada data yang bisa Anda geser sendiri. Setiap model dibandingkan dengan tebakan kelas terbanyak, karena ketepatan yang tidak melampaui tebakan bukanlah pembelajaran.",
      "kNN, k-means, decision trees, and regression on data you can drag yourself. Every model is compared against guessing the majority class, because accuracy that does not beat guessing is not learning.",
    ),
    load: () => import("./ml.js"),
  },
  {
    slug: "nlp",
    session: 10,
    title: bi("Pemrosesan Bahasa Alami", "Natural Language Processing"),
    blurb: bi(
      "Teks Bahasa Indonesia diproses tahap demi tahap, termasuk pencarian kata dasar yang harus menangani awalan peluluh — “menyapu” berasal dari “sapu”, bukan “nyapu”, dan tidak ada algoritma Bahasa Inggris yang tahu itu.",
      "Indonesian text processed stage by stage, including a stemmer that must handle dissolving prefixes — “menyapu” comes from “sapu”, not “nyapu”, and no English algorithm knows that.",
    ),
    load: () => import("./nlp.js"),
  },
  {
    slug: "knowledge",
    session: 7,
    title: bi("Representasi Pengetahuan", "Knowledge Representation"),
    blurb: bi(
      "Tabel kebenaran menjawab “apakah benar” dengan mencoba semua kemungkinan; barisnya berlipat dua tiap proposisi ditambahkan. Resolusi menjawab pertanyaan yang sama dengan membuktikan — menyangkal kesimpulan lalu mencari kontradiksi. Bandingkan keduanya di sini pada kasus yang sama.",
      "A truth table answers “is it true” by trying every possibility; its rows double with each proposition. Resolution answers the same question by proving — negating the conclusion and hunting a contradiction. Compare the two here on the same cases.",
    ),
    load: () => import("./knowledge.js"),
  },
  {
    slug: "robotics",
    session: 14,
    title: bi("Robotika", "Robotics"),
    blurb: bi(
      "Kendali PID, kinematika lengan, dan perencanaan lintasan. Setel penguatannya sampai sistemnya berayun, lalu perhatikan bahwa yang membuatnya stabil bukan penguatan yang besar melainkan yang seimbang.",
      "PID control, arm kinematics, and path planning. Tune the gains until the system oscillates, then notice that what stabilises it is not large gain but balanced gain.",
    ),
    load: () => import("./robotics.js"),
  },
];

/**
 * Seluruh sesi silabus, termasuk yang dua sesinya berbagi satu laboratorium.
 *
 * Sesi 5 dan 6 sama-sama menunjuk ke laboratorium kabur, begitu pula sesi 12
 * dan 13 ke pembelajaran mesin. Menampilkannya apa adanya lebih jujur daripada
 * memaksakan satu sesi satu laboratorium.
 */
export const SYLLABUS: { session: number; title: Bilingual; slug?: string }[] = [
  { session: 1, title: LABS[0].title, slug: LABS[0].slug },
  { session: 2, title: LABS[1].title, slug: LABS[1].slug },
  { session: 3, title: LABS[2].title, slug: LABS[2].slug },
  { session: 4, title: LABS[3].title, slug: LABS[3].slug },
  { session: 5, title: bi("Logika Fuzzy I", "Fuzzy Logic I"), slug: "fuzzy-logic" },
  { session: 6, title: bi("Logika Fuzzy II", "Fuzzy Logic II"), slug: "fuzzy-logic" },
  { session: 7, title: bi("Representasi Pengetahuan", "Knowledge Representation"), slug: "knowledge" },
  { session: 8, title: bi("Pencarian & Heuristik", "Search & Heuristics"), slug: "search" },
  { session: 9, title: bi("Jaringan Syaraf Tiruan", "Neural Networks"), slug: "neural-network" },
  { session: 10, title: bi("Pemrosesan Bahasa Alami", "Natural Language Processing"), slug: "nlp" },
  { session: 11, title: bi("Sistem Pakar", "Expert Systems"), slug: "expert-system" },
  { session: 12, title: bi("Sains Data & Big Data", "Data Science & Big Data"), slug: "machine-learning" },
  { session: 13, title: bi("Pembelajaran Mesin", "Machine Learning"), slug: "machine-learning" },
  { session: 14, title: bi("Robotika", "Robotics"), slug: "robotics" },
];

/** Mencari laboratorium berdasarkan slug-nya. */
export function findLab(slug: string): Lab | undefined {
  return LABS.find((l) => l.slug === slug);
}
