/**
 * Laboratorium Sesi 1 — Pengantar Kecerdasan Buatan.
 *
 * ELIZA (Weizenbaum, 1966), dengan mesinnya dibiarkan terbuka.
 *
 * Setiap balasan disertai aturan mana yang menang, berapa keutamaannya, dan
 * bagian mana dari kalimat pengguna yang dipantulkan kembali. Itu disengaja:
 * ilusi ELIZA jauh lebih tipis kalau mesinnya kelihatan, dan justru
 * kesenjangan antara betapa sederhana mesinnya dengan betapa kuat kesan yang
 * ditimbulkannya itulah pelajaran sebenarnya dari sesi pertama.
 */

import * as engine from "../engine.js";
import { bi, pick } from "../i18n.js";
import { buttonRow, card, clear, el, errorNote, table } from "../ui.js";
import { figure, pipeline, rankedBars } from "../viz.js";

/** Satu giliran percakapan. */
interface Turn {
  user: string;
  reply: engine.ElizaReply;
}

const SUGGESTIONS = [
  "Halo",
  "Saya merasa lelah akhir-akhir ini",
  "Saya ingin pindah kerja",
  "Ibu saya sering mengkhawatirkan saya",
  "Saya tidak bisa tidur nyenyak",
  "Kenapa semuanya terasa berat",
  "Cuaca hari ini cerah sekali",
];

/**
 * Memasang laboratorium ke dalam elemen yang diberikan.
 *
 * Keterangannya -- judul, nomor sesi, penjelasan -- ada di
 * `labs/registry.ts`, bukan di sini, supaya daftar isi bisa ditampilkan
 * tanpa mengunduh mesin seluruh laboratorium lebih dulu.
 */
export function mount(root: HTMLElement): () => void {
    let turns: Turn[] = [];
    let seed = 1;

    const summary = engine.elizaScriptSummary();
    const script = engine.elizaScript();

    const controls = el("div");
    const output = el("div");

    function send(text: string): void {
      const trimmed = text.trim();
      if (trimmed.length === 0) return;
      try {
        const reply = engine.elizaRespond(trimmed, seed);
        turns.push({ user: trimmed, reply });
        seed += 1;
      } catch (error) {
        clear(output);
        output.append(errorNote(String((error as Error).message)));
        return;
      }
      render();
    }

    function render(): void {
      clear(output);

      if (turns.length > 0) {
        output.append(
          card(
            pick(bi("Percakapan", "Conversation")),
            table(
              [
                pick(bi("Anda", "You")),
                "ELIZA",
                pick(bi("Aturan", "Rule")),
                pick(bi("Keutamaan", "Priority")),
              ],
              turns.map((t) => [
                t.user,
                t.reply.text,
                t.reply.used_fallback
                  ? pick(bi("(cadangan)", "(fallback)"))
                  : t.reply.matched_keyword,
                t.reply.used_fallback ? "—" : String(t.reply.priority),
              ]),
            ),
          ),
        );

        const terakhirGiliran = turns[turns.length - 1];
        const terakhir = terakhirGiliran.reply;
        output.append(
          card(
            pick(bi("Bagaimana balasan terakhir dibuat", "How the last reply was made")),
            figure({
              title: bi("Empat tahap dari kalimat ke balasan", "Four stages from sentence to reply"),
              summary: terakhir.used_fallback
                ? bi(
                    "Tahap pencocokan gagal, jadi tiga tahap sisanya dilewati dan ELIZA " +
                      "mengambil kalimat cadangan begitu saja. Kalimat cadangan inilah yang " +
                      "paling sering terasa mendalam — padahal ia sama sekali tidak melihat " +
                      "apa yang Anda tulis.",
                    "The matching stage failed, so the remaining three stages were skipped and " +
                      "ELIZA simply took a stock line. Those stock lines are what most often feel " +
                      "profound — even though they never looked at what you wrote.",
                  )
                : bi(
                    `Kalimat Anda dicocokkan dengan kata kunci "${terakhir.matched_keyword}" ` +
                      `berkeutamaan ${terakhir.priority}, potongannya diambil, kata gantinya ` +
                      `ditukar, lalu dimasukkan ke templat. Tidak ada tahap yang memahami apa pun; ` +
                      `seluruhnya hanya pencocokan pola dan penukaran kata.`,
                    `Your sentence matched the keyword "${terakhir.matched_keyword}" with priority ` +
                      `${terakhir.priority}, a fragment was taken, its pronouns were swapped, and ` +
                      `the result was slotted into a template. No stage understands anything; it is ` +
                      `all pattern matching and word substitution.`,
                  ),
              body: pipeline([
                {
                  label: bi("Masukan", "Input"),
                  value: terakhirGiliran.user,
                },
                {
                  label: bi("Cocokkan kata kunci", "Match keyword"),
                  value: terakhir.matched_keyword || pick(bi("tidak ada yang cocok", "no match")),
                  note: terakhir.used_fallback
                    ? undefined
                    : pick(bi(`keutamaan ${terakhir.priority}`, `priority ${terakhir.priority}`)),
                  skipped: terakhir.used_fallback,
                },
                {
                  label: bi("Pantulkan potongan", "Reflect fragment"),
                  value: terakhir.reflected_fragment || "—",
                  note: pick(bi("kata ganti ditukar", "pronouns swapped")),
                  skipped: terakhir.used_fallback || terakhir.reflected_fragment.length === 0,
                },
                {
                  label: bi("Susun balasan", "Assemble reply"),
                  value: terakhir.text,
                  note: pick(
                    terakhir.used_fallback
                      ? bi("dari daftar cadangan", "from the fallback list")
                      : bi("dari templat aturan", "from the rule template"),
                  ),
                },
              ]),
            }),
            table(
              [pick(bi("Tahap", "Stage")), pick(bi("Hasil", "Result"))],
              [
                [
                  pick(bi("Kata kunci cocok", "Keyword matched")),
                  terakhir.matched_keyword || pick(bi("tidak ada", "none")),
                ],
                [
                  pick(bi("Keutamaan aturan", "Rule priority")),
                  terakhir.used_fallback ? "—" : String(terakhir.priority),
                ],
                [
                  pick(bi("Potongan dipantulkan", "Reflected fragment")),
                  terakhir.reflected_fragment || "—",
                ],
                [
                  pick(bi("Memakai cadangan", "Used fallback")),
                  terakhir.used_fallback
                    ? pick(bi("ya", "yes"))
                    : pick(bi("tidak", "no")),
                ],
              ],
            ),
            el("p", {
              class: "note",
              text: terakhir.used_fallback
                ? pick(
                    bi(
                      "Tidak ada kata kunci yang cocok, jadi ELIZA memakai kalimat cadangan. Kalimat cadangan inilah yang paling sering membuat percakapan terasa mendalam, padahal ia sama sekali tidak bergantung pada apa yang Anda tulis.",
                      "No keyword matched, so ELIZA used a stock line. These stock lines are what most often make a conversation feel profound, even though they do not depend on what you wrote at all.",
                    ),
                  )
                : pick(
                    bi(
                      "Potongan kalimat Anda dipantulkan kembali setelah kata gantinya ditukar. Tidak ada pemahaman di dalamnya, hanya satu tabel penukaran dan satu templat kalimat.",
                      "Your sentence fragment is reflected back with pronouns swapped. There is no understanding in it, only one substitution table and one sentence template.",
                    ),
                  ),
            }),
          ),
        );
      }

      output.append(
        card(
          pick(bi("Seluruh isi otaknya", "The entire brain")),
          table(
            [pick(bi("Ukuran", "Measure")), pick(bi("Nilai", "Value"))],
            [
              [pick(bi("Jumlah aturan", "Rules")), String(summary.rules)],
              [
                pick(bi("Kalimat balasan", "Response sentences")),
                String(summary.total_responses),
              ],
              [pick(bi("Kalimat cadangan", "Fallback lines")), String(summary.fallbacks)],
              [
                pick(bi("Pasangan penukaran", "Reflection pairs")),
                String(summary.reflections),
              ],
            ],
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                `Seluruh “kecerdasan” ELIZA muat dalam ${summary.total_responses + summary.fallbacks} kalimat yang ditulis manusia dan ${summary.reflections} pasangan kata ganti. Uji Turing lebih banyak berbicara tentang manusia yang menilai daripada tentang mesin yang dinilai.`,
                `The whole of ELIZA's "intelligence" fits in ${summary.total_responses + summary.fallbacks} human-written sentences and ${summary.reflections} pronoun pairs. The Turing test says more about the human judging than the machine being judged.`,
              ),
            ),
          }),
        ),
        card(
          pick(bi("Aturan dan keutamaannya", "Rules and priorities")),
          figure({
            title: bi("Urutan keutamaan aturan", "Rule priority order"),
            summary: bi(
              `Aturan berkeutamaan lebih tinggi diperiksa lebih dulu dan menang. ` +
                `Itulah satu-satunya "penalaran" yang dimiliki ELIZA: ${script.rules.length} angka ` +
                `yang menentukan siapa mengalahkan siapa. Tanpa penomoran ini, aturan paling ` +
                `umum akan selalu menang dan percakapannya langsung terasa hambar.`,
              `Higher-priority rules are checked first and win. That is the whole of ELIZA's ` +
                `"reasoning": ${script.rules.length} numbers deciding who beats whom. Without them, ` +
                `the most general rule would always win and the conversation would fall flat.`,
            ),
            body: rankedBars(
              script.rules
                .slice()
                .sort((a, b) => b.priority - a.priority)
                .map((r, i) => ({
                  label: r.keyword,
                  value: r.priority,
                  highlight: i === 0,
                  detail: `${r.responses.length} ${pick(bi("balasan", "replies"))}`,
                })),
              (v) => String(v),
            ),
          }),
          table(
            [
              pick(bi("Kata kunci", "Keyword")),
              pick(bi("Keutamaan", "Priority")),
              pick(bi("Balasan", "Responses")),
            ],
            script.rules
              .slice()
              .sort((a, b) => b.priority - a.priority)
              .map((r) => [r.keyword, r.priority, r.responses.join(" / ")]),
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Aturan berkeutamaan lebih tinggi menang. Itu sebabnya “saya merasa” mengalahkan “saya” — tanpa penomoran itu, ELIZA selalu menjawab dengan aturan paling umum dan percakapannya langsung terasa hambar.",
                "Higher-priority rules win. That is why “I feel” beats “I” — without those numbers, ELIZA would always answer with the most general rule and the conversation would fall flat immediately.",
              ),
            ),
          }),
        ),
      );
    }

    function renderControls(): void {
      clear(controls);

      const input = el("input", {
        attrs: {
          type: "text",
          placeholder: pick(bi("Tulis sesuatu…", "Type something…")),
          "aria-label": pick(bi("Pesan Anda", "Your message")),
        },
      });
      input.addEventListener("keydown", (event) => {
        if ((event as KeyboardEvent).key === "Enter") {
          send(input.value);
          input.value = "";
        }
      });

      controls.append(
        card(
          pick(bi("Bicara dengan ELIZA", "Talk to ELIZA")),
          el("div", { class: "field", children: [input] }),
          buttonRow([
            {
              label: pick(bi("Kirim", "Send")),
              primary: true,
              onClick: () => {
                send(input.value);
                input.value = "";
              },
            },
            {
              label: pick(bi("Mulai ulang", "Restart")),
              onClick: () => {
                turns = [];
                render();
              },
            },
          ]),
        ),
        card(
          pick(bi("Coba kalimat ini", "Try these")),
          buttonRow(
            SUGGESTIONS.map((s) => ({
              label: s,
              onClick: () => send(s),
            })),
          ),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Perhatikan kalimat terakhir: ia tidak memuat kata kunci apa pun, jadi ELIZA hanya bisa menjawab dengan kalimat cadangan.",
                "Note the last one: it contains no keyword at all, so ELIZA can only answer with a stock line.",
              ),
            ),
          }),
        ),
      );
    }

    root.append(el("div", { class: "grid-2", children: [controls, output] }));
    renderControls();
    render();

    return () => {
      clear(root);
    };
}
