/**
 * Catatan dan definisi untuk tiap laboratorium.
 *
 * Dipisahkan dari berkas laboratoriumnya dengan sengaja. Definisi adalah bahan
 * yang paling sering perlu dibaca ulang, dibandingkan antarsesi, dan diperiksa
 * ketepatannya — jauh lebih mudah bila seluruhnya berada di satu tempat
 * daripada tersebar di empat belas berkas antarmuka.
 *
 * Tiap catatan memuat tiga bagian:
 *
 * - **Definisi** — istilah beserta artinya, dalam kalimat yang bisa dipakai
 *   menjawab soal, bukan parafrase kabur.
 * - **Rumus** — bentuk matematisnya, disertai keterangan kapan ia berlaku dan
 *   kapan tidak. Rumus tanpa syarat berlakunya adalah jebakan.
 * - **Rujukan** — sumber aslinya, supaya bisa ditelusuri sendiri.
 */

import type { Bilingual } from "../i18n.js";
import { bi } from "../i18n.js";

/** Satu istilah beserta artinya. */
export interface Definition {
  term: Bilingual;
  meaning: Bilingual;
}

/** Satu rumus beserta syarat berlakunya. */
export interface Formula {
  name: Bilingual;
  /**
   * Rumusnya sendiri, tidak pernah diterjemahkan.
   *
   * Sebuah substitusi angka bukan kalimat, dan lambang yang berubah bentuk
   * antarbahasa memutus hubungannya dengan gambar di sebelahnya. Karena itu
   * ekspresinya ditulis dengan lambang yang sama di kedua bahasa — `∧`, `∨`,
   * `⟺`, `argmax` — bukan dengan "dan", "atau", atau "bila". Ada uji yang
   * menolak kata Indonesia yang menyelinap ke sini.
   */
  expression: string;
  note: Bilingual;
}

/** Satu rujukan pustaka. */
export interface Reference {
  text: string;
  url?: string;
}

/** Catatan lengkap sebuah laboratorium. */
export interface LabNotes {
  /** Ringkasan satu paragraf tentang apa yang sebenarnya dihitung di sini. */
  summary: Bilingual;
  definitions: Definition[];
  formulas: Formula[];
  /** Kesalahpahaman yang paling sering muncul pada topik ini. */
  pitfalls: Bilingual[];
  references: Reference[];
}

/** Catatan tiap laboratorium, dikunci dengan slug-nya. */
export const NOTES: Record<string, LabNotes> = {
  eliza: {
    summary: bi(
      "ELIZA mencocokkan kata kunci di dalam kalimat Anda, memilih aturan berkeutamaan tertinggi, menukar kata gantinya, lalu memasangkan sisa kalimat ke dalam templat. Tidak ada representasi makna, tidak ada model dunia, dan tidak ada ingatan antargiliran.",
      "ELIZA matches keywords inside your sentence, picks the highest-priority rule, swaps the pronouns, and slots the remaining fragment into a template. There is no meaning representation, no world model, and no memory between turns.",
    ),
    definitions: [
      {
        term: bi("Uji Turing", "The Turing test"),
        meaning: bi(
          "Usulan Alan Turing (1950) untuk mengganti pertanyaan “dapatkah mesin berpikir” dengan pertanyaan yang bisa diuji: dapatkah penilai manusia membedakan jawaban mesin dari jawaban manusia lewat percakapan tertulis. Perhatikan bahwa yang diuji adalah kemampuan menyerupai, bukan kemampuan berpikir.",
          "Alan Turing's 1950 proposal to replace “can machines think” with a testable question: can a human judge distinguish a machine's written answers from a person's. Note that what is tested is the ability to resemble, not the ability to think.",
        ),
      },
      {
        term: bi("Efek ELIZA", "The ELIZA effect"),
        meaning: bi(
          "Kecenderungan manusia memberi makna, pemahaman, dan bahkan empati kepada program yang sebenarnya hanya memanipulasi lambang. Dinamai dari reaksi pengguna ELIZA yang tetap merasa dipahami setelah diberi tahu cara kerjanya.",
          "The human tendency to attribute meaning, understanding, and even empathy to a program that merely manipulates symbols. Named after ELIZA's users, who kept feeling understood even after being told how it worked.",
        ),
      },
      {
        term: bi("Pencocokan pola", "Pattern matching"),
        meaning: bi(
          "Menemukan bentuk tertentu di dalam masukan tanpa menafsirkan artinya. ELIZA mencari kata kunci sebagai kata utuh, bukan sebagai potongan huruf, agar “ya” tidak ditemukan di dalam “budaya”.",
          "Finding a given shape inside the input without interpreting its meaning. ELIZA looks for keywords as whole words rather than letter fragments, so that “no” is not found inside “nothing”.",
        ),
      },
      {
        term: bi("Keutamaan aturan", "Rule priority"),
        meaning: bi(
          "Nilai yang menentukan aturan mana yang menang ketika beberapa kata kunci cocok sekaligus. Tanpa penomoran ini, aturan paling umum selalu menang dan percakapannya langsung terasa hambar.",
          "A number deciding which rule wins when several keywords match at once. Without it, the most general rule always wins and the conversation immediately falls flat.",
        ),
      },
    ],
    formulas: [
      {
        name: bi("Pemilihan aturan", "Rule selection"),
        expression: "rule = argmax( priority )  over  { r : keyword(r) ⊂ input }",
        note: bi(
          "Seri diputus oleh urutan penulisan, sehingga naskah bisa disusun dengan sengaja.",
          "Ties are broken by declaration order, so the script can be arranged deliberately.",
        ),
      },
    ],
    pitfalls: [
      bi(
        "Menganggap ELIZA “memahami” karena jawabannya terasa tepat. Yang terjadi adalah kalimat Anda dipantulkan kembali; kesan mendalam justru paling sering datang dari kalimat cadangan yang sama sekali tidak bergantung pada apa yang Anda tulis.",
        "Assuming ELIZA “understands” because its answers feel apt. What happens is your sentence being reflected back; the sense of depth most often comes from stock lines that do not depend on what you wrote at all.",
      ),
      bi(
        "Menyimpulkan dari uji Turing bahwa mesin yang lulus pasti cerdas. Yang diukur adalah ketidakmampuan penilai membedakan, dan itu bisa dicapai dengan menipu, bukan dengan berpikir.",
        "Concluding from the Turing test that a machine which passes must be intelligent. What is measured is the judge's inability to tell them apart, and that can be achieved by deceiving rather than by thinking.",
      ),
    ],
    references: [
      {
        text: "Weizenbaum, J. (1966). ELIZA — a computer program for the study of natural language communication between man and machine. Communications of the ACM, 9(1), 36–45.",
        url: "https://doi.org/10.1145/365153.365168",
      },
      {
        text: "Turing, A. M. (1950). Computing Machinery and Intelligence. Mind, LIX(236), 433–460.",
        url: "https://doi.org/10.1093/mind/LIX.236.433",
      },
    ],
  },

  agents: {
    summary: bi(
      "Sebuah agen memetakan barisan persepsi menjadi tindakan. Yang membedakan keempat jenis di sini bukan kecanggihan algoritmanya, melainkan seberapa banyak yang mereka simpan: agen tanpa ingatan tidak punya cara mengetahui bahwa pekerjaannya selesai.",
      "An agent maps a sequence of percepts to actions. What separates the four kinds here is not algorithmic sophistication but how much they retain: an agent with no memory has no way to know its work is done.",
    ),
    definitions: [
      {
        term: bi("Agen", "Agent"),
        meaning: bi(
          "Apa pun yang mempersepsi lingkungannya lewat sensor dan bertindak atasnya lewat aktuator. Definisi ini sengaja luas: termonstat pun termasuk agen.",
          "Anything that perceives its environment through sensors and acts upon it through actuators. The definition is deliberately broad: a thermostat qualifies.",
        ),
      },
      {
        term: bi("PEAS", "PEAS"),
        meaning: bi(
          "Kerangka merumuskan masalah agen: Performance measure (ukuran keberhasilan), Environment (lingkungan), Actuators (aktuator), Sensors (sensor). Merumuskan PEAS dengan benar menentukan agen macam apa yang diperlukan.",
          "A framework for stating an agent problem: Performance measure, Environment, Actuators, Sensors. Getting PEAS right determines what kind of agent is needed at all.",
        ),
      },
      {
        term: bi("Agen refleks sederhana", "Simple reflex agent"),
        meaning: bi(
          "Memilih tindakan hanya dari persepsi saat ini, memakai aturan kondisi-aksi. Cepat dan murah, tetapi buta terhadap apa pun yang tidak sedang terlihat.",
          "Chooses actions from the current percept alone, using condition-action rules. Fast and cheap, but blind to anything not currently visible.",
        ),
      },
      {
        term: bi("Agen berbasis model", "Model-based agent"),
        meaning: bi(
          "Menyimpan keadaan internal yang mencatat bagian dunia yang tidak sedang terlihat. Inilah yang memungkinkannya menyimpulkan bahwa seluruh ruangan sudah bersih.",
          "Maintains internal state recording the parts of the world not currently visible. This is what lets it conclude that every room is already clean.",
        ),
      },
      {
        term: bi("Agen berbasis tujuan", "Goal-based agent"),
        meaning: bi(
          "Menyimpan keadaan yang ingin dicapai, lalu memilih tindakan yang mendekatkannya. Memerlukan perencanaan, bukan sekadar reaksi.",
          "Holds a description of the state it wants to reach and chooses actions that bring it closer. This requires planning, not merely reacting.",
        ),
      },
      {
        term: bi("Agen berbasis utilitas", "Utility-based agent"),
        meaning: bi(
          "Menimbang seberapa diinginkannya tiap keadaan, bukan sekadar tercapai atau tidak. Ini satu-satunya jenis yang boleh memutuskan bahwa sebuah tujuan tidak sepadan diperjuangkan.",
          "Weighs how desirable each state is, not merely whether it is reached. This is the only kind that may decide a goal is not worth pursuing.",
        ),
      },
      {
        term: bi("Ruang keadaan", "State space"),
        meaning: bi(
          "Himpunan seluruh keadaan yang bisa dicapai, beserta tindakan yang menghubungkannya. Merumuskan masalah sebagai ruang keadaan sering kali sudah separuh pemecahannya.",
          "The set of all reachable states together with the actions linking them. Framing a problem as a state space is often half of solving it.",
        ),
      },
    ],
    formulas: [
      {
        name: bi("Keterjangkauan teko air", "Water-jug reachability"),
        expression: "reachable(t)  ⟺  t mod gcd(a, b) = 0  ∧  t ≤ max(a, b)",
        note: bi(
          "Akibat teorema Bézout. Memeriksanya di muka membedakan “mustahil” dari “tidak ketemu”, dua hal yang terlihat sama di layar.",
          "A consequence of Bézout's theorem. Checking it up front distinguishes “impossible” from “not found”, two things that look identical on screen.",
        ),
      },
      {
        name: bi("Keamanan misionaris", "Missionary safety"),
        expression: "safe  ⟺  (M_L = 0 ∨ M_L ≥ C_L)  ∧  (M_R = 0 ∨ M_R ≥ C_R)",
        note: bi(
          "Aturannya bukan “kanibal selalu lebih sedikit”. Tepi tanpa misionaris selalu aman berapa pun kanibalnya, dan perbedaan itu menentukan ada tidaknya penyelesaian.",
          "The rule is not “cannibals must always be fewer”. A bank with no missionaries is always safe however many cannibals stand there, and that distinction decides whether a solution exists.",
        ),
      },
    ],
    pitfalls: [
      bi(
        "Mengira agen refleks “gagal” karena terus bergerak. Ia tidak gagal — ia memang tidak punya cara mengetahui kapan berhenti, dan itulah alasan agen berbasis model ada.",
        "Thinking a reflex agent “fails” because it keeps moving. It does not fail — it simply has no way to know when to stop, and that is why model-based agents exist.",
      ),
      bi(
        "Mengukur pemborosan dari berkurangnya kotoran. Langkah menuju kotoran tidak mengurangi kotoran, tetapi jelas bukan pemborosan.",
        "Measuring waste by dirt removed. A step toward dirt removes none, yet is clearly not waste.",
      ),
    ],
    references: [
      {
        text: "Russell, S. & Norvig, P. (2021). Artificial Intelligence: A Modern Approach, 4th ed., Ch. 2: Intelligent Agents.",
      },
    ],
  },

  "certainty-factor": {
    summary: bi(
      "Certainty factor menakar keyakinan ketika buktinya tidak lengkap. Angkanya bukan probabilitas dan tidak mengikuti aksioma probabilitas — ia ukuran heuristik yang dirancang agar pakar bisa menyatakan keyakinannya dalam angka.",
      "Certainty factors quantify belief when evidence is incomplete. The numbers are not probabilities and do not obey the probability axioms — they are a heuristic measure designed so that an expert can state confidence numerically.",
    ),
    definitions: [
      {
        term: bi("MB (Measure of Belief)", "MB (measure of belief)"),
        meaning: bi(
          "Seberapa kuat sebuah bukti mendukung hipotesis, bernilai 0 sampai 1. Nilai nol berarti bukti itu tidak mendukung sama sekali, bukan berarti menentang.",
          "How strongly a piece of evidence supports a hypothesis, from 0 to 1. Zero means the evidence lends no support, not that it opposes.",
        ),
      },
      {
        term: bi("MD (Measure of Disbelief)", "MD (measure of disbelief)"),
        meaning: bi(
          "Seberapa kuat sebuah bukti menentang hipotesis, bernilai 0 sampai 1. MB dan MD diukur terpisah karena bukti bisa sekaligus mendukung sebagian dan menentang sebagian.",
          "How strongly a piece of evidence opposes a hypothesis, from 0 to 1. MB and MD are measured separately because evidence can partly support and partly oppose at once.",
        ),
      },
      {
        term: bi("CF (Certainty Factor)", "CF (certainty factor)"),
        meaning: bi(
          "Selisih MB dikurangi MD, bernilai −1 sampai +1. Nilai di sekitar nol berarti bukti yang ada belum memutuskan apa pun — berbeda dari “kemungkinannya lima puluh persen”.",
          "MB minus MD, ranging from −1 to +1. A value near zero means the evidence has decided nothing — which is different from “the probability is fifty percent”.",
        ),
      },
      {
        term: bi("Kombinasi paralel", "Parallel combination"),
        meaning: bi(
          "Menggabungkan dua CF yang datang dari bukti berbeda untuk hipotesis yang sama. Dua bukti yang sama-sama mendukung akan saling menguatkan, tetapi tidak pernah melampaui satu.",
          "Combining two CFs arising from different evidence for the same hypothesis. Two supporting pieces reinforce each other, but never exceed one.",
        ),
      },
      {
        term: bi("Kombinasi berantai", "Sequential combination"),
        meaning: bi(
          "Meneruskan keyakinan lewat rantai aturan: keyakinan kesimpulan adalah keyakinan aturan dikalikan keyakinan premisnya.",
          "Propagating belief along a chain of rules: the conclusion's certainty is the rule's certainty times the premise's certainty.",
        ),
      },
    ],
    formulas: [
      {
        name: bi("Certainty factor dasar", "Basic certainty factor"),
        expression: "CF = MB − MD",
        note: bi(
          "Berlaku untuk satu bukti terhadap satu hipotesis.",
          "Applies to one piece of evidence against one hypothesis.",
        ),
      },
      {
        name: bi("Kombinasi paralel", "Parallel combination"),
        expression:
          "CF₁ + CF₂(1 − CF₁)   [CF₁ ≥ 0 ∧ CF₂ ≥ 0]\n" +
          "CF₁ + CF₂(1 + CF₁)   [CF₁ ≤ 0 ∧ CF₂ ≤ 0]\n" +
          "(CF₁ + CF₂) / (1 − min(|CF₁|, |CF₂|))   [sgn CF₁ ≠ sgn CF₂]",
        note: bi(
          "Bersifat komutatif dan berelemen identitas nol, jadi urutan menggabungkan bukti tidak mengubah hasil. Kasus +1 melawan −1 diperlakukan sebagai nol karena penyebutnya menjadi nol.",
          "Commutative with zero as identity, so the order of combining evidence does not change the result. The +1 against −1 case is treated as zero because the denominator vanishes.",
        ),
      },
      {
        name: bi("Kombinasi berantai", "Sequential combination"),
        expression: "CF_conclusion = CF_rule × max( CF_premise, 0 )",
        note: bi(
          "Premis berkeyakinan negatif tidak menyalakan aturan sama sekali.",
          "A premise with negative certainty does not fire the rule at all.",
        ),
      },
      {
        name: bi("Premis majemuk", "Compound premises"),
        expression: "AND → min(CF₁, …, CFₙ)      OR → max(CF₁, …, CFₙ)",
        note: bi(
          "Mengikuti operator Zadeh, sama seperti pada logika kabur.",
          "Follows the Zadeh operators, the same as in fuzzy logic.",
        ),
      },
    ],
    pitfalls: [
      bi(
        "Memperlakukan CF sebagai probabilitas. CF tidak memenuhi aksioma probabilitas: CF sebuah hipotesis dan ingkarannya tidak berjumlah satu, dan kombinasinya tidak mengikuti teorema Bayes.",
        "Treating CF as a probability. CF does not satisfy the probability axioms: a hypothesis and its negation do not sum to one, and combination does not follow Bayes' theorem.",
      ),
      bi(
        "Membaca CF = 0 sebagai “kemungkinannya lima puluh persen”. Artinya adalah bukti yang ada belum memutuskan apa pun.",
        "Reading CF = 0 as “fifty percent likely”. It means the evidence has decided nothing.",
      ),
    ],
    references: [
      {
        text: "Shortliffe, E. H. & Buchanan, B. G. (1975). A model of inexact reasoning in medicine. Mathematical Biosciences, 23(3–4), 351–379.",
        url: "https://doi.org/10.1016/0025-5564(75)90047-4",
      },
    ],
  },

  bayesian: {
    summary: bi(
      "Teorema Bayes membalik arah sebuah peluang bersyarat. Ia mengubah “seberapa sering gejala muncul pada yang sakit” menjadi “seberapa mungkin sakit bila gejalanya muncul” — dua angka yang sangat berbeda dan sangat sering tertukar.",
      "Bayes' theorem reverses a conditional probability. It turns “how often the symptom appears in the ill” into “how likely illness is given the symptom” — two very different numbers that are very often confused.",
    ),
    definitions: [
      {
        term: bi("Prior — P(H)", "Prior — P(H)"),
        meaning: bi(
          "Peluang hipotesis sebelum bukti apa pun dilihat. Pada diagnosis, ini prevalensi penyakit di populasi. Mengabaikannya adalah sumber kekeliruan terbesar dalam penalaran diagnostik.",
          "The probability of the hypothesis before any evidence. In diagnosis this is the prevalence in the population. Ignoring it is the single largest source of diagnostic error.",
        ),
      },
      {
        term: bi("Likelihood — P(E|H)", "Likelihood — P(E|H)"),
        meaning: bi(
          "Peluang munculnya bukti bila hipotesisnya benar. Pada uji medis inilah yang disebut sensitivitas.",
          "The probability of seeing the evidence if the hypothesis holds. In medical testing this is sensitivity.",
        ),
      },
      {
        term: bi("Bukti — P(E)", "Evidence — P(E)"),
        meaning: bi(
          "Peluang munculnya bukti secara keseluruhan, dari hipotesis mana pun. Dihitung dengan hukum probabilitas total.",
          "The overall probability of the evidence, from whatever cause. Computed by the law of total probability.",
        ),
      },
      {
        term: bi("Posterior — P(H|E)", "Posterior — P(H|E)"),
        meaning: bi(
          "Peluang hipotesis setelah bukti diperhitungkan. Inilah yang biasanya ingin diketahui, dan yang paling sering keliru disamakan dengan likelihood.",
          "The probability of the hypothesis once the evidence is accounted for. This is usually what you want, and what is most often confused with the likelihood.",
        ),
      },
      {
        term: bi("Naive Bayes", "Naive Bayes"),
        meaning: bi(
          "Pengklasifikasi yang mengandaikan seluruh fitur saling bebas bila kelasnya diketahui. Andaian itu hampir selalu salah, tetapi hasilnya sering tetap baik karena yang menentukan adalah urutan peringkat kelas, bukan nilai peluangnya.",
          "A classifier assuming all features are independent given the class. That assumption is almost always false, yet results are often good because what matters is the ranking of classes, not the probability values.",
        ),
      },
      {
        term: bi("Penghalusan Laplace", "Laplace smoothing"),
        meaning: bi(
          "Menambahkan hitungan semu agar kombinasi yang belum pernah muncul tidak berpeluang nol. Tanpa itu, satu fitur yang belum pernah terlihat membuat seluruh perkalian menjadi nol.",
          "Adding pseudo-counts so unseen combinations do not get zero probability. Without it, a single unseen feature zeroes the entire product.",
        ),
      },
    ],
    formulas: [
      {
        name: bi("Teorema Bayes", "Bayes' theorem"),
        expression: "P(H|E) = P(E|H) · P(H) / P(E)",
        note: bi(
          "Tidak terdefinisi bila P(E) = 0; bukti yang mustahil tidak bisa memberi informasi apa pun.",
          "Undefined when P(E) = 0; impossible evidence cannot inform anything.",
        ),
      },
      {
        name: bi("Hukum probabilitas total", "Law of total probability"),
        expression: "P(E) = Σᵢ P(E|Hᵢ) · P(Hᵢ)",
        note: bi(
          "Berlaku bila hipotesis-hipotesisnya saling lepas dan mencakup seluruh kemungkinan.",
          "Valid when the hypotheses are mutually exclusive and exhaustive.",
        ),
      },
      {
        name: bi("Rasio kemungkinan", "Likelihood ratio"),
        expression: "LR+ = P(E|H) / P(E|¬H)",
        note: bi(
          "Menyatakan seberapa kuat bukti menggeser keyakinan, terlepas dari prior. Nilai 1 berarti bukti itu tidak informatif sama sekali.",
          "States how strongly the evidence shifts belief, independent of the prior. A value of 1 means the evidence is entirely uninformative.",
        ),
      },
      {
        name: bi("Naive Bayes", "Naive Bayes"),
        expression: "kelas = argmax_c  P(c) · Πᵢ P(xᵢ | c)",
        note: bi(
          "Dihitung pada ranah logaritma agar tidak terjadi underflow ketika fiturnya banyak.",
          "Computed in log space to avoid underflow when there are many features.",
        ),
      },
    ],
    pitfalls: [
      bi(
        "Menyamakan P(E|H) dengan P(H|E). Pada penyakit langka, tes yang sensitif 99 persen tetap lebih sering memberi positif palsu daripada positif benar — dan itu sepenuhnya akibat prior yang kecil.",
        "Equating P(E|H) with P(H|E). For a rare disease, a 99 percent sensitive test still produces more false positives than true ones — entirely because of the small prior.",
      ),
      bi(
        "Melupakan prior karena angkanya terasa tidak relevan. Justru prior yang paling menentukan hasilnya ketika kejadiannya jarang.",
        "Dropping the prior because the number feels irrelevant. The prior dominates the result precisely when the event is rare.",
      ),
    ],
    references: [
      {
        text: "Bayes, T. (1763). An Essay towards solving a Problem in the Doctrine of Chances. Philosophical Transactions of the Royal Society.",
      },
      {
        text: "Mitchell, T. (1997). Machine Learning, Ch. 6: Bayesian Learning.",
      },
    ],
  },

  "fuzzy-logic": {
    summary: bi(
      "Logika kabur membolehkan keanggotaan sebagian: sebuah nilai bisa termasuk himpunan “panas” sebesar 0,7 sekaligus “hangat” sebesar 0,3. Ini bukan ketidakpastian tentang fakta, melainkan ketidaktajaman batas kategorinya.",
      "Fuzzy logic allows partial membership: a value can belong to “hot” to degree 0.7 and “warm” to degree 0.3 at once. This is not uncertainty about a fact but vagueness in the category boundary.",
    ),
    definitions: [
      {
        term: bi("Himpunan kabur", "Fuzzy set"),
        meaning: bi(
          "Himpunan yang keanggotaannya bertingkat, dinyatakan sebuah fungsi dari semesta ke rentang 0 sampai 1. Himpunan tegas adalah kasus khususnya yang hanya bernilai 0 atau 1.",
          "A set whose membership is graded, given by a function from the universe into 0 to 1. A crisp set is the special case taking only 0 or 1.",
        ),
      },
      {
        term: bi("Fungsi keanggotaan", "Membership function"),
        meaning: bi(
          "Fungsi yang memberi derajat keanggotaan tiap nilai. Bentuk segitiga dan trapesium paling sering dipakai karena murah dihitung dan mudah dijelaskan kepada pakar yang menyusun aturannya.",
          "The function assigning each value its degree of membership. Triangular and trapezoidal shapes dominate because they are cheap to compute and easy to explain to the expert writing the rules.",
        ),
      },
      {
        term: bi("Variabel linguistik", "Linguistic variable"),
        meaning: bi(
          "Variabel yang nilainya berupa kata, bukan angka: “suhu” bernilai “dingin”, “hangat”, atau “panas”. Tiap kata itu adalah sebuah himpunan kabur pada semesta yang sama.",
          "A variable whose values are words rather than numbers: “temperature” takes “cold”, “warm”, or “hot”. Each word is a fuzzy set over the same universe.",
        ),
      },
      {
        term: bi("Fuzzifikasi", "Fuzzification"),
        meaning: bi(
          "Mengubah masukan tegas menjadi derajat keanggotaan pada tiap himpunan.",
          "Turning a crisp input into degrees of membership in each set.",
        ),
      },
      {
        term: bi("Derajat penyalaan", "Firing strength"),
        meaning: bi(
          "Seberapa kuat sebuah aturan aktif, dihitung dari derajat premis-premisnya lewat operator AND atau OR.",
          "How strongly a rule is active, computed from its premise degrees through the AND or OR operator.",
        ),
      },
      {
        term: bi("Defuzzifikasi", "Defuzzification"),
        meaning: bi(
          "Mengubah daerah keluaran kabur kembali menjadi satu angka yang bisa dipakai. Metode yang berbeda menghasilkan angka yang berbeda dari daerah yang sama, dan tidak ada yang “paling benar”.",
          "Turning the fuzzy output region back into a single usable number. Different methods give different numbers from the same region, and none is “most correct”.",
        ),
      },
      {
        term: bi("Potongan alfa", "Alpha cut"),
        meaning: bi(
          "Himpunan tegas berisi seluruh nilai yang derajat keanggotaannya minimal alfa. Menghubungkan himpunan kabur kembali ke himpunan biasa.",
          "The crisp set of all values whose membership is at least alpha. It links fuzzy sets back to ordinary ones.",
        ),
      },
    ],
    formulas: [
      {
        name: bi("Keanggotaan segitiga", "Triangular membership"),
        expression: "μ(x) = 0  [x ≤ a ∨ x ≥ c];  (x−a)/(b−a)  [a < x < b];  (c−x)/(c−b)  [b ≤ x < c]",
        note: bi(
          "Puncak harus diperiksa lebih dulu: bentuk berkaki berimpit seperti (5, 5, 10) bernilai penuh tepat di x = 5, dan bentuk itu lazim dipakai di tepi semesta.",
          "The peak must be checked first: a shape with coincident legs such as (5, 5, 10) is fully true exactly at x = 5, and such shapes are common at the edges of a universe.",
        ),
      },
      {
        name: bi("Operator Zadeh", "Zadeh operators"),
        expression: "AND → min(a, b)      OR → max(a, b)      NOT → 1 − a",
        note: bi(
          "Memenuhi hukum De Morgan, tetapi tidak memenuhi hukum kontradiksi: min(a, 1−a) tidak selalu nol.",
          "Satisfies De Morgan's laws but not the law of contradiction: min(a, 1−a) is not always zero.",
        ),
      },
      {
        name: bi("Inferensi Mamdani", "Mamdani inference"),
        expression: "μ_keluaran(y) = maxᵢ [ min(αᵢ, μ_konsekuenᵢ(y)) ]",
        note: bi(
          "Tiap aturan memotong himpunan keluarannya pada derajat penyalaan, lalu seluruhnya digabung dengan maksimum.",
          "Each rule clips its output set at the firing strength, then all are aggregated by maximum.",
        ),
      },
      {
        name: bi("Defuzzifikasi centroid", "Centroid defuzzification"),
        expression: "y* = Σ y·μ(y) / Σ μ(y)",
        note: bi(
          "Tidak terdefinisi bila tidak ada aturan yang menyala. Melaporkannya sebagai galat jauh lebih jujur daripada mengembalikan titik tengah semesta.",
          "Undefined when no rule fires. Reporting that as an error is far more honest than returning the midpoint of the universe.",
        ),
      },
      {
        name: bi("Sugeno orde nol", "Zero-order Sugeno"),
        expression: "y* = Σ αᵢ·zᵢ / Σ αᵢ",
        note: bi(
          "Keluaran tiap aturan berupa satu bilangan tetap, bukan himpunan, sehingga jauh lebih murah dihitung.",
          "Each rule's output is a single constant rather than a set, making it far cheaper to compute.",
        ),
      },
    ],
    pitfalls: [
      bi(
        "Menyamakan derajat keanggotaan dengan probabilitas. Keanggotaan 0,7 pada himpunan “panas” bukan berarti “kemungkinan 70 persen panas”, melainkan “panas sampai tingkat 0,7”.",
        "Equating membership with probability. A membership of 0.7 in “hot” does not mean “70 percent likely to be hot” but “hot to degree 0.7”.",
      ),
      bi(
        "Memakai himpunan bahu yang salah di tepi semesta. Trapesium (5, 8, 10, 10) harus bernilai penuh di x = 10; kalau tidak, seluruh aturan mati di ujung atas tanpa pesan galat apa pun.",
        "Getting shoulder sets wrong at the universe edge. A trapezoid (5, 8, 10, 10) must be fully true at x = 10; otherwise every rule dies at the top end with no error message.",
      ),
    ],
    references: [
      {
        text: "Zadeh, L. A. (1965). Fuzzy sets. Information and Control, 8(3), 338–353.",
        url: "https://doi.org/10.1016/S0019-9958(65)90241-X",
      },
      {
        text: "Zadeh, L. A. (2008). Is there a need for fuzzy logic? Information Sciences, 178(13), 2751–2779.",
        url: "https://doi.org/10.1016/j.ins.2008.02.012",
      },
      {
        text: "Mamdani, E. H. & Assilian, S. (1975). An experiment in linguistic synthesis with a fuzzy logic controller.",
      },
    ],
  },

  knowledge: {
    summary: bi(
      "Representasi pengetahuan menanyakan bentuk apa yang harus dipakai menyimpan apa yang diketahui, agar kesimpulan bisa ditarik darinya. Bentuk yang berbeda memudahkan pertanyaan yang berbeda.",
      "Knowledge representation asks what form knowledge should be stored in so that conclusions can be drawn from it. Different forms make different questions easy.",
    ),
    definitions: [
      {
        term: bi("Proposisi", "Proposition"),
        meaning: bi(
          "Pernyataan yang bernilai benar atau salah, tanpa struktur internal. Logika proposisi tidak bisa menyatakan “semua manusia fana”; untuk itu diperlukan logika predikat.",
          "A statement that is either true or false, with no internal structure. Propositional logic cannot say “all humans are mortal”; that needs predicate logic.",
        ),
      },
      {
        term: bi("Tautologi", "Tautology"),
        meaning: bi(
          "Rumus yang benar pada seluruh baris tabel kebenarannya. Aturan penalaran yang sah selalu berbentuk tautologi.",
          "A formula true on every row of its truth table. Every valid inference rule takes the form of a tautology.",
        ),
      },
      {
        term: bi("Kepuasan", "Satisfiability"),
        meaning: bi(
          "Sebuah rumus disebut dapat dipuaskan bila ada minimal satu penugasan nilai yang membuatnya benar. Kontradiksi adalah rumus yang tidak dapat dipuaskan.",
          "A formula is satisfiable if at least one assignment makes it true. A contradiction is a formula that is not satisfiable.",
        ),
      },
      {
        term: bi("Bentuk normal konjungtif", "Conjunctive normal form"),
        meaning: bi(
          "Konjungsi dari disjungsi-disjungsi literal. Setiap rumus proposisi punya padanan dalam bentuk ini, dan resolusi hanya bekerja pada bentuk ini.",
          "A conjunction of disjunctions of literals. Every propositional formula has an equivalent in this form, and resolution only works on it.",
        ),
      },
      {
        term: bi("Resolusi", "Resolution"),
        meaning: bi(
          "Aturan penalaran yang menghapuskan sepasang literal berlawanan dari dua klausa. Dipakai membuktikan dengan menyangkal kesimpulan lalu mencari kontradiksi.",
          "An inference rule cancelling a pair of opposing literals from two clauses. Used to prove by negating the conclusion and hunting a contradiction.",
        ),
      },
      {
        term: bi("Klausa kosong", "The empty clause"),
        meaning: bi(
          "Klausa tanpa literal sama sekali, dilambangkan □. Kemunculannya berarti kontradiksi telah diturunkan, sehingga kesimpulan yang disangkal tadi pasti benar.",
          "A clause with no literals at all, written □. Its appearance means a contradiction has been derived, so the negated conclusion must be true.",
        ),
      },
      {
        term: bi("Jaringan semantik", "Semantic network"),
        meaning: bi(
          "Graf berarah dengan simpul sebagai konsep dan sisi sebagai hubungan. Kekuatannya pada pewarisan: sifat yang ditulis sekali berlaku untuk seluruh turunannya.",
          "A directed graph with concepts as nodes and relations as edges. Its strength is inheritance: a property written once applies to every descendant.",
        ),
      },
      {
        term: bi("Bingkai", "Frame"),
        meaning: bi(
          "Struktur bersolot yang mewakili sebuah konsep, dengan pewarisan dari induknya. Slot anak menimpa slot induk, sehingga pengecualian bisa dinyatakan tanpa membatalkan aturan umumnya.",
          "A slotted structure representing a concept, inheriting from a parent. Child slots override parent slots, so exceptions can be stated without cancelling the general rule.",
        ),
      },
    ],
    formulas: [
      {
        name: bi("Ukuran tabel kebenaran", "Truth-table size"),
        expression: "rows = 2ⁿ,  n = |propositions|",
        note: bi(
          "Enam belas proposisi sudah berarti 65.536 baris. Di situlah tabel kebenaran berhenti berguna dan resolusi mengambil alih.",
          "Sixteen propositions already means 65,536 rows. That is where truth tables stop being useful and resolution takes over.",
        ),
      },
      {
        name: bi("Aturan resolusi", "The resolution rule"),
        expression: "(A ∨ p),  (B ∨ ¬p)  ⊢  (A ∨ B)",
        note: bi(
          "Klausa hasil yang memuat literal beserta ingkarannya selalu benar, sehingga tidak membawa informasi baru dan boleh dibuang.",
          "A resolvent containing both a literal and its negation is always true, carries no new information, and may be discarded.",
        ),
      },
      {
        name: bi("Menghapus implikasi", "Removing implication"),
        expression: "A → B  ≡  ¬A ∨ B          A ↔ B  ≡  (¬A ∨ B) ∧ (¬B ∨ A)",
        note: bi(
          "Langkah pertama menuju bentuk normal konjungtif.",
          "The first step toward conjunctive normal form.",
        ),
      },
      {
        name: bi("Hukum De Morgan", "De Morgan's laws"),
        expression: "¬(A ∧ B) ≡ ¬A ∨ ¬B          ¬(A ∨ B) ≡ ¬A ∧ ¬B",
        note: bi(
          "Dipakai mendorong ingkaran sampai ke proposisi dasar.",
          "Used to push negations down to the atomic propositions.",
        ),
      },
    ],
    pitfalls: [
      bi(
        "Mengira “tidak terbukti” sama dengan “salah”. Resolusi yang gagal menurunkan klausa kosong hanya menyatakan kesimpulan itu tidak mengikuti dari basis pengetahuan yang ada.",
        "Reading “not proved” as “false”. A resolution that fails to derive the empty clause only says the conclusion does not follow from the knowledge base at hand.",
      ),
      bi(
        "Membiarkan basis pengetahuan yang melingkar. Penelusuran akan berputar sampai tumpukan pemanggilan habis, bukan melaporkan bahwa aturannya memang saling merujuk.",
        "Leaving a circular knowledge base. The search spins until the call stack is exhausted rather than reporting that the rules refer to each other.",
      ),
    ],
    references: [
      {
        text: "Robinson, J. A. (1965). A Machine-Oriented Logic Based on the Resolution Principle. Journal of the ACM, 12(1), 23–41.",
        url: "https://doi.org/10.1145/321250.321253",
      },
      {
        text: "Quillian, M. R. (1968). Semantic Memory. Dalam Semantic Information Processing.",
      },
      {
        text: "Minsky, M. (1974). A Framework for Representing Knowledge. MIT-AI Laboratory Memo 306.",
      },
    ],
  },

  search: {
    summary: bi(
      "Pencarian mengubah pemecahan masalah menjadi penelusuran graf keadaan. Yang membedakan algoritmanya bukan jawabannya, melainkan berapa banyak keadaan yang harus mereka periksa untuk sampai ke sana.",
      "Search turns problem solving into traversing a graph of states. What separates the algorithms is not the answer but how many states they must examine to reach it.",
    ),
    definitions: [
      {
        term: bi("Simpul dibuka", "Nodes expanded"),
        meaning: bi(
          "Simpul yang diambil dari daftar tunggu dan diperiksa tetangganya. Jumlah simpul yang dibuka adalah ukuran biaya sebenarnya sebuah pencarian, bukan panjang jalurnya.",
          "A node taken from the frontier and whose neighbours are examined. The number of expansions is the true cost measure of a search, not the path length.",
        ),
      },
      {
        term: bi("Daftar tunggu", "Frontier"),
        meaning: bi(
          "Kumpulan simpul yang sudah ditemukan tetapi belum dibuka. Ukurannya menentukan kebutuhan memori, dan di sinilah DFS jauh lebih hemat daripada BFS.",
          "The set of discovered but not yet expanded nodes. Its size determines memory use, and this is where DFS is far cheaper than BFS.",
        ),
      },
      {
        term: bi("Optimal", "Optimal"),
        meaning: bi(
          "Menjamin menemukan jalur termurah, bukan sekadar menemukan jalur. Jaminan ini bersifat teoretis; algoritma tak optimal kadang beruntung menemukan jalur terpendek.",
          "Guaranteed to find a cheapest path, not merely a path. The guarantee is theoretical; a non-optimal algorithm sometimes gets lucky.",
        ),
      },
      {
        term: bi("Lengkap", "Complete"),
        meaning: bi(
          "Menjamin menemukan penyelesaian bila ada. DFS tidak lengkap pada ruang tak berhingga; hill climbing tidak lengkap bahkan pada ruang berhingga.",
          "Guaranteed to find a solution if one exists. DFS is incomplete on infinite spaces; hill climbing is incomplete even on finite ones.",
        ),
      },
      {
        term: bi("Heuristik", "Heuristic"),
        meaning: bi(
          "Taksiran biaya dari sebuah keadaan menuju tujuan. Ia memandu pencarian tanpa menjamin apa pun, kecuali bila memenuhi sifat admissible.",
          "An estimate of the cost from a state to the goal. It guides the search without guaranteeing anything, unless it is admissible.",
        ),
      },
      {
        term: bi("Admissible", "Admissible"),
        meaning: bi(
          "Heuristik yang tidak pernah menaksir lebih besar daripada biaya sebenarnya. Sifat inilah yang membuat A* dijamin optimal; heuristik yang menaksir berlebih bisa membuat A* melewatkan jalur terbaik.",
          "A heuristic that never overestimates the true cost. This property is what makes A* optimal; an overestimating heuristic can make A* miss the best path.",
        ),
      },
      {
        term: bi("Minimum lokal", "Local minimum"),
        meaning: bi(
          "Keadaan yang seluruh tetangganya terlihat lebih buruk, padahal ada keadaan yang jauh lebih baik di tempat lain. Inilah yang menjebak hill climbing.",
          "A state whose neighbours all look worse, although a far better state exists elsewhere. This is what traps hill climbing.",
        ),
      },
    ],
    formulas: [
      {
        name: bi("Fungsi evaluasi A*", "The A* evaluation function"),
        expression: "f(n) = g(n) + h(n)",
        note: bi(
          "g adalah biaya yang sudah ditempuh, h taksiran sisanya. Bila h selalu nol, A* merosot menjadi pencarian biaya seragam.",
          "g is the cost already paid, h the estimated remainder. If h is always zero, A* degrades into uniform-cost search.",
        ),
      },
      {
        name: bi("Jarak Manhattan", "Manhattan distance"),
        expression: "h(n) = |x₁ − x₂| + |y₁ − y₂|",
        note: bi(
          "Admissible untuk gerak empat arah berbiaya seragam, tetapi menaksir berlebih bila gerak diagonal diizinkan.",
          "Admissible for uniform-cost four-way movement, but overestimates when diagonal moves are allowed.",
        ),
      },
      {
        name: bi("Penerimaan simulated annealing", "Simulated-annealing acceptance"),
        expression: "P(accept) = 1  [Δ < 0];  exp(−Δ / T)  [Δ ≥ 0]",
        note: bi(
          "Suhu T menurun seiring waktu, sehingga perilakunya berangsur menyerupai hill climbing.",
          "The temperature T decreases over time, so the behaviour gradually approaches hill climbing.",
        ),
      },
    ],
    pitfalls: [
      bi(
        "Menilai algoritma dari panjang jalurnya. Pada peta contoh di laboratorium ini, tujuh algoritma menemukan jalur sama panjang sambil membuka jumlah sel yang berbeda seratus kali lipat.",
        "Judging an algorithm by path length. On the sample map in this lab, seven algorithms find equally long paths while expanding cell counts that differ a hundredfold.",
      ),
      bi(
        "Melupakan pemutus seri pada A*. Di ruang terbuka, ribuan simpul punya nilai f identik, dan tanpa pemutus seri A* membuka seluruh peta — persis sebanyak pencarian tanpa heuristik.",
        "Forgetting tie-breaking in A*. In open space thousands of nodes share the same f, and without tie-breaking A* expands the whole map — exactly as many as a heuristic-free search.",
      ),
    ],
    references: [
      {
        text: "Hart, P. E., Nilsson, N. J. & Raphael, B. (1968). A Formal Basis for the Heuristic Determination of Minimum Cost Paths. IEEE Transactions on Systems Science and Cybernetics, 4(2), 100–107.",
        url: "https://doi.org/10.1109/TSSC.1968.300136",
      },
      {
        text: "Kirkpatrick, S., Gelatt, C. D. & Vecchi, M. P. (1983). Optimization by Simulated Annealing. Science, 220(4598), 671–680.",
      },
    ],
  },

  "neural-network": {
    summary: bi(
      "Jaringan syaraf tiruan menyusun fungsi sederhana berlapis-lapis, lalu menyesuaikan bobotnya sampai keluarannya mendekati target. Perambatan balik adalah cara menghitung ke arah mana tiap bobot harus digeser.",
      "A neural network composes simple functions in layers, then adjusts the weights until the output approaches the target. Backpropagation is how the direction to shift each weight is computed.",
    ),
    definitions: [
      {
        term: bi("Perceptron", "Perceptron"),
        meaning: bi(
          "Neuron tunggal yang menghitung jumlah berbobot masukannya lalu melewatkannya ke fungsi ambang. Hanya bisa memisahkan kelas yang terpisahkan satu garis lurus.",
          "A single neuron computing a weighted sum of its inputs and passing it through a threshold. It can only separate classes that a single straight line can divide.",
        ),
      },
      {
        term: bi("Fungsi aktivasi", "Activation function"),
        meaning: bi(
          "Fungsi tak linear yang diterapkan pada keluaran tiap neuron. Tanpa ketaklinearan, berapa pun banyak lapisannya tetap setara dengan satu lapisan.",
          "The non-linear function applied to each neuron's output. Without non-linearity, any number of layers collapses into one.",
        ),
      },
      {
        term: bi("Perambatan balik", "Backpropagation"),
        meaning: bi(
          "Menghitung turunan galat terhadap tiap bobot dengan aturan rantai, dari lapisan keluaran mundur ke masukan. Bukan algoritma pembelajaran melainkan cara menghitung gradien.",
          "Computing the derivative of the error with respect to every weight by the chain rule, from the output layer backwards. It is not a learning algorithm but a way to compute gradients.",
        ),
      },
      {
        term: bi("Laju belajar", "Learning rate"),
        meaning: bi(
          "Seberapa jauh bobot digeser mengikuti gradien. Terlalu kecil membuat pelatihan lambat; terlalu besar membuatnya berayun dan tidak pernah menetap.",
          "How far the weights move along the gradient. Too small makes training slow; too large makes it oscillate and never settle.",
        ),
      },
      {
        term: bi("Momentum", "Momentum"),
        meaning: bi(
          "Menambahkan sebagian langkah sebelumnya ke langkah sekarang. Mempercepat pelatihan, tetapi juga memperbesar langkah efektifnya kira-kira 1/(1−momentum) kali.",
          "Adding part of the previous step to the current one. It speeds training, but also amplifies the effective step by roughly 1/(1−momentum).",
        ),
      },
      {
        term: bi("Epoch", "Epoch"),
        meaning: bi(
          "Satu kali melewatkan seluruh data latih. Bukan ukuran waktu maupun ukuran kualitas; jaringan bisa memburuk setelah epoch tertentu.",
          "One full pass over the training data. Neither a measure of time nor of quality; a network can get worse after a certain epoch.",
        ),
      },
      {
        term: bi("Inisialisasi Xavier", "Xavier initialisation"),
        meaning: bi(
          "Menetapkan bobot awal dalam rentang yang menjaga besar sinyal tetap wajar saat melewati banyak lapisan. Bobot awal yang terlalu besar membuat aktivasi jenuh sejak awal.",
          "Setting initial weights in a range that keeps signal magnitude sensible across many layers. Initial weights that are too large saturate the activations from the start.",
        ),
      },
    ],
    formulas: [
      {
        name: bi("Keluaran neuron", "Neuron output"),
        expression: "y = φ( Σᵢ wᵢxᵢ + b )",
        note: bi(
          "φ adalah fungsi aktivasi, b adalah bias yang menggeser ambangnya.",
          "φ is the activation function and b the bias shifting the threshold.",
        ),
      },
      {
        name: bi("Sigmoid dan turunannya", "The sigmoid and its derivative"),
        expression: "σ(x) = 1 / (1 + e^(−x))          σ′ = σ(1 − σ)",
        note: bi(
          "Bentuk turunannya yang dinyatakan lewat keluaran inilah yang membuat perambatan balik murah: keluaran tiap neuron sudah tersimpan.",
          "Expressing the derivative through the output is what makes backpropagation cheap: each neuron's output is already stored.",
        ),
      },
      {
        name: bi("Delta lapisan keluaran", "Output-layer delta"),
        expression: "δ = (target − keluaran) · φ′(keluaran)",
        note: bi(
          "Untuk lapisan tersembunyi, δ dirambatkan mundur lewat bobot lapisan sesudahnya.",
          "For hidden layers, δ is propagated backwards through the weights of the following layer.",
        ),
      },
      {
        name: bi("Laju belajar efektif", "Effective learning rate"),
        expression: "laju_efektif ≈ laju / (1 − momentum)",
        note: bi(
          "Pada momentum 0,9 nilainya sepuluh kali lipat. Pengukuran pada kumpulan data spiral: laju 0,08 tuntas dalam 70 epoch, sedangkan laju 0,2 dengan momentum sama macet di 50 persen.",
          "At momentum 0.9 this is tenfold. Measured on the spiral dataset: a rate of 0.08 finishes in 70 epochs, while 0.2 at the same momentum stalls at 50 percent.",
        ),
      },
    ],
    pitfalls: [
      bi(
        "Mengira galat yang menurun membuktikan gradiennya benar. Jaringan yang gradiennya salah pun sering tetap belajar, hanya lebih lambat dan berhenti di tempat yang keliru. Satu-satunya bukti adalah membandingkan gradien analitik dengan selisih hingga.",
        "Assuming a falling error proves the gradient is right. A network with a wrong gradient often still learns, just more slowly and stopping in the wrong place. The only proof is comparing analytic gradients against finite differences.",
      ),
      bi(
        "Menaikkan momentum tanpa menurunkan laju belajar. Keduanya berlipat, dan langkah efektif di atas satu membuat pelatihan berayun alih-alih menurun.",
        "Raising momentum without lowering the learning rate. The two multiply, and an effective step above one makes training oscillate rather than descend.",
      ),
    ],
    references: [
      {
        text: "Rumelhart, D. E., Hinton, G. E. & Williams, R. J. (1986). Learning representations by back-propagating errors. Nature, 323, 533–536.",
        url: "https://doi.org/10.1038/323533a0",
      },
      {
        text: "Minsky, M. & Papert, S. (1969). Perceptrons. MIT Press — sumber batas XOR.",
      },
      {
        text: "Glorot, X. & Bengio, Y. (2010). Understanding the difficulty of training deep feedforward neural networks.",
      },
    ],
  },

  nlp: {
    summary: bi(
      "Pemrosesan bahasa alami mengubah teks menjadi bentuk yang bisa dihitung. Tiap tahap membuang sebagian informasi dengan sengaja, dan yang menentukan mutu hasilnya adalah ketepatan memilih apa yang dibuang.",
      "Natural language processing turns text into something computable. Each stage deliberately discards information, and the quality of the result hinges on choosing correctly what to discard.",
    ),
    definitions: [
      {
        term: bi("Tokenisasi", "Tokenisation"),
        meaning: bi(
          "Memecah teks menjadi satuan kata. Untuk Bahasa Indonesia, tanda hubung harus dipertahankan di tengah kata karena dipakai untuk pengulangan: “anak-anak” adalah satu kata.",
          "Splitting text into word units. For Indonesian, hyphens inside words must be kept because they mark reduplication: “anak-anak” is one word.",
        ),
      },
      {
        term: bi("Kata henti", "Stopword"),
        meaning: bi(
          "Kata yang sangat sering muncul sehingga dianggap tidak membedakan makna. Daftar yang terlalu panjang berbahaya: membuang “tidak” mengubah “tidak bagus” menjadi “bagus”.",
          "Words so frequent they are treated as non-distinguishing. An over-long list is dangerous: removing “not” turns “not good” into “good”.",
        ),
      },
      {
        term: bi("Stemming", "Stemming"),
        meaning: bi(
          "Mencari kata dasar dengan mengupas imbuhan. Bahasa Indonesia menuntut kamus karena sebagian awalan meluluhkan huruf pertama kata dasarnya.",
          "Finding the root by stripping affixes. Indonesian requires a dictionary because some prefixes dissolve the root's first letter.",
        ),
      },
      {
        term: bi("Nazief-Adriani", "Nazief-Adriani"),
        meaning: bi(
          "Algoritma stemming Bahasa Indonesia yang mengupas berurutan: partikel, kata ganti kepemilikan, akhiran turunan, lalu awalan — dan memeriksa kamus di tiap tahap.",
          "The Indonesian stemming algorithm stripping in order: particles, possessives, derivational suffixes, then prefixes — checking the dictionary at each stage.",
        ),
      },
      {
        term: bi("TF-IDF", "TF-IDF"),
        meaning: bi(
          "Bobot yang tinggi untuk kata yang sering muncul di satu dokumen tetapi jarang di dokumen lain. Menangkap kata yang membedakan, bukan yang sekadar sering.",
          "A weight that is high for terms frequent in one document but rare across the corpus. It captures distinguishing words, not merely frequent ones.",
        ),
      },
      {
        term: bi("Kemiripan kosinus", "Cosine similarity"),
        meaning: bi(
          "Sudut antara dua vektor dokumen. Tidak terpengaruh panjang dokumen, sehingga dua tulisan tentang hal sama tetap dinilai mirip walau panjangnya jauh berbeda.",
          "The angle between two document vectors. Unaffected by document length, so two texts on the same subject stay similar at very different lengths.",
        ),
      },
      {
        term: bi("Jarak Levenshtein", "Levenshtein distance"),
        meaning: bi(
          "Jumlah penyisipan, penghapusan, dan penggantian karakter minimum untuk mengubah satu kata menjadi kata lain. Harus dihitung per karakter Unicode, bukan per bita.",
          "The minimum number of character insertions, deletions, and substitutions turning one word into another. Must be counted in Unicode characters, not bytes.",
        ),
      },
    ],
    formulas: [
      {
        name: bi("TF-IDF", "TF-IDF"),
        expression: "tfidf(t, d) = tf(t, d) × idf(t)          idf(t) = ln((1 + N) / (1 + df(t))) + 1",
        note: bi(
          "Bentuk IDF yang dihaluskan dipakai karena bentuk mentah ln(N/df) memberi nol untuk kata yang muncul di semua dokumen, sehingga kata itu lenyap dari perhitungan.",
          "The smoothed IDF is used because the raw ln(N/df) gives zero for terms in every document, erasing them from the calculation.",
        ),
      },
      {
        name: bi("Kemiripan kosinus", "Cosine similarity"),
        expression: "cos(a, b) = (a · b) / (‖a‖ ‖b‖)",
        note: bi(
          "Vektor nol tidak punya arah; kemiripannya dilaporkan nol alih-alih menghasilkan pembagian dengan nol.",
          "A zero vector has no direction; the similarity is reported as zero rather than dividing by zero.",
        ),
      },
      {
        name: bi("Urutan pengupasan imbuhan", "Affix-stripping order"),
        expression: "partikel → kepemilikan → akhiran → awalan",
        note: bi(
          "Urutannya penting. Membaliknya menghasilkan kata dasar yang salah pada kata berimbuhan rangkap seperti “dituliskannya”.",
          "The order matters. Reversing it yields the wrong root for multiply affixed words such as “dituliskannya”.",
        ),
      },
    ],
    pitfalls: [
      bi(
        "Menerapkan aturan peluluhan tanpa syarat. Awalan “mem-” meluluhkan huruf p pada “memukul” yang berasal dari “pukul”, tetapi “membaca” berasal dari “baca” tanpa peluluhan. Satu-satunya cara memutuskan adalah mencoba kedua kandidat lalu bertanya kepada kamus.",
        "Applying the dissolution rule unconditionally. The prefix “mem-” dissolves the p in “memukul” from “pukul”, but “membaca” comes from “baca” with no dissolution. The only way to decide is to try both candidates and ask the dictionary.",
      ),
      bi(
        "Menghitung jarak sunting per bita. Huruf beraksen memakai dua bita dalam UTF-8, sehingga “café” dan “cafe” akan dinilai berjarak dua, bukan satu.",
        "Counting edit distance in bytes. An accented letter takes two bytes in UTF-8, so “café” and “cafe” would be scored two apart instead of one.",
      ),
    ],
    references: [
      {
        text: "Nazief, B. & Adriani, M. (1996). Confix-Stripping: Approach to Stemming Algorithm for Bahasa Indonesia. Universitas Indonesia.",
      },
      {
        text: "Levenshtein, V. I. (1966). Binary codes capable of correcting deletions, insertions, and reversals.",
      },
      {
        text: "Salton, G. & Buckley, C. (1988). Term-weighting approaches in automatic text retrieval.",
      },
    ],
  },

  "expert-system": {
    summary: bi(
      "Sistem pakar memisahkan pengetahuan dari mesin yang memakainya. Basis pengetahuannya bisa diubah tanpa menyentuh mesin inferensinya, dan itulah yang membuatnya bisa dirawat pakar yang bukan pemrogram.",
      "An expert system separates knowledge from the machinery that uses it. The knowledge base can change without touching the inference engine, which is what lets a non-programmer expert maintain it.",
    ),
    definitions: [
      {
        term: bi("Basis pengetahuan", "Knowledge base"),
        meaning: bi(
          "Kumpulan aturan dan fakta yang mewakili keahlian seorang pakar. Terpisah dari mesin inferensi, sehingga bisa diperbarui tanpa memprogram ulang.",
          "The rules and facts representing an expert's knowledge. Kept separate from the inference engine so it can be updated without reprogramming.",
        ),
      },
      {
        term: bi("Mesin inferensi", "Inference engine"),
        meaning: bi(
          "Bagian yang menerapkan aturan pada fakta untuk menghasilkan kesimpulan baru. Sama untuk semua bidang; yang berganti hanya basis pengetahuannya.",
          "The part applying rules to facts to derive new conclusions. It is the same across domains; only the knowledge base changes.",
        ),
      },
      {
        term: bi("Memori kerja", "Working memory"),
        meaning: bi(
          "Fakta yang diketahui saat ini, termasuk yang baru disimpulkan. Isinya berubah selama penalaran berlangsung.",
          "The facts currently known, including newly derived ones. Its contents change as reasoning proceeds.",
        ),
      },
      {
        term: bi("Runut maju", "Forward chaining"),
        meaning: bi(
          "Berangkat dari fakta yang ada dan menyalakan aturan sampai tidak ada lagi kesimpulan baru. Cocok bila datanya lengkap dan pertanyaannya “apa yang bisa disimpulkan”.",
          "Starting from known facts and firing rules until nothing new follows. Suited to complete data and the question “what follows”.",
        ),
      },
      {
        term: bi("Runut mundur", "Backward chaining"),
        meaning: bi(
          "Berangkat dari hipotesis dan menelusuri mundur mencari dukungannya. Cocok bila pertanyaannya “benarkah dugaan ini”, karena hanya fakta yang relevan yang perlu ditanyakan.",
          "Starting from a hypothesis and tracing backwards for support. Suited to the question “is this true”, because only relevant facts need be asked.",
        ),
      },
      {
        term: bi("Fasilitas penjelasan", "Explanation facility"),
        meaning: bi(
          "Kemampuan menjawab “kenapa pertanyaan ini diajukan” dan “bagaimana kesimpulan ini diperoleh”. Tanpa itu, sistem pakar hanyalah tebakan bercangkang komputer.",
          "The ability to answer “why is this being asked” and “how was this concluded”. Without it, an expert system is just a guess in a computer's shell.",
        ),
      },
      {
        term: bi("Fakta daun", "Leaf fact"),
        meaning: bi(
          "Fakta yang muncul sebagai premis tetapi tidak bisa disimpulkan aturan mana pun. Fakta seperti ini harus bisa ditanyakan; kalau tidak, ia diam-diam dianggap tidak berlaku.",
          "A fact appearing as a premise but derivable by no rule. Such facts must be askable; otherwise they are silently treated as false.",
        ),
      },
    ],
    formulas: [
      {
        name: bi("Pemangkasan runut mundur", "Backward-chaining pruning"),
        expression: "stop  ⟺  CF_best ≥ max( CF_rule : r ∈ remaining )",
        note: bi(
          "Sah karena keyakinan sebuah aturan tidak pernah melebihi keyakinan aturan itu sendiri. Pemangkasan ini mengurangi pertanyaan tanpa mengubah jawaban.",
          "Valid because a rule's conclusion never exceeds the rule's own certainty. This pruning reduces questions without changing the answer.",
        ),
      },
      {
        name: bi("Penggabungan bukti", "Combining evidence"),
        expression: "CF_new = combine_parallel( CF_old, CF_from_rule )",
        note: bi(
          "Aturan yang sama dengan dukungan yang sama tidak dijalankan dua kali; tanpa penjagaan itu keyakinannya merangkak naik ke satu tanpa bukti tambahan.",
          "The same rule with the same support is not fired twice; without that guard the certainty creeps toward one with no new evidence.",
        ),
      },
    ],
    pitfalls: [
      bi(
        "Membiarkan premis tanpa sumber. Fakta yang tidak bisa disimpulkan maupun ditanyakan diperlakukan sebagai tidak berlaku, sehingga sebagian aturan tidak pernah menyala tanpa pesan galat apa pun.",
        "Leaving premises with no source. A fact that can neither be derived nor asked is treated as false, so some rules never fire and nothing reports an error.",
      ),
      bi(
        "Menyamakan “tidak menjawab” dengan “menjawab tidak”. Sistem pakar yang baik memperlakukan keduanya berbeda, karena ketiadaan informasi bukan bukti yang menentang.",
        "Equating “no answer” with “answered no”. A good expert system treats them differently, because absence of information is not evidence against.",
      ),
    ],
    references: [
      {
        text: "Buchanan, B. G. & Shortliffe, E. H. (1984). Rule-Based Expert Systems: The MYCIN Experiments.",
      },
      {
        text: "Modul Sesi 11 IND323, studi kasus “Dokter Virtual”, Universitas Esa Unggul.",
      },
    ],
  },

  "machine-learning": {
    summary: bi(
      "Pembelajaran mesin mencari pola dalam data alih-alih memprogramnya. Bagian yang paling menentukan bukan pemilihan algoritmanya, melainkan kejujuran cara mengukur hasilnya.",
      "Machine learning finds patterns in data instead of programming them. The decisive part is not the choice of algorithm but the honesty of the evaluation.",
    ),
    definitions: [
      {
        term: bi("Pembelajaran terbimbing", "Supervised learning"),
        meaning: bi(
          "Belajar dari contoh yang sudah berlabel. KNN, pohon keputusan, dan regresi termasuk di sini.",
          "Learning from labelled examples. KNN, decision trees, and regression belong here.",
        ),
      },
      {
        term: bi("Pembelajaran tak terbimbing", "Unsupervised learning"),
        meaning: bi(
          "Menemukan struktur tanpa label sama sekali. K-Means termasuk di sini: warna kelompok yang dihasilkannya adalah temuannya sendiri, bukan kelas yang Anda berikan.",
          "Finding structure with no labels at all. K-Means belongs here: the cluster colours it produces are its own finding, not classes you supplied.",
        ),
      },
      {
        term: bi("Pembelajar malas", "Lazy learner"),
        meaning: bi(
          "Model yang tidak melatih apa pun; seluruh kerja terjadi saat memprediksi. KNN adalah contohnya, dan itulah sebabnya ia lambat pada data besar.",
          "A model that trains nothing; all work happens at prediction time. KNN is the example, and that is why it is slow on large data.",
        ),
      },
      {
        term: bi("Entropi", "Entropy"),
        meaning: bi(
          "Ukuran ketidakpastian sebuah sebaran, dalam bit. Bernilai nol bila seluruh label sama, dan maksimum bila semua kelas muncul sama banyak.",
          "A measure of a distribution's uncertainty, in bits. Zero when all labels agree, maximal when all classes are equally frequent.",
        ),
      },
      {
        term: bi("Perolehan informasi", "Information gain"),
        meaning: bi(
          "Berkurangnya entropi setelah data dipecah menurut sebuah atribut. ID3 memilih atribut dengan perolehan tertinggi.",
          "The reduction in entropy after splitting on an attribute. ID3 picks the attribute with the highest gain.",
        ),
      },
      {
        term: bi("Inertia", "Inertia"),
        meaning: bi(
          "Jumlah kuadrat jarak tiap titik ke pusat kelompoknya. Selalu menurun saat jumlah kelompok dinaikkan, sehingga tidak bisa dipakai memilih jumlah kelompok terbaik.",
          "The sum of squared distances from each point to its cluster centre. It always falls as clusters increase, so it cannot be used to choose the best number of clusters.",
        ),
      },
      {
        term: bi("Matriks konfusi", "Confusion matrix"),
        meaning: bi(
          "Tabel yang mencacah ramalan benar dan salah untuk tiap kelas. Menunjukkan bukan hanya berapa banyak yang salah, tetapi salah menjadi apa.",
          "A table counting correct and incorrect predictions per class. It shows not only how many are wrong but what they were mistaken for.",
        ),
      },
      {
        term: bi("Ketepatan pembanding", "Baseline accuracy"),
        meaning: bi(
          "Ketepatan yang dicapai dengan selalu menebak kelas terbanyak. Model yang tidak melampaui angka ini belum mempelajari apa pun.",
          "The accuracy achieved by always guessing the majority class. A model that does not beat it has learned nothing.",
        ),
      },
    ],
    formulas: [
      {
        name: bi("Entropi Shannon", "Shannon entropy"),
        expression: "H(S) = − Σᵢ pᵢ log₂ pᵢ",
        note: bi(
          "Pada dataset “bermain tenis” klasik dengan 9 Ya dan 5 Tidak, nilainya 0,940 bit.",
          "On the classic “play tennis” dataset with 9 Yes and 5 No, this is 0.940 bits.",
        ),
      },
      {
        name: bi("Perolehan informasi", "Information gain"),
        expression: "Gain(S, A) = H(S) − Σᵥ (|Sᵥ| / |S|) · H(Sᵥ)",
        note: bi(
          "Pada dataset yang sama, atribut Cuaca memberi 0,247 bit — tertinggi di antara keempat atributnya.",
          "On the same dataset, the Outlook attribute gives 0.247 bits — the highest of the four.",
        ),
      },
      {
        name: bi("Ketakmurnian Gini", "Gini impurity"),
        expression: "Gini(S) = 1 − Σᵢ pᵢ²",
        note: bi(
          "Lebih murah dihitung daripada entropi karena tidak memakai logaritma, dan pada dua kelas nilainya tidak pernah melebihi entropi.",
          "Cheaper than entropy because it uses no logarithm, and for two classes never exceeds the entropy.",
        ),
      },
      {
        name: bi("Skor F1", "F1 score"),
        expression: "F1 = 2 · presisi · kepekaan / (presisi + kepekaan)",
        note: bi(
          "F1 makro merata-ratakan tanpa membobot jumlah anggota kelas, sehingga kelas minoritas tidak tertelan.",
          "Macro F1 averages without weighting by class size, so a minority class is not drowned out.",
        ),
      },
      {
        name: bi("Regresi linear", "Linear regression"),
        expression: "b = Σ(x−x̄)(y−ȳ) / Σ(x−x̄)²          a = ȳ − b·x̄",
        note: bi(
          "Bentuk tertutup; untuk satu peubah jawabannya bisa dihitung langsung dan pasti optimal.",
          "A closed form; for one variable the answer is computable directly and is provably optimal.",
        ),
      },
    ],
    pitfalls: [
      bi(
        "Menilai model hanya dari ketepatan. Pada data yang 99 persen satu kelas, menebak kelas itu terus-menerus sudah memberi ketepatan 99 persen tanpa mempelajari apa pun.",
        "Judging a model by accuracy alone. On data that is 99 percent one class, always guessing that class already gives 99 percent accuracy without learning anything.",
      ),
      bi(
        "Memilih jumlah kelompok dengan mencari inertia terkecil. Inertia selalu menurun saat k dinaikkan, jadi jawabannya selalu “sebanyak titiknya”. Yang dicari adalah tempat penurunannya melandai.",
        "Choosing the cluster count by minimising inertia. Inertia always falls as k rises, so the answer is always “as many as there are points”. What you seek is where the fall flattens.",
      ),
      bi(
        "Mengukur ketepatan pada data yang dipakai melatih. Angkanya selalu terlalu optimistis; evaluasi yang sungguhan memerlukan data uji terpisah.",
        "Measuring accuracy on the training data. The number is always too optimistic; real evaluation needs a held-out test set.",
      ),
    ],
    references: [
      {
        text: "Quinlan, J. R. (1986). Induction of Decision Trees. Machine Learning, 1(1), 81–106.",
        url: "https://doi.org/10.1007/BF00116251",
      },
      {
        text: "MacQueen, J. (1967). Some methods for classification and analysis of multivariate observations.",
      },
      {
        text: "Arthur, D. & Vassilvitskii, S. (2007). k-means++: The advantages of careful seeding.",
      },
      {
        text: "Taha, K. (2025). Big Data Analytics in IoT, social media, NLP, and information security. Journal of Big Data, 12(150).",
        url: "https://doi.org/10.1186/s40537-025-01192-9",
      },
    ],
  },

  robotics: {
    summary: bi(
      "Robotika menyambungkan perhitungan dengan gerak nyata. Yang paling banyak mengajarkan sesuatu di sini adalah kegagalannya, karena kegagalan robot bersifat fisik dan tidak bisa disembunyikan.",
      "Robotics connects computation to physical motion. What teaches most here is the failure modes, because a robot's failures are physical and cannot be hidden.",
    ),
    definitions: [
      {
        term: bi("Penggerak diferensial", "Differential drive"),
        meaning: bi(
          "Susunan dua roda yang kecepatannya diatur terpisah. Selisih kecepatan menentukan arah belok, dan rerata keduanya menentukan laju majunya.",
          "A two-wheel arrangement with independently driven speeds. The difference sets the turn, the average sets the forward speed.",
        ),
      },
      {
        term: bi("Kedudukan", "Pose"),
        meaning: bi(
          "Posisi dan arah hadap robot, biasanya ditulis (x, y, θ). Sudut θ harus dinormalkan; tanpa itu selisih antara 179° dan −179° terbaca 358°, bukan 2°.",
          "The robot's position and heading, usually written (x, y, θ). The angle must be normalised; otherwise the difference between 179° and −179° reads as 358° instead of 2°.",
        ),
      },
      {
        term: bi("Kendali PID", "PID control"),
        meaning: bi(
          "Kendali yang menggabungkan galat saat ini (P), tumpukan galat masa lalu (I), dan laju perubahan galat (D). Tiga penguatan itu harus disetel bersama, tidak bisa sendiri-sendiri.",
          "A controller combining current error (P), accumulated past error (I), and the rate of change (D). The three gains must be tuned together, not separately.",
        ),
      },
      {
        term: bi("Lonjakan", "Overshoot"),
        meaning: bi(
          "Seberapa jauh sistem melampaui sasarannya sebelum kembali. Lonjakan yang besar menandakan penguatan proporsional terlalu tinggi.",
          "How far the system passes its target before returning. Large overshoot signals excessive proportional gain.",
        ),
      },
      {
        term: bi("Penumpukan integral", "Integral windup"),
        meaning: bi(
          "Bagian integral yang terus menumpuk saat keluaran sudah mentok. Akibatnya kendali sangat lambat pulih ketika galatnya akhirnya berbalik arah.",
          "The integral term continuing to accumulate while the output is saturated. The controller then recovers very slowly once the error finally reverses.",
        ),
      },
      {
        term: bi("Kinematika maju", "Forward kinematics"),
        meaning: bi(
          "Menghitung posisi ujung lengan dari sudut sendinya. Selalu punya jawaban tunggal.",
          "Computing the tip position from the joint angles. It always has a single answer.",
        ),
      },
      {
        term: bi("Kinematika balik", "Inverse kinematics"),
        meaning: bi(
          "Menghitung sudut sendi dari posisi ujung yang diinginkan. Biasanya punya lebih dari satu jawaban, dan memilih di antaranya adalah keputusan perancang.",
          "Computing joint angles from a desired tip position. It usually has more than one answer, and choosing between them is a design decision.",
        ),
      },
      {
        term: bi("Medan potensial", "Potential field"),
        meaning: bi(
          "Perencanaan lintasan dengan memperlakukan tujuan sebagai penarik dan rintangan sebagai penolak. Cepat dan sederhana, tetapi punya cacat bawaan berupa minimum lokal.",
          "Path planning by treating the goal as an attractor and obstacles as repellers. Fast and simple, but with a built-in local-minimum flaw.",
        ),
      },
    ],
    formulas: [
      {
        name: bi("Penggerak diferensial", "Differential drive"),
        expression: "v = (v_kanan + v_kiri) / 2          ω = (v_kanan − v_kiri) / L",
        note: bi(
          "L adalah jarak antarroda. Kasus ω = 0 harus dipisahkan karena rumus busurnya membagi dengan ω.",
          "L is the wheel base. The ω = 0 case must be separated because the arc formula divides by ω.",
        ),
      },
      {
        name: bi("Kendali PID", "PID control"),
        expression: "u(t) = Kp·e(t) + Ki·∫e dt + Kd·de/dt",
        note: bi(
          "Turunan pada langkah pertama tidak punya makna dan harus dianggap nol; memakai galat sebelumnya yang belum ada menghasilkan lonjakan palsu.",
          "The derivative on the first step is meaningless and must be taken as zero; using a non-existent previous error produces a spurious spike.",
        ),
      },
      {
        name: bi("Kinematika maju dua sendi", "Two-joint forward kinematics"),
        expression: "x = L₁cos θ₁ + L₂cos(θ₁+θ₂)          y = L₁sin θ₁ + L₂sin(θ₁+θ₂)",
        note: bi(
          "Sudut sendi kedua diukur relatif terhadap lengan pertama, bukan terhadap sumbu.",
          "The second joint angle is measured relative to the first link, not to the axis.",
        ),
      },
      {
        name: bi("Jangkauan lengan", "Arm reach"),
        expression: "|L₁ − L₂| ≤ jarak ≤ L₁ + L₂",
        note: bi(
          "Titik di luar rentang ini tidak punya penyelesaian sama sekali; melaporkannya sebagai galat lebih jujur daripada mengembalikan sudut yang meleset.",
          "A point outside this range has no solution at all; reporting an error is more honest than returning angles that miss.",
        ),
      },
      {
        name: bi("Gaya tolak medan potensial", "Potential-field repulsion"),
        expression: "F = k · (1/d − 1/d₀) / d²,   d < d₀",
        note: bi(
          "Hanya berlaku di dalam jari-jari pengaruh d₀, dan menguat sangat tajam saat mendekat.",
          "Applies only inside the influence radius d₀, and rises very sharply on approach.",
        ),
      },
    ],
    pitfalls: [
      bi(
        "Menyetel satu penguatan PID sendirian. Ketiganya saling mempengaruhi; menaikkan Kp tanpa menyesuaikan Kd hampir selalu menghasilkan ayunan.",
        "Tuning one PID gain in isolation. All three interact; raising Kp without adjusting Kd almost always produces oscillation.",
      ),
      bi(
        "Mengira robot yang berhenti di depan rintangan berarti ada bug. Pada medan potensial itu cacat bawaan metodenya: gaya tarik dan gaya tolak saling meniadakan tepat di titik itu.",
        "Assuming a robot stalling before an obstacle means a bug. In potential fields it is the method's built-in flaw: attraction and repulsion cancel exactly there.",
      ),
      bi(
        "Melupakan penormalan sudut. Robot akan berputar hampir satu putaran penuh untuk koreksi yang sebenarnya hanya dua derajat.",
        "Forgetting angle normalisation. The robot then turns almost a full revolution for a correction of two degrees.",
      ),
    ],
    references: [
      {
        text: "Khatib, O. (1986). Real-Time Obstacle Avoidance for Manipulators and Mobile Robots. The International Journal of Robotics Research, 5(1), 90–98.",
        url: "https://doi.org/10.1177/027836498600500106",
      },
      {
        text: "Siegwart, R. & Nourbakhsh, I. (2011). Introduction to Autonomous Mobile Robots, 2nd ed.",
      },
    ],
  },
};

/** Catatan untuk sebuah slug, bila ada. */
export function notesFor(slug: string): LabNotes | undefined {
  return NOTES[slug];
}
