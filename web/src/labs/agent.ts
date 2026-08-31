/**
 * Laboratorium Sesi 2 — Agen Cerdas, Masalah, dan Ruang Keadaan.
 *
 * Empat jenis agen dijalankan pada dunia yang sama sehingga perbedaannya
 * muncul sebagai angka, bukan sebagai definisi di buku. Ditambah dua masalah
 * ruang keadaan klasik yang bisa diubah parameternya.
 */

import * as engine from "../engine.js";
import { T, bi, pick } from "../i18n.js";
import { buttonRow, card, clear, el, errorNote, fmt, slider, table } from "../ui.js";
import { canvasSvg, cycle, figure, rankedBars, svg, svgText } from "../viz.js";

/**
 * Deretan pasangan teko beserta tinggi airnya di tiap langkah.
 *
 * Isi teko adalah besaran yang paling mudah dibaca sebagai tinggi, dan justru
 * di situlah polanya terlihat: penyelesaian teko air selalu berupa satu teko
 * yang berulang kali penuh lalu dituang, sementara yang lain menampung sisanya.
 * Tabel angka memuat data yang sama tetapi menyembunyikan polanya.
 */
function stripTeko(
  langkah: engine.JugStep[],
  kapA: number,
  kapB: number,
  sasaran: number,
): SVGSVGElement {
  const kolom = 46;
  const tinggi = 74;
  const W = Math.max(240, langkah.length * kolom + 30);
  const H = tinggi + 44;
  const root = canvasSvg(W, H);

  langkah.forEach((s, i) => {
    const x = 16 + i * kolom;
    const gambarTeko = (dx: number, isi: number, kap: number): void => {
      const h = kap === 0 ? 0 : (isi / kap) * tinggi;
      root.append(
        svg("rect", {
          x: x + dx,
          y: 8,
          width: 15,
          height: tinggi,
          rx: 3,
          fill: "none",
          stroke: "var(--border-strong)",
        }),
        svg("rect", {
          x: x + dx,
          y: 8 + tinggi - h,
          width: 15,
          height: h,
          rx: 3,
          // Teko yang isinya persis sasaran diberi warna aksen, sehingga
          // langkah tempat masalahnya selesai langsung terlihat.
          fill: isi === sasaran && isi > 0 ? "var(--accent)" : "var(--text-faint)",
          opacity: isi === sasaran && isi > 0 ? 0.85 : 0.5,
        }),
      );
    };
    gambarTeko(0, s.a, kapA);
    gambarTeko(19, s.b, kapB);
    root.append(
      svgText(x + 17, tinggi + 22, `${s.a}/${s.b}`, {
        "text-anchor": "middle",
        "font-size": 9,
        "font-family": "var(--font-mono)",
      }),
      svgText(x + 17, tinggi + 34, String(i + 1), {
        "text-anchor": "middle",
        "font-size": 8,
        fill: "var(--text-faint)",
      }),
    );
  });
  return root;
}

/**
 * Kedua tepi sungai di tiap langkah, beserta letak perahunya.
 *
 * Aturan keselamatannya menyangkut perbandingan jumlah di satu tepi, dan
 * perbandingan jauh lebih mudah dilihat daripada dibaca. Sebuah langkah yang
 * hampir melanggar aturan langsung tampak sebagai dua tumpukan yang nyaris
 * sama tinggi.
 */
function stripSungai(
  langkah: engine.CrossingStep[],
  totalM: number,
  totalK: number,
): SVGSVGElement {
  const kolom = 52;
  const W = Math.max(260, langkah.length * kolom + 30);
  const H = 128;
  const root = canvasSvg(W, H);
  const titik = 9;

  langkah.forEach((s, i) => {
    const x = 16 + i * kolom;
    const gambarTepi = (y: number, m: number, k: number): void => {
      for (let j = 0; j < m; j += 1) {
        root.append(
          svg("circle", { cx: x + 6 + j * titik, cy: y, r: 3.4, fill: "var(--accent)" }),
        );
      }
      for (let j = 0; j < k; j += 1) {
        root.append(
          svg("rect", {
            x: x + 3 + j * titik,
            y: y + 8,
            width: 6.8,
            height: 6.8,
            fill: "var(--warn)",
          }),
        );
      }
    };
    gambarTepi(16, s.missionaries_left, s.cannibals_left);
    gambarTepi(78, totalM - s.missionaries_left, totalK - s.cannibals_left);

    root.append(
      svg("line", {
        x1: x,
        y1: 62,
        x2: x + kolom - 8,
        y2: 62,
        stroke: "var(--border-strong)",
        "stroke-dasharray": "2 3",
      }),
      // Perahu digambar menempel di tepi tempat ia berada; itulah satu-satunya
      // hal yang membatasi langkah berikutnya.
      svg("path", {
        d: `M ${x + 4} ${s.boat_left ? 56 : 68} h 18 l -3 6 h -12 Z`,
        fill: "var(--text-muted)",
      }),
      svgText(x + kolom / 2 - 4, H - 6, String(i + 1), {
        "text-anchor": "middle",
        "font-size": 8,
        fill: "var(--text-faint)",
      }),
    );
  });
  return root;
}

/**
 * Deretan ruangan beserta letak agen dan kotoran yang tersisa.
 *
 * Dunia penyedot debu cukup kecil untuk digambar seluruhnya, dan menggambarnya
 * seluruhnya jauh lebih jelas daripada menuliskan "posisi 3, kotor di 0 dan 4".
 * Pembaca tidak perlu menyusun ulang gambarnya di kepala.
 */
function stripDunia(kotor: boolean[], posisi: number): SVGSVGElement {
  const kotak = 54;
  const W = Math.max(200, kotor.length * kotak + 20);
  const H = 96;
  const root = canvasSvg(W, H);

  kotor.forEach((isiKotor, i) => {
    const x = 10 + i * kotak;
    root.append(
      svg("rect", {
        x,
        y: 26,
        width: kotak - 6,
        height: kotak - 6,
        rx: 7,
        fill: isiKotor ? "var(--warn)" : "var(--surface-2)",
        opacity: isiKotor ? 0.32 : 1,
        stroke: i === posisi ? "var(--accent)" : "var(--border-strong)",
        "stroke-width": i === posisi ? 2.5 : 1,
      }),
      svgText(x + (kotak - 6) / 2, 50, isiKotor ? "•••" : "", {
        "text-anchor": "middle",
        "font-size": 14,
        fill: "var(--warn)",
      }),
      svgText(x + (kotak - 6) / 2, 88, String(i), {
        "text-anchor": "middle",
        "font-size": 10,
      }),
    );
    if (i === posisi) {
      root.append(
        svgText(x + (kotak - 6) / 2, 18, "▼", {
          "text-anchor": "middle",
          "font-size": 12,
          fill: "var(--accent)",
        }),
      );
    }
  });
  return root;
}

type Tab = "vacuum" | "jug" | "missionaries";

/**
 * Memasang laboratorium ke dalam elemen yang diberikan.
 *
 * Keterangannya -- judul, nomor sesi, penjelasan -- ada di
 * `labs/registry.ts`, bukan di sini, supaya daftar isi bisa ditampilkan
 * tanpa mengunduh mesin seluruh laboratorium lebih dulu.
 */
export function mount(root: HTMLElement): () => void {
    let tab: Tab = "vacuum";
    let dirty = [true, false, true, false, true];
    let position = 0;
    let maxSteps = 60;
    let jugA = 3;
    let jugB = 5;
    let jugTarget = 4;
    let missionaries = 3;
    let cannibals = 3;
    let boat = 2;

    const controls = el("div");
    const output = el("div");

    function renderVacuum(): void {
      let runs: engine.AgentRun[];
      try {
        runs = engine.agentCompare(dirty, position, maxSteps);
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }

      const nama: Record<string, { id: string; en: string }> = {
        simple_reflex: bi("Refleks sederhana", "Simple reflex"),
        model_based: bi("Refleks bermodel", "Model-based"),
        goal_based: bi("Berbasis tujuan", "Goal-based"),
        utility_based: bi("Berbasis utilitas", "Utility-based"),
      };

      // Contoh satu putaran diambil dari langkah pertama agen berbasis tujuan:
      // ia satu-satunya yang keempat tahapnya benar-benar terisi.
      const contoh = runs.find((r) => r.kind === "goal_based")?.steps[0];

      output.append(
        card(
          pick(bi("Dunia yang dihadapi", "The world being faced")),
          figure({
            title: bi("Peta ruangan", "Room map"),
            summary: bi(
              `${dirty.filter(Boolean).length} dari ${dirty.length} ruangan kotor. ` +
                `Segitiga menandai posisi awal agen. Setiap agen di bawah menghadapi ` +
                `dunia yang sama persis ini, jadi selisih angkanya sepenuhnya berasal ` +
                `dari cara mereka memutuskan, bukan dari keberuntungan.`,
              `${dirty.filter(Boolean).length} of ${dirty.length} rooms are dirty. ` +
                `The triangle marks the agent's starting position. Every agent below faces ` +
                `exactly this world, so any difference in the numbers comes entirely from ` +
                `how they decide, not from luck.`,
            ),
            body: stripDunia(dirty, position),
            legend: [
              { color: "var(--warn)", label: bi("ruangan kotor", "dirty room") },
              { color: "var(--accent)", label: bi("posisi agen", "agent position") },
            ],
          }),
        ),
        ...(contoh
          ? [card(
              pick(bi("Satu putaran agen", "One agent cycle")),
              figure({
                title: bi("Gelang indera–putus–tindak", "Sense–decide–act loop"),
                summary: bi(
                  "Agen bukan fungsi yang dipanggil sekali, melainkan gelang yang berputar: " +
                    "tindakannya mengubah lingkungan, dan lingkungan yang sudah berubah itulah " +
                    "yang diinderanya pada putaran berikutnya. Angka di sini diambil dari " +
                    "putaran pertama agen berbasis tujuan.",
                  "An agent is not a function called once but a loop: its action changes the " +
                    "environment, and that changed environment is what it senses on the next " +
                    "turn. The values shown come from the goal-based agent's first cycle.",
                ),
                body: cycle(
                  [
                    { label: bi("Indera", "Percept"), value: contoh.perceived_dirty ? "kotor" : "bersih" },
                    { label: bi("Keadaan", "State"), value: `${pick(bi("ruang", "room"))} ${contoh.position}` },
                    { label: bi("Tindakan", "Action"), value: contoh.action },
                    { label: bi("Lingkungan", "Environment"), value: `${contoh.dirty_after} ${pick(bi("kotor", "dirty"))}` },
                  ],
                  0,
                ),
              }),
            )]
          : []),
        card(
          pick(bi("Biaya yang dibayar tiap agen", "What each agent pays")),
          figure({
            title: bi("Biaya dan gerak sia-sia", "Cost and wasted moves"),
            summary: bi(
              "Batang atas adalah biaya total tiap agen pada dunia yang sama. " +
                "Agen refleks sederhana hampir selalu paling boros, dan bukan karena " +
                "aturannya buruk: tanpa ingatan ia tidak punya cara mengetahui bahwa " +
                "pekerjaannya sudah selesai, jadi ia terus bergerak sampai dihentikan paksa.",
              "Each bar is one agent's total cost on the same world. The simple reflex agent " +
                "is almost always the most wasteful, and not because its rules are bad: with " +
                "no memory it has no way to know its work is done, so it keeps moving until " +
                "forced to stop.",
            ),
            body: rankedBars(
              runs.map((r) => ({
                label: pick(nama[r.kind] ?? bi(r.kind, r.kind)),
                value: r.cost,
                highlight: r.cost === Math.min(...runs.map((x) => x.cost)),
                detail: `${r.wasted_moves} ${pick(bi("sia-sia", "wasted"))}`,
              })),
              (v) => fmt(v, 0),
            ),
            legend: [
              { color: "var(--accent)", label: bi("paling hemat", "cheapest") },
              { color: "var(--text-faint)", label: bi("lainnya", "others") },
            ],
          }),
        ),
        card(
          pick(bi("Empat agen, satu dunia", "Four agents, one world")),
          table(
            [
              pick(bi("Agen", "Agent")),
              pick(bi("Ingatan", "Memory")),
              pick(bi("Selesai", "Finished")),
              pick(bi("Langkah", "Steps")),
              pick(bi("Biaya", "Cost")),
              pick(bi("Gerak sia-sia", "Wasted moves")),
            ],
            runs.map((r) => [
              pick(nama[r.kind] ?? bi(r.kind, r.kind)),
              r.kind === "simple_reflex" ? "—" : "✓",
              r.finished ? "✓" : "—",
              String(r.steps.length),
              r.cost,
              String(r.wasted_moves),
            ]),
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Kolom “gerak sia-sia” diukur dari jarak ke kotoran terdekat, bukan dari berkurangnya kotoran. Gerak menuju kotoran memang tidak mengurangi kotoran, tetapi jelas bukan pemborosan — yang sia-sia adalah gerak yang tidak mendekatkan agen kepada apa pun.",
                "The “wasted moves” column is measured by distance to the nearest dirt, not by dirt removed. A move toward dirt removes none, yet is clearly not waste — what is wasted is a move that brings the agent closer to nothing.",
              ),
            ),
          }),
        ),
      );

      // Jejak agen pertama dan terakhir, untuk dibandingkan berdampingan.
      for (const r of [runs[0], runs[2]]) {
        output.append(
          card(
            `${pick(bi("Jejak", "Trace"))}: ${pick(nama[r.kind] ?? bi(r.kind, r.kind))}`,
            table(
              [
                "#",
                pick(bi("Di ruangan", "In room")),
                pick(bi("Kotor?", "Dirty?")),
                pick(bi("Tindakan", "Action")),
                pick(bi("Sisa kotor", "Dirt left")),
              ],
              r.steps
                .slice(0, 20)
                .map((s) => [
                  String(s.step),
                  String(s.position),
                  s.perceived_dirty ? "✓" : "—",
                  s.action,
                  String(s.dirty_after),
                ]),
            ),
            r.steps.length > 20
              ? el("p", {
                  class: "note",
                  text: pick(
                    bi(
                      `Dipotong pada 20 dari ${r.steps.length} langkah.`,
                      `Truncated at 20 of ${r.steps.length} steps.`,
                    ),
                  ),
                })
              : null,
          ),
        );
      }
    }

    function renderJug(): void {
      let steps: engine.JugStep[];
      try {
        steps = engine.agentWaterJug(jugA, jugB, jugTarget);
      } catch (error) {
        output.append(
          errorNote(String((error as Error).message)),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Keterjangkauan diperiksa lebih dulu memakai teorema Bézout: sasaran hanya bisa dicapai bila ia kelipatan pembagi bersama terbesar kedua kapasitas. Memeriksanya di muka jauh lebih jujur daripada membiarkan pencarian berjalan lalu melaporkan “tidak ditemukan” — dua hal itu terlihat sama di layar tetapi berbeda maknanya.",
                "Reachability is checked up front using Bézout's theorem: the target is attainable only if it is a multiple of the greatest common divisor of the two capacities. Checking first is far more honest than letting the search run and reporting “not found” — the two look identical on screen but mean different things.",
              ),
            ),
          }),
        );
        return;
      }

      output.append(
        ...(steps.length > 0
          ? [
              card(
                pick(bi("Isi teko di tiap langkah", "Jug levels at each step")),
                figure({
                  title: bi("Tinggi air kedua teko", "Water level in both jugs"),
                  summary: bi(
                    `${steps.length} langkah membawa teko dari kosong ke sasaran ${jugTarget} liter. ` +
                      `Batang tersorot adalah teko yang isinya persis sasaran. Perhatikan polanya: ` +
                      `satu teko berulang kali diisi penuh lalu dituang, dan sisa yang tak tertampung ` +
                      `itulah yang perlahan menyusun jawabannya.`,
                    `${steps.length} steps take the jugs from empty to the ${jugTarget}-litre target. ` +
                      `The highlighted bar is the jug holding exactly the target. Watch the pattern: ` +
                      `one jug is filled and poured out again and again, and the remainder that will ` +
                      `not fit is what slowly builds the answer.`,
                  ),
                  body: stripTeko(steps, jugA, jugB, jugTarget),
                  legend: [
                    { color: "var(--accent)", label: bi("isi sama dengan sasaran", "level equals target") },
                    { color: "var(--text-faint)", label: bi("isi lain", "other levels") },
                  ],
                }),
              ),
            ]
          : []),
        card(
          pick(T.result),
          steps.length > 0
            ? table(
                [
                  "#",
                  pick(bi("Tindakan", "Action")),
                  `A (${jugA})`,
                  `B (${jugB})`,
                ],
                steps.map((s, i) => [String(i + 1), s.action, String(s.a), String(s.b)]),
              )
            : el("p", {
                class: "note",
                text: pick(
                  bi(
                    "Sasaran nol sudah tercapai sejak awal; tidak ada langkah yang diperlukan.",
                    "A target of zero is already satisfied; no steps needed.",
                  ),
                ),
              }),
          el("p", {
            class: "note",
            text: pick(
              bi(
                `Diselesaikan dalam ${steps.length} langkah dengan pencarian melebar, sehingga inilah jalur terpendek yang mungkin.`,
                `Solved in ${steps.length} steps by breadth-first search, so this is the shortest possible sequence.`,
              ),
            ),
          }),
        ),
      );
    }

    function renderMissionaries(): void {
      let steps: engine.CrossingStep[];
      try {
        steps = engine.agentMissionaries(missionaries, cannibals, boat);
      } catch (error) {
        output.append(
          errorNote(String((error as Error).message)),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Sebagian susunan memang tidak punya penyelesaian yang aman sama sekali. Empat berbanding empat dengan perahu berkapasitas dua adalah contohnya — cobalah, lalu naikkan kapasitas perahunya.",
                "Some configurations have no safe solution at all. Four against four with a two-seat boat is one — try it, then raise the boat capacity.",
              ),
            ),
          }),
        );
        return;
      }

      output.append(
        card(
          pick(bi("Kedua tepi di tiap langkah", "Both banks at each step")),
          figure({
            title: bi("Peta penyeberangan", "Crossing map"),
            summary: bi(
              `Tepi kiri di atas garis, tepi kanan di bawahnya, perahu menempel di tepi ` +
                `tempatnya berada. Aturan keselamatannya menyangkut perbandingan jumlah di ` +
                `satu tepi, jadi yang layak diperhatikan bukan langkah mana yang aman ` +
                `melainkan seberapa tipis jaraknya: penyelesaian ${steps.length} langkah ini ` +
                `berkali-kali berjalan tepat di batas.`,
              `The left bank is above the line, the right bank below it, and the boat sits on ` +
                `whichever side it is moored. The safety rule is about the ratio on one bank, so ` +
                `what matters is not which steps are safe but how narrow the margin is: this ` +
                `${steps.length}-step solution repeatedly runs right along the edge.`,
            ),
            body: stripSungai(steps, missionaries, cannibals),
            legend: [
              { color: "var(--accent)", label: bi("misionaris", "missionary") },
              { color: "var(--warn)", label: bi("kanibal", "cannibal") },
              { color: "var(--text-muted)", label: bi("perahu", "boat") },
            ],
          }),
        ),
        card(
          pick(T.result),
          table(
            [
              "#",
              pick(bi("Penyeberangan", "Crossing")),
              pick(bi("Misionaris kiri", "Missionaries left")),
              pick(bi("Kanibal kiri", "Cannibals left")),
              pick(bi("Perahu", "Boat")),
            ],
            steps.map((s, i) => [
              String(i + 1),
              s.action,
              String(s.missionaries_left),
              String(s.cannibals_left),
              s.boat_left ? pick(bi("kiri", "left")) : pick(bi("kanan", "right")),
            ]),
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                `Selesai dalam ${steps.length} penyeberangan. Aturannya bukan “kanibal selalu lebih sedikit”, melainkan “kanibal tidak boleh lebih banyak dari misionaris yang hadir” — tepi tanpa misionaris selalu aman berapa pun kanibalnya, dan perbedaan itu menentukan ada tidaknya penyelesaian.`,
                `Solved in ${steps.length} crossings. The rule is not “cannibals must always be fewer” but “cannibals must not outnumber the missionaries present” — a bank with no missionaries is always safe however many cannibals stand on it, and that distinction decides whether a solution exists at all.`,
              ),
            ),
          }),
        ),
      );
    }

    function render(): void {
      clear(output);
      switch (tab) {
        case "vacuum":
          renderVacuum();
          break;
        case "jug":
          renderJug();
          break;
        case "missionaries":
          renderMissionaries();
          break;
      }
    }

    function renderControls(): void {
      clear(controls);

      const tabs = buttonRow(
        (
          [
            ["vacuum", bi("Dunia penyedot debu", "Vacuum world")],
            ["jug", bi("Teko air", "Water jug")],
            ["missionaries", bi("Misionaris & kanibal", "Missionaries & cannibals")],
          ] as [Tab, { id: string; en: string }][]
        ).map(([t, label]) => ({
          label: pick(label),
          primary: t === tab,
          onClick: () => {
            tab = t;
            renderControls();
            render();
          },
        })),
      );

      const extras: HTMLElement[] = [];

      if (tab === "vacuum") {
        extras.push(
          card(
            pick(bi("Ruangan", "Rooms")),
            buttonRow(
              dirty.map((d, i) => ({
                label: `${i}${d ? " ●" : " ○"}`,
                primary: i === position,
                onClick: () => {
                  dirty[i] = !dirty[i];
                  renderControls();
                  render();
                },
              })),
            ),
            el("p", {
              class: "note",
              text: pick(
                bi(
                  "Klik untuk mengubah kotor dan bersih. Bulatan penuh berarti kotor; tombol bersorot adalah posisi awal agen.",
                  "Click to toggle dirty and clean. A filled circle means dirty; the highlighted button is the agent's starting room.",
                ),
              ),
            }),
            slider({
              label: pick(bi("Posisi awal", "Starting room")),
              min: 0,
              max: Math.max(0, dirty.length - 1),
              step: 1,
              value: Math.min(position, dirty.length - 1),
              format: (v) => String(v),
              onInput: (v) => {
                position = v;
                renderControls();
                render();
              },
            }),
            slider({
              label: pick(bi("Batas langkah", "Step limit")),
              min: 10,
              max: 200,
              step: 10,
              value: maxSteps,
              format: (v) => String(v),
              onInput: (v) => {
                maxSteps = v;
                render();
              },
            }),
            buttonRow([
              {
                label: pick(bi("Tambah ruangan", "Add room")),
                onClick: () => {
                  if (dirty.length < 12) dirty.push(true);
                  renderControls();
                  render();
                },
              },
              {
                label: pick(bi("Kurangi ruangan", "Remove room")),
                onClick: () => {
                  if (dirty.length > 2) dirty.pop();
                  if (position >= dirty.length) position = dirty.length - 1;
                  renderControls();
                  render();
                },
              },
              {
                label: pick(bi("Semua bersih", "All clean")),
                onClick: () => {
                  dirty = dirty.map(() => false);
                  renderControls();
                  render();
                },
              },
            ]),
          ),
        );
      }

      if (tab === "jug") {
        extras.push(
          card(
            pick(T.controls),
            slider({
              label: pick(bi("Kapasitas teko A", "Jug A capacity")),
              min: 1,
              max: 12,
              step: 1,
              value: jugA,
              format: (v) => String(v),
              onInput: (v) => {
                jugA = v;
                render();
              },
            }),
            slider({
              label: pick(bi("Kapasitas teko B", "Jug B capacity")),
              min: 1,
              max: 12,
              step: 1,
              value: jugB,
              format: (v) => String(v),
              onInput: (v) => {
                jugB = v;
                render();
              },
            }),
            slider({
              label: pick(bi("Sasaran", "Target")),
              min: 0,
              max: 12,
              step: 1,
              value: jugTarget,
              format: (v) => String(v),
              onInput: (v) => {
                jugTarget = v;
                render();
              },
            }),
            el("p", {
              class: "note",
              text: pick(
                bi(
                  "Coba kapasitas 2 dan 4 dengan sasaran 3: mustahil, dan alasannya dijelaskan, bukan sekadar dilaporkan gagal.",
                  "Try capacities 2 and 4 with target 3: impossible, and the reason is explained rather than merely reported as a failure.",
                ),
              ),
            }),
          ),
        );
      }

      if (tab === "missionaries") {
        extras.push(
          card(
            pick(T.controls),
            slider({
              label: pick(bi("Misionaris", "Missionaries")),
              min: 1,
              max: 6,
              step: 1,
              value: missionaries,
              format: (v) => String(v),
              onInput: (v) => {
                missionaries = v;
                render();
              },
            }),
            slider({
              label: pick(bi("Kanibal", "Cannibals")),
              min: 1,
              max: 6,
              step: 1,
              value: cannibals,
              format: (v) => String(v),
              onInput: (v) => {
                cannibals = v;
                render();
              },
            }),
            slider({
              label: pick(bi("Kapasitas perahu", "Boat capacity")),
              min: 2,
              max: 5,
              step: 1,
              value: boat,
              format: (v) => String(v),
              onInput: (v) => {
                boat = v;
                render();
              },
            }),
          ),
        );
      }

      controls.append(card(pick(bi("Bagian", "Section")), tabs), ...extras);
    }

    root.append(el("div", { class: "grid-2", children: [controls, output] }));
    renderControls();
    render();

    return () => {
      clear(root);
    };
}
