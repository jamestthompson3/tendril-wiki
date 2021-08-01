const CACHE_NAME = "%VERSION%";
const PAGES = [
  "./style.css",
  "./userstyles.css",
  "./mobile.css",
  "./index.html",
  "./script.js",
];

// install pages
self.addEventListener("install", installWorker);

async function installWorker(e) {
  await self.skipWaiting();
}

self.addEventListener("activate", activateServiceWorker);

async function activateServiceWorker(event) {
  await deleteOldCaches();
  await installCachedFiles();
  event.waitUntil(clients.claim()); // make the current sw the active sw in all cached pages
}

async function installCachedFiles() {
  const cache = await caches.open(CACHE_NAME);
  return cache.addAll(PAGES);
}

async function deleteOldCaches() {
  const keys = await caches.keys();
  const oldVersions = keys.filter((name) => {
    name !== CACHE_NAME;
  });
  return Promise.all(oldVersions.map((key) => caches.delete(key)));
}

self.addEventListener("fetch", (event) => {
  if (
    event.request.mode === "navigate" ||
    event.request.destination === "style" ||
    event.request.destination === "script" ||
    event.request.destination === "image"
  ) {
    event.respondWith(cacheResponse(event.request, event));
  }
});

async function cacheResponse(request, event) {
  const cache = await caches.open(CACHE_NAME);
  // Create promises for both the network response,
  // and a copy of the response that can be used in the cache.
  try {
    const fetchResponseP = fetch(request);
    const fetchResponseCloneP = fetchResponseP.then((r) => r.clone());

    event.waitUntil(
      (async function () {
        await cache.put(request, await fetchResponseCloneP);
      })()
    );
    return fetchResponseP;
  } catch (e) {
    const match = await cache.match(request.url);
    if (match) {
      return match;
    }
  }
}
