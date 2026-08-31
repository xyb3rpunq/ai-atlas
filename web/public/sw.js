/**
 * Pekerja layanan AI ATLAS.
 *
 * Situs ini tidak punya peladen: seluruh perhitungannya terjadi di peramban.
 * Karena itu, sekali asetnya tersimpan, laboratoriumnya bisa dipakai penuh
 * tanpa jaringan sama sekali — di kereta, di ruang kuliah dengan wifi buruk,
 * atau di mana pun kuotanya habis.
 *
 * Strategi:
 *   - Navigasi  → jaringan dulu, singgahan sebagai cadangan. Pengguna selalu
 *                 mendapat versi terbaru bila daring, tetapi tetap bisa membuka
 *                 situs saat luring.
 *   - Aset ber-hash (`/assets/...`) → singgahan dulu. Namanya sudah memuat
 *                 hash isi, jadi berkas yang sama tidak pernah berubah.
 *   - Lainnya   → jaringan dulu.
 *
 * .Deckyx
 */

const VERSION = "v1";
const CACHE = `ai-atlas-${VERSION}`;

/** Berkas yang disimpan saat pemasangan agar kunjungan luring pertama berhasil. */
const PRECACHE = ["./", "./index.html", "./manifest.webmanifest", "./icon.svg"];

self.addEventListener("install", (event) => {
  event.waitUntil(
    (async () => {
      const cache = await caches.open(CACHE);
      // `addAll` gagal seluruhnya bila satu berkas meleset, jadi tiap berkas
      // disimpan sendiri-sendiri dan kegagalan satu tidak membatalkan sisanya.
      await Promise.all(
        PRECACHE.map(async (url) => {
          try {
            await cache.add(new Request(url, { cache: "reload" }));
          } catch {
            /* Berkas ini akan diambil saat dibutuhkan. */
          }
        }),
      );
      await self.skipWaiting();
    })(),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    (async () => {
      const names = await caches.keys();
      await Promise.all(
        names.filter((n) => n.startsWith("ai-atlas-") && n !== CACHE).map((n) => caches.delete(n)),
      );
      await self.clients.claim();
    })(),
  );
});

self.addEventListener("fetch", (event) => {
  const request = event.request;

  // Hanya permintaan GET yang aman disinggahkan.
  if (request.method !== "GET") return;

  const url = new URL(request.url);
  // Permintaan lintas asal dilewatkan apa adanya.
  if (url.origin !== self.location.origin) return;

  if (request.mode === "navigate") {
    event.respondWith(
      (async () => {
        try {
          const fresh = await fetch(request);
          const cache = await caches.open(CACHE);
          cache.put(request, fresh.clone());
          return fresh;
        } catch {
          const cached = await caches.match(request);
          if (cached) return cached;
          const fallback = await caches.match("./index.html");
          if (fallback) return fallback;
          return new Response("Luring dan belum ada salinan tersimpan.", {
            status: 503,
            headers: { "Content-Type": "text/plain; charset=utf-8" },
          });
        }
      })(),
    );
    return;
  }

  const immutable = url.pathname.includes("/assets/");
  event.respondWith(
    (async () => {
      if (immutable) {
        const cached = await caches.match(request);
        if (cached) return cached;
      }
      try {
        const fresh = await fetch(request);
        if (fresh.ok && fresh.type === "basic") {
          const cache = await caches.open(CACHE);
          cache.put(request, fresh.clone());
        }
        return fresh;
      } catch {
        const cached = await caches.match(request);
        if (cached) return cached;
        throw new Error(`gagal mengambil ${url.pathname}`);
      }
    })(),
  );
});
