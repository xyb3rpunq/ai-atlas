/**
 * Laboratorium Sesi 2 — Agen Cerdas, Masalah, dan Ruang Keadaan.
 *
 * Empat jenis agen dijalankan pada dunia yang sama sehingga perbedaannya
 * muncul sebagai angka, bukan sebagai definisi di buku. Ditambah dua masalah
 * ruang keadaan klasik yang bisa diubah parameternya.
 */

import * as engine from "../engine.js";
import { T, bi, pick } from "../i18n.js";
import { buttonRow, card, clear, el, errorNote, slider, table } from "../ui.js";
import type { Lab } from "./registry.js";

type Tab = "vacuum" | "jug" | "missionaries";

export const agentLab: Lab = {
  slug: "agents",
  session: 2,
  title: bi("Agen Cerdas & Ruang Keadaan", "Agents & State Space"),
  blurb: bi(
    "Empat jenis agen pada dunia yang sama. Yang membedakannya bukan kecanggihan, melainkan seberapa banyak yang mereka ingat: agen tanpa ingatan tidak punya cara mengetahui bahwa pekerjaannya sudah selesai, jadi ia terus bergerak sampai dihentikan paksa.",
    "Four kinds of agent on one world. What separates them is not sophistication but how much they remember: an agent without memory has no way to know its work is done, so it keeps moving until forced to stop.",
  ),

  mount(root: HTMLElement): () => void {
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

      output.append(
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
  },
};
