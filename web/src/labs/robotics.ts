/**
 * Laboratorium Sesi 14 — Pengenalan Robotika.
 *
 * Tiga bagian, dan dua di antaranya justru memperagakan kegagalan:
 *
 * - **Kendali PID** — penguatan yang salah tidak sekadar lambat, ia membuat
 *   sistem berayun makin lebar.
 * - **Lengan dua sendi** — satu titik punya dua penyelesaian sudut, dan
 *   memilih salah satunya adalah keputusan perancang, bukan matematika.
 * - **Medan potensial** — cepat dan sederhana, tetapi punya cacat bawaan
 *   berupa minimum lokal tempat robot berhenti di depan rintangan padahal
 *   tujuannya terlihat jelas.
 */

import * as engine from "../engine.js";
import { T, bi, pick } from "../i18n.js";
import { buttonRow, card, clear, el, errorNote, fmt, slider, table } from "../ui.js";

type Tab = "pid" | "arm" | "path";

/**
 * Memasang laboratorium ke dalam elemen yang diberikan.
 *
 * Keterangannya -- judul, nomor sesi, penjelasan -- ada di
 * `labs/registry.ts`, bukan di sini, supaya daftar isi bisa ditampilkan
 * tanpa mengunduh mesin seluruh laboratorium lebih dulu.
 */
export function mount(root: HTMLElement): () => void {
    let tab: Tab = "pid";
    let kp = 1.2;
    let ki = 0.4;
    let kd = 0.2;
    let setpoint = 10;
    let theta1 = 0.6;
    let theta2 = 0.8;
    let length1 = 2;
    let length2 = 1.5;
    let repulsive = 1.5;
    let obstacleY = 0.4;

    const controls = el("div");
    const output = el("div");
    const canvas = el("canvas", {
      attrs: {
        role: "img",
        "aria-label": pick(
          bi("Peragaan grafis bagian yang aktif.", "Graphical view of the active section."),
        ),
      },
    });

    function ctx2d(w: number, h: number): CanvasRenderingContext2D | null {
      const dpr = Math.min(globalThis.devicePixelRatio || 1, 2);
      canvas.width = Math.round(w * dpr);
      canvas.height = Math.round(h * dpr);
      canvas.style.aspectRatio = `${w} / ${h}`;
      const c = canvas.getContext("2d");
      if (!c) return null;
      c.setTransform(dpr, 0, 0, dpr, 0, 0);
      const style = getComputedStyle(document.documentElement);
      c.fillStyle = style.getPropertyValue("--bg-grid").trim() || "#0d131c";
      c.fillRect(0, 0, w, h);
      return c;
    }

    function accent(): string {
      return getComputedStyle(document.documentElement).getPropertyValue("--accent").trim() ||
        "#4dd4c8";
    }

    function drawPid(run: engine.ControlRun): void {
      const W = 460;
      const H = 220;
      const c = ctx2d(W, H);
      if (!c || run.steps.length < 2) return;

      const values = run.steps.map((s) => s.value);
      const lo = Math.min(0, ...values, setpoint);
      const hi = Math.max(...values, setpoint);
      const span = Math.max(hi - lo, 1e-6);
      const x = (i: number) => (i / (run.steps.length - 1)) * (W - 10) + 5;
      const y = (v: number) => H - 10 - ((v - lo) / span) * (H - 20);

      // Garis sasaran.
      const style = getComputedStyle(document.documentElement);
      c.strokeStyle = style.getPropertyValue("--border-strong").trim() || "#2c3d54";
      c.setLineDash([4, 4]);
      c.beginPath();
      c.moveTo(5, y(setpoint));
      c.lineTo(W - 5, y(setpoint));
      c.stroke();
      c.setLineDash([]);

      c.strokeStyle = accent();
      c.lineWidth = 2;
      c.beginPath();
      run.steps.forEach((s, i) => {
        if (i === 0) c.moveTo(x(i), y(s.value));
        else c.lineTo(x(i), y(s.value));
      });
      c.stroke();
    }

    function drawArm(x: number, y: number, elbowX: number, elbowY: number): void {
      const SIZE = 460;
      const c = ctx2d(SIZE, SIZE);
      if (!c) return;
      const reach = length1 + length2;
      const toPx = (v: number) => (v / (reach * 2.2)) * SIZE + SIZE / 2;

      const style = getComputedStyle(document.documentElement);
      // Lingkaran jangkauan.
      c.strokeStyle = style.getPropertyValue("--border").trim() || "#1f2b3c";
      c.beginPath();
      c.arc(SIZE / 2, SIZE / 2, (reach / (reach * 2.2)) * SIZE, 0, Math.PI * 2);
      c.stroke();

      c.strokeStyle = accent();
      c.lineWidth = 4;
      c.lineCap = "round";
      c.beginPath();
      c.moveTo(SIZE / 2, SIZE / 2);
      c.lineTo(toPx(elbowX), SIZE - toPx(elbowY));
      c.lineTo(toPx(x), SIZE - toPx(y));
      c.stroke();

      for (const [px, py] of [
        [SIZE / 2, SIZE / 2],
        [toPx(elbowX), SIZE - toPx(elbowY)],
        [toPx(x), SIZE - toPx(y)],
      ]) {
        c.beginPath();
        c.arc(px, py, 5, 0, Math.PI * 2);
        c.fillStyle = accent();
        c.fill();
      }
    }

    function drawPath(path: engine.PotentialPath, obstacleYValue: number): void {
      const W = 460;
      const H = 260;
      const c = ctx2d(W, H);
      if (!c) return;
      const toPxX = (v: number) => (v / 10) * (W - 20) + 10;
      const toPxY = (v: number) => H / 2 - (v / 5) * (H / 2 - 10);

      const style = getComputedStyle(document.documentElement);
      c.fillStyle = style.getPropertyValue("--warn").trim() || "#f0b429";
      c.globalAlpha = 0.3;
      c.beginPath();
      c.arc(toPxX(5), toPxY(obstacleYValue), (2 / 10) * (W - 20), 0, Math.PI * 2);
      c.fill();
      c.globalAlpha = 1;

      c.strokeStyle = accent();
      c.lineWidth = 2.5;
      c.beginPath();
      path.points.forEach(([px, py], i) => {
        if (i === 0) c.moveTo(toPxX(px), toPxY(py));
        else c.lineTo(toPxX(px), toPxY(py));
      });
      c.stroke();

      c.fillStyle = style.getPropertyValue("--danger").trim() || "#f2686c";
      c.beginPath();
      c.arc(toPxX(9), toPxY(0), 6, 0, Math.PI * 2);
      c.fill();
    }

    function renderPid(): void {
      let run: engine.ControlRun;
      try {
        run = engine.roboticsPid(kp, ki, kd, 1000, setpoint, 2, 400);
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }
      drawPid(run);

      output.append(
        card(
          pick(T.result),
          table(
            [pick(bi("Ukuran", "Measure")), pick(bi("Nilai", "Value"))],
            [
              [
                pick(bi("Menetap", "Settled")),
                run.settled ? pick(bi("ya", "yes")) : pick(bi("tidak", "no")),
              ],
              [
                pick(bi("Waktu menetap", "Settling time")),
                run.settling_time === null ? "—" : `${fmt(run.settling_time, 2)} s`,
              ],
              [pick(bi("Lonjakan", "Overshoot")), `${fmt(run.overshoot_percent, 1)}%`],
              [pick(bi("Galat akhir", "Final error")), run.final_error],
            ],
          ),
          el("p", {
            class: "note",
            text:
              run.overshoot_percent > 30
                ? pick(
                    bi(
                      "Lonjakannya besar. Penguatan proporsional yang terlalu tinggi membuat sistem melampaui sasaran, lalu mengoreksi berlebihan ke arah sebaliknya — dan siklus itu berulang makin lebar.",
                      "The overshoot is large. Too much proportional gain drives the system past the target, then over-corrects the other way — and the cycle widens.",
                    ),
                  )
                : pick(
                    bi(
                      "Naikkan penguatan proporsional sampai di atas dua puluh dan perhatikan grafiknya mulai berayun. Kendali PID yang salah setel tidak sekadar lambat.",
                      "Push the proportional gain above twenty and watch the curve begin to oscillate. A mistuned PID is not merely slow.",
                    ),
                  ),
          }),
        ),
      );
    }

    function renderArm(): void {
      let forward: engine.ForwardKinematics;
      try {
        forward = engine.roboticsForward(theta1, theta2, length1, length2);
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }
      drawArm(forward.x, forward.y, forward.elbow_x, forward.elbow_y);

      let inverse: engine.ArmAngles[] | null = null;
      try {
        inverse = engine.roboticsInverse(forward.x, forward.y, length1, length2);
      } catch {
        inverse = null;
      }

      output.append(
        card(
          pick(bi("Kinematika maju", "Forward kinematics")),
          table(
            [pick(bi("Besaran", "Quantity")), pick(bi("Nilai", "Value"))],
            [
              [pick(bi("Sudut pangkal", "Base angle")), `${fmt((theta1 * 180) / Math.PI, 1)}°`],
              [pick(bi("Sudut siku", "Elbow angle")), `${fmt((theta2 * 180) / Math.PI, 1)}°`],
              [pick(bi("Ujung x", "Tip x")), forward.x],
              [pick(bi("Ujung y", "Tip y")), forward.y],
            ],
          ),
        ),
        card(
          pick(bi("Kinematika balik", "Inverse kinematics")),
          inverse
            ? table(
                [
                  pick(bi("Penyelesaian", "Solution")),
                  pick(bi("Sudut pangkal", "Base angle")),
                  pick(bi("Sudut siku", "Elbow angle")),
                ],
                inverse.map((s, i) => [
                  i === 0 ? pick(bi("siku bawah", "elbow down")) : pick(bi("siku atas", "elbow up")),
                  `${fmt((s.theta1 * 180) / Math.PI, 1)}°`,
                  `${fmt((s.theta2 * 180) / Math.PI, 1)}°`,
                ]),
              )
            : errorNote(pick(bi("Titik di luar jangkauan.", "Point out of reach."))),
          el("p", {
            class: "note",
            text: pick(
              bi(
                "Satu titik hampir selalu punya dua penyelesaian: siku menekuk ke atas atau ke bawah. Keduanya sah, dan memilih salah satunya adalah keputusan perancang — biasanya yang paling sedikit menggerakkan sendi.",
                "One point almost always has two solutions: elbow up or elbow down. Both are valid, and choosing between them is a design decision — usually whichever moves the joints least.",
              ),
            ),
          }),
        ),
      );
    }

    function renderPath(): void {
      const obstacles = [{ x: 5, y: obstacleY, radius: 2 }];
      let path: engine.PotentialPath;
      try {
        path = engine.roboticsPath(9, 0, obstacles, repulsive, 500);
      } catch (error) {
        output.append(errorNote(String((error as Error).message)));
        return;
      }
      drawPath(path, obstacleY);

      output.append(
        card(
          pick(T.result),
          table(
            [pick(bi("Ukuran", "Measure")), pick(bi("Nilai", "Value"))],
            [
              [
                pick(bi("Tujuan tercapai", "Goal reached")),
                path.reached ? pick(bi("ya", "yes")) : pick(bi("tidak", "no")),
              ],
              [
                pick(bi("Terjebak minimum lokal", "Stuck in local minimum")),
                path.stuck_in_local_minimum
                  ? pick(bi("ya", "yes"))
                  : pick(bi("tidak", "no")),
              ],
              [pick(bi("Panjang lintasan", "Path length")), path.length],
              [pick(bi("Titik lintasan", "Path points")), String(path.points.length)],
            ],
          ),
          el("p", {
            class: "note",
            text: path.stuck_in_local_minimum
              ? pick(
                  bi(
                    "Robot berhenti walau tujuannya terlihat jelas. Gaya tarik dan gaya tolak saling meniadakan tepat di titik itu. Ini bukan bug melainkan cacat bawaan medan potensial — geser rintangannya sedikit ke atas atau ke bawah dan lintasannya langsung ketemu.",
                    "The robot stops even though the goal is in plain sight. Attraction and repulsion cancel exactly there. This is not a bug but the built-in flaw of potential fields — nudge the obstacle up or down and a path appears immediately.",
                  ),
                )
              : pick(
                  bi(
                    "Geser rintangan ke y = 0 tepat di garis antara robot dan tujuan, lalu naikkan gaya tolaknya. Robot akan berhenti di depan rintangan — cacat bawaan metode ini, bukan kesalahan penyetelan.",
                    "Move the obstacle to y = 0, exactly on the line between robot and goal, then raise the repulsive gain. The robot will stall in front of it — the built-in flaw of this method, not a tuning mistake.",
                  ),
                ),
          }),
        ),
      );
    }

    function render(): void {
      clear(output);
      switch (tab) {
        case "pid":
          renderPid();
          break;
        case "arm":
          renderArm();
          break;
        case "path":
          renderPath();
          break;
      }
    }

    function renderControls(): void {
      clear(controls);

      const tabs = buttonRow(
        (
          [
            ["pid", bi("Kendali PID", "PID control")],
            ["arm", bi("Lengan dua sendi", "Two-joint arm")],
            ["path", bi("Medan potensial", "Potential field")],
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

      if (tab === "pid") {
        extras.push(
          card(
            pick(T.controls),
            slider({
              label: "Kp",
              min: 0.1,
              max: 40,
              step: 0.1,
              value: kp,
              format: (v) => fmt(v, 1),
              onInput: (v) => {
                kp = v;
                render();
              },
            }),
            slider({
              label: "Ki",
              min: 0,
              max: 5,
              step: 0.05,
              value: ki,
              format: (v) => fmt(v, 2),
              onInput: (v) => {
                ki = v;
                render();
              },
            }),
            slider({
              label: "Kd",
              min: 0,
              max: 3,
              step: 0.05,
              value: kd,
              format: (v) => fmt(v, 2),
              onInput: (v) => {
                kd = v;
                render();
              },
            }),
            slider({
              label: pick(bi("Sasaran", "Setpoint")),
              min: 1,
              max: 20,
              step: 1,
              value: setpoint,
              format: (v) => String(v),
              onInput: (v) => {
                setpoint = v;
                render();
              },
            }),
          ),
        );
      }

      if (tab === "arm") {
        extras.push(
          card(
            pick(T.controls),
            slider({
              label: pick(bi("Sudut pangkal", "Base angle")),
              min: -3.14,
              max: 3.14,
              step: 0.02,
              value: theta1,
              format: (v) => `${fmt((v * 180) / Math.PI, 0)}°`,
              onInput: (v) => {
                theta1 = v;
                render();
              },
            }),
            slider({
              label: pick(bi("Sudut siku", "Elbow angle")),
              min: -3.14,
              max: 3.14,
              step: 0.02,
              value: theta2,
              format: (v) => `${fmt((v * 180) / Math.PI, 0)}°`,
              onInput: (v) => {
                theta2 = v;
                render();
              },
            }),
            slider({
              label: pick(bi("Panjang lengan 1", "Link 1 length")),
              min: 0.5,
              max: 3,
              step: 0.1,
              value: length1,
              format: (v) => fmt(v, 1),
              onInput: (v) => {
                length1 = v;
                render();
              },
            }),
            slider({
              label: pick(bi("Panjang lengan 2", "Link 2 length")),
              min: 0.5,
              max: 3,
              step: 0.1,
              value: length2,
              format: (v) => fmt(v, 1),
              onInput: (v) => {
                length2 = v;
                render();
              },
            }),
          ),
        );
      }

      if (tab === "path") {
        extras.push(
          card(
            pick(T.controls),
            slider({
              label: pick(bi("Posisi tegak rintangan", "Obstacle vertical position")),
              min: -2,
              max: 2,
              step: 0.05,
              value: obstacleY,
              format: (v) => fmt(v, 2),
              onInput: (v) => {
                obstacleY = v;
                render();
              },
            }),
            slider({
              label: pick(bi("Gaya tolak", "Repulsive gain")),
              min: 0.2,
              max: 8,
              step: 0.1,
              value: repulsive,
              format: (v) => fmt(v, 1),
              onInput: (v) => {
                repulsive = v;
                render();
              },
            }),
            buttonRow([
              {
                label: pick(bi("Susun jebakan", "Set the trap")),
                onClick: () => {
                  obstacleY = 0;
                  repulsive = 5;
                  renderControls();
                  render();
                },
              },
            ]),
          ),
        );
      }

      controls.append(card(pick(bi("Bagian", "Section")), tabs), ...extras);
    }

    root.append(
      el("div", {
        class: "grid-2",
        children: [controls, card(pick(bi("Peragaan", "View")), canvas)],
      }),
      output,
    );
    renderControls();
    render();

    const observer = new MutationObserver(() => render());
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["data-theme"],
    });

    return () => {
      observer.disconnect();
      clear(root);
    };
}
